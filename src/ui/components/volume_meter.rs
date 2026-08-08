use gpui::prelude::*;
use gpui::*;

use crate::ui::foundation::colors;

const THRESHOLDS: [f32; 7] = [0.05, 0.22, 0.38, 0.52, 0.65, 0.80, 0.92];

pub fn volume_meter(level: f32, peak: bool) -> AnyElement {
    div()
        .h(px(10.0))
        .flex()
        .items_center()
        .gap(px(4.0))
        .flex_none()
        .children((0..8).map(move |index| {
            let active = if index == 7 {
                peak
            } else {
                level >= THRESHOLDS[index]
            };
            let color = match index {
                0..=4 => colors::green(),
                5 => colors::yellow(),
                _ => colors::red(),
            };
            div()
                .size(px(8.0))
                .rounded_full()
                .bg(if active { color } else { rgba(0xb7b5ac40) })
                .when(active, |element| element.shadow_sm())
        }))
        .into_any_element()
}
