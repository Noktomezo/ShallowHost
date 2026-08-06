use std::sync::Arc;

use gpui::prelude::*;
use gpui::*;
use gpui_component::StyledExt;

use super::controls::{IconButtonStyle, chain_navigation_button, icon_button};
use super::{PluginItem, PluginScanState};
use crate::config::PluginSettings;
use crate::engine::Engine;
use crate::ui::badge::{BadgeStyle, badge};
use crate::ui::colors;
use crate::ui::i18n;
use crate::ui::routes::{DropdownCallbacks, NavigateCallback, Route};
use crate::ui::smooth_scroll::{PageScrollbar, SmoothListScroll};

// 40 px content + 32 px card padding + 12 px inter-row spacing.
const ESTIMATED_ROW_HEIGHT: Pixels = px(84.0);
const LIST_OVERDRAW: Pixels = px(178.0);

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
        let vst2_paths = self.settings.vst2_paths.clone();
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
                            let vst2 = vst2_paths.clone();
                            let vst3 = vst3_paths.clone();
                            let scan_state = scan_state.clone();
                            let task = cx
                                .background_spawn(async move { engine.scan_plugins(&vst2, &vst3) });
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
    list: ListState,
}

pub(super) fn render(
    window: &mut Window,
    cx: &mut App,
    header: HeaderContext,
    plugins: Arc<Vec<PluginItem>>,
    engine: Arc<Engine>,
    on_navigate: NavigateCallback,
) -> AnyElement {
    // The header is row zero, so it participates in the same viewport and scroll
    // position as the cards instead of introducing a nested scrolling region.
    let item_count = plugins.len() + 1;
    let state = window
        .use_keyed_state("plugins-virtual-list", cx, |_, _| VirtualListState {
            list: ListState::new(item_count, ListAlignment::Top, LIST_OVERDRAW)
                .with_uniform_item_height(ESTIMATED_ROW_HEIGHT),
        })
        .clone();
    let list_state = state.read(cx).list.clone();
    if list_state.item_count() != item_count {
        list_state.reset_with_uniform_height(item_count, ESTIMATED_ROW_HEIGHT);
    }

    let scrollbar_state = list_state.clone();
    let render_plugins = Arc::clone(&plugins);
    let plugin_count = plugins.len();
    let content = list(list_state, move |row, _window, cx| {
        if row == 0 {
            return div()
                .w_full()
                .px_4()
                .pt_4()
                .pb_3()
                .child(header.render(plugin_count, cx))
                .into_any_element();
        }

        let index = row - 1;
        let Some(plugin) = render_plugins.get(index).cloned() else {
            return div().into_any_element();
        };
        div()
            .w_full()
            .px_4()
            .pb(if index + 1 == plugin_count {
                px(16.0)
            } else {
                px(12.0)
            })
            .child(render_plugin_card(
                index,
                plugin,
                Arc::clone(&engine),
                on_navigate.clone(),
            ))
            .into_any_element()
    });

    div()
        .relative()
        .size_full()
        .child(SmoothListScroll::new(
            "plugins-virtual-list-smooth-scroll",
            scrollbar_state.clone(),
            content.size_full(),
        ))
        .child(PageScrollbar::new(
            "plugins-virtual-list-scrollbar",
            scrollbar_state,
        ))
        .into_any_element()
}

fn render_plugin_card(
    index: usize,
    plugin: PluginItem,
    engine: Arc<Engine>,
    on_navigate: NavigateCallback,
) -> AnyElement {
    let add_engine = Arc::clone(&engine);
    let remove_engine = engine;
    let plugin_id = plugin.id.clone();
    let remove_id = plugin.id.clone();
    let plugin_path = plugin.path.clone();

    div()
        .id(SharedString::from(format!("plugin-card-{index}")))
        .w_full()
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
                                        .text_sm()
                                        .font_bold()
                                        .text_color(colors::base_200())
                                        .child(plugin.name),
                                )
                                .child(badge(plugin.format.to_uppercase(), BadgeStyle::Purple))
                                .when(plugin.in_chain, |row| {
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
                .child(if plugin.in_chain {
                    chain_navigation_button(SharedString::from(format!("btn-in-chain-{index}")))
                        .on_click(move |_, window, cx| on_navigate(Route::Home, window, cx))
                        .into_any_element()
                } else {
                    icon_button(
                        SharedString::from(format!("btn-add-chain-{index}")),
                        "assets/icons/plus.svg",
                        i18n::t("plugins.addToChain"),
                        IconButtonStyle::Outline,
                        false,
                    )
                    .on_click(move |_, _, cx| {
                        if let Err(error) = add_engine.add_to_chain(&plugin_id) {
                            eprintln!("failed to add plugin to JUCE chain: {error}");
                        }
                        cx.refresh_windows();
                    })
                    .into_any_element()
                })
                .child(
                    icon_button(
                        SharedString::from(format!("btn-reveal-{index}")),
                        "assets/icons/folder.svg",
                        i18n::t("plugins.reveal"),
                        IconButtonStyle::Outline,
                        false,
                    )
                    .on_click(move |_, _, _| reveal_plugin(&plugin_path)),
                )
                .child(
                    icon_button(
                        SharedString::from(format!("btn-remove-{index}")),
                        "assets/icons/trash-2.svg",
                        i18n::t("plugins.remove"),
                        IconButtonStyle::Danger,
                        false,
                    )
                    .on_click(move |_, _, cx| {
                        if let Err(error) = remove_engine.remove_cached_plugin(&remove_id) {
                            eprintln!("failed to remove cached plugin: {error}");
                        }
                        cx.refresh_windows();
                    }),
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
