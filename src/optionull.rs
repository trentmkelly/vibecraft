#![allow(dead_code)]

pub fn optionull_or_else<T>(value: Option<T>, default_value: T) -> T {
    value.unwrap_or(default_value)
}

pub fn optionull_map<T, R>(value: Option<T>, map: impl FnOnce(T) -> R) -> Option<R> {
    value.map(map)
}

pub fn optionull_map_or_default<T, R>(
    value: Option<T>,
    map: impl FnOnce(T) -> R,
    default_value: R,
) -> R {
    value.map_or(default_value, map)
}

pub fn optionull_map_or_else<T, R>(
    value: Option<T>,
    map: impl FnOnce(T) -> R,
    else_supplier: impl FnOnce() -> R,
) -> R {
    match value {
        Some(value) => map(value),
        None => else_supplier(),
    }
}

pub fn optionull_first<T>(collection: &[T]) -> Option<&T> {
    collection.first()
}

pub fn optionull_first_or_default<'a, T>(collection: &'a [T], default_value: &'a T) -> &'a T {
    collection.first().unwrap_or(default_value)
}

pub fn optionull_first_or_else<'a, T>(
    collection: &'a [T],
    else_supplier: impl FnOnce() -> &'a T,
) -> &'a T {
    match collection.first() {
        Some(value) => value,
        None => else_supplier(),
    }
}

pub fn optionull_is_null_or_empty<T>(array: Option<&[T]>) -> bool {
    array.is_none_or(<[T]>::is_empty)
}

#[cfg(test)]
mod tests {
    use super::{
        optionull_first, optionull_first_or_default, optionull_first_or_else,
        optionull_is_null_or_empty, optionull_map, optionull_map_or_default, optionull_map_or_else,
        optionull_or_else,
    };
    use std::cell::Cell;

    #[test]
    fn optionull_or_else_and_map_match_java_null_branches() {
        assert_eq!(optionull_or_else(Some("value"), "fallback"), "value");
        assert_eq!(optionull_or_else(None, "fallback"), "fallback");

        assert_eq!(optionull_map(Some(4), |value| value * 2), Some(8));
        assert_eq!(optionull_map::<i32, i32>(None, |value| value * 2), None);

        assert_eq!(
            optionull_map_or_default(Some("abc"), |value| value.len(), 99),
            3
        );
        assert_eq!(
            optionull_map_or_default::<&str, usize>(None, |value| value.len(), 99),
            99
        );
    }

    #[test]
    fn optionull_map_or_else_matches_java_supplier_laziness() {
        let else_calls = Cell::new(0);
        let mapped = optionull_map_or_else(
            Some("abc"),
            |value| value.len(),
            || {
                else_calls.set(else_calls.get() + 1);
                99
            },
        );
        assert_eq!(mapped, 3);
        assert_eq!(else_calls.get(), 0);

        let null_mapped = optionull_map_or_else(
            None::<&str>,
            |value| value.len(),
            || {
                else_calls.set(else_calls.get() + 1);
                99
            },
        );
        assert_eq!(null_mapped, 99);
        assert_eq!(else_calls.get(), 1);
    }

    #[test]
    fn optionull_first_helpers_match_java_iterator_order_and_laziness() {
        let values = ["first", "second"];
        let default = "default";
        let fallback = "fallback";
        let else_calls = Cell::new(0);

        assert_eq!(optionull_first(&values), Some(&"first"));
        assert_eq!(optionull_first::<&str>(&[]), None);
        assert_eq!(optionull_first_or_default(&values, &default), &"first");
        assert_eq!(
            optionull_first_or_default::<&str>(&[], &default),
            &"default"
        );

        assert_eq!(
            optionull_first_or_else(&values, || {
                else_calls.set(else_calls.get() + 1);
                &fallback
            }),
            &"first"
        );
        assert_eq!(else_calls.get(), 0);
        assert_eq!(
            optionull_first_or_else::<&str>(&[], || {
                else_calls.set(else_calls.get() + 1);
                &fallback
            }),
            &"fallback"
        );
        assert_eq!(else_calls.get(), 1);
    }

    #[test]
    fn optionull_is_null_or_empty_matches_all_java_array_overloads() {
        assert!(optionull_is_null_or_empty::<i32>(None));
        assert!(optionull_is_null_or_empty(Some(&[] as &[i32])));
        assert!(!optionull_is_null_or_empty(Some(&[1, 2, 3])));

        assert!(optionull_is_null_or_empty(Some(&[] as &[bool])));
        assert!(!optionull_is_null_or_empty(Some(&[true])));
        assert!(optionull_is_null_or_empty(Some(&[] as &[u8])));
        assert!(!optionull_is_null_or_empty(Some(&[0_u8])));
        assert!(optionull_is_null_or_empty(Some(&[] as &[char])));
        assert!(!optionull_is_null_or_empty(Some(&['a'])));
        assert!(optionull_is_null_or_empty(Some(&[] as &[i16])));
        assert!(!optionull_is_null_or_empty(Some(&[1_i16])));
        assert!(optionull_is_null_or_empty(Some(&[] as &[i64])));
        assert!(!optionull_is_null_or_empty(Some(&[1_i64])));
        assert!(optionull_is_null_or_empty(Some(&[] as &[f32])));
        assert!(!optionull_is_null_or_empty(Some(&[1.0_f32])));
        assert!(optionull_is_null_or_empty(Some(&[] as &[f64])));
        assert!(!optionull_is_null_or_empty(Some(&[1.0_f64])));
        assert!(optionull_is_null_or_empty(Some(&[] as &[&str])));
        assert!(!optionull_is_null_or_empty(Some(&["x"])));
    }
}
