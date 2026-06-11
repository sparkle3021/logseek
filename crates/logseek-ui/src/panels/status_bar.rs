//! Status bar panel.

use crate::app::state::AppState;
use crate::theme::{colors, fonts};
use crate::theme::colors::ThemeMode;
use egui::{RichText, TopBottomPanel};

pub fn show(ctx: &egui::Context, state: &AppState) {
    let theme_colors = state.theme.colors();
    
    TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.label(RichText::new(format!("日志源: {}", state.sources.len())).size(fonts::FONT_SIZE_SMALL).color(theme_colors.text_secondary));
            ui.separator();
            ui.label(RichText::new(format!("记录数: {}", state.total_records())).size(fonts::FONT_SIZE_SMALL).color(theme_colors.text_secondary));

            if state.search_time_ms > 0.0 {
                ui.separator();
                ui.label(RichText::new(format!("搜索耗时: {:.1}ms", state.search_time_ms)).size(fonts::FONT_SIZE_SMALL).color(theme_colors.text_secondary));
            }

            if !state.search_results.is_empty() {
                ui.separator();
                ui.label(RichText::new(format!("匹配: {}", state.search_results.len())).size(fonts::FONT_SIZE_SMALL).color(theme_colors.info));
            }

            if !state.status_message.is_empty() {
                ui.separator();
                ui.label(RichText::new(&state.status_message).size(fonts::FONT_SIZE_SMALL).color(theme_colors.text_secondary));
            }
        });
    });
}
