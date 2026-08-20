use notify::RecursiveMode;
use notify_debouncer_mini::new_debouncer;
use std::path::Path;
use std::sync::mpsc::channel;
use std::time::Duration;

use bloom_logger;

pub fn watch(path: impl Into<String>, restart: impl Fn()) -> notify::Result<()> {
    let (tx, rx) = channel();

    let mut watcher = new_debouncer(Duration::from_millis(1000), move |result| {
        tx.send(result).expect("Failed to send event");
    })?;

    let path = path.into();
    let path = Path::new(&path);
    watcher
        .watcher()
        .watch(path, RecursiveMode::Recursive)
        .map_err(|e| {
            bloom_logger::log(
                bloom_logger::DEV_STREAM,
                format!("Failed to watch {:?}: {e}", path),
            );
            e
        })?;

    bloom_logger::log(
        bloom_logger::DEV_STREAM,
        format!("Watching {:?} for changes...", path),
    );

    loop {
        match rx.recv_timeout(Duration::from_secs(1)) {
            Ok(Ok(_)) => {
                restart();

                bloom_logger::log(bloom_logger::DEV_STREAM, "Restarted Process");
            }

            Ok(Err(e)) => {
                eprintln!("Watch error: {:?}", e);
            }

            Err(_) => {
                // no edits occured
            }
        }
    }
}
