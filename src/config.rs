use mdbook_preprocessor::errors::{Error, Result};
use serde::Deserialize;
use std::{
    fs,
    path::{Path, PathBuf},
};

#[derive(Debug, Deserialize, Default)]
struct BookToml {
    #[serde(default)]
    book: BookSection,
    #[serde(default)]
    preprocessor: Preprocessors,
}

#[derive(Debug, Deserialize, Default)]
struct BookSection {
    #[serde(default = "default_src")]
    src: String,
}

fn default_src() -> String {
    "src".to_string()
}

#[derive(Debug, Deserialize, Default)]
struct Preprocessors {
    #[serde(default)]
    frntmtr: FrntmtrConfig,
}

#[derive(Debug, Deserialize, Default)]
struct FrntmtrConfig {
    template: Option<String>,
}

pub struct ServeConfig {
    pub book_root: PathBuf,
    pub src_dir: PathBuf,
    pub template_path: PathBuf,
}

pub fn load(book_toml_path: &Path) -> Result<ServeConfig> {
    let raw = fs::read_to_string(book_toml_path)
        .map_err(|e| Error::msg(format!("Failed to read {}: {e}", book_toml_path.display())))?;

    let cfg: BookToml = toml::from_str(&raw)
        .map_err(|e| Error::msg(format!("Failed to parse {}: {e}", book_toml_path.display())))?;

    // book_root is the directory containing book.toml (not a config key)
    let mut book_root = book_toml_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .to_path_buf();

    // Defensive: avoid empty PathBuf edge cases
    if book_root.as_os_str().is_empty() {
        book_root = PathBuf::from(".");
    }

    // Canonicalize so `Command::current_dir(book_root)` cannot fail with ENOENT.
    let book_root = book_root.canonicalize().map_err(|e| {
        Error::msg(format!(
            "book_root does not exist ({}) : {e}",
            book_root.display()
        ))
    })?;

    let src_dir = book_root.join(&cfg.book.src);

    let template_rel = cfg
        .preprocessor
        .frntmtr
        .template
        .as_deref()
        .ok_or_else(|| Error::msg("Missing [preprocessor.frntmtr].template in book.toml"))?;

    let template_path = book_root.join(template_rel);

    Ok(ServeConfig {
        book_root,
        src_dir,
        template_path,
    })
}