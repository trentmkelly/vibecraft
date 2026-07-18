//! Inclusive ordered ranges and interval codec forms matching Minecraft's utility.

#![allow(dead_code)]

use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct InclusiveRange<T> {
    min_inclusive: T,
    max_inclusive: T,
}

impl<T: Ord> InclusiveRange<T> {
    pub fn new(min_inclusive: T, max_inclusive: T) -> Result<Self, String> {
        if min_inclusive > max_inclusive {
            return Err("min_inclusive must be less than or equal to max_inclusive".to_string());
        }
        Ok(Self {
            min_inclusive,
            max_inclusive,
        })
    }

    pub fn single(value: T) -> Self
    where
        T: Clone,
    {
        Self {
            min_inclusive: value.clone(),
            max_inclusive: value,
        }
    }

    pub fn create(min_inclusive: T, max_inclusive: T) -> Result<Self, String> {
        Self::new(min_inclusive, max_inclusive)
    }

    pub fn min_inclusive(&self) -> &T {
        &self.min_inclusive
    }

    pub fn max_inclusive(&self) -> &T {
        &self.max_inclusive
    }

    pub fn map<S: Ord>(&self, mapper: impl Fn(&T) -> S) -> Result<InclusiveRange<S>, String> {
        InclusiveRange::new(mapper(&self.min_inclusive), mapper(&self.max_inclusive))
    }

    pub fn is_value_in_range(&self, value: &T) -> bool {
        value >= &self.min_inclusive && value <= &self.max_inclusive
    }

    pub fn contains(&self, sub_range: &Self) -> bool {
        sub_range.min_inclusive() >= self.min_inclusive()
            && sub_range.max_inclusive() <= self.max_inclusive()
    }

    pub fn validate_bounds(&self, min_allowed_inclusive: &T, max_allowed_inclusive: &T) -> Result<(), String>
    where
        T: fmt::Display,
    {
        if &self.min_inclusive < min_allowed_inclusive {
            return Err(format!(
                "Range limit too low, expected at least {min_allowed_inclusive} [{}-{}]",
                self.min_inclusive, self.max_inclusive
            ));
        }
        if &self.max_inclusive > max_allowed_inclusive {
            return Err(format!(
                "Range limit too high, expected at most {max_allowed_inclusive} [{}-{}]",
                self.min_inclusive, self.max_inclusive
            ));
        }
        Ok(())
    }
}

impl InclusiveRange<i32> {
    /// Java's `INT` codec accepts a point, a two-value array, or interval object.
    pub fn decode_json(value: &serde_json::Value) -> Result<Self, String> {
        if let Some(point) = value.as_i64() {
            let point = i32::try_from(point).map_err(|_| "range point is out of i32 range".to_string())?;
            return Ok(Self::single(point));
        }
        if let Some(values) = value.as_array() {
            if values.len() != 2 {
                return Err(format!("range array must have 2 values, got {}", values.len()));
            }
            return Self::new(json_i32(&values[0], "range minimum")?, json_i32(&values[1], "range maximum")?);
        }
        let object = value
            .as_object()
            .ok_or_else(|| "range must be an integer, two-value array, or object".to_string())?;
        Self::new(
            json_i32(
                object
                    .get("min_inclusive")
                    .ok_or_else(|| "range object is missing min_inclusive".to_string())?,
                "min_inclusive",
            )?,
            json_i32(
                object
                    .get("max_inclusive")
                    .ok_or_else(|| "range object is missing max_inclusive".to_string())?,
                "max_inclusive",
            )?,
        )
    }

    /// Java's `intervalCodec` writes a single point when both bounds are equal,
    /// otherwise its first alternative writes the two-value array form.
    pub fn encode_json(&self) -> serde_json::Value {
        if self.min_inclusive == self.max_inclusive {
            serde_json::json!(self.min_inclusive)
        } else {
            serde_json::json!([self.min_inclusive, self.max_inclusive])
        }
    }
}

fn json_i32(value: &serde_json::Value, field: &str) -> Result<i32, String> {
    let value = value
        .as_i64()
        .ok_or_else(|| format!("{field} must be an integer"))?;
    i32::try_from(value).map_err(|_| format!("{field} is out of i32 range"))
}

impl<T: fmt::Display> fmt::Display for InclusiveRange<T> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "[{}, {}]", self.min_inclusive, self.max_inclusive)
    }
}

#[cfg(test)]
mod tests {
    use super::InclusiveRange;

    #[cfg(vibecraft_has_decompiled_sources)]
    #[test]
    fn inclusive_range_matches_java_source() {
        const JAVA: &str = vibecraft_java_source!("/net/minecraft/util/InclusiveRange.java");
        assert_eq!(JAVA.lines().count(), 67);
        for fragment in [
            "public record InclusiveRange<T extends Comparable<T>>(T minInclusive, T maxInclusive)",
            "public static final Codec<InclusiveRange<Integer>> INT = codec(Codec.INT)",
            "min_inclusive must be less than or equal to max_inclusive",
            "public InclusiveRange(final T value)",
            "ExtraCodecs.intervalCodec(",
            "public boolean isValueInRange(final T value)",
            "public boolean contains(final InclusiveRange<T> subRange)",
        ] {
            assert!(JAVA.contains(fragment), "missing InclusiveRange source fragment: {fragment}");
        }
    }

    #[test]
    fn constructor_mapping_membership_and_containment_match_java() {
        assert_eq!(
            InclusiveRange::new(4, 2),
            Err("min_inclusive must be less than or equal to max_inclusive".to_string())
        );
        let range = InclusiveRange::new(2, 8).unwrap_or_else(|error| panic!("valid range: {error}"));
        assert_eq!(range.to_string(), "[2, 8]");
        assert!(range.is_value_in_range(&2));
        assert!(range.is_value_in_range(&8));
        assert!(!range.is_value_in_range(&9));
        let child = InclusiveRange::new(3, 7).unwrap_or_else(|error| panic!("valid child: {error}"));
        assert!(range.contains(&child));
        assert_eq!(range.map(|value| value.to_string()), Ok(InclusiveRange::new("2".to_string(), "8".to_string()).unwrap_or_else(|error| panic!("mapped range: {error}"))));
    }

    #[test]
    fn integer_codec_uses_java_point_array_object_and_bound_forms() {
        assert_eq!(InclusiveRange::decode_json(&serde_json::json!(5)), Ok(InclusiveRange::single(5)));
        assert_eq!(InclusiveRange::decode_json(&serde_json::json!([2, 6])), Ok(InclusiveRange::new(2, 6).unwrap_or_else(|error| panic!("valid array range: {error}"))));
        assert_eq!(InclusiveRange::decode_json(&serde_json::json!({ "min_inclusive": 2, "max_inclusive": 6 })), Ok(InclusiveRange::new(2, 6).unwrap_or_else(|error| panic!("valid object range: {error}"))));
        let range = InclusiveRange::new(2, 6).unwrap_or_else(|error| panic!("valid range: {error}"));
        assert_eq!(range.encode_json(), serde_json::json!([2, 6]));
        assert_eq!(InclusiveRange::single(5).encode_json(), serde_json::json!(5));
        assert!(range.validate_bounds(&3, &8).is_err());
        assert!(range.validate_bounds(&0, &5).is_err());
        assert_eq!(range.validate_bounds(&0, &8), Ok(()));
    }
}
