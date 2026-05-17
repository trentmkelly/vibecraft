#![allow(dead_code)]

use std::time::{Duration, Instant};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerIdleTracker {
    last_action: Option<Instant>,
    won_game: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IdleTimeoutDecision {
    Keep,
    Disconnect { translation_key: &'static str },
}

impl PlayerIdleTracker {
    pub fn new() -> Self {
        Self {
            last_action: None,
            won_game: false,
        }
    }

    pub fn reset_last_action_time(&mut self, now: Instant) {
        self.last_action = Some(now);
    }

    pub fn clear_last_action_time(&mut self) {
        self.last_action = None;
    }

    pub fn set_won_game(&mut self, won_game: bool) {
        self.won_game = won_game;
    }

    pub fn last_action(&self) -> Option<Instant> {
        self.last_action
    }

    pub fn idle_timeout_decision(&self, now: Instant, timeout_minutes: u32) -> IdleTimeoutDecision {
        if self.won_game || timeout_minutes == 0 {
            return IdleTimeoutDecision::Keep;
        }
        let Some(last_action) = self.last_action else {
            return IdleTimeoutDecision::Keep;
        };
        if now.duration_since(last_action) > Duration::from_secs(u64::from(timeout_minutes) * 60) {
            IdleTimeoutDecision::Disconnect {
                translation_key: "multiplayer.disconnect.idling",
            }
        } else {
            IdleTimeoutDecision::Keep
        }
    }
}

impl Default for PlayerIdleTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::{IdleTimeoutDecision, PlayerIdleTracker};
    use std::time::{Duration, Instant};

    #[test]
    fn idle_timeout_is_disabled_at_zero_or_before_first_action() {
        let now = Instant::now();
        let mut tracker = PlayerIdleTracker::new();

        assert_eq!(
            tracker.idle_timeout_decision(now + Duration::from_secs(999), 5),
            IdleTimeoutDecision::Keep
        );
        tracker.reset_last_action_time(now);
        assert_eq!(
            tracker.idle_timeout_decision(now + Duration::from_secs(999), 0),
            IdleTimeoutDecision::Keep
        );
    }

    #[test]
    fn idle_timeout_uses_strictly_greater_than_configured_minutes() {
        let now = Instant::now();
        let mut tracker = PlayerIdleTracker::new();
        tracker.reset_last_action_time(now);

        assert_eq!(
            tracker.idle_timeout_decision(now + Duration::from_secs(300), 5),
            IdleTimeoutDecision::Keep
        );
        assert_eq!(
            tracker.idle_timeout_decision(now + Duration::from_secs(301), 5),
            IdleTimeoutDecision::Disconnect {
                translation_key: "multiplayer.disconnect.idling"
            }
        );
    }

    #[test]
    fn idle_timeout_does_not_disconnect_players_who_won_game() {
        let now = Instant::now();
        let mut tracker = PlayerIdleTracker::new();
        tracker.reset_last_action_time(now);
        tracker.set_won_game(true);

        assert_eq!(
            tracker.idle_timeout_decision(now + Duration::from_secs(301), 5),
            IdleTimeoutDecision::Keep
        );
    }

    #[test]
    fn reset_last_action_time_extends_idle_window() {
        let now = Instant::now();
        let mut tracker = PlayerIdleTracker::new();
        tracker.reset_last_action_time(now);
        tracker.reset_last_action_time(now + Duration::from_secs(240));

        assert_eq!(
            tracker.idle_timeout_decision(now + Duration::from_secs(301), 5),
            IdleTimeoutDecision::Keep
        );
    }
}
