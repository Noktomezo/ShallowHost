use crate::engine::audio_io::AudioEngine;
use crate::scanner::vst3::{scan_vst3_raw, ScannedPlugin};
use std::process::Command;

#[tauri::command]
pub async fn scan_plugins(
    engine: tauri::State<'_, AudioEngine>,
) -> Result<Vec<ScannedPlugin>, String> {
    let infos = tauri::async_runtime::spawn_blocking(scan_vst3_raw)
        .await
        .map_err(|e| e.to_string())??;
    let plugins: Vec<ScannedPlugin> = infos.iter().map(ScannedPlugin::from_info).collect();
    engine.cache_scan_results(infos);
    Ok(plugins)
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
