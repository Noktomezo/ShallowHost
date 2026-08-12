use gpui::prelude::*;
use gpui::*;
use std::rc::Rc;
use std::time::Instant;

use super::{Language, ThemeMode, card, resolve_path, row, separator, setting_copy};
use crate::ui::components::card_header::{card_header_layout, card_heading};
use crate::ui::components::dropdown_overlay::adaptive_dropdown;
use crate::ui::components::marquee_text::{MarqueeFade, MarqueeText};
use crate::ui::components::toggle_switch::toggle_switch;
use crate::ui::foundation::colors;
use crate::ui::foundation::control_style::{
    ControlTypography, DROPDOWN_CONTROL_HEIGHT, DROPDOWN_CONTROL_WIDTH, DROPDOWN_ICON_LABEL_WIDTH,
    DROPDOWN_MENU_ICON_LABEL_WIDTH, DROPDOWN_TRAILING_GUTTER,
};
use crate::ui::foundation::motion::{
    CONTROL_MOTION, DropdownMotion, MENU_MOTION, mix_color, set_dropdown_item_hovered,
    set_dropdown_open,
};
use crate::ui::shell::routes::{LanguageCallback, ThemeCallback, TransparencyCallback};

const CONTROL_HEIGHT: Pixels = DROPDOWN_CONTROL_HEIGHT;
const CONTROL_WIDTH: Pixels = DROPDOWN_CONTROL_WIDTH;

#[derive(Clone, Copy)]
enum AppearanceValue {
    Theme(ThemeMode),
    Language(Language),
}

#[derive(Clone)]
pub(super) struct AppearanceOption {
    value: AppearanceValue,
    pub(super) label: SharedString,
    pub(super) icon_path: &'static str,
    pub(super) uses_flag: bool,
}

type SelectCallback = Rc<dyn Fn(AppearanceValue, &mut Window, &mut App)>;

pub(super) struct AppearanceCardProps {
    pub selected_theme: ThemeMode,
    pub selected_language: Language,
    pub transparent_shell: bool,
    pub transparency_changed_at: Option<Instant>,
    pub on_change_theme: ThemeCallback,
    pub on_change_language: LanguageCallback,
    pub on_change_transparency: TransparencyCallback,
    pub theme_motion: Entity<DropdownMotion>,
    pub language_motion: Entity<DropdownMotion>,
}

pub(super) fn appearance_card(props: AppearanceCardProps, cx: &App) -> AnyElement {
    card()
        .child(card_header_layout().child(card_heading(
            "assets/icons/palette.svg",
            colors::yellow(),
            "settings.appearance",
            "settings.appearanceDescription",
        )))
        .child(separator())
        .child(
            div()
                .p_4()
                .flex()
                .flex_col()
                .gap_4()
                .child(
                    row()
                        .px_0()
                        .py_0()
                        .child(setting_copy("settings.theme", "settings.themeDescription"))
                        .child(theme_dropdown(
                            props.selected_theme,
                            props.on_change_theme,
                            props.theme_motion,
                            cx,
                        )),
                )
                .child(
                    row()
                        .px_0()
                        .py_0()
                        .child(setting_copy(
                            "settings.language",
                            "settings.languageDescription",
                        ))
                        .child(language_dropdown(
                            props.selected_language,
                            props.on_change_language,
                            props.language_motion,
                            cx,
                        )),
                )
                .child(transparency_row(
                    props.transparent_shell,
                    props.transparency_changed_at,
                    props.on_change_transparency,
                )),
        )
        .into_any_element()
}

fn theme_dropdown(
    selected: ThemeMode,
    callback: ThemeCallback,
    motion: Entity<DropdownMotion>,
    cx: &App,
) -> AnyElement {
    let options = ThemeMode::all()
        .iter()
        .copied()
        .map(|theme| AppearanceOption {
            value: AppearanceValue::Theme(theme),
            label: theme.label().into(),
            icon_path: theme.icon_path(),
            uses_flag: false,
        })
        .collect::<Vec<_>>();
    let selected_index = ThemeMode::all()
        .iter()
        .position(|theme| *theme == selected)
        .unwrap_or_default();
    let on_select = Rc::new(move |value, window: &mut Window, cx: &mut App| {
        if let AppearanceValue::Theme(theme) = value {
            callback(theme, window, cx);
        }
    });
    appearance_dropdown(
        "theme-dropdown",
        options,
        selected_index,
        on_select,
        motion,
        cx,
    )
}

fn language_dropdown(
    selected: Language,
    callback: LanguageCallback,
    motion: Entity<DropdownMotion>,
    cx: &App,
) -> AnyElement {
    let options = Language::all()
        .iter()
        .copied()
        .map(|language| AppearanceOption {
            value: AppearanceValue::Language(language),
            label: language.label().into(),
            icon_path: language.icon_path(),
            uses_flag: language.uses_flag(),
        })
        .collect::<Vec<_>>();
    let selected_index = Language::all()
        .iter()
        .position(|language| *language == selected)
        .unwrap_or_default();
    let on_select = Rc::new(move |value, window: &mut Window, cx: &mut App| {
        if let AppearanceValue::Language(language) = value {
            callback(language, window, cx);
        }
    });
    appearance_dropdown(
        "language-dropdown",
        options,
        selected_index,
        on_select,
        motion,
        cx,
    )
}

fn appearance_dropdown(
    id: &'static str,
    options: Vec<AppearanceOption>,
    selected_index: usize,
    on_select: SelectCallback,
    motion: Entity<DropdownMotion>,
    cx: &App,
) -> AnyElement {
    let selected = options
        .get(selected_index)
        .cloned()
        .or_else(|| options.first().cloned());
    let trigger = super::appearance_trigger::DropdownTrigger::new(id, selected, motion.clone());
    let menu_motion = motion.clone();
    let menu = render_menu(id, &options, selected_index, &on_select, &menu_motion, cx);

    adaptive_dropdown(id, trigger, menu, motion, cx)
}

fn render_menu(
    id: &'static str,
    options: &[AppearanceOption],
    selected_index: usize,
    on_select: &SelectCallback,
    motion: &Entity<DropdownMotion>,
    cx: &App,
) -> AnyElement {
    let motion_state = motion.read(cx);
    let closing = motion_state.closing();
    let hovered_item = motion_state.hovered_item();
    let animation_id = ElementId::NamedInteger(
        SharedString::from(format!("{id}-menu-motion")),
        motion_state.menu_revision(),
    );
    div()
        .id(SharedString::from(format!("{id}-menu-scroll")))
        .w(CONTROL_WIDTH)
        .max_h(px(340.0))
        .overflow_y_scroll()
        .p(px(0.0))
        .flex()
        .flex_col()
        .gap(px(0.0))
        .occlude()
        .bg(colors::base_950())
        .border_1()
        .border_color(colors::base_800())
        .rounded_md()
        .shadow_lg()
        .children(options.iter().cloned().enumerate().map(|(index, option)| {
            let on_select = on_select.clone();
            let close_motion = motion.clone();
            let item_hovered = hovered_item == Some(index);
            let item_animating = motion_state.item_animating(index);
            let item_motion = motion.clone();
            let item_animation_id = ElementId::NamedInteger(
                SharedString::from(format!("{id}-option-{index}-hover")),
                u64::from(item_hovered),
            );
            let resting_background = if index == selected_index {
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
                            let progress = if item_hovered { delta } else { 1.0 - delta };
                            element.bg(mix_color(resting_background, colors::base_800(), progress))
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
                    set_dropdown_item_hovered(&item_motion, index, *hovered, window, cx);
                })
                .on_click(move |_, window, cx| {
                    on_select(option.value, window, cx);
                    set_dropdown_open(&close_motion, false, window, cx);
                })
                .child(item_background)
                .child(
                    div()
                        .flex()
                        .flex_1()
                        .min_w_0()
                        .h_full()
                        .items_center()
                        .gap_2()
                        .child(local_icon(option.icon_path, option.uses_flag))
                        .child(
                            MarqueeText::new(
                                SharedString::from(format!("{id}-option-{index}-marquee")),
                                option.label,
                                if index == selected_index {
                                    DROPDOWN_ICON_LABEL_WIDTH
                                } else {
                                    DROPDOWN_MENU_ICON_LABEL_WIDTH
                                },
                            )
                            .active(item_hovered)
                            .fade_with_motion(
                                SharedString::from(format!("{id}-option-{index}-fade")),
                                MarqueeFade::new(
                                    resting_background,
                                    colors::base_800(),
                                    item_hovered,
                                    item_animating,
                                ),
                            ),
                        ),
                )
                .when(index == selected_index, |element| {
                    element.child(
                        svg()
                            .path(resolve_path("assets/icons/check.svg"))
                            .size_4()
                            .text_color(colors::orange()),
                    )
                })
                .into_any_element()
        }))
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

fn transparency_row(
    checked: bool,
    changed_at: Option<Instant>,
    callback: TransparencyCallback,
) -> AnyElement {
    row()
        .id("appearance-transparency")
        .px_0()
        .py_0()
        .cursor_pointer()
        .on_click(move |_, window, cx| callback(!checked, window, cx))
        .child(setting_copy(
            "settings.transparency",
            "settings.transparencyDescription",
        ))
        .child(toggle_switch(
            "appearance-transparency",
            checked,
            true,
            changed_at,
        ))
        .into_any_element()
}

pub(super) fn local_icon(path: &'static str, uses_flag: bool) -> AnyElement {
    if uses_flag {
        img(resolve_path(path))
            .w(px(18.0))
            .h(px(12.0))
            .rounded(px(2.0))
            .into_any_element()
    } else {
        svg()
            .path(resolve_path(path))
            .size_4()
            .text_color(colors::base_500())
            .into_any_element()
    }
}
