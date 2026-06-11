//! File watcher for real-time log monitoring.

use crate::domain::types::SourceId;
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver};

#[derive(Debug, Clone)]
pub enum WatchEvent {
    Modified(SourceId),
    Rotated(SourceId),
    Error(String),
}

pub struct FileWatcher {
    _watcher: RecommendedWatcher,
    _receiver: Receiver<WatchEvent>,
    _watched: HashMap<PathBuf, SourceId>,
}

impl FileWatcher {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let (tx, rx) = mpsc::channel();
        let watcher = RecommendedWatcher::new(
            move |result: Result<Event, notify::Error>| {
                if let Ok(event) = result {
                    match event.kind {
                        EventKind::Modify(_) => { let _ = tx.send(WatchEvent::Error("modified".into())); }
                        _ => {}
                    }
                }
            },
            Config::default(),
        )?;
        Ok(Self { _watcher: watcher, _receiver: rx, _watched: HashMap::new() })
    }

    pub fn watch(&mut self, path: &std::path::Path, source_id: SourceId) -> Result<(), Box<dyn std::error::Error>> {
        self._watcher.watch(path, RecursiveMode::NonRecursive)?;
        self._watched.insert(path.to_path_buf(), source_id);
        Ok(())
    }

    pub fn unwatch(&mut self, path: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
        self._watcher.unwatch(path)?;
        self._watched.remove(path);
        Ok(())
    }

    pub fn poll_event(&self) -> Option<WatchEvent> {
        self._receiver.try_recv().ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_creation() { assert!(FileWatcher::new().is_ok()); }

    #[test]
    fn test_watch_unwatch() {
        let mut w = FileWatcher::new().unwrap();
        let f = NamedTempFile::new().unwrap();
        w.watch(f.path(), SourceId::from_path(f.path())).unwrap();
        w.unwatch(f.path()).unwrap();
    }
}
