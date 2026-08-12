use gpui::prelude::*;
use gpui::*;
use std::sync::Arc;
use std::time::Instant;

use crate::infrastructure::engine::Engine;
use crate::ui::components::audio_dropdown::audio_dropdown;
use crate::ui::components::card_header::{PAGE_HEADER_GAP, card_header_layout, card_heading};
use crate::ui::components::smooth_scroll::SmoothVerticalScroll;
use crate::ui::foundation::colors;
use crate::ui::foundation::control_style::ControlTypography;
use crate::ui::foundation::i18n;
use crate::ui::foundation::motion::mix_color;
use crate::ui::shell::routes::{AudioMeterState, DropdownCallbacks, NavigateCallback};
use crate::ui::state::audio_controls::AudioControls;
use crate::ui::state::chain_operations::ChainOperationState;

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
    on_set_mono: crate::ui::shell::routes::MonoCallback,
    audio: AudioControls,
    is_mono: bool,
    mono_changed_at: Option<Instant>,
    meter: AudioMeterState,
    on_change_audio_routing: crate::ui::shell::routes::AudioRoutingCallback,
    chain_operations: Entity<ChainOperationState>,
}

impl HomePage {
    pub fn new(
        callbacks: &DropdownCallbacks,
        engine: Arc<Engine>,
        audio: AudioControls,
        is_mono: bool,
        mono_changed_at: Option<Instant>,
        meter: AudioMeterState,
        chain_operations: Entity<ChainOperationState>,
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
            chain_operations,
        }
    }

    pub fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let mut chain = match self.engine.cached_chain() {
            Ok(chain) => chain,
            Err(error) => {
                eprintln!("failed to read JUCE chain: {error}");
                Vec::new()
            }
        };
        let operation = self.chain_operations.read(cx);
        for item in &mut chain {
            item.initializing = item
                .unique_id
                .as_deref()
                .is_some_and(|unique_id| operation.is_adding(unique_id));
            item.removing = operation.is_clearing() || operation.is_removing(&item.id);
        }
        for pending in operation.pending_plugins() {
            if !chain
                .iter()
                .any(|item| item.unique_id.as_deref() == Some(pending.unique_id.as_str()))
            {
                chain.push(pending.chain_item());
            }
        }

        SmoothVerticalScroll::new(
            "home-page-scroll",
            div()
                .w_full()
                .p_4()
                .flex()
                .flex_col()
                .gap(PAGE_HEADER_GAP)
                .child(page_header())
                .child(self.audio_card(cx))
                .child(chain_card(
                    Arc::clone(&self.engine),
                    self.on_navigate.clone(),
                    chain,
                    self.chain_operations.clone(),
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
        let mut settings = div().p_4().flex().flex_col().gap_4().child(config_row(
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
    card_header_layout().child(card_heading(icon_path, icon_color, title, description))
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
                .font_weight(FontWeight::MEDIUM)
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
    cx: &App,
) -> Stateful<Div> {
    let hover_key = SharedString::from(format!("home-action-{id}"));
    let pressed_hover_key = hover_key.clone();
    let hover = crate::ui::foundation::hover_motion::progress(&hover_key, cx);
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
        .bg(if primary {
            background
        } else {
            mix_color(background, colors::base_850(), hover)
        })
        .border_1()
        .border_color(if primary {
            mix_color(
                colors::orange(),
                colors::accent_foreground().opacity(0.45),
                hover,
            )
        } else {
            colors::base_800()
        })
        .text_color(foreground)
        .on_hover(move |hovered, window, cx| {
            crate::ui::foundation::hover_motion::set_hovered(
                hover_key.clone(),
                *hovered,
                window,
                cx,
            );
        })
        .on_mouse_down(MouseButton::Left, move |_, window, cx| {
            crate::ui::foundation::hover_motion::clear_hover(&pressed_hover_key, window, cx);
        })
        .child(div().control_text().child(i18n::t(label)))
        .child(icon(icon_name, foreground))
}

fn icon_button(
    id: impl Into<ElementId>,
    icon_name: &'static str,
    tooltip: &'static str,
    destructive: bool,
    active: Option<bool>,
    disabled: bool,
    cx: &App,
) -> Stateful<Div> {
    let id = id.into();
    let hover_key = button_motion_key(&id);
    let hover = if disabled {
        0.0
    } else {
        crate::ui::foundation::hover_motion::progress(&hover_key, cx)
    };
    let state = active.map_or(0.0, |active| {
        crate::ui::foundation::hover_motion::state_progress(&hover_key, active, cx)
    });
    let resting_foreground = mix_color(colors::base_200(), colors::orange(), state);
    let hover_foreground = if destructive {
        colors::red()
    } else if active.is_some() {
        colors::orange()
    } else {
        colors::base_100()
    };
    let foreground = mix_color(resting_foreground, hover_foreground, hover);
    let hover_background = if destructive {
        colors::red().opacity(0.15)
    } else if active.is_some() {
        colors::orange().opacity(0.16)
    } else {
        colors::base_850()
    };
    let hover_border = if destructive {
        colors::red().opacity(0.4)
    } else if active.is_some() {
        colors::orange().opacity(0.7)
    } else {
        colors::base_700()
    };
    let button = div()
        .id(id.clone())
        .size(px(34.0))
        .flex()
        .items_center()
        .justify_center()
        .rounded_md()
        .bg(mix_color(colors::base_900(), hover_background, hover))
        .border_1()
        .border_color(mix_color(colors::base_800(), hover_border, hover))
        .text_color(foreground)
        .child(if active.is_some() {
            stateful_bypass_icon(state, foreground)
        } else {
            icon(icon_name, foreground)
        })
        .when(disabled, |button| button.cursor_default().opacity(0.5))
        .when(!disabled, |button| button.cursor_pointer());
    crate::ui::components::cursor_tooltip::attach_with_hover_motion(
        button,
        id,
        hover_key,
        i18n::t(tooltip),
    )
}

fn button_motion_key(id: &ElementId) -> SharedString {
    SharedString::from(format!("home-button-{id:?}"))
}

fn stateful_bypass_icon(progress: f32, color: Rgba) -> AnyElement {
    div()
        .relative()
        .size_4()
        .child(
            svg()
                .path(crate::ui::resolve_asset_path("assets/icons/circle-off.svg"))
                .size_4()
                .text_color(color)
                .opacity(1.0 - progress),
        )
        .child(
            svg()
                .absolute()
                .inset_0()
                .path(crate::ui::resolve_asset_path(
                    "assets/icons/circle-check.svg",
                ))
                .size_4()
                .text_color(color)
                .opacity(progress),
        )
        .into_any_element()
}

fn icon(name: &'static str, color: Rgba) -> AnyElement {
    svg()
        .path(crate::ui::resolve_asset_path(&format!(
            "assets/icons/{name}"
        )))
        .size_4()
        .text_color(color)
        .into_any_element()
}

fn separator() -> Div {
    div().h(px(1.0)).w_full().bg(colors::base_800())
}
