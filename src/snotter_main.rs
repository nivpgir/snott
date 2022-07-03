// hide console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![cfg_attr(debug_assertions, allow(dead_code))]

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
        Box::new(|cc| {
            cc.egui_ctx.set_visuals(eframe::egui::Visuals::dark());
            Box::new(snote2::app::Snotter::default())
        }),
    );
}
