use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Instant;

use gpui::prelude::*;
use gpui::*;

use super::PluginItem;
use crate::ui::components::badge::{BadgeStyle, badge};
use crate::ui::foundation::motion::{CONTROL_MOTION, MENU_MOTION, changed_recently, mix_color};
use crate::ui::foundation::{colors, i18n};

#[derive(Clone, Copy)]
struct Transition {
    revision: u64,
    changed_at: Instant,
}

#[derive(Default)]
pub struct PluginLibraryState {
    grouped_by_author: bool,
    mode_revision: u64,
    mode_changed_at: Option<Instant>,
    open_authors: HashSet<String>,
    closing_authors: HashSet<String>,
    author_transitions: HashMap<String, Transition>,
}

impl PluginLibraryState {
    pub fn grouped_by_author(&self) -> bool {
        self.grouped_by_author
    }

    pub fn mode_motion(&self) -> (u64, bool) {
        (
            self.mode_revision,
            changed_recently(self.mode_changed_at, CONTROL_MOTION),
        )
    }

    pub fn toggle_mode(&mut self, cx: &mut Context<Self>) -> bool {
        self.grouped_by_author = !self.grouped_by_author;
        self.mode_revision = self.mode_revision.wrapping_add(1);
        self.mode_changed_at = Some(Instant::now());
        cx.notify();
        self.grouped_by_author
    }

    fn author_visible(&self, author: &str) -> bool {
        self.open_authors.contains(author) || self.closing_authors.contains(author)
    }

    fn author_open(&self, author: &str) -> bool {
        self.open_authors.contains(author)
    }

    fn author_motion(&self, author: &str) -> (u64, bool) {
        self.author_transitions
            .get(author)
            .map_or((0, false), |transition| {
                (
                    transition.revision,
                    changed_recently(Some(transition.changed_at), MENU_MOTION),
                )
            })
    }
}

#[derive(Clone)]
pub(super) enum LibraryRow {
    Plugin(PluginItem),
    AuthorHeader(AuthorHeader),
    AuthorPlugin {
        author: String,
        plugin: PluginItem,
        closing: bool,
        last: bool,
        revision: u64,
        animating: bool,
    },
}

#[derive(Clone)]
pub(super) struct AuthorHeader {
    pub author: String,
    pub count: usize,
    pub open: bool,
    pub visible: bool,
    pub revision: u64,
    pub animating: bool,
}

pub(super) fn build_rows(
    plugins: &[PluginItem],
    state: &PluginLibraryState,
) -> Arc<Vec<LibraryRow>> {
    if !state.grouped_by_author {
        return Arc::new(plugins.iter().cloned().map(LibraryRow::Plugin).collect());
    }

    let mut rows = Vec::with_capacity(plugins.len());
    let mut start = 0;
    while start < plugins.len() {
        let author = plugins[start].vendor.clone();
        let mut end = start + 1;
        while end < plugins.len() && plugins[end].vendor.eq_ignore_ascii_case(&author) {
            end += 1;
        }

        let visible = state.author_visible(&author);
        let open = state.author_open(&author);
        let (revision, animating) = state.author_motion(&author);
        rows.push(LibraryRow::AuthorHeader(AuthorHeader {
            author: author.clone(),
            count: end - start,
            open,
            visible,
            revision,
            animating,
        }));
        if visible {
            rows.extend(
                plugins[start..end]
                    .iter()
                    .enumerate()
                    .map(|(index, plugin)| LibraryRow::AuthorPlugin {
                        author: author.clone(),
                        plugin: plugin.clone(),
                        closing: !open,
                        last: index + 1 == end - start,
                        revision,
                        animating,
                    }),
            );
        }
        start = end;
    }
    Arc::new(rows)
}

pub(super) fn set_author_open(
    state: &Entity<PluginLibraryState>,
    author: String,
    open: bool,
    window: &mut Window,
    cx: &mut App,
) {
    let revision = state.update(cx, |state, cx| {
        if open {
            state.open_authors.insert(author.clone());
            state.closing_authors.remove(&author);
        } else {
            state.open_authors.remove(&author);
            state.closing_authors.insert(author.clone());
        }
        let revision = state
            .author_transitions
            .get(&author)
            .map_or(1, |transition| transition.revision.wrapping_add(1));
        state.author_transitions.insert(
            author.clone(),
            Transition {
                revision,
                changed_at: Instant::now(),
            },
        );
        cx.notify();
        revision
    });
    window.refresh();

    if open {
        return;
    }
    let state = state.clone();
    cx.spawn(async move |cx| {
        cx.background_executor().timer(MENU_MOTION).await;
        state.update(cx, |state, cx| {
            let current_revision = state
                .author_transitions
                .get(&author)
                .map(|transition| transition.revision);
            if current_revision == Some(revision) && !state.open_authors.contains(&author) {
                state.closing_authors.remove(&author);
                cx.notify();
            }
        });
        cx.refresh();
    })
    .detach();
}

pub(super) fn render_author_header(
    header: AuthorHeader,
    state: Entity<PluginLibraryState>,
    cx: &App,
) -> AnyElement {
    let AuthorHeader {
        author,
        count,
        open,
        visible,
        revision,
        animating,
    } = header;
    let stable_author = format!("{}-{author}", author.len());
    let hover_key = SharedString::from(format!("plugin-author-{stable_author}-hover"));
    let hover = crate::ui::foundation::hover_motion::progress(&hover_key, cx);
    let title = if author.is_empty() {
        i18n::t("plugins.unknownAuthor")
    } else {
        author.clone()
    };
    let description = i18n::t("plugins.authorPluginCount").replace("%{count}", &count.to_string());
    let click_author = author.clone();
    let chevron = author_chevron(&stable_author, open, revision, animating);

    div()
        .id(SharedString::from(format!("plugin-author-{stable_author}")))
        .w_full()
        .h(super::virtualized::ROW_HEIGHT)
        .px_4()
        .flex()
        .flex_col()
        .cursor_pointer()
        .on_hover(move |hovered, window, cx| {
            crate::ui::foundation::hover_motion::set_hovered(
                hover_key.clone(),
                *hovered,
                window,
                cx,
            );
        })
        .on_click(move |_, window, cx| {
            set_author_open(&state, click_author.clone(), !open, window, cx);
        })
        .child(
            div()
                .h(super::virtualized::CARD_HEIGHT)
                .p_4()
                .flex_none()
                .bg(mix_color(colors::base_950(), colors::base_900(), hover))
                .border_1()
                .border_color(mix_color(colors::base_800(), colors::base_700(), hover))
                .when(!visible, |card| card.rounded_lg())
                .when(visible, |card| card.rounded_t(px(8.0)))
                .flex()
                .items_center()
                .justify_between()
                .gap_4()
                .child(
                    div()
                        .min_w_0()
                        .flex_1()
                        .flex()
                        .items_center()
                        .gap_4()
                        .child(
                            div()
                                .size(px(40.0))
                                .flex_none()
                                .flex()
                                .items_center()
                                .justify_center()
                                .bg(colors::base_900())
                                .border_1()
                                .border_color(colors::base_800())
                                .rounded_md()
                                .child(
                                    svg()
                                        .path(crate::ui::resolve_asset_path(
                                            "assets/icons/building.svg",
                                        ))
                                        .size_5()
                                        .text_color(colors::purple()),
                                ),
                        )
                        .child(
                            div()
                                .min_w_0()
                                .flex_1()
                                .flex()
                                .flex_col()
                                .gap(px(2.0))
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .gap_2()
                                        .child(
                                            div()
                                                .min_w_0()
                                                .truncate()
                                                .text_sm()
                                                .font_weight(FontWeight::BOLD)
                                                .text_color(colors::base_200())
                                                .child(title),
                                        )
                                        .child(badge(count.to_string(), BadgeStyle::Purple)),
                                )
                                .child(
                                    div()
                                        .truncate()
                                        .text_xs()
                                        .text_color(colors::base_500())
                                        .child(description),
                                ),
                        ),
                )
                .child(chevron),
        )
        .when(visible, |row| {
            row.child(
                div()
                    .h(px(12.0))
                    .bg(colors::base_950())
                    .border_l_1()
                    .border_r_1()
                    .border_color(colors::base_800()),
            )
        })
        .into_any_element()
}

pub(super) fn render_author_plugin_shell(
    author: &str,
    plugin_id: &str,
    closing: bool,
    last: bool,
    revision: u64,
    animating: bool,
    plugin_card: AnyElement,
) -> AnyElement {
    let stable_author = format!("{}-{author}", author.len());
    let content = div()
        .w_full()
        .h(super::virtualized::CARD_HEIGHT)
        .px_4()
        .relative()
        .top(px(-6.0))
        .child(plugin_card);
    let content = if animating {
        content
            .with_animation(
                ElementId::NamedInteger(
                    SharedString::from(format!("author-{stable_author}-plugin-{plugin_id}")),
                    revision,
                ),
                Animation::new(MENU_MOTION).with_easing(ease_in_out),
                move |element, delta| {
                    let progress = if closing { 1.0 - delta } else { delta };
                    element
                        .opacity(progress)
                        .top(px(-6.0 - 6.0 * (1.0 - progress)))
                },
            )
            .into_any_element()
    } else {
        content.into_any_element()
    };

    div()
        .w_full()
        .h(super::virtualized::ROW_HEIGHT)
        .px_4()
        .child(
            div()
                .w_full()
                .h(if last {
                    super::virtualized::CARD_HEIGHT
                } else {
                    super::virtualized::ROW_HEIGHT
                })
                .bg(colors::base_950())
                .border_l_1()
                .border_r_1()
                .when(last, |body| body.border_b_1().rounded_b_lg())
                .border_color(colors::base_800())
                .child(content),
        )
        .into_any_element()
}

fn author_chevron(author: &str, open: bool, revision: u64, animating: bool) -> AnyElement {
    let chevron = div().size_4().flex_none();
    if animating {
        chevron
            .with_animation(
                ElementId::NamedInteger(
                    SharedString::from(format!("plugin-author-{author}-chevron")),
                    revision,
                ),
                Animation::new(CONTROL_MOTION).with_easing(ease_in_out),
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
    }
}

fn chevron_svg(progress: f32) -> Svg {
    svg()
        .path(crate::ui::resolve_asset_path(
            "assets/icons/chevron-down.svg",
        ))
        .size_4()
        .text_color(colors::base_500())
        .with_transformation(Transformation::rotate(Radians(
            std::f32::consts::PI * progress,
        )))
}

#[cfg(test)]
mod tests {
    use super::{LibraryRow, PluginLibraryState, build_rows};
    use crate::ui::pages::plugins::PluginItem;

    fn plugin(id: &str, author: &str) -> PluginItem {
        PluginItem {
            id: id.into(),
            name: id.into(),
            vendor: author.into(),
            format: String::from("VST3"),
            path: String::new(),
            in_chain: false,
            initializing: false,
        }
    }

    #[test]
    fn collapsed_authors_only_emit_header_rows() {
        let mut state = PluginLibraryState {
            grouped_by_author: true,
            ..PluginLibraryState::default()
        };
        let plugins = vec![
            plugin("a", "Acme"),
            plugin("b", "Acme"),
            plugin("c", "Waves"),
        ];
        assert_eq!(build_rows(&plugins, &state).len(), 2);

        state.open_authors.insert(String::from("Acme"));
        let rows = build_rows(&plugins, &state);
        assert_eq!(rows.len(), 4);
        assert!(matches!(rows[1], LibraryRow::AuthorPlugin { .. }));
    }
}
