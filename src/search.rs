use crate::rag::{SearchResult, WebDoc};
use regex::Regex;
use serde::Serialize;
use std::{collections::HashSet, fmt::Display};
use url::Url;

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

#[derive(Clone, Debug, Serialize)]
pub struct Finding {
    pub search: String,
    pub relevance: f64,
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

impl From<SearchResult> for Finding {
    fn from(value: SearchResult) -> Self {
        todo!()
    }
}

const TRIM: &[char] = &[')', ']', '}', '.', ',', ';', ':', '"', '\'', '>', ' '];
const URL_EXTRACTION_REGEX: &str =
    r#"https?://[A-Za-z0-9\-._~:/?#\[\]@!$&'()*+,;=%]+"#;

pub fn extract_urls(s: &str) -> HashSet<Url> {
    let re = Regex::new(URL_EXTRACTION_REGEX).unwrap();

    re.find_iter(s)
        .map(|m| m.as_str().trim_end_matches(TRIM))
        .filter_map(|m| Url::parse(m).ok())
        .collect()
}
