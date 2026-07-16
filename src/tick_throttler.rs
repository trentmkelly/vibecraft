//! Counter throttling matching Minecraft's `TickThrottler`.

#![allow(dead_code)]

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TickThrottler {
    increment_step: i32,
    threshold: i32,
    count: i32,
}

impl TickThrottler {
    pub const fn new(increment_step: i32, threshold: i32) -> Self {
        Self {
            increment_step,
            threshold,
            count: 0,
        }
    }

    pub fn increment(&mut self) {
        self.count += self.increment_step;
    }

    pub fn tick(&mut self) {
        if self.count > 0 {
            self.count -= 1;
        }
    }

    pub const fn is_under_threshold(self) -> bool {
        self.count < self.threshold
    }

    pub const fn count(self) -> i32 {
        self.count
    }
}

#[cfg(test)]
mod tests {
    use super::TickThrottler;

    #[cfg(vibecraft_has_decompiled_sources)]
    #[test]
    fn tick_throttler_matches_java_source() {
        const JAVA: &str = vibecraft_java_source!("/net/minecraft/util/TickThrottler.java");
        assert_eq!(JAVA.lines().count(), 26);
        for fragment in [
            "private final int incrementStep",
            "private final int threshold",
            "private int count",
            "this.count = this.count + this.incrementStep",
            "if (this.count > 0)",
            "return this.count < this.threshold",
        ] {
            assert!(JAVA.contains(fragment), "missing TickThrottler source fragment: {fragment}");
        }
    }

    #[test]
    fn tick_throttler_increments_decays_and_uses_strict_threshold() {
        let mut throttler = TickThrottler::new(3, 4);
        assert!(throttler.is_under_threshold());
        throttler.increment();
        assert_eq!(throttler.count(), 3);
        assert!(throttler.is_under_threshold());
        throttler.increment();
        assert_eq!(throttler.count(), 6);
        assert!(!throttler.is_under_threshold());
        throttler.tick();
        assert_eq!(throttler.count(), 5);
        for _ in 0..8 {
            throttler.tick();
        }
        assert_eq!(throttler.count(), 0);
        throttler.tick();
        assert_eq!(throttler.count(), 0);
    }
}
