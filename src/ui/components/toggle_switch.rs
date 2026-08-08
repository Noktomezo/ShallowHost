use gpui::prelude::*;
use gpui::*;
use std::time::{Duration, Instant};

use crate::ui::foundation::colors;
use crate::ui::foundation::motion::{changed_recently, mix_color};

const SWITCH_MOTION: Duration = Duration::from_millis(180);
const SWITCH_TRAVEL: Pixels = px(16.0);

pub fn toggle_switch(
    id: &'static str,
    checked: bool,
    enabled: bool,
    changed_at: Option<Instant>,
) -> AnyElement {
    let animate = enabled && changed_recently(changed_at, SWITCH_MOTION);
    let animation_id = ElementId::NamedInteger(
        SharedString::from(format!("{id}-switch-motion")),
        u64::from(checked),
    );
    let track = div()
        .w(px(38.0))
        .h(px(22.0))
        .p(px(2.0))
        .flex()
        .items_center()
        .rounded_full()
        .border_1();

    if animate {
        track
            .with_animation(
                animation_id,
                Animation::new(SWITCH_MOTION).with_easing(ease_in_out),
                move |element, delta| switch_frame(element, switch_progress(checked, delta)),
            )
            .into_any_element()
    } else {
        switch_frame(track, if checked { 1.0 } else { 0.0 }).into_any_element()
    }
}

fn switch_progress(checked: bool, delta: f32) -> f32 {
    if checked { delta } else { 1.0 - delta }
}

fn switch_frame(track: Div, progress: f32) -> Div {
    track
        .bg(mix_color(colors::base_850(), colors::orange(), progress))
        .border_color(mix_color(colors::base_700(), colors::orange(), progress))
        .child(
            div()
                .size(px(16.0))
                .ml(SWITCH_TRAVEL * progress)
                .rounded_full()
                .bg(mix_color(
                    colors::base_300(),
                    colors::accent_foreground(),
                    progress,
                ))
                .shadow_sm(),
        )
}

#[cfg(test)]
mod tests {
    use super::switch_progress;

    #[test]
    fn switch_animation_runs_in_both_directions() {
        assert_eq!(switch_progress(true, 0.25), 0.25);
        assert_eq!(switch_progress(false, 0.25), 0.75);
    }
}
