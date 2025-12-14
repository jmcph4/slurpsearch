use crate::{cli::Opts, fetch::*, search::*};
use clap::Parser;
use std::fs;
use tracing::info;
use url::Url;

pub mod cli;
pub mod fetch;
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

    info!("Commencing full-text search...");
    let search_results = search(&successful, &opts.needle);
    search_results
        .iter()
        .for_each(|finding| println!("{finding}"));

    Ok(())
}
