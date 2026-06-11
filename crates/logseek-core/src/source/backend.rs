//! Log backend with memory-mapped file access.

use crate::cache::index_cache;
use crate::domain::fingerprint::FileFingerprint;
use crate::domain::types::{LogFormat, LogRecord, SourceId};
use crate::parser::csv::CsvParser;
use crate::parser::jsonl::JsonlParser;
use crate::parser::text::TextParser;
use crate::parser::trait_def::LogParser;
use crate::source::index::{LineIndex, RecordIndex};
use crate::utils::helpers;
use memmap2::Mmap;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub struct LogBackend {
    path: PathBuf,
    mmap: Mmap,
    index: LineIndex,
    parser: Arc<dyn LogParser>,
    fingerprint: FileFingerprint,
}

impl LogBackend {
    pub fn open(path: &Path, format: LogFormat, source_id: SourceId) -> Result<Self, std::io::Error> {
        let file = File::open(path)?;
        let mmap = unsafe { Mmap::map(&file)? };
        let fingerprint = FileFingerprint::from_path(path)?;
        let cache_path = helpers::cache_dir().join(index_cache::cache_filename(path));
        let index = index_cache::load_index(&cache_path, &fingerprint).unwrap_or_else(|| {
            let idx = if mmap.len() > 4 * 1024 * 1024 { LineIndex::build_parallel(&mmap) } else { LineIndex::build(&mmap) };
            let _ = index_cache::save_index(&cache_path, &fingerprint, &idx);
            idx
        });
        let parser: Arc<dyn LogParser> = match format {
            LogFormat::Text => Arc::new(TextParser::new(source_id)),
            LogFormat::Jsonl => Arc::new(JsonlParser::new(source_id)),
            LogFormat::Csv => Arc::new(CsvParser::new(source_id)),
        };
        Ok(Self { path: path.to_path_buf(), mmap, index, parser, fingerprint })
    }

    pub fn record_at(&self, record_no: u64) -> Option<LogRecord> {
        let offset = self.index.offset_of(record_no)?;
        self.parser.parse_record(&self.mmap, offset)
    }

    pub fn record_count(&self) -> u64 { self.index.record_count() }
    pub fn raw_bytes(&self) -> &[u8] { &self.mmap }
    pub fn needs_reindex(&self) -> bool { self.fingerprint.has_changed(&self.path) }

    pub fn reload(&mut self) -> Result<(), std::io::Error> {
        let file = File::open(&self.path)?;
        self.mmap = unsafe { Mmap::map(&file)? };
        self.fingerprint = FileFingerprint::from_path(&self.path)?;
        self.index = LineIndex::build(&self.mmap);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_open_text() {
        let mut f = NamedTempFile::with_suffix(".log").unwrap();
        write!(f, "a\nb\nc\n").unwrap();
        let b = LogBackend::open(f.path(), LogFormat::Text, SourceId::from_path(f.path())).unwrap();
        assert_eq!(b.record_count(), 3);
    }

    #[test]
    fn test_record_at() {
        let mut f = NamedTempFile::with_suffix(".log").unwrap();
        write!(f, "a\nb\nc\n").unwrap();
        let b = LogBackend::open(f.path(), LogFormat::Text, SourceId::from_path(f.path())).unwrap();
        assert_eq!(b.record_at(1).unwrap().raw, "b");
    }

    #[test]
    fn test_out_of_bounds() {
        let mut f = NamedTempFile::with_suffix(".log").unwrap();
        write!(f, "a\n").unwrap();
        let b = LogBackend::open(f.path(), LogFormat::Text, SourceId::from_path(f.path())).unwrap();
        assert!(b.record_at(100).is_none());
    }
}
