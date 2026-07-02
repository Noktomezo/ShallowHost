use crate::engine::audio_io::AudioEngine;
use crate::engine::chain::ChainItem;

#[tauri::command]
pub fn add_to_chain(
    engine: tauri::State<'_, AudioEngine>,
    plugin_id: String,
) -> Result<(), String> {
    engine.add_to_chain(plugin_id)
}

#[tauri::command]
pub fn remove_from_chain(
    engine: tauri::State<'_, AudioEngine>,
    plugin_id: String,
) -> Result<(), String> {
    engine.remove_from_chain(plugin_id)
}

#[tauri::command]
pub fn move_plugin(
    engine: tauri::State<'_, AudioEngine>,
    plugin_id: String,
    up: bool,
) -> Result<(), String> {
    engine.move_plugin(plugin_id, up)
}

#[tauri::command]
pub fn reorder_chain(
    engine: tauri::State<'_, AudioEngine>,
    plugin_id: String,
    to_index: usize,
) -> Result<(), String> {
    engine.reorder_chain(plugin_id, to_index)
}

#[tauri::command]
pub fn bypass_plugin(
    engine: tauri::State<'_, AudioEngine>,
    plugin_id: String,
    bypassed: bool,
) -> Result<(), String> {
    engine.bypass_plugin(plugin_id, bypassed)
}

#[tauri::command]
pub fn get_chain(engine: tauri::State<'_, AudioEngine>) -> Result<Vec<ChainItem>, String> {
    engine.get_chain()
}
