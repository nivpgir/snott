
use eframe::egui::{self, Modifiers, TextEdit, Widget, WidgetText, text::CCursorRange};

#[derive(Debug)]
pub(crate) struct AutocompletePopup<F, C>
where
    F: FnOnce(&C),
    C: Clone + Into<WidgetText>,
{
    items: Vec<C>,
    parent: egui::Response,
    select_action: F
    // selection: Option<Candidate>
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Selection{
    Chosen(usize),
    Marked(usize),
    Nothing
}

#[derive(Debug)]
pub enum AutcompleteOutput<C>{
    Chosen(usize, C, String),
    Marked(usize, C, String),
}

impl<F, C> AutocompletePopup<F, C>
where
    C: Clone + Into<WidgetText> + std::fmt::Debug,
    F: FnOnce(&C)

{
    pub fn id (&self) -> egui::Id{
	self.parent.id.with("::ac")
    }

    pub fn new(items: impl IntoIterator<Item=C>,
	       parent: &egui::Response,
	       select_action: F) -> Self{
	Self{
	    select_action,
	    items: items.into_iter().collect(),
	    parent: parent.clone(),
	}
    }

    fn get_selection(&self, selection: Selection) -> Option<AutcompleteOutput<C>>{
	selection.into_output(&self.items)
    }
    // fn save_state(&self, ui: &mut egui::Ui, selection: Selection){
    // 	match selection{
    // 	    s @ Selection::Chosen(i) => ui.memory().close_popup(),
    // 	    o => {o;}
    // 	}
    // }

    // fn do_selection(&self, selection: usize){
    // 	dbg!(selection);

    // 	let action = self.select_action;
    // 	action(&self.items[selection]);
    // }

    fn create_autocomplete_labels(&self, ui: &mut egui::Ui, marked_value: Selection)
				  -> Option<Vec<egui::Response>>{

	egui::popup_below_widget(ui, self.id(), &self.parent, |ui|{
	    self.items.iter().enumerate()
		.map(|(i, item)|{
		    ui.selectable_label(marked_value == Selection::Marked(i) ,item.clone())
		})}.collect())
    }

    fn update_mark_by_keyboard(&mut self, mark: Selection, ui: &mut egui::Ui) -> Selection{
	use egui::Key;
	let ret = if ui.input_mut().consume_key(Modifiers::NONE, Key::ArrowUp){
	    mark.dec().clamp(self.items.len())
	} else if ui.input_mut().consume_key(Modifiers::NONE, Key::ArrowDown){
	    mark.inc().clamp(self.items.len())
	} else if ui.input_mut().consume_key(Modifiers::NONE, Key::Enter){
	    mark.into_choice()
	} else {
	    mark
	};
	ret
    }

    pub fn show_popup(&mut self, ui: &mut egui::Ui) -> Option<AutcompleteOutput<C>>{
	if self.parent.gained_focus(){
	    ui.memory().open_popup(self.id())
	}

	let mut marked_value: Selection = *ui.memory().data
	    .get_temp_mut_or(self.id(), Selection::Nothing);

	marked_value = self.update_mark_by_keyboard(marked_value, ui);
	if let Some(label_responses) = self.create_autocomplete_labels(ui, marked_value){
	    marked_value = update_mark_by_mouse_interaction(marked_value, label_responses);
	}

	if let Selection::Chosen(_) = marked_value {
	    ui.memory().data.insert_temp(self.id(), Selection::Nothing);
	} else {
	    ui.memory().data.insert_temp(self.id(), marked_value);
	}
	self.get_selection(marked_value)

    }
}

pub fn get_cursor_pos(parent_id: egui::Id, ui: &egui::Ui) -> Option<CCursorRange>{
    TextEdit::load_state(ui.ctx(), parent_id)
	.and_then(|s|s.ccursor_range())
}

pub fn set_cursor_pos(parent_id: egui::Id, ui: &mut egui::Ui, cursor: CCursorRange) {
    if let Some(mut state) = TextEdit::load_state(ui.ctx(), parent_id) {
	state.set_ccursor_range(Some(cursor));
	TextEdit::store_state(ui.ctx(), parent_id, state);
    }
}


fn check_mouse_interactions((i, response): (usize, egui::Response)) -> Selection{
    if response.clicked(){
	Selection::Chosen(i)
    } else if response.ctx.input().pointer.is_moving() && response.hovered() {
	// TODO: BUG:
	// for some reason response.hovered() is always false
	Selection::Marked(i)
    } else {
	Selection::Nothing
    }
}


fn update_mark_by_mouse_interaction(current_mark: Selection, mouse_interactions: Vec<egui::Response>)
				    -> Selection{
    use std::ops::ControlFlow;
    let new_mark = mouse_interactions.into_iter().enumerate()
	.map(check_mouse_interactions)
	.try_fold(current_mark, |prev, next|{
	    match prev.update(next){
		m @ Selection::Chosen(_) => ControlFlow::Break(m),
		m => ControlFlow::Continue(m)
	    }
	});
    match new_mark{
    // for some reason there is no way of getting the inner value without matching,
    // even when they both hold the same type, idk why...
	ControlFlow::Break(v) => v,
	ControlFlow::Continue(v) => v,
    }
}

impl<C, F> Widget for AutocompletePopup<F, C>
where
    C: Clone + Into<WidgetText> + std::fmt::Debug,
    F: FnOnce(&C) {

    fn ui(mut self, ui: &mut egui::Ui) -> egui::Response {

	self.show_popup(ui);
	self.parent
    }
}



impl Selection {

    pub(crate) fn dec(&self) -> Self {
	match self {
	    Self::Chosen(i) => Self::Chosen(i.saturating_sub(1)),
	    Self::Marked(i) => Self::Marked(i.saturating_sub(1)),
	    Self::Nothing => Self::Marked(0)
	}
    }

    pub(crate) fn inc(&self) -> Self {
	match self {
	    Self::Chosen(i) => Self::Chosen(i.saturating_add(1)),
	    Self::Marked(i) => Self::Marked(i.saturating_add(1)),
	    Self::Nothing => Self::Marked(0)
	}
    }
    pub(crate) fn clamp(&self, max: usize) -> Self {
	match *self {
	    Self::Chosen(i) => Self::Chosen(i.clamp(0, max)),
	    Self::Marked(i) => Self::Marked(i.clamp(0, max)),
	    Self::Nothing => Self::Marked(0)
	}
    }

    pub(crate) fn into_choice(self) -> Self {
        match self{
	    Self::Marked(i) => Self::Chosen(i),
	    Self::Nothing => Self::Chosen(0),
	    choice => choice
	}
    }

    pub(crate) fn update(self, other: Self) -> Self{
	    match (self, other){
		(Selection::Nothing, other) => other,
		(other, Selection::Nothing) => other,
		(Selection::Chosen(i), _) |
		(Selection::Marked(_), Selection::Chosen(i)) => Selection::Chosen(i),
		(Selection::Marked(i), Selection::Marked(_)) => Selection::Marked(i),
	    }
    }

    fn into_output<C: Clone + Into<WidgetText> + std::fmt::Debug>(self, items: &[C])
								  -> Option<AutcompleteOutput<C>> {
	let index = *match &self{
	    Selection::Chosen(i) |
	    Selection::Marked(i) => i,
	    Selection::Nothing => return None
	};
	let value: C = items[index].clone();
	let text: WidgetText = value.clone().into();
	match self{
	    Selection::Chosen(i) => Some(AutcompleteOutput::Chosen(i, value, text.text().to_string())),
	    Selection::Marked(i) => Some(AutcompleteOutput::Marked(i, value, text.text().to_string())),
	    _ => unreachable!()
	}

    }
}
