use gpui::prelude::*;
use gpui::*;
use gpui_component::ActiveTheme;
use gpui_component::scroll::{Scrollbar, ScrollbarHandle, ScrollbarShow};
use std::time::Instant;

const THIN_WIDTH: Pixels = px(6.0);
const THICK_WIDTH: Pixels = px(8.0);
const THIN_INSET: Pixels = px(5.0);
const THICK_INSET: Pixels = px(4.0);
const SETTLE_DISTANCE: Pixels = px(0.25);
const VISUAL_RESPONSE_SECONDS: f32 = 0.045;

pub trait PageScrollHandle: ScrollbarHandle + Clone {
    fn viewport_height(&self) -> Pixels;
    fn max_scroll_y(&self) -> Pixels;
}

impl PageScrollHandle for ScrollHandle {
    fn viewport_height(&self) -> Pixels {
        self.bounds().size.height
    }

    fn max_scroll_y(&self) -> Pixels {
        self.max_offset().y
    }
}

impl PageScrollHandle for UniformListScrollHandle {
    fn viewport_height(&self) -> Pixels {
        self.0.borrow().base_handle.bounds().size.height
    }

    fn max_scroll_y(&self) -> Pixels {
        self.0.borrow().base_handle.max_offset().y
    }
}

struct PageScrollbarState {
    hovered: bool,
    expansion: f32,
    thumb_height: Option<Pixels>,
    last_frame: Instant,
}

impl PageScrollbarState {
    fn new() -> Self {
        Self {
            hovered: false,
            expansion: 0.0,
            thumb_height: None,
            last_frame: Instant::now(),
        }
    }

    fn advance(
        &mut self,
        target_height: Option<Pixels>,
        reduce_motion: bool,
    ) -> (Option<Pixels>, f32, bool) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_frame).as_secs_f32();
        self.last_frame = now;
        let target_expansion = if self.hovered { 1.0 } else { 0.0 };

        if reduce_motion {
            self.expansion = target_expansion;
            self.thumb_height = target_height;
            return (self.thumb_height, self.expansion, false);
        }

        let frame_seconds = elapsed.clamp(1.0 / 240.0, 1.0 / 30.0);
        let progress = 1.0 - (-frame_seconds / VISUAL_RESPONSE_SECONDS).exp();
        self.expansion = approach(self.expansion, target_expansion, progress, 0.01);

        self.thumb_height = match (self.thumb_height, target_height) {
            (None, Some(target)) => Some(target),
            (Some(current), Some(target)) => Some(approach_pixels(current, target, progress)),
            (Some(current), None) => {
                let next = approach_pixels(current, Pixels::ZERO, progress);
                (next > SETTLE_DISTANCE).then_some(next)
            }
            (None, None) => None,
        };

        let height_animating = match (self.thumb_height, target_height) {
            (Some(current), Some(target)) => (current - target).abs() > SETTLE_DISTANCE,
            (Some(_), None) => true,
            (None, Some(_)) => true,
            (None, None) => false,
        };
        let expansion_animating = (self.expansion - target_expansion).abs() > 0.01;
        (
            self.thumb_height,
            self.expansion,
            height_animating || expansion_animating,
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct ThumbTarget {
    container_height: Pixels,
    height: Pixels,
    progress: f32,
}

#[derive(IntoElement)]
pub struct PageScrollbar<H: PageScrollHandle> {
    id: ElementId,
    handle: H,
}

impl<H: PageScrollHandle> PageScrollbar<H> {
    pub fn new(id: impl Into<ElementId>, handle: H) -> Self {
        Self {
            id: id.into(),
            handle,
        }
    }
}

impl<H: PageScrollHandle> RenderOnce for PageScrollbar<H> {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let state = window
            .use_keyed_state((self.id.clone(), "scrollbar-state"), cx, |_, _| {
                PageScrollbarState::new()
            })
            .clone();
        let target = thumb_target(
            self.handle.viewport_height(),
            self.handle.max_scroll_y(),
            self.handle.offset().y,
        );
        let target_height = target.map(|target| target.height);
        let reduce_motion = cx.reduce_motion();
        let (thumb_height, expansion, animating) =
            state.update(cx, |state, _| state.advance(target_height, reduce_motion));
        if animating {
            window.request_animation_frame();
        }

        let hover_state = state.clone();
        let idle_thumb = target.zip(thumb_height).map(|(target, height)| {
            let height = height.min(target.container_height);
            let top = (target.container_height - height) * target.progress;
            let width = THIN_WIDTH + (THICK_WIDTH - THIN_WIDTH) * expansion;
            let inset = THIN_INSET + (THICK_INSET - THIN_INSET) * expansion;
            div()
                .absolute()
                .top(top)
                .right(inset)
                .w(width)
                .h(height)
                .rounded(width / 2.0)
                .bg(cx.theme().tokens.scrollbar_thumb.background)
        });

        div()
            .absolute()
            .inset_0()
            .when_some(idle_thumb, |layer, thumb| layer.child(thumb))
            .child(
                div()
                    .id((self.id.clone(), "scrollbar-hover-zone"))
                    .absolute()
                    .top_0()
                    .right_0()
                    .bottom_0()
                    .w(px(16.0))
                    .on_hover(move |hovered, window, cx| {
                        if hover_state.read(cx).hovered == *hovered {
                            return;
                        }
                        hover_state.update(cx, |state, cx| {
                            state.hovered = *hovered;
                            state.last_frame = Instant::now();
                            cx.notify();
                        });
                        window.refresh();
                    })
                    .child(
                        div().absolute().inset_0().opacity(expansion).child(
                            Scrollbar::vertical(&self.handle)
                                .scrollbar_show(ScrollbarShow::Hover)
                                .id((self.id, "scrollbar")),
                        ),
                    ),
            )
    }
}

fn thumb_target(
    container_height: Pixels,
    max_scroll: Pixels,
    offset_y: Pixels,
) -> Option<ThumbTarget> {
    if container_height <= Pixels::ZERO || max_scroll <= Pixels::ZERO {
        return None;
    }

    let content_height = container_height + max_scroll;
    let height = (container_height / content_height * container_height)
        .max(px(48.0))
        .min(container_height);
    let progress = (-offset_y / max_scroll).clamp(0.0, 1.0);
    Some(ThumbTarget {
        container_height,
        height,
        progress,
    })
}

fn approach(current: f32, target: f32, progress: f32, settle: f32) -> f32 {
    let next = current + (target - current) * progress;
    if (next - target).abs() <= settle {
        target
    } else {
        next
    }
}

fn approach_pixels(current: Pixels, target: Pixels, progress: f32) -> Pixels {
    let next = current + (target - current) * progress;
    if (next - target).abs() <= SETTLE_DISTANCE {
        target
    } else {
        next
    }
}

#[cfg(test)]
mod tests {
    use super::{approach, approach_pixels, thumb_target};
    use gpui::px;

    #[test]
    fn thumb_tracks_position_and_hides_without_overflow() {
        assert_eq!(thumb_target(px(400.0), px(0.0), px(0.0)), None);
        let target = thumb_target(px(400.0), px(400.0), px(-200.0))
            .expect("overflow produces a scrollbar thumb");
        assert_eq!(target.height, px(200.0));
        assert_eq!(target.progress, 0.5);
    }

    #[test]
    fn visual_values_approach_without_overshooting() {
        assert_eq!(approach(0.0, 1.0, 0.5, 0.0), 0.5);
        assert_eq!(approach_pixels(px(100.0), px(50.0), 0.5), px(75.0));
    }
}
