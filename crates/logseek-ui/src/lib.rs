//! logseek-ui — User interface for the log-seek application.
//!
//! This crate provides:
//! - Application state management
//! - UI panels (Toolbar, Sources, Status Bar)
//! - Virtual log view with search highlighting
//! - Structured view for JSONL/CSV (table mode)
//! - Theme and styling

pub mod app;
pub mod panels;
pub mod views;
pub mod widgets;
pub mod theme;
