use std::time::Duration;

use gpui::*;

const MARQUEE_DURATION: Duration = Duration::from_millis(1_800);
const END_GUTTER: Pixels = px(8.0);

#[derive(IntoElement)]
pub struct MarqueeText {
    id: ElementId,
    text: SharedString,
    active: bool,
}

impl MarqueeText {
    pub fn new(id: impl Into<ElementId>, text: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            text: text.into(),
            active: false,
        }
    }

    pub fn active(mut self, active: bool) -> Self {
        self.active = active;
        self
    }
}

impl RenderOnce for MarqueeText {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let viewport = div().min_w_0().flex_1().h_full().overflow_hidden();

        if self.active {
            let text = self.text;
            viewport
                .with_animation(
                    self.id,
                    Animation::new(MARQUEE_DURATION)
                        .repeat()
                        .with_easing(bounce(ease_in_out)),
                    move |element, progress| {
                        element.child(MarqueeTextElement::new(text.clone(), progress))
                    },
                )
                .into_any_element()
        } else {
            viewport
                .child(MarqueeTextElement::new(self.text, 0.0))
                .into_any_element()
        }
    }
}

struct MarqueeTextElement {
    text: SharedString,
    progress: f32,
}

impl MarqueeTextElement {
    fn new(text: SharedString, progress: f32) -> Self {
        Self { text, progress }
    }
}

struct PrepaintState {
    line: ShapedLine,
    origin: Point<Pixels>,
}

impl IntoElement for MarqueeTextElement {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl Element for MarqueeTextElement {
    type RequestLayoutState = ();
    type PrepaintState = PrepaintState;

    fn id(&self) -> Option<ElementId> {
        None
    }

    fn source_location(&self) -> Option<&'static core::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let mut style = Style::default();
        style.size.width = relative(1.0).into();
        style.size.height = relative(1.0).into();
        (window.request_layout(style, [], cx), ())
    }

    fn prepaint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _: &mut Self::RequestLayoutState,
        window: &mut Window,
        _: &mut App,
    ) -> Self::PrepaintState {
        let text_style = window.text_style();
        let run = TextRun {
            len: self.text.len(),
            font: text_style.font(),
            color: text_style.color,
            background_color: None,
            underline: None,
            strikethrough: None,
        };
        let font_size = text_style.font_size.to_pixels(window.rem_size());
        let line = window
            .text_system()
            .shape_line(self.text.clone(), font_size, &[run], None);
        let overflow = marquee_shift(line.width(), bounds.size.width);
        let origin = point(bounds.left() - overflow * self.progress, bounds.top());

        PrepaintState { line, origin }
    }

    fn paint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _: &mut Self::RequestLayoutState,
        prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        window.with_content_mask(Some(ContentMask { bounds }), |window| {
            if let Err(error) = prepaint.line.paint(
                prepaint.origin,
                bounds.size.height,
                TextAlign::Left,
                None,
                window,
                cx,
            ) {
                eprintln!("failed to paint dropdown marquee text: {error}");
            }
        });
    }
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
