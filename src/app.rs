use std::{error::Error, fmt::Display, ops::{Deref, DerefMut, Not}, path::PathBuf, str::FromStr};

use eframe::egui::{self, Sense, WidgetText, text_edit::{CCursorRange, TextEditOutput, TextEditState}};

use crate::{autocomplete_popup::{AutocompleteOutput, AutocompletePopup}, custom_window, snote::{self, snote_widget}};

#[derive(Debug, Default)]
pub struct Snotter {
    snots_dir: PathBuf,
    search_query: String,
    note: (Option<PathBuf>, Option<snote::SNote>),
}

struct ValueButton<T>{
    button: egui::Button,
    pub value: T
}

impl<T: Default + Display> Default for ValueButton<T>{
    fn default() -> Self {
	let val: T = Default::default();
        Self { button: egui::Button::new(val.to_string()), value: val }
    }
}

impl<T: Display> egui::Widget for ValueButton<T>{
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        ui.add(self.button)
    }
}

#[derive(Clone, Debug)]
struct WidgetTextWrap<T>(T);

impl std::fmt::Display for WidgetTextWrap<PathBuf> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0
            .to_path_buf()
            .file_name()
            .expect("failed to get filename!")
            .to_string_lossy()
            .fmt(f)
    }
}

impl<T> Deref for WidgetTextWrap<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for WidgetTextWrap<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<WidgetTextWrap<PathBuf>> for WidgetText {
    fn from(other: WidgetTextWrap<PathBuf>) -> Self {
        other.display().to_string().into()
    }
}

impl eframe::App for Snotter {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        custom_window::custom_window_frame(ctx, frame, "snott", |ui| {
            ui.vertical_centered_justified(|ui| {
                self.top_bar(ui);

		ui.add(self.search_bar(ctx));

		if self.snote_editor(ui).changed(){
		    self.save_note().unwrap_or(())
		};
	    });
	})
    }
}


type ACItem = AutocompleteOutput<WidgetTextWrap<PathBuf>>;
impl Snotter {
    fn top_bar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal_top(|ui| {
            egui::widgets::global_dark_light_mode_switch(ui);
	    ui.add(self.snot_dir_button())
        });
    }
    fn snot_dir_button(&mut self) -> impl egui::Widget + '_{
	|ui: &mut egui::Ui|{
	    let button = ui.button(self.snots_dir.display().to_string());
	    if button.clicked() {
		self.snots_dir = rfd::FileDialog::new()
		    .pick_folder()
		    .unwrap_or_else(|| self.snots_dir.clone());
	    }
	    button
	}
    }
    fn search_bar<'u, 'c: 'u>(&'u mut self, ctx: &'c egui::Context) -> impl egui::Widget + 'u{
	|ui: &mut egui::Ui|{
	    let TextEditOutput { response, state, .. } =
		egui::TextEdit::singleline(&mut self.search_query)
		.show(ui);
	    let notes: Vec<_> = self
		.get_matching_notes()
		.iter()
		.map(|f| WidgetTextWrap(f.to_path_buf()))
		.collect();
	    if response.gained_focus() || response.changed() {
		if notes.is_empty().not() {
		    ui.memory().open_popup(response.id.with("::ac"));
		}
            }
	    let ac_output = {
		// let popup_response =
		//     egui::popup_below_widget(ui, self.id, response, self.make_completion_widget());

		// if let Some(Chosen(_)) = popup_response {
		//     ui.memory().data.remove::<Selection>(self.id);
		//     ui.memory().close_popup();
		//     parent.request_focus();
		// } else if let Some(selection) = popup_response {
		//     ui.memory().data.insert_temp(self.id, selection)
		// }
		// if let Some(c) = popup_response {
		//     self.items.get(c.into_inner()).map(|v| c.map(|_| v.clone()))
		// } else {
		//     None
		// }
		// ui.push_id("::ac", |ui|{
		egui::popup_below_widget(ui, response.id.with("::ac"), &response,
					 AutocompletePopup::new(notes, response.clone())
					 .make_completion_widget())
		// }
	    };
	    self.update_from_autocomplete(ac_output.flatten(), ctx, state, response.id);
	    response
	}
    }

    fn save_note(&self) -> Result<(), Box<dyn Error>> {
	if let (Some(file_path), Some(note)) = &self.note{
	    std::fs::write(file_path, &note.raw_content)?;
	}
	Ok(())
    }
    fn get_matching_notes(&self) -> Vec<PathBuf> {
        self.snots_dir
            .read_dir()
            .map(|d| {
                d.filter_map(std::result::Result::ok)
                    .filter(|f| f.path().extension() == Some("snot".as_ref()))
                    .filter_map(|f| {
                        f.file_type()
                            .ok()
                            .and_then(|f_t| f_t.is_file().then(|| f.path()))
                    })
                    .filter(|f| f.to_string_lossy().contains(self.search_query.as_str()))
                    .collect()
            })
            .unwrap_or_default()
    }

    fn update_from_autocomplete(
        &mut self,
        s: Option<ACItem>,
        ctx: &egui::Context,
        search_bar: TextEditState,
	id: egui::Id
    ) {
        if let Some(AutocompleteOutput::Chosen(chosen)) = s {
            self.select_file_from_autocomplete(chosen.0);
            self.update_cursor_from_autocomplete(ctx, search_bar, id);
        }
    }

    fn select_file_from_autocomplete(&mut self, chosen: PathBuf) {
        let file_content = std::fs::read_to_string(&chosen)
	    .ok()
	    .or_else(||Some(format!("failed to read {}", chosen.display())));
        let note = file_content
	    .map(|s|
		 snote::SNote::from_str(&s)
		 .unwrap_or_else(|_|snote::SNote::new().set_raw(s))
	    );
        self.search_query = chosen.display().to_string();
	self.note = (Some(chosen), note);
    }


    fn update_cursor_from_autocomplete(&self, ctx: &egui::Context,
				       mut state: TextEditState,
				       id: egui::Id) {
        if let Some(c) = state.ccursor_range() {
            let [_, last_cursor] = c.sorted();
            let cursor = CCursorRange::one(last_cursor + self.search_query.len());
            state.set_ccursor_range(Some(cursor));
            state.store(ctx, id);
        }
    }

    fn snote_editor(&'_ mut self, ui: &mut egui::Ui) -> egui::Response{
	self.note.1.as_mut().map(|note|{
	    ui.add(snote_widget(&mut note.raw_content))
	}).unwrap_or_else(||empty_widget(ui))
    }
}



fn empty_widget(ui: &mut egui::Ui) -> egui::Response{
    ui.allocate_response(
	egui::Vec2::ZERO,
	Sense{
	    click: false,
	    drag: false,
	    focusable: false,
	})
}

