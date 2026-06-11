//! Query engine for searching log records.

use crate::domain::types::SearchHit;
use crate::source::backend::LogBackend;
use rayon::prelude::*;
use regex::Regex;

#[derive(Debug, Clone)]
pub struct SearchQuery {
    pub pattern: String,
    pub is_regex: bool,
    pub case_insensitive: bool,
    pub whole_word: bool,
    pub field_filters: Vec<FieldFilter>,
}

#[derive(Debug, Clone)]
pub struct FieldFilter {
    pub key: String,
    pub value: String,
}

pub struct QueryEngine;

impl QueryEngine {
    pub fn search(backend: &LogBackend, query: &SearchQuery) -> Result<Vec<SearchHit>, Box<dyn std::error::Error>> {
        // Build the pattern: escape first (if not regex), then add word boundaries, then case flag
        let base_pattern = if query.is_regex {
            query.pattern.clone()
        } else {
            regex::escape(&query.pattern)
        };
        let word_pattern = if query.whole_word && !query.pattern.is_empty() {
            format!(r"\b{}\b", base_pattern)
        } else {
            base_pattern
        };
        let pattern = if query.case_insensitive {
            format!("(?i){}", word_pattern)
        } else {
            word_pattern
        };
        let re = Regex::new(&pattern)?;
        let total = backend.record_count();
        let mut hits = vec![];
        for i in 0..total {
            if let Some(rec) = backend.record_at(i) {
                let pattern_ok = re.is_match(&rec.raw);
                let filters_ok = query.field_filters.iter().all(|f| rec.fields.iter().any(|ff| ff.key == f.key && ff.value == f.value));
                if pattern_ok && filters_ok {
                    hits.push(SearchHit { source_id: rec.source_id, record_index: i, byte_range: rec.byte_range });
                }
            }
        }
        Ok(hits)
    }

    pub fn search_parallel(backend: &LogBackend, query: &SearchQuery) -> Result<Vec<SearchHit>, Box<dyn std::error::Error>> {
        // Build the pattern: escape first (if not regex), then add word boundaries, then case flag
        let base_pattern = if query.is_regex {
            query.pattern.clone()
        } else {
            regex::escape(&query.pattern)
        };
        let word_pattern = if query.whole_word && !query.pattern.is_empty() {
            format!(r"\b{}\b", base_pattern)
        } else {
            base_pattern
        };
        let pattern = if query.case_insensitive {
            format!("(?i){}", word_pattern)
        } else {
            word_pattern
        };
        let re = Regex::new(&pattern)?;
        let total = backend.record_count();
        let hits: Vec<SearchHit> = (0..total).into_par_iter().filter_map(|i| {
            backend.record_at(i).and_then(|rec| {
                let pattern_ok = re.is_match(&rec.raw);
                let filters_ok = query.field_filters.iter().all(|f| rec.fields.iter().any(|ff| ff.key == f.key && ff.value == f.value));
                if pattern_ok && filters_ok { Some(SearchHit { source_id: rec.source_id, record_index: i, byte_range: rec.byte_range }) } else { None }
            })
        }).collect();
        Ok(hits)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::types::{LogFormat, SourceId};
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn backend(content: &[u8]) -> LogBackend {
        let mut f = NamedTempFile::with_suffix(".log").unwrap();
        f.write_all(content).unwrap();
        LogBackend::open(f.path(), LogFormat::Text, SourceId::from_path(f.path())).unwrap()
    }

    #[test]
    fn test_plain_search() {
        let b = backend(b"error\ngood\nerror\n");
        let q = SearchQuery { pattern: "error".into(), is_regex: false, case_insensitive: false, whole_word: false, field_filters: vec![] };
        assert_eq!(QueryEngine::search(&b, &q).unwrap().len(), 2);
    }

    #[test]
    fn test_regex_search() {
        let b = backend(b"err.*\ngood\n");
        let q = SearchQuery { pattern: "err.*".into(), is_regex: true, case_insensitive: false, whole_word: false, field_filters: vec![] };
        assert_eq!(QueryEngine::search(&b, &q).unwrap().len(), 1);
    }

    #[test]
    fn test_no_matches() {
        let b = backend(b"line\n");
        let q = SearchQuery { pattern: "xyz".into(), is_regex: false, case_insensitive: false, whole_word: false, field_filters: vec![] };
        assert!(QueryEngine::search(&b, &q).unwrap().is_empty());
    }

    #[test]
    fn test_invalid_regex() {
        let b = backend(b"test\n");
        let q = SearchQuery { pattern: "[bad".into(), is_regex: true, case_insensitive: false, whole_word: false, field_filters: vec![] };
        assert!(QueryEngine::search(&b, &q).is_err());
    }
}
