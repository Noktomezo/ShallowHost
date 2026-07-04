use crate::engine::audio_io::{AudioConfig, AudioDevices, AudioEngine};

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
