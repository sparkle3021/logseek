//! UI font and spacing constants.

/// Default font sizes
pub const FONT_SIZE_BODY: f32 = 14.0;
pub const FONT_SIZE_HEADING: f32 = 18.0;
pub const FONT_SIZE_MONO: f32 = 13.0;
pub const FONT_SIZE_SMALL: f32 = 12.0;

/// Layout constants
pub const ROW_HEIGHT: f32 = 20.0;
pub const PADDING: f32 = 8.0;
pub const SPACING: f32 = 4.0;
pub const PANEL_WIDTH_SOURCES: f32 = 220.0;
pub const PANEL_HEIGHT_STATUS: f32 = 24.0;

/// Font configuration that can be changed at runtime
#[derive(Debug, Clone)]
pub struct FontConfig {
    pub body_size: f32,
    pub mono_size: f32,
    pub line_height_multiplier: f32,
}

impl Default for FontConfig {
    fn default() -> Self {
        Self {
            body_size: FONT_SIZE_BODY,
            mono_size: FONT_SIZE_MONO,
            line_height_multiplier: 1.2,
        }
    }
}

impl FontConfig {
    /// Get the actual row height based on font size and line height multiplier
    pub fn row_height(&self) -> f32 {
        self.mono_size * self.line_height_multiplier
    }
    
    /// Get preset configurations
    pub fn presets() -> Vec<(&'static str, FontConfig)> {
        vec![
            ("默认", FontConfig::default()),
            ("小字体", FontConfig {
                body_size: 12.0,
                mono_size: 11.0,
                line_height_multiplier: 1.15,
            }),
            ("大字体", FontConfig {
                body_size: 16.0,
                mono_size: 15.0,
                line_height_multiplier: 1.3,
            }),
            ("代码阅读", FontConfig {
                body_size: 14.0,
                mono_size: 14.0,
                line_height_multiplier: 1.5,
            }),
        ]
    }
}
