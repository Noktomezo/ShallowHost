use gpui::prelude::*;
use gpui::*;
use gpui_updater::UpdateStatus;
use std::rc::Rc;
use std::time::Duration;

use crate::ui::components::badge::progress_ring;
use crate::ui::foundation::colors;
use crate::ui::foundation::motion::{UPDATE_PULSE_MOTION, update_pulse_opacity};
use crate::ui::resolve_asset_path;

pub type ToggleSidebarCallback = Rc<dyn Fn(&mut Window, &mut App)>;
pub type UpdateCallback = Rc<dyn Fn(&mut Window, &mut App)>;
pub type CloseCallback = Rc<dyn Fn(&mut Window, &mut App)>;

pub fn render_titlebar(
    is_maximized: bool,
    sidebar_progress: f32,
    update_status: &UpdateStatus,
    on_toggle_sidebar: ToggleSidebarCallback,
    on_update: UpdateCallback,
    on_close: CloseCallback,
    cx: &App,
) -> AnyElement {
    let update_button = titlebar_update_button(update_status, on_update, cx);
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
                    base_button("sidebar-toggle-btn", false, cx)
                        .on_mouse_down(MouseButton::Left, move |_, window, cx| {
                            window.prevent_default();
                            cx.stop_propagation();
                        })
                        .on_click(move |_, window, cx| on_toggle_sidebar(window, cx))
                        .child(sidebar_toggle_icon(sidebar_progress)),
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
                .gap_0()
                .pr(px(4.0))
                .flex_shrink_0()
                .children(update_button)
                .child(
                    base_button("win-minimize-btn", false, cx)
                        .window_control_area(WindowControlArea::Min)
                        .on_mouse_down(MouseButton::Left, |_, window, cx| {
                            window.prevent_default();
                            cx.stop_propagation();
                            window.minimize_window();
                        })
                        .child(titlebar_icon("assets/icons/window-minimize.svg")),
                )
                .child(
                    base_button("win-maximize-btn", false, cx)
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
                    base_button("win-close-btn", true, cx)
                        .window_control_area(WindowControlArea::Close)
                        .on_mouse_down(MouseButton::Left, |_, window, cx| {
                            window.prevent_default();
                            cx.stop_propagation();
                        })
                        .on_click(move |_, window, cx| on_close(window, cx))
                        .child(destructive_titlebar_icon(
                            "assets/icons/window-close.svg",
                            cx,
                        )),
                ),
        )
        .into_any_element()
}

fn titlebar_update_button(
    status: &UpdateStatus,
    on_update: UpdateCallback,
    cx: &App,
) -> Option<AnyElement> {
    let is_available = matches!(status, UpdateStatus::Available(_));
    let progress = match status {
        UpdateStatus::Downloading { downloaded, total } => total
            .filter(|total| *total > 0)
            .map_or(0.0, |total| download_progress(*downloaded, total)),
        _ => 0.0,
    };
    let is_downloading = matches!(status, UpdateStatus::Downloading { .. });
    let is_restarting = matches!(status, UpdateStatus::Installing | UpdateStatus::Staged(_));
    if !is_available && !is_downloading && !is_restarting {
        return None;
    }

    let icon = if is_downloading {
        progress_ring(progress)
    } else if is_restarting {
        restarting_icon()
    } else {
        svg()
            .external_path(resolve_asset_path("assets/icons/cloud-download.svg"))
            .size_4()
            .text_color(colors::orange())
            .with_animation(
                "titlebar-update-available-pulse",
                Animation::new(UPDATE_PULSE_MOTION).repeat(),
                |icon, delta| icon.opacity(update_pulse_opacity(delta)),
            )
            .into_any_element()
    };

    let id = ElementId::Name("titlebar-update-btn".into());
    let hover_key = SharedString::from("titlebar-button-titlebar-update-btn");
    let button = base_button_visual(id.clone(), false, &hover_key, cx)
        .when(is_available, |button| {
            button
                .on_mouse_down(MouseButton::Left, |_, window, cx| {
                    window.prevent_default();
                    cx.stop_propagation();
                })
                .on_click(move |_, window, cx| on_update(window, cx))
        })
        .when(!is_available, |button| button.cursor_default())
        .child(icon);

    Some(
        crate::ui::components::cursor_tooltip::attach_with_hover_motion(
            button,
            ElementId::Name("titlebar-update-tooltip".into()),
            hover_key,
            crate::ui::foundation::i18n::t(if is_restarting {
                "update.restarting"
            } else if is_downloading {
                "update.installing"
            } else {
                "update.availableTooltip"
            }),
        )
        .into_any_element(),
    )
}

fn restarting_icon() -> AnyElement {
    svg()
        .external_path(resolve_asset_path("assets/icons/refresh-cw.svg"))
        .size_4()
        .text_color(colors::orange())
        .with_animation(
            "titlebar-update-restarting",
            Animation::new(Duration::from_millis(850)).repeat(),
            |icon, delta| {
                icon.with_transformation(Transformation::rotate(Radians(
                    -std::f32::consts::TAU * delta,
                )))
            },
        )
        .into_any_element()
}

fn download_progress(downloaded: u64, total: u64) -> f32 {
    let percent = downloaded.saturating_mul(100) / total;
    let percent = u8::try_from(percent.min(100)).unwrap_or(100);
    f32::from(percent) / 100.0
}

fn base_button(id: &'static str, destructive: bool, cx: &App) -> Stateful<Div> {
    let hover_key = SharedString::from(format!("titlebar-button-{id}"));
    base_button_visual(ElementId::Name(id.into()), destructive, &hover_key, cx).on_hover(
        move |hovered, window, cx| {
            crate::ui::foundation::hover_motion::set_hovered(
                hover_key.clone(),
                *hovered,
                window,
                cx,
            );
        },
    )
}

fn base_button_visual(
    id: ElementId,
    destructive: bool,
    hover_key: &SharedString,
    cx: &App,
) -> Stateful<Div> {
    let hover = crate::ui::foundation::hover_motion::progress(hover_key, cx);
    div()
        .id(id)
        .when(destructive, |button| button.group("titlebar-destructive"))
        .size(px(32.0))
        .flex()
        .items_center()
        .justify_center()
        .rounded_md()
        .cursor_pointer()
        .bg(if destructive {
            colors::red().opacity(0.15 * hover)
        } else {
            rgba(0xffffff12).opacity(hover)
        })
        .text_color(crate::ui::foundation::motion::mix_color(
            colors::base_200(),
            if destructive {
                colors::red()
            } else {
                colors::base_100()
            },
            hover,
        ))
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

fn sidebar_toggle_icon(sidebar_progress: f32) -> Div {
    let expanded = sidebar_progress.clamp(0.0, 1.0);
    div()
        .relative()
        .size_4()
        .child(
            svg()
                .external_path(resolve_asset_path("assets/icons/panel-left-close.svg"))
                .size_4()
                .text_color(colors::base_200())
                .opacity(expanded),
        )
        .child(
            svg()
                .absolute()
                .inset_0()
                .external_path(resolve_asset_path("assets/icons/panel-left-open.svg"))
                .size_4()
                .text_color(colors::base_200())
                .opacity(1.0 - expanded),
        )
}

fn destructive_titlebar_icon(path: &'static str, cx: &App) -> Div {
    let path = resolve_asset_path(path);
    let hover = crate::ui::foundation::hover_motion::progress(
        &SharedString::from("titlebar-button-win-close-btn"),
        cx,
    );
    div()
        .relative()
        .size_4()
        .child(
            div()
                .id("titlebar-destructive-hover-icon")
                .absolute()
                .inset_0()
                .child(
                    svg()
                        .external_path(path.clone())
                        .size_4()
                        .text_color(colors::base_200())
                        .opacity(1.0 - hover),
                ),
        )
        .child(
            div()
                .id("titlebar-destructive-active-icon")
                .absolute()
                .inset_0()
                .group_active("titlebar-destructive", |style| style.invisible())
                .child(
                    svg()
                        .external_path(path.clone())
                        .size_4()
                        .text_color(colors::red())
                        .opacity(hover),
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

#[cfg(test)]
mod tests {
    use super::download_progress;

    #[test]
    fn download_progress_is_normalized_and_capped() {
        assert_eq!(download_progress(0, 100), 0.0);
        assert_eq!(download_progress(50, 100), 0.5);
        assert_eq!(download_progress(150, 100), 1.0);
    }
}
