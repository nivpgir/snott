
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

    let sync_dir: PathBuf = settings
	.get_string("sync_dir").as_ref()
	.map(tilde).unwrap().as_ref()
	.into();
    eframe::run_native(
        "Quick Snote",
        options,
        Box::new(|cc| {
            cc.egui_ctx.set_visuals(eframe::egui::Visuals::dark());
            Box::new(QuickSnote::new(sync_dir))
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
        .set_default("sync_dir",home_dir.display().to_string()).unwrap()
        .build()
        .unwrap()
}
