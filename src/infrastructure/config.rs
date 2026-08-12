use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

use crate::domain::preferences::{Language, ThemeMode};

const CURRENT_CONFIG_VERSION: u32 = 2;
const STEINBERG_VST2_PATHS: [&str; 4] = [
    r"C:\Program Files\Steinberg\VstPlugins",
    r"C:\Program Files (x86)\Steinberg\VstPlugins",
    r"C:\Program Files\Common Files\Steinberg\VST2",
    r"C:\Program Files (x86)\Common Files\Steinberg\VST2",
];

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct AppConfig {
    pub version: u32,
    pub theme: ThemeMode,
    pub language: Language,
    pub transparent_shell: bool,
    pub audio: AudioSettings,
    pub plugins: PluginSettings,
    pub system: SystemSettings,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            version: CURRENT_CONFIG_VERSION,
            theme: ThemeMode::System,
            language: Language::System,
            transparent_shell: true,
            audio: AudioSettings::default(),
            plugins: PluginSettings::default(),
            system: SystemSettings::default(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct AudioSettings {
    pub driver: String,
    pub input_device: Option<String>,
    pub output_device: Option<String>,
    pub devices_by_driver: BTreeMap<String, DriverDeviceSelection>,
    pub sample_rate: i32,
    pub buffer_size: i32,
    pub is_mono: bool,
    pub active_inputs: Vec<usize>,
    pub active_outputs: Vec<usize>,
}

impl Default for AudioSettings {
    fn default() -> Self {
        Self {
            driver: String::from("wasapi"),
            input_device: None,
            output_device: None,
            devices_by_driver: BTreeMap::new(),
            sample_rate: 48_000,
            buffer_size: 512,
            is_mono: false,
            active_inputs: vec![0, 1],
            active_outputs: vec![0, 1],
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct DriverDeviceSelection {
    pub input: Option<String>,
    pub output: Option<String>,
    pub active_inputs: Vec<usize>,
    pub active_outputs: Vec<usize>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct PluginSettings {
    pub vst2_paths: Vec<String>,
    pub vst3_paths: Vec<String>,
    pub group_by_author: bool,
}

impl Default for PluginSettings {
    fn default() -> Self {
        Self {
            vst2_paths: vec![
                String::from(r"C:\Program Files\VSTPlugins"),
                String::from(r"C:\Program Files\Common Files\VST2"),
                String::from(r"C:\Program Files (x86)\VSTPlugins"),
                String::from(STEINBERG_VST2_PATHS[0]),
                String::from(STEINBERG_VST2_PATHS[1]),
                String::from(STEINBERG_VST2_PATHS[2]),
                String::from(STEINBERG_VST2_PATHS[3]),
            ],
            vst3_paths: vec![
                String::from(r"C:\Program Files\Common Files\VST3"),
                String::from(r"C:\Program Files (x86)\Common Files\VST3"),
            ],
            group_by_author: false,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct SystemSettings {
    pub autostart: bool,
    pub autostart_to_tray: bool,
    pub minimize_to_tray: bool,
    pub auto_check_updates: bool,
}

impl Default for SystemSettings {
    fn default() -> Self {
        Self {
            autostart: false,
            autostart_to_tray: false,
            minimize_to_tray: false,
            auto_check_updates: true,
        }
    }
}

pub struct ConfigStore {
    path: PathBuf,
    cache_dir: PathBuf,
    config: AppConfig,
}

impl ConfigStore {
    pub fn beside_executable() -> Result<Self, ConfigError> {
        let executable = std::env::current_exe().map_err(ConfigError::CurrentExecutable)?;
        let directory = executable
            .parent()
            .ok_or(ConfigError::MissingExecutableDirectory)?;
        let path = directory.join("config.toml");
        let cache_dir = directory.join("cache");
        fs::create_dir_all(&cache_dir).map_err(|source| ConfigError::CreateCache {
            path: cache_dir.clone(),
            source,
        })?;

        let config_exists = path.exists();
        let mut config = if config_exists {
            let contents = fs::read_to_string(&path).map_err(|source| ConfigError::Read {
                path: path.clone(),
                source,
            })?;
            toml::from_str(&contents).map_err(|source| ConfigError::Parse {
                path: path.clone(),
                source,
            })?
        } else {
            AppConfig::default()
        };
        let config_migrated = migrate_config(&mut config);

        let store = Self {
            path,
            cache_dir,
            config,
        };
        store.migrate_legacy_cache()?;
        if !config_exists || config_migrated {
            store.save()?;
        }
        Ok(store)
    }

    pub fn config(&self) -> &AppConfig {
        &self.config
    }

    pub fn config_mut(&mut self) -> &mut AppConfig {
        &mut self.config
    }

    pub fn cache_dir(&self) -> &Path {
        &self.cache_dir
    }

    pub fn save(&self) -> Result<(), ConfigError> {
        let contents = toml::to_string_pretty(&self.config).map_err(ConfigError::Serialize)?;
        fs::write(&self.path, contents).map_err(|source| ConfigError::Write {
            path: self.path.clone(),
            source,
        })
    }

    fn migrate_legacy_cache(&self) -> Result<(), ConfigError> {
        let target = self.cache_dir.join("plugins.xml");
        if target.exists() {
            return Ok(());
        }
        let Some(local_app_data) = std::env::var_os("LOCALAPPDATA") else {
            return Ok(());
        };
        let source = PathBuf::from(local_app_data)
            .join("ShallowHost")
            .join("plugins.xml");
        if !source.exists() {
            return Ok(());
        }
        fs::copy(&source, &target).map_err(|error| ConfigError::MigrateCache {
            source,
            target,
            error,
        })?;
        Ok(())
    }
}

fn migrate_config(config: &mut AppConfig) -> bool {
    if config.version >= CURRENT_CONFIG_VERSION {
        return false;
    }

    for path in STEINBERG_VST2_PATHS {
        if !config
            .plugins
            .vst2_paths
            .iter()
            .any(|existing| existing.eq_ignore_ascii_case(path))
        {
            config.plugins.vst2_paths.push(path.into());
        }
    }
    config.version = CURRENT_CONFIG_VERSION;
    true
}

#[derive(Debug)]
pub enum ConfigError {
    CurrentExecutable(std::io::Error),
    MissingExecutableDirectory,
    CreateCache {
        path: PathBuf,
        source: std::io::Error,
    },
    Read {
        path: PathBuf,
        source: std::io::Error,
    },
    Parse {
        path: PathBuf,
        source: toml::de::Error,
    },
    Serialize(toml::ser::Error),
    Write {
        path: PathBuf,
        source: std::io::Error,
    },
    MigrateCache {
        source: PathBuf,
        target: PathBuf,
        error: std::io::Error,
    },
}

impl fmt::Display for ConfigError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CurrentExecutable(error) => {
                write!(formatter, "cannot locate executable: {error}")
            }
            Self::MissingExecutableDirectory => {
                formatter.write_str("executable has no parent directory")
            }
            Self::CreateCache { path, source } => write!(
                formatter,
                "cannot create cache directory {}: {source}",
                path.display()
            ),
            Self::Read { path, source } => {
                write!(formatter, "cannot read {}: {source}", path.display())
            }
            Self::Parse { path, source } => {
                write!(formatter, "cannot parse {}: {source}", path.display())
            }
            Self::Serialize(error) => write!(formatter, "cannot serialize config: {error}"),
            Self::Write { path, source } => {
                write!(formatter, "cannot write {}: {source}", path.display())
            }
            Self::MigrateCache {
                source,
                target,
                error,
            } => write!(
                formatter,
                "cannot migrate plugin cache from {} to {}: {error}",
                source.display(),
                target.display()
            ),
        }
    }
}

impl std::error::Error for ConfigError {}

#[cfg(test)]
mod tests {
    use super::{AppConfig, CURRENT_CONFIG_VERSION, PluginSettings, migrate_config};

    #[test]
    fn old_config_gets_default_vst2_paths_without_losing_vst3_paths() {
        let config: AppConfig = toml::from_str(
            r#"
                [plugins]
                vst3_paths = ["D:\\Audio\\VST3"]
            "#,
        )
        .expect("test config is valid TOML");

        assert_eq!(config.plugins.vst3_paths, [r"D:\Audio\VST3"]);
        assert_eq!(
            config.plugins.vst2_paths,
            PluginSettings::default().vst2_paths
        );
        assert!(!config.plugins.group_by_author);
    }

    #[test]
    fn plugin_grouping_round_trips() {
        let mut config = AppConfig::default();
        config.plugins.group_by_author = true;

        let encoded = toml::to_string(&config).expect("test config serializes");
        let decoded: AppConfig = toml::from_str(&encoded).expect("test config deserializes");

        assert!(decoded.plugins.group_by_author);
    }

    #[test]
    fn version_one_config_adds_steinberg_paths_once() {
        let mut config = AppConfig {
            version: 1,
            ..AppConfig::default()
        };
        config.plugins.vst2_paths = vec![String::from(r"C:\Program Files\Steinberg\VstPlugins")];

        assert!(migrate_config(&mut config));
        assert_eq!(config.version, CURRENT_CONFIG_VERSION);
        assert_eq!(config.plugins.vst2_paths.len(), 4);
        assert!(!migrate_config(&mut config));
        assert_eq!(config.plugins.vst2_paths.len(), 4);
    }
}
