
use eframe::egui::{self, Modifiers, Widget, WidgetText};

#[derive(Debug)]
pub(crate) struct AutocompletePopup<C>
where
    C: Clone + Into<WidgetText>,
{
    items: Vec<C>,
    parent: egui::Response,
}

type Selection = Option<AutocompleteOutput<usize>>;

#[derive(Debug, PartialEq, Clone)]
pub enum AutocompleteOutput<C>{
    Chosen(C),
    Marked(C),
}

use AutocompleteOutput::*;
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

    fn create_autocomplete_labels(&self, ui: &mut egui::Ui, marked_value: &Selection)
				  -> Option<Vec<egui::Response>>{

	egui::popup_below_widget(ui, self.id(), &self.parent, |ui|{
	    self.items.iter().enumerate()
		.map(|(i, item)|{
		    ui.selectable_label(*marked_value == Some(Marked(i)) ,item.clone())
		})}.collect())
    }

    fn update_mark_by_keyboard(&mut self, mark: Selection, ui: &mut egui::Ui) -> Selection{
	use egui::Key;
	let ret = if ui.input_mut().consume_key(Modifiers::NONE, Key::ArrowUp){
	    mark.map(AutocompleteOutput::dec).or(Some(AutocompleteOutput::Marked(0)))
	} else if ui.input_mut().consume_key(Modifiers::NONE, Key::ArrowDown){
	    mark.map(AutocompleteOutput::inc).or(Some(AutocompleteOutput::Marked(0)))
	} else if ui.input_mut().consume_key(Modifiers::NONE, Key::Enter){
	    mark.map(AutocompleteOutput::into_choice)
		.or(Some(AutocompleteOutput::Chosen(0)))
	} else {
	    mark
	}.map(|c: AutocompleteOutput<_>|c.clamp(self.items.len()-1));
	ret
    }

    pub fn show(&mut self, ui: &mut egui::Ui) -> Option<AutocompleteOutput<C>>{
	if self.parent.gained_focus() || self.parent.changed(){
	    ui.memory().open_popup(self.id())
	}

	let mut marked_value = ui.memory().data.get_temp::<AutocompleteOutput<usize>>(self.id());

	marked_value = self.update_mark_by_keyboard(marked_value.clone(), ui);
	if let Some(label_responses) = self.create_autocomplete_labels(ui, &marked_value){
	    marked_value = update_mark_by_mouse_interaction(marked_value.clone(), label_responses);
	}

	if let Some(AutocompleteOutput::Chosen(_)) = &marked_value {
	    ui.memory().data.remove::<AutocompleteOutput<usize>>(self.id());
	    ui.memory().close_popup();
	    self.parent.request_focus();
	} else if let Some(mark) = marked_value.clone(){
	    ui.memory().data.insert_temp(self.id(), mark)
	}
	marked_value.map(|c|c.map(&self.items))
    }
}


fn check_mouse_interactions((i, response): (usize, egui::Response)) -> Selection{
    if response.clicked(){
	Some(AutocompleteOutput::Chosen(i))
    } else if response.ctx.input().pointer.is_moving() && response.hovered() {
	// TODO: BUG:
	// for some reason response.hovered() is always false
	Some(AutocompleteOutput::Marked(i))
    } else {
	None
    }
}


fn update_mark_by_mouse_interaction(current_mark: Selection, mouse_interactions: Vec<egui::Response>)
				    -> Selection{
    use std::ops::ControlFlow;
    let new_mark = mouse_interactions.into_iter().enumerate()
	.map(check_mouse_interactions)
	.try_fold(current_mark, |prev, next|{
	    if let (Some(p), Some(n)) = (prev.clone(), next.clone()){
		if let AutocompleteOutput::Chosen(i) = p.clone().update(n){
		    ControlFlow::Break(Some(AutocompleteOutput::Chosen(i)))
		} else{
		    ControlFlow::Continue(Some(p))
		}
	    } else {
		ControlFlow::Continue(prev.or(next))
	    }
	});
    match new_mark{
	// for some reason there is no way of getting the inner value without matching,
	// even when they both hold the same type, idk why...
	ControlFlow::Break(v) => v,
	ControlFlow::Continue(v) => v,
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
    pub(crate) fn clamp(self, max: usize) -> Self {
	match self {
	    Self::Chosen(i) => Self::Chosen(i.clamp(0, max)),
	    Self::Marked(i) => Self::Marked(i.clamp(0, max)),
	}
    }

    pub(crate) fn into_choice(self) -> Self {
        match self{
	    Self::Marked(i) => Self::Chosen(i),
	    choice => choice
	}
    }

    pub(crate) fn update(self, other: Self) -> Self{
	match (self, other){
	    (Self::Chosen(i), _) |
	    (Self::Marked(_), Self::Chosen(i)) => Self::Chosen(i),
	    (Self::Marked(i), Self::Marked(_)) => Self::Marked(i),
	}
    }
    fn map<C>(self, items: &[C]) -> AutocompleteOutput<C>
    where C: Clone + Into<WidgetText> + std::fmt::Debug{
	match self{
	    Self::Chosen(i) => AutocompleteOutput::Chosen(items[i].clone()),
	    Self::Marked(i) => AutocompleteOutput::Marked(items[i].clone()),
	}

    }
}
