use serde::Serialize;

// ponytail: stubs — Phase 2+ will implement via FFI

#[derive(Debug, Clone, Serialize)]
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

#[derive(Debug, Clone, Serialize)]
pub struct ChainItem {
    pub id: String,
    pub name: String,
    pub format: String,
    pub vendor: String,
    pub bypassed: bool,
    pub unique_id: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
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

#[tauri::command]
pub async fn scan_plugins(
    vst2_paths: Vec<String>,
    vst3_paths: Vec<String>,
) -> Result<Vec<ScannedPlugin>, String> {
    let _ = vst2_paths;
    let _ = vst3_paths;
    Ok(Vec::new())
}

#[tauri::command]
pub async fn reveal_plugin(path: String) -> Result<(), String> {
    let _ = path;
    Ok(())
}

#[tauri::command]
pub async fn add_to_chain(_plugin_id: String) -> Result<(), String> {
    Err("not implemented".into())
}

#[tauri::command]
pub async fn remove_from_chain(_plugin_id: String) -> Result<(), String> {
    Err("not implemented".into())
}

#[tauri::command]
pub async fn move_plugin(_plugin_id: String, _up: bool) -> Result<(), String> {
    Err("not implemented".into())
}

#[tauri::command]
pub async fn reorder_chain(_plugin_id: String, _to_index: usize) -> Result<(), String> {
    Err("not implemented".into())
}

#[tauri::command]
pub async fn bypass_plugin(_plugin_id: String, _bypassed: bool) -> Result<(), String> {
    Err("not implemented".into())
}

#[tauri::command]
pub fn get_chain() -> Result<Vec<ChainItem>, String> {
    Ok(Vec::new())
}

#[tauri::command]
pub fn get_plugin_parameters(_plugin_id: String) -> Result<Vec<ParamInfo>, String> {
    Ok(Vec::new())
}

#[tauri::command]
pub fn set_plugin_parameter(_plugin_id: String, _param_index: usize, _value: f64) -> Result<(), String> {
    Err("not implemented".into())
}

#[tauri::command]
pub async fn open_plugin_gui(_plugin_id: String) -> Result<(), String> {
    Err("not implemented".into())
}
