use std::sync::Arc;

use gpui::prelude::*;
use gpui::*;

use super::controls::{IconButtonStyle, chain_navigation_button, icon_button};
use super::virtualized::CARD_HEIGHT;
use super::{PluginItem, PluginScanState};
use crate::infrastructure::engine::Engine;
use crate::ui::components::badge::{BadgeStyle, badge, loading_badge};
use crate::ui::foundation::{colors, i18n, plugin_format};
use crate::ui::shell::routes::{NavigateCallback, Route};
use crate::ui::state::chain_operations::{self, ChainOperationState, PendingPlugin};

pub(super) fn render_plugin_card(
    plugin: PluginItem,
    engine: Arc<Engine>,
    on_navigate: NavigateCallback,
    chain_operations: Entity<ChainOperationState>,
    scan_state: Entity<PluginScanState>,
    surface: Rgba,
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
        .bg(surface)
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
                                .path(crate::ui::resolve_asset_path("assets/icons/box.svg"))
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
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(colors::base_200())
                                        .child(plugin.name),
                                )
                                .child(badge(
                                    plugin_format::display_name(&plugin.format),
                                    plugin_format::badge_style(&plugin.format),
                                ))
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
