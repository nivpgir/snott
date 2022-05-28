use std::{cell::RefCell, ops::{Deref, DerefMut}, path::PathBuf, str::FromStr, sync::{RwLock}};

use eframe::egui::{self, TextBuffer, WidgetText};

use crate::autocomplete_popup::{AutcompleteOutput, AutocompletePopup, set_cursor_pos};

#[derive(Debug)]
pub(crate) struct Snotter {
    snots_dir: PathBuf,
    search_query: String
}

impl Default for Snotter {
    fn default() -> Self{
	Self {
	    snots_dir: PathBuf::from_str(&std::env::var("HOME")
					 .unwrap_or_else(|_| "".to_string())).unwrap(),
	    search_query: Default::default()
	}
    }
}


#[derive(Default)]
struct SyncTextBuffer<T: TextBuffer>(RwLock<RefCell<T>>);

#[derive(Clone, Debug)]
struct MyWidgetText<T>(T);

impl<T> Deref for MyWidgetText<T>{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for MyWidgetText<T>{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<MyWidgetText<PathBuf>> for WidgetText{
    fn from(other: MyWidgetText<PathBuf>) -> Self {
        other.0.to_string_lossy().to_string().into()
    }
}

impl eframe::App for Snotter {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
	let files: Vec<_> = self.snots_dir.read_dir().unwrap()
	    .map(|f|f.unwrap()).collect();
	let file_candidates: Vec<_> = files.iter()
	    .filter_map(|f|f.file_name().to_string_lossy().to_string()
			.contains(self.search_query.as_str())
			.then(||MyWidgetText(f.path())))
	    .collect();
        custon_window_frame(ctx, frame, "snote", |ui|{
	    ui.label(&self.snots_dir.to_string_lossy().to_string());
	    egui::widgets::global_dark_light_mode_switch(ui);

	    let search_bar = egui::TextEdit::singleline(&mut self.search_query)
		.desired_width(ui.available_width()).show(ui);

	    let cursor = search_bar.cursor_range.map(|c|c.as_ccursor_range());
	    AutocompletePopup::new(
		file_candidates,
		&search_bar.response,
		|choice|{
		    self.search_query = choice.to_string_lossy().to_string();
		}).show_popup(ui).and_then(|choice|{
		    if let AutcompleteOutput::Chosen(_, _, text) =  choice{
			self.search_query = text;
		    };
		    Some(())
		});

	    if let Some(c) = cursor {
		set_cursor_pos(search_bar.response.id, ui, c);
	    }

	})
    }
}

fn custon_window_frame(
    ctx: &egui::Context,
    frame: &mut eframe::Frame,
    title: &str,
    add_contents: impl FnOnce(&mut egui::Ui),
) {
    use egui::*;
    let text_color = ctx.style().visuals.text_color();

    // Height of the title bar
    let height = 28.0;

    CentralPanel::default()
        .frame(Frame::none())
        .show(ctx, |ui| {
            let rect = ui.max_rect();
            let painter = ui.painter();

            // Paint the frame:
            painter.rect(
                rect.shrink(1.0),
                10.0,
                ctx.style().visuals.window_fill(),
                Stroke::new(1.0, text_color),
            );

            // Paint the title:
            painter.text(
                rect.center_top() + vec2(0.0, height / 2.0),
                Align2::CENTER_CENTER,
                title,
                FontId::proportional(height - 2.0),
                text_color,
            );

            // Paint the line under the title:
            painter.line_segment(
                [
                    rect.left_top() + vec2(2.0, height),
                    rect.right_top() + vec2(-2.0, height),
                ],
                Stroke::new(1.0, text_color),
            );

            // Add the close button:
            let close_response = ui.put(
                Rect::from_min_size(rect.left_top(), Vec2::splat(height)),
                Button::new("X").frame(false),
            );
            if close_response.clicked() {
                frame.quit();
            }

            // Interact with the title bar (drag to move window):
            let title_bar_rect = {
                let mut rect = rect;
                rect.max.y = rect.min.y + height;
                rect
            };
            let title_bar_response =
                ui.interact(title_bar_rect, Id::new("title_bar"), Sense::drag());
            if title_bar_response.drag_started() {
                frame.drag_window();
            }

            // Add the contents:
            let content_rect = {
                let mut rect = rect;
                rect.min.y = title_bar_rect.max.y;
                rect
            }
            .shrink(4.0);
            let mut content_ui = ui.child_ui(content_rect, *ui.layout());
            add_contents(&mut content_ui);
        });
}
