use gpui::SharedString;

use crate::ui::components::badge::BadgeStyle;

pub fn display_name(format: &str) -> SharedString {
    if format.eq_ignore_ascii_case("VST") {
        "VST2".into()
    } else {
        format.to_uppercase().into()
    }
}

pub fn badge_style(format: &str) -> BadgeStyle {
    if format.eq_ignore_ascii_case("VST") || format.eq_ignore_ascii_case("VST2") {
        BadgeStyle::Cyan
    } else {
        BadgeStyle::Purple
    }
}

#[cfg(test)]
mod tests {
    use super::{badge_style, display_name};
    use crate::ui::components::badge::BadgeStyle;

    #[test]
    fn distinguishes_juces_legacy_vst_name() {
        assert_eq!(display_name("VST"), "VST2");
        assert_eq!(display_name("VST3"), "VST3");
    }

    #[test]
    fn gives_vst2_a_distinct_badge_color() {
        assert!(matches!(badge_style("VST"), BadgeStyle::Cyan));
        assert!(matches!(badge_style("VST2"), BadgeStyle::Cyan));
        assert!(matches!(badge_style("VST3"), BadgeStyle::Purple));
    }
}
