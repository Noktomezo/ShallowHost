use std::sync::Arc;

use gpui::*;

use crate::infrastructure::config::ConfigStore;
use crate::infrastructure::engine::Engine;
use crate::infrastructure::single_instance::{AcquireResult, SingleInstance};
use crate::ui::{self, MainView};

#[cfg(debug_assertions)]
const APP_ID: &str = "Noktomezo.ShallowHost.Dev";
#[cfg(not(debug_assertions))]
const APP_ID: &str = "Noktomezo.ShallowHost";
#[cfg(debug_assertions)]
pub(crate) const APP_TITLE: &str = "ShallowHost (Dev)";
#[cfg(not(debug_assertions))]
pub(crate) const APP_TITLE: &str = "ShallowHost";

pub(crate) fn run() {
    let single_instance = match SingleInstance::acquire(APP_ID) {
        Ok(AcquireResult::Primary(instance)) => instance,
        Ok(AcquireResult::Secondary) => return,
        Err(error) => {
            eprintln!("failed to initialize the single-instance guard: {error}");
            return;
        }
    };

    let app = Application::with_platform(gpui_platform::current_platform(false))
        .with_assets(crate::ui::EmbeddedAssets);

    app.run(move |cx: &mut App| {
        cx.set_app_identity(APP_ID, APP_TITLE);
        load_fonts(cx);
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

        let bounds = Bounds::centered(None, size(px(900.0), px(700.0)), cx);
        let window_background = if storage.config().transparent_shell {
            WindowBackgroundAppearance::Blurred
        } else {
            WindowBackgroundAppearance::Opaque
        };

        if let Err(error) = cx.open_window(
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
                window.set_window_title(APP_TITLE);
                let engine = Arc::clone(&engine);
                cx.new(|cx| MainView::new(engine, storage, single_instance, window, cx))
            },
        ) {
            eprintln!("failed to open the application window: {error}");
        }
    });
}

fn load_fonts(cx: &mut App) {
    if let Err(error) = cx.text_system().add_fonts(vec![
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
    ]) {
        eprintln!("failed to add static IBM Plex Sans fonts: {error}");
    }
}
