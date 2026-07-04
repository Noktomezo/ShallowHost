use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;

use super::types::{AudioConfig, AudioDevices};
use crate::commands::scanner::ScannedPlugin;
use crate::engine::chain::ChainItem;
use crate::engine::chain::ParamInfo;
use crate::ffi;

pub struct AudioEngine {
    pub running: Mutex<bool>,
    pub data_dir: Arc<Mutex<Option<PathBuf>>>,
    pub config: Mutex<AudioConfig>,
}

impl AudioEngine {
    pub fn new() -> Self {
        Self {
            running: Mutex::new(false),
            data_dir: Arc::new(Mutex::new(None)),
            config: Mutex::new(AudioConfig::default()),
        }
    }

    pub fn set_data_dir(&self, dir: PathBuf) {
        *self.data_dir.lock().unwrap() = Some(dir);
    }

    pub fn audio_config_file(&self) -> Option<PathBuf> {
        self.data_dir
            .lock()
            .unwrap()
            .as_ref()
            .map(|d| d.join("audio_config.json"))
    }

    pub fn chain_state_file(&self) -> Option<PathBuf> {
        self.data_dir
            .lock()
            .unwrap()
            .as_ref()
            .map(|d| d.join("chain_state.json"))
    }

    pub fn save_audio_config(&self) {
        let Some(path) = self.audio_config_file() else {
            return;
        };
        let config = self.config.lock().unwrap().clone();
        let json = match serde_json::to_string_pretty(&config) {
            Ok(j) => j,
            Err(e) => {
                eprintln!("[shallow-host] failed to serialize audio config: {e}");
                return;
            }
        };
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Err(e) = std::fs::write(&path, json) {
            eprintln!("[shallow-host] failed to save audio config: {e}");
        }
    }

    pub fn load_audio_config(&self) {
        let Some(path) = self.audio_config_file() else {
            return;
        };
        if !path.exists() {
            return;
        }
        match std::fs::read_to_string(&path) {
            Ok(json) => {
                if let Ok(mut config) = serde_json::from_str::<AudioConfig>(&json) {
                    if config.buffer_size == 0 {
                        config.buffer_size = 512;
                    }
                    if config.sample_rate == 0 {
                        config.sample_rate = 48000;
                    }
                    *self.config.lock().unwrap() = config;
                }
            }
            Err(e) => eprintln!("[shallow-host] failed to load audio config: {e}"),
        }
    }

    pub fn set_config(&self, mut config: AudioConfig) {
        if config.buffer_size == 0 {
            config.buffer_size = 512;
        }
        if config.sample_rate == 0 {
            config.sample_rate = 48000;
        }
        *self.config.lock().unwrap() = config;
        self.save_audio_config();
    }

    pub fn get_config(&self) -> AudioConfig {
        self.config.lock().unwrap().clone()
    }

    pub fn start(&self) -> Result<(), String> {
        let config = self.config.lock().unwrap().clone();

        let is_none_device = |dev: &Option<String>| {
            dev.is_none() || dev.as_deref() == Some("") || dev.as_deref() == Some("__none")
        };

        if is_none_device(&config.output_device) {
            let _ = ffi::audio_stop();
            *self.running.lock().unwrap() = false;
            return Ok(());
        }

        let input_mask = config
            .active_inputs
            .as_ref()
            .map(|v| v.iter().fold(0i32, |acc, &idx| acc | (1 << idx)))
            .unwrap_or(-1);

        let output_mask = config
            .active_outputs
            .as_ref()
            .map(|v| v.iter().fold(0i32, |acc, &idx| acc | (1 << idx)))
            .unwrap_or(-1);

        let success = ffi::audio_start(
            Some(&config.driver),
            config.input_device.as_deref(),
            config.output_device.as_deref(),
            config.sample_rate as i32,
            config.buffer_size as i32,
            config.mono,
            input_mask,
            output_mask,
        );
        if success {
            *self.running.lock().unwrap() = true;
            Ok(())
        } else {
            Err("Failed to start audio in JUCE".to_string())
        }
    }

    pub fn stop(&self) -> Result<(), String> {
        if ffi::audio_stop() {
            *self.running.lock().unwrap() = false;
            Ok(())
        } else {
            Err("Failed to stop audio in JUCE".to_string())
        }
    }

    pub fn devices(&self) -> Result<AudioDevices, String> {
        let (driver, device) = {
            let config = self.config.lock().unwrap();
            (
                config.driver.clone(),
                config.output_device.clone().unwrap_or_default(),
            )
        };
        let json = ffi::get_audio_devices(&driver, &device);
        serde_json::from_str::<AudioDevices>(&json).map_err(|e| e.to_string())
    }

    pub fn is_running(&self) -> bool {
        *self.running.lock().unwrap()
    }

    pub fn scan_plugins(&self) -> Result<Vec<ScannedPlugin>, String> {
        let json = ffi::scan_plugins();
        serde_json::from_str::<Vec<ScannedPlugin>>(&json).map_err(|e| e.to_string())
    }

    pub fn add_to_chain(&self, plugin_id: &str) -> Result<(), String> {
        let node_id = ffi::add_to_chain(plugin_id);
        if !node_id.is_empty() {
            self.persist();
            Ok(())
        } else {
            Err("Failed to add plugin to chain".to_string())
        }
    }

    pub fn remove_from_chain(&self, node_id: String) -> Result<(), String> {
        if ffi::remove_from_chain(&node_id) {
            self.persist();
            Ok(())
        } else {
            Err("Failed to remove plugin from chain".to_string())
        }
    }

    pub fn move_plugin(&self, node_id: String, up: bool) -> Result<(), String> {
        if ffi::move_plugin(&node_id, up) {
            self.persist();
            Ok(())
        } else {
            Err("Failed to move plugin".to_string())
        }
    }

    pub fn reorder_chain(&self, node_id: String, to_index: usize) -> Result<(), String> {
        if ffi::reorder_chain(&node_id, to_index as i32) {
            self.persist();
            Ok(())
        } else {
            Err("Failed to reorder plugin".to_string())
        }
    }

    pub fn bypass_plugin(&self, node_id: String, bypassed: bool) -> Result<(), String> {
        if ffi::bypass_plugin(&node_id, bypassed) {
            self.persist();
            Ok(())
        } else {
            Err("Failed to bypass plugin".to_string())
        }
    }

    pub fn get_chain(&self) -> Result<Vec<ChainItem>, String> {
        let json = ffi::get_chain();
        serde_json::from_str::<Vec<ChainItem>>(&json).map_err(|e| e.to_string())
    }

    pub fn get_parameters(&self, node_id: String) -> Result<Vec<ParamInfo>, String> {
        let json = ffi::get_plugin_parameters(&node_id);
        serde_json::from_str::<Vec<ParamInfo>>(&json).map_err(|e| e.to_string())
    }

    pub fn set_parameter(
        &self,
        node_id: String,
        param_index: usize,
        value: f64,
    ) -> Result<(), String> {
        if ffi::set_plugin_parameter(&node_id, param_index as i32, value as f32) {
            Ok(())
        } else {
            Err("Failed to set parameter".to_string())
        }
    }

    pub fn open_plugin_gui(&self, node_id: String) -> Result<(), String> {
        if ffi::open_plugin_gui(&node_id) {
            Ok(())
        } else {
            Err("Failed to open plugin GUI".to_string())
        }
    }

    pub fn close_plugin_gui(&self, node_id: String) -> Result<(), String> {
        if ffi::close_plugin_gui(&node_id) {
            Ok(())
        } else {
            Err("Failed to close plugin GUI".to_string())
        }
    }

    pub fn persist(&self) {
        let Some(path) = self.chain_state_file() else {
            return;
        };
        let state = ffi::save_state();
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Err(e) = std::fs::write(&path, state) {
            eprintln!("[shallow-host] failed to save chain state: {e}");
        }
    }

    pub fn restore_from_disk(&self) {
        let Some(path) = self.chain_state_file() else {
            return;
        };
        if !path.exists() {
            return;
        }
        if let Ok(state) = std::fs::read_to_string(&path) {
            ffi::load_state(&state);
        }
    }
}
