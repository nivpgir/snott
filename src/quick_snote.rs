use std::path::Path;
use std::{fs, path::PathBuf};
use chrono::{Local, DateTime};

use eframe::egui;

use crate::snote::snote_widget;

#[derive(Debug)]
pub struct QuickSnote {
    creation_time: DateTime<Local>,
    text: String,
    approved: bool,
    pub sync_dir: PathBuf,
    pub timestamp_format: String,
}

impl QuickSnote{
    pub fn new(sync_dir: impl AsRef<Path>) -> Self{
        Self{
            creation_time: Local::now(),
            text: Default::default(),
            sync_dir: sync_dir.as_ref().into(),
            approved: false,
            timestamp_format: Default::default(),
        }
    }
    pub fn with_time_format(self, time_format: impl AsRef<str>) -> Self{
        Self{
            timestamp_format: time_format.as_ref().to_string(),
            ..self
        }
    }

	}
    }
}
impl Default for QuickSnote{
    fn default() -> Self {
        Self {
            creation_time: Local::now(),
            sync_dir: Default::default(),
            text: Default::default(),
            approved: false,
            timestamp_format: Default::default(),
        }
    }
}
impl eframe::App for QuickSnote {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui|{
            if ui.input().modifiers.ctrl {
		if  ui.input().key_pressed(egui::Key::Enter) {
		    self.approved = true;
		    frame.quit();
		}
                else if ui.input().key_pressed(egui::Key::Q) {
                    frame.quit();
                }
	    }

            ui.centered_and_justified(|ui| {
                ui.add(snote_widget(&mut self.text)).request_focus();
            });
        });
    }

    fn on_exit(&mut self, _gl: &eframe::glow::Context) {
	// TODO: remove the .txt suffix when I get android app support
        let name_format = format!("{}.snot.txt", self.timestamp_format);
        let file_name = self.creation_time.format(&name_format);
        let full_path = self.sync_dir.join(PathBuf::from(&file_name.to_string()));
        if self.approved {
            fs::write(&full_path, &self.text)
                .unwrap_or_else(
                    |e|panic!("failed to save file {}: {}", &full_path.display(), e)
                )
        }
    }
}
