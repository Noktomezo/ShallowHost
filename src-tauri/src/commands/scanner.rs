use crate::engine::audio_io::AudioEngine;
use std::process::Command;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
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

#[tauri::command]
pub async fn scan_plugins(
    engine: tauri::State<'_, AudioEngine>,
) -> Result<Vec<ScannedPlugin>, String> {
    engine.scan_plugins()
}

#[tauri::command]
pub async fn reveal_plugin(path: String) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        Command::new("explorer.exe")
            .args(["/select,", &path])
            .spawn()
            .map_err(|e| e.to_string())?;
        Ok(())
    })
    .await
    .map_err(|e| e.to_string())?
}
