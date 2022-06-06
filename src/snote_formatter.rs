use std::sync::Arc;

use chumsky::Parser;
use eframe::egui;

use crate::snote;


fn simple_text_layout(ui: &egui::Ui, text: &str) -> Arc<egui::Galley>{
    ui.fonts().layout_job(
	egui::text::LayoutJob::simple(text.to_string(),
				      egui::TextStyle::Monospace.resolve(&ui.style()),
				      egui::Color32::BLACK,
				      0.0
	))
}


pub fn snote_layouter(ui: &egui::Ui, text: &str, _wrap_width: f32) -> Arc<egui::Galley>{
    snote::snote().parse(text)
        .map(|blocks|{
	    let mut job = egui::text::LayoutJob::default();
	    blocks.iter().for_each(|b|{
		let s = match b{
		    snote::SNote::NewlinePadding(l) => "\n".repeat(*l),
		    snote::SNote::Block(st) => st.clone(),
		    snote::SNote::Headline(st) => st.clone()

		};
		job.append(&s, 0.0,
			   egui::TextFormat::simple(
			       egui::TextStyle::Heading.resolve(ui.style()),
			       egui::Color32::DARK_BLUE)
		);
	    });
	    ui.fonts().layout_job(job)
    }).ok().unwrap_or_else(||simple_text_layout(ui, text))
}
