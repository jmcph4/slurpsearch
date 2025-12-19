use std::path::PathBuf;

use clap::Parser;

#[derive(Clone, Debug, Parser)]
pub struct Opts {
    /// Path to file to search
    pub haystack: PathBuf,
    /// Search prompt
    pub prompt: String,
}
