use crate::{cli::Opts, fetch::*, rag::RagStore, search::*};
use clap::Parser;
use rig::completion::Prompt;
use std::fs;
use tracing::{error, info};
use url::Url;

pub mod cli;
pub mod fetch;
pub mod rag;
pub mod search;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .init();
    let opts = Opts::parse();
    let contents = fs::read_to_string(&opts.haystack)?;
    let urls = extract_urls(contents.as_ref());
    info!(
        "Extracted {} URLs from {}",
        urls.len(),
        opts.haystack.display()
    );

    info!("Retrieving HTML...");
    let successful: Vec<(Url, String)> = fetch_all_html(urls, 32)
        .await?
        .into_iter()
        .filter_map(|(url, res)| res.ok().map(|html| (url, html)))
        .collect();
    info!("Retrieved {} webpages", successful.len());

    if successful.is_empty() {
        return Ok(());
    }

    if let Some(needle) = opts.needle {
        info!("Commencing full-text search...");
        let search_results = search(&successful, &needle);
        search_results
            .iter()
            .for_each(|finding| println!("{finding}"));
    } else {
        info!("Embedding webpages...");
        let rag = RagStore::try_from_documents(&successful)
            .await
            .inspect_err(|e| error!("Failed to embed webpages: {e}"))?;
        let resp = rag
            .agent()
            .prompt(opts.prompt)
            .await
            .inspect_err(|e| error!("Failed to prompt model: {e}"))?;
        println!("{resp}");
    }

    Ok(())
}
