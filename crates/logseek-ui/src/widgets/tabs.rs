//! Tab bar widget for managing multiple log file tabs.

use crate::app::state::AppState;
use crate::theme::{colors, fonts};
use egui::{RichText, CentralPanel, ScrollArea};

/// Show tab bar above the main content area
pub fn show(ctx: &egui::Context, state: &mut AppState) {
    if state.sources.is_empty() {
        return;
    }

    egui::TopBottomPanel::top("tab_bar")
        .exact_height(32.0)
        .show(ctx, |ui| {
            let theme_colors = state.theme.colors();
            
            ui.horizontal(|ui| {
                // Scrollable tab area
                ScrollArea::horizontal()
                    .max_width(ui.available_width() - 50.0)  // Leave space for actions
                    .show(ui, |ui| {
                        ui.spacing_mut().item_spacing.x = 0.0;
                        
                        let mut to_activate = None;
                        let mut to_close = None;
                        
                        for (i, source) in state.sources.iter().enumerate() {
                            let is_active = state.active_source == Some(i);
                            
                            // Tab styling
                            let (bg_color, text_color, border_color) = if is_active {
                                (
                                    theme_colors.bg_panel,
                                    theme_colors.accent,
                                    theme_colors.accent,
                                )
                            } else {
                                (
                                    theme_colors.bg,
                                    theme_colors.text_secondary,
                                    theme_colors.bg,
                                )
                            };
                            
                            // Tab frame
                            let frame = egui::Frame::NONE
                                .fill(bg_color)
                                .stroke(egui::Stroke::new(1.0, border_color))
                                .inner_margin(egui::Margin::symmetric(8, 4));
                            
                            let tab_response = frame.show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    // File name
                                    let label = RichText::new(&source.name)
                                        .size(fonts::FONT_SIZE_SMALL)
                                        .color(text_color);
                                    
                                    if ui.selectable_label(false, label).clicked() {
                                        to_activate = Some(i);
                                    }
                                    
                                    // Close button
                                    if ui.small_button("×").on_hover_text("关闭").clicked() {
                                        to_close = Some(i);
                                    }
                                });
                            });
                        }
                        
                        // Apply actions
                        if let Some(i) = to_activate {
                            state.set_active(i);
                        }
                        if let Some(i) = to_close {
                            state.remove_source(i);
                        }
                    });
                
                // Tab actions
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // Close all tabs button
                    if ui.small_button("×").on_hover_text("关闭所有").clicked() {
                        state.sources.clear();
                        state.active_source = None;
                        state.search_results.clear();
                        state.current_match_index = None;
                    }
                });
            });
        });
}
