use std::time::Duration;

use gpui::prelude::*;
use gpui::*;

use crate::ui::components::cursor_tooltip;
use crate::ui::foundation::control_style::ControlTypography;
use crate::ui::foundation::motion::{mix_color, refresh_rotation};
use crate::ui::foundation::{colors, i18n};
use crate::ui::resolve_asset_path;

#[derive(Clone, Copy)]
pub(super) enum IconButtonStyle {
    Outline,
    Primary,
}

pub(super) fn icon_button(
    id: impl Into<ElementId>,
    icon_path: &'static str,
    tooltip: impl Into<SharedString>,
    variant: IconButtonStyle,
    spinning: bool,
    disabled: bool,
    cx: &App,
) -> Stateful<Div> {
    let id = id.into();
    let hover_key = SharedString::from(format!("plugins-button-{id:?}"));
    let hover = if disabled {
        0.0
    } else {
        crate::ui::foundation::hover_motion::progress(&hover_key, cx)
    };
    let (background, border, foreground, hover_background, hover_border) = match variant {
        IconButtonStyle::Outline => (
            colors::base_900(),
            colors::base_800(),
            colors::base_200(),
            colors::base_850(),
            colors::base_700(),
        ),
        IconButtonStyle::Primary => (
            colors::orange(),
            colors::orange(),
            colors::accent_foreground(),
            colors::orange(),
            colors::accent_foreground().opacity(0.45),
        ),
    };
    let tooltip = tooltip.into();
    let icon = svg()
        .path(resolve_asset_path(icon_path))
        .size(px(17.0))
        .text_color(foreground);
    let icon = if spinning {
        icon.with_animation(
            "plugin-scan-spinner",
            Animation::new(Duration::from_millis(850)).repeat(),
            |icon, delta| icon.with_transformation(Transformation::rotate(refresh_rotation(delta))),
        )
        .into_any_element()
    } else {
        icon.into_any_element()
    };

    let button = div()
        .id(id.clone())
        .size(px(34.0))
        .flex_none()
        .flex()
        .items_center()
        .justify_center()
        .bg(mix_color(background, hover_background, hover))
        .border_1()
        .border_color(mix_color(border, hover_border, hover))
        .rounded_md()
        .child(icon)
        .when(disabled, |button| button.cursor_default().opacity(0.5))
        .when(!disabled, |button| button.cursor_pointer());
    cursor_tooltip::attach_with_hover_motion(button, id, hover_key, tooltip)
}

pub(super) fn chain_navigation_button(id: impl Into<ElementId>, cx: &App) -> Stateful<Div> {
    let id = id.into();
    let hover_key = SharedString::from(format!("plugins-chain-button-{id:?}"));
    let pressed_hover_key = hover_key.clone();
    let hover = crate::ui::foundation::hover_motion::progress(&hover_key, cx);
    div()
        .id(id)
        .h(px(34.0))
        .px_3()
        .flex_none()
        .flex()
        .items_center()
        .justify_center()
        .gap_2()
        .cursor_pointer()
        .bg(colors::orange())
        .border_1()
        .border_color(mix_color(
            colors::orange(),
            colors::accent_foreground().opacity(0.45),
            hover,
        ))
        .rounded_md()
        .control_text()
        .text_color(colors::accent_foreground())
        .on_hover(move |hovered, window, cx| {
            crate::ui::foundation::hover_motion::set_hovered(
                hover_key.clone(),
                *hovered,
                window,
                cx,
            );
        })
        .on_mouse_down(MouseButton::Left, move |_, window, cx| {
            crate::ui::foundation::hover_motion::clear_hover(&pressed_hover_key, window, cx);
        })
        .child(i18n::t("plugins.goToChain"))
        .child(
            svg()
                .path(resolve_asset_path("assets/icons/arrow-right.svg"))
                .size_4()
                .text_color(colors::accent_foreground()),
        )
}
