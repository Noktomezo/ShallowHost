use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ThemeMode {
    System,
    Light,
    Dark,
}

impl ThemeMode {
    pub const fn all() -> &'static [Self] {
        &[Self::System, Self::Light, Self::Dark]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Language {
    System,
    Russian,
    English,
}

impl Language {
    pub const fn all() -> &'static [Self] {
        &[Self::System, Self::Russian, Self::English]
    }
}
