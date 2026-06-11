//! Font settings panel.

use crate::app::state::AppState;
use crate::theme::{colors, fonts};
use crate::theme::fonts::FontConfig;
use egui::{RichText, Window};

pub fn show(ctx: &egui::Context, state: &mut AppState) {
    if !state.show_font_settings {
        return;
    }

    let theme_colors = state.theme.colors();
    let mut open = state.show_font_settings;

    Window::new("🔤 字体设置")
        .open(&mut open)
        .resizable(true)
        .default_width(300.0)
        .show(ctx, |ui| {
            // Preset buttons
            ui.label(RichText::new("预设方案").size(fonts::FONT_SIZE_BODY));
            ui.horizontal_wrapped(|ui| {
                for (name, config) in FontConfig::presets() {
                    if ui.button(name).clicked() {
                        state.font_config = config;
                    }
                }
            });
            
            ui.separator();
            
            // Body font size slider
            ui.label(RichText::new("正文字体大小").size(fonts::FONT_SIZE_SMALL));
            ui.add(egui::Slider::new(&mut state.font_config.body_size, 10.0..=24.0).suffix(" px"));
            
            // Mono font size slider
            ui.label(RichText::new("等宽字体大小").size(fonts::FONT_SIZE_SMALL));
            ui.add(egui::Slider::new(&mut state.font_config.mono_size, 10.0..=24.0).suffix(" px"));
            
            // Line height multiplier slider
            ui.label(RichText::new("行高倍数").size(fonts::FONT_SIZE_SMALL));
            ui.add(egui::Slider::new(&mut state.font_config.line_height_multiplier, 1.0..=2.0).step_by(0.05));
            
            ui.separator();
            
            // Preview
            ui.label(RichText::new("预览效果").size(fonts::FONT_SIZE_BODY));
            
            let preview_frame = egui::Frame::NONE
                .fill(theme_colors.bg_panel)
                .inner_margin(8)
                .stroke(egui::Stroke::new(1.0, theme_colors.text_secondary));
            
            preview_frame.show(ui, |ui| {
                ui.label(
                    RichText::new("正文字体示例 Body Text")
                        .size(state.font_config.body_size)
                        .color(theme_colors.text_primary),
                );
                ui.label(
                    RichText::new("等宽字体示例 Monospace")
                        .size(state.font_config.mono_size)
                        .color(theme_colors.text_primary),
                );
                ui.label(
                    RichText::new("2024-01-15 10:30:45 INFO [main] Application started")
                        .size(state.font_config.mono_size)
                        .color(theme_colors.info),
                );
                ui.label(
                    RichText::new("2024-01-15 10:30:46 ERROR [db] Connection failed")
                        .size(state.font_config.mono_size)
                        .color(theme_colors.error),
                );
            });
            
            ui.separator();
            
            // Current settings display
            ui.label(
                RichText::new(format!(
                    "行高: {:.1}px (字体 {:.1}px × {:.2})",
                    state.font_config.row_height(),
                    state.font_config.mono_size,
                    state.font_config.line_height_multiplier
                ))
                .size(fonts::FONT_SIZE_SMALL)
                .color(theme_colors.text_secondary),
            );
        });

    state.show_font_settings = open;
}
