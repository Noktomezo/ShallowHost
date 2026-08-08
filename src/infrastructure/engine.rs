#![allow(
    dead_code,
    reason = "the safe wrapper intentionally covers the complete copied JUCE host API; not every native control is exposed by the current GPUI pages yet"
)]

mod ffi;

use serde::{Deserialize, Serialize};
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

#[derive(Debug)]
pub enum EngineError {
    BridgeFailure {
        operation: &'static str,
        message: String,
    },
    NativeFailure(&'static str),
    ChainIndexOutOfRange(usize),
    InvalidResponse(serde_json::Error),
    LockPoisoned,
    ReadChainState {
        path: PathBuf,
        source: std::io::Error,
    },
    WriteChainState {
        path: PathBuf,
        source: std::io::Error,
    },
}

impl fmt::Display for EngineError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BridgeFailure { operation, message } => {
                write!(formatter, "JUCE bridge failed to {operation}: {message}")
            }
            Self::NativeFailure(operation) => {
                write!(formatter, "JUCE operation failed: {operation}")
            }
            Self::ChainIndexOutOfRange(index) => {
                write!(
                    formatter,
                    "plugin chain index {index} does not fit the native API"
                )
            }
            Self::InvalidResponse(error) => {
                write!(formatter, "JUCE returned invalid JSON: {error}")
            }
            Self::LockPoisoned => formatter.write_str("JUCE call lock was poisoned"),
            Self::ReadChainState { path, source } => {
                write!(
                    formatter,
                    "cannot read plugin chain state {}: {source}",
                    path.display()
                )
            }
            Self::WriteChainState { path, source } => {
                write!(
                    formatter,
                    "cannot write plugin chain state {}: {source}",
                    path.display()
                )
            }
        }
    }
}

impl std::error::Error for EngineError {}

impl From<serde_json::Error> for EngineError {
    fn from(error: serde_json::Error) -> Self {
        Self::InvalidResponse(error)
    }
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Eq)]
pub struct AudioDevices {
    pub inputs: Vec<DeviceInfo>,
    pub outputs: Vec<DeviceInfo>,
    #[serde(default)]
    pub input_channels: Vec<String>,
    #[serde(default)]
    pub output_channels: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct DeviceInfo {
    pub name: String,
    #[serde(default, rename = "default")]
    pub is_default: bool,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct ChainItem {
    pub id: String,
    pub name: String,
    pub vendor: String,
    pub format: String,
    pub bypassed: bool,
    pub unique_id: Option<String>,
    #[serde(default)]
    pub initializing: bool,
    #[serde(default)]
    pub removing: bool,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct ScannedPlugin {
    pub name: String,
    pub vendor: String,
    pub version: String,
    pub category: String,
    pub path: String,
    pub unique_id: String,
    pub format: String,
    pub has_editor: bool,
    pub accepts_midi: bool,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct ParameterInfo {
    pub index: usize,
    pub name: String,
    pub value: f32,
    pub text_value: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct AudioConfig<'a> {
    pub driver: &'a str,
    pub input: Option<&'a str>,
    pub output: Option<&'a str>,
    pub sample_rate: i32,
    pub buffer_size: i32,
    pub input_mask: i32,
    pub output_mask: i32,
    pub is_mono: bool,
}

pub struct Engine {
    call_lock: Mutex<()>,
    plugins: Mutex<Vec<ScannedPlugin>>,
    chain_cache: Mutex<Vec<ChainItem>>,
    chain_state_path: PathBuf,
}

impl Engine {
    pub fn new(data_dir: &Path) -> Result<Self, EngineError> {
        let engine = Self {
            call_lock: Mutex::new(()),
            plugins: Mutex::new(Vec::new()),
            chain_cache: Mutex::new(Vec::new()),
            chain_state_path: data_dir.join("chain.json"),
        };
        let _guard = engine.lock()?;
        ffi::init()?;
        ffi::set_data_dir(&data_dir.to_string_lossy())?;
        let cached_plugins = parse_json(&ffi::scan_plugins("[]")?)?;
        drop(_guard);
        *engine
            .plugins
            .lock()
            .map_err(|_| EngineError::LockPoisoned)? = cached_plugins;
        Ok(engine)
    }

    pub fn audio_start(&self, config: &AudioConfig<'_>) -> Result<(), EngineError> {
        let _guard = self.lock()?;
        ffi::audio_start(config)?
            .then_some(())
            .ok_or(EngineError::NativeFailure("start audio"))
    }

    pub fn audio_stop(&self) -> Result<(), EngineError> {
        let _guard = self.lock()?;
        ffi::audio_stop()?
            .then_some(())
            .ok_or(EngineError::NativeFailure("stop audio"))
    }

    pub fn audio_levels(&self) -> Result<(f32, f32), EngineError> {
        let _guard = self.lock()?;
        ffi::audio_levels()
    }

    pub fn audio_devices(&self, driver: &str, device: &str) -> Result<AudioDevices, EngineError> {
        let _guard = self.lock()?;
        parse_json(&ffi::audio_devices(driver, device)?)
    }

    pub fn scan_plugins(&self, vst3_paths: &[String]) -> Result<Vec<ScannedPlugin>, EngineError> {
        let vst3 = serde_json::to_string(vst3_paths)?;
        let _guard = self.lock()?;
        let plugins: Vec<ScannedPlugin> = parse_json(&ffi::scan_plugins(&vst3)?)?;
        drop(_guard);
        let mut cache = self.plugins.lock().map_err(|_| EngineError::LockPoisoned)?;
        *cache = plugins.clone();
        Ok(plugins)
    }

    pub fn plugins(&self) -> Result<Vec<ScannedPlugin>, EngineError> {
        self.plugins
            .lock()
            .map(|plugins| plugins.clone())
            .map_err(|_| EngineError::LockPoisoned)
    }

    pub fn remove_cached_plugin(&self, unique_id: &str) -> Result<(), EngineError> {
        let mut plugins = self.plugins.lock().map_err(|_| EngineError::LockPoisoned)?;
        plugins.retain(|plugin| plugin.unique_id != unique_id);
        Ok(())
    }

    pub fn chain(&self) -> Result<Vec<ChainItem>, EngineError> {
        let _guard = self.lock()?;
        self.cache_chain_from_native()
    }

    pub fn cached_chain(&self) -> Result<Vec<ChainItem>, EngineError> {
        self.chain_cache
            .lock()
            .map(|chain| chain.clone())
            .map_err(|_| EngineError::LockPoisoned)
    }

    pub fn add_to_chain(&self, unique_id: &str) -> Result<(), EngineError> {
        let _guard = self.lock()?;
        let response = ffi::add_to_chain(unique_id)?;
        if response.is_empty() || response == "false" {
            return Err(EngineError::NativeFailure("add plugin to chain"));
        }
        self.cache_chain_from_native()?;
        drop(_guard);
        self.save_chain_state()
    }

    pub fn clear_chain(&self) -> Result<(), EngineError> {
        let _guard = self.lock()?;
        ffi::clear_chain()?;
        self.cache_chain_from_native()?;
        drop(_guard);
        self.save_chain_state()
    }

    pub fn remove_from_chain(&self, node_id: &str) -> Result<(), EngineError> {
        let _guard = self.lock()?;
        let removed = ffi::remove_from_chain(node_id)?
            .then_some(())
            .ok_or(EngineError::NativeFailure("remove plugin from chain"));
        if removed.is_ok() {
            self.cache_chain_from_native()?;
        }
        drop(_guard);
        removed?;
        self.save_chain_state()
    }

    pub fn reorder_chain(&self, node_id: &str, to_index: usize) -> Result<(), EngineError> {
        let native_index =
            i32::try_from(to_index).map_err(|_| EngineError::ChainIndexOutOfRange(to_index))?;
        let _guard = self.lock()?;
        let reordered = ffi::reorder_chain(node_id, native_index)?
            .then_some(())
            .ok_or(EngineError::NativeFailure("reorder plugin chain"));
        if reordered.is_ok() {
            self.cache_chain_from_native()?;
        }
        drop(_guard);
        reordered?;
        self.save_chain_state()
    }

    pub fn bypass_plugin(&self, node_id: &str, bypassed: bool) -> Result<(), EngineError> {
        let _guard = self.lock()?;
        let changed = ffi::bypass_plugin(node_id, bypassed)?
            .then_some(())
            .ok_or(EngineError::NativeFailure("change plugin bypass"));
        if changed.is_ok() {
            self.cache_chain_from_native()?;
        }
        drop(_guard);
        changed?;
        self.save_chain_state()
    }

    pub fn open_plugin_gui(&self, node_id: &str, title: &str) -> Result<(), EngineError> {
        let _guard = self.lock()?;
        ffi::open_plugin_gui(node_id, title)?
            .then_some(())
            .ok_or(EngineError::NativeFailure("open plugin editor"))
    }

    pub fn set_mono_mode(&self, mono: bool) -> Result<(), EngineError> {
        let _guard = self.lock()?;
        ffi::set_mono_mode(mono)
    }

    pub fn parameters(&self, node_id: &str) -> Result<Vec<ParameterInfo>, EngineError> {
        let _guard = self.lock()?;
        parse_json(&ffi::parameters(node_id)?)
    }

    pub fn restore_chain_state(&self) -> Result<(), EngineError> {
        let state = match fs::read_to_string(&self.chain_state_path) {
            Ok(state) => state,
            Err(source) if source.kind() == std::io::ErrorKind::NotFound => return Ok(()),
            Err(source) => {
                return Err(EngineError::ReadChainState {
                    path: self.chain_state_path.clone(),
                    source,
                });
            }
        };
        let _guard = self.lock()?;
        ffi::load_state(&state)?
            .then_some(())
            .ok_or(EngineError::NativeFailure("restore plugin chain state"))?;
        self.cache_chain_from_native()?;
        Ok(())
    }

    pub fn saved_chain_placeholders(&self) -> Result<Vec<ChainItem>, EngineError> {
        let state = match fs::read_to_string(&self.chain_state_path) {
            Ok(state) => state,
            Err(source) if source.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(source) => {
                return Err(EngineError::ReadChainState {
                    path: self.chain_state_path.clone(),
                    source,
                });
            }
        };
        parse_saved_chain_placeholders(&state)
    }

    pub fn save_chain_state(&self) -> Result<(), EngineError> {
        let _guard = self.lock()?;
        let state = ffi::save_state()?;
        drop(_guard);
        fs::write(&self.chain_state_path, state).map_err(|source| EngineError::WriteChainState {
            path: self.chain_state_path.clone(),
            source,
        })
    }

    fn lock(&self) -> Result<std::sync::MutexGuard<'_, ()>, EngineError> {
        self.call_lock.lock().map_err(|_| EngineError::LockPoisoned)
    }

    fn cache_chain_from_native(&self) -> Result<Vec<ChainItem>, EngineError> {
        let chain = parse_json(&ffi::chain()?)?;
        let mut cache = self
            .chain_cache
            .lock()
            .map_err(|_| EngineError::LockPoisoned)?;
        cache.clone_from(&chain);
        Ok(chain)
    }
}

impl Drop for Engine {
    fn drop(&mut self) {
        if self.call_lock.lock().is_ok() {
            if let Ok(state) = ffi::save_state()
                && let Err(source) = fs::write(&self.chain_state_path, state)
            {
                eprintln!(
                    "failed to save plugin chain state {} during shutdown: {source}",
                    self.chain_state_path.display()
                );
            }
            if let Err(error) = ffi::shutdown() {
                eprintln!("failed to shut down JUCE: {error}");
            }
        }
    }
}

fn parse_json<T: for<'de> Deserialize<'de>>(json: &str) -> Result<T, EngineError> {
    serde_json::from_str(json).map_err(EngineError::from)
}

fn parse_saved_chain_placeholders(json: &str) -> Result<Vec<ChainItem>, EngineError> {
    #[derive(Deserialize)]
    struct SavedChainItem {
        unique_id: Option<String>,
        name: Option<String>,
        vendor: Option<String>,
        format: Option<String>,
        bypassed: Option<bool>,
    }

    let items: Vec<SavedChainItem> = parse_json(json)?;
    Ok(items
        .into_iter()
        .enumerate()
        .map(|(index, item)| ChainItem {
            id: format!("saved-{index}"),
            name: item.name.unwrap_or_else(|| String::from("Plugin")),
            vendor: item.vendor.unwrap_or_default(),
            format: item.format.unwrap_or_else(|| String::from("VST3")),
            bypassed: item.bypassed.unwrap_or(false),
            unique_id: item.unique_id,
            initializing: true,
            removing: false,
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::{AudioDevices, ChainItem, parse_json, parse_saved_chain_placeholders};

    #[test]
    fn parses_audio_devices_from_native_contract() {
        let devices: AudioDevices = parse_json(
            r#"{"inputs":[{"name":"Mic","default":true}],"outputs":[],"input_channels":["L"],"output_channels":[]}"#,
        )
        .expect("fixture is valid JUCE device JSON");

        assert_eq!(devices.inputs[0].name, "Mic");
        assert!(devices.inputs[0].is_default);
        assert_eq!(devices.input_channels, ["L"]);
    }

    #[test]
    fn parses_chain_item_from_native_contract() {
        let chain: Vec<ChainItem> = parse_json(
            r#"[{"id":"42","name":"Effect","vendor":"Vendor","format":"VST3","bypassed":false,"unique_id":"uid"}]"#,
        )
        .expect("fixture is valid JUCE chain JSON");

        assert_eq!(chain[0].id, "42");
        assert_eq!(chain[0].unique_id.as_deref(), Some("uid"));
        assert!(!chain[0].bypassed);
    }

    #[test]
    fn reads_saved_chain_as_initializing_placeholders() {
        let chain = parse_saved_chain_placeholders(
            r#"[{"unique_id":"uid","name":"Clear","vendor":"Supertone","format":"VST3","bypassed":true,"state":"ignored"}]"#,
        )
        .expect("fixture is valid saved chain JSON");

        assert_eq!(chain[0].id, "saved-0");
        assert_eq!(chain[0].name, "Clear");
        assert!(chain[0].bypassed);
        assert!(chain[0].initializing);
    }
}
