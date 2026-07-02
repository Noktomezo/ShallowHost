use std::path::Path;

use serde::Serialize;
use truce_rack::core::scanner::PluginScanner;
use truce_rack::core::PluginInfo;
use truce_rack::vst3::{Vst3Plugin, Vst3Scanner};
use vst3::ComPtr;
use vst3::Steinberg::{IPluginFactory, IPluginFactoryTrait, PFactoryInfo};
use windows::core::HSTRING;
use windows::Win32::System::LibraryLoader::{LoadLibraryExW, LOAD_WITH_ALTERED_SEARCH_PATH};

#[derive(Debug, Clone, Serialize)]
pub struct ScannedPlugin {
    pub name: String,
    pub vendor: String,
    pub version: u32,
    pub category: String,
    pub path: String,
    pub unique_id: String,
    pub format: String,
    pub has_editor: bool,
    pub accepts_midi: bool,
}

pub fn scan_vst3_raw() -> Result<Vec<PluginInfo>, String> {
    let scanner = Vst3Scanner::new();

    // truce-rack's scan() is non-recursive (read_dir only). Walk subdirs
    // ourselves, collect unique parent dirs containing .vst3 entries, then
    // scan_path each — deduplicate results by unique_id.
    let parent_dirs = collect_vst3_parent_dirs(&truce_rack::vst3::default_vst3_paths());
    let mut seen = std::collections::HashSet::new();
    let mut infos: Vec<PluginInfo> = Vec::new();
    for dir in &parent_dirs {
        if let Ok(found) = scanner.scan_path(dir) {
            for info in found {
                if seen.insert(info.unique_id.clone()) {
                    infos.push(info);
                }
            }
        }
    }

    // ponytail: collect loaded libs to keep them alive until scan returns.
    // FreeLibrary crashes some VST3 plugins in DllMain PROCESS_DETACH.
    let mut _keep_alive: Vec<windows::Win32::Foundation::HMODULE> = Vec::new();
    for info in &mut infos {
        if info.vendor.is_empty() {
            info.vendor = extract_vendor(&info.path, &mut _keep_alive);
        }
    }
    Ok(infos)
}

/// Recursively walk `roots`, collect every directory that directly contains
/// at least one `.vst3` entry. Returns unique, sorted paths.
fn collect_vst3_parent_dirs(roots: &[std::path::PathBuf]) -> Vec<std::path::PathBuf> {
    let mut out: Vec<std::path::PathBuf> = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for root in roots {
        walk_for_vst3_parents(root, &mut out, &mut seen);
    }
    out.sort();
    out
}

fn walk_for_vst3_parents(
    dir: &Path,
    out: &mut Vec<std::path::PathBuf>,
    seen: &mut std::collections::HashSet<std::path::PathBuf>,
) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    let mut has_vst3 = false;
    let mut subdirs: Vec<std::path::PathBuf> = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        let is_vst3 = path
            .file_name()
            .and_then(|n| n.to_str())
            .is_some_and(|n| n.ends_with(".vst3"));
        if is_vst3 {
            has_vst3 = true;
        } else if path.is_dir() {
            subdirs.push(path);
        }
    }
    if has_vst3 && seen.insert(dir.to_path_buf()) {
        out.push(dir.to_path_buf());
    }
    for sub in subdirs {
        walk_for_vst3_parents(&sub, out, seen);
    }
}

pub fn load_vst3(info: &PluginInfo) -> Result<Vst3Plugin, String> {
    let scanner = Vst3Scanner::new();
    scanner.load(info).map_err(|e| e.to_string())
}

impl ScannedPlugin {
    pub fn from_info(info: &PluginInfo) -> Self {
        Self {
            name: info.name.clone(),
            vendor: info.vendor.clone(),
            version: info.version,
            category: format!("{:?}", info.category),
            path: info.path.display().to_string(),
            unique_id: info.unique_id.clone(),
            format: info.format.to_string(),
            has_editor: info.has_editor,
            accepts_midi: info.accepts_midi,
        }
    }
}

/// Extract vendor name from a VST3 bundle by calling IPluginFactory::getFactoryInfo.
/// Returns empty string on any failure (scanner already ran, this is best-effort enrichment).
fn extract_vendor(
    bundle_path: &Path,
    keep_alive: &mut Vec<windows::Win32::Foundation::HMODULE>,
) -> String {
    let Some(binary) = bundle_binary_path(bundle_path) else {
        return String::new();
    };

    // LOAD_WITH_ALTERED_SEARCH_PATH: search the DLL's own directory for dependencies.
    let path_wide = HSTRING::from(binary.as_os_str());
    let hmodule = unsafe { LoadLibraryExW(&path_wide, None, LOAD_WITH_ALTERED_SEARCH_PATH) };
    let Ok(hmodule) = hmodule else {
        eprintln!(
            "[vendor] LoadLibraryExW failed for {}: {}",
            binary.display(),
            std::io::Error::last_os_error()
        );
        return String::new();
    };

    let factory_ptr = unsafe {
        let sym = windows::Win32::System::LibraryLoader::GetProcAddress(
            hmodule,
            windows::core::PCSTR(c"GetPluginFactory".as_ptr() as *const u8),
        );
        match sym {
            Some(f) => {
                let func: unsafe extern "C" fn() -> *mut IPluginFactory = std::mem::transmute(f);
                func()
            }
            None => return String::new(),
        }
    };

    let Some(factory) = (unsafe { ComPtr::from_raw(factory_ptr) }) else {
        return String::new();
    };

    // Safety: PFactoryInfo is #[repr(C)] + Copy, zeroed is valid.
    let mut info: PFactoryInfo = unsafe { std::mem::zeroed() };
    let result = unsafe { factory.getFactoryInfo(&mut info) };
    if result != 0 {
        eprintln!("[vendor] getFactoryInfo failed for {}: result={}", binary.display(), result);
        return String::new();
    }

    let vendor = char8_array_to_string(&info.vendor);
    eprintln!("[vendor] {} → vendor='{}'", binary.display(), vendor);

    // Keep the module loaded — FreeLibrary crashes some plugins in DllMain.
    keep_alive.push(hmodule);

    vendor
}

/// Compute the actual binary path inside a VST3 bundle.
/// Bundle format: `Plugin.vst3/Contents/{arch}-win/Plugin.vst3`
/// Flat format: `Plugin.vst3` (single DLL file) — return as-is.
fn bundle_binary_path(bundle: &Path) -> Option<std::path::PathBuf> {
    if !bundle.is_dir() {
        return Some(bundle.to_path_buf());
    }
    let stem = bundle.file_stem()?.to_str()?;
    let arch = std::env::consts::ARCH;
    Some(
        bundle
            .join("Contents")
            .join(format!("{arch}-win"))
            .join(format!("{stem}.vst3")),
    )
}

/// Convert a null-terminated char8 (i8) array to a String.
fn char8_array_to_string(arr: &[i8]) -> String {
    let bytes: Vec<u8> = arr
        .iter()
        .take_while(|c| **c != 0)
        .map(|c| *c as u8)
        .collect();
    String::from_utf8_lossy(&bytes).into_owned()
}
