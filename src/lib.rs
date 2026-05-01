use anyhow::Result;
use mdbook_preprocessor::book::{Book, BookItem, SectionNumber};
use mdbook_preprocessor::{Preprocessor, PreprocessorContext};
use serde::{Deserialize, Serialize};
use std::io;

pub struct CustomChaptering;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    /// Enable custom chapter numbering (default: true)
    #[serde(default)]
    pub enabled: Option<bool>,
}

impl CustomChaptering {
    pub fn new() -> Self {
        CustomChaptering
    }

    fn enabled(ctx: &PreprocessorContext) -> bool {
        ctx.config
            .get::<toml::Value>("preprocessor.custom-chaptering")
            .ok()
            .flatten()
            .and_then(|v| v.get("enabled").and_then(|e| e.as_bool()))
            .unwrap_or(true)
    }

    /// Parse the leading dot-notation token from a title.
    ///
    /// Examples:
    /// - `26.5 First chapter` -> `[26, 5]`, `First chapter`
    /// - `5. Some month` -> `[5]`, `Some month`
    fn parse_prefix(title: &str) -> Option<(Vec<u32>, String)> {
        let (token, rest) = title.split_once(char::is_whitespace)?;
        if !token.contains('.') {
            return None;
        }

        let numbers: Vec<u32> = token
            .trim_end_matches('.')
            .split('.')
            .filter(|s| !s.is_empty())
            .map(|s| s.parse::<u32>())
            .collect::<std::result::Result<Vec<_>, _>>()
            .ok()?;

        if numbers.is_empty() {
            return None;
        }

        Some((numbers, rest.trim_start().to_string()))
    }

    fn process_items(items: &mut [BookItem], parent_number: &[u32]) {
        let mut sibling_index: u32 = 1;

        for item in items.iter_mut() {
            let BookItem::Chapter(ch) = item else {
                continue;
            };

            let mut number = parent_number.to_vec();

            if let Some((mut prefix, stripped_name)) = Self::parse_prefix(&ch.name) {
                // Keep the hierarchy safe for mdBook's TOC renderer:
                // - top-level items may start with up to 2 components
                // - nested items only advance by one level
                if parent_number.is_empty() {
                    if prefix.len() > 2 {
                        prefix.truncate(2);
                    }
                    number.extend(prefix);
                } else {
                    number.push(*prefix.last().unwrap_or(&sibling_index));
                }

                ch.name = stripped_name;
            } else {
                number.push(sibling_index);
            }

            ch.number = Some(SectionNumber::new(number.clone()));
            sibling_index += 1;

            Self::process_items(&mut ch.sub_items, &number);
        }
    }
}

impl Preprocessor for CustomChaptering {
    fn name(&self) -> &str {
        "custom-chaptering"
    }

    fn run(&self, ctx: &PreprocessorContext, mut book: Book) -> Result<Book> {
        if !Self::enabled(ctx) {
            return Ok(book);
        }

        Self::process_items(&mut book.items, &[]);
        Ok(book)
    }

    fn supports_renderer(&self, _renderer: &str) -> Result<bool> {
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
    fn test_parse_prefix() {
        assert_eq!(
            CustomChaptering::parse_prefix("26.5 First chapter"),
            Some((vec![26, 5], "First chapter".to_string()))
        );
        assert_eq!(
            CustomChaptering::parse_prefix("5. Some month"),
            Some((vec![5], "Some month".to_string()))
        );
        assert_eq!(CustomChaptering::parse_prefix("Just a title"), None);
        assert_eq!(CustomChaptering::parse_prefix("26 First chapter"), None);
    }

    #[test]
    fn test_numbering_is_hierarchical() {
        use mdbook_preprocessor::book::Chapter;

        let mut book = Book::new_with_items(vec![BookItem::Chapter(Chapter {
            name: "26.5 First chapter".into(),
            content: String::new(),
            number: None,
            sub_items: vec![BookItem::Chapter(Chapter {
                name: "5. Some month".into(),
                content: String::new(),
                number: None,
                sub_items: vec![BookItem::Chapter(Chapter {
                    name: "1. Day within month".into(),
                    content: String::new(),
                    number: None,
                    sub_items: vec![],
                    path: None,
                    source_path: None,
                    parent_names: vec![],
                })],
                path: None,
                source_path: None,
                parent_names: vec![],
            })],
            path: None,
            source_path: None,
            parent_names: vec![],
        })]);

        CustomChaptering::process_items(&mut book.items, &[]);

        let BookItem::Chapter(root) = &book.items[0] else { panic!() };
        assert_eq!(root.number.as_ref().unwrap().to_string(), "26.5.");
        assert_eq!(root.name, "First chapter");

        let BookItem::Chapter(child) = &root.sub_items[0] else { panic!() };
        assert_eq!(child.number.as_ref().unwrap().to_string(), "26.5.5.");
        assert_eq!(child.name, "Some month");

        let BookItem::Chapter(grandchild) = &child.sub_items[0] else { panic!() };
        assert_eq!(grandchild.number.as_ref().unwrap().to_string(), "26.5.5.1.");
        assert_eq!(grandchild.name, "Day within month");
    }
}
