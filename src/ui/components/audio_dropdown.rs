use gpui::prelude::*;
use gpui::*;
use std::time::Duration;

use super::dropdown_overlay::adaptive_dropdown;
use super::marquee_text::{MarqueeFade, MarqueeText, control_text_width};
use super::smooth_scroll::ScrollableColumn;
use crate::ui::foundation::colors;
use crate::ui::foundation::control_style::{
    ControlTypography, DROPDOWN_CONTROL_HEIGHT, DROPDOWN_CONTROL_WIDTH, DROPDOWN_LABEL_WIDTH,
    DROPDOWN_TRAILING_GUTTER,
};
use crate::ui::foundation::motion::{
    CONTROL_MOTION, DropdownMotion, MENU_MOTION, mix_color, set_dropdown_hovered,
    set_dropdown_item_hovered, set_dropdown_open,
};
use crate::ui::resolve_asset_path;

pub const CONTROL_HEIGHT: Pixels = DROPDOWN_CONTROL_HEIGHT;
pub const CONTROL_WIDTH: Pixels = DROPDOWN_CONTROL_WIDTH;

#[derive(Clone, Debug)]
pub struct DropdownChoice {
    pub label: SharedString,
    pub value: SharedString,
    muted_suffix: Option<SharedString>,
}

impl DropdownChoice {
    pub fn new(value: impl Into<SharedString>, label: impl Into<SharedString>) -> Self {
        Self {
            label: label.into(),
            value: value.into(),
            muted_suffix: None,
        }
    }

    pub fn with_muted_suffix(mut self, suffix: impl Into<SharedString>) -> Self {
        self.muted_suffix = Some(suffix.into());
        self
    }
}

pub struct AudioDropdownState {
    choices: Vec<DropdownChoice>,
    selected: usize,
    motion: Entity<DropdownMotion>,
}

impl AudioDropdownState {
    pub fn new(
        choices: Vec<DropdownChoice>,
        selected: usize,
        motion: Entity<DropdownMotion>,
    ) -> Self {
        let selected = selected.min(choices.len().saturating_sub(1));
        Self {
            choices,
            selected,
            motion,
        }
    }

    pub fn selected_value(&self) -> Option<&str> {
        self.choices
            .get(self.selected)
            .map(|choice| choice.value.as_ref())
    }

    pub fn replace_choices(&mut self, choices: Vec<DropdownChoice>, selected: usize) {
        self.choices = choices;
        self.selected = selected.min(self.choices.len().saturating_sub(1));
    }

    pub fn select_value(&mut self, value: &str) -> bool {
        let Some(index) = self
            .choices
            .iter()
            .position(|choice| choice.value.as_ref() == value)
        else {
            return false;
        };
        self.selected = index;
        true
    }
}

#[derive(Clone, Debug)]
pub struct AudioDropdownEvent;

impl EventEmitter<AudioDropdownEvent> for AudioDropdownState {}

pub fn reset_audio_dropdown(state: &Entity<AudioDropdownState>, cx: &mut App) {
    let motion = state.read(cx).motion.clone();
    crate::ui::foundation::motion::reset_dropdown_interaction(&motion, cx);
}

pub fn audio_dropdown(
    id: &'static str,
    state: &Entity<AudioDropdownState>,
    cx: &App,
) -> AnyElement {
    let selected_choice = state
        .read(cx)
        .choices
        .get(state.read(cx).selected)
        .cloned()
        .unwrap_or_else(|| DropdownChoice::new("", "—"));
    let motion = state.read(cx).motion.clone();
    let trigger = DropdownTrigger::new(id, selected_choice, motion.clone());
    let dropdown = state.clone();
    let menu_motion = motion.clone();
    let menu = render_menu(id, &dropdown, &menu_motion, cx);

    adaptive_dropdown(id, trigger, menu, motion, cx)
}

fn render_menu(
    id: &'static str,
    dropdown: &Entity<AudioDropdownState>,
    motion: &Entity<DropdownMotion>,
    cx: &App,
) -> AnyElement {
    let choices = dropdown.read(cx).choices.clone();
    let selected = dropdown.read(cx).selected;
    let motion_state = motion.read(cx);
    let closing = motion_state.closing();
    let hovered_item = motion_state.hovered_item();
    let animation_id = ElementId::NamedInteger(
        SharedString::from(format!("{id}-menu-motion")),
        motion_state.menu_revision(),
    );

    div()
        .w(CONTROL_WIDTH)
        .p(px(0.0))
        .occlude()
        .overflow_hidden()
        .bg(colors::base_950())
        .border_1()
        .border_color(colors::base_800())
        .rounded_md()
        .shadow_lg()
        .child(
            ScrollableColumn::new(
                SharedString::from(format!("{id}-menu-scroll")),
                px(280.0),
                div().w_full().flex().flex_col().gap(px(0.0)).children(
                    choices.into_iter().enumerate().map(|(index, choice)| {
                        let dropdown = dropdown.clone();
                        let close_motion = motion.clone();
                        let item_hovered = hovered_item == Some(index);
                        let item_animating = motion_state.item_animating(index);
                        let item_motion = motion.clone();
                        let item_animation_id = ElementId::NamedInteger(
                            SharedString::from(format!("{id}-option-{index}-hover")),
                            u64::from(item_hovered),
                        );
                        let resting_background = if index == selected {
                            colors::base_850()
                        } else {
                            colors::base_950()
                        };
                        let item_background = if item_animating {
                            div()
                                .absolute()
                                .inset_0()
                                .with_animation(
                                    item_animation_id,
                                    Animation::new(CONTROL_MOTION).with_easing(ease_in_out),
                                    move |element, delta| {
                                        let progress =
                                            if item_hovered { delta } else { 1.0 - delta };
                                        element.bg(mix_color(
                                            resting_background,
                                            colors::base_800(),
                                            progress,
                                        ))
                                    },
                                )
                                .into_any_element()
                        } else {
                            div()
                                .absolute()
                                .inset_0()
                                .bg(if item_hovered {
                                    colors::base_800()
                                } else {
                                    resting_background
                                })
                                .into_any_element()
                        };
                        div()
                            .id(SharedString::from(format!("{id}-option-{index}")))
                            .relative()
                            .w_full()
                            .h(CONTROL_HEIGHT)
                            .px_2()
                            .flex()
                            .items_center()
                            .gap(DROPDOWN_TRAILING_GUTTER)
                            .cursor_pointer()
                            .control_text()
                            .text_color(colors::base_200())
                            .on_hover(move |hovered, window, cx| {
                                set_dropdown_item_hovered(
                                    &item_motion,
                                    index,
                                    *hovered,
                                    window,
                                    cx,
                                );
                            })
                            .on_click(move |_, window, cx| {
                                dropdown.update(cx, |state, cx| {
                                    state.selected = index;
                                    cx.emit(AudioDropdownEvent);
                                    cx.notify();
                                });
                                set_dropdown_open(&close_motion, false, window, cx);
                            })
                            .child(item_background)
                            .child(choice_label(
                                format!("{id}-option-{index}-marquee"),
                                choice,
                                item_hovered,
                                DROPDOWN_LABEL_WIDTH,
                                MarqueeFade::new(
                                    resting_background,
                                    colors::base_800(),
                                    item_hovered,
                                    item_animating,
                                ),
                            ))
                            .when(index == selected, |element| {
                                element.child(
                                    svg()
                                        .path(resolve_asset_path("assets/icons/check.svg"))
                                        .size_4()
                                        .text_color(colors::orange()),
                                )
                            })
                            .into_any_element()
                    }),
                ),
            )
            .w_full(),
        )
        .with_animation(
            animation_id,
            Animation::new(MENU_MOTION).with_easing(ease_in_out),
            move |element, delta| {
                let progress = if closing { 1.0 - delta } else { delta };
                element.opacity(progress).mt(px(-4.0 * (1.0 - progress)))
            },
        )
        .into_any_element()
}

#[derive(IntoElement)]
struct DropdownTrigger {
    id: &'static str,
    choice: DropdownChoice,
    motion: Entity<DropdownMotion>,
}

impl DropdownTrigger {
    fn new(id: &'static str, choice: DropdownChoice, motion: Entity<DropdownMotion>) -> Self {
        Self { id, choice, motion }
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
            .gap(DROPDOWN_TRAILING_GUTTER)
            .flex_none()
            .cursor_pointer()
            .control_text()
            .text_color(colors::base_200())
            .on_hover(move |hovered, window, cx| {
                set_dropdown_hovered(&motion, *hovered, window, cx);
            })
            .child(surface)
            .child(border)
            .child(div().relative().min_w_0().flex_1().child(choice_label(
                format!("{}-trigger-marquee", self.id),
                self.choice,
                hovered,
                DROPDOWN_LABEL_WIDTH,
                MarqueeFade::new(
                    colors::base_900(),
                    colors::base_850(),
                    surface_active,
                    surface_animating,
                ),
            )))
            .child(chevron)
    }
}

fn choice_label(
    id: String,
    choice: DropdownChoice,
    marquee_active: bool,
    max_width: Pixels,
    fade: MarqueeFade,
) -> AudioChoiceLabel {
    AudioChoiceLabel {
        id: SharedString::from(id),
        choice,
        marquee_active,
        max_width,
        fade,
    }
}

#[derive(IntoElement)]
struct AudioChoiceLabel {
    id: SharedString,
    choice: DropdownChoice,
    marquee_active: bool,
    max_width: Pixels,
    fade: MarqueeFade,
}

impl RenderOnce for AudioChoiceLabel {
    fn render(self, window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let suffix_width = self
            .choice
            .muted_suffix
            .as_ref()
            .map_or(Pixels::ZERO, |suffix| control_text_width(suffix, window));
        let suffix_gap = if self.choice.muted_suffix.is_some() {
            px(4.0)
        } else {
            Pixels::ZERO
        };
        let label_width = (self.max_width - suffix_width - suffix_gap).max(px(24.0));
        let fade_id = SharedString::from(format!("{}-fade", self.id));

        div()
            .min_w_0()
            .flex_1()
            .h_full()
            .flex()
            .items_center()
            .gap_1()
            .child(
                MarqueeText::new(self.id, self.choice.label, label_width)
                    .active(self.marquee_active)
                    .fade_with_motion(fade_id, self.fade),
            )
            .when_some(self.choice.muted_suffix, |element, suffix| {
                element.child(
                    div()
                        .flex_none()
                        .text_color(colors::base_500())
                        .child(suffix),
                )
            })
    }
}

fn chevron_svg(progress: f32) -> Svg {
    svg()
        .path(resolve_asset_path("assets/icons/chevron-down.svg"))
        .size_4()
        .text_color(colors::base_500())
        .with_transformation(Transformation::rotate(Radians(
            std::f32::consts::PI * progress,
        )))
}
