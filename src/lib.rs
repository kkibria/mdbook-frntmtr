use std::collections::HashMap;

use mdbook_preprocessor::book::{Book, BookItem};
use mdbook_preprocessor::errors::Result;
use mdbook_preprocessor::{Preprocessor, PreprocessorContext};

use regex::Regex;

pub struct Frntmtr;

impl Frntmtr {
    pub fn new() -> Frntmtr {
        Frntmtr
    }
}

impl Preprocessor for Frntmtr {
    fn name(&self) -> &str {
        "frntmtr"
    }

    fn run(&self, _ctx: &PreprocessorContext, mut book: Book) -> Result<Book> {
        // A delimiter line that is exactly '---' with optional whitespace, in multiline mode.
        let delim_re = Regex::new(r"(?m)^---\s*$").unwrap();

        book.for_each_mut(|item: &mut BookItem| {
            if let BookItem::Chapter(ref mut chapter) = *item {
                let content = chapter.content.clone();

                // Split into: [before, frontmatter, after]
                let parts: Vec<&str> = delim_re.splitn(content.as_str(), 3).collect();
                if parts.len() != 3 {
                    return;
                }

                // Parse frontmatter lines: "key: value"
                let mut meta: HashMap<&str, &str> = HashMap::new();
                for line in parts[1].lines() {
                    let kv: Vec<&str> = line.splitn(2, ':').collect();
                    if kv.len() == 2 {
                        meta.insert(kv[0].trim(), kv[1].trim());
                    }
                }

                // Replace chapter contents with body (after frontmatter)
                chapter.content = parts[2].to_string();

                // Replace placeholders like: {{ page.title }}
                for (key, val) in meta.iter() {
                    let mut pat = r"\{\{\s*page\.".to_owned();
                    pat.push_str(&regex::escape(key));
                    pat.push_str(r"\s*\}\}");

                    let re = Regex::new(&pat).unwrap();
                    chapter.content = re.replace_all(&chapter.content, *val).to_string();
                }

                // Update Title if present
                if let Some(title) = meta.get("title") {
                    chapter.name = (*title).to_string();
                }
            }
        });

        Ok(book)
    }

    fn supports_renderer(&self, renderer: &str) -> Result<bool> {
        Ok(renderer == "html")
    }
}
