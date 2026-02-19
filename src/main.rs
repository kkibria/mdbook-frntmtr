use clap::{Arg, Command};
use mdbook_preprocessor::errors::{Error, Result};
use mdbook_preprocessor::parse_input;
use mdbook_preprocessor::Preprocessor;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process;
use walkdir::WalkDir;

use minijinja::{Environment, context};

use mdbook_frntmtr::Frntmtr;

fn make_app() -> Command {
    Command::new("mdbook-frntmtr")
        .about("mdBook frontmatter preprocessor + utilities")
        .subcommand(
            Command::new("supports")
                .arg(Arg::new("renderer").required(true)),
        )
        .subcommand(
            Command::new("fix")
                .about("Inject frontmatter into markdown files")
                .arg(Arg::new("folder").required(true))
                .arg(
                    Arg::new("template")
                        .long("template")
                        .required(true)
                        .value_name("FILE"),
                ),
        )
}

fn main() {
    let matches = make_app().get_matches();
    let preprocessor = Frntmtr::new();

    match matches.subcommand() {
        Some(("supports", sub)) => {
            let renderer = sub.get_one::<String>("renderer").unwrap();
            let ok = preprocessor.supports_renderer(renderer).unwrap_or(false);
            process::exit(if ok { 0 } else { 1 });
        }

        Some(("fix", sub)) => {
            let folder = sub.get_one::<String>("folder").unwrap();
            let template_path = sub.get_one::<String>("template").unwrap();
            if let Err(e) = handle_fix(folder, template_path) {
                eprintln!("{e}");
                process::exit(1);
            }
        }

        _ => {
            if let Err(e) = handle_preprocessing(&preprocessor) {
                eprintln!("{e}");
                process::exit(1);
            }
        }
    }
}

fn handle_fix(folder: &str, template_path: &str) -> Result<()> {
    let template_path = PathBuf::from(template_path);
    let (env, template_name) = load_template_env(&template_path)?;

    for entry in WalkDir::new(folder)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|x| x == "md").unwrap_or(false))
    {
        process_markdown(entry.path(), &env, &template_name)?;
    }

    Ok(())
}

fn load_template_env(template_path: &Path) -> Result<(Environment<'static>, String)> {
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

    Ok((env, name))
}

fn process_markdown(
    path: &Path,
    env: &Environment<'static>,
    template_name: &str,
) -> Result<()> {
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

    let tmpl = env
        .get_template(template_name)
        .map_err(|e| Error::msg(format!("Template load error: {e}")))?;

    let rendered = tmpl
        .render(context! {
            title => title,
            // page => { title => title },
        })
        .map_err(|e| Error::msg(format!("Template render error: {e}")))?;

    let remaining: Vec<&str> = content.lines().skip(index + 1).collect();

    let mut new_content = String::new();
    new_content.push_str(&rendered);
    new_content.push('\n');

    if !remaining.is_empty() {
        new_content.push_str(&remaining.join("\n"));
    }

    println!("adding frontmatter to {}", path.display());

    fs::write(path, new_content)
        .map_err(|e| Error::msg(format!("Write error {}: {e}", path.display())))?;

    Ok(())
}

fn handle_preprocessing(pre: &dyn Preprocessor) -> Result<()> {
    let (ctx, book) = parse_input(io::stdin())?;
    let processed = pre.run(&ctx, book)?;
    serde_json::to_writer(io::stdout(), &processed)
        .map_err(|e| Error::msg(format!("Write JSON error: {e}")))?;
    Ok(())
}
