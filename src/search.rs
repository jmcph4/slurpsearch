use rayon::prelude::*;
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
    pub url: Url,
    pub position: TextPosition,
}

impl Display for Finding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Found hit for \"{}\" in {} on {}",
            self.search, self.url, self.position
        )
    }
}

const TRIM: &[char] = &[')', ']', '}', '.', ',', ';', ':', '"', '\'', '>', ' '];

pub fn extract_urls(s: &str) -> HashSet<Url> {
    let re = Regex::new(r#"https?://[A-Za-z0-9\-._~:/?#\[\]@!$&'()*+,;=%]+"#)
        .unwrap();

    re.find_iter(s)
        .map(|m| m.as_str().trim_end_matches(TRIM))
        .filter_map(|m| Url::parse(m).ok())
        .collect()
}

pub fn search(haystack: &[(Url, String)], needle: &str) -> Vec<Finding> {
    if needle.is_empty() {
        return Vec::new();
    }

    haystack
        .par_iter()
        .flat_map_iter(|(url, text)| {
            text.lines().enumerate().flat_map(move |(line_idx, line)| {
                let mut out = Vec::new();
                let mut search_start = 0;

                while let Some(pos) = line[search_start..].find(needle) {
                    let column = search_start + pos;

                    out.push(Finding {
                        search: needle.to_string(),
                        url: url.clone(),
                        position: TextPosition {
                            line: line_idx + 1,
                            column: column + 1,
                        },
                    });

                    search_start = column + needle.len();
                    if search_start > line.len() {
                        break;
                    }
                }

                out
            })
        })
        .collect()
}
