use std::collections::HashMap;
use std::ffi::c_void;
use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, SyncSender};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use arc_swap::ArcSwap;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::SampleFormat;
use parking_lot::Mutex;
use ringbuf::traits::{Consumer, Producer, Split};
use ringbuf::HeapRb;
use truce_rack::core::buffer::{AudioBuffer, BusRange};
use truce_rack::core::bus::BusLayout;
use truce_rack::core::editor::WindowHandle;
use truce_rack::core::events::EventList;
use truce_rack::core::plugin::{PluginCore, ProcessContext, ProcessStatus};
use truce_rack::core::PluginInfo;

use crate::engine::chain::{chain_to_items, Chain, ChainEntry, ChainItem, ParamInfo};
use crate::scanner::vst3::load_vst3;
use crate::state::store::PersistedScanEntry;

const DEFAULT_SAMPLE_RATE: f64 = 48000.0;
const MAX_BLOCK_SIZE: usize = 4096;

pub struct AudioEngine {
    cmd_tx: SyncSender<AudioCmd>,
    running: Mutex<bool>,
    scan_cache: Mutex<HashMap<String, PluginInfo>>,
    pub chain_handle: Arc<ArcSwap<Chain>>,
    data_dir: Arc<Mutex<Option<PathBuf>>>,
    config: Mutex<AudioConfig>,
}

enum AudioCmd {
    Start {
        config: AudioConfig,
        reply: mpsc::Sender<Result<(), String>>,
    },
    Stop {
        reply: mpsc::Sender<Result<(), String>>,
    },
    Devices {
        reply: mpsc::Sender<AudioDevices>,
    },
    AddToChain {
        info: PluginInfo,
        reply: mpsc::Sender<Result<(), String>>,
    },
    RemoveFromChain {
        id: String,
        reply: mpsc::Sender<Result<(), String>>,
    },
    MovePlugin {
        id: String,
        up: bool,
        reply: mpsc::Sender<Result<(), String>>,
    },
    ReorderChain {
        id: String,
        to_index: usize,
        reply: mpsc::Sender<Result<(), String>>,
    },
    BypassPlugin {
        id: String,
        bypassed: bool,
        reply: mpsc::Sender<Result<(), String>>,
    },
    GetChain {
        reply: mpsc::Sender<Vec<ChainItem>>,
    },
    Persist {
        reply: mpsc::Sender<Result<(), String>>,
    },
    RestoreFromDisk {
        reply: mpsc::Sender<Result<(), String>>,
    },
    GetParameters {
        plugin_id: String,
        reply: mpsc::Sender<Result<Vec<ParamInfo>, String>>,
    },
    SetParameter {
        plugin_id: String,
        param_index: usize,
        value: f64,
        reply: mpsc::Sender<Result<(), String>>,
    },
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct AudioConfig {
    pub driver: String,
    pub input_device: Option<String>,
    pub output_device: Option<String>,
    pub sample_rate: u32,
    pub buffer_size: u32,
    pub mono: bool,
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            driver: "wasapi".to_string(),
            input_device: None,
            output_device: None,
            sample_rate: 48000,
            buffer_size: 512,
            mono: false,
        }
    }
}

#[derive(serde::Serialize, Clone)]
pub struct AudioDevices {
    pub inputs: Vec<DeviceInfo>,
    pub outputs: Vec<DeviceInfo>,
}

#[derive(serde::Serialize, Clone)]
pub struct DeviceInfo {
    pub name: String,
    pub default: bool,
}

impl AudioEngine {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::sync_channel(16);
        let chain_handle: Arc<ArcSwap<Chain>> = Arc::new(ArcSwap::from_pointee(Vec::new()));
        let chain_for_thread = chain_handle.clone();
        let data_dir: Arc<Mutex<Option<PathBuf>>> = Arc::new(Mutex::new(None));
        let data_dir_for_thread = data_dir.clone();
        thread::spawn(move || audio_thread(rx, chain_for_thread, data_dir_for_thread));
        Self {
            cmd_tx: tx,
            running: Mutex::new(false),
            scan_cache: Mutex::new(HashMap::new()),
            chain_handle,
            data_dir,
            config: Mutex::new(AudioConfig::default()),
        }
    }

    pub fn set_data_dir(&self, dir: PathBuf) {
        *self.data_dir.lock() = Some(dir);
    }

    fn scan_cache_file(&self) -> Option<PathBuf> {
        self.data_dir
            .lock()
            .as_ref()
            .map(|d| d.join("scan_cache.json"))
    }

    fn audio_config_file(&self) -> Option<PathBuf> {
        self.data_dir
            .lock()
            .as_ref()
            .map(|d| d.join("audio_config.json"))
    }

    pub fn save_audio_config(&self) {
        let Some(path) = self.audio_config_file() else {
            return;
        };
        let config = self.config.lock().clone();
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
                if let Ok(config) = serde_json::from_str::<AudioConfig>(&json) {
                    *self.config.lock() = config;
                }
            }
            Err(e) => eprintln!("[shallow-host] failed to load audio config: {e}"),
        }
    }

    pub fn set_config(&self, config: AudioConfig) {
        *self.config.lock() = config;
        self.save_audio_config();
    }

    pub fn get_config(&self) -> AudioConfig {
        self.config.lock().clone()
    }

    pub fn persist(&self) {
        let (tx, rx) = mpsc::channel();
        let _ = self.cmd_tx.send(AudioCmd::Persist { reply: tx });
        let _ = rx.recv_timeout(Duration::from_secs(5));
    }

    pub fn restore_from_disk(&self) {
        let (tx, rx) = mpsc::channel();
        let _ = self.cmd_tx.send(AudioCmd::RestoreFromDisk { reply: tx });
        let _ = rx.recv_timeout(Duration::from_secs(30));
    }

    pub fn cache_scan_results(&self, infos: Vec<PluginInfo>) {
        let entries: Vec<PersistedScanEntry> =
            infos.iter().map(PersistedScanEntry::from_info).collect();
        {
            let mut cache = self.scan_cache.lock();
            cache.clear();
            for info in infos {
                cache.insert(info.unique_id.clone(), info);
            }
        }
        if let Some(path) = self.scan_cache_file() {
            if let Err(e) = crate::state::store::save_scan_cache(&path, &entries) {
                eprintln!("[shallow-host] failed to save scan cache: {e}");
            }
        }
    }

    pub fn load_scan_cache(&self) {
        let Some(path) = self.scan_cache_file() else {
            return;
        };
        match crate::state::store::load_scan_cache(&path) {
            Ok(entries) => {
                let mut cache = self.scan_cache.lock();
                for entry in entries {
                    let info = entry.to_info();
                    cache.insert(info.unique_id.clone(), info);
                }
            }
            Err(e) => eprintln!("[shallow-host] failed to load scan cache: {e}"),
        }
    }

    pub fn start(&self) -> Result<(), String> {
        let config = self.config.lock().clone();
        let (tx, rx) = mpsc::channel();
        self.cmd_tx
            .send(AudioCmd::Start { config, reply: tx })
            .map_err(|e| e.to_string())?;
        // 5 retries × 500ms = up to 2.5s inside audio thread
        let result = rx
            .recv_timeout(Duration::from_secs(10))
            .map_err(|e| e.to_string())?;
        if result.is_ok() {
            *self.running.lock() = true;
        }
        result
    }

    pub fn stop(&self) -> Result<(), String> {
        let (tx, rx) = mpsc::channel();
        self.cmd_tx
            .send(AudioCmd::Stop { reply: tx })
            .map_err(|e| e.to_string())?;
        let result = rx
            .recv_timeout(Duration::from_secs(5))
            .map_err(|e| e.to_string())?;
        if result.is_ok() {
            *self.running.lock() = false;
        }
        result
    }

    pub fn devices(&self) -> Result<AudioDevices, String> {
        let (tx, rx) = mpsc::channel();
        self.cmd_tx
            .send(AudioCmd::Devices { reply: tx })
            .map_err(|e| e.to_string())?;
        rx.recv_timeout(Duration::from_secs(5))
            .map_err(|e| e.to_string())
    }

    pub fn is_running(&self) -> bool {
        *self.running.lock()
    }

    pub fn add_to_chain(&self, plugin_id: String) -> Result<(), String> {
        let info = self
            .scan_cache
            .lock()
            .get(&plugin_id)
            .ok_or_else(|| format!("plugin {plugin_id} not in scan cache"))?
            .clone();
        let (tx, rx) = mpsc::channel();
        self.cmd_tx
            .send(AudioCmd::AddToChain { info, reply: tx })
            .map_err(|e| e.to_string())?;
        let result = rx
            .recv_timeout(Duration::from_secs(30))
            .map_err(|e| e.to_string())?;
        if result.is_ok() {
            self.persist();
        }
        result
    }

    pub fn remove_from_chain(&self, id: String) -> Result<(), String> {
        let (tx, rx) = mpsc::channel();
        self.cmd_tx
            .send(AudioCmd::RemoveFromChain { id, reply: tx })
            .map_err(|e| e.to_string())?;
        let result = rx
            .recv_timeout(Duration::from_secs(5))
            .map_err(|e| e.to_string())?;
        if result.is_ok() {
            self.persist();
        }
        result
    }

    pub fn move_plugin(&self, id: String, up: bool) -> Result<(), String> {
        let (tx, rx) = mpsc::channel();
        self.cmd_tx
            .send(AudioCmd::MovePlugin { id, up, reply: tx })
            .map_err(|e| e.to_string())?;
        let result = rx
            .recv_timeout(Duration::from_secs(5))
            .map_err(|e| e.to_string())?;
        if result.is_ok() {
            self.persist();
        }
        result
    }

    pub fn reorder_chain(&self, id: String, to_index: usize) -> Result<(), String> {
        let (tx, rx) = mpsc::channel();
        self.cmd_tx
            .send(AudioCmd::ReorderChain {
                id,
                to_index,
                reply: tx,
            })
            .map_err(|e| e.to_string())?;
        let result = rx
            .recv_timeout(Duration::from_secs(5))
            .map_err(|e| e.to_string())?;
        if result.is_ok() {
            self.persist();
        }
        result
    }

    pub fn bypass_plugin(&self, id: String, bypassed: bool) -> Result<(), String> {
        let (tx, rx) = mpsc::channel();
        self.cmd_tx
            .send(AudioCmd::BypassPlugin {
                id,
                bypassed,
                reply: tx,
            })
            .map_err(|e| e.to_string())?;
        let result = rx
            .recv_timeout(Duration::from_secs(5))
            .map_err(|e| e.to_string())?;
        if result.is_ok() {
            self.persist();
        }
        result
    }

    pub fn get_chain(&self) -> Result<Vec<ChainItem>, String> {
        let (tx, rx) = mpsc::channel();
        self.cmd_tx
            .send(AudioCmd::GetChain { reply: tx })
            .map_err(|e| e.to_string())?;
        rx.recv_timeout(Duration::from_secs(5))
            .map_err(|e| e.to_string())
    }

    pub fn get_parameters(&self, plugin_id: String) -> Result<Vec<ParamInfo>, String> {
        let (tx, rx) = mpsc::channel();
        self.cmd_tx
            .send(AudioCmd::GetParameters {
                plugin_id,
                reply: tx,
            })
            .map_err(|e| e.to_string())?;
        rx.recv_timeout(Duration::from_secs(5))
            .map_err(|e| e.to_string())?
    }

    pub fn set_parameter(
        &self,
        plugin_id: String,
        param_index: usize,
        value: f64,
    ) -> Result<(), String> {
        let (tx, rx) = mpsc::channel();
        self.cmd_tx
            .send(AudioCmd::SetParameter {
                plugin_id,
                param_index,
                value,
                reply: tx,
            })
            .map_err(|e| e.to_string())?;
        let result = rx
            .recv_timeout(Duration::from_secs(5))
            .map_err(|e| e.to_string())?;
        if result.is_ok() {
            eprintln!("[persist] param set ok, persisting immediately");
            self.persist();
        }
        result
    }

    pub fn open_editor(&self, plugin_id: &str, parent: *mut c_void) -> Result<(u32, u32), String> {
        let chain = self.chain_handle.load();
        let entry = chain
            .iter()
            .find(|e| e.id == plugin_id)
            .ok_or_else(|| format!("plugin {plugin_id} not in chain"))?;
        let mut plugin = entry.plugin.lock();
        let editor = plugin
            .editor()
            .ok_or_else(|| format!("plugin {} has no editor", plugin_id))?;
        editor
            .open(WindowHandle::HWND(parent), 1.0)
            .map_err(|e| e.to_string())?;
        let size = editor.size().unwrap_or((400, 300));
        // Some VST3 editors attach as a 0x0 child until the host sends onSize.
        Ok(editor.set_size(size.0, size.1).unwrap_or(size))
    }

    pub fn close_editor(&self, plugin_id: &str) -> Result<(), String> {
        let chain = self.chain_handle.load();
        let entry = chain
            .iter()
            .find(|e| e.id == plugin_id)
            .ok_or_else(|| format!("plugin {plugin_id} not in chain"))?;
        let mut plugin = entry.plugin.lock();
        if let Some(editor) = plugin.editor() {
            editor.close();
        }
        Ok(())
    }
}

fn audio_thread(
    rx: Receiver<AudioCmd>,
    chain_handle: Arc<ArcSwap<Chain>>,
    data_dir: Arc<Mutex<Option<PathBuf>>>,
) {
    #[cfg(windows)]
    {
        use windows::Win32::System::Com::{CoInitializeEx, COINIT_MULTITHREADED};
        unsafe {
            let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
        }
    }

    let host = cpal::default_host();
    let mut streams: Option<(cpal::Stream, cpal::Stream)> = None;
    let mut current_config: Option<(f64, usize)> = None;
    // ponytail: keep old chain alive one generation so plugin drop happens on this thread (COM-initialized),
    // not on cpal's callback thread. Drop prev when next mutation proves callback moved to newer chain.
    let mut prev_chain: Option<Arc<Chain>> = None;

    while let Ok(cmd) = rx.recv() {
        match cmd {
            AudioCmd::Start { config, reply } => {
                if streams.is_some() {
                    let _ = reply.send(Ok(()));
                    continue;
                }
                // ponytail: retry up to 5x with 500ms delay — WASAPI
                // sometimes fails on first attempt at app launch
                // because the audio endpoint isn't fully enumerated.
                let mut last_err = String::new();
                let mut started = false;
                for attempt in 0..5u32 {
                    match start_streams(&host, &chain_handle, &config) {
                        Ok((input, output, cfg)) => {
                            streams = Some((input, output));
                            current_config = Some(cfg);
                            reactivate_all(&chain_handle, cfg.0, cfg.1);
                            started = true;
                            break;
                        }
                        Err(e) => {
                            last_err = e.to_string();
                            eprintln!(
                                "[shallow-host] start_streams attempt {} failed: {last_err}",
                                attempt + 1
                            );
                            thread::sleep(Duration::from_millis(500));
                        }
                    }
                }
                if started {
                    let _ = reply.send(Ok(()));
                } else {
                    let _ = reply.send(Err(last_err));
                }
            }
            AudioCmd::Stop { reply } => {
                streams = None;
                current_config = None;
                let _ = reply.send(Ok(()));
            }
            AudioCmd::Devices { reply } => {
                let _ = reply.send(list_devices(&host));
            }
            AudioCmd::AddToChain { info, reply } => {
                drop(prev_chain.take());
                prev_chain = Some(chain_handle.load_full());
                let result = add_to_chain(&chain_handle, &info, current_config);
                let _ = reply.send(result);
            }
            AudioCmd::RemoveFromChain { id, reply } => {
                drop(prev_chain.take());
                prev_chain = Some(chain_handle.load_full());
                remove_from_chain(&chain_handle, &id);
                let _ = reply.send(Ok(()));
            }
            AudioCmd::MovePlugin { id, up, reply } => {
                drop(prev_chain.take());
                prev_chain = Some(chain_handle.load_full());
                let result = move_plugin(&chain_handle, &id, up);
                let _ = reply.send(result);
            }
            AudioCmd::ReorderChain {
                id,
                to_index,
                reply,
            } => {
                drop(prev_chain.take());
                prev_chain = Some(chain_handle.load_full());
                let result = reorder_chain(&chain_handle, &id, to_index);
                let _ = reply.send(result);
            }
            AudioCmd::BypassPlugin {
                id,
                bypassed,
                reply,
            } => {
                // bypass doesn't swap the chain (atomic flag only), no drop race
                bypass_plugin(&chain_handle, &id, bypassed);
                let _ = reply.send(Ok(()));
            }
            AudioCmd::GetChain { reply } => {
                let chain = chain_handle.load();
                let _ = reply.send(chain_to_items(&chain));
            }
            AudioCmd::Persist { reply } => {
                let result = (|| -> Result<(), String> {
                    let Some(path) = data_dir.lock().as_ref().map(|d| d.join("state.json")) else {
                        return Ok(());
                    };
                    let chain = chain_handle.load();
                    eprintln!(
                        "[persist] persist() on audio thread, {} plugins",
                        chain.len()
                    );
                    crate::state::store::save_chain(&path, &chain)
                })();
                let _ = reply.send(result);
            }
            AudioCmd::RestoreFromDisk { reply } => {
                let result = (|| -> Result<(), String> {
                    let Some(path) = data_dir.lock().as_ref().map(|d| d.join("state.json")) else {
                        return Ok(());
                    };
                    let plugins = crate::state::store::load_chain(&path)?;
                    if plugins.is_empty() {
                        return Ok(());
                    }
                    let chain = crate::state::store::restore_chain(plugins);
                    chain_handle.store(Arc::new(chain));
                    Ok(())
                })();
                let _ = reply.send(result);
            }
            AudioCmd::GetParameters { plugin_id, reply } => {
                let chain = chain_handle.load();
                let result = get_parameters(&chain, &plugin_id);
                let _ = reply.send(result);
            }
            AudioCmd::SetParameter {
                plugin_id,
                param_index,
                value,
                reply,
            } => {
                let chain = chain_handle.load();
                let result = set_parameter(&chain, &plugin_id, param_index, value);
                let _ = reply.send(result);
            }
        }
    }
}

fn reactivate_all(chain_handle: &Arc<ArcSwap<Chain>>, sample_rate: f64, max_block: usize) {
    let chain = chain_handle.load();
    for entry in chain.iter() {
        let mut plugin = entry.plugin.lock();
        if plugin.is_active() {
            plugin.deactivate();
        }
        let _ = plugin.activate(BusLayout::stereo(), sample_rate, max_block);
    }
}

fn add_to_chain(
    chain_handle: &Arc<ArcSwap<Chain>>,
    info: &PluginInfo,
    current_config: Option<(f64, usize)>,
) -> Result<(), String> {
    eprintln!(
        "[chain] loading plugin: {} vendor='{}' from {}",
        info.name,
        info.vendor,
        info.path.display()
    );
    let mut plugin = load_vst3(info).map_err(|e| {
        eprintln!("[chain] load FAILED: {e}");
        e
    })?;
    eprintln!("[chain] loaded ok, activating...");
    let (sr, bs) = current_config.unwrap_or((DEFAULT_SAMPLE_RATE, MAX_BLOCK_SIZE));
    plugin.activate(BusLayout::stereo(), sr, bs).map_err(|e| {
        eprintln!("[chain] activate FAILED: {e}");
        e.to_string()
    })?;
    eprintln!("[chain] activated ok, storing in chain...");
    let entry = Arc::new(ChainEntry::new(info, Box::new(plugin)));
    let old = chain_handle.load();
    let mut new_chain: Chain = old.iter().cloned().collect();
    new_chain.push(entry);
    chain_handle.store(Arc::new(new_chain));
    eprintln!("[chain] stored ok");
    Ok(())
}

fn remove_from_chain(chain_handle: &Arc<ArcSwap<Chain>>, id: &str) {
    let old = chain_handle.load();
    let new_chain: Chain = old.iter().filter(|e| e.id != id).cloned().collect();
    chain_handle.store(Arc::new(new_chain));
}

fn move_plugin(chain_handle: &Arc<ArcSwap<Chain>>, id: &str, up: bool) -> Result<(), String> {
    let old = chain_handle.load();
    let pos = old
        .iter()
        .position(|e| e.id == id)
        .ok_or_else(|| format!("plugin {id} not in chain"))?;
    let target = if up {
        pos.checked_sub(1)
    } else {
        Some(pos + 1).filter(|&i| i < old.len())
    };
    let Some(target) = target else {
        return Ok(()); // already at edge
    };
    let mut new_chain: Chain = old.iter().cloned().collect();
    new_chain.swap(pos, target);
    chain_handle.store(Arc::new(new_chain));
    Ok(())
}

fn reorder_chain(
    chain_handle: &Arc<ArcSwap<Chain>>,
    id: &str,
    to_index: usize,
) -> Result<(), String> {
    let old = chain_handle.load();
    let pos = old
        .iter()
        .position(|e| e.id == id)
        .ok_or_else(|| format!("plugin {id} not in chain"))?;
    if pos == to_index || to_index >= old.len() {
        return Ok(());
    }
    let mut new_chain: Chain = old.iter().cloned().collect();
    let entry = new_chain.remove(pos);
    let insert_at = if to_index > pos {
        to_index - 1
    } else {
        to_index
    };
    new_chain.insert(insert_at.min(new_chain.len()), entry);
    chain_handle.store(Arc::new(new_chain));
    Ok(())
}

fn bypass_plugin(chain_handle: &Arc<ArcSwap<Chain>>, id: &str, bypassed: bool) {
    let chain = chain_handle.load();
    if let Some(entry) = chain.iter().find(|e| e.id == id) {
        entry
            .bypassed
            .store(bypassed, std::sync::atomic::Ordering::Relaxed);
    }
}

fn get_parameters(chain: &Chain, plugin_id: &str) -> Result<Vec<ParamInfo>, String> {
    use truce_rack::core::info::ParameterFlags;
    let entry = chain
        .iter()
        .find(|e| e.id == plugin_id)
        .ok_or_else(|| format!("plugin {plugin_id} not in chain"))?;
    let plugin = entry.plugin.lock();
    let count = plugin.parameter_count();
    let mut params = Vec::new();
    for i in 0..count {
        let Ok(info) = plugin.parameter_info(i) else {
            continue;
        };
        if info.flags.contains(ParameterFlags::HIDDEN)
            || info.flags.contains(ParameterFlags::READ_ONLY)
        {
            continue;
        }
        let value = plugin.parameter_value(i).unwrap_or(info.default);
        params.push(ParamInfo {
            index: i,
            name: info.name.clone(),
            unit: info.unit.clone(),
            min: info.min,
            max: info.max,
            default: info.default,
            step_count: info.step_count,
            value,
        });
    }
    Ok(params)
}

fn set_parameter(chain: &Chain, plugin_id: &str, index: usize, value: f64) -> Result<(), String> {
    let entry = chain
        .iter()
        .find(|e| e.id == plugin_id)
        .ok_or_else(|| format!("plugin {plugin_id} not in chain"))?;
    let mut plugin = entry.plugin.lock();
    plugin
        .set_parameter(index, value)
        .map_err(|e| e.to_string())
}

fn list_devices(host: &cpal::Host) -> AudioDevices {
    let default_input = host.default_input_device().map(|d| d.to_string());
    let default_output = host.default_output_device().map(|d| d.to_string());

    let inputs = host
        .input_devices()
        .map(|devs| {
            devs.map(|d| DeviceInfo {
                name: d.to_string(),
                default: Some(&d.to_string()) == default_input.as_ref(),
            })
            .collect()
        })
        .unwrap_or_default();

    let outputs = host
        .output_devices()
        .map(|devs| {
            devs.map(|d| DeviceInfo {
                name: d.to_string(),
                default: Some(&d.to_string()) == default_output.as_ref(),
            })
            .collect()
        })
        .unwrap_or_default();

    AudioDevices { inputs, outputs }
}

type StreamResult = Result<(cpal::Stream, cpal::Stream, (f64, usize)), Box<dyn std::error::Error>>;

fn find_device(host: &cpal::Host, name: &str, input: bool) -> Option<cpal::Device> {
    if input {
        host.input_devices().ok()?.find(|d| d.to_string() == name)
    } else {
        host.output_devices().ok()?.find(|d| d.to_string() == name)
    }
}

fn start_streams(
    host: &cpal::Host,
    chain_handle: &Arc<ArcSwap<Chain>>,
    config: &AudioConfig,
) -> StreamResult {
    let input_dev = match &config.input_device {
        Some(name) => find_device(host, name, true)
            .or_else(|| {
                eprintln!("[shallow-host] input device '{name}' not found, using default");
                host.default_input_device()
            })
            .ok_or("no input device available")?,
        None => host.default_input_device().ok_or("no input device")?,
    };

    let output_dev = match &config.output_device {
        Some(name) => find_device(host, name, false)
            .or_else(|| {
                eprintln!("[shallow-host] output device '{name}' not found, using default");
                host.default_output_device()
            })
            .ok_or("no output device available")?,
        None => host.default_output_device().ok_or("no output device")?,
    };

    let input_supported = input_dev.default_input_config()?;
    let output_supported = output_dev.default_output_config()?;

    if input_supported.sample_format() != SampleFormat::F32 {
        return Err(format!(
            "input sample format {:?} not supported (expected F32)",
            input_supported.sample_format()
        )
        .into());
    }
    if output_supported.sample_format() != SampleFormat::F32 {
        return Err(format!(
            "output sample format {:?} not supported (expected F32)",
            output_supported.sample_format()
        )
        .into());
    }

    let in_ch = input_supported.channels();
    let out_ch = output_supported.channels();
    // ponytail: use device's default sample rate — WASAPI shared mode
    // rejects arbitrary rates. Config's sample_rate is a preference only.
    let sample_rate = output_supported.sample_rate();
    let buffer_size = cpal::BufferSize::Fixed(config.buffer_size);

    let input_config = cpal::StreamConfig {
        channels: in_ch,
        sample_rate,
        buffer_size,
    };
    let output_config = cpal::StreamConfig {
        channels: out_ch,
        sample_rate,
        buffer_size,
    };

    let in_ch = in_ch as usize;
    let out_ch = out_ch as usize;
    let out_rate = sample_rate as usize;

    let buf_cap = (out_rate * out_ch * 2).max(4096);
    let (mut producer, mut consumer) = HeapRb::<f32>::new(buf_cap).split();

    let sample_rate = out_rate as f64;
    let chain_for_cb = chain_handle.clone();
    let mono = config.mono;

    let mut scratch = ScratchBuffers::new(MAX_BLOCK_SIZE, out_ch);

    let input_stream = input_dev.build_input_stream(
        input_config,
        move |data: &[f32], _| {
            if mono && in_ch >= 2 && out_ch >= 2 {
                let frames = data.len() / in_ch;
                for i in 0..frames {
                    let left = data[i * in_ch];
                    let _ = producer.push_slice(&[left, left]);
                }
            } else if in_ch == out_ch {
                let _ = producer.push_slice(data);
            } else if in_ch == 1 && out_ch == 2 {
                for &s in data {
                    let _ = producer.push_slice(&[s, s]);
                }
            } else {
                let _ = producer.push_slice(data);
            }
        },
        |err| eprintln!("[shallow-host] input stream error: {err}"),
        None,
    )?;

    let output_stream = output_dev.build_output_stream(
        output_config,
        move |data: &mut [f32], _| {
            let frames = (data.len() / out_ch).min(MAX_BLOCK_SIZE);

            scratch.interleaved[..frames * out_ch].fill(0.0);
            let _read = consumer.pop_slice(&mut scratch.interleaved[..frames * out_ch]);

            let chain_guard = chain_for_cb.load();
            let chain: &Chain = &chain_guard;

            let has_active = !chain.is_empty()
                && chain
                    .iter()
                    .any(|e| !e.bypassed.load(std::sync::atomic::Ordering::Relaxed));

            if !has_active || out_ch < 2 {
                data[..frames * out_ch].copy_from_slice(&scratch.interleaved[..frames * out_ch]);
                return;
            }

            for i in 0..frames {
                scratch.in_l[i] = scratch.interleaved[i * out_ch];
                scratch.in_r[i] = scratch.interleaved[i * out_ch + 1];
            }

            let bus_ranges = [BusRange::new(0, 2)];
            let events = EventList::new();
            let mut output_events = EventList::new();

            for entry in chain.iter() {
                if entry.bypassed.load(std::sync::atomic::Ordering::Relaxed) {
                    continue;
                }
                let mut plugin = entry.plugin.lock();

                {
                    let inputs: &[&[f32]] = &[&scratch.in_l[..frames], &scratch.in_r[..frames]];
                    let outputs: &mut [&mut [f32]] =
                        &mut [&mut scratch.out_l[..frames], &mut scratch.out_r[..frames]];
                    let mut audio_buf =
                        AudioBuffer::new(inputs, outputs, frames, &bus_ranges, &bus_ranges);
                    let mut context = ProcessContext {
                        sample_rate,
                        max_block_size: MAX_BLOCK_SIZE,
                        transport: None,
                        output_events: &mut output_events,
                    };
                    match plugin.process(&mut audio_buf, &events, &mut context) {
                        Ok(ProcessStatus::Error) => {
                            if !entry
                                .process_failed
                                .swap(true, std::sync::atomic::Ordering::Relaxed)
                            {
                                eprintln!(
                                    "[shallow-host] plugin process returned error: {}",
                                    entry.name
                                );
                            }
                            continue;
                        }
                        Err(e) => {
                            if !entry
                                .process_failed
                                .swap(true, std::sync::atomic::Ordering::Relaxed)
                            {
                                eprintln!(
                                    "[shallow-host] plugin process failed: {}: {e}",
                                    entry.name
                                );
                            }
                            continue;
                        }
                        Ok(_) => {}
                    }
                }

                output_events.clear();
                std::mem::swap(&mut scratch.in_l, &mut scratch.out_l);
                std::mem::swap(&mut scratch.in_r, &mut scratch.out_r);
            }

            for i in 0..frames {
                data[i * out_ch] = scratch.in_l[i];
                data[i * out_ch + 1] = scratch.in_r[i];
            }
        },
        |err| eprintln!("[shallow-host] output stream error: {err}"),
        None,
    )?;

    input_stream.play()?;
    output_stream.play()?;

    Ok((input_stream, output_stream, (sample_rate, MAX_BLOCK_SIZE)))
}

struct ScratchBuffers {
    interleaved: Vec<f32>,
    in_l: Vec<f32>,
    in_r: Vec<f32>,
    out_l: Vec<f32>,
    out_r: Vec<f32>,
}

impl ScratchBuffers {
    fn new(max_frames: usize, channels: usize) -> Self {
        Self {
            interleaved: vec![0f32; max_frames * channels],
            in_l: vec![0f32; max_frames],
            in_r: vec![0f32; max_frames],
            out_l: vec![0f32; max_frames],
            out_r: vec![0f32; max_frames],
        }
    }
}
