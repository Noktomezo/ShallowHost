mod commands;
mod engine;
mod scanner;
mod state;

use engine::audio_io::AudioEngine;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .manage(AudioEngine::new())
        .setup(|app| {
            let engine = app.state::<AudioEngine>();
            if let Ok(data_dir) = app.path().app_data_dir() {
                engine.set_data_dir(data_dir);
                engine.load_audio_config();
                engine.load_scan_cache();
                engine.restore_from_disk();
            }
            // Start audio immediately — not tied to React lifecycle,
            // so HMR / route changes don't stop/restart streams.
            if let Err(e) = engine.start() {
                eprintln!("[shallow-host] audio failed to start on launch: {e}");
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::audio::start_audio,
            commands::audio::stop_audio,
            commands::audio::get_audio_devices,
            commands::audio::is_audio_running,
            commands::audio::get_audio_config,
            commands::audio::set_audio_config,
            commands::scanner::scan_plugins,
            commands::scanner::reveal_plugin,
            commands::chain::add_to_chain,
            commands::chain::remove_from_chain,
            commands::chain::move_plugin,
            commands::chain::reorder_chain,
            commands::chain::bypass_plugin,
            commands::chain::get_chain,
            commands::params::get_plugin_parameters,
            commands::params::set_plugin_parameter,
            commands::window::open_plugin_gui,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app, event| {
            if let tauri::RunEvent::Exit = event {
                app.state::<AudioEngine>().persist();
            }
        });
}
