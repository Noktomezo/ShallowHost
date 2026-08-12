#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[macro_use]
extern crate rust_i18n;

i18n!("src/locales", fallback = "en");

mod app;
mod domain;
mod infrastructure;
mod ui;

fn main() {
    app::run();
}
