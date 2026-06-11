//! Sources panel.

use crate::app::state::{AppState, LogSource};
use crate::theme::{colors, fonts};
use crate::theme::colors::ThemeMode;
use egui::{RichText, SidePanel};
use logseek_core::domain::types::{LogFormat, SourceId};
use logseek_core::source::backend::LogBackend;
use std::path::PathBuf;
use std::sync::Arc;

pub fn show(ctx: &egui::Context, state: &mut AppState) {
    let theme_colors = state.theme.colors();
    
    SidePanel::left("sources_panel")
        .resizable(true)
        .default_width(fonts::PANEL_WIDTH_SOURCES)
        .width_range(150.0..=400.0)
        .show(ctx, |ui| {
            ui.heading(RichText::new("日志源").size(fonts::FONT_SIZE_HEADING));
            ui.separator();

            if ui.button("📂 打开文件").clicked() {
                open_file_dialog(state);
            }
            ui.separator();

            egui::ScrollArea::vertical().show(ui, |ui| {
                let mut to_remove = None;
                let mut to_activate = None;
                for (i, source) in state.sources.iter().enumerate() {
                    let is_active = state.active_source == Some(i);
                    let text_color = if is_active { theme_colors.accent } else { theme_colors.text_primary };
                    ui.horizontal(|ui| {
                        let label = RichText::new(&source.name).color(text_color);
                        if ui.selectable_label(is_active, label).clicked() {
                            to_activate = Some(i);
                        }
                        if ui.button("❌").clicked() {
                            to_remove = Some(i);
                        }
                    });
                }
                if let Some(i) = to_activate {
                    state.set_active(i);
                }
                if let Some(i) = to_remove {
                    state.remove_source(i);
                }
            });

            ui.separator();
            ui.label(
                RichText::new(format!("{} 个日志源", state.sources.len()))
                    .size(fonts::FONT_SIZE_SMALL)
                    .color(theme_colors.text_secondary),
            );
        });
}

fn open_file_dialog(state: &mut AppState) {
    let dialog = rfd::FileDialog::new()
        .set_title("打开日志文件")
        .add_filter("日志文件", &["log", "txt", "out", "jsonl", "ndjson", "csv"])
        .add_filter("所有文件", &["*"]);

    if let Some(path) = dialog.pick_file() {
        open_log_file(state, path);
    }
}

fn open_log_file(state: &mut AppState, path: PathBuf) {
    let format = detect_format(&path);
    let source_id = SourceId::from_path(&path);
    let name = path.file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "未知文件".to_string());

    match LogBackend::open(&path, format, source_id.clone()) {
        Ok(backend) => {
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

fn detect_format(path: &PathBuf) -> LogFormat {
    path.extension()
        .and_then(|ext| ext.to_str())
        .and_then(|ext| LogFormat::from_extension(ext))
        .unwrap_or(LogFormat::Text)
}
