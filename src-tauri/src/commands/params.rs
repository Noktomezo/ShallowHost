use crate::engine::audio_io::AudioEngine;
use crate::engine::chain::ParamInfo;

#[tauri::command]
pub fn get_plugin_parameters(
    engine: tauri::State<'_, AudioEngine>,
    plugin_id: String,
) -> Result<Vec<ParamInfo>, String> {
    engine.get_parameters(plugin_id)
}

#[tauri::command]
pub fn set_plugin_parameter(
    engine: tauri::State<'_, AudioEngine>,
    plugin_id: String,
    param_index: usize,
    value: f64,
) -> Result<(), String> {
    engine.set_parameter(plugin_id, param_index, value)
}
