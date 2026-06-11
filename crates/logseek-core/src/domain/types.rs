//! Core domain types for log-seek.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Unique identifier for a log source.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SourceId(pub String);

/// Log file format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LogFormat {
    Text,
    Jsonl,
    Csv,
}

/// Byte range within a file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ByteRange {
    pub start: u64,
    pub end: u64,
}

/// A key-value field from structured logs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FieldValue {
    pub key: String,
    pub value: String,
}

/// A parsed log record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogRecord {
    pub source_id: SourceId,
    pub raw: String,
    pub fields: Vec<FieldValue>,
    pub byte_range: ByteRange,
}

/// A search hit result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchHit {
    pub source_id: SourceId,
    pub record_index: u64,
    pub byte_range: ByteRange,
}

impl LogFormat {
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "log" | "txt" | "out" => Some(Self::Text),
            "jsonl" | "ndjson" => Some(Self::Jsonl),
            "csv" => Some(Self::Csv),
            _ => None,
        }
    }
}

impl SourceId {
    pub fn from_path(path: &std::path::Path) -> Self {
        Self(path.to_string_lossy().to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logformat_from_extension() {
        assert_eq!(LogFormat::from_extension("log"), Some(LogFormat::Text));
        assert_eq!(LogFormat::from_extension("jsonl"), Some(LogFormat::Jsonl));
        assert_eq!(LogFormat::from_extension("csv"), Some(LogFormat::Csv));
        assert_eq!(LogFormat::from_extension("xyz"), None);
    }

    #[test]
    fn test_logformat_serde_roundtrip() {
        for format in [LogFormat::Text, LogFormat::Jsonl, LogFormat::Csv] {
            let json = serde_json::to_string(&format).unwrap();
            let de: LogFormat = serde_json::from_str(&json).unwrap();
            assert_eq!(format, de);
        }
    }

    #[test]
    fn test_byterange_serde_roundtrip() {
        let range = ByteRange { start: 100, end: 200 };
        let json = serde_json::to_string(&range).unwrap();
        let de: ByteRange = serde_json::from_str(&json).unwrap();
        assert_eq!(range, de);
    }

    #[test]
    fn test_logrecord_construction() {
        let record = LogRecord {
            source_id: SourceId("test".to_string()),
            raw: "test line".to_string(),
            fields: vec![FieldValue { key: "level".to_string(), value: "INFO".to_string() }],
            byte_range: ByteRange { start: 0, end: 9 },
        };
        assert_eq!(record.source_id.0, "test");
        assert_eq!(record.fields.len(), 1);
    }

    #[test]
    fn test_source_id_from_path() {
        let path = PathBuf::from("/var/log/test.log");
        let id = SourceId::from_path(&path);
        assert_eq!(id.0, "/var/log/test.log");
    }
}
