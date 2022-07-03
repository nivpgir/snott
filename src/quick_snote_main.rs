
use std::{path::PathBuf, str::FromStr};

use eframe::egui;
use snote2::quick_snote::QuickSnote;

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
        "quick",
        options,
        Box::new(|cc| {
            cc.egui_ctx.set_visuals(eframe::egui::Visuals::dark());
            Box::new(QuickSnote::default())
        }),
    );
}

