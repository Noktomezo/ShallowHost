use std::path::PathBuf;

pub fn resolve_asset_path(rel_path: &str) -> String {
    // 1. Check relative to executable directory (release binary execution)
    if let Some(parent) = std::env::current_exe()
        .ok()
        .and_then(|e| e.parent().map(|p| p.to_path_buf()))
    {
        let p = parent.join(rel_path);
        if p.exists() {
            return p.to_string_lossy().to_string();
        }
    }

    // 2. Check relative to current working directory (dev execution)
    if let Ok(cwd) = std::env::current_dir() {
        let p = cwd.join(rel_path);
        if p.exists() {
            return p.to_string_lossy().to_string();
        }
    }

    // 3. Fallback to manifest directory if building / testing in cargo workspace
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let manifest_path = manifest_dir.join(rel_path);
    if manifest_path.exists() {
        return manifest_path.to_string_lossy().to_string();
    }

    rel_path.to_string()
}
