use gpui::prelude::*;
use gpui::*;
use gpui_component::StyledExt;
use std::sync::Arc;
use std::time::Instant;

use crate::engine::Engine;
use crate::ui::audio_controls::AudioControls;
use crate::ui::audio_dropdown::audio_dropdown;
use crate::ui::card_header::card_heading;
use crate::ui::colors;
use crate::ui::control_style::ControlTypography;
use crate::ui::i18n;
use crate::ui::routes::{AudioMeterState, DropdownCallbacks, NavigateCallback};
use crate::ui::smooth_scroll::SmoothVerticalScroll;

mod audio_panel;
mod chain_drag;
mod chain_panel;
mod mode_selector;

use audio_panel::{meter_and_dropdown, page_header, render_asio_channels};
use chain_panel::chain_card;
use mode_selector::mode_selector;

pub(crate) fn init(cx: &mut App) {
    chain_drag::init(cx);
}

pub(crate) fn update_chain_drag_mouse(position: Point<Pixels>, cx: &mut App) -> bool {
    chain_drag::update_mouse_position(position, cx)
}

pub struct HomePage {
    engine: Arc<Engine>,
    on_navigate: NavigateCallback,
    on_set_mono: crate::ui::routes::MonoCallback,
    audio: AudioControls,
    is_mono: bool,
    mono_changed_at: Option<Instant>,
    meter: AudioMeterState,
    on_change_audio_routing: crate::ui::routes::AudioRoutingCallback,
}

impl HomePage {
    pub fn new(
        callbacks: &DropdownCallbacks,
        engine: Arc<Engine>,
        audio: AudioControls,
        is_mono: bool,
        mono_changed_at: Option<Instant>,
        meter: AudioMeterState,
    ) -> Self {
        Self {
            engine,
            on_navigate: callbacks.on_navigate.clone(),
            on_set_mono: callbacks.on_set_mono.clone(),
            audio,
            is_mono,
            mono_changed_at,
            meter,
            on_change_audio_routing: callbacks.on_change_audio_routing.clone(),
        }
    }

    pub fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let chain = match self.engine.chain() {
            Ok(chain) => chain,
            Err(error) => {
                eprintln!("failed to read JUCE chain: {error}");
                Vec::new()
            }
        };

        SmoothVerticalScroll::new(
            "home-page-scroll",
            div()
                .w_full()
                .p_4()
                .flex()
                .flex_col()
                .gap_3()
                .child(page_header())
                .child(self.audio_card(cx))
                .child(chain_card(
                    Arc::clone(&self.engine),
                    self.on_navigate.clone(),
                    chain,
                    cx,
                )),
        )
    }

    fn audio_card(&self, cx: &App) -> AnyElement {
        let is_asio = self.audio.is_asio(cx);
        let has_output_device = self.audio.has_output_device(cx);
        let has_input_device = self.audio.has_input_device(cx);
        let output_control = meter_and_dropdown(
            if has_output_device {
                self.meter.output_level
            } else {
                0.0
            },
            has_output_device && self.meter.output_peak,
            audio_dropdown("audio-output", &self.audio.output, cx),
        );
        let input_control = meter_and_dropdown(
            if has_input_device {
                self.meter.input_level
            } else {
                0.0
            },
            has_input_device && self.meter.input_peak,
            audio_dropdown("audio-input", &self.audio.input, cx),
        );
        let mut settings = div().p_4().flex().flex_col().gap_3().child(config_row(
            "home.driver",
            "home.driverDescription",
            audio_dropdown("audio-driver", &self.audio.driver, cx),
        ));

        if is_asio {
            settings = settings.child(config_row(
                "home.device",
                "home.deviceDescription",
                audio_dropdown("audio-asio-device", &self.audio.output, cx),
            ));
            if has_output_device {
                settings = settings.child(render_asio_channels(
                    &self.audio,
                    self.meter,
                    self.on_change_audio_routing.clone(),
                    cx,
                ));
            }
        } else {
            settings = settings
                .child(config_row(
                    "home.outputDevice",
                    "home.outputDeviceDescription",
                    output_control,
                ))
                .child(config_row(
                    "home.inputDevice",
                    "home.inputDeviceDescription",
                    input_control,
                ));
        }

        settings = settings
            .child(config_row(
                "home.sampleRate",
                "home.sampleRateDescription",
                audio_dropdown("audio-sample-rate", &self.audio.sample_rate, cx),
            ))
            .child(config_row(
                "home.bufferSize",
                "home.bufferSizeDescription",
                audio_dropdown("audio-buffer-size", &self.audio.buffer_size, cx),
            ));

        card()
            .child(
                card_header(
                    "assets/icons/audio-waveform.svg",
                    colors::orange(),
                    "home.audio",
                    "home.audioDescription",
                )
                .child(mode_selector(
                    self.is_mono,
                    self.mono_changed_at,
                    self.on_set_mono.clone(),
                )),
            )
            .child(separator())
            .child(settings)
            .into_any_element()
    }
}

fn card() -> Div {
    div()
        .w_full()
        .flex()
        .flex_col()
        .bg(colors::base_950())
        .border_1()
        .border_color(colors::base_800())
        .rounded_lg()
}

fn card_header(
    icon_path: &'static str,
    icon_color: Rgba,
    title: &'static str,
    description: &'static str,
) -> Div {
    div()
        .w_full()
        .p_4()
        .flex()
        .items_center()
        .justify_between()
        .gap_4()
        .child(card_heading(icon_path, icon_color, title, description))
}

fn config_row(title: &'static str, description: &'static str, control: AnyElement) -> Div {
    div()
        .w_full()
        .flex()
        .items_center()
        .justify_between()
        .gap_4()
        .child(setting_copy(title, description))
        .child(control)
}

fn setting_copy(title: &'static str, description: &'static str) -> AnyElement {
    div()
        .flex_1()
        .flex()
        .flex_col()
        .gap(px(1.0))
        .child(
            div()
                .text_sm()
                .font_medium()
                .text_color(colors::base_200())
                .child(i18n::t(title)),
        )
        .child(
            div()
                .text_xs()
                .text_color(colors::base_500())
                .child(i18n::t(description)),
        )
        .into_any_element()
}

fn action_button(
    id: &'static str,
    label: &'static str,
    icon_name: &'static str,
    primary: bool,
) -> Stateful<Div> {
    let background = if primary {
        colors::orange()
    } else {
        colors::base_900()
    };
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
        .rounded_md()
        .cursor_pointer()
        .bg(background)
        .border_1()
        .border_color(if primary {
            colors::orange()
        } else {
            colors::base_800()
        })
        .text_color(foreground)
        .hover(move |style| {
            if primary {
                style.border_color(colors::accent_foreground().opacity(0.45))
            } else {
                style.bg(colors::base_850())
            }
        })
        .child(div().control_text().child(i18n::t(label)))
        .child(icon(icon_name, foreground))
}

fn icon_button(
    id: impl Into<ElementId>,
    icon_name: &'static str,
    tooltip: &'static str,
    destructive: bool,
) -> Stateful<Div> {
    let id = id.into();
    let hover_group = SharedString::from(format!("home-icon-button-{id:?}"));
    let foreground = colors::base_200();
    let button = div()
        .id(id.clone())
        .group(hover_group.clone())
        .size(px(34.0))
        .flex()
        .items_center()
        .justify_center()
        .rounded_md()
        .cursor_pointer()
        .bg(colors::base_900())
        .border_1()
        .border_color(colors::base_800())
        .text_color(foreground)
        .hover(move |style| {
            if destructive {
                style
                    .bg(colors::red().opacity(0.15))
                    .border_color(colors::red().opacity(0.4))
                    .text_color(colors::red())
            } else {
                style.bg(colors::base_850())
            }
        })
        .child(semantic_icon(icon_name, &hover_group, destructive));
    crate::ui::cursor_tooltip::attach(button, id, i18n::t(tooltip))
}

fn semantic_icon(
    icon_name: &'static str,
    hover_group: &SharedString,
    destructive: bool,
) -> AnyElement {
    let path = crate::ui::resolve_asset_path(&format!("assets/icons/{icon_name}"));
    div()
        .relative()
        .size_4()
        .child(
            div()
                .absolute()
                .inset_0()
                .when(destructive, |icon| {
                    icon.group_hover(hover_group.clone(), |style| style.invisible())
                })
                .child(
                    svg()
                        .external_path(path.clone())
                        .size_4()
                        .text_color(colors::base_200()),
                ),
        )
        .when(destructive, |icons| {
            icons.child(
                div()
                    .absolute()
                    .inset_0()
                    .invisible()
                    .group_hover(hover_group.clone(), |style| style.visible())
                    .child(svg().external_path(path).size_4().text_color(colors::red())),
            )
        })
        .into_any_element()
}

fn icon(name: &'static str, color: Rgba) -> AnyElement {
    svg()
        .external_path(crate::ui::resolve_asset_path(&format!(
            "assets/icons/{name}"
        )))
        .size_4()
        .text_color(color)
        .into_any_element()
}

fn separator() -> Div {
    div().h(px(1.0)).w_full().bg(colors::base_800())
}
