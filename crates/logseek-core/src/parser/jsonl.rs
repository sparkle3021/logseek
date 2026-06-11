//! JSON Lines parser.

use crate::domain::types::{ByteRange, FieldValue, LogRecord, SourceId};
use crate::parser::trait_def::LogParser;
use crate::utils::helpers::bytes_to_string;
use memchr::memchr;
use serde_json::Value;

pub struct JsonlParser { source_id: SourceId }

impl JsonlParser {
    pub fn new(source_id: SourceId) -> Self { Self { source_id } }
}

impl LogParser for JsonlParser {
    fn parse_record(&self, mmap: &[u8], offset: u64) -> Option<LogRecord> {
        let start = offset as usize;
        if start >= mmap.len() { return None; }
        let end = memchr(b'\n', &mmap[start..]).map(|p| start + p + 1).unwrap_or(mmap.len());
        let raw = bytes_to_string(&mmap[start..end]).trim_end().to_string();
        let fields = serde_json::from_str::<Value>(&raw).ok()
            .and_then(|v| if let Value::Object(m) = v {
                Some(m.into_iter().map(|(k, v)| FieldValue { key: k, value: match v { Value::String(s) => s, o => o.to_string() } }).collect())
            } else { None }).unwrap_or_default();
        Some(LogRecord { source_id: self.source_id.clone(), raw, fields, byte_range: ByteRange { start: offset, end: end as u64 } })
    }

    fn next_record_offset(&self, mmap: &[u8], current: u64) -> Option<u64> {
        let pos = current as usize;
        if pos >= mmap.len() { return None; }
        memchr(b'\n', &mmap[pos..]).map(|o| current + o as u64 + 1)
    }

    fn field_names(&self) -> Vec<String> { vec![] }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn parser() -> JsonlParser { JsonlParser::new(SourceId("t".into())) }

    #[test]
    fn test_valid_object() {
        let r = parser().parse_record(b"{\"a\":\"1\"}\n", 0).unwrap();
        assert_eq!(r.fields.len(), 1);
        assert_eq!(r.fields[0].key, "a");
    }

    #[test]
    fn test_malformed_json() {
        let r = parser().parse_record(b"not json\n", 0).unwrap();
        assert!(r.fields.is_empty());
    }
}
