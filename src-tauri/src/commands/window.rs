use crate::engine::audio_io::AudioEngine;
use tauri::Manager;

#[tauri::command]
pub async fn open_plugin_gui(app: tauri::AppHandle, plugin_id: String) -> Result<(), String> {
    let engine = app.state::<AudioEngine>();
    engine.open_plugin_gui(plugin_id)
}

#[tauri::command]
pub async fn close_plugin_gui(app: tauri::AppHandle, plugin_id: String) -> Result<(), String> {
    let engine = app.state::<AudioEngine>();
    engine.close_plugin_gui(plugin_id)
}
