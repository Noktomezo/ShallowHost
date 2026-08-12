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
    French,
    German,
    Spanish,
    Japanese,
    ChineseSimplified,
    Korean,
    Polish,
    PortugueseBrazil,
    Italian,
    ChineseTraditional,
    Ukrainian,
}

impl Language {
    pub const fn all() -> &'static [Self] {
        &[
            Self::System,
            Self::Russian,
            Self::English,
            Self::French,
            Self::German,
            Self::Spanish,
            Self::Japanese,
            Self::ChineseSimplified,
            Self::Korean,
            Self::Polish,
            Self::PortugueseBrazil,
            Self::Italian,
            Self::ChineseTraditional,
            Self::Ukrainian,
        ]
    }
}
