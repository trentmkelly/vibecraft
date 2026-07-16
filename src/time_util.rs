#![allow(dead_code)]

use crate::random_source::{random_source_next_i32, RandomSourceKind};

/// Java `UniformInt` value provider.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UniformInt {
    pub min_inclusive: i32,
    pub max_inclusive: i32,
}

impl UniformInt {
    pub const fn of(min_inclusive: i32, max_inclusive: i32) -> Self {
        Self {
            min_inclusive,
            max_inclusive,
        }
    }

    pub fn validate(self) -> Result<Self, String> {
        if self.max_inclusive < self.min_inclusive {
            Err(format!(
                "Max must be at least min, min_inclusive: {}, max_inclusive: {}",
                self.min_inclusive, self.max_inclusive
            ))
        } else {
            Ok(self)
        }
    }

    pub fn sample(self, random: &mut RandomSourceKind) -> i32 {
        // This deliberately uses i32 wrapping arithmetic: Java's
        // `Mth.randomBetweenInclusive` computes `max - min + 1` as an int
        // before delegating to `RandomSource.nextInt`, including its invalid
        // (non-positive) bound failure for overflowing ranges.
        let bound = self
            .max_inclusive
            .wrapping_sub(self.min_inclusive)
            .wrapping_add(1);
        self.min_inclusive + random_source_next_i32(random, bound)
    }

    pub fn to_json(self) -> serde_json::Value {
        serde_json::json!({
            "min_inclusive": self.min_inclusive,
            "max_inclusive": self.max_inclusive,
        })
    }

    pub fn from_json(value: &serde_json::Value) -> Result<Self, String> {
        let object = value
            .as_object()
            .ok_or_else(|| "uniform int must be a JSON object".to_string())?;
        let min_inclusive = object
            .get("min_inclusive")
            .and_then(serde_json::Value::as_i64)
            .ok_or_else(|| "uniform int is missing min_inclusive".to_string())?
            .try_into()
            .map_err(|_| "min_inclusive is outside the i32 range".to_string())?;
        let max_inclusive = object
            .get("max_inclusive")
            .and_then(serde_json::Value::as_i64)
            .ok_or_else(|| "uniform int is missing max_inclusive".to_string())?
            .try_into()
            .map_err(|_| "max_inclusive is outside the i32 range".to_string())?;
        Self::of(min_inclusive, max_inclusive).validate()
    }
}

impl std::fmt::Display for UniformInt {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "[{}-{}]", self.min_inclusive, self.max_inclusive)
    }
}

/// Java `TimeUtil` constants and helpers.
pub struct TimeUtil;

impl TimeUtil {
    pub const NANOSECONDS_PER_SECOND: i64 = 1_000_000_000;
    pub const NANOSECONDS_PER_MILLISECOND: i64 = 1_000_000;
    pub const MILLISECONDS_PER_SECOND: i64 = 1_000;
    pub const SECONDS_PER_HOUR: i64 = 3_600;
    pub const SECONDS_PER_MINUTE: i32 = 60;

    pub const fn range_of_seconds(min_inclusive: i32, max_inclusive: i32) -> UniformInt {
        UniformInt::of(min_inclusive * 20, max_inclusive * 20)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(vibecraft_has_decompiled_sources)]
    #[test]
    fn time_util_and_uniform_int_match_java_sources() {
        const TIME_JAVA: &str = vibecraft_java_source!("/net/minecraft/util/TimeUtil.java");
        const UNIFORM_JAVA: &str =
            vibecraft_java_source!("/net/minecraft/util/valueproviders/UniformInt.java");
        assert_eq!(TIME_JAVA.lines().count(), 16);
        for fragment in [
            "NANOSECONDS_PER_SECOND",
            "NANOSECONDS_PER_MILLISECOND",
            "MILLISECONDS_PER_SECOND",
            "SECONDS_PER_HOUR",
            "SECONDS_PER_MINUTE",
            "UniformInt.of(minInclusive * 20, maxInclusive * 20)",
        ] {
            assert!(TIME_JAVA.contains(fragment), "missing TimeUtil source fragment: {fragment}");
        }
        assert_eq!(UNIFORM_JAVA.lines().count(), 41);
        for fragment in [
            "public record UniformInt(int minInclusive, int maxInclusive)",
            "Codec.INT.fieldOf(\"min_inclusive\")",
            "Codec.INT.fieldOf(\"max_inclusive\")",
            "Max must be at least min",
            "return \"[\" + this.minInclusive + \"-\" + this.maxInclusive + \"]\";",
        ] {
            assert!(UNIFORM_JAVA.contains(fragment), "missing UniformInt source fragment: {fragment}");
        }
    }

    #[test]
    fn time_util_constants_and_uniform_int_codec_match_vanilla() {
        assert_eq!(TimeUtil::NANOSECONDS_PER_SECOND, 1_000_000_000);
        assert_eq!(TimeUtil::NANOSECONDS_PER_MILLISECOND, 1_000_000);
        assert_eq!(TimeUtil::MILLISECONDS_PER_SECOND, 1_000);
        assert_eq!(TimeUtil::SECONDS_PER_HOUR, 3_600);
        assert_eq!(TimeUtil::SECONDS_PER_MINUTE, 60);
        let range = TimeUtil::range_of_seconds(3, 7);
        assert_eq!(range, UniformInt::of(60, 140));
        assert_eq!(range.to_string(), "[60-140]");
        assert_eq!(UniformInt::from_json(&range.to_json()), Ok(range));
        assert!(UniformInt::of(10, 9).validate().is_err());

        let mut random = RandomSourceKind::new(1234, crate::random_source::RandomAlgorithm::Legacy);
        for _ in 0..64 {
            assert!((60..=140).contains(&range.sample(&mut random)));
        }
    }
}
