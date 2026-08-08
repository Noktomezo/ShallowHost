use gpui::prelude::*;
use gpui::*;
use std::time::Duration;

use super::appearance::{AppearanceOption, local_icon};
use super::resolve_path;
use crate::ui::foundation::colors;
use crate::ui::foundation::control_style::ControlTypography;
use crate::ui::foundation::motion::{
    CONTROL_MOTION, DropdownMotion, mix_color, set_dropdown_hovered,
};

const CONTROL_HEIGHT: Pixels = px(34.0);
const CONTROL_WIDTH: Pixels = px(220.0);

#[derive(IntoElement)]
pub(super) struct DropdownTrigger {
    id: &'static str,
    selected: Option<AppearanceOption>,
    motion: Entity<DropdownMotion>,
}

impl DropdownTrigger {
    pub(super) fn new(
        id: &'static str,
        selected: Option<AppearanceOption>,
        motion: Entity<DropdownMotion>,
    ) -> Self {
        Self {
            id,
            selected,
            motion,
        }
    }
}

impl RenderOnce for DropdownTrigger {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let motion_state = self.motion.read(cx);
        let open = motion_state.open();
        let hovered = motion_state.hovered();
        let surface_animating = motion_state.surface_animating();
        let open_animating = motion_state.open_animating();
        let animation_id = ElementId::NamedInteger(
            SharedString::from(format!("{}-chevron", self.id)),
            u64::from(open),
        );
        let surface_active = open || hovered;
        let surface_animation_id = ElementId::NamedInteger(
            SharedString::from(format!("{}-surface", self.id)),
            u64::from(surface_active),
        );
        let border_animation_id = ElementId::NamedInteger(
            SharedString::from(format!("{}-border", self.id)),
            u64::from(open),
        );
        let motion = self.motion.clone();
        let surface = div().absolute().inset_0().rounded_md();
        let surface = if surface_animating {
            surface
                .with_animation(
                    surface_animation_id,
                    Animation::new(CONTROL_MOTION).with_easing(ease_in_out),
                    move |element, delta| {
                        let progress = if surface_active { delta } else { 1.0 - delta };
                        element.bg(mix_color(colors::base_900(), colors::base_850(), progress))
                    },
                )
                .into_any_element()
        } else {
            surface
                .bg(if surface_active {
                    colors::base_850()
                } else {
                    colors::base_900()
                })
                .into_any_element()
        };
        let border = div().absolute().inset_0().rounded_md().border_1();
        let border = if open_animating {
            border
                .with_animation(
                    border_animation_id,
                    Animation::new(CONTROL_MOTION).with_easing(ease_in_out),
                    move |element, delta| {
                        let progress = if open { delta } else { 1.0 - delta };
                        element.border_color(mix_color(
                            colors::base_800(),
                            colors::orange(),
                            progress,
                        ))
                    },
                )
                .into_any_element()
        } else {
            border
                .border_color(if open {
                    colors::orange()
                } else {
                    colors::base_800()
                })
                .into_any_element()
        };
        let chevron = div().relative().size_4().flex_none();
        let chevron = if open_animating {
            chevron
                .with_animation(
                    animation_id,
                    Animation::new(Duration::from_millis(160)).with_easing(ease_in_out),
                    move |element, delta| {
                        let progress = if open { delta } else { 1.0 - delta };
                        element.child(chevron_svg(progress))
                    },
                )
                .into_any_element()
        } else {
            chevron
                .child(chevron_svg(if open { 1.0 } else { 0.0 }))
                .into_any_element()
        };

        div()
            .id(self.id)
            .relative()
            .w(CONTROL_WIDTH)
            .h(CONTROL_HEIGHT)
            .px_2()
            .flex()
            .items_center()
            .justify_between()
            .gap_2()
            .flex_none()
            .cursor_pointer()
            .control_text()
            .text_color(colors::base_200())
            .on_hover(move |hovered, window, cx| {
                set_dropdown_hovered(&motion, *hovered, window, cx);
            })
            .child(surface)
            .child(border)
            .child(if let Some(selected) = self.selected {
                div()
                    .relative()
                    .min_w_0()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(local_icon(selected.icon_path, selected.uses_flag))
                    .child(div().truncate().child(selected.label))
                    .into_any_element()
            } else {
                div().relative().child("—").into_any_element()
            })
            .child(chevron)
    }
}

fn chevron_svg(progress: f32) -> Svg {
    svg()
        .external_path(resolve_path("assets/icons/chevron-down.svg"))
        .size_4()
        .text_color(colors::base_500())
        .with_transformation(Transformation::rotate(Radians(
            std::f32::consts::PI * progress,
        )))
}
