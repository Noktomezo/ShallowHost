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
                    -std::f32::consts::TAU * delta,
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

pub fn progress_badge(text: impl Into<SharedString>, progress: f32) -> Div {
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
        .child(progress_ring(progress))
        .child(text.into())
}

pub fn progress_ring(progress: f32) -> AnyElement {
    let progress = progress.clamp(0.0, 1.0);
    canvas(
        |_, _, _| {},
        move |bounds, _, window, _| {
            let center = point(
                bounds.origin.x + bounds.size.width / 2.0,
                bounds.origin.y + bounds.size.height / 2.0,
            );
            let radius = px(6.0);
            let top = point(center.x, center.y - radius);
            let bottom = point(center.x, center.y + radius);
            let radii = point(radius, radius);

            let mut track = PathBuilder::stroke(px(2.0));
            track.move_to(top);
            track.arc_to(radii, px(0.0), false, true, bottom);
            track.arc_to(radii, px(0.0), false, true, top);
            if let Ok(track) = track.build() {
                window.paint_path(track, colors::orange().opacity(0.25));
            }

            if progress <= f32::EPSILON {
                return;
            }

            let mut arc = PathBuilder::stroke(px(2.0));
            arc.move_to(top);
            if progress >= 1.0 - f32::EPSILON {
                arc.arc_to(radii, px(0.0), false, true, bottom);
                arc.arc_to(radii, px(0.0), false, true, top);
            } else {
                let angle = -std::f32::consts::FRAC_PI_2 + std::f32::consts::TAU * progress;
                let end = point(
                    center.x + radius * angle.cos(),
                    center.y + radius * angle.sin(),
                );
                arc.arc_to(radii, px(0.0), progress > 0.5, true, end);
            }
            if let Ok(arc) = arc.build() {
                window.paint_path(arc, colors::orange());
            }
        },
    )
    .size_4()
    .into_any_element()
}
