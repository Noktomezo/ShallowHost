use gpui::{App, AppContext as _, Entity};
use gpui_updater::{
    EngineConfig, StaticManifestSource, UpdateStatus, Updater, Verification, Version,
};

const RELEASE_OWNER: &str = "Noktomezo";
const RELEASE_REPOSITORY: &str = "ShallowHost";
const MINISIGN_PUBLIC_KEY: &str = "RWSeWrBbDqi6SGEfcTvdy+8CgdwKGxVK30mNPRJC953JSPStzZYl2RbU";
const MOCK_UPDATE_ENV: &str = "SHALLOWHOST_MOCK_UPDATE";

#[must_use]
pub fn is_mock_preview() -> bool {
    cfg!(debug_assertions) && std::env::var_os(MOCK_UPDATE_ENV).is_some()
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
        return;
    }
    updater.update(cx, Updater::check);
}

pub fn run_primary_action(updater: &Entity<Updater>, cx: &mut App) {
    updater.update(cx, |updater, cx| match updater.status() {
        UpdateStatus::Available(_) => updater.download_and_install(cx),
        UpdateStatus::Staged(_) => updater.restart(cx),
        status if !status.is_busy() => updater.check(cx),
        _ => {}
    });
}

#[cfg(test)]
mod tests {
    use super::{release_asset_patterns, release_manifest_url};

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
