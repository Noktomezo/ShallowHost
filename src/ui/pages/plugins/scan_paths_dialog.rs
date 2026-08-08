use gpui::prelude::*;
use gpui::*;
use gpui_component::StyledExt;
use gpui_component::scroll::ScrollableElement;

use crate::infrastructure::config::PluginSettings;
use crate::ui::foundation::control_style::ControlTypography;
use crate::ui::foundation::{colors, i18n};
use crate::ui::resolve_asset_path;
use crate::ui::shell::routes::{DropdownCallbacks, PluginPathUpdate};

pub(super) fn render_scan_paths_dialog(
    settings: &PluginSettings,
    callbacks: &DropdownCallbacks,
) -> AnyElement {
    let close_overlay = callbacks.on_set_scan_paths_open.clone();
    let close_button = callbacks.on_set_scan_paths_open.clone();
    let reset = callbacks.on_update_plugin_path.clone();

    div()
        .id("scan-paths-overlay")
        .absolute()
        .inset_0()
        .flex()
        .items_center()
        .justify_center()
        .occlude()
        .bg(rgba(0x100f0f99))
        .on_click(move |_, window, cx| close_overlay(false, window, cx))
        .child(
            div()
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
                .child(div().p_4().flex().flex_col().gap_4().child(path_section(
                    "plugins.vst3SearchPaths",
                    "plugins.noVst3Paths",
                    &settings.vst3_paths,
                    callbacks,
                )))
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
                ),
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
    title: &'static str,
    empty_text: &'static str,
    paths: &[String],
    callbacks: &DropdownCallbacks,
) -> AnyElement {
    let picker = callbacks.on_pick_plugin_path.clone();
    let update = callbacks.on_update_plugin_path.clone();
    let section_id = "vst3";

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
                    .on_click(move |_, window, cx| picker(window, cx)),
                ),
        )
        .child(
            div()
                .w_full()
                .max_h(px(140.0))
                .p_2()
                .flex()
                .flex_col()
                .gap(px(6.0))
                .overflow_y_scrollbar()
                .bg(colors::base_900().opacity(0.45))
                .border_1()
                .border_color(colors::base_800())
                .rounded_md()
                .when(paths.is_empty(), |element| {
                    element.child(
                        div()
                            .py_1()
                            .text_center()
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
                                        PluginPathUpdate::Remove(path_for_remove.clone()),
                                        window,
                                        cx,
                                    );
                                })
                                .child(
                                    svg()
                                        .external_path(resolve_asset_path(
                                            "assets/icons/trash-2.svg",
                                        ))
                                        .size(px(15.0))
                                        .text_color(colors::red()),
                                ),
                        )
                })),
        )
        .into_any_element()
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
