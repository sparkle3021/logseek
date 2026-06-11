//! Index cache for fast startup.

use crate::domain::fingerprint::FileFingerprint;
use crate::source::index::{LineIndex, RecordIndex};
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::Path;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

const MAGIC: &[u8; 5] = b"OLIDX";
const VERSION: u16 = 1;

pub fn cache_filename(path: &Path) -> String {
    let mut h = DefaultHasher::new();
    path.to_string_lossy().hash(&mut h);
    format!("{:016x}.olidx", h.finish())
}

pub fn save_index(path: &Path, fp: &FileFingerprint, idx: &LineIndex) -> Result<(), std::io::Error> {
    let mut w = BufWriter::new(File::create(path)?);
    w.write_all(MAGIC)?;
    w.write_all(&VERSION.to_le_bytes())?;
    w.write_all(&fp.file_size.to_le_bytes())?;
    w.write_all(&fp.modified_time.to_le_bytes())?;
    w.write_all(&(idx.record_count() as u64).to_le_bytes())?;
    for &o in idx.offsets() { w.write_all(&o.to_le_bytes())?; }
    w.flush()
}

pub fn load_index(path: &Path, fp: &FileFingerprint) -> Option<LineIndex> {
    let mut r = BufReader::new(File::open(path).ok()?);
    let mut magic = [0u8; 5]; r.read_exact(&mut magic).ok()?;
    if magic != *MAGIC { return None; }
    let mut vb = [0u8; 2]; r.read_exact(&mut vb).ok()?;
    if u16::from_le_bytes(vb) != VERSION { return None; }
    let mut sb = [0u8; 8]; r.read_exact(&mut sb).ok()?;
    let sz = u64::from_le_bytes(sb);
    let mut mb = [0u8; 8]; r.read_exact(&mut mb).ok()?;
    let mt = u64::from_le_bytes(mb);
    if sz != fp.file_size || mt != fp.modified_time { return None; }
    let mut cb = [0u8; 8]; r.read_exact(&mut cb).ok()?;
    let count = u64::from_le_bytes(cb) as usize;
    let mut offsets = Vec::with_capacity(count);
    for _ in 0..count {
        let mut ob = [0u8; 8]; r.read_exact(&mut ob).ok()?;
        offsets.push(u64::from_le_bytes(ob));
    }
    Some(LineIndex::from_offsets(offsets))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_roundtrip() {
        let f = NamedTempFile::with_suffix(".olidx").unwrap();
        let fp = FileFingerprint { file_size: 100, modified_time: 1000 };
        let idx = LineIndex::from_offsets(vec![0, 10, 20]);
        save_index(f.path(), &fp, &idx).unwrap();
        let loaded = load_index(f.path(), &fp).unwrap();
        assert_eq!(loaded.offsets(), &[0, 10, 20]);
    }

    #[test]
    fn test_fingerprint_mismatch() {
        let f = NamedTempFile::with_suffix(".olidx").unwrap();
        let fp = FileFingerprint { file_size: 100, modified_time: 1000 };
        save_index(f.path(), &fp, &LineIndex::from_offsets(vec![0])).unwrap();
        let fp2 = FileFingerprint { file_size: 200, modified_time: 2000 };
        assert!(load_index(f.path(), &fp2).is_none());
    }

    #[test]
    fn test_deterministic_path() {
        assert_eq!(cache_filename(Path::new("/a/b.log")), cache_filename(Path::new("/a/b.log")));
    }
}
