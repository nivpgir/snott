#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

pub(crate) mod autocomplete_popup;
mod app;

// hide console window on Windows in release
use eframe::egui;


fn main() {
    let options = eframe::NativeOptions {
        decorated: false,
        transparent: true,
        min_window_size: Some(egui::vec2(320.0, 100.0)),
        resizable: true,
	..Default::default()
    };

    // tracing_subscriber::fmt::init();

    eframe::run_native(
        "eframe template",
        options,
        Box::new(|_cc| Box::new(app::Snotter::default())),
    );
}


