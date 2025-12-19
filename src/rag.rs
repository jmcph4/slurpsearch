use std::collections::HashMap;
use std::fmt::Display;

use eyre::Result;
use rig::Embed;
use rig::agent::Agent;
use rig::client::CompletionClient;
use rig::completion::Prompt;
use rig::providers::openai::responses_api::ResponsesCompletionModel;
use rig::vector_store::in_memory_store::InMemoryVectorIndex;
use rig::{
    client::{EmbeddingsClient, ProviderClient},
    embeddings::EmbeddingsBuilder,
    providers::openai,
    vector_store::in_memory_store::InMemoryVectorStore,
};
use serde::Deserialize;
use serde::Serialize;
use tracing::debug;
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
#[derive(Clone, Debug, Serialize, PartialEq, Eq, rig::Embed)]
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
    pub documents: HashMap<DocumentId, WebDoc>,
}

impl RagStore {
    /// Build a [`RagStore`] from the provided documents
    ///
    /// Constructs [`WebDoc`]s from the provided (URL, contents) pairs, embeds
    /// them (via remote calls to [`EMBEDDING_MODEL`]), and inserts these
    /// embeddings into the vector store.
    pub async fn try_from_documents(docs: &[(Url, String)]) -> Result<Self> {
        let client = openai::Client::from_env();

        let documents: Vec<WebDoc> = docs
            .iter()
            .map(|(url, html)| WebDoc {
                url: url.clone(),
                text: html.clone(),
            })
            .collect();

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
            documents: documents
                .iter()
                .cloned()
                .enumerate()
                .map(|(i, doc)| (format!("doc{i}"), doc))
                .collect(),
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
            .dynamic_context(2, self.index())
            .build()
    }

    /// Search the document store
    ///
    /// Returns [`SearchResult`]s in descending order of relevance.
    pub async fn search(&self, query: &str) -> eyre::Result<Vec<Finding>> {
        // NOTE(jmcph4): actual web request flies out the door here
        let resp_text = self.agent().prompt(query).await?;
        debug!("Received completion response: {resp_text}");

        let mut results: Vec<SearchResult> = serde_json::from_str(&resp_text)?;
        results.sort_by_key(|x| x.relevance);
        results.reverse();

        Ok(results
            .iter()
            .map(|x| Finding {
                search: query.to_owned(),
                relevance: x.relevance as f64 / 100.0,
                doc: self.documents.get(&x.document_id).unwrap().clone(),
            })
            .collect())
    }
}
