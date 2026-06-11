//! log-seek application entry point.

// Hide console window on Windows
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1600.0, 900.0])
            .with_min_inner_size([800.0, 600.0]),
        ..Default::default()
    };

    eframe::run_native(
        "log-seek",
        options,
        Box::new(|cc| Ok(Box::new(logseek_ui::app::App::new(cc)))),
    )
}
