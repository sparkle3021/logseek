//! Record index trait and line index implementation.

use memchr::memchr;
use rayon::prelude::*;

pub trait RecordIndex: Send + Sync {
    fn record_count(&self) -> u64;
    fn offset_of(&self, record_no: u64) -> Option<u64>;
}

pub struct LineIndex { offsets: Vec<u64> }

impl LineIndex {
    pub fn build(data: &[u8]) -> Self {
        let mut offsets = Vec::with_capacity(data.len() / 80);
        offsets.push(0);
        let mut pos = 0;
        while pos < data.len() {
            if let Some(nl) = memchr(b'\n', &data[pos..]) {
                pos += nl + 1;
                if pos < data.len() { offsets.push(pos as u64); }
            } else { break; }
        }
        Self { offsets }
    }

    pub fn build_parallel(data: &[u8]) -> Self {
        let chunk_size = 4 * 1024 * 1024;
        let n = (data.len() + chunk_size - 1) / chunk_size;
        let chunks: Vec<Vec<u64>> = (0..n).into_par_iter().map(|i| {
            let s = i * chunk_size;
            let e = std::cmp::min(s + chunk_size, data.len());
            let base = s as u64;
            let mut v = vec![];
            let mut p = 0;
            while let Some(nl) = memchr(b'\n', &data[s+p..e]) { v.push(base + (p + nl) as u64 + 1); p += nl + 1; }
            v
        }).collect();
        let mut all: Vec<u64> = chunks.into_iter().flatten().collect();
        all.sort_unstable();
        let mut offsets = vec![0u64];
        offsets.extend(all.into_iter().filter(|&o| (o as usize) < data.len()));
        Self { offsets }
    }

    pub fn append_new_offsets(&mut self, data: &[u8], from: u64) {
        let start = from as usize;
        if start >= data.len() { return; }
        // Add the starting offset if not already present
        if self.offsets.last() != Some(&from) {
            self.offsets.push(from);
        }
        let mut pos = start;
        while pos < data.len() {
            if let Some(nl) = memchr(b'\n', &data[pos..]) {
                pos += nl + 1;
                if pos < data.len() { self.offsets.push(pos as u64); }
            } else { break; }
        }
    }

    pub fn offsets(&self) -> &[u64] { &self.offsets }
    pub fn from_offsets(offsets: Vec<u64>) -> Self { Self { offsets } }
}

impl RecordIndex for LineIndex {
    fn record_count(&self) -> u64 { self.offsets.len() as u64 }
    fn offset_of(&self, record_no: u64) -> Option<u64> { self.offsets.get(record_no as usize).copied() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_empty() { assert_eq!(LineIndex::build(b"").record_count(), 1); }

    #[test]
    fn test_build_multiple() { assert_eq!(LineIndex::build(b"a\nb\nc\n").record_count(), 3); }

    #[test]
    fn test_offset_lookup() {
        let idx = LineIndex::build(b"a\nb\nc\n");
        assert_eq!(idx.offset_of(0), Some(0));
        assert_eq!(idx.offset_of(1), Some(2));
    }

    #[test]
    fn test_parallel_matches_sequential() {
        let d = b"a\nb\nc\nd\ne\n";
        assert_eq!(LineIndex::build(d).offsets(), LineIndex::build_parallel(d).offsets());
    }

    #[test]
    fn test_incremental_append() {
        let mut idx = LineIndex::build(b"a\n");
        idx.append_new_offsets(b"a\nb\nc\n", 2);
        assert_eq!(idx.record_count(), 3);
    }
}
