use gpui::{FontWeight, Pixels, Styled, px};

pub const CONTROL_FONT_FAMILY: &str = "IBM Plex Sans";
pub const DROPDOWN_CONTROL_HEIGHT: Pixels = px(34.0);
pub const DROPDOWN_CONTROL_WIDTH: Pixels = px(160.0);

pub trait ControlTypography: Styled + Sized {
    fn control_text(self) -> Self {
        self.font_family(CONTROL_FONT_FAMILY)
            .text_size(px(12.0))
            .font_weight(FontWeight::MEDIUM)
    }
}

impl<T: Styled> ControlTypography for T {}
