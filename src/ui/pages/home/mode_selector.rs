use gpui::prelude::*;
use gpui::*;
use std::time::Instant;

use crate::ui::foundation::colors;
use crate::ui::foundation::control_style::ControlTypography;
use crate::ui::foundation::i18n;
use crate::ui::foundation::motion::{MODE_MOTION, changed_recently, mix_color};
use crate::ui::shell::routes::MonoCallback;

pub(super) fn mode_selector(
    is_mono: bool,
    changed_at: Option<Instant>,
    callback: MonoCallback,
) -> AnyElement {
    let stereo_callback = callback.clone();
    let animate = changed_recently(changed_at, MODE_MOTION);
    div()
        .relative()
        .w(px(160.0))
        .h(px(34.0))
        .p(px(2.0))
        .flex()
        .items_center()
        .bg(colors::base_900())
        .border_1()
        .border_color(colors::base_800())
        .rounded_md()
        .overflow_hidden()
        .child(mode_thumb(is_mono, animate))
        .child(mode_button(
            "audio-stereo",
            "home.stereo",
            !is_mono,
            animate,
            move |window, cx| stereo_callback(false, window, cx),
        ))
        .child(mode_button(
            "audio-mono",
            "home.mono",
            is_mono,
            animate,
            move |window, cx| callback(true, window, cx),
        ))
        .into_any_element()
}

fn mode_button(
    id: &'static str,
    label: &'static str,
    active: bool,
    animate: bool,
    on_click: impl Fn(&mut Window, &mut App) + 'static,
) -> AnyElement {
    let active_foreground = colors::accent_foreground();
    let animation_id = ElementId::NamedInteger(
        SharedString::from(format!("{id}-label-motion")),
        u64::from(active),
    );
    let button = div()
        .id(id)
        .relative()
        .flex_1()
        .h(px(24.0))
        .flex()
        .items_center()
        .justify_center()
        .rounded_sm()
        .control_text()
        .cursor_pointer()
        .on_click(move |_, window, cx| {
            cx.stop_propagation();
            on_click(window, cx);
        })
        .child(i18n::t(label));
    if animate {
        button
            .with_animation(
                animation_id,
                Animation::new(MODE_MOTION).with_easing(ease_in_out),
                move |element, delta| {
                    let progress = if active { delta } else { 1.0 - delta };
                    element.text_color(mix_color(colors::base_500(), active_foreground, progress))
                },
            )
            .into_any_element()
    } else {
        button
            .text_color(if active {
                active_foreground
            } else {
                colors::base_500()
            })
            .into_any_element()
    }
}

fn mode_thumb(is_mono: bool, animate: bool) -> AnyElement {
    let animation_id = ElementId::NamedInteger(
        SharedString::from("audio-mode-thumb-motion"),
        u64::from(is_mono),
    );
    let thumb = div()
        .absolute()
        .top(px(2.0))
        .left(px(2.0))
        .w(px(77.0))
        .h(px(28.0))
        .rounded_sm();
    if animate {
        thumb
            .with_animation(
                animation_id,
                Animation::new(MODE_MOTION).with_easing(ease_in_out),
                move |element, delta| {
                    let progress = if is_mono { delta } else { 1.0 - delta };
                    element.ml(px(77.0 * progress)).bg(mix_color(
                        colors::orange(),
                        colors::purple(),
                        progress,
                    ))
                },
            )
            .into_any_element()
    } else {
        let progress = if is_mono { 1.0 } else { 0.0 };
        thumb
            .ml(px(77.0 * progress))
            .bg(mix_color(colors::orange(), colors::purple(), progress))
            .into_any_element()
    }
}
