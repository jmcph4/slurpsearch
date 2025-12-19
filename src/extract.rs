use eyre::Result;
use scraper::{ElementRef, Html, Selector};
use std::collections::HashSet;
use url::Url;

use crate::rag::WebDoc;

pub fn extract_text(url: Url, html: &str) -> Result<Vec<WebDoc>> {
    let document = Html::parse_document(html);

    // Single selector so we preserve DOM order.
    let block_sel =
        Selector::parse("p,li,blockquote,pre,code,h1,h2,h3,h4,h5,h6").unwrap();

    let mut out = Vec::new();
    let mut seen = HashSet::<String>::new();

    for node in document.select(&block_sel) {
        if is_boilerplate(node) {
            continue;
        }
        if is_nested_block(node) {
            continue;
        }

        let text = normalize_text(node.text());
        if text.is_empty() {
            continue;
        }

        // Dedup identical paragraphs (common with nested blocks / repeated chrome).
        if !seen.insert(text.clone()) {
            continue;
        }

        out.push(WebDoc {
            url: url.clone(),
            text,
        });
    }

    Ok(out)
}

fn is_nested_block(node: ElementRef<'_>) -> bool {
    // If an ancestor is also a "block" tag we extract, skip this node to avoid duplicates like:
    // <pre><code>...</code></pre>
    for anc in node.ancestors().skip(1) {
        if let Some(el) = ElementRef::wrap(anc)
            && is_block_tag(el.value().name())
        {
            return true;
        }
    }
    false
}

fn is_block_tag(tag: &str) -> bool {
    matches!(
        tag,
        "p" | "li"
            | "blockquote"
            | "pre"
            | "code"
            | "h1"
            | "h2"
            | "h3"
            | "h4"
            | "h5"
            | "h6"
    )
}

fn is_boilerplate(node: ElementRef<'_>) -> bool {
    // Drop anything inside obvious chrome containers or with obvious chrome-y attributes.
    for anc in node.ancestors() {
        if let Some(el) = ElementRef::wrap(anc) {
            let tag = el.value().name();

            if matches!(tag, "nav" | "header" | "footer" | "aside") {
                return true;
            }

            if let Some(role) = el.value().attr("role")
                && role.eq_ignore_ascii_case("navigation")
            {
                return true;
            }

            if let Some(v) = el.value().attr("aria-hidden")
                && v.eq_ignore_ascii_case("true")
            {
                return true;
            }

            if let Some(id) = el.value().attr("id")
                && looks_like_chrome(id)
            {
                return true;
            }

            if let Some(class) = el.value().attr("class")
                && looks_like_chrome(class)
            {
                return true;
            }
        }
    }
    false
}

fn looks_like_chrome(s: &str) -> bool {
    // Conservative substring heuristics. Lowercase + substring match.
    let s = s.to_ascii_lowercase();
    // Keep this short: false positives are costly.
    const BAD: [&str; 12] = [
        "nav",
        "navbar",
        "menu",
        "footer",
        "header",
        "sidebar",
        "breadcrumb",
        "breadcrumbs",
        "cookie",
        "consent",
        "subscribe",
        "newsletter",
    ];
    BAD.iter().any(|k| s.contains(k))
}

fn normalize_text<'a>(iter: impl Iterator<Item = &'a str>) -> String {
    let mut s = String::new();
    let mut last_was_space = false;

    for chunk in iter {
        for ch in chunk.chars() {
            if ch.is_whitespace() {
                if !last_was_space {
                    s.push(' ');
                    last_was_space = true;
                }
            } else {
                s.push(ch);
                last_was_space = false;
            }
        }
    }

    s.trim().to_string()
}
