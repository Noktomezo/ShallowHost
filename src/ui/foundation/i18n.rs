use crate::ui::shell::routes::Language;

pub fn detect_system_language() -> &'static str {
    if let Ok(lang) = std::env::var("LANG").or_else(|_| std::env::var("LC_ALL")) {
        let locale = supported_locale(&lang);
        if locale != "en" || lang.to_ascii_lowercase().starts_with("en") {
            return locale;
        }
    }

    #[cfg(target_os = "windows")]
    {
        use std::ffi::OsString;
        use std::os::windows::ffi::OsStringExt;

        unsafe extern "system" {
            fn GetUserDefaultLocaleName(lpLocaleName: *mut u16, cchLocaleName: i32) -> i32;
        }

        let mut buf = [0u16; 85];
        // SAFETY: `buf` is writable for the supplied length and remains alive for the call.
        let len = unsafe { GetUserDefaultLocaleName(buf.as_mut_ptr(), buf.len() as i32) };
        if len > 1 {
            let locale = OsString::from_wide(&buf[..(len as usize - 1)])
                .to_string_lossy()
                .into_owned();
            return supported_locale(&locale);
        }
    }

    // Fallback to English if system language is not supported
    "en"
}

fn supported_locale(locale: &str) -> &'static str {
    let locale = locale.to_ascii_lowercase().replace('_', "-");
    match locale.as_str() {
        value
            if value.starts_with("zh-tw")
                || value.starts_with("zh-hk")
                || value.starts_with("zh-mo")
                || value.starts_with("zh-hant") =>
        {
            "zh-TW"
        }
        value if value.starts_with("zh") => "zh-CN",
        value if value.starts_with("pt") => "pt-BR",
        value if value.starts_with("ru") => "ru",
        value if value.starts_with("fr") => "fr",
        value if value.starts_with("de") => "de",
        value if value.starts_with("es") => "es",
        value if value.starts_with("ja") => "ja",
        value if value.starts_with("ko") => "ko",
        value if value.starts_with("pl") => "pl",
        value if value.starts_with("it") => "it",
        value if value.starts_with("uk") => "uk",
        _ => "en",
    }
}

impl Language {
    pub fn code(&self) -> &'static str {
        match self {
            Self::System => detect_system_language(),
            Self::Russian => "ru",
            Self::English => "en",
            Self::French => "fr",
            Self::German => "de",
            Self::Spanish => "es",
            Self::Japanese => "ja",
            Self::ChineseSimplified => "zh-CN",
            Self::Korean => "ko",
            Self::Polish => "pl",
            Self::PortugueseBrazil => "pt-BR",
            Self::Italian => "it",
            Self::ChineseTraditional => "zh-TW",
            Self::Ukrainian => "uk",
        }
    }
}

pub fn set_language(lang: Language) {
    rust_i18n::set_locale(lang.code());
}

pub fn t(key: &str) -> String {
    rust_i18n::t!(key).to_string()
}

#[cfg(test)]
mod tests {
    use super::supported_locale;

    #[test]
    fn maps_supported_system_locales() {
        for (input, expected) in [
            ("de-DE", "de"),
            ("pt_BR.UTF-8", "pt-BR"),
            ("zh-Hant-HK", "zh-TW"),
            ("zh_CN", "zh-CN"),
            ("uk-UA", "uk"),
            ("nl-NL", "en"),
        ] {
            assert_eq!(supported_locale(input), expected);
        }
    }
}
