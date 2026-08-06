use super::pages::Language;

pub fn detect_system_language() -> &'static str {
    if let Ok(lang) = std::env::var("LANG").or_else(|_| std::env::var("LC_ALL")) {
        let lang_lower = lang.to_lowercase();
        if lang_lower.starts_with("ru") {
            return "ru";
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
        let len = unsafe { GetUserDefaultLocaleName(buf.as_mut_ptr(), buf.len() as i32) };
        if len > 1 {
            let locale = OsString::from_wide(&buf[..(len as usize - 1)])
                .to_string_lossy()
                .to_lowercase();
            if locale.starts_with("ru") {
                return "ru";
            }
        }
    }

    // Fallback to English if system language is not supported
    "en"
}

impl Language {
    pub fn code(&self) -> &'static str {
        match self {
            Self::System => detect_system_language(),
            Self::Russian => "ru",
            Self::English => "en",
        }
    }
}

pub fn set_language(lang: Language) {
    rust_i18n::set_locale(lang.code());
}

pub fn t(key: &str) -> String {
    rust_i18n::t!(key).to_string()
}
