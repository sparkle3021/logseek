//! Application state management.

pub mod state;

use crate::app::state::AppState;
use crate::panels;
use crate::views;
use crate::widgets;
use crate::theme::colors::ThemeMode;

pub struct App {
    state: AppState,
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Load Chinese font
        Self::setup_chinese_font(&cc.egui_ctx);
        Self {
            state: AppState::new(),
        }
    }

    fn setup_chinese_font(ctx: &egui::Context) {
        // Try to load system Chinese fonts
        let font_data = Self::find_chinese_font();
        
        if let Some(data) = font_data {
            let mut fonts = egui::FontDefinitions::default();
            fonts.font_data.insert("chinese".to_owned(), std::sync::Arc::new(egui::FontData::from_owned(data)));
            
            // Add Chinese font to all font families
            for family in [egui::FontFamily::Proportional, egui::FontFamily::Monospace] {
                fonts.families
                    .entry(family)
                    .or_default()
                    .insert(0, "chinese".to_owned());
            }
            
            ctx.set_fonts(fonts);
        }
    }

    fn find_chinese_font() -> Option<Vec<u8>> {
        // Common Chinese font paths on Windows
        let font_paths = [
            "C:/Windows/Fonts/msyh.ttc",      // 微软雅黑
            "C:/Windows/Fonts/msyhbd.ttc",     // 微软雅黑 Bold
            "C:/Windows/Fonts/simhei.ttf",     // 黑体
            "C:/Windows/Fonts/simsun.ttc",     // 宋体
            "C:/Windows/Fonts/simkai.ttf",     // 楷体
        ];

        for path in &font_paths {
            if let Ok(data) = std::fs::read(path) {
                return Some(data);
            }
        }

        None
    }

    fn apply_theme(ctx: &egui::Context, theme: ThemeMode) {
        let colors = theme.colors();
        let mut visuals = match theme {
            ThemeMode::Dark => egui::Visuals::dark(),
            ThemeMode::Light => egui::Visuals::light(),
        };
        
        // Customize colors
        visuals.window_fill = colors.bg;
        visuals.panel_fill = colors.bg;
        visuals.extreme_bg_color = colors.bg_panel;
        visuals.faint_bg_color = colors.bg_panel;
        
        // Text colors
        visuals.override_text_color = Some(colors.text_primary);
        
        // Selection colors
        visuals.selection.bg_fill = colors.selected;
        visuals.selection.stroke = egui::Stroke::new(1.0, colors.accent);
        
        // Hyperlink color
        visuals.hyperlink_color = colors.accent;
        
        // Widget colors
        visuals.widgets.inactive.bg_fill = colors.bg_panel;
        visuals.widgets.inactive.bg_stroke = egui::Stroke::new(0.5, colors.text_secondary);
        visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, colors.text_primary);
        
        visuals.widgets.hovered.bg_fill = colors.selected;
        visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.0, colors.accent);
        visuals.widgets.hovered.fg_stroke = egui::Stroke::new(1.5, colors.text_primary);
        
        visuals.widgets.active.bg_fill = colors.accent;
        visuals.widgets.active.bg_stroke = egui::Stroke::new(1.0, colors.accent);
        visuals.widgets.active.fg_stroke = egui::Stroke::new(2.0, colors.text_primary);
        
        visuals.widgets.noninteractive.bg_fill = colors.bg;
        visuals.widgets.noninteractive.bg_stroke = egui::Stroke::new(0.5, colors.text_secondary);
        visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, colors.text_primary);
        
        ctx.set_visuals(visuals);
    }

    fn show_update_dialog(ctx: &egui::Context, state: &mut AppState) {
        egui::Window::new("文件更新提示")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .fixed_size([300.0, 120.0])
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(10.0);
                    ui.label(
                        egui::RichText::new("检测到文件存在更新，是否重新加载？")
                            .size(14.0)
                    );
                    ui.add_space(20.0);
                    ui.horizontal(|ui| {
                        ui.add_space(20.0);
                        if ui.add_sized([100.0, 30.0], egui::Button::new("是")).clicked() {
                            state.accept_file_reload();
                        }
                        ui.add_space(30.0);
                        if ui.add_sized([100.0, 30.0], egui::Button::new("否")).clicked() {
                            state.decline_file_reload();
                        }
                    });
                    ui.add_space(10.0);
                });
            });
    }

    fn show_clear_confirm_dialog(ctx: &egui::Context, state: &mut AppState) {
        let source_name = state.active_source
            .and_then(|i| state.sources.get(i))
            .map(|s| s.name.clone())
            .unwrap_or_else(|| "未知文件".to_string());

        egui::Window::new("确认清空")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .fixed_size([350.0, 140.0])
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(10.0);
                    ui.label(
                        egui::RichText::new(format!("确定要清空 '{}' 的内容吗？", source_name))
                            .size(14.0)
                    );
                    ui.add_space(5.0);
                    ui.label(
                        egui::RichText::new("此操作不可撤销！")
                            .size(12.0)
                            .color(egui::Color32::from_rgb(200, 50, 50))
                    );
                    ui.add_space(20.0);
                    ui.horizontal(|ui| {
                        ui.add_space(20.0);
                        if ui.add_sized([100.0, 30.0], egui::Button::new("确定")).clicked() {
                            state.confirm_clear_log();
                        }
                        ui.add_space(30.0);
                        if ui.add_sized([100.0, 30.0], egui::Button::new("取消")).clicked() {
                            state.cancel_clear_log();
                        }
                    });
                    ui.add_space(10.0);
                });
            });
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Auto-refresh logic (tail -f mode)
        if self.state.auto_refresh {
            self.state.auto_refresh_if_needed();
            ctx.request_repaint_after(std::time::Duration::from_secs(1));
        } else {
            // Check for file changes when auto-refresh is OFF
            self.state.check_file_changes_for_dialog();
            ctx.request_repaint_after(std::time::Duration::from_secs(5));
        }
        
        // Apply theme
        Self::apply_theme(ctx, self.state.theme);
        
        // Show file update dialog if needed
        if self.state.show_update_dialog {
            Self::show_update_dialog(ctx, &mut self.state);
        }
        
        // Show clear confirmation dialog if needed
        if self.state.show_clear_confirm_dialog {
            Self::show_clear_confirm_dialog(ctx, &mut self.state);
        }
        
        // Show font settings window if enabled
        panels::font_settings::show(ctx, &mut self.state);
        
        // Show search results panel if enabled (right side, before main content)
        if self.state.show_search_results && !self.state.search_results.is_empty() {
            panels::search_results::show(ctx, &mut self.state);
        }
        
        panels::toolbar::show(ctx, &mut self.state);
        panels::status_bar::show(ctx, &self.state);
        // Use workspace panel instead of sources panel
        panels::workspace::show(ctx, &mut self.state);
        
        // Show tab bar for multiple files
        widgets::tabs::show(ctx, &mut self.state);

        if self.state.is_structured_format() && self.state.show_structured_view {
            views::structured_view::show(ctx, &mut self.state);
        } else {
            views::log_view::show(ctx, &mut self.state);
        }
    }
}
