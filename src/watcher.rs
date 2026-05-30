use crate::config::Config;
use crate::loader::ConfigLoader;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

pub struct ConfigWatcher;

impl ConfigWatcher {
    pub fn watch<F>(path: &str, callback: F) -> Result<(), String>
    where
        F: Fn(Config) + Send + 'static,
    {
        let path_owned = path.to_string();
        let handle = thread::spawn(move || {
            let mut last_modified = std::fs::metadata(&path_owned)
                .ok()
                .and_then(|m| m.modified().ok());

            loop {
                thread::sleep(Duration::from_secs(1));
                let current_modified = std::fs::metadata(&path_owned)
                    .ok()
                    .and_then(|m| m.modified().ok());

                if current_modified != last_modified {
                    last_modified = current_modified;
                    if let Ok(config) = ConfigLoader::from_file(&path_owned) {
                        callback(config);
                    }
                }
            }
        });

        Ok(())
    }

    pub fn watch_with_interval<F>(path: &str, interval_ms: u64, callback: F) -> Result<(), String>
    where
        F: Fn(Config) + Send + 'static,
    {
        let path_owned = path.to_string();
        let handle = thread::spawn(move || {
            let mut last_modified = std::fs::metadata(&path_owned)
                .ok()
                .and_then(|m| m.modified().ok());

            loop {
                thread::sleep(Duration::from_millis(interval_ms));
                let current_modified = std::fs::metadata(&path_owned)
                    .ok()
                    .and_then(|m| m.modified().ok());

                if current_modified != last_modified {
                    last_modified = current_modified;
                    if let Ok(config) = ConfigLoader::from_file(&path_owned) {
                        callback(config);
                    }
                }
            }
        });

        Ok(())
    }
}
