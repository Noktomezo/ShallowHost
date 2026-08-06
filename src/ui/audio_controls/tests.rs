use std::collections::HashMap;

use super::{
    AudioRoutingState, DropdownChoice, buffer_size_parts, choice_index, clear_routing,
    device_channel_mask, selected_choice_device,
};

#[test]
fn formats_approximate_buffer_latency() {
    assert_eq!(
        buffer_size_parts(512, 48_000),
        (String::from("512"), Some(String::from("(10.7 ms)")))
    );
    assert_eq!(
        buffer_size_parts(256, 44_100),
        (String::from("256"), Some(String::from("(5.8 ms)")))
    );
}

#[test]
fn clears_stale_routing_when_asio_device_is_absent() {
    let mut routing = AudioRoutingState {
        input_channels: vec![String::from("Input 1")],
        output_channels: vec![String::from("Output 1")],
        active_inputs: vec![0],
        active_outputs: vec![0],
        input_changed_at: HashMap::new(),
        output_changed_at: HashMap::new(),
    };

    clear_routing(&mut routing);

    assert!(routing.input_channels.is_empty());
    assert!(routing.output_channels.is_empty());
    assert!(routing.active_inputs.is_empty());
    assert!(routing.active_outputs.is_empty());
}

#[test]
fn uses_default_channels_for_selected_wasapi_devices() {
    assert_eq!(device_channel_mask(false, Some("Speakers"), &[]), -1);
    assert_eq!(device_channel_mask(false, Some("Microphone"), &[0, 1]), -1);
    assert_eq!(device_channel_mask(false, None, &[]), 0);
    assert_eq!(device_channel_mask(false, Some("__none"), &[]), 0);
}

#[test]
fn preserves_explicit_asio_channel_masks() {
    assert_eq!(device_channel_mask(true, Some("ASIO Device"), &[0, 2]), 5);
    assert_eq!(device_channel_mask(true, Some("ASIO Device"), &[]), 0);
    assert_eq!(device_channel_mask(true, Some("__none"), &[0, 1]), 0);
}

#[test]
fn restores_only_an_exact_remembered_device() {
    let choices = vec![
        DropdownChoice::new("__none", "None"),
        DropdownChoice::new("Device A", "Device A"),
    ];

    assert_eq!(choice_index(&choices, "Device A"), 1);
    assert_eq!(choice_index(&choices, "Missing Device"), 0);
    assert_eq!(selected_choice_device(&choices, 0), None);
    assert_eq!(
        selected_choice_device(&choices, 1),
        Some(String::from("Device A"))
    );
}
