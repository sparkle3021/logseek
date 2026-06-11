//! UI color theme.

use egui::Color32;

/// Theme mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeMode {
    Dark,
    Light,
}

/// Color scheme for the current theme
pub struct ColorScheme {
    pub bg: Color32,
    pub bg_panel: Color32,
    pub text_primary: Color32,
    pub text_secondary: Color32,
    pub accent: Color32,
    pub highlight: Color32,
    pub highlight_bg: Color32,
    pub error: Color32,
    pub warn: Color32,
    pub info: Color32,
    pub debug: Color32,
    pub selected: Color32,
}

impl ThemeMode {
    pub fn colors(&self) -> ColorScheme {
        match self {
            ThemeMode::Dark => ColorScheme {
                bg: Color32::from_rgb(30, 30, 30),
                bg_panel: Color32::from_rgb(40, 40, 40),
                text_primary: Color32::from_rgb(220, 220, 220),
                text_secondary: Color32::from_rgb(160, 160, 160),
                accent: Color32::from_rgb(70, 130, 230),
                highlight: Color32::from_rgb(255, 255, 100),
                highlight_bg: Color32::from_rgb(80, 80, 40),
                error: Color32::from_rgb(240, 80, 80),
                warn: Color32::from_rgb(240, 180, 60),
                info: Color32::from_rgb(100, 180, 240),
                debug: Color32::from_rgb(160, 160, 160),
                selected: Color32::from_rgb(60, 80, 120),
            },
            ThemeMode::Light => ColorScheme {
                bg: Color32::from_rgb(245, 245, 245),
                bg_panel: Color32::from_rgb(255, 255, 255),
                text_primary: Color32::from_rgb(30, 30, 30),
                text_secondary: Color32::from_rgb(100, 100, 100),
                accent: Color32::from_rgb(50, 100, 200),
                highlight: Color32::from_rgb(200, 150, 0),
                highlight_bg: Color32::from_rgb(255, 240, 180),
                error: Color32::from_rgb(200, 50, 50),
                warn: Color32::from_rgb(180, 130, 0),
                info: Color32::from_rgb(50, 120, 200),
                debug: Color32::from_rgb(120, 120, 120),
                selected: Color32::from_rgb(200, 220, 255),
            },
        }
    }
}

// Legacy constants for backward compatibility
pub const BG_DARK: Color32 = Color32::from_rgb(30, 30, 30);
pub const BG_PANEL: Color32 = Color32::from_rgb(40, 40, 40);
pub const TEXT_PRIMARY: Color32 = Color32::from_rgb(220, 220, 220);
pub const TEXT_SECONDARY: Color32 = Color32::from_rgb(160, 160, 160);
pub const ACCENT: Color32 = Color32::from_rgb(70, 130, 230);
pub const HIGHLIGHT: Color32 = Color32::from_rgb(255, 255, 100);
pub const ERROR: Color32 = Color32::from_rgb(240, 80, 80);
pub const WARN: Color32 = Color32::from_rgb(240, 180, 60);
pub const INFO: Color32 = Color32::from_rgb(100, 180, 240);
pub const DEBUG: Color32 = Color32::from_rgb(160, 160, 160);
pub const SELECTED: Color32 = Color32::from_rgb(60, 80, 120);
