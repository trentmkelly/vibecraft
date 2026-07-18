//! ID-to-value lookup builders matching Minecraft's `ByIdMap`.
//!
//! Java builds these maps eagerly and rejects invalid value sets with diagnostic
//! messages that include the conflicting values.  Keeping that validation at
//! construction time prevents an enum-like id map from failing later at a less
//! useful lookup site.
#![allow(dead_code)]

use std::fmt::Display;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutOfBoundsStrategy {
    Zero,
    Wrap,
    Clamp,
}

/// Builds a sparse id map, returning `default` for ids which are not present.
pub fn sparse<T: Clone + Display>(
    id: impl Fn(&T) -> i32,
    values: &[T],
    default: T,
) -> Result<impl Fn(i32) -> T, String> {
    let pairs = checked_pairs(id, values)?;

    Ok(move |key| {
        pairs
            .iter()
            .find(|(entry, _)| *entry == key)
            .map_or_else(|| default.clone(), |(_, value)| value.clone())
    })
}

/// Builds a dense id map whose ids must be every index from zero through
/// `values.len() - 1`.
pub fn continuous<T: Clone + Display>(
    id: impl Fn(&T) -> i32,
    values: &[T],
    strategy: OutOfBoundsStrategy,
) -> Result<impl Fn(i32) -> T, String> {
    if values.is_empty() {
        return Err("Empty value list".to_owned());
    }

    let mut sorted = vec![None; values.len()];
    for value in values {
        let key = id(value);
        if key < 0 || key as usize >= values.len() {
            return Err(format!(
                "Values are not continous, found index {key} for value {value}"
            ));
        }

        let slot = &mut sorted[key as usize];
        if let Some(previous) = slot {
            return Err(duplicate_id_error(key, value, previous));
        }
        *slot = Some(value.clone());
    }

    let sorted = sorted
        .into_iter()
        .enumerate()
        .map(|(index, value)| value.ok_or_else(|| format!("Missing value at index: {index}")))
        .collect::<Result<Vec<_>, _>>()?;
    let length = sorted.len() as i32;

    Ok(move |key| {
        let index = match strategy {
            OutOfBoundsStrategy::Zero if key < 0 || key >= length => 0,
            OutOfBoundsStrategy::Zero => key,
            OutOfBoundsStrategy::Wrap => key.rem_euclid(length),
            OutOfBoundsStrategy::Clamp => key.clamp(0, length - 1),
        };
        sorted[index as usize].clone()
    })
}

fn checked_pairs<T: Clone + Display>(
    id: impl Fn(&T) -> i32,
    values: &[T],
) -> Result<Vec<(i32, T)>, String> {
    if values.is_empty() {
        return Err("Empty value list".to_owned());
    }

    let mut result = Vec::with_capacity(values.len());
    for value in values {
        let key = id(value);
        if let Some((_, previous)) = result.iter().find(|(seen, _)| *seen == key) {
            return Err(duplicate_id_error(key, value, previous));
        }
        result.push((key, value.clone()));
    }
    Ok(result)
}

fn duplicate_id_error<T: Display>(id: i32, current: &T, previous: &T) -> String {
    format!("Duplicate entry on id {id}: current={current}, previous={previous}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fmt::{self, Display};

    #[derive(Clone)]
    struct Entry(i32, &'static str);

    impl Display for Entry {
        fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(formatter, "({}, {})", self.0, self.1)
        }
    }

    #[cfg(vibecraft_has_decompiled_sources)]
    #[test]
    fn source() {
        const JAVA: &str = vibecraft_java_source!("/net/minecraft/util/ByIdMap.java");
        assert_eq!(JAVA.lines().count(), 85);
        for fragment in [
            "Empty value list",
            "Duplicate entry on id ",
            "current=",
            "previous=",
            "Values are not continous",
            "Missing value at index:",
            "case WRAP",
            "case CLAMP",
        ] {
            assert!(JAVA.contains(fragment));
        }
    }

    #[test]
    fn sparse_lookup_returns_default_for_an_unknown_id() {
        let map = sparse(|value: &Entry| value.0, &[Entry(2, "two")], Entry(0, "zero"))
            .unwrap_or_else(|error| panic!("{error}"));

        assert_eq!(map(2).1, "two");
        assert_eq!(map(8).1, "zero");
    }

    #[test]
    fn continuous_lookup_applies_each_out_of_bounds_strategy() {
        for (strategy, expected) in [
            (OutOfBoundsStrategy::Zero, 0),
            (OutOfBoundsStrategy::Wrap, 2),
            (OutOfBoundsStrategy::Clamp, 2),
        ] {
            let map = continuous(|value: &i32| *value, &[0, 1, 2], strategy)
                .unwrap_or_else(|error| panic!("{error}"));
            assert_eq!(map(5), expected);
        }
    }

    #[test]
    fn validation_errors_match_java_diagnostics() {
        let duplicate = match sparse(
            |value: &Entry| value.0,
            &[Entry(4, "first"), Entry(4, "second")],
            Entry(0, "default"),
        ) {
            Err(error) => error,
            Ok(_) => panic!("duplicate ids must be rejected"),
        };
        assert_eq!(duplicate, "Duplicate entry on id 4: current=(4, second), previous=(4, first)");

        let outside = match continuous(
            |value: &Entry| value.0,
            &[Entry(3, "outside")],
            OutOfBoundsStrategy::Zero,
        ) {
            Err(error) => error,
            Ok(_) => panic!("out-of-range ids must be rejected"),
        };
        assert_eq!(outside, "Values are not continous, found index 3 for value (3, outside)");

        let duplicate = match continuous(
            |value: &Entry| value.0,
            &[Entry(0, "first"), Entry(0, "second")],
            OutOfBoundsStrategy::Zero,
        ) {
            Err(error) => error,
            Ok(_) => panic!("duplicate ids must be rejected"),
        };
        assert_eq!(duplicate, "Duplicate entry on id 0: current=(0, second), previous=(0, first)");
    }
}
