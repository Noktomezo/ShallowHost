//! Windows install strategies: the bare-`.exe` rename trick, or a staged MSI
//! handoff.
//!
//! **Bare exe** (artifact ends `.exe`): Windows refuses to *delete* a running
//! executable but allows *renaming* it. We move the current exe aside to
//! `*.old.exe`, drop the freshly downloaded exe into place, and leave the
//! stale copy to be cleaned up on next launch.
//!
//! **MSI** (artifact ends `.msi`): running msiexec from inside the app would
//! race ourselves — a well-authored per-user package closes the running app
//! (`WiX`'s `util:CloseApplication`), i.e. the very process driving this install,
//! killing the updater mid-`Installing`. So the MSI is *staged* instead: we
//! write a small `.vbs` beside the verified artifact that installs the package
//! (`/passive /norestart` — progress UI, no surprise reboot) and relaunches
//! the app, and return that script as the restart path. GPUI's Windows
//! `restart` waits for this process to exit before spawning the restart path,
//! which is exactly the ordering msiexec needs: by the time the script runs,
//! the app's files are no longer in use (a still-running background helper is
//! the package's `CloseApplication` problem, as on a manual install).
//!
//! Dispatch is on the **artifact's** extension, not the install root's: the
//! artifact is what we're about to execute or copy, so it must prove its own
//! shape. (Dispatching on the root alone would let a mismatched payload — say
//! an `.msi` served under an `exe` manifest format — be copied over the
//! running binary byte-for-byte.)

use std::fs;
use std::path::Path;

use super::Installed;
use crate::error::{Error, Result};

pub(crate) fn install(artifact: &Path, install_root: &Path) -> Result<Installed> {
    match artifact
        .extension()
        .and_then(|e| e.to_str())
        .map(str::to_ascii_lowercase)
        .as_deref()
    {
        Some("exe") => install_bare_exe(artifact, install_root),
        Some("msi") => stage_msi(artifact, install_root),
        _ => Err(Error::UnsupportedPlatform(
            "windows update artifact (expected .exe or .msi)",
        )),
    }
}

/// Replace the running `.exe` in place via the rename trick.
fn install_bare_exe(new_exe: &Path, install_root: &Path) -> Result<Installed> {
    let root_is_exe = install_root
        .extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| e.eq_ignore_ascii_case("exe"));
    if !root_is_exe {
        return Err(Error::Install(format!(
            "bare-exe update needs an .exe install root, got {}",
            install_root.display()
        )));
    }

    let old = install_root.with_extension("old.exe");
    let _ = fs::remove_file(&old);
    if install_root.exists() {
        fs::rename(install_root, &old)
            .map_err(|e| Error::Install(format!("could not rename running exe: {e}")))?;
    }
    if let Err(e) = fs::copy(new_exe, install_root) {
        // Roll back so we never leave the app without its executable.
        if old.exists() {
            let _ = fs::rename(&old, install_root);
        }
        return Err(Error::Install(format!("could not place new exe: {e}")));
    }

    Ok(Installed {
        restart_path: Some(install_root.to_path_buf()),
    })
}

/// Stage a verified `.msi`: write the apply script next to it and hand that
/// back as the restart path. Nothing is installed until the app exits.
fn stage_msi(msi: &Path, install_root: &Path) -> Result<Installed> {
    // The relaunch target is the current exe path: an upgrade MSI installs to
    // the same per-user folder, so the path stays valid and holds the new
    // binary once msiexec finishes.
    let script_path = msi.with_extension("apply.vbs");
    fs::write(&script_path, apply_script(msi, install_root)?)
        .map_err(|e| Error::Install(format!("could not stage MSI apply script: {e}")))?;
    Ok(Installed {
        restart_path: Some(script_path),
    })
}

/// The staged apply script: install the MSI, then relaunch the app — but only
/// on success, so a failed upgrade leaves the old (still installed) version
/// for the user to start rather than masking the failure with a relaunch.
fn apply_script(msi: &Path, relaunch: &Path) -> Result<Vec<u8>> {
    let msi = vbs_string(msi)?;
    let relaunch = vbs_string(relaunch)?;
    let script = format!(
        "Option Explicit\r\n\
         Dim shell, exitCode, installerPath, relaunchPath\r\n\
         installerPath = {msi}\r\n\
         relaunchPath = {relaunch}\r\n\
         Set shell = CreateObject(\"WScript.Shell\")\r\n\
         exitCode = shell.Run(\"msiexec.exe /i \"\"\" & installerPath & \"\"\" /passive /norestart\", 1, True)\r\n\
         If exitCode = 0 Then\r\n\
           shell.Run \"\"\"\" & relaunchPath & \"\"\"\", 1, False\r\n\
         End If\r\n\
         WScript.Quit exitCode\r\n"
    );
    Ok(utf16_le_with_bom(&script))
}

fn vbs_string(path: &Path) -> Result<String> {
    let s = path
        .to_str()
        .ok_or_else(|| Error::Install(format!("non-UTF-8 path: {}", path.display())))?;
    if s.contains('\r') || s.contains('\n') {
        return Err(Error::Install(format!(
            "path cannot be represented in VBScript: {s}"
        )));
    }
    Ok(format!("\"{}\"", s.replace('"', "\"\"")))
}

fn utf16_le_with_bom(value: &str) -> Vec<u8> {
    let mut encoded = Vec::with_capacity(2 + value.len() * 2);
    encoded.extend_from_slice(&[0xff, 0xfe]);
    encoded.extend(value.encode_utf16().flat_map(u16::to_le_bytes));
    encoded
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn dispatch_rejects_unknown_artifact_kinds() {
        // Neither .exe nor .msi: never copied, never staged.
        let err = install(Path::new("C:\\dl\\app.zip"), Path::new("C:\\app\\app.exe")).unwrap_err();
        assert!(matches!(err, Error::UnsupportedPlatform(_)));
    }

    #[test]
    fn bare_exe_requires_an_exe_install_root() {
        // An .exe payload aimed at a non-exe root (e.g. a directory) is a
        // config error, not something to clobber.
        let err = install(Path::new("C:\\dl\\app.exe"), Path::new("C:\\app")).unwrap_err();
        assert!(matches!(err, Error::Install(_)));
    }

    #[test]
    fn exe_rename_trick_swaps_and_keeps_the_old_copy() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("app.exe");
        let new = dir.path().join("download.exe");
        fs::write(&root, b"old").unwrap();
        fs::write(&new, b"new").unwrap();

        let installed = install(&new, &root).unwrap();

        assert_eq!(installed.restart_path.as_deref(), Some(root.as_path()));
        assert_eq!(fs::read(&root).unwrap(), b"new");
        assert_eq!(fs::read(root.with_extension("old.exe")).unwrap(), b"old");
    }

    #[test]
    fn msi_stages_an_apply_script_as_the_restart_path() {
        let dir = tempfile::tempdir().unwrap();
        let msi = dir.path().join("App-v2.msi");
        fs::write(&msi, b"msi").unwrap();
        let root = dir.path().join("App.exe");

        let installed = install(&msi, &root).unwrap();

        let script_path = installed.restart_path.unwrap();
        assert_eq!(script_path, msi.with_extension("apply.vbs"));
        let script = decode_utf16_script(&fs::read(script_path).unwrap());
        assert!(script.contains("msiexec.exe /i"));
        assert!(script.contains(&format!("installerPath = \"{}\"", msi.display())));
        assert!(script.contains(&format!("relaunchPath = \"{}\"", root.display())));
        // Relaunch is gated on msiexec succeeding.
        assert!(script.contains("If exitCode = 0 Then"));
        // The MSI itself must not have been touched, let alone executed.
        assert_eq!(fs::read(&msi).unwrap(), b"msi");
    }

    #[test]
    fn script_is_utf16_and_preserves_unicode_paths() {
        let script = apply_script(
            Path::new("C:\\Пользователь\\обновление.msi"),
            Path::new("C:\\Программы\\ShallowHost.exe"),
        )
        .unwrap();

        assert_eq!(&script[..2], &[0xff, 0xfe]);
        let decoded = decode_utf16_script(&script);
        assert!(decoded.contains("Пользователь"));
        assert!(decoded.contains("Программы"));
    }

    fn decode_utf16_script(bytes: &[u8]) -> String {
        assert_eq!(&bytes[..2], &[0xff, 0xfe]);
        let units = bytes[2..]
            .chunks_exact(2)
            .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
            .collect::<Vec<_>>();
        String::from_utf16(&units).unwrap()
    }
}
