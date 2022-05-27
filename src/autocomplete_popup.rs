use eframe::egui::{self, Modifiers, Widget};

#[derive(Debug)]
pub(crate) struct AutocompletePopup<F: FnMut(String)>{
    items: Vec<String>,
    parent: egui::Response,
    select_action: F
}

type Acmem = AutocompleteMemory;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum AutocompleteMemory{
    Chosen(usize),
    Marked(usize),
    Nothing
}

impl<F: FnMut(String)> AutocompletePopup<F>{
    pub fn id (&self) -> egui::Id{
	self.parent.id.with("::ac")
    }
    pub fn new(items: impl IntoIterator<Item=String>,
	       parent: egui::Response,
	       select_action: F) -> Self{
	Self{
	    items: items.into_iter().collect(),
	    parent,
	    select_action
	}
    }

    fn do_selection(&mut self, selection: usize){
	dbg!(selection);
	let selected_value = self.items[selection].clone();
	let action = &mut self.select_action;
	action(selected_value);
    }

    fn create_autocomplete_labels(&self, ui: &mut egui::Ui, marked_value: Acmem)
				  -> Option<Vec<egui::Response>>{

	egui::popup_below_widget(ui, self.id(), &self.parent, |ui|{
	    self.items
		.iter()
		.enumerate()
		.map(|(i, item)|ui.selectable_label(marked_value == Acmem::Marked(i) ,item)
		)}.collect())
    }

    fn update_mark_by_keyboard(&mut self, mark: Acmem, ui: &mut egui::Ui) -> Acmem{
	use egui::Key;
	if ui.input_mut().consume_key(Modifiers::NONE, Key::ArrowUp){
	    mark.dec().clamp(self.items.len())
	} else if ui.input_mut().consume_key(Modifiers::NONE, Key::ArrowDown){
	    mark.inc().clamp(self.items.len())
	} else if ui.input_mut().consume_key(Modifiers::NONE, Key::Enter){
	    mark.into_choice()
	} else {
	    mark
	}
    }
}
fn check_mouse_interactions((i, response): (usize, egui::Response)) -> Acmem{
    if response.clicked(){
	Acmem::Chosen(i)
    } else if response.ctx.input().pointer.is_moving() && response.hovered() {
	Acmem::Marked(i)
    } else {
	Acmem::Nothing
    }
}


fn update_mark_by_mouse_interaction(current_mark: Acmem, mouse_interactions: Vec<egui::Response>)
				    -> Acmem{
    use std::ops::ControlFlow;
    let new_mark = mouse_interactions.into_iter().enumerate()
	.map(check_mouse_interactions)
	.try_fold(current_mark, |prev, next|{
	    match prev.update(next){
		m @ Acmem::Chosen(_) => ControlFlow::Break(m),
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

impl<F: FnMut(String)> Widget for AutocompletePopup<F>{
    fn ui(mut self, ui: &mut egui::Ui) -> egui::Response {
	if self.parent.gained_focus(){
	    ui.memory().open_popup(self.id())
	}

	let mut marked_value: Acmem = *ui.memory().data
	    .get_temp_mut_or(self.id(), Acmem::Marked(0));

	marked_value = self.update_mark_by_keyboard(marked_value, ui);
	if let Some(label_responses) = self.create_autocomplete_labels(ui, marked_value){
	    marked_value = update_mark_by_mouse_interaction(marked_value, label_responses);
	}

	if let Acmem::Chosen(i) = marked_value{
	    self.do_selection(i);
	    ui.memory().data.insert_temp(self.id(), Acmem::Nothing);
	    ui.memory().close_popup();
	} else {
	    ui.memory().data.insert_temp(self.id(), marked_value);
	}
	self.parent
    }
}

impl AutocompleteMemory {

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
		(Acmem::Nothing, other) => other,
		(other, Acmem::Nothing) => other,
		(Acmem::Chosen(i), _) |
		(Acmem::Marked(_), Acmem::Chosen(i)) => Acmem::Chosen(i),
		(Acmem::Marked(i), Acmem::Marked(_)) => Acmem::Marked(i),
	    }
    }
}
