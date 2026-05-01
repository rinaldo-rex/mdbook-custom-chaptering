use anyhow::Result;
use mdbook_preprocessor::book::{Book, BookItem, SectionNumber};
use mdbook_preprocessor::{Preprocessor, PreprocessorContext};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::io;

pub struct CustomChaptering;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    /// Enable custom chapter numbering (default: true)
    #[serde(default)]
    pub enabled: Option<bool>,
}

// Regex for parsing: "26.5 Title" or "26.5.1 Title"
static PARSE_RE: once_cell::sync::Lazy<Regex> = once_cell::sync::Lazy::new(|| {
    Regex::new(r"^(\d+\.\d+(?:\.\d+)*)\s+(.+)$").unwrap()
});

impl CustomChaptering {
    pub fn new() -> Self {
        CustomChaptering
    }

    fn parse_custom_number(title: &str) -> Option<SectionNumber> {
        let caps = PARSE_RE.captures(title)?;
        let number_str = caps.get(1)?.as_str();
        
        let numbers: Vec<u32> = number_str
            .split('.')
            .filter_map(|s| s.parse().ok())
            .collect();
        
        if numbers.is_empty() {
            return None;
        }
        
        Some(SectionNumber::new(numbers))
    }

    fn process_chapter(item: &mut BookItem) {
        if let BookItem::Chapter(ref mut ch) = item {
            if let Some(number) = Self::parse_custom_number(&ch.name) {
                ch.number = Some(number);
                
                if let Some(caps) = PARSE_RE.captures(&ch.name) {
                    if let Some(title) = caps.get(2) {
                        ch.name = title.as_str().to_string();
                    }
                }
            } else {
                ch.number = None;
            }
        }
    }
}

impl Preprocessor for CustomChaptering {
    fn name(&self) -> &str {
        "custom-chaptering"
    }

    fn run(&self, ctx: &PreprocessorContext, book: Book) -> Result<Book> {
        // Check if enabled (default to true if not specified)
        let enabled = ctx
            .config
            .get::<toml::Value>("preprocessor.custom-chaptering")
            .ok()
            .flatten()
            .and_then(|v| v.get("enabled").and_then(|e| e.as_bool()))
            .unwrap_or(true);

        if !enabled {
            return Ok(book);
        }

        let mut book = book;
        book.for_each_mut(|item| Self::process_chapter(item));
        Ok(book)
    }

    fn supports_renderer(&self, _renderer: &str) -> Result<bool> {
        // Support all renderers - let mdbook handle filtering
        Ok(true)
    }
}

pub fn handle_preprocessing() -> Result<()> {
    let pre = CustomChaptering::new();
    let (ctx, book) = mdbook_preprocessor::parse_input(io::stdin())?;

    let processed_book = pre.run(&ctx, book)?;
    serde_json::to_writer(io::stdout(), &processed_book)?;


    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_custom_number() {
        assert_eq!(
            CustomChaptering::parse_custom_number("26.5 First chapter"),
            Some(SectionNumber::new(vec![26, 5]))
        );
        
        assert_eq!(
            CustomChaptering::parse_custom_number("26.5.1 Third chapter"),
            Some(SectionNumber::new(vec![26, 5, 1]))
        );
        
        assert_eq!(
            CustomChaptering::parse_custom_number("2026.5.1 Day within month"),
            Some(SectionNumber::new(vec![2026, 5, 1]))
        );
        
        assert_eq!(
            CustomChaptering::parse_custom_number("Just a title"),
            None
        );
        
        assert_eq!(
            CustomChaptering::parse_custom_number("26 First chapter"),
            None
        );
    }
}