use clap::{Arg, Command};
use mdbook_preprocessor::errors::{Error, Result};
use mdbook_preprocessor::parse_input;
use mdbook_preprocessor::Preprocessor;
use std::io;
use std::path::Path;
use std::process;

use mdbook_frntmtr::Frntmtr;

mod config;
mod fix;
mod watcher;

fn make_app() -> Command {
    Command::new("mdbook-frntmtr")
        .about("mdBook frontmatter preprocessor + utilities")
        .subcommand(
            Command::new("supports")
                .about("Check whether a renderer is supported by this preprocessor")
                .arg(Arg::new("renderer").required(true)),
        )
        .subcommand(
            Command::new("serve")
                .about("Run mdbook serve + watch book.src to auto-inject frontmatter")
                // Everything after `--` goes to mdbook:
                // mdbook-frntmtr serve -- --open -p 3000
                .trailing_var_arg(true)
                .arg(Arg::new("mdbook_args").num_args(0..).last(true)),
        )
}

fn main() {
    let matches = make_app().get_matches();
    let preprocessor = Frntmtr::new();

    match matches.subcommand() {
        Some(("supports", sub)) => handle_supports(&preprocessor, sub),

        Some(("serve", sub)) => {
            let mdbook_args: Vec<String> = sub
                .get_many::<String>("mdbook_args")
                .map(|vals| vals.map(|s| s.to_string()).collect())
                .unwrap_or_else(Vec::new);

            if let Err(e) = run_serve(mdbook_args) {
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

fn handle_supports(pre: &dyn Preprocessor, sub: &clap::ArgMatches) -> ! {
    let renderer = sub.get_one::<String>("renderer").expect("Required argument");
    let supported = match pre.supports_renderer(renderer) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("{e}");
            process::exit(1);
        }
    };
    process::exit(if supported { 0 } else { 1 });
}

fn handle_preprocessing(pre: &dyn Preprocessor) -> Result<()> {
    let (ctx, book) = parse_input(io::stdin())?;
    let processed = pre.run(&ctx, book)?;
    serde_json::to_writer(io::stdout(), &processed)
        .map_err(|e| Error::msg(format!("Write JSON error: {e}")))?;
    Ok(())
}

fn run_serve(mdbook_args: Vec<String>) -> Result<()> {
    let sc = config::load(Path::new("book.toml"))?;
    let engine = fix::TemplateEngine::new(&sc.template_path)?;

    {
        let src_dir = sc.src_dir.clone();
        std::thread::spawn(move || {
            if let Err(e) = watcher::watch(src_dir, engine) {
                eprintln!("watcher error: {e}");
            }
        });
    }

    let mdbook = which::which("mdbook")
        .map_err(|e| Error::msg(format!("Could not find `mdbook` on PATH: {e}")))?;

    eprintln!("book_root: {}", sc.book_root.display());
    eprintln!("book_root is_dir: {}", sc.book_root.is_dir());
    eprintln!("src_dir: {}", sc.src_dir.display());
    eprintln!("src_dir is_dir: {}", sc.src_dir.is_dir());
    eprintln!("template_path: {}", sc.template_path.display());
    eprintln!("template is_file: {}", sc.template_path.is_file());
    eprintln!("mdbook path: {}", mdbook.display());
    eprintln!("mdbook exists: {}", mdbook.exists());
    eprintln!("mdbook args: {:?}", mdbook_args);

    if !sc.book_root.is_dir() {
        return Err(Error::msg(format!(
            "book_root is not a directory: {}",
            sc.book_root.display()
        )));
    }

    let status = process::Command::new(&mdbook)
        .current_dir(&sc.book_root)
        .arg("serve")
        .args(&mdbook_args)
        .status()
        .map_err(|e| {
            Error::msg(format!(
                "failed to start mdbook serve (cwd={} exe={}): {e}",
                sc.book_root.display(),
                mdbook.display()
            ))
        })?;

    process::exit(status.code().unwrap_or(1));
}