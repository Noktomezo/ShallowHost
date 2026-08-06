use gpui::{FontWeight, Styled, px};

pub const CONTROL_FONT_FAMILY: &str = "IBM Plex Sans";

pub trait ControlTypography: Styled + Sized {
    fn control_text(self) -> Self {
        self.font_family(CONTROL_FONT_FAMILY)
            .text_size(px(12.0))
            .font_weight(FontWeight::MEDIUM)
    }
}

impl<T: Styled> ControlTypography for T {}
