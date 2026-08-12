use std::sync::Arc;

use gpui::prelude::*;
use gpui::*;

use super::card::render_plugin_card;
use super::controls::{IconButtonStyle, icon_button, library_mode_button};
use super::grouped::{self, LibraryRow, PluginLibraryState};
use super::search;
use super::{PluginItem, PluginScanState};
use crate::infrastructure::config::PluginSettings;
use crate::infrastructure::engine::Engine;
use crate::ui::components::badge::{BadgeStyle, badge, loading_badge};
use crate::ui::components::cursor_tooltip;
use crate::ui::components::smooth_scroll::{
    PageScrollbar, SmoothUniformListScroll, SmoothVerticalScroll,
};
use crate::ui::components::text_input::TextInputState;
use crate::ui::foundation::colors;
use crate::ui::foundation::i18n;
use crate::ui::foundation::plugin_format;
use crate::ui::shell::routes::{DropdownCallbacks, NavigateCallback};
use crate::ui::state::chain_operations::ChainOperationState;

// 40 px content + 32 px card padding + 2 px border + 12 px inter-row spacing.
// Keep every virtual row at this exact height: uniform_list measures one row
// and computes the complete scrollbar extent before off-screen rows are rendered.
pub(super) const CARD_HEIGHT: Pixels = px(74.0);
pub(super) const ROW_HEIGHT: Pixels = px(86.0);

#[derive(Clone)]
pub(super) struct HeaderContext {
    engine: Arc<Engine>,
    settings: PluginSettings,
    callbacks: DropdownCallbacks,
    scan_state: Entity<PluginScanState>,
    search: Entity<TextInputState>,
    library_state: Entity<PluginLibraryState>,
    wheel_enabled: bool,
}

impl HeaderContext {
    pub(super) fn new(
        engine: Arc<Engine>,
        settings: PluginSettings,
        callbacks: DropdownCallbacks,
        scan_state: Entity<PluginScanState>,
        search: Entity<TextInputState>,
        library_state: Entity<PluginLibraryState>,
        wheel_enabled: bool,
    ) -> Self {
        Self {
            engine,
            settings,
            callbacks,
            scan_state,
            search,
            library_state,
            wheel_enabled,
        }
    }

    fn render(
        &self,
        format_counts: plugin_format::Counts,
        window: &mut Window,
        cx: &App,
    ) -> AnyElement {
        let plugin_count = format_counts.total();
        let open_scan_paths = self.callbacks.on_set_scan_paths_open.clone();
        let scan_engine = Arc::clone(&self.engine);
        let plugin_settings = self.settings.clone();
        let scan_status = self.scan_state.read(cx);
        let scanning = scan_status.scanning;
        let scan_progress = scan_status.progress;
        let scan_state = self.scan_state.clone();
        let search_control = search::render(&self.search, window, cx);
        let grouped_by_author = self.library_state.read(cx).grouped_by_author();
        let (mode_revision, mode_animating) = self.library_state.read(cx).mode_motion();
        let mode_state = self.library_state.clone();
        let mode_search = self.search.clone();

        div()
            .w_full()
            .flex()
            .flex_row()
            .items_center()
            .justify_between()
            .child(
                div()
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
                                    .text_xl()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(colors::base_200())
                                    .child(i18n::t("plugins.title")),
                            )
                            .child(cursor_tooltip::attach(
                                div().id("plugins-format-counts-tooltip").child(badge(
                                    plugin_count.to_string(),
                                    if plugin_count == 0 {
                                        BadgeStyle::Red
                                    } else {
                                        BadgeStyle::Purple
                                    },
                                )),
                                ElementId::Name("plugins-format-counts-tooltip".into()),
                                format!(
                                    "VST2: {}\nVST3: {}",
                                    format_counts.vst2, format_counts.vst3
                                ),
                            ))
                            .when(scanning, |title_row| {
                                title_row.child(loading_badge(format!(
                                    "{:.0}%",
                                    scan_progress.clamp(0.0, 1.0) * 100.0
                                )))
                            }),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(colors::base_500())
                            .child(i18n::t("plugins.description")),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_2()
                    .child(search_control)
                    .child(
                        library_mode_button(grouped_by_author, mode_revision, mode_animating, cx)
                            .on_click(move |_, window, cx| {
                                let grouped_by_author =
                                    mode_state.update(cx, |state, cx| state.toggle_mode(cx));
                                mode_search.update(cx, |search, cx| {
                                    search.set_value("", cx);
                                    search.set_placeholder(
                                        i18n::t(if grouped_by_author {
                                            "plugins.searchAuthors"
                                        } else {
                                            "plugins.search"
                                        }),
                                        cx,
                                    );
                                });
                                window.refresh();
                            }),
                    )
                    .child(
                        icon_button(
                            "btn-scan-paths",
                            "assets/icons/folder-cog.svg",
                            i18n::t("plugins.scanPathsTitle"),
                            IconButtonStyle::Outline,
                            false,
                            false,
                            cx,
                        )
                        .on_click(move |_, window, cx| open_scan_paths(true, window, cx)),
                    )
                    .child(
                        icon_button(
                            "btn-scan-now",
                            "assets/icons/refresh-cw.svg",
                            i18n::t("plugins.scan"),
                            IconButtonStyle::Primary,
                            scanning,
                            scanning,
                            cx,
                        )
                        .on_click(move |_, window, cx| {
                            if scan_state.read(cx).scanning {
                                return;
                            }
                            scan_state.update(cx, |state, cx| {
                                state.scanning = true;
                                state.progress = 0.0;
                                cx.notify();
                            });
                            window.refresh();
                            let engine = Arc::clone(&scan_engine);
                            let settings = plugin_settings.clone();
                            let scan_state = scan_state.clone();
                            cx.spawn(async move |cx| {
                                let start_engine = Arc::clone(&engine);
                                let mut result = cx
                                    .background_spawn(async move {
                                        start_engine.start_plugin_scan(&settings)
                                    })
                                    .await;

                                loop {
                                    let done = match result {
                                        Ok(step) => {
                                            let done = step.done;
                                            scan_state.update(cx, |state, cx| {
                                                state.progress = step.progress;
                                                cx.notify();
                                            });
                                            cx.refresh();
                                            done
                                        }
                                        Err(error) => {
                                            eprintln!("JUCE plugin scan failed: {error}");
                                            break;
                                        }
                                    };
                                    if done {
                                        break;
                                    }

                                    let next_engine = Arc::clone(&engine);
                                    result = cx
                                        .background_spawn(
                                            async move { next_engine.scan_next_plugin() },
                                        )
                                        .await;
                                }

                                scan_state.update(cx, |state, cx| {
                                    state.scanning = false;
                                    state.progress = 0.0;
                                    cx.notify();
                                });
                                cx.refresh();
                            })
                            .detach();
                        }),
                    ),
            )
            .into_any_element()
    }
}

struct VirtualListState {
    scroll: UniformListScrollHandle,
}

pub(super) fn render(
    window: &mut Window,
    cx: &mut App,
    header: HeaderContext,
    plugins: Arc<Vec<PluginItem>>,
    engine: Arc<Engine>,
    on_navigate: NavigateCallback,
    chain_operations: Entity<ChainOperationState>,
) -> AnyElement {
    // The header is row zero, so it participates in the same viewport and scroll
    // position as the cards instead of introducing a nested scrolling region.
    let format_counts = plugin_format::counts(plugins.iter().map(|plugin| plugin.format.as_str()));
    if header.library_state.read(cx).grouped_by_author() {
        return render_grouped(
            window,
            cx,
            header,
            plugins,
            engine,
            on_navigate,
            chain_operations,
        );
    }

    let item_count = plugins.len() + 1;
    let state = window
        .use_keyed_state("plugins-virtual-list", cx, |_, _| VirtualListState {
            scroll: UniformListScrollHandle::new(),
        })
        .clone();
    let scroll_handle = state.read(cx).scroll.clone();
    let render_plugins = Arc::clone(&plugins);
    let card_scan_state = header.scan_state.clone();
    let wheel_enabled = header.wheel_enabled;
    let content = uniform_list(
        "plugins-uniform-list",
        item_count,
        move |range, window, cx| {
            range
                .map(|row| {
                    if row == 0 {
                        return div()
                            .w_full()
                            .h(ROW_HEIGHT)
                            .px_4()
                            .pt_4()
                            .pb_3()
                            .child(header.render(format_counts, window, cx))
                            .into_any_element();
                    }

                    let index = row - 1;
                    let Some(plugin) = render_plugins.get(index).cloned() else {
                        return div().h(ROW_HEIGHT).into_any_element();
                    };
                    div()
                        .w_full()
                        .h(ROW_HEIGHT)
                        .px_4()
                        .pb_3()
                        .child(render_plugin_card(
                            plugin,
                            Arc::clone(&engine),
                            on_navigate.clone(),
                            chain_operations.clone(),
                            card_scan_state.clone(),
                            cx,
                        ))
                        .into_any_element()
                })
                .collect::<Vec<_>>()
        },
    )
    .track_scroll(&scroll_handle);

    div()
        .relative()
        .size_full()
        .child(
            SmoothUniformListScroll::new(
                "plugins-virtual-list-smooth-scroll",
                scroll_handle.clone(),
                content.size_full(),
            )
            .wheel_enabled(wheel_enabled),
        )
        .child(PageScrollbar::new(
            "plugins-virtual-list-scrollbar",
            scroll_handle,
        ))
        .into_any_element()
}

fn render_grouped(
    window: &mut Window,
    cx: &mut App,
    header: HeaderContext,
    plugins: Arc<Vec<PluginItem>>,
    engine: Arc<Engine>,
    on_navigate: NavigateCallback,
    chain_operations: Entity<ChainOperationState>,
) -> AnyElement {
    let library_state = header.library_state.clone();
    let scan_state = header.scan_state.clone();
    let wheel_enabled = header.wheel_enabled;
    let rows = grouped::build_rows(&plugins, library_state.read(cx));
    let format_counts = plugin_format::counts(plugins.iter().map(|plugin| plugin.format.as_str()));
    let mut content = div().w_full().flex().flex_col().child(
        div()
            .w_full()
            .h(ROW_HEIGHT)
            .px_4()
            .pt_4()
            .pb_3()
            .child(header.render(format_counts, window, cx)),
    );

    // ponytail: collapsed author groups keep this tree small; switch this branch to
    // GPUI ListState only if users routinely expand enough groups to regress rendering.
    for row in rows.iter().cloned() {
        let element = match row {
            LibraryRow::AuthorHeader(header) => {
                grouped::render_author_header(header, library_state.clone(), cx)
            }
            LibraryRow::AuthorPlugin {
                author,
                plugin,
                first,
                closing,
                last,
                revision,
                animating,
            } => {
                let plugin_id = plugin.id.clone();
                let plugin_card = render_plugin_card(
                    plugin,
                    Arc::clone(&engine),
                    on_navigate.clone(),
                    chain_operations.clone(),
                    scan_state.clone(),
                    cx,
                );
                grouped::render_author_plugin_shell(
                    &author,
                    &plugin_id,
                    grouped::AuthorPluginLayout {
                        first,
                        closing,
                        last,
                        revision,
                        animating,
                    },
                    plugin_card,
                )
            }
        };
        content = content.child(element);
    }

    SmoothVerticalScroll::new("plugins-grouped-scroll", content)
        .wheel_enabled(wheel_enabled)
        .into_any_element()
}
