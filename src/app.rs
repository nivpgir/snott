use std::{cell::RefCell, ops::{Deref, DerefMut}, path::PathBuf, str::FromStr, sync::RwLock};

use eframe::egui::{self, TextBuffer, WidgetText, text_edit::{CCursorRange, TextEditOutput}};

use crate::autocomplete_popup::{AutocompleteOutput, AutocompletePopup};

#[derive(Debug)]
pub(crate) struct Snotter {
    snots_dir: Option<PathBuf>,
    search_query: String,
    selected_file: Option<String>
}

impl Default for Snotter {
    fn default() -> Self{
	Self {
	    // snots_dir: PathBuf::from_str(&std::env::var("HOME")
	    // 				 .unwrap_or_else(|_| "".to_string())).ok(),
	    snots_dir: PathBuf::new().with_file_name(".").canonicalize().ok(),
	    search_query: Default::default(),
	    selected_file: None
	}
    }
}


#[derive(Default)]
struct SyncTextBuffer<T: TextBuffer>(RwLock<RefCell<T>>);

#[derive(Clone, Debug)]
struct WidgetTextWrap<T>(T);

impl std::fmt::Display for WidgetTextWrap<PathBuf>{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
	write!(f, "{}",
	       self.0
	       .to_path_buf().file_name()
	       .expect("failed to get filename!")
	       .to_string_lossy())
    }
}

impl<T> Deref for WidgetTextWrap<T>{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for WidgetTextWrap<T>{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<WidgetTextWrap<PathBuf>> for WidgetText{
    fn from(other: WidgetTextWrap<PathBuf>) -> Self {
        other.display().to_string().into()
    }
}

impl eframe::App for Snotter {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        custom_window_frame(ctx, frame, "snott", |ui|{
	    ui.horizontal_top(|ui|{
		egui::widgets::global_dark_light_mode_switch(ui);
		if self.snots_dir.is_none() ||
		    ui.button(&self.snots_dir.as_ref().unwrap().display().to_string())
		    .clicked() {
			self.snots_dir = rfd::FileDialog::new().pick_folder();
		    }
	    });
	    ui.vertical_centered_justified(|ui|{

		let notes: Vec<_> = self.get_matching_notes().iter()
		    .map(|f|WidgetTextWrap(f.to_path_buf())).collect();


		let search_bar = egui::TextEdit::singleline(&mut self.search_query).show(ui);

		if let Some(note_file) = &mut self.selected_file{
		    ui.code_editor(note_file);
		}
		let ac_output = AutocompletePopup::new(notes, &search_bar.response)
		    .show(ui, &search_bar.response);
		self.update_from_autocomplete(ac_output, ctx, search_bar);
	    });
	})
    }
}





type ACItem = AutocompleteOutput<WidgetTextWrap<PathBuf>>;
impl Snotter{
    fn get_matching_notes(&self) -> Vec<PathBuf> {
	if let Some(dir) = &self.snots_dir{
	    dir.read_dir().map(
	    |d|d.filter_map(std::result::Result::ok)
		.filter(|f|f.path().extension() == Some("snot".as_ref()))
		.filter_map(|f|f.file_type().ok().and_then(|f_t|f_t.is_file().then(||f.path())))
		.filter(|f|f.to_string_lossy().contains(self.search_query.as_str()))
		.collect()
	    ).unwrap_or_default()
	} else {
	    vec![]
	}
    }

    fn select_file_from_autocomplete(&mut self, chosen: WidgetTextWrap<PathBuf>){
	self.search_query = chosen.to_string();
	self.selected_file = std::fs::read_to_string(&chosen.0).ok()
	    .or_else(||Some(format!("failed to read {:?}", chosen.0.as_os_str())));
    }

    fn update_from_autocomplete(&mut self, s: Option<ACItem>,
				ctx: &egui::Context,
				search_bar: TextEditOutput){
	if let Some(AutocompleteOutput::Chosen(chosen)) = s {
	    self.select_file_from_autocomplete(chosen);
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
