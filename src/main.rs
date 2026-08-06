#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[macro_use]
extern crate rust_i18n;

i18n!("locales", fallback = "ru");

mod config;
mod engine;
mod system_integration;
mod ui;
mod updater;

use config::ConfigStore;
use engine::Engine;
use gpui::*;
use gpui_component::{Root, Theme};
use std::sync::Arc;
use ui::MainView;

fn load_fonts(cx: &mut App) {
    cx.text_system()
        .add_fonts(vec![
            std::borrow::Cow::Borrowed(include_bytes!(
                "../assets/fonts/IBM Plex Sans/static/IBMPlexSans-Regular.ttf"
            )),
            std::borrow::Cow::Borrowed(include_bytes!(
                "../assets/fonts/IBM Plex Sans/static/IBMPlexSans-Medium.ttf"
            )),
            std::borrow::Cow::Borrowed(include_bytes!(
                "../assets/fonts/IBM Plex Sans/static/IBMPlexSans-SemiBold.ttf"
            )),
            std::borrow::Cow::Borrowed(include_bytes!(
                "../assets/fonts/IBM Plex Sans/static/IBMPlexSans-Bold.ttf"
            )),
            std::borrow::Cow::Borrowed(include_bytes!(
                "../assets/fonts/IBM Plex Sans/static/IBMPlexSans-Italic.ttf"
            )),
        ])
        .expect("failed to add static IBM Plex Sans fonts");
}

fn main() {
    let app = Application::with_platform(gpui_platform::current_platform(false))
        .with_assets(gpui_component_assets::Assets);

    app.run(|cx: &mut App| {
        load_fonts(cx);
        gpui_component::init(cx);
        ui::init(cx);

        let storage = match ConfigStore::beside_executable() {
            Ok(storage) => storage,
            Err(error) => {
                eprintln!("failed to initialize portable storage: {error}");
                return;
            }
        };
        let engine = match Engine::new(storage.cache_dir()) {
            Ok(engine) => Arc::new(engine),
            Err(error) => {
                eprintln!("failed to initialize JUCE: {error}");
                return;
            }
        };

        Theme::global_mut(cx).font_family = "IBM Plex Sans".into();

        let bounds = Bounds::centered(None, size(px(900.0), px(700.0)), cx);
        let window_background = if storage.config().transparent_shell {
            WindowBackgroundAppearance::Blurred
        } else {
            WindowBackgroundAppearance::Opaque
        };

        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                window_min_size: Some(size(px(900.0), px(700.0))),
                titlebar: None,
                window_background,
                kind: WindowKind::Normal,
                window_decorations: Some(WindowDecorations::Server),
                ..Default::default()
            },
            move |window, cx| {
                let engine = Arc::clone(&engine);
                let view = cx.new(|cx| MainView::new(engine, storage, window, cx));
                cx.new(|cx| {
                    Root::new(view, window, cx)
                        // The app view owns the opaque/50%-tinted surfaces. Keeping Root clear
                        // lets Windows Acrylic remain visible through those alpha layers.
                        .bg(rgba(0x00000000))
                })
            },
        )
        .unwrap();
    });
}
