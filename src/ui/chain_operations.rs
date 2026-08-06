use std::sync::Arc;

use gpui::{App, AppContext, Entity};

use crate::engine::{ChainItem, Engine, EngineError};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PendingPlugin {
    pub unique_id: String,
    pub name: String,
    pub vendor: String,
    pub format: String,
}

impl PendingPlugin {
    pub fn from_chain_item(item: ChainItem) -> Option<Self> {
        Some(Self {
            unique_id: item.unique_id?,
            name: item.name,
            vendor: item.vendor,
            format: item.format,
        })
    }

    pub fn chain_item(&self) -> ChainItem {
        ChainItem {
            id: format!("pending-{}", self.unique_id),
            name: self.name.clone(),
            vendor: self.vendor.clone(),
            format: self.format.clone(),
            bypassed: false,
            unique_id: Some(self.unique_id.clone()),
            initializing: true,
            removing: false,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum ChainOperation {
    Adding(PendingPlugin),
    Restoring(Vec<PendingPlugin>),
    Removing(String),
    Clearing,
}

#[derive(Default)]
pub struct ChainOperationState {
    operation: Option<ChainOperation>,
}

impl ChainOperationState {
    pub fn is_busy(&self) -> bool {
        self.operation.is_some()
    }

    pub fn pending_plugins(&self) -> &[PendingPlugin] {
        match &self.operation {
            Some(ChainOperation::Adding(plugin)) => std::slice::from_ref(plugin),
            Some(ChainOperation::Restoring(plugins)) => plugins,
            Some(ChainOperation::Removing(_) | ChainOperation::Clearing) | None => &[],
        }
    }

    pub fn is_adding(&self, unique_id: &str) -> bool {
        self.pending_plugins()
            .iter()
            .any(|plugin| plugin.unique_id == unique_id)
    }

    pub fn is_removing(&self, node_id: &str) -> bool {
        matches!(&self.operation, Some(ChainOperation::Removing(id)) if id == node_id)
    }

    pub fn is_clearing(&self) -> bool {
        matches!(self.operation, Some(ChainOperation::Clearing))
    }

    pub fn begin_restore(&mut self, plugins: Vec<PendingPlugin>) -> bool {
        self.begin(ChainOperation::Restoring(plugins))
    }

    pub fn finish_restore(&mut self) {
        if matches!(self.operation, Some(ChainOperation::Restoring(_))) {
            self.finish();
        }
    }

    fn begin(&mut self, operation: ChainOperation) -> bool {
        if self.operation.is_some() {
            return false;
        }
        self.operation = Some(operation);
        true
    }

    fn finish(&mut self) {
        self.operation = None;
    }
}

pub fn add_plugin(
    state: Entity<ChainOperationState>,
    engine: Arc<Engine>,
    plugin: PendingPlugin,
    cx: &mut App,
) {
    start_operation(
        state,
        engine,
        ChainOperation::Adding(plugin.clone()),
        move |engine| engine.add_to_chain(&plugin.unique_id),
        "add plugin to JUCE chain",
        cx,
    );
}

pub fn remove_plugin(
    state: Entity<ChainOperationState>,
    engine: Arc<Engine>,
    node_id: String,
    cx: &mut App,
) {
    start_operation(
        state,
        engine,
        ChainOperation::Removing(node_id.clone()),
        move |engine| engine.remove_from_chain(&node_id),
        "remove plugin from JUCE chain",
        cx,
    );
}

pub fn clear_chain(state: Entity<ChainOperationState>, engine: Arc<Engine>, cx: &mut App) {
    start_operation(
        state,
        engine,
        ChainOperation::Clearing,
        Engine::clear_chain,
        "clear JUCE plugin chain",
        cx,
    );
}

fn start_operation(
    state: Entity<ChainOperationState>,
    engine: Arc<Engine>,
    operation: ChainOperation,
    work: impl FnOnce(&Engine) -> Result<(), EngineError> + Send + 'static,
    description: &'static str,
    cx: &mut App,
) {
    let started = state.update(cx, |state, cx| {
        let started = state.begin(operation);
        if started {
            cx.notify();
        }
        started
    });
    if !started {
        return;
    }

    cx.refresh_windows();
    let task = cx.background_spawn(async move { work(&engine) });
    cx.spawn(async move |cx| {
        let result = task.await;
        if let Err(error) = result {
            eprintln!("failed to {description}: {error}");
        }
        state.update(cx, |state, cx| {
            state.finish();
            cx.notify();
        });
        cx.refresh();
    })
    .detach();
}

#[cfg(test)]
mod tests {
    use super::{ChainOperation, ChainOperationState, PendingPlugin};

    fn plugin() -> PendingPlugin {
        PendingPlugin {
            unique_id: String::from("vst3.test"),
            name: String::from("Test"),
            vendor: String::from("Vendor"),
            format: String::from("VST3"),
        }
    }

    #[test]
    fn serializes_chain_operations_and_exposes_pending_plugin() {
        let mut state = ChainOperationState::default();
        assert!(state.begin(ChainOperation::Adding(plugin())));
        assert!(state.is_busy());
        assert!(state.is_adding("vst3.test"));
        assert!(!state.begin(ChainOperation::Clearing));

        state.finish();

        assert!(!state.is_busy());
        assert!(state.pending_plugins().is_empty());
    }

    #[test]
    fn pending_plugin_builds_disabled_placeholder() {
        let item = plugin().chain_item();
        assert!(item.initializing);
        assert_eq!(item.unique_id.as_deref(), Some("vst3.test"));
    }

    #[test]
    fn exposes_all_plugins_while_restoring_saved_chain() {
        let mut state = ChainOperationState::default();
        let second = PendingPlugin {
            unique_id: String::from("vst3.second"),
            ..plugin()
        };

        assert!(state.begin_restore(vec![plugin(), second]));
        assert_eq!(state.pending_plugins().len(), 2);
        assert!(state.is_adding("vst3.test"));
        assert!(state.is_adding("vst3.second"));

        state.finish_restore();
        assert!(!state.is_busy());
    }
}
