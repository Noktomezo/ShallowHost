use gpui::prelude::*;
use gpui::*;
use gpui_component::StyledExt;
use gpui_component::scroll::ScrollableElement;

use crate::ui::components::volume_meter::volume_meter;
use crate::ui::foundation::colors;
use crate::ui::foundation::control_style::ControlTypography;
use crate::ui::foundation::i18n;
use crate::ui::foundation::motion::{CONTROL_MOTION, mix_color};
use crate::ui::shell::routes::{AudioMeterState, AudioRoutingCallback};
use crate::ui::state::audio_controls::{AudioControls, AudioRoutingState, ChannelDirection};

pub(super) fn page_header() -> AnyElement {
    div()
        .flex()
        .flex_col()
        .gap(px(2.0))
        .child(
            div()
                .text_xl()
                .font_semibold()
                .text_color(colors::base_200())
                .child(i18n::t("home.title")),
        )
        .child(
            div()
                .text_sm()
                .text_color(colors::base_500())
                .child(i18n::t("home.description")),
        )
        .into_any_element()
}

pub(super) fn meter_and_dropdown(level: f32, peak: bool, dropdown: AnyElement) -> AnyElement {
    div()
        .flex()
        .items_center()
        .justify_end()
        .gap_3()
        .flex_none()
        .child(volume_meter(level, peak))
        .child(dropdown)
        .into_any_element()
}

pub(super) fn render_asio_channels(
    audio: &AudioControls,
    meter: AudioMeterState,
    callback: AudioRoutingCallback,
    cx: &App,
) -> AnyElement {
    let routing = audio.routing.read(cx);
    div()
        .w_full()
        .flex()
        .gap_4()
        .child(channel_panel(
            "home.activeOutputChannels",
            routing,
            ChannelDirection::Output,
            meter.output_level,
            meter.output_peak,
            callback.clone(),
            cx,
        ))
        .child(channel_panel(
            "home.activeInputChannels",
            routing,
            ChannelDirection::Input,
            meter.input_level,
            meter.input_peak,
            callback,
            cx,
        ))
        .into_any_element()
}

pub(super) fn channel_panel(
    title: &'static str,
    routing: &AudioRoutingState,
    direction: ChannelDirection,
    level: f32,
    peak: bool,
    callback: AudioRoutingCallback,
    cx: &App,
) -> AnyElement {
    let (channels, active) = match direction {
        ChannelDirection::Input => (&routing.input_channels, &routing.active_inputs),
        ChannelDirection::Output => (&routing.output_channels, &routing.active_outputs),
    };
    let pairs = group_channels(channels);
    div()
        .min_w_0()
        .flex_1()
        .flex()
        .flex_col()
        .gap_2()
        .child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .gap_2()
                .child(
                    div()
                        .text_sm()
                        .font_medium()
                        .text_color(colors::base_200())
                        .child(i18n::t(title)),
                )
                .child(volume_meter(level, peak)),
        )
        .child(
            div()
                .w_full()
                .max_h(px(160.0))
                .p_3()
                .flex()
                .flex_col()
                .gap(px(6.0))
                .overflow_y_scrollbar()
                .bg(rgba(0xffffff08))
                .border_1()
                .border_color(colors::base_800())
                .rounded_md()
                .when(pairs.is_empty(), |element| {
                    element.child(
                        div()
                            .text_xs()
                            .text_color(colors::base_500())
                            .child(i18n::t("home.noChannelsAvailable")),
                    )
                })
                .children(pairs.into_iter().enumerate().map(|(row, pair)| {
                    let checked = pair.indices.iter().all(|index| active.contains(index));
                    let animate_checkbox = routing.channel_animating(direction, row);
                    let callback = callback.clone();
                    let indices = pair.indices;
                    let direction_name = match direction {
                        ChannelDirection::Input => "input",
                        ChannelDirection::Output => "output",
                    };
                    let checkbox_id =
                        SharedString::from(format!("channel-{direction_name}-{row}-checkbox"));
                    let hover_key =
                        SharedString::from(format!("channel-{direction_name}-{row}-hover"));
                    let hover = crate::ui::foundation::hover_motion::progress(&hover_key, cx);
                    div()
                        .id(SharedString::from(format!(
                            "channel-{direction_name}-{row}"
                        )))
                        .h(px(26.0))
                        .px_1()
                        .flex()
                        .items_center()
                        .gap_2()
                        .rounded_sm()
                        .cursor_pointer()
                        .bg(colors::base_850().opacity(hover))
                        .on_hover(move |hovered, window, cx| {
                            crate::ui::foundation::hover_motion::set_hovered(
                                hover_key.clone(),
                                *hovered,
                                window,
                                cx,
                            );
                        })
                        .on_click(move |_, window, cx| {
                            callback(direction, indices.clone(), !checked, window, cx);
                        })
                        .child(channel_checkbox(
                            checkbox_id,
                            checked,
                            animate_checkbox,
                            hover,
                        ))
                        .child(
                            div()
                                .min_w_0()
                                .truncate()
                                .control_text()
                                .text_color(colors::base_300())
                                .child(pair.label),
                        )
                })),
        )
        .into_any_element()
}

fn channel_checkbox(id: SharedString, checked: bool, animate: bool, hover: f32) -> AnyElement {
    let state = u64::from(checked);
    let box_animation_id = ElementId::NamedInteger(id.clone(), state);
    let check_animation_id =
        ElementId::NamedInteger(SharedString::from(format!("{id}-mark")), state);
    let icon = svg()
        .external_path(crate::ui::resolve_asset_path("assets/icons/check.svg"))
        .size(px(12.0))
        .text_color(colors::accent_foreground());
    let icon = if animate {
        icon.with_animation(
            check_animation_id,
            Animation::new(CONTROL_MOTION).with_easing(ease_in_out),
            move |icon, delta| {
                let progress = if checked { delta } else { 1.0 - delta };
                icon.opacity(progress)
                    .with_transformation(Transformation::scale(size(
                        0.65 + 0.35 * progress,
                        0.65 + 0.35 * progress,
                    )))
            },
        )
        .into_any_element()
    } else {
        let progress = if checked { 1.0 } else { 0.0 };
        icon.opacity(progress)
            .with_transformation(Transformation::scale(size(
                0.65 + 0.35 * progress,
                0.65 + 0.35 * progress,
            )))
            .into_any_element()
    };
    let checkbox = div()
        .size(px(16.0))
        .flex()
        .items_center()
        .justify_center()
        .flex_none()
        .rounded_sm()
        .border_1()
        .child(icon);
    if animate {
        checkbox
            .with_animation(
                box_animation_id,
                Animation::new(CONTROL_MOTION).with_easing(ease_in_out),
                move |element, delta| {
                    let progress = if checked { delta } else { 1.0 - delta };
                    element
                        .border_color(mix_color(
                            mix_color(colors::base_700(), colors::base_500(), hover),
                            colors::orange(),
                            progress,
                        ))
                        .bg(mix_color(
                            mix_color(colors::base_900(), colors::base_850(), hover),
                            colors::orange(),
                            progress,
                        ))
                },
            )
            .into_any_element()
    } else {
        checkbox
            .border_color(if checked {
                colors::orange()
            } else {
                mix_color(colors::base_700(), colors::base_500(), hover)
            })
            .bg(if checked {
                colors::orange()
            } else {
                mix_color(colors::base_900(), colors::base_850(), hover)
            })
            .into_any_element()
    }
}

struct ChannelPair {
    label: String,
    indices: Vec<usize>,
}

fn group_channels(channels: &[String]) -> Vec<ChannelPair> {
    channels
        .chunks(2)
        .enumerate()
        .map(|(pair_index, pair)| ChannelPair {
            label: match pair {
                [left, right] => format_channel_pair(left, right),
                [single] => single.clone(),
                _ => String::new(),
            },
            indices: (pair_index * 2..pair_index * 2 + pair.len()).collect(),
        })
        .collect()
}

fn format_channel_pair(left: &str, right: &str) -> String {
    if left == right {
        return left.to_owned();
    }
    let common = left
        .chars()
        .zip(right.chars())
        .take_while(|(left, right)| left == right)
        .count();
    let prefix: String = left.chars().take(common).collect();
    let left_suffix: String = left.chars().skip(common).collect();
    let right_suffix: String = right.chars().skip(common).collect();
    if prefix.is_empty() || left_suffix.is_empty() || right_suffix.is_empty() {
        format!("{left} / {right}")
    } else {
        format!("{prefix}{left_suffix} + {right_suffix}")
    }
}
