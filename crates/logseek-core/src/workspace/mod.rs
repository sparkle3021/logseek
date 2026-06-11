//! Workspace management for log-seek.
//!
//! Workspaces store references to log files and directories,
//! allowing users to quickly reload their working set.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// A workspace containing log sources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workspace {
    /// Workspace name
    pub name: String,
    /// List of log file paths
    pub log_files: Vec<PathBuf>,
    /// List of log directory paths
    pub log_dirs: Vec<PathBuf>,
    /// Last active source index
    pub active_source: Option<usize>,
    /// Creation timestamp
    pub created_at: String,
    /// Last modified timestamp
    pub modified_at: String,
}

impl Workspace {
    /// Create a new workspace
    pub fn new(name: String) -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        Self {
            name,
            log_files: Vec::new(),
            log_dirs: Vec::new(),
            active_source: None,
            created_at: now.clone(),
            modified_at: now,
        }
    }

    /// Add a log file to the workspace
    pub fn add_file(&mut self, path: PathBuf) {
        if !self.log_files.contains(&path) {
            self.log_files.push(path);
            self.modified_at = chrono::Utc::now().to_rfc3339();
        }
    }

    /// Add a log directory to the workspace
    pub fn add_dir(&mut self, path: PathBuf) {
        if !self.log_dirs.contains(&path) {
            self.log_dirs.push(path);
            self.modified_at = chrono::Utc::now().to_rfc3339();
        }
    }

    /// Remove a log file from the workspace
    pub fn remove_file(&mut self, path: &PathBuf) {
        self.log_files.retain(|p| p != path);
        self.modified_at = chrono::Utc::now().to_rfc3339();
    }

    /// Remove a log directory from the workspace
    pub fn remove_dir(&mut self, path: &PathBuf) {
        self.log_dirs.retain(|p| p != path);
        self.modified_at = chrono::Utc::now().to_rfc3339();
    }

    /// Save workspace to file
    pub fn save(&self, path: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    /// Load workspace from file
    pub fn load(path: &std::path::Path) -> Result<Self, Box<dyn std::error::Error>> {
        let json = std::fs::read_to_string(path)?;
        let workspace: Workspace = serde_json::from_str(&json)?;
        Ok(workspace)
    }
}
