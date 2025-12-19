use crate::{
    cli::Opts,
    extract::extract_text,
    fetch::*,
    rag::{RagStore, WebDoc},
    search::*,
};
use clap::Parser;
use std::fs;
use tracing::{error, info};
use url::Url;

pub mod cli;
pub mod extract;
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

    info!("Extracting text from webpages...");
    let docs: Vec<WebDoc> = successful
        .iter()
        .filter_map(|(url, html)| extract_text(url.clone(), html).ok())
        .flatten()
        .collect();
    info!("Text extraction complete");

    info!("Embedding {} documents...", docs.len());
    let rag = RagStore::try_from_documents(docs)
        .await
        .inspect_err(|e| error!("Failed to embed webpages: {e}"))?;
    info!("Embedded documents");
    info!("Commencing search...");
    let findings = rag
        .search(&opts.prompt, Some(DEFAULT_RELEVANCE_THRESHOLD))
        .await
        .inspect_err(|e| error!("Failed to prompt model: {e}"))?;
    info!("Found {} findings", findings.len());
    findings.iter().for_each(|x| println!("{x}"));

    Ok(())
}
