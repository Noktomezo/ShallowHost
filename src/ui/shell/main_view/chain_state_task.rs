use gpui::{AppContext as _, Context};
use std::sync::Arc;
use std::time::Duration;

use super::MainView;

const STATE_SAVE_POLL: Duration = Duration::from_millis(500);

impl MainView {
    pub(super) fn start_chain_state_task(&mut self, cx: &mut Context<Self>) {
        let engine = Arc::clone(&self.engine);
        self._chain_state_task = cx.spawn(async move |_, cx| {
            let mut observed_revision = match engine.state_revision() {
                Ok(revision) => revision,
                Err(error) => {
                    eprintln!("failed to read initial plugin state revision: {error}");
                    0
                }
            };
            let mut saved_revision = observed_revision;
            loop {
                cx.background_executor().timer(STATE_SAVE_POLL).await;
                let revision = match engine.state_revision() {
                    Ok(revision) => revision,
                    Err(error) => {
                        eprintln!("failed to read plugin state revision: {error}");
                        continue;
                    }
                };
                if !should_save_state(&mut observed_revision, saved_revision, revision) {
                    continue;
                }

                let save_engine = Arc::clone(&engine);
                match cx
                    .background_spawn(async move { save_engine.save_chain_state() })
                    .await
                {
                    Ok(()) => saved_revision = revision,
                    Err(error) => eprintln!("failed to autosave plugin chain state: {error}"),
                }
            }
        });
    }
}

fn should_save_state(observed: &mut u64, saved: u64, current: u64) -> bool {
    if current != *observed {
        *observed = current;
        return false;
    }
    current != saved
}

#[cfg(test)]
mod tests {
    use super::should_save_state;

    #[test]
    fn saves_only_after_a_revision_stays_stable_for_one_poll() {
        let mut observed = 0;

        assert!(!should_save_state(&mut observed, 0, 1));
        assert!(should_save_state(&mut observed, 0, 1));
        assert!(!should_save_state(&mut observed, 1, 1));
        assert!(!should_save_state(&mut observed, 1, 2));
    }
}
