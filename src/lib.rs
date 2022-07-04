// hide console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![cfg_attr(debug_assertions, allow(dead_code))]

pub(crate) mod custom_window;
pub mod app;
pub(crate) mod autocomplete_popup;
pub mod snote;
pub mod quick_snote;
// pub mod quick_snote_main;
// mod snote_parser;
// mod snote_hightlighter;

// hide console window on Windows in release
