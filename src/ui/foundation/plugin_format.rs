use gpui::SharedString;

pub fn display_name(format: &str) -> SharedString {
    if format.eq_ignore_ascii_case("VST") {
        "VST2".into()
    } else {
        format.to_uppercase().into()
    }
}

#[cfg(test)]
mod tests {
    use super::display_name;

    #[test]
    fn distinguishes_juces_legacy_vst_name() {
        assert_eq!(display_name("VST"), "VST2");
        assert_eq!(display_name("VST3"), "VST3");
    }
}
