use std::fmt::Debug;

use eframe::{egui::{self, WidgetText}, emath::NumExt};

#[derive(Debug)]
pub(crate) struct AutocompletePopup<C>
where
    C: Clone + Into<WidgetText>,
{
    items: Vec<C>,
    id: egui::Id,
    response: egui::Response,
}

#[derive(Debug, PartialEq, Clone)]
pub enum AutocompleteOutput<C> {
    Chosen(C),
    Marked(C),
}

impl<T: Default> Default for AutocompleteOutput<T> {
    fn default() -> Self {
        Marked(T::default())
    }
}

impl<T: Copy> Copy for AutocompleteOutput<T> {}

use AutocompleteOutput::*;

type Selection = AutocompleteOutput<usize>;
impl<C> AutocompletePopup<C>
where
    C: Clone + Into<WidgetText> + std::fmt::Debug,
{
    pub fn new(items: impl IntoIterator<Item = C>, parent: egui::Response) -> Self {
        Self {
            items: items.into_iter().collect(),
            id: parent.id.with("::ac"),
	    response: parent
        }
    }

    fn update_selection_by_keyboard(selection: Selection, ui: &mut egui::Ui) -> Selection {
        use egui::{Key, Modifiers};
        if ui.input_mut().consume_key(Modifiers::NONE, Key::ArrowUp) {
            selection.dec()
        } else if ui.input_mut().consume_key(Modifiers::NONE, Key::ArrowDown) {
            selection.inc()
        } else if ui.input_mut().consume_key(Modifiers::NONE, Key::Enter) {
            selection.into_choice()
        } else {
            selection
        }
    }

    pub fn make_completion_widget(&self)
			      -> impl FnOnce(&mut egui::Ui) -> Option<AutocompleteOutput<C>> + '_ {
        move |ui: &mut egui::Ui| {

            let prev_selection = *ui
                .memory()
                .data
                .get_temp_mut_or_default::<Selection>(self.id);

            // draw
            let row_height = ui.text_style_height(&egui::TextStyle::Body);
            let mut scroll = egui::ScrollArea::vertical().show_rows(
                ui,
                row_height,
                self.items.len(),
                |ui, rows| {
                    rows.clone()
                        .map(|row_num| self.draw_label(ui, prev_selection.into_inner(), row_num))
                        .zip(rows)
                        .filter_map(check_mouse_interactions)
                        .find(|s| s.is_chosen())
                },
            );

            // move selection to viewed area after scroll
            let row_height_with_spacing = row_height + ui.spacing().item_spacing.y;
            let rect_height = scroll.inner_rect.height() - scroll.inner_rect.top();
            let clamped = {
                let new_offset = scroll.state.offset.y;
                let min_allowed_row = (new_offset / row_height_with_spacing).ceil() as usize;
                let max_allowed_row =
                    ((new_offset + rect_height) / row_height_with_spacing).floor() as usize;
                prev_selection.map(|i| i.clamp(min_allowed_row, max_allowed_row))
            };

            // update selection by mouse and keyboard
            let keyboard = Self::update_selection_by_keyboard(clamped, ui);
            let new_selection = if keyboard != clamped {
                keyboard
            } else {
                scroll.inner.unwrap_or(clamped)
            };

            // update viewed scroll area after updating the selection
            let selection_pos = new_selection.into_inner() as f32 * row_height_with_spacing;
            scroll.state.offset.y = scroll
                .state
                .offset
                .y
                .at_least(selection_pos - rect_height + row_height_with_spacing)
                .at_most(selection_pos);
            scroll.state.store(ui.ctx(), scroll.id);

            let new_selection =
		new_selection.map(|i| i.at_most(self.items.len().saturating_sub(1)));
	    if let Chosen(_) = new_selection {
		ui.memory().data.remove::<Selection>(self.id);
		ui.memory().close_popup();
	    } else {
		ui.memory().data.insert_temp(self.id, new_selection)
	    }
	    self.items
		.get(new_selection.into_inner())
		.map(|v| new_selection.map(|_| v.clone()))
        }
    }

    fn draw_label(&self, ui: &mut egui::Ui, selected_num: usize, row_num: usize) -> egui::Response {
        let is_marked = selected_num == row_num;
        let item = self.items[row_num].clone();
        ui.selectable_label(is_marked, item)
    }
}

fn check_mouse_interactions((response, i): (egui::Response, usize)) -> Option<Selection> {
    if response.clicked() {
        Some(Chosen(i))
    } else if response.ctx.input().pointer.is_moving() && response.hovered() {
        // TODO: BUG:
        // for some reason response.hovered() is always false
        Some(Marked(i))
    } else {
        None
    }
}

impl<C: Into<WidgetText> + Clone + Debug> egui::Widget for AutocompletePopup<C>{
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let w = self.make_completion_widget();
	w(ui);
	self.response
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
        match self {
            Self::Marked(i) => Self::Chosen(i),
            choice => choice,
        }
    }

    fn map<C, F: FnOnce(T) -> C>(self, f: F) -> AutocompleteOutput<C> {
        match self {
            Self::Chosen(i) => Chosen(f(i)),
            Self::Marked(i) => Marked(f(i)),
        }
    }

    pub fn into_inner(self) -> T {
        match self {
            Self::Chosen(i) | Self::Marked(i) => i,
        }
    }
}
