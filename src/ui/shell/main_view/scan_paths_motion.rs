use gpui::{Context, Task};

use super::MainView;
use crate::ui::foundation::motion::DIALOG_MOTION;

impl MainView {
    pub(super) fn dismiss_scan_paths_immediately(&mut self) {
        if !self.scan_paths_open && !self.scan_paths_closing {
            return;
        }

        self.scan_paths_open = false;
        self.scan_paths_closing = false;
        self.scan_paths_revision = self.scan_paths_revision.wrapping_add(1);
        self._scan_paths_motion_task = Task::ready(());
    }

    pub(super) fn set_scan_paths_open(&mut self, open: bool, cx: &mut Context<Self>) {
        if self.scan_paths_open == open && !(open && self.scan_paths_closing) {
            return;
        }

        self.scan_paths_open = open;
        self.scan_paths_closing = !open;
        self.scan_paths_revision = self.scan_paths_revision.wrapping_add(1);
        let revision = self.scan_paths_revision;

        if open || cx.reduce_motion() {
            self.scan_paths_closing = false;
            self._scan_paths_motion_task = Task::ready(());
        } else {
            self._scan_paths_motion_task = cx.spawn(async move |view, cx| {
                cx.background_executor().timer(DIALOG_MOTION).await;
                let _intentionally_ignored = view.update(&mut *cx, |view, cx| {
                    if !view.scan_paths_open && view.scan_paths_revision == revision {
                        view.scan_paths_closing = false;
                        cx.notify();
                    }
                });
            });
        }
        cx.notify();
    }
}
