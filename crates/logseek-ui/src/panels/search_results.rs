//! Search results panel - displays aggregated search results from all sources.

use crate::app::state::AppState;
use crate::theme::{colors, fonts};
use crate::theme::colors::ThemeMode;
use egui::{RichText, SidePanel, ScrollArea};
use logseek_core::domain::types::SearchHit;

/// Search result with additional context for display
#[derive(Clone)]
pub struct SearchResultItem {
    pub hit: SearchHit,
    pub source_name: String,
    pub line_number: u64,
    pub content: String,
    pub match_start: usize,
    pub match_end: usize,
}

pub fn show(ctx: &egui::Context, state: &mut AppState) {
    // Build search result items if we have results
    let result_items = build_result_items(state);
    
    SidePanel::right("search_results_panel")
        .resizable(true)
        .default_width(350.0)
        .width_range(200.0..=600.0)
        .show(ctx, |ui| {
            let theme_colors = state.theme.colors();
            
            // Header
            ui.horizontal(|ui| {
                ui.heading(RichText::new("搜索结果").size(fonts::FONT_SIZE_HEADING));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("✖").clicked() {
                        state.show_search_results = false;
                    }
                });
            });
            ui.separator();
            
            // Summary
            if !result_items.is_empty() {
                ui.label(
                    RichText::new(format!("共 {} 条匹配", result_items.len()))
                        .size(fonts::FONT_SIZE_SMALL)
                        .color(theme_colors.text_secondary),
                );
                ui.separator();
            }
            
            // Results list
            if result_items.is_empty() {
                ui.centered_and_justified(|ui| {
                    ui.label(
                        RichText::new("无搜索结果")
                            .size(fonts::FONT_SIZE_BODY)
                            .color(theme_colors.text_secondary),
                    );
                });
                return;
            }
            
            ScrollArea::vertical().show(ui, |ui| {
                for (index, item) in result_items.iter().enumerate() {
                    let is_selected = state.selected_search_result == Some(index);
                    
                    // Create a frame for each result
                    let bg_color = if is_selected {
                        theme_colors.selected
                    } else if index % 2 == 0 {
                        theme_colors.bg_panel
                    } else {
                        theme_colors.bg
                    };
                    
                    // Use a button-like approach for reliable click detection
                    let frame = egui::Frame::NONE
                        .fill(bg_color)
                        .inner_margin(egui::Margin::symmetric(10, 8))
                        .outer_margin(egui::Margin::symmetric(2, 2));
                    
                    let response = frame.show(ui, |ui| {
                        // Set minimum width to ensure clickable area
                        ui.set_min_width(ui.available_width());
                        
                        // Source name and line number header
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new(&item.source_name)
                                    .size(fonts::FONT_SIZE_SMALL)
                                    .strong()
                                    .color(theme_colors.accent),
                            );
                            ui.label(
                                RichText::new(format!("行 {}", item.line_number + 1))
                                    .size(fonts::FONT_SIZE_SMALL)
                                    .color(theme_colors.text_secondary),
                            );
                        });
                        
                        ui.add_space(4.0);
                        
                        // Content with highlight - show full content
                        ui.label(highlight_match(
                            &item.content,
                            item.match_start,
                            item.match_end,
                            &theme_colors,
                            ui,
                        ));
                    }).response;
                    
                    // Make the entire frame area clickable
                    let click_response = ui.interact(
                        response.rect,
                        egui::Id::new(("search_result_item", index)),
                        egui::Sense::click()
                    );
                    
                    // Handle click
                    if click_response.clicked() {
                        state.selected_search_result = Some(index);
                        state.jump_to_result(item);
                    }
                    
                    // Hover effect - change cursor
                    if click_response.hovered() {
                        ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::PointingHand);
                    }
                }
            });
        });
}

fn build_result_items(state: &AppState) -> Vec<SearchResultItem> {
    let mut items = Vec::new();
    
    for hit in &state.search_results {
        // Find the source for this hit
        let source = state.sources.iter().find(|s| s.id == hit.source_id);
        let source_name = source.map(|s| s.name.clone()).unwrap_or_else(|| "Unknown".to_string());
        
        // Get the record content
        if let Some(source) = source {
            if let Some(record) = source.backend.record_at(hit.record_index) {
                let pattern = &state.search_query.pattern;
                let content = &record.raw;
                
                // Find match position in content
                let (match_start, match_end) = if !pattern.is_empty() {
                    if state.search_query.is_regex {
                        regex::Regex::new(pattern)
                            .ok()
                            .and_then(|re| re.find(content))
                            .map(|m| (m.start(), m.end()))
                            .unwrap_or((0, 0))
                    } else {
                        content.to_lowercase().find(&pattern.to_lowercase())
                            .map(|pos| (pos, pos + pattern.len()))
                            .unwrap_or((0, 0))
                    }
                } else {
                    (0, 0)
                };
                
                items.push(SearchResultItem {
                    hit: hit.clone(),
                    source_name,
                    line_number: hit.record_index,
                    content: content.clone(),
                    match_start,
                    match_end,
                });
            }
        }
    }
    
    items
}

fn highlight_match(text: &str, match_start: usize, match_end: usize, theme_colors: &colors::ColorScheme, ui: &egui::Ui) -> egui::text::LayoutJob {
    let mut job = egui::text::LayoutJob::default();
    let font_id = egui::TextStyle::Monospace.resolve(ui.style());
    
    if match_start >= match_end || match_end > text.len() {
        job.append(text, 0.0, egui::TextFormat::simple(font_id, theme_colors.text_primary));
        return job;
    }
    
    // Before match
    if match_start > 0 {
        job.append(&text[..match_start], 0.0, egui::TextFormat::simple(font_id.clone(), theme_colors.text_primary));
    }
    
    // Match highlight
    let mut highlight_fmt = egui::TextFormat::simple(font_id.clone(), theme_colors.highlight);
    highlight_fmt.background = theme_colors.highlight_bg;
    job.append(&text[match_start..match_end], 0.0, highlight_fmt);
    
    // After match
    if match_end < text.len() {
        job.append(&text[match_end..], 0.0, egui::TextFormat::simple(font_id, theme_colors.text_primary));
    }
    
    job
}
