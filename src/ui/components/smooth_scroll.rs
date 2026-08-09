use std::time::Instant;

use gpui::prelude::*;
use gpui::*;

mod scrollbar;

pub use scrollbar::PageScrollbar;

const SETTLE_DISTANCE: Pixels = px(0.5);
const RESPONSE_SECONDS: f32 = 0.065;

#[derive(IntoElement)]
pub struct ScrollableColumn {
    id: ElementId,
    max_height: Pixels,
    child: AnyElement,
    base: Div,
}

impl ScrollableColumn {
    pub fn new(id: impl Into<ElementId>, max_height: Pixels, child: impl IntoElement) -> Self {
        Self {
            id: id.into(),
            max_height,
            child: child.into_any_element(),
            base: div(),
        }
    }
}

impl Styled for ScrollableColumn {
    fn style(&mut self) -> &mut StyleRefinement {
        self.base.style()
    }
}

impl RenderOnce for ScrollableColumn {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let handle = window
            .use_keyed_state((self.id.clone(), "scroll-handle"), cx, |_, _| {
                ScrollHandle::new()
            })
            .read(cx)
            .clone();
        self.base
            .relative()
            .overflow_hidden()
            .max_h(self.max_height)
            .child(
                div()
                    .id((self.id.clone(), "area"))
                    .w_full()
                    .max_h(self.max_height)
                    .track_scroll(&handle)
                    .overflow_y_scroll()
                    .child(self.child),
            )
            .child(PageScrollbar::new(self.id, handle))
    }
}

struct SmoothScrollState {
    handle: ScrollHandle,
    target_y: Pixels,
    running: bool,
    last_frame: Instant,
}

struct SmoothListState {
    target_y: Pixels,
    running: bool,
    last_frame: Instant,
}

impl SmoothListState {
    fn new() -> Self {
        Self {
            target_y: Pixels::ZERO,
            running: false,
            last_frame: Instant::now(),
        }
    }
}

impl SmoothScrollState {
    fn new() -> Self {
        Self {
            handle: ScrollHandle::new(),
            target_y: Pixels::ZERO,
            running: false,
            last_frame: Instant::now(),
        }
    }
}

/// A vertically scrollable area whose wheel input eases toward an accumulated target.
#[derive(IntoElement)]
pub struct SmoothVerticalScroll {
    id: ElementId,
    child: AnyElement,
}

impl SmoothVerticalScroll {
    /// Create a smooth vertical scroll area with a stable element id.
    pub fn new(id: impl Into<ElementId>, child: impl IntoElement) -> Self {
        Self {
            id: id.into(),
            child: child.into_any_element(),
        }
    }
}

impl RenderOnce for SmoothVerticalScroll {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let state = window
            .use_keyed_state((self.id.clone(), "state"), cx, |_, _| {
                SmoothScrollState::new()
            })
            .clone();
        let handle = state.read(cx).handle.clone();
        let wheel_state = state.clone();

        div()
            .id(self.id.clone())
            .relative()
            .size_full()
            .child(
                div()
                    .id((self.id.clone(), "area"))
                    .size_full()
                    .flex()
                    .flex_col()
                    .track_scroll(&handle)
                    .overflow_y_scroll()
                    .on_scroll_wheel(move |event, window, cx| {
                        let delta = event.delta.pixel_delta(window.line_height());
                        let delta_y = if delta.y.is_zero() { delta.x } else { delta.y };
                        if delta_y.is_zero() {
                            return;
                        }

                        handle_wheel(&wheel_state, delta_y, window, cx);
                        // GPUI applies its native offset first. handle_wheel restores that
                        // immediate jump and replaces it with the animated target.
                        cx.stop_propagation();
                    })
                    .child(self.child),
            )
            .child(PageScrollbar::new(self.id, handle))
    }
}

/// Adds eased wheel scrolling to GPUI's fixed-height virtualized list.
#[derive(IntoElement)]
pub struct SmoothUniformListScroll {
    id: ElementId,
    handle: UniformListScrollHandle,
    child: AnyElement,
    wheel_enabled: bool,
}

impl SmoothUniformListScroll {
    pub fn new(
        id: impl Into<ElementId>,
        handle: UniformListScrollHandle,
        child: impl IntoElement,
    ) -> Self {
        Self {
            id: id.into(),
            handle,
            child: child.into_any_element(),
            wheel_enabled: true,
        }
    }

    pub fn wheel_enabled(mut self, enabled: bool) -> Self {
        self.wheel_enabled = enabled;
        self
    }
}

impl RenderOnce for SmoothUniformListScroll {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let state = window
            .use_keyed_state((self.id.clone(), "smooth-list-state"), cx, |_, _| {
                SmoothListState::new()
            })
            .clone();

        CaptureListWheel {
            child: div()
                .id(self.id)
                .size_full()
                .child(self.child)
                .into_any_element(),
            state,
            handle: UniformScrollHandle(self.handle),
            wheel_enabled: self.wheel_enabled,
        }
    }
}

trait SmoothVirtualHandle: Clone + 'static {
    fn offset(&self) -> Point<Pixels>;
    fn max_scroll_y(&self) -> Pixels;
    fn set_offset(&self, offset: Point<Pixels>);
}

#[derive(Clone)]
struct UniformScrollHandle(UniformListScrollHandle);

impl SmoothVirtualHandle for UniformScrollHandle {
    fn offset(&self) -> Point<Pixels> {
        self.0.0.borrow().base_handle.offset()
    }

    fn max_scroll_y(&self) -> Pixels {
        self.0.0.borrow().base_handle.max_offset().y
    }

    fn set_offset(&self, offset: Point<Pixels>) {
        self.0.0.borrow().base_handle.set_offset(offset);
    }
}

struct CaptureListWheel<H: SmoothVirtualHandle> {
    child: AnyElement,
    state: Entity<SmoothListState>,
    handle: H,
    wheel_enabled: bool,
}

impl<H: SmoothVirtualHandle> IntoElement for CaptureListWheel<H> {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl<H: SmoothVirtualHandle> Element for CaptureListWheel<H> {
    type RequestLayoutState = ();
    type PrepaintState = ();

    fn id(&self) -> Option<ElementId> {
        None
    }

    fn source_location(&self) -> Option<&'static core::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _global_id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        (self.child.request_layout(window, cx), ())
    }

    fn prepaint(
        &mut self,
        _global_id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        _bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        self.child.prepaint(window, cx);
    }

    fn paint(
        &mut self,
        _global_id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        _prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        self.child.paint(window, cx);

        if !self.wheel_enabled {
            return;
        }

        let state = self.state.clone();
        let handle = self.handle.clone();
        window.on_mouse_event(move |event: &ScrollWheelEvent, phase, window, cx| {
            if phase != DispatchPhase::Capture || !bounds.contains(&event.position) {
                return;
            }

            let delta = event.delta.pixel_delta(px(20.0));
            let delta_y = if delta.y.is_zero() { delta.x } else { delta.y };
            if delta_y.is_zero() {
                return;
            }

            handle_list_wheel(&state, &handle, delta_y, window, cx);
            cx.stop_propagation();
        });
    }
}

fn handle_wheel(
    state: &Entity<SmoothScrollState>,
    delta_y: Pixels,
    window: &mut Window,
    cx: &mut App,
) {
    let reduce_motion = cx.reduce_motion();
    let should_schedule = state.update(cx, |state, _| {
        let applied_offset = state.handle.offset();
        let max_scroll = state.handle.max_offset().y;
        let current_y = (applied_offset.y - delta_y).clamp(-max_scroll, Pixels::ZERO);
        state.handle.set_offset(point(applied_offset.x, current_y));

        if !state.running {
            state.target_y = current_y;
        }
        state.target_y = coalesced_target(current_y, state.target_y, delta_y, max_scroll);

        if reduce_motion {
            state
                .handle
                .set_offset(point(applied_offset.x, state.target_y));
            state.running = false;
            false
        } else if state.running {
            false
        } else {
            state.running = true;
            state.last_frame = Instant::now();
            true
        }
    });

    window.refresh();
    if should_schedule {
        schedule_frame(state.clone(), window);
    }
}

fn schedule_frame(state: Entity<SmoothScrollState>, window: &Window) {
    window.on_next_frame(move |window, cx| advance_frame(state, window, cx));
}

fn handle_list_wheel(
    state: &Entity<SmoothListState>,
    handle: &impl SmoothVirtualHandle,
    delta_y: Pixels,
    window: &mut Window,
    cx: &mut App,
) {
    let reduce_motion = cx.reduce_motion();
    let should_schedule = state.update(cx, |state, _| {
        let applied_offset = handle.offset();
        let max_scroll = handle.max_scroll_y();
        let current_y = applied_offset.y.clamp(-max_scroll, Pixels::ZERO);

        if !state.running {
            state.target_y = current_y;
        }
        state.target_y = coalesced_target(current_y, state.target_y, delta_y, max_scroll);

        if reduce_motion {
            handle.set_offset(point(applied_offset.x, state.target_y));
            state.running = false;
            false
        } else if state.running {
            false
        } else {
            state.running = true;
            state.last_frame = Instant::now();
            true
        }
    });

    window.refresh();
    if should_schedule {
        schedule_list_frame(state.clone(), handle.clone(), window);
    }
}

fn schedule_list_frame(
    state: Entity<SmoothListState>,
    handle: impl SmoothVirtualHandle,
    window: &Window,
) {
    window.on_next_frame(move |window, cx| advance_list_frame(state, handle, window, cx));
}

fn advance_list_frame(
    state: Entity<SmoothListState>,
    handle: impl SmoothVirtualHandle,
    window: &mut Window,
    cx: &mut App,
) {
    let keep_running = state.update(cx, |state, _| {
        let now = Instant::now();
        let elapsed = now.duration_since(state.last_frame).as_secs_f32();
        state.last_frame = now;

        let current = handle.offset();
        let max_scroll = handle.max_scroll_y();
        state.target_y = state.target_y.clamp(-max_scroll, Pixels::ZERO);
        let distance = state.target_y - current.y;
        if distance.abs() <= SETTLE_DISTANCE {
            handle.set_offset(point(current.x, state.target_y));
            state.running = false;
            return false;
        }

        let frame_seconds = elapsed.clamp(1.0 / 240.0, 1.0 / 30.0);
        let progress = 1.0 - (-frame_seconds / RESPONSE_SECONDS).exp();
        handle.set_offset(point(current.x, current.y + distance * progress));
        true
    });

    window.refresh();
    if keep_running {
        schedule_list_frame(state, handle, window);
    }
}

fn advance_frame(state: Entity<SmoothScrollState>, window: &mut Window, cx: &mut App) {
    let keep_running = state.update(cx, |state, _| {
        let now = Instant::now();
        let elapsed = now.duration_since(state.last_frame).as_secs_f32();
        state.last_frame = now;

        let current = state.handle.offset();
        let max_scroll = state.handle.max_offset().y;
        state.target_y = state.target_y.clamp(-max_scroll, Pixels::ZERO);
        let distance = state.target_y - current.y;
        if distance.abs() <= SETTLE_DISTANCE {
            state.handle.set_offset(point(current.x, state.target_y));
            state.running = false;
            return false;
        }

        let frame_seconds = elapsed.clamp(1.0 / 240.0, 1.0 / 30.0);
        let progress = 1.0 - (-frame_seconds / RESPONSE_SECONDS).exp();
        state
            .handle
            .set_offset(point(current.x, current.y + distance * progress));
        true
    });

    window.refresh();
    if keep_running {
        schedule_frame(state, window);
    }
}

fn coalesced_target(current: Pixels, target: Pixels, delta: Pixels, max_scroll: Pixels) -> Pixels {
    let pending = target - current;
    let reverses_direction = (pending < Pixels::ZERO && delta > Pixels::ZERO)
        || (pending > Pixels::ZERO && delta < Pixels::ZERO);
    let next = if reverses_direction {
        current + delta
    } else {
        target + delta
    };
    next.clamp(-max_scroll, Pixels::ZERO)
}

#[cfg(test)]
mod tests {
    use super::coalesced_target;
    use gpui::px;

    #[test]
    fn wheel_targets_accumulate_and_clamp() {
        assert_eq!(
            coalesced_target(px(-20.0), px(-40.0), px(-30.0), px(100.0)),
            px(-70.0)
        );
        assert_eq!(
            coalesced_target(px(-80.0), px(-90.0), px(-30.0), px(100.0)),
            px(-100.0)
        );
    }

    #[test]
    fn reversing_wheel_direction_discards_old_momentum() {
        assert_eq!(
            coalesced_target(px(-40.0), px(-80.0), px(15.0), px(100.0)),
            px(-25.0)
        );
    }
}
