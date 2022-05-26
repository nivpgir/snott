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
    fn do_selection(self, selection: usize, ui: &mut egui::Ui) -> egui::Response{
	dbg!(selection);
	let selected_value = self.items[selection].clone();
	let mut action = self.select_action;
	action(selected_value);
	ui.memory().close_popup();
	self.parent
    }
    fn create_autocomplete_labels(&self, ui: &mut egui::Ui, marked_value: Acmem) -> Vec<Acmem>{

	self.items.iter().enumerate().map(move |(i, item)| {
	    (i, ui.selectable_label(marked_value == Acmem::Marked(i) ,item))
	}).map(check_mouse_interactions).collect::<Vec<_>>()
    }

    fn update_mark_by_keyboard(&mut self, ui: &mut egui::Ui, mark: Acmem) -> Acmem{
	use egui::Key;
	let ret = match (mark, &ui.input().keys_down){
	    (v, keys) if keys.contains(&Key::Enter) => v.into_choice(),
	    (v, keys) if keys.contains(&Key::ArrowDown) => v.inc().clamp(self.items.len()),
	    (v, keys) if keys.contains(&Key::ArrowUp) => v.dec().clamp(self.items.len()),
	    (v, _) => v
	};
	if ret != mark{
	    self.parent.mark_changed()
	}
	ui.input_mut().consume_key(Modifiers::NONE, Key::ArrowUp).then(||Key::ArrowUp);
	ui.input_mut().consume_key(Modifiers::NONE, Key::ArrowDown).then(||Key::ArrowDown);
	ui.input_mut().consume_key(Modifiers::NONE, Key::Enter).then(||Key::Enter);
	ret

    }
}
fn check_mouse_interactions((i, response): (usize, egui::Response)) -> Acmem{
    if response.clicked(){
	Acmem::Chosen(i)
    } else if response.hovered() {
	Acmem::Marked(i)
    } else {
	Acmem::Nothing
    }
}


fn collect_clicked_and_hovered_labels(label_mouse_interactions: Vec<Acmem>)
			       -> Acmem{
    label_mouse_interactions.into_iter()
	.fold(Acmem::Nothing, |prev, next|{
	    match (prev, next){
		(Acmem::Nothing, other) => other,
		(other, Acmem::Nothing) => other,
		(Acmem::Chosen(i), _) |
		(Acmem::Marked(_), Acmem::Chosen(i)) => Acmem::Chosen(i),
		(Acmem::Marked(i), Acmem::Marked(_)) => Acmem::Marked(i),
	    }
	})
}

impl<F: FnMut(String)> Widget for AutocompletePopup<F>{
    fn ui(mut self, ui: &mut egui::Ui) -> egui::Response {
	if self.parent.gained_focus(){
	    ui.memory().open_popup(self.id())
	}
	let mut marked_value: Acmem = *ui.memory().data.get_temp_mut_or(self.id(), Acmem::Nothing);

	if let Some(popup) = egui::popup_below_widget(
	    ui,
	    self.id(),
	    &self.parent,
	    |ui| self.create_autocomplete_labels(ui, marked_value)) {
	    marked_value = match collect_clicked_and_hovered_labels(popup){
		Acmem::Chosen(i) => return { dbg!("click"); self.do_selection(i, ui) },
		m @ Acmem::Marked(_) => m,
		_ => marked_value
	    };

	    marked_value = self.update_mark_by_keyboard(ui, marked_value);

	    if let Acmem::Chosen(i) = marked_value{
		dbg!(marked_value);
		dbg!("enter");
		return self.do_selection(i, ui);
	    }

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
}
