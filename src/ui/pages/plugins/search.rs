use gpui::prelude::*;
use gpui::*;

use super::PluginItem;
use crate::ui::components::audio_dropdown::CONTROL_HEIGHT;
use crate::ui::components::text_input::{TextInput, TextInputState};
use crate::ui::foundation::colors;
use crate::ui::foundation::control_style::ControlTypography;
use crate::ui::foundation::motion::mix_color;

const SEARCH_HOVER_KEY: &str = "plugins-search-hover";
pub(crate) const SEARCH_FOCUS_KEY: &str = "plugins-search-focus";

pub(super) fn render(search: &Entity<TextInputState>, window: &mut Window, cx: &App) -> AnyElement {
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
    let focus_handle = search.read(cx).focus_handle().clone();

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
        .cursor(CursorStyle::IBeam)
        .on_hover(move |hovered, window, cx| {
            crate::ui::foundation::hover_motion::set_hovered(
                hover_key.clone(),
                *hovered,
                window,
                cx,
            );
        })
        .on_mouse_down(MouseButton::Left, move |_, window, cx| {
            window.focus(&focus_handle, cx);
        })
        .child(
            div()
                .size_full()
                .px_2()
                .flex()
                .items_center()
                .gap_2()
                .control_text()
                .text_color(colors::base_200())
                .child(
                    svg()
                        .path(crate::ui::resolve_asset_path("assets/icons/search.svg"))
                        .size_4()
                        .flex_none()
                        .text_color(colors::base_500()),
                )
                .child(
                    div()
                        .min_w_0()
                        .h_full()
                        .flex_1()
                        .child(TextInput::new(search)),
                )
                .when(has_query, |input| {
                    input.child(
                        div()
                            .id("plugins-search-clear")
                            .size_5()
                            .flex()
                            .items_center()
                            .justify_center()
                            .cursor_pointer()
                            .child(
                                svg()
                                    .path(crate::ui::resolve_asset_path("assets/icons/x.svg"))
                                    .size_3()
                                    .text_color(colors::base_300()),
                            )
                            .on_click(move |_, window, cx| {
                                clear_search.update(cx, |search, cx| {
                                    search.set_value("", cx);
                                });
                                let focus_handle = clear_search.read(cx).focus_handle().clone();
                                window.focus(&focus_handle, cx);
                            }),
                    )
                }),
        )
        .into_any_element()
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
