use gpui::prelude::*;
use gpui::*;

use super::MainView;
use crate::ui::foundation::colors;
use crate::ui::foundation::motion::mix_color;
use crate::ui::shell::navigation::NavigationItem;

impl MainView {
    pub(super) fn render_nav_item(
        &self,
        item: &NavigationItem,
        collapsed: bool,
        sidebar_progress: f32,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let route = item.route;
        let is_selected = self.current_route == route;
        let is_hovered = self.hovered_route == Some(route) && !is_selected;
        let is_unhovered = self.unhovered_route == Some(route) && !is_selected;
        let item_label = SharedString::from(item.label());
        let tooltip_source = ElementId::Name(format!("{}-collapsed-tooltip", item.id).into());
        let hover_tooltip_source = tooltip_source.clone();
        let pressed_tooltip_source = tooltip_source.clone();
        let tooltip_label = item_label.clone();

        let selected_alpha = if is_selected {
            let elapsed = self.selected_at.elapsed().as_secs_f32();
            (elapsed / 0.15).clamp(0.0, 1.0)
        } else if self.deselected_route == Some(route) {
            if let Some(at) = self.deselected_at {
                (1.0 - at.elapsed().as_secs_f32() / 0.15).clamp(0.0, 1.0)
            } else {
                0.0
            }
        } else {
            0.0
        };

        let hover_alpha = if is_hovered {
            if let Some(at) = self.hovered_at {
                (at.elapsed().as_secs_f32() / 0.15).clamp(0.0, 1.0)
            } else {
                1.0
            }
        } else if is_unhovered {
            if let Some(at) = self.unhovered_at {
                (1.0 - at.elapsed().as_secs_f32() / 0.15).clamp(0.0, 1.0)
            } else {
                0.0
            }
        } else {
            0.0
        };

        let active_t = if selected_alpha > 0.001 {
            selected_alpha
        } else {
            hover_alpha
        };

        let target_foreground = if selected_alpha > 0.001 {
            colors::accent_foreground()
        } else {
            colors::orange()
        };
        let icon_color = mix_color(colors::base_200(), target_foreground, active_t);

        if (selected_alpha > 0.0 && selected_alpha < 1.0)
            || (hover_alpha > 0.0 && hover_alpha < 1.0)
        {
            cx.on_next_frame(window, |_, _, cx| cx.notify());
        }

        div()
            .id(item.id)
            .relative()
            .h(px(32.0))
            .w_full()
            .flex()
            .flex_row()
            .items_center()
            .px(px(8.0))
            .gap(px(8.0))
            .rounded_md()
            .cursor_pointer()
            .when(selected_alpha > 0.001, |this| {
                this.bg(colors::orange().opacity(selected_alpha))
            })
            .when(selected_alpha <= 0.001 && hover_alpha > 0.001, |this| {
                this.bg(colors::orange().opacity(hover_alpha * 0.16))
            })
            .on_hover(cx.listener(move |this, is_hovered, window, cx| {
                this.set_hovered_route(route, *is_hovered, cx);
                if collapsed {
                    crate::ui::components::cursor_tooltip::set_hovered(
                        hover_tooltip_source.clone(),
                        tooltip_label.clone(),
                        *is_hovered,
                        window,
                        cx,
                    );
                } else {
                    crate::ui::components::cursor_tooltip::hide_source(
                        &hover_tooltip_source,
                        window,
                        cx,
                    );
                }
            }))
            .on_mouse_down(MouseButton::Left, move |_, window, cx| {
                crate::ui::components::cursor_tooltip::hide_source(
                    &pressed_tooltip_source,
                    window,
                    cx,
                );
            })
            .on_click(cx.listener(move |this, _, window, cx| {
                this.navigate(route, window, cx);
            }))
            .child(
                div()
                    .relative()
                    .flex_shrink_0()
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(
                        svg()
                            .external_path(crate::ui::resolve_asset_path(item.icon_path))
                            .size_4()
                            .text_color(icon_color),
                    ),
            )
            .child(
                div()
                    .relative()
                    .flex_1()
                    .text_sm()
                    .text_color(icon_color.opacity(sidebar_progress))
                    .ml(px(-14.0 * (1.0 - sidebar_progress)))
                    .truncate()
                    .overflow_hidden()
                    .child(item_label),
            )
            .into_any_element()
    }
}
