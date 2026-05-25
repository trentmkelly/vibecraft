use std::time::{Duration, Instant};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PacketRateDecision {
    Allow,
    Kick { reason: String },
}

#[derive(Debug, Clone)]
pub struct PacketRateLimiter {
    limit_per_second: u32,
    last_second_tick: Instant,
    received_packets: u32,
    average_received_packets: f32,
    kicked: bool,
}

impl PacketRateLimiter {
    pub fn new(limit_per_second: u32, now: Instant) -> Self {
        Self {
            limit_per_second,
            last_second_tick: now,
            received_packets: 0,
            average_received_packets: 0.0,
            kicked: false,
        }
    }

    pub fn record_packet(&mut self, now: Instant) -> PacketRateDecision {
        if self.limit_per_second == 0 {
            return PacketRateDecision::Allow;
        }

        if let PacketRateDecision::Kick { reason } = self.tick(now) {
            return PacketRateDecision::Kick { reason };
        }

        self.received_packets = self.received_packets.saturating_add(1);
        self.current_decision()
    }

    pub fn tick(&mut self, now: Instant) -> PacketRateDecision {
        if self.limit_per_second == 0 {
            return PacketRateDecision::Allow;
        }

        while now.duration_since(self.last_second_tick) >= Duration::from_secs(1) {
            // Java: Connection.tickSecond() computes
            // averageReceivedPackets = Mth.lerp(0.75F, receivedPackets, averageReceivedPackets).
            self.average_received_packets = self.received_packets as f32
                + 0.75 * (self.average_received_packets - self.received_packets as f32);
            self.received_packets = 0;
            self.last_second_tick += Duration::from_secs(1);
            if let PacketRateDecision::Kick { reason } = self.current_decision() {
                return PacketRateDecision::Kick { reason };
            }
        }

        PacketRateDecision::Allow
    }

    fn current_decision(&mut self) -> PacketRateDecision {
        if self.kicked || self.average_received_packets > self.limit_per_second as f32 {
            self.kicked = true;
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
        for _ in 0..9 {
            assert_eq!(limiter.record_packet(now), PacketRateDecision::Allow);
        }
        assert!(matches!(
            limiter.tick(now + Duration::from_secs(1)),
            PacketRateDecision::Kick { .. }
        ));
    }

    #[test]
    fn resets_after_one_second() {
        let now = Instant::now();
        let mut limiter = PacketRateLimiter::new(1, now);
        assert_eq!(limiter.record_packet(now), PacketRateDecision::Allow);
        assert_eq!(
            limiter.tick(now + Duration::from_secs(1)),
            PacketRateDecision::Allow
        );
        assert_eq!(
            limiter.record_packet(now + Duration::from_secs(1)),
            PacketRateDecision::Allow
        );
    }

    #[test]
    fn sustained_packets_use_java_smoothed_average() {
        let now = Instant::now();
        let mut limiter = PacketRateLimiter::new(10, now);
        for _ in 0..20 {
            assert_eq!(limiter.record_packet(now), PacketRateDecision::Allow);
        }
        assert_eq!(
            limiter.tick(now + Duration::from_secs(1)),
            PacketRateDecision::Allow
        );
        for _ in 0..20 {
            assert_eq!(
                limiter.record_packet(now + Duration::from_secs(1)),
                PacketRateDecision::Allow
            );
        }
        assert_eq!(
            limiter.tick(now + Duration::from_secs(2)),
            PacketRateDecision::Allow
        );
        for _ in 0..20 {
            assert_eq!(
                limiter.record_packet(now + Duration::from_secs(2)),
                PacketRateDecision::Allow
            );
        }
        assert!(matches!(
            limiter.tick(now + Duration::from_secs(3)),
            PacketRateDecision::Kick { .. }
        ));
    }
}
