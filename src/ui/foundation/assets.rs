use gpui::{AssetSource, SharedString};
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "assets"]
#[include = "icons/**/*"]
pub struct EmbeddedAssets;

impl AssetSource for EmbeddedAssets {
    fn load(&self, path: &str) -> gpui::Result<Option<std::borrow::Cow<'static, [u8]>>> {
        Ok(Self::get(path).map(|asset| asset.data))
    }

    fn list(&self, path: &str) -> gpui::Result<Vec<SharedString>> {
        Ok(Self::iter()
            .filter(|asset| asset.starts_with(path))
            .map(SharedString::from)
            .collect())
    }
}

pub fn resolve_asset_path(rel_path: &str) -> String {
    rel_path
        .strip_prefix("assets/")
        .unwrap_or(rel_path)
        .to_owned()
}

#[cfg(test)]
mod tests {
    use super::{EmbeddedAssets, resolve_asset_path};
    use gpui::AssetSource as _;

    #[test]
    fn runtime_assets_are_embedded() {
        let path = resolve_asset_path("assets/icons/audio-waveform.svg");
        assert!(
            EmbeddedAssets
                .load(&path)
                .expect("asset lookup works")
                .is_some()
        );
    }
}
