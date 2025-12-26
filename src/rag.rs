use std::fmt::Display;

use eyre::Result;
use rig::Embed;
use rig::agent::Agent;
use rig::client::CompletionClient;
use rig::providers::openai::responses_api::ResponsesCompletionModel;
use rig::vector_store::VectorStoreIndex;
use rig::vector_store::in_memory_store::InMemoryVectorIndex;
use rig::vector_store::request::VectorSearchRequestBuilder;
use rig::{
    client::{EmbeddingsClient, ProviderClient},
    embeddings::EmbeddingsBuilder,
    providers::openai,
    vector_store::in_memory_store::InMemoryVectorStore,
};
use serde::Deserialize;
use serde::Serialize;
use url::Url;

use crate::search::Finding;

/// Name of the model to use for inference
const COMPLETION_MODEL: &str = "gpt-5.2";

/// Name of the model to use for text embeddings
const EMBEDDING_MODEL: &str = "text-embedding-3-large";

/// String to prefix query prompts with
const INSTRUCTIONS: &str = r#"Find the most relevant documents based on the following query. Respond only with valid JSON. Respond with a list of JSON objects of the form:

 - `document_id`: the unique identifier of the document
 - `relevance`: how relevant the document is to the query (as an integer in the inclusive range 0-100)
 - `reason`: justification for why this is the case
"#;

/// Represents a document within the RAG system
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, rig::Embed)]
pub struct WebDoc {
    pub url: Url,
    #[embed]
    pub text: String,
}

/// Represents a search result returned from the model
#[derive(Clone, Debug, Deserialize)]
pub struct SearchResult {
    pub document_id: String,
    pub relevance: u64,
    pub reason: String,
}

impl Display for SearchResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}%", self.document_id, self.relevance)
    }
}

/// Uniquely identifies a given document for the purposes of embedding
pub type DocumentId = String;

#[derive(Clone)]
pub struct RagStore {
    pub client: openai::Client,
    pub store: InMemoryVectorStore<WebDoc>,
    pub model: openai::EmbeddingModel,
}

impl RagStore {
    /// Build a [`RagStore`] from the provided documents
    ///
    /// Constructs [`WebDoc`]s from the provided (URL, contents) pairs, embeds
    /// them (via remote calls to [`EMBEDDING_MODEL`]), and inserts these
    /// embeddings into the vector store.
    pub async fn try_from_documents(documents: Vec<WebDoc>) -> Result<Self> {
        let client = openai::Client::from_env();

        let embedding_model = client.embedding_model(EMBEDDING_MODEL);
        /* NOTE(jmcph4): actual request flies out the door here */
        let embeddings = EmbeddingsBuilder::new(embedding_model.clone())
            .documents(documents.clone())?
            .build()
            .await?;

        Ok(Self {
            client,
            store: InMemoryVectorStore::from_documents(embeddings),
            model: embedding_model,
        })
    }

    pub fn index(&self) -> InMemoryVectorIndex<openai::EmbeddingModel, WebDoc> {
        self.store.clone().index(self.model.clone())
    }

    /// Return a handle to the completion model
    pub fn agent(&self) -> Agent<ResponsesCompletionModel> {
        self.client
            .agent(COMPLETION_MODEL)
            .preamble(INSTRUCTIONS)
            .dynamic_context(self.store.len(), self.index())
            .build()
    }

    /// Search the document store
    ///
    /// Returns [`SearchResult`]s in descending order of relevance.
    pub async fn search(
        &self,
        query: &str,
        relevance_threshold: Option<f64>,
    ) -> eyre::Result<Vec<Finding>> {
        let search_request = VectorSearchRequestBuilder::default()
            .query(query)
            .samples(self.store.len() as u64);
        let results = self.index().top_n(search_request.build()?).await?;

        let mut findings: Vec<Finding> = results
            .iter()
            .cloned()
            .map(|(score, _, doc)| Finding {
                search: query.to_string(),
                relevance: score,
                doc,
            })
            .collect();
        findings.sort_by_key(|x| (x.relevance * 100.0) as u64);
        findings.reverse();
        Ok(findings)
    }
}
