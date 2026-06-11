//! Log parser trait definition.

use crate::domain::types::LogRecord;

pub trait LogParser: Send + Sync {
    fn parse_record(&self, mmap: &[u8], offset: u64) -> Option<LogRecord>;
    fn next_record_offset(&self, mmap: &[u8], current: u64) -> Option<u64>;
    fn field_names(&self) -> Vec<String>;
}
