pub(crate) mod components;
pub(crate) mod foundation;
pub(crate) mod pages;
pub(crate) mod shell;
pub(crate) mod state;

pub(crate) use foundation::assets::resolve_asset_path;
pub(crate) use shell::main_view::MainView;

pub(crate) fn init(cx: &mut gpui::App) {
    foundation::hover_motion::init(cx);
    components::cursor_tooltip::init(cx);
    pages::home::init(cx);
}
