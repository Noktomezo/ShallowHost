use gpui::prelude::*;
use gpui::*;
use std::rc::Rc;

use super::{colors, resolve_asset_path};

pub type ToggleSidebarCallback = Rc<dyn Fn(&mut Window, &mut App)>;
pub type CloseCallback = Rc<dyn Fn(&mut Window, &mut App)>;

pub fn render_titlebar(
    is_maximized: bool,
    on_toggle_sidebar: ToggleSidebarCallback,
    on_close: CloseCallback,
) -> AnyElement {
    div()
        .id("titlebar")
        .h(px(40.0))
        .w_full()
        .flex()
        .items_center()
        .justify_between()
        .window_control_area(WindowControlArea::Drag)
        .child(
            div()
                .size(px(40.0))
                .flex()
                .items_center()
                .justify_center()
                .flex_shrink_0()
                .child(
                    base_button("sidebar-toggle-btn", false)
                        .on_mouse_down(MouseButton::Left, move |_, window, cx| {
                            window.prevent_default();
                            cx.stop_propagation();
                        })
                        .on_click(move |_, window, cx| on_toggle_sidebar(window, cx))
                        .child(titlebar_icon("assets/icons/panel-left.svg")),
                ),
        )
        .child(
            div()
                .flex_1()
                .h_full()
                .window_control_area(WindowControlArea::Drag),
        )
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(4.0))
                .pr(px(4.0))
                .flex_shrink_0()
                .child(
                    base_button("win-minimize-btn", false)
                        .window_control_area(WindowControlArea::Min)
                        .on_mouse_down(MouseButton::Left, |_, window, cx| {
                            window.prevent_default();
                            cx.stop_propagation();
                            window.minimize_window();
                        })
                        .child(titlebar_icon("assets/icons/window-minimize.svg")),
                )
                .child(
                    base_button("win-maximize-btn", false)
                        .window_control_area(WindowControlArea::Max)
                        .on_mouse_down(MouseButton::Left, |_, window, cx| {
                            window.prevent_default();
                            cx.stop_propagation();
                            if window.is_maximized() {
                                restore_active_window();
                            } else {
                                window.zoom_window();
                            }
                        })
                        .child(titlebar_icon(if is_maximized {
                            "assets/icons/window-restore.svg"
                        } else {
                            "assets/icons/window-maximize.svg"
                        })),
                )
                .child(
                    base_button("win-close-btn", true)
                        .window_control_area(WindowControlArea::Close)
                        .on_mouse_down(MouseButton::Left, move |_, window, cx| {
                            window.prevent_default();
                            cx.stop_propagation();
                            on_close(window, cx);
                        })
                        .child(destructive_titlebar_icon("assets/icons/window-close.svg")),
                ),
        )
        .into_any_element()
}

fn base_button(id: &'static str, destructive: bool) -> Stateful<Div> {
    div()
        .id(id)
        .when(destructive, |button| button.group("titlebar-destructive"))
        .size(px(32.0))
        .flex()
        .items_center()
        .justify_center()
        .rounded_md()
        .cursor_pointer()
        .text_color(colors::base_200())
        .hover(move |style| {
            if destructive {
                style
                    .bg(colors::red().opacity(0.15))
                    .text_color(colors::red())
            } else {
                style.bg(rgba(0xffffff12)).text_color(colors::base_100())
            }
        })
        .active(move |style| {
            if destructive {
                style
                    .bg(colors::red())
                    .text_color(colors::accent_foreground())
            } else {
                style.bg(rgba(0xffffff20)).text_color(colors::base_100())
            }
        })
}

fn titlebar_icon(path: &'static str) -> Svg {
    svg()
        .external_path(resolve_asset_path(path))
        .size_4()
        .text_color(colors::base_200())
}

fn destructive_titlebar_icon(path: &'static str) -> Div {
    let path = resolve_asset_path(path);
    div()
        .relative()
        .size_4()
        .child(
            div()
                .id("titlebar-destructive-hover-icon")
                .absolute()
                .inset_0()
                .group_hover("titlebar-destructive", |style| style.invisible())
                .child(
                    svg()
                        .external_path(path.clone())
                        .size_4()
                        .text_color(colors::base_200()),
                ),
        )
        .child(
            div()
                .id("titlebar-destructive-active-icon")
                .absolute()
                .inset_0()
                .invisible()
                .group_hover("titlebar-destructive", |style| style.visible())
                .group_active("titlebar-destructive", |style| style.invisible())
                .child(
                    svg()
                        .external_path(path.clone())
                        .size_4()
                        .text_color(colors::red()),
                ),
        )
        .child(
            div()
                .id("titlebar-destructive-pressed-icon")
                .absolute()
                .inset_0()
                .invisible()
                .group_active("titlebar-destructive", |style| style.visible())
                .child(
                    svg()
                        .external_path(path)
                        .size_4()
                        .text_color(colors::accent_foreground()),
                ),
        )
}

#[cfg(target_os = "windows")]
fn restore_active_window() {
    use std::ffi::c_void;

    const SW_RESTORE: i32 = 9;

    #[link(name = "user32")]
    unsafe extern "system" {
        fn GetActiveWindow() -> *mut c_void;
        fn ShowWindowAsync(window: *mut c_void, command: i32) -> i32;
    }

    // SAFETY: Both declarations match the Win32 ABI. This handler runs on the UI thread that
    // owns the active app window; the handle is checked before being passed back to user32.
    unsafe {
        let window = GetActiveWindow();
        if !window.is_null() {
            ShowWindowAsync(window, SW_RESTORE);
        }
    }
}

#[cfg(not(target_os = "windows"))]
fn restore_active_window() {}
