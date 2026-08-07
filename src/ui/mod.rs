pub mod assets;
pub mod audio_controls;
pub mod audio_dropdown;
pub mod badge;
pub mod card_header;
pub mod chain_operations;
pub mod colors;
pub mod control_style;
pub mod cursor_tooltip;
pub mod dropdown_overlay;
pub mod hover_motion;
pub mod i18n;
pub mod main_view;
pub mod motion;
pub mod navigation;
pub mod pages;
pub mod routes;
pub mod smooth_scroll;
pub mod titlebar;
pub mod toggle_switch;
pub mod volume_meter;

pub use assets::resolve_asset_path;
pub use main_view::MainView;

pub fn init(cx: &mut gpui::App) {
    hover_motion::init(cx);
    cursor_tooltip::init(cx);
    pages::home::init(cx);
}
