use gpui::prelude::*;
use gpui::*;

use crate::ui::foundation::{colors, i18n};
use crate::ui::resolve_asset_path;

pub const PAGE_HEADER_GAP: Pixels = px(13.0);

pub fn card_header_layout() -> Div {
    div()
        .w_full()
        .px_4()
        .py(px(14.0))
        .flex()
        .items_center()
        .justify_between()
        .gap_4()
}

pub fn card_heading(
    icon_path: &'static str,
    icon_color: Rgba,
    title: &'static str,
    description: &'static str,
) -> AnyElement {
    card_heading_with_suffix(icon_path, icon_color, title, description, None)
}

pub fn card_heading_with_suffix(
    icon_path: &'static str,
    icon_color: Rgba,
    title: &'static str,
    description: &'static str,
    suffix: Option<AnyElement>,
) -> AnyElement {
    div()
        .min_w_0()
        .flex_1()
        .flex()
        .items_center()
        .gap_4()
        .child(
            div()
                .flex_none()
                .size(px(40.0))
                .flex()
                .items_center()
                .justify_center()
                .bg(colors::base_900())
                .border_1()
                .border_color(colors::base_800())
                .rounded_md()
                .child(
                    svg()
                        .external_path(resolve_asset_path(icon_path))
                        .size_5()
                        .text_color(icon_color),
                ),
        )
        .child(
            div()
                .min_w_0()
                .flex_1()
                .flex()
                .flex_col()
                .gap(px(2.0))
                .child(
                    div()
                        .min_w_0()
                        .flex()
                        .items_center()
                        .gap_2()
                        .child(
                            div()
                                .min_w_0()
                                .truncate()
                                .text_sm()
                                .font_weight(FontWeight::BOLD)
                                .text_color(colors::base_200())
                                .child(i18n::t(title)),
                        )
                        .when_some(suffix, |element, suffix| element.child(suffix)),
                )
                .child(
                    div()
                        .truncate()
                        .text_xs()
                        .text_color(colors::base_500())
                        .child(i18n::t(description)),
                ),
        )
        .into_any_element()
}
