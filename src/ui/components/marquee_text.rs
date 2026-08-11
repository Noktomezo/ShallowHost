use std::time::Duration;

use gpui::*;

use crate::ui::foundation::control_style::CONTROL_FONT_FAMILY;

const MARQUEE_DURATION: Duration = Duration::from_millis(1_800);
const END_GUTTER: Pixels = px(8.0);
const CONTROL_FONT_SIZE: Pixels = px(12.0);

#[derive(IntoElement)]
pub struct MarqueeText {
    id: ElementId,
    text: SharedString,
    max_width: Pixels,
    active: bool,
}

impl MarqueeText {
    pub fn new(id: impl Into<ElementId>, text: impl Into<SharedString>, max_width: Pixels) -> Self {
        Self {
            id: id.into(),
            text: text.into(),
            max_width,
            active: false,
        }
    }

    pub fn active(mut self, active: bool) -> Self {
        self.active = active;
        self
    }
}

impl RenderOnce for MarqueeText {
    fn render(self, window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let text_width = control_text_width(&self.text, window);
        let viewport_width = text_width.min(self.max_width);
        let shift = marquee_shift(text_width, viewport_width);
        let viewport = div()
            .w(viewport_width)
            .h_full()
            .flex_none()
            .flex()
            .items_center()
            .overflow_hidden();

        if self.active && shift > Pixels::ZERO {
            let text = self.text;
            viewport
                .with_animation(
                    self.id,
                    Animation::new(MARQUEE_DURATION)
                        .repeat()
                        .with_easing(bounce(ease_in_out)),
                    move |element, progress| {
                        element.child(
                            div()
                                .relative()
                                .left(-shift * progress)
                                .flex_none()
                                .whitespace_nowrap()
                                .child(text.clone()),
                        )
                    },
                )
                .into_any_element()
        } else {
            viewport
                .child(div().flex_none().whitespace_nowrap().child(self.text))
                .into_any_element()
        }
    }
}

pub fn control_text_width(text: &SharedString, window: &mut Window) -> Pixels {
    let mut control_font = font(CONTROL_FONT_FAMILY);
    control_font.weight = FontWeight::MEDIUM;
    let run = TextRun {
        len: text.len(),
        font: control_font,
        color: window.text_style().color,
        background_color: None,
        underline: None,
        strikethrough: None,
    };
    window
        .text_system()
        .shape_line(text.clone(), CONTROL_FONT_SIZE, &[run], None)
        .width()
}

fn marquee_shift(text_width: Pixels, viewport_width: Pixels) -> Pixels {
    if text_width > viewport_width {
        text_width - viewport_width + END_GUTTER
    } else {
        Pixels::ZERO
    }
}

#[cfg(test)]
mod tests {
    use gpui::{Pixels, px};

    use super::marquee_shift;

    #[test]
    fn short_text_does_not_move() {
        assert_eq!(marquee_shift(px(80.0), px(100.0)), Pixels::ZERO);
    }

    #[test]
    fn overflowing_text_reveals_the_end_with_a_gutter() {
        assert_eq!(marquee_shift(px(140.0), px(100.0)), px(48.0));
    }
}
