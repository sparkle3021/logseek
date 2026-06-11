//! Application state management.

use logseek_core::domain::types::{LogFormat, LogRecord, SearchHit, SourceId};
use logseek_core::query::engine::{FieldFilter, SearchQuery};
use logseek_core::session::session::Session;
use logseek_core::source::backend::LogBackend;
use logseek_core::workspace::Workspace;
use std::path::PathBuf;
use std::sync::Arc;
use crate::theme::colors::ThemeMode;
use crate::theme::fonts::FontConfig;

/// A log source with its backend.
pub struct LogSource {
    pub id: SourceId,
    pub name: String,
    pub path: PathBuf,
    pub format: LogFormat,
    pub backend: Arc<LogBackend>,
}

/// Default workspace name - cannot be deleted
pub const DEFAULT_WORKSPACE_NAME: &str = "默认工作区";

/// Central application state.
pub struct AppState {
    pub sources: Vec<LogSource>,
    pub active_source: Option<usize>,
    pub search_query: SearchQuery,
    pub search_results: Vec<SearchHit>,
    pub search_time_ms: f64,
    pub session: Session,
    pub show_structured_view: bool,
    pub status_message: String,
    pub theme: ThemeMode,
    pub show_search_results: bool,
    pub selected_search_result: Option<usize>,
    pub scroll_to_line: Option<u64>,
    pub current_match_index: Option<usize>,
    pub font_config: FontConfig,
    pub show_font_settings: bool,
    // Filter state
    pub filter_pattern: String,
    pub filter_enabled: bool,
    pub filter_is_regex: bool,
    pub filtered_lines: Vec<u64>,
    // Workspace state
    pub current_workspace: Option<Workspace>,
    pub available_workspaces: Vec<String>,
    pub show_new_workspace_dialog: bool,
    pub new_workspace_name: String,
    // Auto-refresh state
    pub auto_refresh: bool,
    pub last_refresh_time: std::time::Instant,
    // File change detection (for non-auto-refresh mode)
    pub show_update_dialog: bool,
    pub last_file_check_time: std::time::Instant,
    pub file_sizes: std::collections::HashMap<std::path::PathBuf, u64>,
    // Confirmation dialogs
    pub show_clear_confirm_dialog: bool,
}

impl AppState {
    pub fn new() -> Self {
        // Load available workspaces
        let available_workspaces = load_available_workspaces();
        
        // Auto-load default workspace if it exists
        let mut initial_workspace = None;
        let mut initial_sources = Vec::new();
        if available_workspaces.contains(&DEFAULT_WORKSPACE_NAME.to_string()) {
            let ws_path = logseek_core::utils::helpers::workspaces_dir()
                .join(format!("{}.json", DEFAULT_WORKSPACE_NAME));
            if let Ok(workspace) = Workspace::load(&ws_path) {
                // Load files from default workspace
                for path in &workspace.log_files {
                    if path.exists() {
                        let format = path.extension()
                            .and_then(|ext| ext.to_str())
                            .and_then(|ext| LogFormat::from_extension(ext))
                            .unwrap_or(LogFormat::Text);
                        let source_id = SourceId::from_path(path);
                        let name = path.file_name()
                            .map(|n| n.to_string_lossy().to_string())
                            .unwrap_or_else(|| "未知文件".to_string());
                        if let Ok(backend) = LogBackend::open(path, format, source_id.clone()) {
                            initial_sources.push(LogSource {
                                id: source_id,
                                name,
                                path: path.clone(),
                                format,
                                backend: Arc::new(backend),
                            });
                        }
                    }
                }
                initial_workspace = Some(workspace);
            }
        }
        
        let active_source = if initial_sources.is_empty() { None } else { Some(0) };
        
        Self {
            sources: initial_sources,
            active_source,
            search_query: SearchQuery { pattern: String::new(), is_regex: false, case_insensitive: true, whole_word: false, field_filters: vec![] },
            search_results: vec![],
            search_time_ms: 0.0,
            session: Session::default_session(),
            show_structured_view: false,
            status_message: String::new(),
            theme: ThemeMode::Light,
            show_search_results: false,
            selected_search_result: None,
            scroll_to_line: None,
            current_match_index: None,
            font_config: FontConfig::default(),
            show_font_settings: false,
            filter_pattern: String::new(),
            filter_enabled: false,
            filter_is_regex: false,
            filtered_lines: Vec::new(),
            // Workspace state
            current_workspace: initial_workspace,
            available_workspaces,
            show_new_workspace_dialog: false,
            new_workspace_name: String::new(),
            // Auto-refresh state
            auto_refresh: false,
            last_refresh_time: std::time::Instant::now(),
            // File change detection
            show_update_dialog: false,
            last_file_check_time: std::time::Instant::now(),
            file_sizes: std::collections::HashMap::new(),
            // Confirmation dialogs
            show_clear_confirm_dialog: false,
        }
    }

    pub fn toggle_theme(&mut self) {
        self.theme = match self.theme {
            ThemeMode::Dark => ThemeMode::Light,
            ThemeMode::Light => ThemeMode::Dark,
        };
    }

    pub fn add_source(&mut self, source: LogSource) {
        self.sources.push(source);
        if self.active_source.is_none() {
            self.active_source = Some(0);
        }
    }

    pub fn remove_source(&mut self, index: usize) {
        if index < self.sources.len() {
            self.sources.remove(index);
            self.active_source = if self.sources.is_empty() {
                None
            } else {
                Some(index.min(self.sources.len() - 1))
            };
        }
    }

    pub fn set_active(&mut self, index: usize) {
        if index < self.sources.len() {
            self.active_source = Some(index);
        }
    }

    pub fn active_backend(&self) -> Option<&LogBackend> {
        self.active_source.and_then(|i| self.sources.get(i)).map(|s| s.backend.as_ref())
    }

    pub fn active_source_id(&self) -> Option<&SourceId> {
        self.active_source.and_then(|i| self.sources.get(i)).map(|s| &s.id)
    }

    pub fn is_structured_format(&self) -> bool {
        self.active_source
            .and_then(|i| self.sources.get(i))
            .map(|s| matches!(s.format, LogFormat::Jsonl | LogFormat::Csv))
            .unwrap_or(false)
    }

    pub fn total_records(&self) -> u64 {
        self.sources.iter().map(|s| s.backend.record_count()).sum()
    }

    /// Execute search on current source (with filter integration)
    pub fn search_current(&mut self) {
        if self.search_query.pattern.is_empty() {
            self.search_results.clear();
            self.search_time_ms = 0.0;
            self.status_message = String::new();
            self.current_match_index = None;
            self.scroll_to_line = None;
            return;
        }

        let backend = match self.active_backend() {
            Some(b) => b,
            None => {
                self.status_message = "没有打开的日志文件".to_string();
                return;
            }
        };

        let start = std::time::Instant::now();
        match logseek_core::query::engine::QueryEngine::search(backend, &self.search_query) {
            Ok(hits) => {
                self.search_time_ms = start.elapsed().as_secs_f64() * 1000.0;
                self.search_results = hits;
                
                // Sort by record index (time order)
                self.search_results.sort_by_key(|h| h.record_index);
                
                // Auto-apply filter if enabled (search + filter integration)
                if self.filter_enabled && !self.filter_pattern.is_empty() {
                    self.apply_filter_internal();
                }
                
                // Auto-scroll to first match
                if !self.search_results.is_empty() {
                    self.current_match_index = Some(0);
                    self.scroll_to_line = Some(self.search_results[0].record_index);
                    self.status_message = format!("搜索完成: {} 条匹配 (1/{})", self.search_results.len(), self.search_results.len());
                } else {
                    self.current_match_index = None;
                    self.status_message = "搜索完成: 0 条匹配".to_string();
                }
            }
            Err(e) => {
                self.status_message = format!("搜索错误: {}", e);
                self.search_results.clear();
                self.current_match_index = None;
            }
        }
    }

    /// Execute search on current workspace sources only
    pub fn search_workspace(&mut self) {
        if self.search_query.pattern.is_empty() {
            self.search_results.clear();
            self.search_time_ms = 0.0;
            self.status_message = String::new();
            self.current_match_index = None;
            self.scroll_to_line = None;
            return;
        }

        // Get workspace file paths to filter sources
        let workspace_paths: std::collections::HashSet<std::path::PathBuf> = 
            if let Some(ref workspace) = self.current_workspace {
                workspace.log_files.iter().cloned().collect()
            } else {
                // No workspace - search all sources
                self.sources.iter().map(|s| s.path.clone()).collect()
            };

        let start = std::time::Instant::now();
        let mut all_hits = Vec::new();

        // Only search sources that belong to current workspace
        for source in &self.sources {
            if workspace_paths.contains(&source.path) {
                match logseek_core::query::engine::QueryEngine::search(&source.backend, &self.search_query) {
                    Ok(hits) => all_hits.extend(hits),
                    Err(e) => {
                        self.status_message = format!("搜索错误: {}", e);
                        return;
                    }
                }
            }
        }

        // Sort by source_id then record_index (time order within each file)
        all_hits.sort_by(|a, b| {
            a.source_id.0.cmp(&b.source_id.0)
                .then(a.record_index.cmp(&b.record_index))
        });

        self.search_time_ms = start.elapsed().as_secs_f64() * 1000.0;
        self.search_results = all_hits;
        
        // Auto-scroll to first match
        if !self.search_results.is_empty() {
            self.current_match_index = Some(0);
            self.scroll_to_line = Some(self.search_results[0].record_index);
            self.switch_to_source_of_match(0);
            self.status_message = format!("搜索工作区: {} 条匹配 (1/{})", self.search_results.len(), self.search_results.len());
        } else {
            self.current_match_index = None;
            self.status_message = "搜索工作区: 0 条匹配".to_string();
        }
    }

    /// Execute search on all sources (legacy - kept for compatibility)
    pub fn search_all(&mut self) {
        // Now delegates to workspace search
        self.search_workspace();
    }

    /// Navigate to the next search match
    pub fn navigate_next(&mut self) {
        if self.search_results.is_empty() {
            return;
        }

        let next_index = match self.current_match_index {
            Some(current) => (current + 1) % self.search_results.len(),
            None => 0,
        };

        self.current_match_index = Some(next_index);
        self.scroll_to_line = Some(self.search_results[next_index].record_index);
        self.switch_to_source_of_match(next_index);
        self.status_message = format!("匹配 {}/{}", next_index + 1, self.search_results.len());
    }

    /// Navigate to the previous search match
    pub fn navigate_previous(&mut self) {
        if self.search_results.is_empty() {
            return;
        }

        let prev_index = match self.current_match_index {
            Some(current) => {
                if current == 0 {
                    self.search_results.len() - 1
                } else {
                    current - 1
                }
            }
            None => self.search_results.len() - 1,
        };

        self.current_match_index = Some(prev_index);
        self.scroll_to_line = Some(self.search_results[prev_index].record_index);
        self.switch_to_source_of_match(prev_index);
        self.status_message = format!("匹配 {}/{}", prev_index + 1, self.search_results.len());
    }

    /// Switch to the source containing the match at the given index
    fn switch_to_source_of_match(&mut self, match_index: usize) {
        if let Some(hit) = self.search_results.get(match_index) {
            for (i, source) in self.sources.iter().enumerate() {
                if source.id == hit.source_id {
                    self.active_source = Some(i);
                    break;
                }
            }
        }
    }

    /// Refresh current source (reload file) - preserves scroll position
    pub fn refresh_current(&mut self) {
        if let Some(source) = self.active_source.and_then(|i| self.sources.get_mut(i)) {
            let old_count = source.backend.record_count();
            match LogBackend::open(&source.path, source.format, source.id.clone()) {
                Ok(backend) => {
                    let new_count = backend.record_count();
                    source.backend = Arc::new(backend);
                    let diff = new_count.saturating_sub(old_count);
                    if diff > 0 {
                        self.status_message = format!("刷新完成: 新增 {} 条记录", diff);
                    } else {
                        self.status_message = "刷新完成: 无新增".to_string();
                    }
                    // Re-run search if active, but DON'T reset scroll position
                    if !self.search_query.pattern.is_empty() {
                        self.search_current_no_scroll();
                    }
                }
                Err(e) => {
                    self.status_message = format!("刷新失败: {}", e);
                }
            }
        } else {
            self.status_message = "没有打开的日志文件".to_string();
        }
        self.last_refresh_time = std::time::Instant::now();
    }

    /// Copy all visible content to clipboard
    pub fn copy_all_to_clipboard(&mut self) {
        let backend = match self.active_backend() {
            Some(b) => b,
            None => {
                self.status_message = "没有打开的日志文件".to_string();
                return;
            }
        };

        let total = backend.record_count();
        let mut content = String::new();

        // If filter is enabled, only copy filtered lines
        if self.filter_enabled && !self.filter_pattern.is_empty() {
            for &line_idx in &self.filtered_lines {
                if let Some(record) = backend.record_at(line_idx) {
                    content.push_str(&record.raw);
                    content.push('\n');
                }
            }
        } else {
            // Copy all lines
            for i in 0..total {
                if let Some(record) = backend.record_at(i) {
                    content.push_str(&record.raw);
                    content.push('\n');
                }
            }
        }

        // Copy to clipboard using arboard
        match arboard::Clipboard::new() {
            Ok(mut clipboard) => {
                if let Err(e) = clipboard.set_text(&content) {
                    self.status_message = format!("复制失败: {}", e);
                } else {
                    let line_count = content.lines().count();
                    self.status_message = format!("已复制 {} 行到剪贴板", line_count);
                }
            }
            Err(e) => {
                self.status_message = format!("复制失败: {}", e);
            }
        }
    }

    /// Copy a single line to clipboard
    pub fn copy_line_to_clipboard(&mut self, line_index: u64) {
        let backend = match self.active_backend() {
            Some(b) => b,
            None => return,
        };

        if let Some(record) = backend.record_at(line_index) {
            match arboard::Clipboard::new() {
                Ok(mut clipboard) => {
                    if let Err(e) = clipboard.set_text(&record.raw) {
                        self.status_message = format!("复制失败: {}", e);
                    } else {
                        self.status_message = format!("已复制第 {} 行", line_index + 1);
                    }
                }
                Err(e) => {
                    self.status_message = format!("复制失败: {}", e);
                }
            }
        }
    }

    /// Show clear confirmation dialog
    pub fn request_clear_log(&mut self) {
        if self.active_source.is_some() {
            self.show_clear_confirm_dialog = true;
        } else {
            self.status_message = "没有打开的日志文件".to_string();
        }
    }

    /// Confirm and clear the content of the current log file
    pub fn confirm_clear_log(&mut self) {
        self.show_clear_confirm_dialog = false;

        let source_info = self.active_source.and_then(|i| {
            self.sources.get(i).map(|s| (s.path.clone(), s.format, s.id.clone(), s.name.clone()))
        });

        if let Some((path, format, source_id, name)) = source_info {
            // Step 1: Remove the source (drops the mmap)
            if let Some(idx) = self.active_source {
                self.sources.remove(idx);
                self.active_source = if self.sources.is_empty() { None } else { Some(0) };
            }
            
            // Step 2: Truncate file to 0 bytes
            match std::fs::OpenOptions::new()
                .write(true)
                .truncate(true)
                .open(&path)
            {
                Ok(_file) => {
                    // Step 3: Re-open the file
                    match LogBackend::open(&path, format, source_id) {
                        Ok(backend) => {
                            let source = LogSource {
                                id: SourceId::from_path(&path),
                                name: name.clone(),
                                path: path.clone(),
                                format,
                                backend: Arc::new(backend),
                            };
                            self.add_source(source);
                            self.search_results.clear();
                            self.current_match_index = None;
                            self.scroll_to_line = None;
                            self.filtered_lines.clear();
                            self.status_message = format!("已清空: {}", name);
                        }
                        Err(e) => {
                            self.status_message = format!("重新加载失败: {}", e);
                        }
                    }
                }
                Err(e) => {
                    self.status_message = format!("清空失败: {}", e);
                    // Try to re-open the file anyway
                    if let Ok(backend) = LogBackend::open(&path, format, SourceId::from_path(&path)) {
                        let source = LogSource {
                            id: SourceId::from_path(&path),
                            name: name.clone(),
                            path: path.clone(),
                            format,
                            backend: Arc::new(backend),
                        };
                        self.add_source(source);
                    }
                }
            }
        }
    }

    /// Cancel clear confirmation
    pub fn cancel_clear_log(&mut self) {
        self.show_clear_confirm_dialog = false;
    }

    /// Search without resetting scroll position (for auto-refresh)
    fn search_current_no_scroll(&mut self) {
        if self.search_query.pattern.is_empty() {
            return;
        }

        let backend = match self.active_backend() {
            Some(b) => b,
            None => return,
        };

        match logseek_core::query::engine::QueryEngine::search(backend, &self.search_query) {
            Ok(hits) => {
                self.search_results = hits;
                self.search_results.sort_by_key(|h| h.record_index);
                if !self.search_results.is_empty() && self.current_match_index.is_none() {
                    self.current_match_index = Some(0);
                }
            }
            Err(_) => {}
        }
    }

    /// Check if any source file has been modified since last refresh
    pub fn check_file_changes(&mut self) -> bool {
        let now = std::time::Instant::now();
        // Only check every 1 second
        if now.duration_since(self.last_refresh_time).as_secs() < 1 {
            return false;
        }

        for source in &self.sources {
            if let Ok(metadata) = std::fs::metadata(&source.path) {
                if let Ok(modified) = metadata.modified() {
                    let backend = &source.backend;
                    // Simple check: if file size changed
                    if metadata.len() > backend.record_count() * 100 {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Auto-refresh all sources if changes detected
    pub fn auto_refresh_if_needed(&mut self) {
        if !self.auto_refresh {
            return;
        }

        let now = std::time::Instant::now();
        if now.duration_since(self.last_refresh_time).as_secs() < 1 {
            return;
        }

        // Refresh current source
        self.refresh_current();
    }

    /// Check for file changes when auto-refresh is OFF
    /// Shows dialog if changes detected (max once per 5 seconds)
    pub fn check_file_changes_for_dialog(&mut self) {
        // Skip if auto-refresh is on (it handles this automatically)
        if self.auto_refresh {
            return;
        }

        // Skip if dialog is already showing
        if self.show_update_dialog {
            return;
        }

        // Only check every 5 seconds
        let now = std::time::Instant::now();
        if now.duration_since(self.last_file_check_time).as_secs() < 5 {
            return;
        }
        self.last_file_check_time = now;

        // Check each source for size changes
        for source in &self.sources {
            if let Ok(metadata) = std::fs::metadata(&source.path) {
                let current_size = metadata.len();
                let last_size = self.file_sizes.get(&source.path).copied().unwrap_or(current_size);

                if current_size > last_size {
                    // File has grown - show update dialog
                    self.show_update_dialog = true;
                    self.status_message = format!("文件 '{}' 存在更新", source.name);
                    break;
                }
            }
        }

        // Update stored file sizes
        self.update_file_sizes();
    }

    /// Update stored file sizes for all sources
    pub fn update_file_sizes(&mut self) {
        for source in &self.sources {
            if let Ok(metadata) = std::fs::metadata(&source.path) {
                self.file_sizes.insert(source.path.clone(), metadata.len());
            }
        }
    }

    /// Accept file reload from dialog
    pub fn accept_file_reload(&mut self) {
        self.show_update_dialog = false;
        self.refresh_current();
    }

    /// Decline file reload from dialog
    pub fn decline_file_reload(&mut self) {
        self.show_update_dialog = false;
        // Update sizes so we don't prompt again for the same change
        self.update_file_sizes();
    }

    /// Apply filter to current source (with search integration)
    pub fn apply_filter(&mut self) {
        self.apply_filter_internal();
        
        // Re-run search if pattern exists (filter + search integration)
        if !self.search_query.pattern.is_empty() {
            self.search_current_no_scroll();
        }
    }

    /// Internal filter logic (doesn't trigger search)
    fn apply_filter_internal(&mut self) {
        self.filtered_lines.clear();
        
        if !self.filter_enabled || self.filter_pattern.is_empty() {
            return;
        }

        let backend = match self.active_backend() {
            Some(b) => b,
            None => return,
        };

        let total = backend.record_count();
        let pattern = &self.filter_pattern;
        
        // Build regex or use simple string matching
        let matches = if self.filter_is_regex {
            match regex::Regex::new(pattern) {
                Ok(re) => {
                    (0..total)
                        .filter(|&i| {
                            backend.record_at(i)
                                .map(|r| re.is_match(&r.raw))
                                .unwrap_or(false)
                        })
                        .collect()
                }
                Err(e) => {
                    self.status_message = format!("正则表达式错误: {}", e);
                    return;
                }
            }
        } else {
            let pattern_lower = pattern.to_lowercase();
            (0..total)
                .filter(|&i| {
                    backend.record_at(i)
                        .map(|r| r.raw.to_lowercase().contains(&pattern_lower))
                        .unwrap_or(false)
                })
                .collect()
        };

        self.filtered_lines = matches;
        self.status_message = format!("过滤: 显示 {}/{} 行", self.filtered_lines.len(), total);
    }

    /// Check if a line should be visible (considering filter)
    pub fn is_line_visible(&self, line_index: u64) -> bool {
        if !self.filter_enabled || self.filter_pattern.is_empty() {
            return true;
        }
        self.filtered_lines.contains(&line_index)
    }

    /// Get visible line count
    pub fn visible_line_count(&self) -> usize {
        if !self.filter_enabled || self.filter_pattern.is_empty() {
            // Return total record count
            self.active_backend()
                .map(|b| b.record_count() as usize)
                .unwrap_or(0)
        } else {
            self.filtered_lines.len()
        }
    }

    /// Jump to a specific search result
    pub fn jump_to_result(&mut self, item: &crate::panels::search_results::SearchResultItem) {
        // Find and set the active source
        for (i, source) in self.sources.iter().enumerate() {
            if source.id == item.hit.source_id {
                self.active_source = Some(i);
                break;
            }
        }
        
        // Set scroll position to the matching line
        self.scroll_to_line = Some(item.line_number);
        
        // Update status
        self.status_message = format!("跳转到 {} 第 {} 行", item.source_name, item.line_number + 1);
    }
}

/// Load available workspace names from disk
fn load_available_workspaces() -> Vec<String> {
    use logseek_core::utils::helpers;
    
    let ws_dir = helpers::workspaces_dir();
    let mut workspaces = Vec::new();
    
    if let Ok(entries) = std::fs::read_dir(&ws_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map(|e| e == "json").unwrap_or(false) {
                if let Some(name) = path.file_stem() {
                    workspaces.push(name.to_string_lossy().to_string());
                }
            }
        }
    }
    
    workspaces.sort();
    workspaces
}

#[cfg(test)]
mod tests {
    use super::*;
    use logseek_core::domain::types::ByteRange;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn make_source(name: &str) -> LogSource {
        let mut f = NamedTempFile::with_suffix(".log").unwrap();
        write!(f, "line1\nline2\n").unwrap();
        let id = SourceId(name.to_string());
        let backend = Arc::new(LogBackend::open(f.path(), LogFormat::Text, id.clone()).unwrap());
        LogSource { id, name: name.to_string(), path: f.path().to_path_buf(), format: LogFormat::Text, backend }
    }

    #[test]
    fn test_add_source() {
        let mut state = AppState::new();
        state.add_source(make_source("test"));
        assert_eq!(state.sources.len(), 1);
        assert_eq!(state.active_source, Some(0));
    }

    #[test]
    fn test_remove_source() {
        let mut state = AppState::new();
        state.add_source(make_source("a"));
        state.add_source(make_source("b"));
        state.remove_source(0);
        assert_eq!(state.sources.len(), 1);
        assert_eq!(state.active_source, Some(0));
    }

    #[test]
    fn test_active_backend() {
        let mut state = AppState::new();
        assert!(state.active_backend().is_none());
        state.add_source(make_source("test"));
        assert!(state.active_backend().is_some());
    }
}
