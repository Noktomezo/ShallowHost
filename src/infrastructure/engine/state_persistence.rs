use super::{ChainItem, Engine, EngineError};
use serde_json::Value;
use std::fs;

impl Engine {
    /// Removes an entry from the last snapshot without asking every live plugin
    /// for its state again. Some plugins lock their audio path while servicing
    /// `getStateInformation`, which can otherwise cause an audible dropout.
    pub(super) fn persist_removed_chain_item(
        &self,
        previous_chain: &[ChainItem],
        removed_index: usize,
    ) -> Result<(), EngineError> {
        let state = match fs::read_to_string(&self.chain_state_path) {
            Ok(state) => state,
            Err(source) if source.kind() == std::io::ErrorKind::NotFound => {
                return self.save_chain_state();
            }
            Err(source) => {
                return Err(EngineError::ReadChainState {
                    path: self.chain_state_path.clone(),
                    source,
                });
            }
        };

        let mut saved_chain: Vec<Value> = match serde_json::from_str(&state) {
            Ok(saved_chain) => saved_chain,
            Err(_) => return self.save_chain_state(),
        };
        let Some(saved_index) = matching_saved_index(&saved_chain, previous_chain, removed_index)
        else {
            return self.save_chain_state();
        };

        saved_chain.remove(saved_index);
        let state = serde_json::to_string(&saved_chain)?;
        fs::write(&self.chain_state_path, state).map_err(|source| EngineError::WriteChainState {
            path: self.chain_state_path.clone(),
            source,
        })
    }
}

fn matching_saved_index(
    saved_chain: &[Value],
    previous_chain: &[ChainItem],
    removed_index: usize,
) -> Option<usize> {
    let removed = previous_chain.get(removed_index)?;

    // The normal case preserves exact chain order, including duplicate plugins.
    if saved_chain.len() == previous_chain.len() {
        return Some(removed_index);
    }

    // Recover from an older/incomplete snapshot by matching the same occurrence
    // of the plugin identifier, rather than deleting an arbitrary duplicate.
    let unique_id = removed.unique_id.as_deref()?;
    let occurrence = previous_chain[..=removed_index]
        .iter()
        .filter(|item| item.unique_id.as_deref() == Some(unique_id))
        .count()
        .checked_sub(1)?;

    saved_chain
        .iter()
        .enumerate()
        .filter(|(_, item)| item.get("unique_id").and_then(Value::as_str) == Some(unique_id))
        .nth(occurrence)
        .map(|(index, _)| index)
}

#[cfg(test)]
mod tests {
    use super::matching_saved_index;
    use crate::infrastructure::engine::ChainItem;
    use serde_json::json;

    fn chain_item(id: &str, unique_id: &str) -> ChainItem {
        ChainItem {
            id: id.into(),
            name: "Effect".into(),
            vendor: "Vendor".into(),
            format: "VST3".into(),
            bypassed: false,
            unique_id: Some(unique_id.into()),
            initializing: false,
            removing: false,
        }
    }

    #[test]
    fn removes_exact_position_when_snapshot_matches_chain() {
        let previous = [chain_item("1", "same"), chain_item("2", "same")];
        let saved = [
            json!({ "unique_id": "same" }),
            json!({ "unique_id": "same" }),
        ];

        assert_eq!(matching_saved_index(&saved, &previous, 1), Some(1));
    }

    #[test]
    fn matches_duplicate_occurrence_in_incomplete_snapshot() {
        let previous = [
            chain_item("1", "same"),
            chain_item("2", "missing"),
            chain_item("3", "same"),
        ];
        let saved = [
            json!({ "unique_id": "same" }),
            json!({ "unique_id": "same" }),
        ];

        assert_eq!(matching_saved_index(&saved, &previous, 2), Some(1));
    }
}
