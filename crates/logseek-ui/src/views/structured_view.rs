//! Structured view for JSONL and CSV (table mode).

use crate::app::state::AppState;
use crate::theme::{colors, fonts};
use crate::theme::colors::ThemeMode;
use egui::{CentralPanel, RichText};
use egui_extras::{Column, TableBuilder};

pub fn show(ctx: &egui::Context, state: &mut AppState) {
    let theme_colors = state.theme.colors();
    
    CentralPanel::default().show(ctx, |ui| {
        let backend = match state.active_backend() {
            Some(b) => b,
            None => return,
        };

        let field_names: Vec<String> = backend
            .record_at(0)
            .map(|r| r.fields.iter().map(|f| f.key.clone()).collect())
            .unwrap_or_default();

        if field_names.is_empty() {
            ui.label(RichText::new("未检测到字段").color(theme_colors.text_secondary));
            return;
        }

        let total = backend.record_count() as usize;

        let mut table = TableBuilder::new(ui)
            .striped(true)
            .resizable(true)
            .cell_layout(egui::Layout::left_to_right(egui::Align::Center));

        for _ in &field_names {
            table = table.column(Column::auto());
        }

        table
            .header(fonts::ROW_HEIGHT, |mut header| {
                for name in &field_names {
                    header.col(|ui| { ui.strong(RichText::new(name).size(fonts::FONT_SIZE_SMALL)); });
                }
            })
            .body(|mut body| {
                body.rows(fonts::ROW_HEIGHT, total, |mut row| {
                    let index = row.index();
                    if let Some(record) = backend.record_at(index as u64) {
                        for field_name in &field_names {
                            row.col(|ui| {
                                let value = record.fields.iter().find(|f| f.key == *field_name).map(|f| f.value.as_str()).unwrap_or("");
                                ui.label(RichText::new(value).size(fonts::FONT_SIZE_MONO).color(theme_colors.text_primary));
                            });
                        }
                    }
                });
            });
    });
}
