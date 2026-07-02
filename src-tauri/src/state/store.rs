use std::path::{Path, PathBuf};
use std::sync::Arc;

use base64::{engine::general_purpose, Engine as _};
use serde::{Deserialize, Serialize};
use truce_rack::core::bus::BusLayout;
use truce_rack::core::plugin::PluginCore;
use truce_rack::core::PluginInfo;

use crate::engine::chain::{Chain, ChainEntry};
use crate::scanner::vst3::load_vst3;

const DEFAULT_SAMPLE_RATE: f64 = 48000.0;
const MAX_BLOCK_SIZE: usize = 4096;

#[derive(Serialize, Deserialize, Clone)]
pub struct PersistedScanEntry {
    pub name: String,
    pub vendor: String,
    pub version: u32,
    pub category: String,
    pub path: String,
    pub unique_id: String,
    pub format: String,
    pub has_editor: bool,
    pub accepts_midi: bool,
}

#[derive(Serialize, Deserialize)]
pub struct PersistedScanCache {
    pub plugins: Vec<PersistedScanEntry>,
}

impl PersistedScanEntry {
    pub fn from_info(info: &PluginInfo) -> Self {
        Self {
            name: info.name.clone(),
            vendor: info.vendor.clone(),
            version: info.version,
            category: format!("{:?}", info.category),
            path: info.path.display().to_string(),
            unique_id: info.unique_id.clone(),
            format: info.format.to_string(),
            has_editor: info.has_editor,
            accepts_midi: info.accepts_midi,
        }
    }

    pub fn to_info(&self) -> PluginInfo {
        PluginInfo {
            name: self.name.clone(),
            vendor: self.vendor.clone(),
            version: self.version,
            category: truce_rack::core::info::PluginCategory::Effect,
            path: PathBuf::from(&self.path),
            unique_id: self.unique_id.clone(),
            format: "vst3",
            has_editor: self.has_editor,
            accepts_midi: self.accepts_midi,
        }
    }
}

pub fn save_scan_cache(path: &Path, entries: &[PersistedScanEntry]) -> Result<(), String> {
    let cache = PersistedScanCache {
        plugins: entries.to_vec(),
    };
    let json = serde_json::to_string_pretty(&cache).map_err(|e| e.to_string())?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    std::fs::write(path, json).map_err(|e| e.to_string())
}

pub fn load_scan_cache(path: &Path) -> Result<Vec<PersistedScanEntry>, String> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let json = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    let cache: PersistedScanCache = serde_json::from_str(&json).map_err(|e| e.to_string())?;
    Ok(cache.plugins)
}

#[derive(Serialize, Deserialize)]
pub struct PersistedPlugin {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub vendor: String,
    pub format: String,
    pub path: String,
    #[serde(default = "default_true")]
    pub has_editor: bool,
    pub bypassed: bool,
    pub state: String, // base64-encoded
}

fn default_true() -> bool {
    true
}

#[derive(Serialize, Deserialize)]
pub struct PersistedState {
    pub chain: Vec<PersistedPlugin>,
}

pub fn save_chain(path: &Path, chain: &Chain) -> Result<(), String> {
    eprintln!("[persist] save_chain to {} ({} plugins)", path.display(), chain.len());
    let mut plugins = Vec::with_capacity(chain.len());
    for entry in chain.iter() {
        let plugin = entry.plugin.lock();
        let state = plugin.save_state().map_err(|e| e.to_string())?;
        let state_b64 = general_purpose::STANDARD.encode(&state);
        let bypassed = entry.bypassed.load(std::sync::atomic::Ordering::Relaxed);
        eprintln!("[persist] plugin {} state_bytes={} b64_len={}", entry.name, state.len(), state_b64.len());
        plugins.push(PersistedPlugin {
            id: entry.id.clone(),
            name: entry.name.clone(),
            vendor: entry.vendor.clone(),
            format: entry.format.clone(),
            path: entry.path.clone(),
            has_editor: entry.has_editor,
            bypassed,
            state: state_b64,
        });
    }
    let state = PersistedState { chain: plugins };
    let json = serde_json::to_string_pretty(&state).map_err(|e| e.to_string())?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    std::fs::write(path, json).map_err(|e| e.to_string())
}

pub fn load_chain(path: &Path) -> Result<Vec<PersistedPlugin>, String> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let json = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    let state: PersistedState = serde_json::from_str(&json).map_err(|e| e.to_string())?;
    Ok(state.chain)
}

pub fn restore_chain(plugins: Vec<PersistedPlugin>) -> Chain {
    eprintln!("[persist] restore_chain: {} plugins from disk", plugins.len());
    let mut chain: Chain = Vec::new();
    for p in plugins {
        let state_bytes = general_purpose::STANDARD
            .decode(&p.state)
            .unwrap_or_default();
        eprintln!("[persist] restoring {} state_bytes={}", p.name, state_bytes.len());
        let info = PluginInfo {
            name: p.name.clone(),
            vendor: p.vendor.clone(),
            version: 0,
            category: truce_rack::core::info::PluginCategory::Effect,
            path: PathBuf::from(&p.path),
            unique_id: p.id.clone(),
            format: "vst3",
            has_editor: p.has_editor,
            accepts_midi: false,
        };
        let Ok(mut plugin) = load_vst3(&info) else {
            eprintln!(
                "[shallow-host] failed to load plugin {} from {}",
                p.name, p.path
            );
            continue;
        };
        // Activate first, THEN load state. Some plugins (e.g. Auburn Sounds
        // Renegate) reset parameter values in setupProcessing/setActive,
        // so loading state before activate loses it.
        let _ = plugin.activate(BusLayout::stereo(), DEFAULT_SAMPLE_RATE, MAX_BLOCK_SIZE);
        if !state_bytes.is_empty() {
            if let Err(e) = plugin.load_state(&state_bytes) {
                eprintln!("[shallow-host] failed to restore state for {}: {e}", p.name);
            }
        }
        let entry = ChainEntry::new(&info, Box::new(plugin));
        entry
            .bypassed
            .store(p.bypassed, std::sync::atomic::Ordering::Relaxed);
        chain.push(Arc::new(entry));
    }
    chain
}
