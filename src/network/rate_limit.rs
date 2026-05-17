use std::time::{Duration, Instant};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PacketRateDecision {
    Allow,
    Kick { reason: String },
}

#[derive(Debug, Clone)]
pub struct PacketRateLimiter {
    limit_per_second: u32,
    window_start: Instant,
    packets_in_window: u32,
}

impl PacketRateLimiter {
    pub fn new(limit_per_second: u32, now: Instant) -> Self {
        Self {
            limit_per_second,
            window_start: now,
            packets_in_window: 0,
        }
    }

    pub fn record_packet(&mut self, now: Instant) -> PacketRateDecision {
        if self.limit_per_second == 0 {
            return PacketRateDecision::Allow;
        }

        if now.duration_since(self.window_start) >= Duration::from_secs(1) {
            self.window_start = now;
            self.packets_in_window = 0;
        }

        self.packets_in_window += 1;
        if self.packets_in_window > self.limit_per_second {
            PacketRateDecision::Kick {
                reason: "disconnect.exceeded_packet_rate".to_string(),
            }
        } else {
            PacketRateDecision::Allow
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{PacketRateDecision, PacketRateLimiter};
    use std::time::{Duration, Instant};

    #[test]
    fn disabled_rate_limit_allows_everything() {
        let now = Instant::now();
        let mut limiter = PacketRateLimiter::new(0, now);
        for _ in 0..100 {
            assert_eq!(limiter.record_packet(now), PacketRateDecision::Allow);
        }
    }

    #[test]
    fn kicks_after_limit_within_one_second_window() {
        let now = Instant::now();
        let mut limiter = PacketRateLimiter::new(2, now);
        assert_eq!(limiter.record_packet(now), PacketRateDecision::Allow);
        assert_eq!(limiter.record_packet(now), PacketRateDecision::Allow);
        assert!(matches!(
            limiter.record_packet(now),
            PacketRateDecision::Kick { .. }
        ));
    }

    #[test]
    fn resets_after_one_second() {
        let now = Instant::now();
        let mut limiter = PacketRateLimiter::new(1, now);
        assert_eq!(limiter.record_packet(now), PacketRateDecision::Allow);
        assert_eq!(
            limiter.record_packet(now + Duration::from_secs(1)),
            PacketRateDecision::Allow
        );
    }
}
