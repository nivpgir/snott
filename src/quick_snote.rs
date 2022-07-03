
// hide console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![cfg_attr(debug_assertions, allow(dead_code))]

use std::{fs, path::PathBuf};
use chrono::{Local, DateTime};


use eframe::egui;

use crate::snote::snote_widget;

const TIME_FORMAT: &str = "[%Y-%m-%d][%H:%M:%S]";

#[derive(Debug)]
pub struct QuickSnote {
    creation_time: DateTime<Local>,
    text: String,
    pub config: PathBuf,
}

impl Default for QuickSnote{
    fn default() -> Self {
        Self {
	    config: dirs::home_dir().unwrap_or_default().join("Sync"),
	    creation_time: Local::now(),
	    text: Default::default(),
	}
    }
}
impl eframe::App for QuickSnote {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
	egui::CentralPanel::default().show(ctx, |ui|{
	    if ui.input().modifiers.ctrl &&
		ui.input().key_pressed(egui::Key::Q) {
		    frame.quit();
		}

            ui.centered_and_justified(|ui| {
		ui.add(snote_widget(&mut self.text)).request_focus()
	    });
	});
    }

    fn on_exit(&mut self, _gl: &eframe::glow::Context) {
	let file_name = self.creation_time.format("%Y-%m-%d_%H-%M-%S.snot");
	let full_path = self.config.join(PathBuf::from(&file_name.to_string()));
	fs::write(&full_path, &self.text)
	    .unwrap_or_else(|e| panic!("failed to save file {}: {}", &full_path.display(), e));
    }
}
