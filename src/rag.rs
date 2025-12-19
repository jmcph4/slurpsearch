use crate::Finding;
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
use url::Url;

/// Name of the model to use for inference
const COMPLETION_MODEL: &str = "gpt-5.2";

/// Name of the model to use for text embeddings
const EMBEDDING_MODEL: &str = "text-embedding-3-large";

/// String to prefix query prompts with
const INSTRUCTIONS: &str = r#"Find the most relevant documents based on the following query. Respond only with valid JSON. Respond with a list of JSON objects of the form:

 - `document_id`: the unique identifier of the document
 - `relevance`: how relevant the document is to the query
 - `reason`: justification for why this is the case
"#;

#[derive(Clone, Debug, Serialize, PartialEq, Eq, rig::Embed)]
pub struct WebDoc {
    pub url: String,

    // The field tagged with #[embed] is what gets converted into text for embeddings.
    #[embed]
    pub text: String,
}

/// Represents a search result returned from the model
#[derive(Clone, Debug, Deserialize)]
pub struct SearchResult {}

#[derive(Clone)]
pub struct RagStore {
    pub client: openai::Client,
    pub store: InMemoryVectorStore<WebDoc>,
    pub model: openai::EmbeddingModel,
}

impl RagStore {
    pub async fn try_from_documents(docs: &[(Url, String)]) -> Result<Self> {
        let client = openai::Client::from_env();

        let documents: Vec<WebDoc> = docs
            .iter()
            .map(|(url, html)| WebDoc {
                url: url.to_string(),
                text: html.clone(),
            })
            .collect();

        let embedding_model = client.embedding_model(EMBEDDING_MODEL);
        // Any embedding model string that OpenAI supports works here
        let embeddings = EmbeddingsBuilder::new(embedding_model.clone())
            .documents(documents)?
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

    pub fn agent(&self) -> Agent<ResponsesCompletionModel> {
        self.client
            .agent(COMPLETION_MODEL)
            .preamble(INSTRUCTIONS)
            .dynamic_context(2, self.index())
            .build()
    }

    pub async fn search(&self, query: &str) -> eyre::Result<Vec<Finding>> {
        // NOTE(jmcph4): actual web request flies out the door here
        let resp_text = self.agent().prompt(query).await?;
        let json_text = resp_text
            .lines()
            .skip(1)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .skip(1)
            .rev()
            .collect::<Vec<_>>()
            .join("\n");
        let parsed: Vec<SearchResult> = serde_json::from_str(&json_text)?;
        Ok(parsed.iter().cloned().map(|x| x.into()).collect())
    }
}
