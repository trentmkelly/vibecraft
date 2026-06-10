#![allow(dead_code)]

use std::time::{Duration, Instant};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PacketRateDecision {
    Allow,
    Kick { reason: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RateKickAction {
    SendDisconnect { reason: String },
    DisconnectAfterSend { reason: String },
    SetReadOnly,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RateKickPlan {
    pub average_received_packets: String,
    pub actions: Vec<RateKickAction>,
}

#[derive(Debug, Clone)]
pub struct RateKickingConnection {
    rate_limit_packets_per_second: u32,
    limiter: PacketRateLimiter,
}

impl RateKickingConnection {
    pub fn new(rate_limit_packets_per_second: u32, now: Instant) -> Self {
        Self {
            rate_limit_packets_per_second,
            limiter: PacketRateLimiter::new(rate_limit_packets_per_second, now),
        }
    }

    pub fn record_packet(&mut self, now: Instant) {
        self.limiter.record_packet(now);
    }

    pub fn tick_second(&mut self, now: Instant) -> Option<RateKickPlan> {
        let decision = self.limiter.tick(now);
        if matches!(decision, PacketRateDecision::Kick { .. }) {
            Some(RateKickPlan {
                average_received_packets: format!("{}", self.limiter.average_received_packets()),
                actions: vec![
                    RateKickAction::SendDisconnect {
                        reason: EXCEEDED_PACKET_RATE_REASON.to_string(),
                    },
                    RateKickAction::DisconnectAfterSend {
                        reason: EXCEEDED_PACKET_RATE_REASON.to_string(),
                    },
                    RateKickAction::SetReadOnly,
                ],
            })
        } else {
            None
        }
    }

    pub fn rate_limit_packets_per_second(&self) -> u32 {
        self.rate_limit_packets_per_second
    }
}

const EXCEEDED_PACKET_RATE_REASON: &str = "disconnect.exceeded_packet_rate";

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
                reason: EXCEEDED_PACKET_RATE_REASON.to_string(),
            }
        } else {
            PacketRateDecision::Allow
        }
    }

    pub fn average_received_packets(&self) -> f32 {
        self.average_received_packets
    }
}

#[cfg(test)]
mod tests {
    use super::{PacketRateDecision, PacketRateLimiter, RateKickAction, RateKickingConnection};
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

    #[test]
    fn rate_kicking_connection_matches_java_disconnect_sequence() {
        const RATE_KICKING_CONNECTION_JAVA: &str = include_str!(
            "../../../decompiled-server-26.1.2/net/minecraft/network/RateKickingConnection.java"
        );
        const CONNECTION_JAVA: &str =
            include_str!("../../../decompiled-server-26.1.2/net/minecraft/network/Connection.java");

        for sentinel in [
            "private static final Component EXCEED_REASON = Component.translatable(\"disconnect.exceeded_packet_rate\");",
            "super(PacketFlow.SERVERBOUND);",
            "this.rateLimitPacketsPerSecond = rateLimitPacketsPerSecond;",
            "protected void tickSecond()",
            "super.tickSecond();",
            "float averageReceivedPackets = this.getAverageReceivedPackets();",
            "if (averageReceivedPackets > this.rateLimitPacketsPerSecond)",
            "this.send(new ClientboundDisconnectPacket(EXCEED_REASON), PacketSendListener.thenRun(() -> this.disconnect(EXCEED_REASON)));",
            "this.setReadOnly();",
        ] {
            assert!(
                RATE_KICKING_CONNECTION_JAVA.contains(sentinel),
                "missing RateKickingConnection sentinel {sentinel}"
            );
        }
        for sentinel in [
            "this.averageReceivedPackets = Mth.lerp(0.75F, this.receivedPackets, this.averageReceivedPackets);",
            "this.receivedPackets = 0;",
            "public float getAverageReceivedPackets()",
        ] {
            assert!(
                CONNECTION_JAVA.contains(sentinel),
                "missing Connection rate-limit sentinel {sentinel}"
            );
        }

        let now = Instant::now();
        let mut connection = RateKickingConnection::new(2, now);
        assert_eq!(connection.rate_limit_packets_per_second(), 2);
        for _ in 0..9 {
            connection.record_packet(now);
        }

        let plan = connection
            .tick_second(now + Duration::from_secs(1))
            .expect("smoothed packet rate should exceed limit");
        assert_eq!(plan.average_received_packets, "2.25");
        assert_eq!(
            plan.actions,
            vec![
                RateKickAction::SendDisconnect {
                    reason: "disconnect.exceeded_packet_rate".to_string(),
                },
                RateKickAction::DisconnectAfterSend {
                    reason: "disconnect.exceeded_packet_rate".to_string(),
                },
                RateKickAction::SetReadOnly,
            ]
        );
    }
}
