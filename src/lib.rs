use anyhow::Result;
use mdbook_preprocessor::book::{Book, BookItem, SectionNumber};
use mdbook_preprocessor::{Preprocessor, PreprocessorContext};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::io;

pub struct CustomChaptering;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    /// Enable custom chapter numbering
    #[serde(default = "default_true")]
    pub enabled: bool,
}

fn default_true() -> bool {
    true
}

// Regex for parsing: "26.5 Title" or "26.5.1 Title"
static PARSE_RE: once_cell::sync::Lazy<Regex> = once_cell::sync::Lazy::new(|| {
    Regex::new(r"^(\d+\.\d+(?:\.\d+)*)\s+(.+)$").unwrap()
});

impl CustomChaptering {
    pub fn new() -> Self {
        CustomChaptering
    }

    /// Parse custom number from SUMMARY.md format: [26.5 Title](./path.md)
    /// Returns the number as a SectionNumber if found
    fn parse_custom_number(title: &str) -> Option<SectionNumber> {
        let caps = PARSE_RE.captures(title)?;
        let number_str = caps.get(1)?.as_str();
        
        // Convert "26.5.1" -> vec![26, 5, 1]
        let numbers: Vec<u32> = number_str
            .split('.')
            .filter_map(|s| s.parse().ok())
            .collect();
        
        if numbers.is_empty() {
            return None;
        }
        
        Some(SectionNumber::new(numbers))
    }

    /// Process a single chapter item - apply custom number if found in title
    fn process_chapter(item: &mut BookItem) {
        if let BookItem::Chapter(ref mut ch) = item {
            // Try to parse custom number from chapter name
            if let Some(number) = Self::parse_custom_number(&ch.name) {
                // Override the chapter number with our custom one
                ch.number = Some(number);
                
                // Strip the number prefix from the name so we don't get duplicates
                // Pattern: "26.5 First chapter" -> "First chapter"
                if let Some(caps) = PARSE_RE.captures(&ch.name) {
                    if let Some(title) = caps.get(2) {
                        ch.name = title.as_str().to_string();
                    }
                }
            } else {
                // No custom number - remove mdbook's auto-numbering
                ch.number = None;
            }
        }
    }
}

impl Preprocessor for CustomChaptering {
    fn name(&self) -> &str {
        "custom-chaptering"
    }

    fn run(&self, ctx: &PreprocessorContext, mut book: Book) -> Result<Book> {
        // Check if enabled (default true if not specified)
        let config: Config = ctx
            .config
            .get("preprocessor.custom-chaptering")
            .unwrap_or(Some(Config { enabled: true }))
            .unwrap_or(Config { enabled: true });

        if !config.enabled {
            return Ok(book);
        }

        // Process all chapters
        book.for_each_mut(|item| Self::process_chapter(item));

        Ok(book)
    }

    fn supports_renderer(&self, renderer: &str) -> Result<bool> {
        // Support all standard renderers
        Ok(renderer == "html" || renderer == "pdf" || renderer == "latex" || renderer == "epub")
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
        
        // No number - should return None
        assert_eq!(
            CustomChaptering::parse_custom_number("Just a title"),
            None
        );
        
        // Number without dot continuation
        assert_eq!(
            CustomChaptering::parse_custom_number("26 First chapter"),
            None
        );
    }
}