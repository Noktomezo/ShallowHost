use std::sync::Arc;

use gpui::prelude::*;
use gpui::*;
use gpui_component::input::InputState;

use crate::infrastructure::config::PluginSettings;
use crate::infrastructure::engine::Engine;
use crate::ui::shell::routes::{DropdownCallbacks, NavigateCallback, ScanPathsDialogState};
use crate::ui::state::chain_operations::ChainOperationState;

mod controls;
mod scan_paths_dialog;
mod virtualized;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PluginItem {
    pub id: String,
    pub name: String,
    pub vendor: String,
    pub format: String,
    pub path: String,
    pub in_chain: bool,
    pub initializing: bool,
}

#[derive(Default)]
pub struct PluginScanState {
    scanning: bool,
    progress: f32,
}

pub struct PluginsPage {
    on_navigate: NavigateCallback,
    engine: Arc<Engine>,
    settings: PluginSettings,
    scan_paths: ScanPathsDialogState,
    callbacks: DropdownCallbacks,
    scan_state: Entity<PluginScanState>,
    search: Entity<InputState>,
    chain_operations: Entity<ChainOperationState>,
}

impl PluginsPage {
    pub fn new(
        cb: &DropdownCallbacks,
        engine: Arc<Engine>,
        settings: PluginSettings,
        scan_paths: ScanPathsDialogState,
        scan_state: Entity<PluginScanState>,
        search: Entity<InputState>,
        chain_operations: Entity<ChainOperationState>,
    ) -> Self {
        Self {
            on_navigate: cb.on_navigate.clone(),
            engine,
            settings,
            scan_paths,
            callbacks: cb.clone(),
            scan_state,
            search,
            chain_operations,
        }
    }

    pub fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let chain_ids = self
            .engine
            .cached_chain()
            .map(|items| {
                items
                    .into_iter()
                    .filter_map(|item| item.unique_id)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        let operations = self.chain_operations.read(cx);
        let search_query = self.search.read(cx).value().to_string();
        let mut plugins = self
            .engine
            .plugins()
            .unwrap_or_default()
            .into_iter()
            .map(|plugin| PluginItem {
                in_chain: chain_ids.contains(&plugin.unique_id),
                initializing: operations.is_adding(&plugin.unique_id),
                id: plugin.unique_id,
                name: plugin.name,
                vendor: plugin.vendor,
                format: plugin.format,
                path: plugin.path,
            })
            .collect::<Vec<_>>();
        plugins.retain(|plugin| plugin_matches_search(plugin, &search_query));
        sort_plugins(&mut plugins);
        let plugins = Arc::new(plugins);
        let scan_paths_visible = self.scan_paths.visible();
        let header = virtualized::HeaderContext::new(
            Arc::clone(&self.engine),
            self.settings.clone(),
            self.callbacks.clone(),
            self.scan_state.clone(),
            self.search.clone(),
            !scan_paths_visible,
        );
        let content = virtualized::render(
            window,
            cx,
            header,
            plugins,
            Arc::clone(&self.engine),
            self.on_navigate.clone(),
            self.chain_operations.clone(),
        );

        div()
            .relative()
            .size_full()
            .child(content)
            .when(scan_paths_visible, |element| {
                element.child(scan_paths_dialog::render_scan_paths_dialog(
                    &self.settings,
                    &self.callbacks,
                    self.scan_paths.open,
                    self.scan_paths.revision,
                ))
            })
    }
}

fn plugin_matches_search(plugin: &PluginItem, query: &str) -> bool {
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

fn sort_plugins(plugins: &mut [PluginItem]) {
    plugins.sort_by_cached_key(|plugin| {
        (
            plugin_name_group(&plugin.name),
            plugin.name.to_lowercase(),
            plugin.vendor.to_lowercase(),
            plugin.id.clone(),
        )
    });
}

fn plugin_name_group(name: &str) -> u8 {
    match name.trim_start().chars().next() {
        Some(character) if character.is_ascii_digit() => 0,
        Some(character) if character.is_ascii_alphabetic() => 1,
        _ => 2,
    }
}

#[cfg(test)]
mod tests {
    use super::{PluginItem, plugin_matches_search, sort_plugins};

    fn plugin(id: &str, name: &str, vendor: &str) -> PluginItem {
        PluginItem {
            id: id.into(),
            name: name.into(),
            vendor: vendor.into(),
            format: String::from("VST3"),
            path: String::new(),
            in_chain: false,
            initializing: false,
        }
    }

    #[test]
    fn sorts_plugins_digits_then_case_insensitive_alphabet() {
        let mut plugins = vec![
            plugin("5", "_Utility", "Vendor"),
            plugin("4", "beta", "Vendor"),
            plugin("3", "Alpha", "Z Vendor"),
            plugin("2", "2Bus", "Vendor"),
            plugin("1", "1Knob", "Vendor"),
            plugin("0", "alpha", "A Vendor"),
        ];

        sort_plugins(&mut plugins);

        assert_eq!(
            plugins
                .iter()
                .map(|plugin| plugin.id.as_str())
                .collect::<Vec<_>>(),
            ["1", "2", "0", "3", "4", "5"]
        );
    }

    #[test]
    fn search_matches_substrings_vendor_and_typo_with_bounded_distance() {
        let clarity = plugin("1", "Clarity Vx - DeReverb", "Waves");

        assert!(plugin_matches_search(&clarity, "clarity"));
        assert!(plugin_matches_search(&clarity, "waves"));
        assert!(plugin_matches_search(&clarity, "claritu waves"));
        assert!(!plugin_matches_search(&clarity, "cx"));
        assert!(!plugin_matches_search(&clarity, "completely unrelated"));
    }
}
