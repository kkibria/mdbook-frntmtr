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
            Ok(Event { paths, .. }) => {
                for path in paths {
                    if path.extension().map(|x| x == "md").unwrap_or(false) {
                        let _ = process_markdown(&path, &engine);
                    }
                }
            }
            Err(_) => {}
        }
    }
}
