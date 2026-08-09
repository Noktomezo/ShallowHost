use std::time::Duration;

use gpui::prelude::*;
use gpui::*;
use gpui_component::Sizable as _;
use gpui_component::input::{Input, InputState};

use super::PluginItem;
use crate::ui::components::audio_dropdown::CONTROL_HEIGHT;
use crate::ui::foundation::colors;
use crate::ui::foundation::control_style::ControlTypography;
use crate::ui::foundation::motion::mix_color;

const SEARCH_HOVER_KEY: &str = "plugins-search-hover";
pub(crate) const SEARCH_FOCUS_KEY: &str = "plugins-search-focus";
const CARET_BLINK_DURATION: Duration = Duration::from_millis(1_000);
const CARET_WIDTH: Pixels = px(2.0);

pub(super) fn render(search: &Entity<InputState>, window: &mut Window, cx: &App) -> AnyElement {
    let has_query = !search.read(cx).value().is_empty();
    let focused = search.focus_handle(cx).is_focused(window);
    let hover_key = SharedString::from(SEARCH_HOVER_KEY);
    let hover = crate::ui::foundation::hover_motion::progress(&hover_key, cx);
    let focus = crate::ui::foundation::hover_motion::state_progress(
        &SharedString::from(SEARCH_FOCUS_KEY),
        focused,
        cx,
    );
    let resting_border = mix_color(colors::base_800(), colors::base_700(), hover);
    let clear_search = search.clone();

    div()
        .id("plugins-search-control")
        .relative()
        .w(px(190.0))
        .h(CONTROL_HEIGHT)
        .flex_none()
        .overflow_hidden()
        .bg(mix_color(
            colors::base_900(),
            colors::base_850(),
            hover.max(focus),
        ))
        .border_1()
        .border_color(mix_color(resting_border, colors::orange(), focus))
        .rounded_md()
        .on_hover(move |hovered, window, cx| {
            crate::ui::foundation::hover_motion::set_hovered(
                hover_key.clone(),
                *hovered,
                window,
                cx,
            );
        })
        .on_mouse_down_out(|_, window, _| window.blur())
        .child(
            Input::new(search)
                .small()
                .size_full()
                .gap_2()
                .appearance(false)
                .focus_bordered(false)
                .control_text()
                .text_color(colors::base_200())
                .prefix(
                    svg()
                        .external_path(crate::ui::resolve_asset_path("assets/icons/search.svg"))
                        .size_4()
                        .text_color(colors::base_500()),
                )
                .when(has_query, |input| {
                    input.suffix(
                        div()
                            .id("plugins-search-clear")
                            .size_5()
                            .flex()
                            .items_center()
                            .justify_center()
                            .cursor_pointer()
                            .child(
                                svg()
                                    .external_path(crate::ui::resolve_asset_path(
                                        "assets/icons/x.svg",
                                    ))
                                    .size_3()
                                    .text_color(colors::base_300()),
                            )
                            .on_click(move |_, window, cx| {
                                clear_search.update(cx, |search, cx| {
                                    search.set_value("", window, cx);
                                });
                            }),
                    )
                }),
        )
        .when(focused, |control| {
            control.child(render_caret(search.clone()))
        })
        .into_any_element()
}

fn render_caret(search: Entity<InputState>) -> AnyElement {
    div()
        .absolute()
        .inset_0()
        .with_animation(
            "plugins-search-caret-blink",
            Animation::new(CARET_BLINK_DURATION).repeat(),
            |caret, delta| caret.opacity(caret_opacity(delta)),
        )
        .child(canvas(
            |_, _, _| {},
            move |_, _, window, cx| {
                let state = search.read(cx);
                let selected_range = state.selected_range();
                if !selected_range.is_empty() {
                    return;
                }

                let Some(bounds) = state.range_to_bounds(&selected_range) else {
                    return;
                };
                let bounds = Bounds::new(bounds.origin, size(CARET_WIDTH, bounds.size.height));
                window.paint_quad(fill(bounds, colors::orange()));
            },
        ))
        .into_any_element()
}

fn caret_opacity(progress: f32) -> f32 {
    0.5 + 0.5 * (std::f32::consts::TAU * progress.clamp(0.0, 1.0)).cos()
}

pub(super) fn matches_plugin(plugin: &PluginItem, query: &str) -> bool {
    let query = query.trim().to_lowercase();
    if query.is_empty() {
        return true;
    }

    let fields = [plugin.name.to_lowercase(), plugin.vendor.to_lowercase()];
    query.split_whitespace().all(|query_word| {
        fields.iter().any(|field| {
            field.contains(query_word)
                || field
                    .split(|character: char| !character.is_alphanumeric())
                    .filter(|word| !word.is_empty())
                    .any(|word| {
                        strsim::levenshtein(query_word, word)
                            <= fuzzy_distance_limit(query_word.chars().count())
                    })
        })
    })
}

const fn fuzzy_distance_limit(query_length: usize) -> usize {
    match query_length {
        0..=2 => 0,
        3..=5 => 1,
        6..=8 => 2,
        _ => 3,
    }
}

pub(crate) fn reset_interaction(window: &mut Window, cx: &mut App) {
    window.blur();
    crate::ui::foundation::hover_motion::clear_hover(
        &SharedString::from(SEARCH_HOVER_KEY),
        window,
        cx,
    );
}

#[cfg(test)]
mod tests {
    use super::caret_opacity;

    #[test]
    fn caret_fades_out_and_back_in() {
        assert!((caret_opacity(0.0) - 1.0).abs() < f32::EPSILON);
        assert!(caret_opacity(0.5).abs() < f32::EPSILON);
        assert!((caret_opacity(1.0) - 1.0).abs() < f32::EPSILON);
    }
}
