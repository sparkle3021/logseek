//! Log parsers for different formats.
//!
//! Provides the `LogParser` trait and implementations for:
//! - Text (plain text logs)
//! - JSON Lines (JSONL/NDJSON)
//! - CSV (comma-separated values)

pub mod csv;
pub mod jsonl;
pub mod text;
pub mod trait_def;
