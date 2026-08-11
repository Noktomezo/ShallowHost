use gpui::prelude::*;
use gpui::*;
use std::time::Duration;

const PAGE_TRANSITION_DURATION: Duration = Duration::from_millis(200);
const PAGE_TRANSITION_OFFSET: f32 = 8.0;

pub(super) struct PageTransition {
    revision: u64,
    enabled: bool,
}

impl PageTransition {
    pub(super) const fn new() -> Self {
        Self {
            revision: 0,
            enabled: false,
        }
    }

    pub(super) fn start(&mut self) {
        self.revision = self.revision.wrapping_add(1);
        self.enabled = true;
    }

    pub(super) fn wrap(&self, content: AnyElement) -> AnyElement {
        let page = div().size_full().relative().child(content);
        if !self.enabled {
            return page.into_any_element();
        }

        let animation_id =
            ElementId::NamedInteger(SharedString::from("page-enter-transition"), self.revision);
        page.with_animation(
            animation_id,
            Animation::new(PAGE_TRANSITION_DURATION).with_easing(ease_out_quint()),
            |page, progress| {
                page.opacity(progress)
                    .top(px(PAGE_TRANSITION_OFFSET * (1.0 - progress)))
            },
        )
        .into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::PageTransition;

    #[test]
    fn transition_is_disabled_until_the_first_navigation() {
        let transition = PageTransition::new();

        assert!(!transition.enabled);
        assert_eq!(transition.revision, 0);
    }

    #[test]
    fn each_navigation_gets_a_fresh_animation_identity() {
        let mut transition = PageTransition::new();

        transition.start();
        let first_revision = transition.revision;
        transition.start();

        assert!(transition.enabled);
        assert_ne!(transition.revision, first_revision);
    }
}
