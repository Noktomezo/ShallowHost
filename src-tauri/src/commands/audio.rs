use crate::engine::audio_io::{AudioConfig, AudioDevices, AudioEngine};
use std::sync::atomic::{AtomicBool, Ordering};

static RESTORED: AtomicBool = AtomicBool::new(false);

#[tauri::command]
pub async fn app_ready(
    app: tauri::AppHandle,
    engine: tauri::State<'_, AudioEngine>,
) -> Result<(), String> {
    if !RESTORED.swap(true, Ordering::SeqCst) {
        let engine_clone = (*engine).clone();
        tauri::async_runtime::spawn(async move {
            engine_clone.restore_from_disk(&app).await;
        });
    }
    Ok(())
}

#[tauri::command]
pub async fn start_audio(engine: tauri::State<'_, AudioEngine>) -> Result<(), String> {
    engine.start()
}

#[tauri::command]
pub async fn stop_audio(engine: tauri::State<'_, AudioEngine>) -> Result<(), String> {
    engine.stop()
}

#[tauri::command]
pub fn get_audio_devices(engine: tauri::State<'_, AudioEngine>) -> Result<AudioDevices, String> {
    engine.devices()
}

#[tauri::command]
pub fn is_audio_running(engine: tauri::State<'_, AudioEngine>) -> bool {
    engine.is_running()
}

#[derive(serde::Serialize)]
pub struct AudioLevels {
    pub input: f32,
    pub output: f32,
}

#[tauri::command]
pub fn get_audio_levels(engine: tauri::State<'_, AudioEngine>) -> AudioLevels {
    let (input, output) = engine.get_audio_levels();
    AudioLevels { input, output }
}

#[tauri::command]
pub fn get_audio_config(engine: tauri::State<'_, AudioEngine>) -> AudioConfig {
    engine.get_config()
}

#[tauri::command]
pub fn set_audio_config(
    engine: tauri::State<'_, AudioEngine>,
    config: AudioConfig,
) -> Result<(), String> {
    engine.set_config(config);
    Ok(())
}

#[tauri::command]
pub async fn restart_audio(engine: tauri::State<'_, AudioEngine>) -> Result<(), String> {
    let _ = engine.stop();
    engine.start()
}
