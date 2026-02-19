use mdbook_preprocessor::errors::{Error, Result};
use minijinja::{Environment, context};
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub struct TemplateEngine {
    env: Environment<'static>,
    template_name: String,
}

impl TemplateEngine {
    pub fn new(template_path: &Path) -> Result<Self> {
        let mut env = Environment::new();

        let dir = template_path
            .parent()
            .ok_or_else(|| Error::msg("Template path has no parent directory"))?
            .to_path_buf();

        env.set_loader(minijinja::path_loader(dir));

        let name = template_path
            .file_name()
            .ok_or_else(|| Error::msg("Template path has no file name"))?
            .to_string_lossy()
            .to_string();

        Ok(Self {
            env,
            template_name: name,
        })
    }

    pub fn render(&self, title: &str) -> Result<String> {
        let tmpl = self
            .env
            .get_template(&self.template_name)
            .map_err(|e| Error::msg(format!("Template load error: {e}")))?;

        tmpl.render(context! {
            title => title,
            page => { title => title },
        })
        .map_err(|e| Error::msg(format!("Template render error: {e}")))
    }
}

pub fn process_markdown(path: &Path, engine: &TemplateEngine) -> Result<()> {
    let content = fs::read_to_string(path)
        .map_err(|e| Error::msg(format!("Read error {}: {e}", path.display())))?;

    let mut first_non_empty = None;
    let mut index = 0;

    for (i, line) in content.lines().enumerate() {
        if !line.trim().is_empty() {
            first_non_empty = Some(line.trim().to_string());
            index = i;
            break;
        }
    }

    let heading = match first_non_empty {
        Some(h) => h,
        None => return Ok(()),
    };

    if heading.contains("---") {
        return Ok(());
    }

    let title = heading.trim_start_matches('#').trim();

    let rendered = engine.render(title)?;

    let remaining: Vec<&str> = content.lines().skip(index + 1).collect();

    let mut new_content = String::new();
    new_content.push_str(&rendered);
    new_content.push('\n');

    if !remaining.is_empty() {
        new_content.push_str(&remaining.join("\n"));
    }

    println!("injecting frontmatter into {}", path.display());

    fs::write(path, new_content)
        .map_err(|e| Error::msg(format!("Write error {}: {e}", path.display())))?;

    Ok(())
}

pub fn process_tree(folder: &Path, engine: &TemplateEngine) -> Result<()> {
    for entry in WalkDir::new(folder)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|x| x == "md").unwrap_or(false))
    {
        process_markdown(entry.path(), engine)?;
    }
    Ok(())
}
