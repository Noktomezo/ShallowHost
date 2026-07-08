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
    vst2_paths: Vec<String>,
    vst3_paths: Vec<String>,
) -> Result<Vec<ScannedPlugin>, String> {
    engine.scan_plugins(vst2_paths, vst3_paths)
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

#[tauri::command]
pub async fn select_directory() -> Result<Option<String>, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let dir = rfd::FileDialog::new().pick_folder();
        Ok(dir.map(|p| p.to_string_lossy().into_owned()))
    })
    .await
    .map_err(|e| e.to_string())?
}
