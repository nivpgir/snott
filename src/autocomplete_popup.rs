
use eframe::{egui::{self, Modifiers, Widget, WidgetText}, emath::NumExt};

#[derive(Debug)]
pub(crate) struct AutocompletePopup<C>
where
    C: Clone + Into<WidgetText>,
{
    items: Vec<C>,
    parent: egui::Response,
}

type Selection = AutocompleteOutput<usize>;

#[derive(Debug, PartialEq, Clone)]
pub enum AutocompleteOutput<C>{
    Chosen(C),
    Marked(C),
}

impl <T: Default> Default for AutocompleteOutput<T>{
    fn default() -> Self {
        Marked(T::default())
    }
}

impl<T: Copy> Copy for AutocompleteOutput<T> {}

use AutocompleteOutput::*;

// #[derive(Clone, Default)]
// struct AutocompleteState{
//     selection: Selection,
//     scroll_offset: f32
// }

impl<C> AutocompletePopup<C>
where
    C: Clone + Into<WidgetText> + std::fmt::Debug,

{
    pub fn id (&self) -> egui::Id{
	self.parent.id.with("::ac")
    }

    pub fn new(items: impl IntoIterator<Item=C>,
	       parent: &egui::Response) -> Self{
	Self{
	    items: items.into_iter().collect(),
	    parent: parent.clone(),
	}
    }

    fn update_selection_by_keyboard(&self, selection: Selection, ui: &mut egui::Ui) -> Selection{
	use egui::Key;
	if ui.input_mut().consume_key(Modifiers::NONE, Key::ArrowUp){
	    selection.dec()
	} else if ui.input_mut().consume_key(Modifiers::NONE, Key::ArrowDown){
	    selection.inc()
	} else if ui.input_mut().consume_key(Modifiers::NONE, Key::Enter){
	    selection.into_choice()
	} else {
	    selection
	}.map(|c|c.at_most(self.items.len().saturating_sub(1)))
    }

    pub fn show(&mut self, ui: &mut egui::Ui) -> Option<AutocompleteOutput<C>>{
	if self.parent.gained_focus() || self.parent.changed(){
	    ui.memory().open_popup(self.id());
	    if self.items.is_empty(){
		ui.memory().close_popup();
	    }
	}

	let popup_response = egui::popup_below_widget(
	    ui, self.id(), &self.parent, self.make_completion_widget()
	);

	if let Some(Chosen(_)) = dbg!(popup_response) {
	    ui.memory().data.remove::<AutocompleteOutput<usize>>(self.id());
	    ui.memory().close_popup();
	    self.parent.request_focus();
	} else if let Some(selection) = popup_response{
	    ui.memory().data.insert_temp(self.id(), selection)
	}
	popup_response.map(|c|c.map(|i|self.items[i].clone()))
    }

    fn make_completion_widget(&self) -> impl FnOnce(&mut egui::Ui) -> Selection + '_{
	|ui: &mut egui::Ui|{
	    let prev_selection = *ui.memory().data
		.get_temp_mut_or_default::<AutocompleteOutput<usize>>(self.id());
	    let prev_scroll = *ui.memory().data.get_temp_mut_or_default::<f32>(self.id());
	    let spacing = ui.spacing().item_spacing.y;
	    let row_height = ui.text_style_height(&egui::TextStyle::Body);
	    let selection_pos = prev_selection.into_inner() as f32 *
		(row_height + spacing);
	    let rect_height = ui.max_rect().height() - ui.max_rect().top();
	    let new_scroll = prev_scroll
		.at_least(selection_pos - rect_height + row_height)
		.at_most(selection_pos);
	    let selected_index = prev_selection.into_inner();
	    let scroll = egui::ScrollArea::vertical()
		.vertical_scroll_offset(new_scroll)
		.show_rows(ui, row_height, self.items.len(), |ui, rows|{
		    rows.map(|row_num|(row_num, ui.selectable_label(selected_index == row_num, self.items[row_num].clone())))
			.filter_map(check_mouse_interactions)
			.find(|s|s.is_chosen())
		});
	    let keyboard_selection = self.update_selection_by_keyboard(prev_selection, ui);
	    ui.memory().data.insert_temp(self.id(), scroll.state.offset.y);
	    scroll.inner.unwrap_or(keyboard_selection)
	}
    }
}

fn check_mouse_interactions((i, response): (usize, egui::Response)) -> Option<Selection>{
    if response.clicked(){
	Some(Chosen(i))
    } else if response.ctx.input().pointer.is_moving() && response.hovered() {
	// TODO: BUG:
	// for some reason response.hovered() is always false
	Some(Marked(i))
    } else {
	None
    }
}


impl<C> Widget for AutocompletePopup<C>
where
    C: Clone + Into<WidgetText> + std::fmt::Debug{

    fn ui(mut self, ui: &mut egui::Ui) -> egui::Response {

	self.show(ui);
	self.parent
    }
}



impl AutocompleteOutput<usize> {

    pub(crate) fn dec(self) -> Self {
	match self {
	    Self::Chosen(i) => Self::Chosen(i.saturating_sub(1)),
	    Self::Marked(i) => Self::Marked(i.saturating_sub(1)),
	}
    }

    pub(crate) fn inc(self) -> Self {
	match self {
	    Self::Chosen(i) => Self::Chosen(i.saturating_add(1)),
	    Self::Marked(i) => Self::Marked(i.saturating_add(1)),
	}
    }

    fn is_chosen(&self) -> bool {
        matches!(self, Chosen(_))
    }
}

impl<T> AutocompleteOutput<T> {
    pub(crate) fn into_choice(self) -> Self {
        match self{
	    Self::Marked(i) => Self::Chosen(i),
	    choice => choice
	}
    }

    fn map<C, F: FnOnce(T) -> C>(self, f: F) -> AutocompleteOutput<C>{
	match self{
	    Self::Chosen(i) => Chosen(f(i)),
	    Self::Marked(i) => Marked(f(i)),
	}
    }

    pub fn into_inner(self) -> T{
	match self{
	    Self::Chosen(i) |
	    Self::Marked(i) => i,
	}
    }
}
