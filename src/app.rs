use std::{cell::RefCell, ops::{Deref, DerefMut}, path::PathBuf, str::FromStr, sync::RwLock};

use eframe::egui::{self, TextBuffer, WidgetText, text_edit::{CCursorRange, TextEditOutput}};

use crate::autocomplete_popup::{AutocompleteOutput, AutocompletePopup};

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

impl std::fmt::Display for MyWidgetText<PathBuf>{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
	write!(f, "{}",
	       self.0.to_path_buf().file_name()
	       .expect("failed to get filename!")
	       .to_string_lossy())
    }
}

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
        other.to_string_lossy().to_string().into()
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
        custom_window_frame(ctx, frame, "snote", |ui|{
	    ui.label(&self.snots_dir.to_string_lossy().to_string());
	    egui::widgets::global_dark_light_mode_switch(ui);

	    let search_bar = egui::TextEdit::singleline(&mut self.search_query)
		.desired_width(ui.available_width()).show(ui);

	    let ac_output = AutocompletePopup::new(file_candidates, &search_bar.response)
		.show(ui, &search_bar.response);
	    self.update_from_autocomplete(ac_output, ctx, search_bar);

	})
    }
}

type ACItem = AutocompleteOutput<MyWidgetText<PathBuf>>;
impl Snotter{
    fn update_query_from_autocomplete(&mut self, chosen: MyWidgetText<PathBuf>){
	self.search_query = chosen.to_string();
    }

    fn update_from_autocomplete(&mut self, s: Option<ACItem>,
				ctx: &egui::Context,
				search_bar: TextEditOutput){
	if let Some(AutocompleteOutput::Chosen(chosen)) = s {
	    self.update_query_from_autocomplete(chosen);
	    self.update_cursor_from_autocomplete(ctx, search_bar);

	}
    }
    fn update_cursor_from_autocomplete(&self, ctx: &egui::Context, mut search_bar: TextEditOutput){
	if let Some(c) = search_bar.state.ccursor_range() {
	    let [_, last_cursor] = c.sorted();
	    let cursor = CCursorRange::one(last_cursor + self.search_query.len());
	    search_bar.state.set_ccursor_range(Some(cursor));
	    search_bar.state.store(ctx, search_bar.response.id);
	}
    }
}

fn custom_window_frame(
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
