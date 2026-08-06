use gpui::*;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use crate::config::{AudioSettings, DriverDeviceSelection};
use crate::engine::{AudioConfig, AudioDevices, DeviceInfo, Engine};

use super::audio_dropdown::{AudioDropdownState, DropdownChoice, reset_audio_dropdown};
use super::i18n;
use super::motion::{CONTROL_MOTION, DropdownMotion, changed_recently};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChannelDirection {
    Input,
    Output,
}

pub struct AudioRoutingState {
    pub input_channels: Vec<String>,
    pub output_channels: Vec<String>,
    pub active_inputs: Vec<usize>,
    pub active_outputs: Vec<usize>,
    input_changed_at: HashMap<usize, Instant>,
    output_changed_at: HashMap<usize, Instant>,
}

impl AudioRoutingState {
    pub fn channel_animating(&self, direction: ChannelDirection, row: usize) -> bool {
        let changed_at = match direction {
            ChannelDirection::Input => self.input_changed_at.get(&row),
            ChannelDirection::Output => self.output_changed_at.get(&row),
        };
        changed_recently(changed_at.copied(), CONTROL_MOTION)
    }
}

#[derive(Clone)]
pub struct AudioControls {
    pub driver: Entity<AudioDropdownState>,
    pub output: Entity<AudioDropdownState>,
    pub input: Entity<AudioDropdownState>,
    pub sample_rate: Entity<AudioDropdownState>,
    pub buffer_size: Entity<AudioDropdownState>,
    pub routing: Entity<AudioRoutingState>,
    device_selections: Entity<HashMap<String, DriverDeviceSelection>>,
}

impl AudioControls {
    pub fn new(devices: &AudioDevices, settings: &AudioSettings, cx: &mut App) -> Self {
        let driver_items = driver_choices();
        let output_items = device_choices(&devices.outputs);
        let input_items = device_choices(&devices.inputs);
        let sample_rate_items = sample_rate_choices();
        let buffer_size_items = buffer_size_choices(settings.sample_rate);
        let output_selected = settings.output_device.as_deref().map_or_else(
            || preferred_device_index(&devices.outputs),
            |value| choice_index(&output_items, value),
        );
        let input_selected = settings.input_device.as_deref().map_or_else(
            || preferred_device_index(&devices.inputs),
            |value| choice_index(&input_items, value),
        );
        let mut device_selections: HashMap<_, _> = settings
            .devices_by_driver
            .iter()
            .map(|(driver, selection)| (driver.clone(), selection.clone()))
            .collect();
        device_selections.insert(
            settings.driver.clone(),
            DriverDeviceSelection {
                input: selected_choice_device(&input_items, input_selected),
                output: selected_choice_device(&output_items, output_selected),
                active_inputs: settings.active_inputs.clone(),
                active_outputs: settings.active_outputs.clone(),
            },
        );

        Self {
            driver: dropdown_entity(cx, |motion| {
                AudioDropdownState::new(
                    driver_items.clone(),
                    choice_index(&driver_items, &settings.driver),
                    motion,
                )
            }),
            output: dropdown_entity(cx, |motion| {
                AudioDropdownState::new(output_items.clone(), output_selected, motion)
            }),
            input: dropdown_entity(cx, |motion| {
                AudioDropdownState::new(input_items.clone(), input_selected, motion)
            }),
            sample_rate: dropdown_entity(cx, |motion| {
                AudioDropdownState::new(
                    sample_rate_items.clone(),
                    choice_index(&sample_rate_items, &settings.sample_rate.to_string()),
                    motion,
                )
            }),
            buffer_size: dropdown_entity(cx, |motion| {
                AudioDropdownState::new(
                    buffer_size_items.clone(),
                    choice_index(&buffer_size_items, &settings.buffer_size.to_string()),
                    motion,
                )
            }),
            routing: cx.new(|_| AudioRoutingState {
                input_channels: devices.input_channels.clone(),
                output_channels: devices.output_channels.clone(),
                active_inputs: settings.active_inputs.clone(),
                active_outputs: settings.active_outputs.clone(),
                input_changed_at: HashMap::new(),
                output_changed_at: HashMap::new(),
            }),
            device_selections: cx.new(|_| device_selections),
        }
    }

    pub fn apply(&self, engine: &Arc<Engine>, cx: &App, is_mono: bool) {
        let settings = self.settings(cx, is_mono);
        let is_asio = settings.driver == "asio";
        let input_mask = device_channel_mask(
            is_asio,
            settings.input_device.as_deref(),
            &settings.active_inputs,
        );
        let output_mask = device_channel_mask(
            is_asio,
            settings.output_device.as_deref(),
            &settings.active_outputs,
        );
        let input = settings.input_device.as_deref().unwrap_or("__none");
        let output = settings.output_device.as_deref().unwrap_or("__none");
        let config = AudioConfig {
            driver: &settings.driver,
            input: Some(input),
            output: Some(output),
            sample_rate: settings.sample_rate,
            buffer_size: settings.buffer_size,
            input_mask,
            output_mask,
            is_mono,
        };

        if let Err(error) = engine.audio_start(&config) {
            eprintln!("failed to apply audio configuration: {error}");
        }
    }

    pub fn settings(&self, cx: &App, is_mono: bool) -> AudioSettings {
        let routing = self.routing.read(cx);
        AudioSettings {
            driver: selected_value(&self.driver, cx).unwrap_or_else(|| String::from("wasapi")),
            input_device: selected_device(&self.input, cx),
            output_device: selected_device(&self.output, cx),
            devices_by_driver: self
                .device_selections
                .read(cx)
                .iter()
                .map(|(driver, selection)| (driver.clone(), selection.clone()))
                .collect(),
            sample_rate: selected_value(&self.sample_rate, cx)
                .and_then(|value| value.parse().ok())
                .unwrap_or(48_000),
            buffer_size: selected_value(&self.buffer_size, cx)
                .and_then(|value| value.parse().ok())
                .unwrap_or(512),
            is_mono,
            active_inputs: routing.active_inputs.clone(),
            active_outputs: routing.active_outputs.clone(),
        }
    }

    pub fn is_asio(&self, cx: &App) -> bool {
        selected_value(&self.driver, cx).as_deref() == Some("asio")
    }

    pub fn has_output_device(&self, cx: &App) -> bool {
        selected_device(&self.output, cx).is_some()
    }

    pub fn has_input_device(&self, cx: &App) -> bool {
        selected_device(&self.input, cx).is_some()
    }

    pub fn refresh_devices(&self, engine: &Arc<Engine>, cx: &mut App) {
        let driver = selected_value(&self.driver, cx).unwrap_or_else(|| String::from("wasapi"));
        let devices = match engine.audio_devices(&driver, "") {
            Ok(devices) => devices,
            Err(error) => {
                eprintln!("failed to enumerate {driver} audio devices: {error}");
                return;
            }
        };
        let remembered = self.device_selections.read(cx).get(&driver).cloned();
        let output_choices = device_choices(&devices.outputs);
        let output_selected = remembered
            .as_ref()
            .and_then(|selection| selection.output.as_deref())
            .map_or(0, |device| choice_index(&output_choices, device));
        let input_choices = device_choices(&devices.inputs);
        let input_selected = remembered
            .as_ref()
            .and_then(|selection| selection.input.as_deref())
            .map_or(0, |device| choice_index(&input_choices, device));

        self.output.update(cx, |state, cx| {
            state.replace_choices(output_choices, output_selected);
            cx.notify();
        });
        self.input.update(cx, |state, cx| {
            state.replace_choices(input_choices, input_selected);
            cx.notify();
        });
        self.update_channels(&devices, cx);
        self.routing.update(cx, |routing, cx| {
            restore_active_channels(routing, remembered.as_ref());
            cx.notify();
        });
    }

    pub fn remember_device_selection(&self, cx: &mut App) {
        let driver = selected_value(&self.driver, cx).unwrap_or_else(|| String::from("wasapi"));
        let (active_inputs, active_outputs) = {
            let routing = self.routing.read(cx);
            (
                routing.active_inputs.clone(),
                routing.active_outputs.clone(),
            )
        };
        let selection = DriverDeviceSelection {
            input: selected_device(&self.input, cx),
            output: selected_device(&self.output, cx),
            active_inputs,
            active_outputs,
        };
        self.device_selections.update(cx, |selections, _| {
            selections.insert(driver, selection);
        });
    }

    pub fn refresh_asio_channels(&self, engine: &Arc<Engine>, cx: &mut App) {
        let Some(device) = selected_device(&self.output, cx) else {
            self.input.update(cx, |state, cx| {
                state.select_value("__none");
                cx.notify();
            });
            self.clear_channels(cx);
            return;
        };
        let devices = match engine.audio_devices("asio", &device) {
            Ok(devices) => devices,
            Err(error) => {
                eprintln!("failed to enumerate ASIO channels for {device}: {error}");
                self.clear_channels(cx);
                return;
            }
        };
        self.input.update(cx, |state, cx| {
            state.select_value(&device);
            cx.notify();
        });
        self.update_channels(&devices, cx);
    }

    pub fn refresh_buffer_latency(&self, cx: &mut App) {
        let sample_rate = selected_value(&self.sample_rate, cx)
            .and_then(|value| value.parse().ok())
            .unwrap_or(48_000);
        let selected_buffer =
            selected_value(&self.buffer_size, cx).unwrap_or_else(|| String::from("512"));
        let choices = buffer_size_choices(sample_rate);
        let selected = choice_index(&choices, &selected_buffer);

        self.buffer_size.update(cx, |state, cx| {
            state.replace_choices(choices, selected);
            cx.notify();
        });
    }

    pub fn toggle_channels(
        &self,
        direction: ChannelDirection,
        indices: &[usize],
        enabled: bool,
        cx: &mut App,
    ) {
        self.routing.update(cx, |routing, cx| {
            let changed = {
                let active = match direction {
                    ChannelDirection::Input => &mut routing.active_inputs,
                    ChannelDirection::Output => &mut routing.active_outputs,
                };
                let was_enabled = indices.iter().all(|index| active.contains(index));
                for index in indices {
                    if enabled && !active.contains(index) {
                        active.push(*index);
                    } else if !enabled {
                        active.retain(|active_index| active_index != index);
                    }
                }
                active.sort_unstable();
                was_enabled != enabled
            };
            if changed {
                let changed_at = match direction {
                    ChannelDirection::Input => &mut routing.input_changed_at,
                    ChannelDirection::Output => &mut routing.output_changed_at,
                };
                let now = Instant::now();
                for row in indices.iter().map(|index| index / 2) {
                    changed_at.insert(row, now);
                }
            }
            cx.notify();
        });
    }

    pub fn reset_dropdown_interactions(&self, cx: &mut App) {
        for dropdown in [
            &self.driver,
            &self.output,
            &self.input,
            &self.sample_rate,
            &self.buffer_size,
        ] {
            reset_audio_dropdown(dropdown, cx);
        }
    }

    fn update_channels(&self, devices: &AudioDevices, cx: &mut App) {
        self.routing.update(cx, |routing, cx| {
            routing.input_channels.clone_from(&devices.input_channels);
            routing.output_channels.clone_from(&devices.output_channels);
            retain_valid(&mut routing.active_inputs, routing.input_channels.len());
            retain_valid(&mut routing.active_outputs, routing.output_channels.len());
            routing.input_changed_at.clear();
            routing.output_changed_at.clear();
            cx.notify();
        });
    }

    fn clear_channels(&self, cx: &mut App) {
        self.routing.update(cx, |routing, cx| {
            clear_routing(routing);
            cx.notify();
        });
    }
}

fn clear_routing(routing: &mut AudioRoutingState) {
    routing.input_channels.clear();
    routing.output_channels.clear();
    routing.active_inputs.clear();
    routing.active_outputs.clear();
    routing.input_changed_at.clear();
    routing.output_changed_at.clear();
}

fn restore_active_channels(
    routing: &mut AudioRoutingState,
    selection: Option<&DriverDeviceSelection>,
) {
    if let Some(selection) = selection {
        routing.active_inputs.clone_from(&selection.active_inputs);
        routing.active_outputs.clone_from(&selection.active_outputs);
    } else {
        routing.active_inputs.clear();
        routing.active_outputs.clear();
    }
}

fn dropdown_entity(
    cx: &mut App,
    build: impl FnOnce(Entity<DropdownMotion>) -> AudioDropdownState,
) -> Entity<AudioDropdownState> {
    let motion = cx.new(|_| DropdownMotion::default());
    cx.new(|_| build(motion))
}

fn selected_value(control: &Entity<AudioDropdownState>, cx: &App) -> Option<String> {
    control.read(cx).selected_value().map(ToOwned::to_owned)
}

fn selected_device(control: &Entity<AudioDropdownState>, cx: &App) -> Option<String> {
    selected_value(control, cx).filter(|value| value != "__none")
}

fn choice_index(choices: &[DropdownChoice], value: &str) -> usize {
    choices
        .iter()
        .position(|choice| choice.value.as_ref() == value)
        .unwrap_or(0)
}

fn selected_choice_device(choices: &[DropdownChoice], selected: usize) -> Option<String> {
    choices
        .get(selected)
        .map(|choice| choice.value.as_ref())
        .filter(|value| *value != "__none")
        .map(ToOwned::to_owned)
}

fn preferred_device_index(devices: &[DeviceInfo]) -> usize {
    devices
        .iter()
        .position(|device| device.is_default)
        .map_or(0, |index| index + 1)
}

fn device_choices(devices: &[DeviceInfo]) -> Vec<DropdownChoice> {
    std::iter::once(DropdownChoice::new("__none", i18n::t("home.noneDevice")))
        .chain(
            devices
                .iter()
                .map(|device| DropdownChoice::new(device.name.clone(), device.name.clone())),
        )
        .collect()
}

fn driver_choices() -> Vec<DropdownChoice> {
    [
        ("wasapi", "WASAPI"),
        ("wasapi_exclusive", "WASAPI (Exclusive)"),
        ("asio", "ASIO"),
    ]
    .into_iter()
    .map(|(value, label)| DropdownChoice::new(value, label))
    .collect()
}

fn sample_rate_choices() -> Vec<DropdownChoice> {
    [44_100, 48_000, 88_200, 96_000, 192_000]
        .into_iter()
        .map(|rate| DropdownChoice::new(rate.to_string(), format!("{} kHz", rate / 1000)))
        .collect()
}

fn buffer_size_choices(sample_rate: i32) -> Vec<DropdownChoice> {
    [8, 16, 32, 64, 128, 256, 512, 1024, 2048]
        .into_iter()
        .map(|size| {
            let (label, latency) = buffer_size_parts(size, sample_rate);
            let choice = DropdownChoice::new(size.to_string(), label);
            match latency {
                Some(latency) => choice.with_muted_suffix(latency),
                None => choice,
            }
        })
        .collect()
}

fn buffer_size_parts(size: i32, sample_rate: i32) -> (String, Option<String>) {
    if sample_rate <= 0 {
        return (size.to_string(), None);
    }

    let latency_ms = f64::from(size) / f64::from(sample_rate) * 1_000.0;
    (size.to_string(), Some(format!("({latency_ms:.1} ms)")))
}

fn channel_mask(indices: &[usize]) -> i32 {
    indices.iter().fold(0_i32, |mask, index| {
        u32::try_from(*index)
            .ok()
            .and_then(|shift| 1_i32.checked_shl(shift))
            .map_or(mask, |bit| mask | bit)
    })
}

fn device_channel_mask(is_asio: bool, device: Option<&str>, indices: &[usize]) -> i32 {
    if device.is_none() || device == Some("__none") {
        return 0;
    }
    if is_asio { channel_mask(indices) } else { -1 }
}

fn retain_valid(active: &mut Vec<usize>, channel_count: usize) {
    active.retain(|index| *index < channel_count);
    if active.is_empty() && channel_count > 0 {
        active.extend(0..channel_count.min(2));
    }
}

#[cfg(test)]
mod tests;
