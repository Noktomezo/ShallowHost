#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[macro_use]
extern crate rust_i18n;

i18n!("assets/locales", fallback = "ru");

mod app;
mod domain;
mod infrastructure;
mod ui;

fn main() {
    app::run();
}
