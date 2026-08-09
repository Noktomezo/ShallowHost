use serde::{Deserialize, Serialize};

use super::{Engine, EngineError, ScannedPlugin, ffi, parse_json};
use crate::infrastructure::config::PluginSettings;

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(crate) struct PluginScanStep {
    pub done: bool,
    pub progress: f32,
    #[serde(default)]
    pub current: String,
    pub plugins: Vec<ScannedPlugin>,
}

impl Engine {
    pub fn scan_plugins(
        &self,
        settings: &PluginSettings,
    ) -> Result<Vec<ScannedPlugin>, EngineError> {
        let paths = plugin_paths_json(settings)?;
        let _guard = self.lock()?;
        let plugins: Vec<ScannedPlugin> = parse_json(&ffi::scan_plugins(&paths)?)?;
        drop(_guard);
        let mut cache = self.plugins.lock().map_err(|_| EngineError::LockPoisoned)?;
        *cache = plugins.clone();
        Ok(plugins)
    }

    pub fn start_plugin_scan(
        &self,
        settings: &PluginSettings,
    ) -> Result<PluginScanStep, EngineError> {
        let paths = plugin_paths_json(settings)?;
        let _guard = self.lock()?;
        let step = parse_json(&ffi::start_plugin_scan(&paths)?)?;
        drop(_guard);
        self.cache_plugin_scan_step(step)
    }

    pub fn scan_next_plugin(&self) -> Result<PluginScanStep, EngineError> {
        let _guard = self.lock()?;
        let step = parse_json(&ffi::scan_next_plugin()?)?;
        drop(_guard);
        self.cache_plugin_scan_step(step)
    }

    fn cache_plugin_scan_step(&self, step: PluginScanStep) -> Result<PluginScanStep, EngineError> {
        let mut cache = self.plugins.lock().map_err(|_| EngineError::LockPoisoned)?;
        cache.clone_from(&step.plugins);
        Ok(step)
    }
}

fn plugin_paths_json(settings: &PluginSettings) -> Result<String, EngineError> {
    #[derive(Serialize)]
    struct ScanPaths<'a> {
        vst2: &'a [String],
        vst3: &'a [String],
    }

    serde_json::to_string(&ScanPaths {
        vst2: &settings.vst2_paths,
        vst3: &settings.vst3_paths,
    })
    .map_err(EngineError::from)
}

#[cfg(test)]
mod tests {
    use super::PluginScanStep;
    use crate::infrastructure::engine::parse_json;

    #[test]
    fn parses_incremental_plugin_scan_step() {
        let step: PluginScanStep =
            parse_json(r#"{"done":false,"progress":0.25,"current":"Plugin.vst3","plugins":[]}"#)
                .expect("fixture is valid incremental scan JSON");

        assert!(!step.done);
        assert_eq!(step.progress, 0.25);
        assert_eq!(step.current, "Plugin.vst3");
        assert!(step.plugins.is_empty());
    }
}
