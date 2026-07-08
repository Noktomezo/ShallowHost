mod commands;
mod engine;
mod ffi;

use engine::audio_io::AudioEngine;
use tauri::{Emitter, Manager};

pub static APP_HANDLE: std::sync::OnceLock<tauri::AppHandle> = std::sync::OnceLock::new();

pub fn run_on_main_thread<F, R>(f: F) -> Result<R, String>
where
    F: FnOnce() -> R + Send + 'static,
    R: Send + 'static,
{
    let app = APP_HANDLE.get().ok_or("AppHandle not initialized")?;
    let (tx, rx) = std::sync::mpsc::channel();
    app.run_on_main_thread(move || {
        let res = f();
        let _ = tx.send(res);
    })
    .map_err(|e| e.to_string())?;
    rx.recv().map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_opener::init())
        .manage(AudioEngine::new())
        .setup(|app| {
            APP_HANDLE.set(app.handle().clone()).unwrap();

            // Initialize C++ host (JUCE MessageManager)
            ffi::init();

            let engine = app.state::<AudioEngine>();
            if let Ok(data_dir) = app.path().app_data_dir() {
                ffi::set_data_dir(&data_dir.to_string_lossy());
                engine.set_data_dir(data_dir);
                engine.load_audio_config();
            }

            // Start audio immediately so the stream starts and sets current_config
            if let Err(e) = engine.start() {
                eprintln!("[shallow-host] audio failed to start on launch: {e}");
            }

            if app.path().app_data_dir().is_ok() {
                let handle = app.handle().clone();
                tauri::async_runtime::spawn(async move {
                    let engine = handle.state::<AudioEngine>();
                    engine.restore_from_disk();
                });
            }

            // ponytail: hotplug polling — every 500ms enumerate devices on a
            // background thread (FFI lock serializes; UI never blocks). On
            // disconnect, fallback config to __none + restart. Emits events
            // so the frontend refreshes dropdowns and syncs config.
            let poll_handle = app.handle().clone();
            std::thread::spawn(move || {
                let mut last_snapshot: Option<(Vec<String>, Vec<String>)> = None;
                loop {
                    std::thread::sleep(std::time::Duration::from_millis(500));
                    let engine = poll_handle.state::<AudioEngine>();
                    if let Ok((devices, config_changed)) = engine.poll_and_recover() {
                        let snap = (
                            devices
                                .inputs
                                .iter()
                                .map(|d| d.name.clone())
                                .collect::<Vec<_>>(),
                            devices
                                .outputs
                                .iter()
                                .map(|d| d.name.clone())
                                .collect::<Vec<_>>(),
                        );
                        let devices_changed = last_snapshot.as_ref() != Some(&snap);
                        if devices_changed || config_changed {
                            let _ = poll_handle.emit("audio-devices-changed", devices);
                        }
                        if config_changed {
                            let _ = poll_handle.emit("audio-config-changed", ());
                        }
                        last_snapshot = Some(snap);
                    }
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::audio::start_audio,
            commands::audio::stop_audio,
            commands::audio::get_audio_devices,
            commands::audio::is_audio_running,
            commands::audio::get_audio_config,
            commands::audio::set_audio_config,
            commands::audio::restart_audio,
            commands::scanner::scan_plugins,
            commands::scanner::reveal_plugin,
            commands::scanner::select_directory,
            commands::chain::add_to_chain,
            commands::chain::remove_from_chain,
            commands::chain::move_plugin,
            commands::chain::reorder_chain,
            commands::chain::bypass_plugin,
            commands::chain::get_chain,
            commands::params::get_plugin_parameters,
            commands::params::set_plugin_parameter,
            commands::window::open_plugin_gui,
            commands::window::close_plugin_gui,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app, event| {
            if let tauri::RunEvent::Exit = event {
                app.state::<AudioEngine>().persist();
                ffi::shutdown();
            }
            if let tauri::RunEvent::ExitRequested { .. } = event {
                // ponytail: force exit short-circuits before RunEvent::Exit runs,
                // so persist + shutdown here — otherwise plugin param tweaks are lost
                // (only chain mutations call persist() mid-session).
                app.state::<AudioEngine>().persist();
                ffi::shutdown();
                std::process::exit(0);
            }
        });
}
