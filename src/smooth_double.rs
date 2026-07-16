//! Smoothed double-delta accumulation matching Minecraft's `SmoothDouble`.

#![allow(dead_code)]

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct SmoothDouble {
    target_value: f64,
    remaining_value: f64,
    last_amount: f64,
}

impl SmoothDouble {
    pub fn get_new_delta_value(&mut self, target_delta: f64, time: f64) -> f64 {
        self.target_value += target_delta;
        let mut delta = self.target_value - self.remaining_value;
        let new_last_amount = 0.5 * (self.last_amount + delta);
        let delta_sign = delta.signum();
        if delta_sign * delta > delta_sign * self.last_amount {
            delta = new_last_amount;
        }

        self.last_amount = new_last_amount;
        self.remaining_value += delta * time;
        delta * time
    }

    pub fn reset(&mut self) {
        self.target_value = 0.0;
        self.remaining_value = 0.0;
        self.last_amount = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::SmoothDouble;

    #[cfg(vibecraft_has_decompiled_sources)]
    #[test]
    fn smooth_double_matches_java_source() {
        const JAVA: &str = vibecraft_java_source!("/net/minecraft/util/SmoothDouble.java");
        assert_eq!(JAVA.lines().count(), 27);
        for fragment in [
            "private double targetValue",
            "private double remainingValue",
            "private double lastAmount",
            "this.targetValue += targetDelta",
            "double newLastAmount = Mth.lerp(0.5, this.lastAmount, delta)",
            "double deltaSign = Math.signum(delta)",
            "this.remainingValue += delta * time",
            "this.targetValue = 0.0",
        ] {
            assert!(JAVA.contains(fragment), "missing SmoothDouble source fragment: {fragment}");
        }
    }

    #[test]
    fn smooth_double_applies_and_resets_vanilla_state_transitions() {
        let mut smooth = SmoothDouble::default();
        assert_eq!(smooth.get_new_delta_value(10.0, 1.0), 5.0);
        assert_eq!(smooth.get_new_delta_value(0.0, 1.0), 5.0);
        assert_eq!(smooth.get_new_delta_value(-10.0, 0.5), -1.25);

        smooth.reset();
        assert_eq!(smooth.get_new_delta_value(0.0, 1.0), 0.0);
    }
}
