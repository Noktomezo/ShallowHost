use gpui::prelude::*;
use gpui::*;
use gpui_component::StyledExt;
use std::time::Duration;

use crate::ui::colors;

#[derive(Clone, Copy)]
pub enum BadgeStyle {
    Neutral,
    Purple,
    Green,
    Red,
    Orange,
}

pub fn badge(text: impl Into<SharedString>, style: BadgeStyle) -> Div {
    let (background, border, foreground) = match style {
        BadgeStyle::Neutral => (colors::base_900(), colors::base_800(), colors::base_300()),
        BadgeStyle::Purple => (
            colors::purple().opacity(0.15),
            colors::purple().opacity(0.4),
            colors::purple(),
        ),
        BadgeStyle::Green => (
            colors::green().opacity(0.15),
            colors::green().opacity(0.4),
            colors::green(),
        ),
        BadgeStyle::Red => (
            colors::red().opacity(0.15),
            colors::red().opacity(0.4),
            colors::red(),
        ),
        BadgeStyle::Orange => (
            colors::orange().opacity(0.15),
            colors::orange().opacity(0.4),
            colors::orange(),
        ),
    };

    div()
        .h(px(20.0))
        .flex_none()
        .flex()
        .items_center()
        .justify_center()
        .px(px(2.0))
        .border_1()
        .border_color(border)
        .rounded_sm()
        .bg(background)
        .text_xs()
        .font_medium()
        .text_color(foreground)
        .child(text.into())
}

pub fn loading_badge(text: impl Into<SharedString>) -> Div {
    let icon = svg()
        .external_path(crate::ui::resolve_asset_path("assets/icons/refresh-cw.svg"))
        .size(px(12.0))
        .text_color(colors::orange())
        .with_animation(
            "chain-operation-spinner",
            Animation::new(Duration::from_millis(850)).repeat(),
            |icon, delta| {
                icon.with_transformation(Transformation::rotate(Radians(
                    std::f32::consts::TAU * delta,
                )))
            },
        );

    div()
        .h(px(20.0))
        .flex_none()
        .flex()
        .items_center()
        .justify_center()
        .gap(px(5.0))
        .px(px(4.0))
        .border_1()
        .border_color(colors::orange().opacity(0.4))
        .rounded_sm()
        .bg(colors::orange().opacity(0.12))
        .text_xs()
        .font_medium()
        .text_color(colors::orange())
        .child(icon)
        .child(text.into())
}
