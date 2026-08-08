use std::time::{Duration, Instant};

const SIDEBAR_MOTION: Duration = Duration::from_millis(200);

pub(super) struct SidebarMotion {
    from: f32,
    to: f32,
    changed_at: Option<Instant>,
}

impl SidebarMotion {
    pub(super) fn expanded() -> Self {
        Self {
            from: 1.0,
            to: 1.0,
            changed_at: None,
        }
    }

    pub(super) fn sample(&self) -> (f32, bool) {
        self.sample_at(Instant::now())
    }

    pub(super) fn set_collapsed(&mut self, collapsed: bool) {
        self.set_collapsed_at(collapsed, Instant::now());
    }

    fn sample_at(&self, now: Instant) -> (f32, bool) {
        let Some(changed_at) = self.changed_at else {
            return (self.to, false);
        };
        let linear =
            now.saturating_duration_since(changed_at).as_secs_f32() / SIDEBAR_MOTION.as_secs_f32();
        if linear >= 1.0 {
            return (self.to, false);
        }
        let eased = ease_in_out_cubic(linear);
        (self.from + (self.to - self.from) * eased, true)
    }

    fn set_collapsed_at(&mut self, collapsed: bool, now: Instant) {
        let (current, _) = self.sample_at(now);
        let target = if collapsed { 0.0 } else { 1.0 };
        self.from = current;
        self.to = target;
        self.changed_at = if (current - target).abs() > f32::EPSILON {
            Some(now)
        } else {
            None
        };
    }
}

fn ease_in_out_cubic(progress: f32) -> f32 {
    let progress = progress.clamp(0.0, 1.0);
    if progress < 0.5 {
        4.0 * progress.powi(3)
    } else {
        1.0 - (-2.0 * progress + 2.0).powi(3) / 2.0
    }
}

#[cfg(test)]
mod tests {
    use super::{SIDEBAR_MOTION, SidebarMotion};
    use std::time::{Duration, Instant};

    #[test]
    fn starts_expanded_without_replaying_an_animation() {
        let motion = SidebarMotion::expanded();

        assert_eq!(motion.sample(), (1.0, false));
    }

    #[test]
    fn reversing_mid_transition_does_not_jump_to_an_endpoint() {
        let start = Instant::now();
        let mut motion = SidebarMotion::expanded();
        motion.set_collapsed_at(true, start);
        let midpoint = start + SIDEBAR_MOTION / 2;
        let (before_reverse, animating) = motion.sample_at(midpoint);

        motion.set_collapsed_at(false, midpoint);
        let (after_reverse, reversed_animating) = motion.sample_at(midpoint);

        assert!(animating);
        assert!(reversed_animating);
        assert!((before_reverse - 0.5).abs() < f32::EPSILON);
        assert!((after_reverse - before_reverse).abs() < f32::EPSILON);
        assert_eq!(
            motion.sample_at(midpoint + SIDEBAR_MOTION + Duration::from_millis(1)),
            (1.0, false)
        );
    }
}
