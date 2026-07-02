use parking_lot::Mutex;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use truce_rack::core::plugin::Plugin;
use truce_rack::core::PluginInfo;

pub type Chain = Vec<Arc<ChainEntry>>;

pub struct ChainEntry {
    pub id: String,
    pub name: String,
    pub vendor: String,
    pub format: String,
    pub path: String,
    pub has_editor: bool,
    pub bypassed: AtomicBool,
    pub process_failed: AtomicBool,
    // ponytail: Mutex per plugin — uncontended on audio thread (~10ns lock),
    // swap to UnsafeCell + !Send guard if RT profiling demands it
    pub plugin: Mutex<Box<dyn Plugin<f32>>>,
}

impl ChainEntry {
    pub fn new(info: &PluginInfo, plugin: Box<dyn Plugin<f32>>) -> Self {
        Self {
            id: info.unique_id.clone(),
            name: info.name.clone(),
            vendor: info.vendor.clone(),
            format: info.format.to_string(),
            path: info.path.display().to_string(),
            has_editor: info.has_editor,
            bypassed: AtomicBool::new(false),
            process_failed: AtomicBool::new(false),
            plugin: Mutex::new(plugin),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ChainItem {
    pub id: String,
    pub name: String,
    pub vendor: String,
    pub format: String,
    pub bypassed: bool,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ParamInfo {
    pub index: usize,
    pub name: String,
    pub unit: String,
    pub min: f64,
    pub max: f64,
    pub default: f64,
    pub step_count: u32,
    pub value: f64,
}

pub fn chain_to_items(chain: &Chain) -> Vec<ChainItem> {
    chain
        .iter()
        .map(|e| ChainItem {
            id: e.id.clone(),
            name: e.name.clone(),
            vendor: e.vendor.clone(),
            format: e.format.clone(),
            bypassed: e.bypassed.load(std::sync::atomic::Ordering::Relaxed),
        })
        .collect()
}
