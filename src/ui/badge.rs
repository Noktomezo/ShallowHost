use gpui::prelude::*;
use gpui::*;
use gpui_component::StyledExt;

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
