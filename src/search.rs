use crate::rag::WebDoc;
use regex::Regex;
use serde::Serialize;
use std::{collections::HashSet, fmt::Display};
use url::Url;

/// Default minimum relevance for a [`Finding`] to be returned to the end user
pub const DEFAULT_RELEVANCE_THRESHOLD: f64 = 0.60; /* 60% */

#[derive(Copy, Clone, Debug, Serialize)]
pub struct TextPosition {
    pub line: usize,
    pub column: usize,
}

impl Display for TextPosition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "line {} column {}", self.line, self.column)
    }
}

/// A search result that is presented to the end user
#[derive(Clone, Debug, Serialize)]
pub struct Finding {
    /// Query used to produce this finding
    pub search: String,
    /// How relevant this finding is as a percentage
    pub relevance: f64,
    /// The associated document
    pub doc: WebDoc,
}

impl Display for Finding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "URL: {}", self.doc.url)?;
        writeln!(f, "Text: {}", self.doc.text)?;
        writeln!(f, "Relevance: {}%", self.relevance * 100.0)?;
        Ok(())
    }
}

/// Set of characters to strip from URLs
const TRIM: &[char] = &[')', ']', '}', '.', ',', ';', ':', '"', '\'', '>', ' '];

/// Regular expression for extracting URLs from text
const URL_EXTRACTION_REGEX: &str =
    r#"https?://[A-Za-z0-9\-._~:/?#\[\]@!$&'()*+,;=%]+"#;

/// Return the set of all URLs from the provided string
///
/// Note that the URLs are not guaranteed to be of the HTTP nor HTTPS domain.
pub fn extract_urls(s: &str) -> HashSet<Url> {
    let re = Regex::new(URL_EXTRACTION_REGEX).unwrap();

    re.find_iter(s)
        .map(|m| m.as_str().trim_end_matches(TRIM))
        .filter_map(|m| Url::parse(m).ok())
        .collect()
}
