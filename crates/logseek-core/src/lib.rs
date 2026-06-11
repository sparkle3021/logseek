//! logseek-core — Core logic for the log-seek application.
//!
//! This crate provides:
//! - Domain types for log sources, records, and search results
//! - Log parsers for Text, JSON Lines, and CSV formats
//! - Memory-mapped file access with line indexing
//! - Index caching for fast startup
//! - Search engine with parallel execution
//! - File watching for real-time updates
//! - Session persistence

pub mod cache;
pub mod domain;
pub mod parser;
pub mod query;
pub mod session;
pub mod source;
pub mod utils;
pub mod watcher;
pub mod workspace;
