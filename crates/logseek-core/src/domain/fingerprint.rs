//! File fingerprint for cache validation.

use serde::{Deserialize, Serialize};
use std::path::Path;
use std::time::UNIX_EPOCH;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileFingerprint {
    pub file_size: u64,
    pub modified_time: u64,
}

impl FileFingerprint {
    pub fn from_path(path: &Path) -> Result<Self, std::io::Error> {
        let metadata = std::fs::metadata(path)?;
        let modified = metadata.modified().unwrap_or(UNIX_EPOCH)
            .duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
        Ok(Self { file_size: metadata.len(), modified_time: modified })
    }

    pub fn has_changed(&self, path: &Path) -> bool {
        Self::from_path(path).map_or(true, |current| current != *self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_fingerprint_from_path() {
        let mut f = NamedTempFile::new().unwrap();
        write!(f, "content").unwrap();
        let fp = FileFingerprint::from_path(f.path()).unwrap();
        assert!(fp.file_size > 0);
    }

    #[test]
    fn test_fingerprint_equality() {
        let a = FileFingerprint { file_size: 100, modified_time: 1000 };
        let b = FileFingerprint { file_size: 100, modified_time: 1000 };
        assert_eq!(a, b);
    }

    #[test]
    fn test_fingerprint_change_detection() {
        let mut f = NamedTempFile::new().unwrap();
        write!(f, "initial").unwrap();
        let fp = FileFingerprint::from_path(f.path()).unwrap();
        assert!(!fp.has_changed(f.path()));
        write!(f, "changed").unwrap();
        assert!(fp.has_changed(f.path()));
    }
}
