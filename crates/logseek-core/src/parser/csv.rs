//! CSV parser.

use crate::domain::types::{ByteRange, FieldValue, LogRecord, SourceId};
use crate::parser::trait_def::LogParser;
use crate::utils::helpers::bytes_to_string;
use memchr::memchr;

pub struct CsvParser { source_id: SourceId, headers: Vec<String> }

impl CsvParser {
    pub fn new(source_id: SourceId) -> Self { Self { source_id, headers: vec![] } }
    pub fn with_headers(source_id: SourceId, headers: Vec<String>) -> Self { Self { source_id, headers } }
}

impl LogParser for CsvParser {
    fn parse_record(&self, mmap: &[u8], offset: u64) -> Option<LogRecord> {
        let start = offset as usize;
        if start >= mmap.len() { return None; }
        let end = memchr(b'\n', &mmap[start..]).map(|p| start + p + 1).unwrap_or(mmap.len());
        let raw = bytes_to_string(&mmap[start..end]).trim_end().to_string();
        let values = parse_csv_line(&raw);
        let fields = self.headers.iter().zip(values.iter()).map(|(k, v)| FieldValue { key: k.clone(), value: v.clone() }).collect();
        Some(LogRecord { source_id: self.source_id.clone(), raw, fields, byte_range: ByteRange { start: offset, end: end as u64 } })
    }

    fn next_record_offset(&self, mmap: &[u8], current: u64) -> Option<u64> {
        let pos = current as usize;
        if pos >= mmap.len() { return None; }
        memchr(b'\n', &mmap[pos..]).map(|o| current + o as u64 + 1)
    }

    fn field_names(&self) -> Vec<String> { self.headers.clone() }
}

fn parse_csv_line(line: &str) -> Vec<String> {
    let mut fields = vec![];
    let mut current = String::new();
    let mut in_quotes = false;
    let mut chars = line.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '"' => { if in_quotes { if chars.peek() == Some(&'"') { current.push('"'); chars.next(); } else { in_quotes = false; } } else { in_quotes = true; } }
            ',' if !in_quotes => { fields.push(current.trim().to_string()); current.clear(); }
            '\r' | '\n' => break,
            _ => current.push(c),
        }
    }
    fields.push(current.trim().to_string());
    fields
}

#[cfg(test)]
mod tests {
    use super::*;
    fn parser() -> CsvParser { CsvParser::with_headers(SourceId("t".into()), vec!["a".into(), "b".into()]) }

    #[test]
    fn test_simple() {
        let r = parser().parse_record(b"x,y\n", 0).unwrap();
        assert_eq!(r.fields[0].value, "x");
        assert_eq!(r.fields[1].value, "y");
    }

    #[test]
    fn test_quoted() {
        let r = parser().parse_record(b"\"hello, world\",y\n", 0).unwrap();
        assert_eq!(r.fields[0].value, "hello, world");
    }

    #[test]
    fn test_field_names() {
        assert_eq!(parser().field_names(), vec!["a", "b"]);
    }
}
