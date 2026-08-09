use gpui::prelude::*;
use gpui::*;
use gpui_component::StyledExt;

use crate::infrastructure::config::PluginSettings;
use crate::ui::components::smooth_scroll::SmoothVerticalScroll;
use crate::ui::foundation::control_style::ControlTypography;
use crate::ui::foundation::motion::{DIALOG_MOTION, mix_color};
use crate::ui::foundation::{colors, i18n};
use crate::ui::resolve_asset_path;
use crate::ui::shell::routes::{DropdownCallbacks, PluginPathKind, PluginPathUpdate};

pub(super) fn render_scan_paths_dialog(
    settings: &PluginSettings,
    callbacks: &DropdownCallbacks,
    open: bool,
    revision: u64,
) -> AnyElement {
    let close_overlay = callbacks.on_set_scan_paths_open.clone();
    let close_button = callbacks.on_set_scan_paths_open.clone();
    let reset = callbacks.on_update_plugin_path.clone();
    let closing = !open;
    let dialog_animation_id =
        ElementId::NamedInteger(SharedString::from("scan-paths-dialog-motion"), revision);
    let overlay_animation_id =
        ElementId::NamedInteger(SharedString::from("scan-paths-overlay-motion"), revision);

    let dialog = div()
        .id("scan-paths-dialog")
        .w(px(560.0))
        .max_h(px(620.0))
        .flex()
        .flex_col()
        .bg(colors::base_950())
        .border_1()
        .border_color(colors::base_800())
        .rounded_lg()
        .shadow_lg()
        .on_click(|_, _, cx| cx.stop_propagation())
        .child(dialog_header())
        .child(separator())
        .child(
            div()
                .p_4()
                .flex()
                .flex_col()
                .gap_4()
                .child(path_section(
                    PluginPathKind::Vst2,
                    "vst2",
                    "plugins.vst2SearchPaths",
                    "plugins.noVst2Paths",
                    &settings.vst2_paths,
                    callbacks,
                ))
                .child(path_section(
                    PluginPathKind::Vst3,
                    "vst3",
                    "plugins.vst3SearchPaths",
                    "plugins.noVst3Paths",
                    &settings.vst3_paths,
                    callbacks,
                )),
        )
        .child(separator())
        .child(
            div()
                .p_4()
                .flex()
                .items_center()
                .justify_between()
                .child(
                    text_button(
                        "scan-paths-reset",
                        "assets/icons/refresh-cw.svg",
                        i18n::t("plugins.resetDefaults"),
                        false,
                    )
                    .on_click(move |_, window, cx| {
                        reset(PluginPathUpdate::Reset, window, cx);
                    }),
                )
                .child(
                    text_button(
                        "scan-paths-done",
                        "assets/icons/check.svg",
                        i18n::t("plugins.done"),
                        true,
                    )
                    .on_click(move |_, window, cx| close_button(false, window, cx)),
                ),
        )
        .with_animation(
            dialog_animation_id,
            Animation::new(DIALOG_MOTION).with_easing(ease_in_out),
            move |dialog, delta| {
                let progress = if closing { 1.0 - delta } else { delta };
                dialog.opacity(progress).mt(px(8.0 * (1.0 - progress)))
            },
        );

    div()
        .id("scan-paths-overlay")
        .absolute()
        .inset_0()
        .flex()
        .items_center()
        .justify_center()
        .occlude()
        .on_scroll_wheel(|_, _, cx| cx.stop_propagation())
        .on_click(move |_, window, cx| close_overlay(false, window, cx))
        .child(dialog)
        .with_animation(
            overlay_animation_id,
            Animation::new(DIALOG_MOTION).with_easing(ease_in_out),
            move |overlay, delta| {
                let progress = if closing { 1.0 - delta } else { delta };
                overlay.bg(mix_color(rgba(0x100f0f00), rgba(0x100f0f99), progress))
            },
        )
        .into_any_element()
}

fn dialog_header() -> AnyElement {
    div()
        .px_4()
        .py_3()
        .flex()
        .flex_col()
        .gap(px(2.0))
        .child(
            div()
                .text_base()
                .font_semibold()
                .text_color(colors::base_200())
                .child(i18n::t("plugins.scanPathsTitle")),
        )
        .child(
            div()
                .text_xs()
                .text_color(colors::base_500())
                .child(i18n::t("plugins.scanPathsDescription")),
        )
        .into_any_element()
}

fn path_section(
    kind: PluginPathKind,
    section_id: &'static str,
    title: &'static str,
    empty_text: &'static str,
    paths: &[String],
    callbacks: &DropdownCallbacks,
) -> AnyElement {
    let picker = callbacks.on_pick_plugin_path.clone();
    let update = callbacks.on_update_plugin_path.clone();
    let list_height = path_list_height(paths.len());
    let scroll_id = SharedString::from(format!("{section_id}-paths-scroll"));
    let list_content = div()
        .w_full()
        .py_2()
        .px_4()
        .flex()
        .flex_col()
        .gap(px(6.0))
        .when(paths.is_empty(), |element| {
            element.child(
                div()
                    .h(px(32.0))
                    .flex_none()
                    .flex()
                    .items_center()
                    .justify_center()
                    .text_xs()
                    .text_color(colors::base_500())
                    .child(i18n::t(empty_text)),
            )
        })
        .children(paths.iter().enumerate().map(move |(index, path)| {
            let update = update.clone();
            let path_for_remove = path.clone();
            div()
                .id(SharedString::from(format!("{section_id}-path-{index}")))
                .h(px(32.0))
                .flex_none()
                .px_2()
                .flex()
                .items_center()
                .justify_between()
                .gap_2()
                .bg(colors::base_900())
                .rounded_sm()
                .child(
                    div()
                        .min_w_0()
                        .flex_1()
                        .truncate()
                        .text_xs()
                        .text_color(colors::base_300())
                        .child(path.clone()),
                )
                .child(
                    div()
                        .id(SharedString::from(format!(
                            "{section_id}-remove-path-{index}"
                        )))
                        .size(px(26.0))
                        .flex_none()
                        .flex()
                        .items_center()
                        .justify_center()
                        .cursor_pointer()
                        .rounded_sm()
                        .hover(|style| style.bg(colors::red().opacity(0.16)))
                        .on_click(move |_, window, cx| {
                            update(
                                PluginPathUpdate::Remove {
                                    kind,
                                    path: path_for_remove.clone(),
                                },
                                window,
                                cx,
                            );
                        })
                        .child(
                            svg()
                                .external_path(resolve_asset_path("assets/icons/trash-2.svg"))
                                .size(px(15.0))
                                .text_color(colors::red()),
                        ),
                )
        }));

    div()
        .flex()
        .flex_col()
        .gap_2()
        .child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .child(
                    div()
                        .text_sm()
                        .font_semibold()
                        .text_color(colors::base_200())
                        .child(i18n::t(title)),
                )
                .child(
                    text_button(
                        SharedString::from(format!("{section_id}-add-path")),
                        "assets/icons/plus.svg",
                        i18n::t("plugins.addFolder"),
                        false,
                    )
                    .on_click(move |_, window, cx| picker(kind, window, cx)),
                ),
        )
        .child(
            div()
                .w_full()
                .h(list_height)
                .flex_none()
                .overflow_hidden()
                .bg(colors::base_900().opacity(0.45))
                .border_1()
                .border_color(colors::base_800())
                .rounded_md()
                .child(SmoothVerticalScroll::new(scroll_id, list_content)),
        )
        .into_any_element()
}

fn path_list_height(path_count: usize) -> Pixels {
    px(match path_count.clamp(1, 4) {
        1 => 50.0,
        2 => 88.0,
        3 => 126.0,
        _ => 164.0,
    })
}

fn text_button(
    id: impl Into<ElementId>,
    icon_path: &'static str,
    label: impl Into<SharedString>,
    primary: bool,
) -> Stateful<Div> {
    let foreground = if primary {
        colors::accent_foreground()
    } else {
        colors::base_200()
    };
    div()
        .id(id)
        .h(px(34.0))
        .px_3()
        .flex()
        .items_center()
        .gap_2()
        .flex_none()
        .cursor_pointer()
        .bg(if primary {
            colors::orange()
        } else {
            colors::base_900()
        })
        .border_1()
        .border_color(if primary {
            colors::orange()
        } else {
            colors::base_800()
        })
        .rounded_md()
        .control_text()
        .text_color(foreground)
        .hover(move |style| {
            if primary {
                style.border_color(colors::accent_foreground().opacity(0.45))
            } else {
                style.bg(colors::base_850())
            }
        })
        .child(
            svg()
                .external_path(resolve_asset_path(icon_path))
                .size(px(15.0))
                .text_color(foreground),
        )
        .child(label.into())
}

fn separator() -> Div {
    div().h(px(1.0)).w_full().bg(colors::base_800())
}

#[cfg(test)]
mod tests {
    use super::path_list_height;
    use gpui::px;

    #[test]
    fn path_list_height_never_exposes_a_partial_row() {
        assert_eq!(path_list_height(0), px(50.0));
        assert_eq!(path_list_height(2), px(88.0));
        assert_eq!(path_list_height(3), px(126.0));
        assert_eq!(path_list_height(4), px(164.0));
        assert_eq!(path_list_height(9), px(164.0));
    }
}
