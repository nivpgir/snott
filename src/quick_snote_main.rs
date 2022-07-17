
use shellexpand::tilde;
use std::path::PathBuf;

use eframe::egui;
use snote2::quick_snote::QuickSnote;
use config::Config;

fn main() {
    let settings = build_config();
    let options = eframe::NativeOptions {
        decorated: false,
        transparent: true,
        min_window_size: Some(egui::vec2(320.0, 100.0)),
        resizable: true,
        ..Default::default()
    };


    eframe::run_native(
        "float-snote",
        options,
        Box::new(move |cc| {
	    let sync_dir: PathBuf = settings
		.get_string("sync_dir").as_ref()
		.map(tilde).unwrap().as_ref()
		.into();
	    let time_format = settings.get_string("timestamp_format").unwrap();
	    let body_font_size = settings.get_float("body_font_size").unwrap() as f32;
	    let headline_font_size = settings.get_float("headline_font_size").unwrap() as f32;
	    let mut s = eframe::egui::Style::default();
	    s.text_styles.insert(egui::TextStyle::Body, egui::FontId::proportional(body_font_size));
	    s.text_styles.insert(egui::TextStyle::Heading, egui::FontId::proportional(headline_font_size));
	    cc.egui_ctx.set_style(s);
	    cc.egui_ctx.set_visuals(eframe::egui::Visuals::dark());
            Box::new(QuickSnote::new(sync_dir)
		     .with_time_format(time_format))
        }),
    );
}

fn build_config() -> config::Config{
    let home_dir = dirs::home_dir().unwrap_or_default();
    let config_file = home_dir
	.join(".config")
	.join("snott")
	.join("config")
	.into();
    Config::builder()
        .add_source::<config::File<_,_>>(config_file)
        .add_source(config::Environment::with_prefix("SNOTT"))
        .set_default("sync_dir", home_dir.display().to_string()).unwrap()
        .set_default("timestamp_format", "%Y-%m-%d_%H-%M-%S").unwrap()
        .set_default("body_font_size", "20").unwrap()
        .set_default("headline_font_size", "40").unwrap()
        .build()
        .unwrap()
}
