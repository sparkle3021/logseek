//! Text log parser.

use crate::domain::types::{ByteRange, LogRecord, SourceId};
use crate::parser::trait_def::LogParser;
use crate::utils::helpers::bytes_to_string;
use memchr::memchr;

pub struct TextParser { source_id: SourceId }

impl TextParser {
    pub fn new(source_id: SourceId) -> Self { Self { source_id } }
}

impl LogParser for TextParser {
    fn parse_record(&self, mmap: &[u8], offset: u64) -> Option<LogRecord> {
        let start = offset as usize;
        if start >= mmap.len() { return None; }
        let end = memchr(b'\n', &mmap[start..]).map(|p| start + p + 1).unwrap_or(mmap.len());
        let raw = bytes_to_string(&mmap[start..end]).trim_end().to_string();
        Some(LogRecord { source_id: self.source_id.clone(), raw, fields: vec![], byte_range: ByteRange { start: offset, end: end as u64 } })
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

    fn parser() -> TextParser { TextParser::new(SourceId("t".into())) }

    #[test]
    fn test_single_line() {
        let r = parser().parse_record(b"hello\n", 0).unwrap();
        assert_eq!(r.raw, "hello");
    }

    #[test]
    fn test_multiple_lines() {
        let p = parser();
        let d = b"a\nb\nc\n";
        assert_eq!(p.parse_record(d, 0).unwrap().raw, "a");
        let off = p.next_record_offset(d, 0).unwrap();
        assert_eq!(p.parse_record(d, off).unwrap().raw, "b");
    }

    #[test]
    fn test_empty() { assert!(parser().parse_record(b"", 0).is_none()); }

    #[test]
    fn test_no_trailing_newline() {
        assert_eq!(parser().parse_record(b"end", 0).unwrap().raw, "end");
    }
}
