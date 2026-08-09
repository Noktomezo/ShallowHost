use std::time::Instant;

use gpui::prelude::*;
use gpui::*;

use crate::ui::foundation::colors;

const THIN_WIDTH: Pixels = px(6.0);
const THICK_WIDTH: Pixels = px(8.0);
const THIN_INSET: Pixels = px(5.0);
const THICK_INSET: Pixels = px(4.0);
const SETTLE_DISTANCE: Pixels = px(0.25);
const VISUAL_RESPONSE_SECONDS: f32 = 0.045;

pub trait PageScrollHandle: Clone + 'static {
    fn base_handle(&self) -> ScrollHandle;

    fn viewport_height(&self) -> Pixels {
        self.base_handle().bounds().size.height
    }

    fn max_scroll_y(&self) -> Pixels {
        self.base_handle().max_offset().y
    }

    fn offset_y(&self) -> Pixels {
        self.base_handle().offset().y
    }

    fn set_offset_y(&self, offset_y: Pixels) {
        let handle = self.base_handle();
        let current = handle.offset();
        handle.set_offset(point(current.x, offset_y));
    }
}

impl PageScrollHandle for ScrollHandle {
    fn base_handle(&self) -> ScrollHandle {
        self.clone()
    }
}

impl PageScrollHandle for UniformListScrollHandle {
    fn base_handle(&self) -> ScrollHandle {
        self.0.borrow().base_handle.clone()
    }
}

struct PageScrollbarState {
    hovered: bool,
    dragging: bool,
    expansion: f32,
    thumb_height: Option<Pixels>,
    drag_origin_y: Pixels,
    drag_start_offset_y: Pixels,
    last_frame: Instant,
}

impl PageScrollbarState {
    fn new() -> Self {
        Self {
            hovered: false,
            dragging: false,
            expansion: 0.0,
            thumb_height: None,
            drag_origin_y: Pixels::ZERO,
            drag_start_offset_y: Pixels::ZERO,
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
        let target_expansion = if self.hovered || self.dragging {
            1.0
        } else {
            0.0
        };

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
            (Some(_), None) | (None, Some(_)) => true,
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
            self.handle.offset_y(),
        );
        let target_height = target.map(|target| target.height);
        let reduce_motion = cx.reduce_motion();
        let (thumb_height, expansion, animating) =
            state.update(cx, |state, _| state.advance(target_height, reduce_motion));
        if animating {
            window.request_animation_frame();
        }

        let hover_state = state.clone();
        let move_state = state.clone();
        let move_handle = self.handle.clone();
        let release_state = state.clone();
        let click_handle = self.handle.clone();
        let click_state = state.clone();
        let thumb = target.zip(thumb_height).map(|(target, height)| {
            let height = height.min(target.container_height);
            let top = (target.container_height - height) * target.progress;
            let width = THIN_WIDTH + (THICK_WIDTH - THIN_WIDTH) * expansion;
            let inset = THIN_INSET + (THICK_INSET - THIN_INSET) * expansion;
            let drag_state = state.clone();
            let drag_handle = self.handle.clone();
            div()
                .id((self.id.clone(), "scrollbar-thumb"))
                .absolute()
                .top(top)
                .right(inset)
                .w(width)
                .h(height)
                .rounded(width / 2.0)
                .bg(colors::base_500())
                .on_mouse_down(MouseButton::Left, move |event, window, cx| {
                    drag_state.update(cx, |state, cx| {
                        state.dragging = true;
                        state.drag_origin_y = event.position.y;
                        state.drag_start_offset_y = drag_handle.offset_y();
                        state.last_frame = Instant::now();
                        cx.notify();
                    });
                    cx.stop_propagation();
                    window.refresh();
                })
        });

        div().absolute().inset_0().child(
            div()
                .id((self.id.clone(), "scrollbar-zone"))
                .absolute()
                .top_0()
                .right_0()
                .bottom_0()
                .w(px(16.0))
                .on_hover(move |hovered, window, cx| {
                    hover_state.update(cx, |state, cx| {
                        if state.hovered == *hovered {
                            return;
                        }
                        state.hovered = *hovered;
                        state.last_frame = Instant::now();
                        cx.notify();
                    });
                    window.refresh();
                })
                .on_mouse_down(MouseButton::Left, move |event, window, cx| {
                    let Some(target) = target else {
                        return;
                    };
                    let bounds = click_handle.base_handle().bounds();
                    let track = (target.container_height - target.height).max(px(1.0));
                    let local_y = event.position.y - bounds.top() - target.height / 2.0;
                    let progress = (local_y / track).clamp(0.0, 1.0);
                    click_handle.set_offset_y(-click_handle.max_scroll_y() * progress);
                    click_state.update(cx, |state, cx| {
                        state.dragging = true;
                        state.drag_origin_y = event.position.y;
                        state.drag_start_offset_y = click_handle.offset_y();
                        cx.notify();
                    });
                    window.refresh();
                })
                .on_mouse_move(move |event, window, cx| {
                    let (dragging, origin_y, start_offset) =
                        move_state.read_with(cx, |state, _| {
                            (
                                state.dragging,
                                state.drag_origin_y,
                                state.drag_start_offset_y,
                            )
                        });
                    let Some(target) = target.filter(|_| dragging) else {
                        return;
                    };
                    let track = (target.container_height - target.height).max(px(1.0));
                    let start_progress =
                        (-start_offset / move_handle.max_scroll_y()).clamp(0.0, 1.0);
                    let progress =
                        (start_progress + (event.position.y - origin_y) / track).clamp(0.0, 1.0);
                    move_handle.set_offset_y(-move_handle.max_scroll_y() * progress);
                    window.refresh();
                })
                .on_mouse_up(MouseButton::Left, move |_, window, cx| {
                    release_drag(&release_state, window, cx);
                })
                .on_mouse_up_out(MouseButton::Left, move |_, window, cx| {
                    release_drag(&state, window, cx);
                })
                .when_some(thumb, |zone, thumb| zone.child(thumb)),
        )
    }
}

fn release_drag(state: &Entity<PageScrollbarState>, window: &mut Window, cx: &mut App) {
    state.update(cx, |state, cx| {
        state.dragging = false;
        state.last_frame = Instant::now();
        cx.notify();
    });
    window.refresh();
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
    use super::{PageScrollbarState, thumb_target};
    use gpui::px;

    #[test]
    fn thumb_tracks_position_and_hides_without_overflow() {
        let top = thumb_target(px(100.0), px(300.0), px(0.0));
        let bottom = thumb_target(px(100.0), px(300.0), px(-300.0));
        assert_eq!(top.map(|target| target.progress), Some(0.0));
        assert_eq!(bottom.map(|target| target.progress), Some(1.0));
        assert!(thumb_target(px(100.0), px(0.0), px(0.0)).is_none());
    }

    #[test]
    fn visual_values_approach_without_overshooting() {
        let mut state = PageScrollbarState::new();
        state.hovered = true;
        state.last_frame -= std::time::Duration::from_millis(16);
        let (_, expansion, _) = state.advance(Some(px(80.0)), false);
        assert!((0.0..=1.0).contains(&expansion));
    }
}
