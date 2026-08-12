use super::{AudioConfig, EngineError};

#[cxx::bridge(namespace = "shallow_host")]
mod bridge {
    struct NativeAudioLevels {
        input: f32,
        output: f32,
    }

    unsafe extern "C++" {
        include!("cxx_bridge.h");

        fn init() -> Result<()>;
        fn shutdown() -> Result<()>;
        fn set_data_dir(path: &str) -> Result<()>;
        #[allow(
            clippy::too_many_arguments,
            reason = "the bridge mirrors JUCE's atomic audio-device setup operation"
        )]
        fn audio_start(
            driver: &str,
            input: &str,
            output: &str,
            sample_rate: i32,
            buffer_size: i32,
            input_mask: i32,
            output_mask: i32,
            mono: bool,
        ) -> Result<bool>;
        fn audio_stop() -> Result<bool>;
        fn audio_levels() -> Result<NativeAudioLevels>;
        fn audio_devices(driver: &str, device: &str) -> Result<String>;
        fn scan_plugins(plugin_paths_json: &str) -> Result<String>;
        fn start_plugin_scan(plugin_paths_json: &str) -> Result<String>;
        fn scan_next_plugin() -> Result<String>;
        fn add_to_chain(unique_id: &str) -> Result<String>;
        fn clear_chain() -> Result<()>;
        fn remove_from_chain(node_id: &str) -> Result<bool>;
        fn reorder_chain(node_id: &str, to_index: i32) -> Result<bool>;
        fn bypass_plugin(node_id: &str, bypassed: bool) -> Result<bool>;
        fn chain() -> Result<String>;
        fn parameters(node_id: &str) -> Result<String>;
        fn open_plugin_gui(node_id: &str, title_prefix: &str) -> Result<bool>;
        fn plugin_gui_open(node_id: &str) -> Result<bool>;
        fn save_state() -> Result<String>;
        fn load_state(state: &str) -> Result<bool>;
        fn state_revision() -> Result<u64>;
        fn set_mono_mode(mono: bool) -> Result<()>;
    }
}

pub fn init() -> Result<(), EngineError> {
    bridge::init().map_err(|error| bridge_error("initialize JUCE", error))
}

pub fn shutdown() -> Result<(), EngineError> {
    bridge::shutdown().map_err(|error| bridge_error("shut down JUCE", error))
}

pub fn set_data_dir(path: &str) -> Result<(), EngineError> {
    bridge::set_data_dir(path).map_err(|error| bridge_error("set data directory", error))
}

pub fn audio_start(config: &AudioConfig<'_>) -> Result<bool, EngineError> {
    bridge::audio_start(
        config.driver,
        config.input.unwrap_or_default(),
        config.output.unwrap_or_default(),
        config.sample_rate,
        config.buffer_size,
        config.input_mask,
        config.output_mask,
        config.is_mono,
    )
    .map_err(|error| bridge_error("start audio", error))
}

pub fn audio_stop() -> Result<bool, EngineError> {
    bridge::audio_stop().map_err(|error| bridge_error("stop audio", error))
}

pub fn audio_levels() -> Result<(f32, f32), EngineError> {
    bridge::audio_levels()
        .map(|levels| (levels.input, levels.output))
        .map_err(|error| bridge_error("read audio levels", error))
}

pub fn audio_devices(driver: &str, device: &str) -> Result<String, EngineError> {
    bridge::audio_devices(driver, device).map_err(|error| bridge_error("list audio devices", error))
}

pub fn scan_plugins(paths: &str) -> Result<String, EngineError> {
    bridge::scan_plugins(paths).map_err(|error| bridge_error("scan plugins", error))
}

pub fn start_plugin_scan(paths: &str) -> Result<String, EngineError> {
    bridge::start_plugin_scan(paths).map_err(|error| bridge_error("start plugin scan", error))
}

pub fn scan_next_plugin() -> Result<String, EngineError> {
    bridge::scan_next_plugin().map_err(|error| bridge_error("scan next plugin", error))
}

pub fn add_to_chain(unique_id: &str) -> Result<String, EngineError> {
    bridge::add_to_chain(unique_id).map_err(|error| bridge_error("add plugin to chain", error))
}

pub fn clear_chain() -> Result<(), EngineError> {
    bridge::clear_chain().map_err(|error| bridge_error("clear plugin chain", error))
}

pub fn remove_from_chain(node_id: &str) -> Result<bool, EngineError> {
    bridge::remove_from_chain(node_id)
        .map_err(|error| bridge_error("remove plugin from chain", error))
}

pub fn reorder_chain(node_id: &str, to_index: i32) -> Result<bool, EngineError> {
    bridge::reorder_chain(node_id, to_index)
        .map_err(|error| bridge_error("reorder plugin chain", error))
}

pub fn bypass_plugin(node_id: &str, bypassed: bool) -> Result<bool, EngineError> {
    bridge::bypass_plugin(node_id, bypassed)
        .map_err(|error| bridge_error("change plugin bypass", error))
}

pub fn chain() -> Result<String, EngineError> {
    bridge::chain().map_err(|error| bridge_error("read plugin chain", error))
}

pub fn save_state() -> Result<String, EngineError> {
    bridge::save_state().map_err(|error| bridge_error("save plugin chain state", error))
}

pub fn load_state(state: &str) -> Result<bool, EngineError> {
    bridge::load_state(state).map_err(|error| bridge_error("restore plugin chain state", error))
}

pub fn state_revision() -> Result<u64, EngineError> {
    bridge::state_revision().map_err(|error| bridge_error("read plugin state revision", error))
}

pub fn parameters(node_id: &str) -> Result<String, EngineError> {
    bridge::parameters(node_id).map_err(|error| bridge_error("read plugin parameters", error))
}

pub fn open_plugin_gui(node_id: &str, title: &str) -> Result<bool, EngineError> {
    bridge::open_plugin_gui(node_id, title)
        .map_err(|error| bridge_error("open plugin editor", error))
}

pub fn plugin_gui_open(node_id: &str) -> Result<bool, EngineError> {
    bridge::plugin_gui_open(node_id)
        .map_err(|error| bridge_error("read plugin editor status", error))
}

pub fn set_mono_mode(mono: bool) -> Result<(), EngineError> {
    bridge::set_mono_mode(mono).map_err(|error| bridge_error("set mono mode", error))
}

fn bridge_error(operation: &'static str, error: cxx::Exception) -> EngineError {
    EngineError::BridgeFailure {
        operation,
        message: error.what().to_owned(),
    }
}
