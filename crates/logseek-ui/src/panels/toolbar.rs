//! Toolbar panel.

use crate::app::state::AppState;
use crate::theme::{colors, fonts};
use crate::theme::colors::ThemeMode;
use egui::{RichText, TopBottomPanel, Id};

pub fn show(ctx: &egui::Context, state: &mut AppState) {
    TopBottomPanel::top("toolbar").show(ctx, |ui| {
        // Ctrl+F to focus search
        let search_input_id = Id::new("search_input");
        if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::F)) {
            ui.memory_mut(|mem| mem.request_focus(search_input_id));
        }
        
        ui.horizontal(|ui| {
            // Search section - always visible
            let response = ui.add(
                egui::TextEdit::singleline(&mut state.search_query.pattern)
                    .id(search_input_id)
                    .hint_text("搜索... (Ctrl+F)")
                    .desired_width(200.0)
            );
            if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                state.search_current();
            }
            
            // Case insensitive toggle
            // Default: case_insensitive = true (button NOT pressed)
            // When clicked: case_insensitive = false (button pressed, case sensitive mode)
            let theme_colors = state.theme.colors();
            let is_case_sensitive = !state.search_query.case_insensitive;
            let case_btn = egui::Button::new("Aa")
                .selected(is_case_sensitive);
            if ui.add(case_btn).on_hover_text("区分大小写").clicked() {
                state.search_query.case_insensitive = !state.search_query.case_insensitive;
            }
            
            // Whole word toggle
            let whole_word_btn = egui::Button::new("W")
                .selected(state.search_query.whole_word);
            if ui.add(whole_word_btn).on_hover_text("全词匹配").clicked() {
                state.search_query.whole_word = !state.search_query.whole_word;
            }
            
            ui.checkbox(&mut state.search_query.is_regex, "正则");
            
            if ui.button("搜索").clicked() {
                state.search_current();
            }
            if ui.button("搜索工作区").clicked() {
                state.search_workspace();
                if !state.search_results.is_empty() {
                    state.show_search_results = true;
                }
            }
            
            // Navigation buttons - always visible
            ui.separator();
            let prev_enabled = !state.search_results.is_empty();
            if ui.add_enabled(prev_enabled, egui::Button::new("◀")).clicked() {
                state.navigate_previous();
            }
            let match_text = if state.search_results.is_empty() {
                "0/0".to_string()
            } else {
                match state.current_match_index {
                    Some(idx) => format!("{}/{}", idx + 1, state.search_results.len()),
                    None => format!("0/{}", state.search_results.len()),
                }
            };
            ui.label(RichText::new(match_text).size(fonts::FONT_SIZE_SMALL).color(state.theme.colors().text_secondary));
            let next_enabled = !state.search_results.is_empty();
            if ui.add_enabled(next_enabled, egui::Button::new("▶")).clicked() {
                state.navigate_next();
            }
            
            ui.separator();
            
            // Filter section - always visible
            let filter_response = ui.text_edit_singleline(&mut state.filter_pattern);
            if filter_response.changed() && state.filter_enabled {
                state.apply_filter();
            }
            
            // Checkbox - apply filter immediately when toggled
            let filter_checkbox = ui.checkbox(&mut state.filter_enabled, "过滤");
            if filter_checkbox.changed() {
                state.apply_filter();
            }
            
            let regex_checkbox = ui.checkbox(&mut state.filter_is_regex, "正则");
            if regex_checkbox.changed() && state.filter_enabled {
                state.apply_filter();
            }
            
            if ui.button("应用").clicked() {
                state.apply_filter();
            }
            if ui.button("清除").on_hover_text("清除所有搜索和过滤条件").clicked() {
                // Clear search
                state.search_query.pattern.clear();
                state.search_results.clear();
                state.search_time_ms = 0.0;
                state.current_match_index = None;
                state.scroll_to_line = None;
                state.selected_search_result = None;
                state.show_search_results = false;
                
                // Clear filter
                state.filter_pattern.clear();
                state.filter_enabled = false;
                state.filter_is_regex = false;
                state.filtered_lines.clear();
                
                // Reset search options to defaults
                state.search_query.is_regex = false;
                state.search_query.case_insensitive = true;
                state.search_query.whole_word = false;
                
                state.status_message = "已清除所有条件".to_string();
            }
            
            ui.separator();
            
            // Results toggle - always visible
            let results_btn_text = if state.show_search_results { "隐藏结果" } else { "显示结果" };
            let results_enabled = !state.search_results.is_empty();
            if ui.add_enabled(results_enabled, egui::Button::new(results_btn_text)).clicked() {
                state.show_search_results = !state.show_search_results;
            }
            
            // Auto-refresh toggle (like tail -f)
            let auto_refresh_btn = egui::Button::new("自动刷新")
                .selected(state.auto_refresh);
            if ui.add(auto_refresh_btn).on_hover_text("自动刷新 (tail -f)").clicked() {
                state.auto_refresh = !state.auto_refresh;
            }
            
            // Manual refresh button
            if ui.button("刷新").on_hover_text("手动刷新").clicked() {
                state.refresh_current();
            }
            
            // Copy all button
            if ui.button("复制").on_hover_text("复制全部内容到剪贴板").clicked() {
                state.copy_all_to_clipboard();
            }
            
            // Clear log button
            if ui.button("清空").on_hover_text("清空当前日志文件内容").clicked() {
                state.request_clear_log();
            }
            
            ui.separator();
            if ui.button("字体").clicked() {
                state.show_font_settings = !state.show_font_settings;
            }
            ui.separator();
            
            let theme_text = match state.theme {
                ThemeMode::Dark => "切换亮色",
                ThemeMode::Light => "切换暗色",
            };
            if ui.button(theme_text).clicked() {
                state.toggle_theme();
            }
        });
    });
}
