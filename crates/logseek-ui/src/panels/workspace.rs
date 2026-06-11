//! Workspace panel - manage workspaces and log sources.

use crate::app::state::{AppState, DEFAULT_WORKSPACE_NAME};
use crate::theme::{colors, fonts};
use crate::theme::colors::ThemeMode;
use egui::{RichText, SidePanel, ScrollArea};
use logseek_core::workspace::Workspace;
use logseek_core::utils::helpers;
use std::path::PathBuf;

pub fn show(ctx: &egui::Context, state: &mut AppState) {
    SidePanel::left("workspace_panel")
        .resizable(true)
        .default_width(250.0)
        .width_range(150.0..=400.0)
        .show(ctx, |ui| {
            let theme_colors = state.theme.colors();
            
            // Header
            ui.horizontal(|ui| {
                ui.heading(RichText::new("工作区").size(fonts::FONT_SIZE_HEADING));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("新建").on_hover_text("新建工作区").clicked() {
                        state.show_new_workspace_dialog = true;
                    }
                    // Close workspace button (only show when workspace is active)
                    if state.current_workspace.is_some() {
                        if ui.button("关闭").on_hover_text("关闭工作区").clicked() {
                            close_workspace(state);
                        }
                    }
                });
            });
            ui.separator();
            
            // Current workspace info
            if let Some(ref workspace) = state.current_workspace {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(&workspace.name)
                            .size(fonts::FONT_SIZE_BODY)
                            .color(theme_colors.accent),
                    );
                    ui.label(
                        RichText::new(format!("({} 个文件)", workspace.log_files.len()))
                            .size(fonts::FONT_SIZE_SMALL)
                            .color(theme_colors.text_secondary),
                    );
                });
                ui.separator();
            }
            
            // Action buttons
            ui.horizontal(|ui| {
                if ui.button("打开文件").clicked() {
                    open_file_dialog(state);
                }
                if ui.button("打开目录").clicked() {
                    open_dir_dialog(state);
                }
                // Clear all logs button (only when workspace has files)
                if let Some(ref workspace) = state.current_workspace {
                    if !workspace.log_files.is_empty() || !workspace.log_dirs.is_empty() {
                        if ui.button("清空日志").on_hover_text("清除所有导入的日志文件").clicked() {
                            clear_workspace_logs(state);
                        }
                    }
                }
            });
            ui.separator();
            
            // Workspace list
            ui.label(RichText::new("工作区列表").size(fonts::FONT_SIZE_SMALL).color(theme_colors.text_secondary));
            
            ScrollArea::vertical().show(ui, |ui| {
                let mut workspaces = state.available_workspaces.clone();
                // Sort to keep default workspace at top
                workspaces.sort_by(|a, b| {
                    if a == DEFAULT_WORKSPACE_NAME {
                        std::cmp::Ordering::Less
                    } else if b == DEFAULT_WORKSPACE_NAME {
                        std::cmp::Ordering::Greater
                    } else {
                        a.cmp(b)
                    }
                });
                
                for ws_name in workspaces.iter() {
                    let is_active = state.current_workspace.as_ref().map(|w| w.name == *ws_name).unwrap_or(false);
                    let is_default = ws_name == DEFAULT_WORKSPACE_NAME;
                    let text_color = if is_active { theme_colors.accent } else { theme_colors.text_primary };
                    
                    ui.horizontal(|ui| {
                        let label = RichText::new(ws_name.as_str()).color(text_color);
                        if ui.selectable_label(is_active, label).clicked() {
                            load_workspace(state, ws_name);
                        }
                        // Only show delete button for non-default workspaces
                        if !is_default {
                            if ui.small_button("×").on_hover_text("删除工作区").clicked() {
                                delete_workspace(state, ws_name);
                            }
                        }
                    });
                }
            });
            
            ui.separator();
            
            // Log files in current workspace
            if let Some(ref workspace) = state.current_workspace {
                let workspace_files = workspace.log_files.clone();
                let workspace_name = workspace.name.clone();
                
                ui.label(RichText::new("日志文件").size(fonts::FONT_SIZE_SMALL).color(theme_colors.text_secondary));
                
                ScrollArea::vertical().show(ui, |ui| {
                    let mut to_open = None;
                    let mut to_remove_from_workspace = None;
                    
                    for (i, file_path) in workspace_files.iter().enumerate() {
                        let file_name = file_path.file_name()
                            .map(|n| n.to_string_lossy().to_string())
                            .unwrap_or_else(|| "未知文件".to_string());
                        
                        // Check if file is already open in a tab
                        let is_open = state.sources.iter().any(|s| s.path == *file_path);
                        let is_active = is_open && state.sources.iter()
                            .enumerate()
                            .any(|(idx, s)| s.path == *file_path && state.active_source == Some(idx));
                        
                        let text_color = if is_active {
                            theme_colors.accent
                        } else if is_open {
                            theme_colors.text_primary
                        } else {
                            theme_colors.text_secondary
                        };
                        
                        ui.horizontal(|ui| {
                            // Click to open file in tab
                            let label = RichText::new(&file_name).color(text_color);
                            if ui.selectable_label(is_active, label).clicked() {
                                if !is_open {
                                    to_open = Some(file_path.clone());
                                } else {
                                    // Switch to this file's tab
                                    for (idx, source) in state.sources.iter().enumerate() {
                                        if source.path == *file_path {
                                            state.set_active(idx);
                                            break;
                                        }
                                    }
                                }
                            }
                            
                            // Remove from workspace (not close tab)
                            if ui.small_button("×").on_hover_text("从工作区移除").clicked() {
                                to_remove_from_workspace = Some(i);
                            }
                        });
                    }
                    
                    // Open file if requested
                    if let Some(path) = to_open {
                        open_log_file(state, path);
                    }
                    
                    // Remove from workspace if requested
                    if let Some(i) = to_remove_from_workspace {
                        if let Some(ref mut ws) = state.current_workspace {
                            let path = ws.log_files[i].clone();
                            ws.remove_file(&path);
                            let ws_path = helpers::workspaces_dir().join(format!("{}.json", workspace_name));
                            let _ = ws.save(&ws_path);
                        }
                    }
                });
            }
        });
    
    // New workspace dialog
    if state.show_new_workspace_dialog {
        show_new_workspace_dialog(ctx, state);
    }
}

fn show_new_workspace_dialog(ctx: &egui::Context, state: &mut AppState) {
    egui::Window::new("新建工作区")
        .collapsible(false)
        .resizable(false)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("名称:");
                ui.text_edit_singleline(&mut state.new_workspace_name);
            });
            ui.horizontal(|ui| {
                if ui.button("创建").clicked() {
                    if !state.new_workspace_name.is_empty() {
                        let workspace = Workspace::new(state.new_workspace_name.clone());
                        let ws_path = helpers::workspaces_dir().join(format!("{}.json", workspace.name));
                        let _ = workspace.save(&ws_path);
                        state.available_workspaces.push(state.new_workspace_name.clone());
                        state.current_workspace = Some(workspace);
                        state.new_workspace_name.clear();
                        state.show_new_workspace_dialog = false;
                    }
                }
                if ui.button("取消").clicked() {
                    state.new_workspace_name.clear();
                    state.show_new_workspace_dialog = false;
                }
            });
        });
}

fn open_file_dialog(state: &mut AppState) {
    let dialog = rfd::FileDialog::new()
        .set_title("打开日志文件")
        .add_filter("日志文件", &["log", "txt", "out", "jsonl", "ndjson", "csv"])
        .add_filter("所有文件", &["*"]);

    if let Some(path) = dialog.pick_file() {
        // Auto-create default workspace if none exists
        if state.current_workspace.is_none() {
            let workspace = Workspace::new("默认工作区".to_string());
            let ws_path = helpers::workspaces_dir().join(format!("{}.json", workspace.name));
            let _ = workspace.save(&ws_path);
            if !state.available_workspaces.contains(&workspace.name) {
                state.available_workspaces.push(workspace.name.clone());
            }
            state.current_workspace = Some(workspace);
        }
        
        // Add file to workspace
        if let Some(ref mut workspace) = state.current_workspace {
            workspace.add_file(path.clone());
            let ws_path = helpers::workspaces_dir().join(format!("{}.json", workspace.name));
            let _ = workspace.save(&ws_path);
        }
        open_log_file(state, path);
    }
}

fn open_dir_dialog(state: &mut AppState) {
    let dialog = rfd::FileDialog::new()
        .set_title("打开日志目录");

    if let Some(path) = dialog.pick_folder() {
        if let Some(ref mut workspace) = state.current_workspace {
            workspace.add_dir(path.clone());
            let ws_path = helpers::workspaces_dir().join(format!("{}.json", workspace.name));
            let _ = workspace.save(&ws_path);
        }
        open_log_dir(state, &path);
    }
}

fn open_log_file(state: &mut AppState, path: PathBuf) {
    use logseek_core::domain::types::{LogFormat, SourceId};
    use logseek_core::source::backend::LogBackend;
    use crate::app::state::LogSource;
    use std::sync::Arc;
    
    let format = detect_format(&path);
    let source_id = SourceId::from_path(&path);
    let name = path.file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "未知文件".to_string());

    match LogBackend::open(&path, format, source_id.clone()) {
        Ok(backend) => {
            // Store file size for change detection
            if let Ok(metadata) = std::fs::metadata(&path) {
                state.file_sizes.insert(path.clone(), metadata.len());
            }
            
            let source = LogSource {
                id: source_id,
                name,
                path: path.clone(),
                format,
                backend: Arc::new(backend),
            };
            state.add_source(source);
            state.status_message = format!("已打开: {}", path.display());
        }
        Err(e) => {
            state.status_message = format!("打开文件失败: {}", e);
        }
    }
}

fn open_log_dir(state: &mut AppState, dir: &PathBuf) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension() {
                    let ext_str = ext.to_string_lossy().to_lowercase();
                    if ["log", "txt", "out", "jsonl", "ndjson", "csv"].contains(&ext_str.as_str()) {
                        open_log_file(state, path);
                    }
                }
            }
        }
    }
}

fn load_workspace(state: &mut AppState, name: &str) {
    let ws_path = helpers::workspaces_dir().join(format!("{}.json", name));
    
    match Workspace::load(&ws_path) {
        Ok(workspace) => {
            let file_count = workspace.log_files.len();
            
            // Clear state but DON'T open files automatically
            state.sources.clear();
            state.active_source = None;
            state.search_results.clear();
            state.search_query.pattern.clear();
            state.search_time_ms = 0.0;
            state.current_match_index = None;
            state.scroll_to_line = None;
            state.selected_search_result = None;
            state.show_search_results = false;
            state.filtered_lines.clear();
            state.filter_pattern.clear();
            state.filter_enabled = false;
            
            // Set workspace (files are listed but NOT opened)
            state.current_workspace = Some(workspace);
            
            state.status_message = format!("已加载工作区: {} ({} 个文件，点击文件名打开)", name, file_count);
        }
        Err(e) => {
            state.status_message = format!("加载工作区失败: {} - {}", name, e);
        }
    }
}

fn delete_workspace(state: &mut AppState, name: &str) {
    let ws_path = helpers::workspaces_dir().join(format!("{}.json", name));
    let _ = std::fs::remove_file(&ws_path);
    state.available_workspaces.retain(|n| n != name);
    if state.current_workspace.as_ref().map(|w| w.name == name).unwrap_or(false) {
        state.current_workspace = None;
    }
    state.status_message = format!("已删除工作区: {}", name);
}

fn detect_format(path: &PathBuf) -> logseek_core::domain::types::LogFormat {
    path.extension()
        .and_then(|ext| ext.to_str())
        .and_then(|ext| logseek_core::domain::types::LogFormat::from_extension(ext))
        .unwrap_or(logseek_core::domain::types::LogFormat::Text)
}

/// Close current workspace and clear all sources
fn close_workspace(state: &mut AppState) {
    state.sources.clear();
    state.active_source = None;
    state.current_workspace = None;
    state.search_results.clear();
    state.search_query.pattern.clear();
    state.search_time_ms = 0.0;
    state.current_match_index = None;
    state.scroll_to_line = None;
    state.selected_search_result = None;
    state.show_search_results = false;
    state.filtered_lines.clear();
    state.filter_pattern.clear();
    state.filter_enabled = false;
    state.status_message = "已关闭工作区".to_string();
}

/// Clear all log files from current workspace
fn clear_workspace_logs(state: &mut AppState) {
    if let Some(ref mut workspace) = state.current_workspace {
        workspace.log_files.clear();
        workspace.log_dirs.clear();
        workspace.active_source = None;
        
        // Save to disk
        let ws_path = helpers::workspaces_dir().join(format!("{}.json", workspace.name));
        let _ = workspace.save(&ws_path);
        
        // Clear state
        state.sources.clear();
        state.active_source = None;
        state.search_results.clear();
        state.search_query.pattern.clear();
        state.current_match_index = None;
        state.scroll_to_line = None;
        
        state.status_message = format!("已清空工作区 '{}' 的所有日志", workspace.name);
    }
}
