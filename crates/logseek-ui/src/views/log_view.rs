//! Log view with virtual scrolling and search highlighting.

use crate::app::state::AppState;
use crate::theme::{colors, fonts};
use crate::theme::colors::ThemeMode;
use egui::{CentralPanel, RichText, ScrollArea, TextFormat, Vec2, FontId, Color32};
use egui::text::LayoutJob;
use regex::Regex;

/// Zebra stripe colors - subtle alternating backgrounds
fn zebra_color(index: usize, theme_colors: &colors::ColorScheme) -> Color32 {
    if index % 2 == 0 {
        theme_colors.bg // Even rows - base background
    } else {
        // Odd rows - slightly different shade
        let bg = theme_colors.bg.to_array();
        let offset = 8u8;
        Color32::from_rgba_premultiplied(
            bg[0].saturating_add(offset),
            bg[1].saturating_add(offset),
            bg[2].saturating_add(offset),
            bg[3],
        )
    }
}

pub fn show(ctx: &egui::Context, state: &mut AppState) {
    let theme_colors = state.theme.colors();
    let font_size = state.font_config.mono_size;
    let row_height = state.font_config.row_height();
    let line_num_width = 60.0; // Width for line numbers
    
    CentralPanel::default().show(ctx, |ui| {
        if state.sources.is_empty() {
            ui.centered_and_justified(|ui| {
                ui.label(RichText::new("打开日志文件以开始").size(fonts::FONT_SIZE_HEADING).color(theme_colors.text_secondary));
            });
            return;
        }

        // Read ALL values we need before borrowing backend
        let current_match_line = state.current_match_index
            .and_then(|idx| state.search_results.get(idx))
            .map(|hit| hit.record_index);
        let search_pattern = state.search_query.pattern.clone();
        let is_regex = state.search_query.is_regex;
        let case_insensitive = state.search_query.case_insensitive;
        let whole_word = state.search_query.whole_word;
        let should_scroll_to = state.scroll_to_line.take();
        let filter_enabled = state.filter_enabled && !state.filter_pattern.is_empty();
        let filtered_lines = state.filtered_lines.clone();
        
        let backend = match state.active_backend() {
            Some(b) => b,
            None => return,
        };

        let total = backend.record_count() as usize;
        if total == 0 {
            ui.centered_and_justified(|ui| {
                ui.label(RichText::new("日志文件为空").size(fonts::FONT_SIZE_HEADING).color(theme_colors.text_secondary));
            });
            return;
        }

        // Calculate visible lines based on filter
        let visible_lines: Vec<u64> = if filter_enabled {
            filtered_lines.clone()
        } else {
            (0..total as u64).collect()
        };
        let visible_count = visible_lines.len();
        
        let spacing = ui.spacing().item_spacing.y;
        
        // Build scroll area - both horizontal and vertical for long lines
        let mut scroll_area = ScrollArea::both()
            .auto_shrink([false, false]);
        
        // If we have a target line, calculate pixel offset and set it
        if let Some(target_line) = should_scroll_to {
            // Find the index of target_line in visible_lines
            let visible_index = visible_lines.iter().position(|&l| l == target_line).unwrap_or(0);
            let target_offset = visible_index as f32 * (row_height + spacing) - spacing;
            scroll_area = scroll_area.vertical_scroll_offset(target_offset);
        }
        
        let font_id = FontId::monospace(font_size);
        let line_num_font_id = FontId::monospace(font_size * 0.9);
        let line_num_color = Color32::from_rgb(100, 100, 120);
        
        // Show filtered lines
        scroll_area.show_rows(ui, row_height, visible_count, |ui, row_range| {
            for visible_idx in row_range {
                if visible_idx >= visible_lines.len() {
                    break;
                }
                let actual_line = visible_lines[visible_idx] as usize;
                
                if let Some(record) = backend.record_at(actual_line as u64) {
                    let is_current_match = current_match_line == Some(actual_line as u64);
                    let is_hit = state.search_results.iter().any(|h| h.record_index == actual_line as u64);
                    
                    // Determine background color
                    let bg_color = if is_current_match {
                        theme_colors.selected // Current match - highlight
                    } else if is_hit && !search_pattern.is_empty() {
                        // Search hit - subtle highlight
                        Color32::from_rgba_premultiplied(
                            theme_colors.highlight_bg.r(),
                            theme_colors.highlight_bg.g(),
                            theme_colors.highlight_bg.b(),
                            40,
                        )
                    } else {
                        zebra_color(visible_idx, &theme_colors) // Zebra stripe
                    };
                    
                    // Render row with background
                    let frame = egui::Frame::NONE
                        .fill(bg_color)
                        .inner_margin(egui::Margin::symmetric(4, 1));
                    
                    frame.show(ui, |ui| {
                        // Use horizontal layout with no wrapping
                        ui.horizontal(|ui| {
                            // Line number - right-aligned, muted color (show actual line number)
                            let line_num = format!("{:>6}", actual_line + 1);
                            ui.label(
                                RichText::new(line_num)
                                    .font(line_num_font_id.clone())
                                    .color(line_num_color),
                            );
                            
                            // Filter indicator
                            if filter_enabled {
                                ui.label(
                                    RichText::new("│")
                                        .font(line_num_font_id.clone())
                                        .color(theme_colors.accent),
                                );
                            } else {
                                ui.separator();
                            }
                            
                            // Log content - use LayoutJob to prevent wrapping
                            if is_hit && !search_pattern.is_empty() {
                                ui.label(highlight_text(&record.raw, &search_pattern, is_regex, case_insensitive, whole_word, ui, &theme_colors, &font_id));
                            } else {
                                // Create a non-wrapping layout job
                                let mut job = LayoutJob::default();
                                job.append(
                                    &record.raw,
                                    0.0,
                                    TextFormat::simple(font_id.clone(), theme_colors.text_primary),
                                );
                                // Prevent wrapping by setting wrap to None
                                job.wrap.max_width = f32::INFINITY;
                                ui.label(job);
                            }
                        });
                    });
                }
            }
        });
    });
}

fn highlight_text(text: &str, pattern: &str, is_regex: bool, case_insensitive: bool, whole_word: bool, ui: &egui::Ui, theme_colors: &colors::ColorScheme, font_id: &FontId) -> LayoutJob {
    let mut job = LayoutJob::default();
    let default_color = theme_colors.text_primary;
    let highlight_color = theme_colors.highlight;

    // Prevent wrapping - allow horizontal scrolling for long lines
    job.wrap.max_width = f32::INFINITY;

    if pattern.is_empty() {
        job.append(text, 0.0, TextFormat::simple(font_id.clone(), default_color));
        return job;
    }

    // Build regex pattern with proper options
    let base_pattern = if is_regex {
        pattern.to_string()
    } else {
        regex::escape(pattern)
    };
    let word_pattern = if whole_word && !pattern.is_empty() {
        format!(r"\b{}\b", base_pattern)
    } else {
        base_pattern
    };
    let regex_pattern = if case_insensitive {
        format!("(?i){}", word_pattern)
    } else {
        word_pattern
    };
    
    let matches: Vec<(usize, usize)> = Regex::new(&regex_pattern)
        .map(|re| re.find_iter(text).map(|m| (m.start(), m.end())).collect())
        .unwrap_or_default();

    if matches.is_empty() {
        job.append(text, 0.0, TextFormat::simple(font_id.clone(), default_color));
        return job;
    }

    let mut last_end = 0;
    for (start, end) in matches {
        if start > last_end {
            job.append(&text[last_end..start], 0.0, TextFormat::simple(font_id.clone(), default_color));
        }
        let mut fmt = TextFormat::simple(font_id.clone(), highlight_color);
        fmt.background = theme_colors.highlight_bg;
        job.append(&text[start..end], 0.0, fmt);
        last_end = end;
    }

    if last_end < text.len() {
        job.append(&text[last_end..], 0.0, TextFormat::simple(font_id.clone(), default_color));
    }

    job
}
