use std::sync::Arc;

use gpui::prelude::*;
use gpui::*;
use gpui_component::StyledExt;

use super::controls::{IconButtonStyle, chain_navigation_button, icon_button};
use super::{PluginItem, PluginScanState};
use crate::infrastructure::config::PluginSettings;
use crate::infrastructure::engine::Engine;
use crate::ui::components::badge::{BadgeStyle, badge, loading_badge};
use crate::ui::components::smooth_scroll::{PageScrollbar, SmoothUniformListScroll};
use crate::ui::foundation::colors;
use crate::ui::foundation::i18n;
use crate::ui::shell::routes::{DropdownCallbacks, NavigateCallback, Route};
use crate::ui::state::chain_operations::{self, ChainOperationState, PendingPlugin};

// 40 px content + 32 px card padding + 12 px inter-row spacing.
// Keep every virtual row at this exact height: uniform_list measures one row
// and computes the complete scrollbar extent before off-screen rows are rendered.
const CARD_HEIGHT: Pixels = px(72.0);
const ROW_HEIGHT: Pixels = px(84.0);

#[derive(Clone)]
pub(super) struct HeaderContext {
    engine: Arc<Engine>,
    settings: PluginSettings,
    callbacks: DropdownCallbacks,
    scan_state: Entity<PluginScanState>,
}

impl HeaderContext {
    pub(super) fn new(
        engine: Arc<Engine>,
        settings: PluginSettings,
        callbacks: DropdownCallbacks,
        scan_state: Entity<PluginScanState>,
    ) -> Self {
        Self {
            engine,
            settings,
            callbacks,
            scan_state,
        }
    }

    fn render(&self, plugin_count: usize, cx: &App) -> AnyElement {
        let open_scan_paths = self.callbacks.on_set_scan_paths_open.clone();
        let scan_engine = Arc::clone(&self.engine);
        let vst3_paths = self.settings.vst3_paths.clone();
        let scanning = self.scan_state.read(cx).scanning;
        let scan_state = self.scan_state.clone();

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
                                    .font_semibold()
                                    .text_color(colors::base_200())
                                    .child(i18n::t("plugins.title")),
                            )
                            .child(badge(
                                plugin_count.to_string(),
                                if plugin_count == 0 {
                                    BadgeStyle::Red
                                } else {
                                    BadgeStyle::Purple
                                },
                            )),
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
                    .child(
                        icon_button(
                            "btn-scan-paths",
                            "assets/icons/settings.svg",
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
                                cx.notify();
                            });
                            window.refresh();
                            let engine = Arc::clone(&scan_engine);
                            let vst3 = vst3_paths.clone();
                            let scan_state = scan_state.clone();
                            let task =
                                cx.background_spawn(async move { engine.scan_plugins(&vst3) });
                            cx.spawn(async move |cx| {
                                let result = task.await;
                                if let Err(error) = result {
                                    eprintln!("JUCE plugin scan failed: {error}");
                                }
                                scan_state.update(cx, |state, cx| {
                                    state.scanning = false;
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
    let item_count = plugins.len() + 1;
    let state = window
        .use_keyed_state("plugins-virtual-list", cx, |_, _| VirtualListState {
            scroll: UniformListScrollHandle::new(),
        })
        .clone();
    let scroll_handle = state.read(cx).scroll.clone();
    let render_plugins = Arc::clone(&plugins);
    let plugin_count = plugins.len();
    let card_scan_state = header.scan_state.clone();
    let content = uniform_list(
        "plugins-uniform-list",
        item_count,
        move |range, _window, cx| {
            range
                .map(|row| {
                    if row == 0 {
                        return div()
                            .w_full()
                            .h(ROW_HEIGHT)
                            .px_4()
                            .pt_4()
                            .pb_3()
                            .child(header.render(plugin_count, cx))
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
        .child(SmoothUniformListScroll::new(
            "plugins-virtual-list-smooth-scroll",
            scroll_handle.clone(),
            content.size_full(),
        ))
        .child(PageScrollbar::new(
            "plugins-virtual-list-scrollbar",
            scroll_handle,
        ))
        .into_any_element()
}

fn render_plugin_card(
    plugin: PluginItem,
    engine: Arc<Engine>,
    on_navigate: NavigateCallback,
    chain_operations: Entity<ChainOperationState>,
    scan_state: Entity<PluginScanState>,
    cx: &App,
) -> AnyElement {
    let stable_id = plugin.id.clone();
    let card_id = SharedString::from(format!("plugin-card-{stable_id}"));
    let in_chain_button_id = SharedString::from(format!("btn-in-chain-{stable_id}"));
    let add_button_id = SharedString::from(format!("btn-add-chain-{stable_id}"));
    let reveal_button_id = SharedString::from(format!("btn-reveal-{stable_id}"));
    let add_engine = engine;
    let add_operations = chain_operations.clone();
    let plugin_path = plugin.path.clone();
    let pending_plugin = PendingPlugin {
        unique_id: plugin.id.clone(),
        name: plugin.name.clone(),
        vendor: plugin.vendor.clone(),
        format: plugin.format.clone(),
    };
    let chain_busy = chain_operations.read(cx).is_busy();
    let scanning = scan_state.read(cx).scanning;

    div()
        .id(card_id)
        .w_full()
        .h(CARD_HEIGHT)
        .p_4()
        .bg(colors::base_950())
        .border_1()
        .border_color(colors::base_800())
        .rounded_lg()
        .flex()
        .flex_row()
        .items_center()
        .justify_between()
        .child(
            div()
                .flex_1()
                .overflow_hidden()
                .flex()
                .flex_row()
                .items_center()
                .gap_4()
                .child(
                    div()
                        .flex_none()
                        .size(px(40.0))
                        .bg(colors::base_900())
                        .border_1()
                        .border_color(colors::base_800())
                        .rounded_md()
                        .flex()
                        .items_center()
                        .justify_center()
                        .child(
                            svg()
                                .external_path(crate::ui::resolve_asset_path(
                                    "assets/icons/box.svg",
                                ))
                                .size_5()
                                .text_color(colors::orange()),
                        ),
                )
                .child(
                    div()
                        .flex_1()
                        .overflow_hidden()
                        .flex()
                        .flex_col()
                        .gap(px(2.0))
                        .child(
                            div()
                                .flex()
                                .flex_row()
                                .items_center()
                                .gap_2()
                                .child(
                                    div()
                                        .truncate()
                                        .text_sm()
                                        .font_bold()
                                        .text_color(colors::base_200())
                                        .child(plugin.name),
                                )
                                .child(badge(plugin.format.to_uppercase(), BadgeStyle::Purple))
                                .when(plugin.initializing, |row| {
                                    row.child(loading_badge(i18n::t("plugins.initializing")))
                                })
                                .when(plugin.in_chain && !plugin.initializing, |row| {
                                    row.child(badge(i18n::t("plugins.inChain"), BadgeStyle::Green))
                                }),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(colors::base_500())
                                .truncate()
                                .child(plugin.vendor),
                        ),
                ),
        )
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap_1()
                .child(if plugin.in_chain || plugin.initializing {
                    chain_navigation_button(in_chain_button_id, cx)
                        .on_click(move |_, window, cx| on_navigate(Route::Home, window, cx))
                        .into_any_element()
                } else {
                    icon_button(
                        add_button_id,
                        "assets/icons/plus.svg",
                        i18n::t("plugins.addToChain"),
                        IconButtonStyle::Outline,
                        false,
                        chain_busy || scanning,
                        cx,
                    )
                    .on_click(move |_, _, cx| {
                        if add_operations.read(cx).is_busy() || scan_state.read(cx).scanning {
                            return;
                        }
                        chain_operations::add_plugin(
                            add_operations.clone(),
                            Arc::clone(&add_engine),
                            pending_plugin.clone(),
                            cx,
                        );
                    })
                    .into_any_element()
                })
                .child(
                    icon_button(
                        reveal_button_id,
                        "assets/icons/folder.svg",
                        i18n::t("plugins.reveal"),
                        IconButtonStyle::Outline,
                        false,
                        false,
                        cx,
                    )
                    .on_click(move |_, _, _| reveal_plugin(&plugin_path)),
                ),
        )
        .into_any_element()
}

fn reveal_plugin(path: &str) {
    if let Err(error) = std::process::Command::new("explorer.exe")
        .arg(format!("/select,{path}"))
        .spawn()
    {
        eprintln!("failed to reveal plugin in Explorer: {error}");
    }
}
