use gpui::{App, AppContext as _, Entity};
use gpui_updater::{EngineConfig, GitHubSource, UpdateStatus, Updater, Verification, Version};

const RELEASE_OWNER: &str = "Noktomezo";
const RELEASE_REPOSITORY: &str = "ShallowHost";
const CHECKSUMS_ASSET: &str = "SHA256SUMS";
const MINISIGN_PUBLIC_KEY: &str = "RWSeWrBbDqi6SGEfcTvdy+8CgdwKGxVK30mNPRJC953JSPStzZYl2RbU";

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
        let source = GitHubSource::new(RELEASE_OWNER, RELEASE_REPOSITORY)
            .asset_patterns(release_asset_patterns())
            .with_checksums(CHECKSUMS_ASSET)
            .with_minisig();
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
    use super::release_asset_patterns;

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
}
