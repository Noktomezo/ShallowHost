use std::sync::Mutex;

pub struct CloseToTray(pub Mutex<bool>);

#[tauri::command]
pub fn is_autostart_launch() -> bool {
    std::env::args().any(|a| a == "--autostart")
}

#[tauri::command]
pub fn set_close_to_tray(state: tauri::State<CloseToTray>, enabled: bool) {
    *state.0.lock().unwrap() = enabled;
}
