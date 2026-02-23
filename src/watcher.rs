use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::PathBuf;
use std::sync::mpsc::channel;
use std::time::Duration;

use crate::fix::{process_markdown, TemplateEngine};

pub fn watch(folder: PathBuf, engine: TemplateEngine) -> notify::Result<()> {
    let (tx, rx) = channel();

    let mut watcher = RecommendedWatcher::new(tx, Config::default())?;
    watcher.watch(&folder, RecursiveMode::Recursive)?;

    println!("Watching {:?}", folder);

    loop {
        match rx.recv_timeout(Duration::from_millis(300)) {
            Ok(Ok(Event { paths, .. })) => {
                for path in paths {
                    if path.extension().map(|x| x == "md").unwrap_or(false) {
                        if let Err(e) = process_markdown(&path, &engine) {
                            eprintln!("process_markdown error ({}): {e}", path.display());
                        }
                    }
                }
            }
            Ok(Err(e)) => {
                eprintln!("watch error: {e}");
            }
            Err(_) => {
                // timeout
            }
        }
    }

}
