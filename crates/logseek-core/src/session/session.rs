//! Session management.

use crate::domain::types::LogFormat;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub version: u32,
    pub name: String,
    pub sources: Vec<SessionSource>,
    pub window: WindowState,
    pub search: SearchState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSource {
    pub path: PathBuf,
    pub format: LogFormat,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowState {
    pub width: u32,
    pub height: u32,
    pub maximized: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchState {
    pub keyword: String,
    pub regex: bool,
}

impl Session {
    pub fn load(path: &std::path::Path) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(serde_json::from_reader(BufReader::new(File::open(path)?))?)
    }

    pub fn save(&self, path: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
        Ok(serde_json::to_writer_pretty(BufWriter::new(File::create(path)?), self)?)
    }

    pub fn default_session() -> Self {
        Self {
            version: 1,
            name: "Default".into(),
            sources: vec![],
            window: WindowState { width: 1600, height: 900, maximized: false },
            search: SearchState { keyword: String::new(), regex: false },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_roundtrip() {
        let s = Session { version: 1, name: "Test".into(), sources: vec![SessionSource { path: "/a.log".into(), format: LogFormat::Text }], window: WindowState { width: 800, height: 600, maximized: true }, search: SearchState { keyword: "err".into(), regex: false } };
        let f = NamedTempFile::with_suffix(".olsession").unwrap();
        s.save(f.path()).unwrap();
        let loaded = Session::load(f.path()).unwrap();
        assert_eq!(loaded.name, "Test");
        assert_eq!(loaded.sources.len(), 1);
    }

    #[test]
    fn test_default() {
        let s = Session::default_session();
        assert_eq!(s.version, 1);
        assert!(s.sources.is_empty());
    }

    #[test]
    fn test_invalid_json() {
        let f = NamedTempFile::with_suffix(".olsession").unwrap();
        std::fs::write(f.path(), "bad").unwrap();
        assert!(Session::load(f.path()).is_err());
    }
}
