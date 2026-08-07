use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::time::Duration;

use gpui::{App, AppContext as _, Entity};
use gpui_updater::{
    EngineConfig, StaticManifestSource, UpdateStatus, Updater, Verification, Version,
};

const RELEASE_OWNER: &str = "Noktomezo";
const RELEASE_REPOSITORY: &str = "ShallowHost";
const MINISIGN_PUBLIC_KEY: &str = "RWSeWrBbDqi6SGEfcTvdy+8CgdwKGxVK30mNPRJC953JSPStzZYl2RbU";
const MOCK_UPDATE_ENV: &str = "SHALLOWHOST_MOCK_UPDATE";
const MOCK_CHECK_DURATION: Duration = Duration::from_secs(2);
static MOCK_CHECKING: AtomicBool = AtomicBool::new(false);
const MOCK_DOWNLOAD_IDLE: u8 = u8::MAX;
const MOCK_DOWNLOAD_STEPS: u8 = 40;
static MOCK_DOWNLOAD_PROGRESS: AtomicU8 = AtomicU8::new(MOCK_DOWNLOAD_IDLE);

#[must_use]
pub fn is_mock_preview() -> bool {
    cfg!(debug_assertions) && std::env::var_os(MOCK_UPDATE_ENV).is_some()
}

#[must_use]
pub fn mock_status() -> Option<UpdateStatus> {
    is_mock_preview().then(|| {
        if MOCK_CHECKING.load(Ordering::Acquire) {
            UpdateStatus::Checking
        } else if let progress = MOCK_DOWNLOAD_PROGRESS.load(Ordering::Acquire)
            && progress != MOCK_DOWNLOAD_IDLE
        {
            UpdateStatus::Downloading {
                downloaded: u64::from(progress),
                total: Some(100),
            }
        } else {
            UpdateStatus::Available(Version::new(99, 0, 0))
        }
    })
}

fn release_manifest_url() -> String {
    format!(
        "https://github.com/{RELEASE_OWNER}/{RELEASE_REPOSITORY}/releases/latest/download/latest.json"
    )
}

fn release_asset_patterns() -> Vec<String> {
    let architecture = match std::env::consts::ARCH {
        "aarch64" => "arm64",
        architecture => architecture,
    };
    vec![
        "windows".to_owned(),
        architecture.to_owned(),
        ".msi".to_owned(),
    ]
}

pub fn new_entity(cx: &mut App) -> Entity<Updater> {
    cx.new(|cx| {
        // The static manifest avoids GitHub REST's shared unauthenticated IP limit, which is
        // especially easy to exhaust through consumer VPNs such as WARP.
        let source = StaticManifestSource::new(release_manifest_url())
            .asset_patterns(release_asset_patterns());
        // Cargo validates the package version as SemVer before compiling the crate.
        let version = Version::parse(env!("CARGO_PKG_VERSION"))
            .expect("Cargo package version must be valid SemVer");
        let config = EngineConfig::new(version)
            .verification(Verification::Strict)
            .minisign_public_key(MINISIGN_PUBLIC_KEY);
        Updater::new(source, config, cx)
    })
}

pub fn start_check(updater: &Entity<Updater>, cx: &mut App) {
    if is_mock_preview() {
        if MOCK_CHECKING.swap(true, Ordering::AcqRel) {
            return;
        }
        cx.refresh_windows();
        cx.spawn(async move |cx| {
            cx.background_executor().timer(MOCK_CHECK_DURATION).await;
            MOCK_CHECKING.store(false, Ordering::Release);
            cx.update(App::refresh_windows);
        })
        .detach();
        return;
    }
    updater.update(cx, Updater::check);
}

pub fn download_and_install(updater: &Entity<Updater>, cx: &mut App) {
    if is_mock_preview() {
        start_mock_download(cx);
        return;
    }
    updater.update(cx, Updater::download_and_install);
}

pub fn restart(updater: &Entity<Updater>, cx: &mut App) {
    updater.update(cx, |updater, cx| updater.restart(cx));
}

fn start_mock_download(cx: &mut App) {
    if MOCK_DOWNLOAD_PROGRESS
        .compare_exchange(MOCK_DOWNLOAD_IDLE, 0, Ordering::AcqRel, Ordering::Acquire)
        .is_err()
    {
        return;
    }

    cx.refresh_windows();
    cx.spawn(async move |cx| {
        for step in 1..=MOCK_DOWNLOAD_STEPS {
            cx.background_executor()
                .timer(Duration::from_millis(50))
                .await;
            MOCK_DOWNLOAD_PROGRESS.store(mock_download_percent(step), Ordering::Release);
            cx.update(App::refresh_windows);
        }
        MOCK_DOWNLOAD_PROGRESS.store(MOCK_DOWNLOAD_IDLE, Ordering::Release);
        cx.update(App::refresh_windows);
    })
    .detach();
}

fn mock_download_percent(step: u8) -> u8 {
    let percent = u16::from(step).saturating_mul(100) / u16::from(MOCK_DOWNLOAD_STEPS);
    u8::try_from(percent.min(100)).unwrap_or(100)
}

#[cfg(test)]
mod tests {
    use super::{mock_download_percent, release_asset_patterns, release_manifest_url};

    #[test]
    fn mock_download_progress_spans_the_full_percentage_range() {
        assert_eq!(mock_download_percent(0), 0);
        assert_eq!(mock_download_percent(20), 50);
        assert_eq!(mock_download_percent(40), 100);
    }

    #[test]
    fn updater_selects_a_windows_msi_for_the_current_architecture() {
        let patterns = release_asset_patterns();

        assert!(patterns.iter().any(|pattern| pattern == "windows"));
        assert!(patterns.iter().any(|pattern| pattern == ".msi"));
        assert!(patterns.iter().any(|pattern| {
            pattern == std::env::consts::ARCH
                || (std::env::consts::ARCH == "aarch64" && pattern == "arm64")
        }));
    }

    #[test]
    fn updater_manifest_uses_the_public_latest_release_redirect() {
        assert_eq!(
            release_manifest_url(),
            "https://github.com/Noktomezo/ShallowHost/releases/latest/download/latest.json"
        );
    }
}
