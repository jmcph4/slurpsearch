use std::path::PathBuf;

use clap::Parser;

#[derive(Clone, Debug, Parser)]
pub struct Opts {
    /// Path to file to search
    pub haystack: PathBuf,
    /// Search term (for exact full-text search)
    #[clap(short, long, action)]
    pub needle: Option<String>,
    /// Search prompt
    #[clap(short, long, action)]
    pub prompt: String,
}
