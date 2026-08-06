use super::{AudioConfig, EngineError};
use std::ffi::{CStr, CString, c_char};
use std::ptr::NonNull;

unsafe extern "C" {
    fn sh_init();
    fn sh_shutdown();
    fn sh_set_data_dir(path: *const c_char);
    fn sh_audio_start(
        driver: *const c_char,
        input: *const c_char,
        output: *const c_char,
        sample_rate: i32,
        buffer_size: i32,
        input_mask: i32,
        output_mask: i32,
        is_mono: bool,
    ) -> bool;
    fn sh_audio_stop() -> bool;
    fn sh_get_audio_levels(input_peak: *mut f32, output_peak: *mut f32);
    fn sh_get_audio_devices(driver: *const c_char, device: *const c_char) -> *mut c_char;
    fn sh_scan_plugins(vst3_paths: *const c_char) -> *mut c_char;
    fn sh_add_to_chain(unique_id: *const c_char) -> *mut c_char;
    fn sh_clear_chain();
    fn sh_remove_from_chain(node_id: *const c_char) -> bool;
    fn sh_reorder_chain(node_id: *const c_char, to_index: i32) -> bool;
    fn sh_bypass_plugin(node_id: *const c_char, bypassed: bool) -> bool;
    fn sh_get_chain() -> *mut c_char;
    fn sh_save_state() -> *mut c_char;
    fn sh_load_state(state: *const c_char) -> bool;
    fn sh_get_plugin_parameters(node_id: *const c_char) -> *mut c_char;
    fn sh_open_plugin_gui(node_id: *const c_char, title_prefix: *const c_char) -> bool;
    fn sh_set_mono_mode(mono: bool);
    fn sh_free_string(pointer: *mut c_char);
}

struct NativeString(NonNull<c_char>);

impl NativeString {
    fn from_raw(pointer: *mut c_char, operation: &'static str) -> Result<Self, EngineError> {
        NonNull::new(pointer)
            .map(Self)
            .ok_or(EngineError::NativeFailure(operation))
    }

    fn to_string(&self) -> Result<String, EngineError> {
        // SAFETY: `self.0` comes from a JUCE export that returns an allocated,
        // NUL-terminated string and remains alive until this guard is dropped.
        let value = unsafe { CStr::from_ptr(self.0.as_ptr()) };
        value
            .to_str()
            .map(str::to_owned)
            .map_err(|_| EngineError::InvalidUtf8)
    }
}

impl Drop for NativeString {
    fn drop(&mut self) {
        // SAFETY: this pointer is uniquely owned by the guard and was allocated
        // by the matching C++ export; `sh_free_string` is its designated deleter.
        unsafe { sh_free_string(self.0.as_ptr()) };
    }
}

pub fn init() {
    // SAFETY: initializes the process-wide JUCE singleton; `Engine` guarantees
    // serialized access and pairs this call with exactly one shutdown.
    unsafe { sh_init() };
}

pub fn shutdown() {
    // SAFETY: called by `Engine::drop` after all safe calls through that owner end.
    unsafe { sh_shutdown() };
}

pub fn set_data_dir(path: &str) -> Result<(), EngineError> {
    let path = c_string(path)?;
    // SAFETY: the pointer is valid and NUL-terminated for the duration of the call.
    unsafe { sh_set_data_dir(path.as_ptr()) };
    Ok(())
}

pub fn audio_start(config: &AudioConfig<'_>) -> bool {
    let Ok(driver) = c_string(config.driver) else {
        return false;
    };
    let Ok(input) = c_string(config.input.unwrap_or_default()) else {
        return false;
    };
    let Ok(output) = c_string(config.output.unwrap_or_default()) else {
        return false;
    };
    // SAFETY: all string pointers are valid for this synchronous call and the
    // scalar arguments exactly match the C ABI declarations in `ffi.cpp`.
    unsafe {
        sh_audio_start(
            driver.as_ptr(),
            input.as_ptr(),
            output.as_ptr(),
            config.sample_rate,
            config.buffer_size,
            config.input_mask,
            config.output_mask,
            config.is_mono,
        )
    }
}

pub fn audio_stop() -> bool {
    // SAFETY: no arguments; access is serialized by `Engine`.
    unsafe { sh_audio_stop() }
}

pub fn audio_levels() -> (f32, f32) {
    let mut input = 0.0;
    let mut output = 0.0;
    // SAFETY: both pointers refer to initialized, writable `f32` values for the call.
    unsafe { sh_get_audio_levels(&mut input, &mut output) };
    (input, output)
}

pub fn audio_devices(driver: &str, device: &str) -> Result<String, EngineError> {
    let driver = c_string(driver)?;
    let device = c_string(device)?;
    // SAFETY: both pointers are valid NUL-terminated strings for the call.
    owned_string(
        unsafe { sh_get_audio_devices(driver.as_ptr(), device.as_ptr()) },
        "list audio devices",
    )
}

pub fn scan_plugins(vst3: &str) -> Result<String, EngineError> {
    let vst3 = c_string(vst3)?;
    // SAFETY: the pointer is a valid NUL-terminated JSON string for the call.
    owned_string(unsafe { sh_scan_plugins(vst3.as_ptr()) }, "scan plugins")
}

pub fn add_to_chain(unique_id: &str) -> Result<String, EngineError> {
    let unique_id = c_string(unique_id)?;
    // SAFETY: pointer is valid and NUL-terminated for the synchronous call.
    owned_string(
        unsafe { sh_add_to_chain(unique_id.as_ptr()) },
        "add plugin to chain",
    )
}

pub fn clear_chain() {
    // SAFETY: no arguments; access is serialized by `Engine`.
    unsafe { sh_clear_chain() };
}

pub fn remove_from_chain(node_id: &str) -> bool {
    let Ok(node_id) = c_string(node_id) else {
        return false;
    };
    // SAFETY: pointer is valid and NUL-terminated for the synchronous call.
    unsafe { sh_remove_from_chain(node_id.as_ptr()) }
}

pub fn reorder_chain(node_id: &str, to_index: i32) -> Result<bool, EngineError> {
    let node_id = c_string(node_id)?;
    // SAFETY: the pointer is valid and NUL-terminated for the synchronous call;
    // `to_index` has the same fixed-width representation as the C++ `int` export.
    Ok(unsafe { sh_reorder_chain(node_id.as_ptr(), to_index) })
}

pub fn bypass_plugin(node_id: &str, bypassed: bool) -> bool {
    let Ok(node_id) = c_string(node_id) else {
        return false;
    };
    // SAFETY: pointer and ABI-compatible bool are valid for the synchronous call.
    unsafe { sh_bypass_plugin(node_id.as_ptr(), bypassed) }
}

pub fn chain() -> Result<String, EngineError> {
    // SAFETY: no arguments; returned allocation is immediately wrapped for RAII cleanup.
    owned_string(unsafe { sh_get_chain() }, "read plugin chain")
}

pub fn save_state() -> Result<String, EngineError> {
    // SAFETY: no arguments; returned allocation is immediately wrapped for RAII cleanup.
    owned_string(unsafe { sh_save_state() }, "save plugin chain state")
}

pub fn load_state(state: &str) -> Result<bool, EngineError> {
    let state = c_string(state)?;
    // SAFETY: the pointer is a valid NUL-terminated JSON string for the synchronous call.
    Ok(unsafe { sh_load_state(state.as_ptr()) })
}

pub fn parameters(node_id: &str) -> Result<String, EngineError> {
    let node_id = c_string(node_id)?;
    // SAFETY: pointer is valid and NUL-terminated for the synchronous call.
    owned_string(
        unsafe { sh_get_plugin_parameters(node_id.as_ptr()) },
        "read plugin parameters",
    )
}

pub fn open_plugin_gui(node_id: &str, title: &str) -> bool {
    let Ok(node_id) = c_string(node_id) else {
        return false;
    };
    let Ok(title) = c_string(title) else {
        return false;
    };
    // SAFETY: both pointers are valid NUL-terminated strings for the call.
    unsafe { sh_open_plugin_gui(node_id.as_ptr(), title.as_ptr()) }
}

pub fn set_mono_mode(mono: bool) {
    // SAFETY: the C ABI accepts an ABI-compatible bool and access is serialized by `Engine`.
    unsafe { sh_set_mono_mode(mono) };
}

fn c_string(value: &str) -> Result<CString, EngineError> {
    CString::new(value).map_err(|_| EngineError::InteriorNul)
}

fn owned_string(pointer: *mut c_char, operation: &'static str) -> Result<String, EngineError> {
    NativeString::from_raw(pointer, operation)?.to_string()
}
