use std::sync::Arc;

use chumsky::Parser;
use eframe::egui::{self, TextFormat};

use super::{SNoteSection, snote};


fn simple_text_layout(ui: &egui::Ui, text: &str) -> egui::text::LayoutJob {
    let format = simple_format(ui);
    egui::text::LayoutJob::simple(
        text.to_string(),
        format.font_id,
        format.color,
        ui.available_width(),
    )
}
pub fn snote_layouter(ui: &egui::Ui, text: &str, _wrap_width: f32) -> Arc<egui::Galley> {
    let job = snote()
        .parse(text)
        .map(|sections| {
            let layout_sections = sections.iter().map(|section| {
		eframe::epaint::text::LayoutSection{
		    leading_space: 0.0,
		    byte_range: section.span(),
		    format: section.highlight_format(ui)
		}
            }).collect();
            egui::text::LayoutJob{
		text: text.to_string(),
		sections: layout_sections,
		..Default::default()
	    }

        })
        .ok()
        .unwrap_or_else(|| simple_text_layout(ui, text));
    ui.fonts().layout_job(job)
}

impl SNoteSection {
    fn highlight_format(&self, ui: &egui::Ui) -> TextFormat {
        match self {
            SNoteSection::Paragraph(_) => Self::paragraph_format(ui),
            SNoteSection::Headline(_) => Self::headline_format(ui),
        }
    }

    fn headline_format(ui: &egui::Ui) -> TextFormat {
        let color = ui.style().visuals.strong_text_color();
        TextFormat {
            font_id: egui::TextStyle::Heading.resolve(ui.style()),
            color,
            underline: egui::Stroke::new(1.0, color),
            ..Default::default()
        }
    }

    fn paragraph_format(ui: &egui::Ui) -> TextFormat {
        simple_format(ui)
    }
}

#[inline]
fn simple_format(ui: &egui::Ui) -> TextFormat {
    TextFormat{
	font_id: egui::TextStyle::Body.resolve(ui.style()),
        color: ui.style().visuals.text_color(),
	..Default::default()
    }
}
