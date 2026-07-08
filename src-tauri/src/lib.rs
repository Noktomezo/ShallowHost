mod commands;
mod engine;
mod ffi;

use commands::system::CloseToTray;
use engine::audio_io::AudioEngine;
use std::sync::Mutex;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Emitter, Manager,
};

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

// ponytail: debug keeps devtools + reload for dev; release kills everything.
#[cfg(debug_assertions)]
fn prevent_default() -> tauri::plugin::TauriPlugin<tauri::Wry> {
    use tauri_plugin_prevent_default::Flags;
    tauri_plugin_prevent_default::Builder::new()
        .with_flags(Flags::all().difference(Flags::DEV_TOOLS | Flags::RELOAD))
        .build()
}

#[cfg(not(debug_assertions))]
fn prevent_default() -> tauri::plugin::TauriPlugin<tauri::Wry> {
    tauri_plugin_prevent_default::init()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.unminimize();
                let _ = window.show();
                let _ = window.set_focus();
            }
        }))
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec!["--autostart"]),
        ))
        .plugin(prevent_default())
        .manage(AudioEngine::new())
        .manage(CloseToTray(Mutex::new(true)))
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

            // System tray: Show/Quit menu, left-click shows window.
            let show_item = MenuItem::with_id(app, "show", "Show ShallowHost", true, None::<&str>)?;
            let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let tray_menu = Menu::with_items(app, &[&show_item, &quit_item])?;
            TrayIconBuilder::with_id("main-tray")
                .icon(app.default_window_icon().cloned().unwrap())
                .menu(&tray_menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id().as_ref() {
                    "show" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.unminimize();
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => (),
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.unminimize();
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                })
                .build(app)?;

            // ponytail: hide window on autostart launch; frontend shows it back
            // if autostartToTray is off (Rust can't read zustand state at setup).
            if std::env::args().any(|a| a == "--autostart") {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.hide();
                }
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
            commands::system::is_autostart_launch,
            commands::system::set_close_to_tray,
        ])
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                let state = window.app_handle().state::<CloseToTray>();
                if *state.0.lock().unwrap() {
                    api.prevent_close();
                    let _ = window.hide();
                }
            }
        })
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
