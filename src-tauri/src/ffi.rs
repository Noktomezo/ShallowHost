use std::ffi::{CStr, CString};
use std::os::raw::c_char;

extern "C" {
    fn sh_init();
    fn sh_shutdown();
    fn sh_set_data_dir(path: *const c_char);
    fn sh_audio_start(
        driver: *const c_char,
        input: *const c_char,
        output: *const c_char,
        sample_rate: i32,
        buffer_size: i32,
        mono: bool,
        input_mask: i32,
        output_mask: i32,
    ) -> bool;
    fn sh_audio_stop() -> bool;

    fn sh_get_audio_devices(driver: *const c_char, device_name: *const c_char) -> *mut c_char;
    fn sh_scan_plugins(vst2_paths: *const c_char, vst3_paths: *const c_char) -> *mut c_char;

    fn sh_add_to_chain(unique_id: *const c_char) -> *mut c_char;
    fn sh_remove_from_chain(node_id: *const c_char) -> bool;
    fn sh_move_plugin(node_id: *const c_char, up: bool) -> bool;
    fn sh_reorder_chain(node_id: *const c_char, to_index: i32) -> bool;
    fn sh_bypass_plugin(node_id: *const c_char, bypassed: bool) -> bool;
    fn sh_get_chain() -> *mut c_char;

    fn sh_get_plugin_parameters(node_id: *const c_char) -> *mut c_char;
    fn sh_set_plugin_parameter(node_id: *const c_char, param_index: i32, value: f32) -> bool;

    fn sh_open_plugin_gui(node_id: *const c_char) -> bool;
    fn sh_close_plugin_gui(node_id: *const c_char) -> bool;

    fn sh_save_state() -> *mut c_char;
    fn sh_load_state(state: *const c_char) -> bool;

    fn sh_free_string(ptr: *mut c_char);
}

use std::sync::{Mutex, OnceLock};

static FFI_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

fn get_lock() -> std::sync::MutexGuard<'static, ()> {
    FFI_LOCK.get_or_init(|| Mutex::new(())).lock().unwrap()
}

unsafe fn to_rust_string(ptr: *mut c_char) -> String {
    if ptr.is_null() {
        return String::new();
    }
    let s = CStr::from_ptr(ptr).to_string_lossy().into_owned();
    sh_free_string(ptr);
    s
}

fn to_c_string(s: &str) -> CString {
    CString::new(s).unwrap_or_else(|_| CString::new("").unwrap())
}

pub fn init() {
    let _lock = get_lock();
    unsafe {
        sh_init();
    }
}

pub fn shutdown() {
    let _lock = get_lock();
    unsafe {
        sh_shutdown();
    }
}

pub fn set_data_dir(path: &str) {
    let _lock = get_lock();
    let path_c = to_c_string(path);
    unsafe {
        sh_set_data_dir(path_c.as_ptr());
    }
}

#[allow(clippy::too_many_arguments)]
pub fn audio_start(
    driver: Option<&str>,
    input: Option<&str>,
    output: Option<&str>,
    sample_rate: i32,
    buffer_size: i32,
    mono: bool,
    input_mask: i32,
    output_mask: i32,
) -> bool {
    let _lock = get_lock();
    let driver_c = to_c_string(driver.unwrap_or(""));
    let input_c = to_c_string(input.unwrap_or(""));
    let output_c = to_c_string(output.unwrap_or(""));
    unsafe {
        sh_audio_start(
            driver_c.as_ptr(),
            input_c.as_ptr(),
            output_c.as_ptr(),
            sample_rate,
            buffer_size,
            mono,
            input_mask,
            output_mask,
        )
    }
}

pub fn audio_stop() -> bool {
    let _lock = get_lock();
    unsafe { sh_audio_stop() }
}

pub fn get_audio_devices(driver: &str, device: &str) -> String {
    let _lock = get_lock();
    let driver_c = to_c_string(driver);
    let device_c = to_c_string(device);
    unsafe { to_rust_string(sh_get_audio_devices(driver_c.as_ptr(), device_c.as_ptr())) }
}

pub fn scan_plugins(vst2_paths_json: &str, vst3_paths_json: &str) -> String {
    let _lock = get_lock();
    let v2_c = to_c_string(vst2_paths_json);
    let v3_c = to_c_string(vst3_paths_json);
    unsafe { to_rust_string(sh_scan_plugins(v2_c.as_ptr(), v3_c.as_ptr())) }
}

pub fn add_to_chain(unique_id: &str) -> String {
    let _lock = get_lock();
    let id_c = to_c_string(unique_id);
    unsafe { to_rust_string(sh_add_to_chain(id_c.as_ptr())) }
}

pub fn remove_from_chain(node_id: &str) -> bool {
    let _lock = get_lock();
    let id_c = to_c_string(node_id);
    unsafe { sh_remove_from_chain(id_c.as_ptr()) }
}

pub fn move_plugin(node_id: &str, up: bool) -> bool {
    let _lock = get_lock();
    let id_c = to_c_string(node_id);
    unsafe { sh_move_plugin(id_c.as_ptr(), up) }
}

pub fn reorder_chain(node_id: &str, to_index: i32) -> bool {
    let _lock = get_lock();
    let id_c = to_c_string(node_id);
    unsafe { sh_reorder_chain(id_c.as_ptr(), to_index) }
}

pub fn bypass_plugin(node_id: &str, bypassed: bool) -> bool {
    let _lock = get_lock();
    let id_c = to_c_string(node_id);
    unsafe { sh_bypass_plugin(id_c.as_ptr(), bypassed) }
}

pub fn get_chain() -> String {
    let _lock = get_lock();
    unsafe { to_rust_string(sh_get_chain()) }
}

pub fn get_plugin_parameters(node_id: &str) -> String {
    let _lock = get_lock();
    let id_c = to_c_string(node_id);
    unsafe { to_rust_string(sh_get_plugin_parameters(id_c.as_ptr())) }
}

pub fn set_plugin_parameter(node_id: &str, param_index: i32, value: f32) -> bool {
    let _lock = get_lock();
    let id_c = to_c_string(node_id);
    unsafe { sh_set_plugin_parameter(id_c.as_ptr(), param_index, value) }
}

pub fn open_plugin_gui(node_id: &str) -> bool {
    let _lock = get_lock();
    let id_c = to_c_string(node_id);
    unsafe { sh_open_plugin_gui(id_c.as_ptr()) }
}

pub fn close_plugin_gui(node_id: &str) -> bool {
    let _lock = get_lock();
    let id_c = to_c_string(node_id);
    unsafe { sh_close_plugin_gui(id_c.as_ptr()) }
}

pub fn save_state() -> String {
    let _lock = get_lock();
    unsafe { to_rust_string(sh_save_state()) }
}

pub fn load_state(state: &str) -> bool {
    let _lock = get_lock();
    let state_c = to_c_string(state);
    unsafe { sh_load_state(state_c.as_ptr()) }
}
