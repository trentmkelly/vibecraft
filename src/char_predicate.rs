#![allow(dead_code)]

use std::fmt;
use std::sync::Arc;

/// Java `net.minecraft.CharPredicate` parity model.
///
/// Java `char` is a UTF-16 code unit, not a Unicode scalar value, so this API
/// intentionally accepts `u16` rather than Rust `char`.
#[derive(Clone)]
pub struct JavaCharPredicate {
    predicate: Arc<dyn Fn(u16) -> bool + Send + Sync + 'static>,
}

impl JavaCharPredicate {
    pub fn new(predicate: impl Fn(u16) -> bool + Send + Sync + 'static) -> Self {
        Self {
            predicate: Arc::new(predicate),
        }
    }

    pub fn test(&self, value: u16) -> bool {
        (self.predicate)(value)
    }

    pub fn and(&self, other: Self) -> Self {
        let left = self.clone();
        Self::new(move |value| left.test(value) && other.test(value))
    }

    pub fn negate(&self) -> Self {
        let predicate = self.clone();
        Self::new(move |value| !predicate.test(value))
    }

    pub fn or(&self, other: Self) -> Self {
        let left = self.clone();
        Self::new(move |value| left.test(value) || other.test(value))
    }
}

impl fmt::Debug for JavaCharPredicate {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("JavaCharPredicate")
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use super::JavaCharPredicate;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    #[test]
    fn char_predicate_test_uses_java_char_code_units() {
        let high_surrogate = JavaCharPredicate::new(|value| (0xD800..=0xDBFF).contains(&value));

        assert!(high_surrogate.test(0xD800));
        assert!(high_surrogate.test(0xDBFF));
        assert!(!high_surrogate.test(u16::from(b'A')));
        assert!(!high_surrogate.test(0xFFFF));
    }

    #[test]
    fn char_predicate_and_matches_java_short_circuit_semantics() {
        let right_calls = Arc::new(AtomicUsize::new(0));
        let calls = Arc::clone(&right_calls);
        let left = JavaCharPredicate::new(|value| value == u16::from(b'a'));
        let right = JavaCharPredicate::new(move |value| {
            calls.fetch_add(1, Ordering::SeqCst);
            (u16::from(b'a')..=u16::from(b'z')).contains(&value)
        });
        let combined = left.and(right);

        assert!(combined.test(u16::from(b'a')));
        assert!(!combined.test(u16::from(b'A')));
        assert_eq!(right_calls.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn char_predicate_or_matches_java_short_circuit_semantics() {
        let right_calls = Arc::new(AtomicUsize::new(0));
        let calls = Arc::clone(&right_calls);
        let left = JavaCharPredicate::new(|value| value == u16::from(b'a'));
        let right = JavaCharPredicate::new(move |value| {
            calls.fetch_add(1, Ordering::SeqCst);
            value == u16::from(b'b')
        });
        let combined = left.or(right);

        assert!(combined.test(u16::from(b'a')));
        assert!(combined.test(u16::from(b'b')));
        assert!(!combined.test(u16::from(b'c')));
        assert_eq!(right_calls.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn char_predicate_negate_matches_java_default_method() {
        let digit =
            JavaCharPredicate::new(|value| (u16::from(b'0')..=u16::from(b'9')).contains(&value));
        let not_digit = digit.negate();

        assert!(!not_digit.test(u16::from(b'7')));
        assert!(not_digit.test(u16::from(b'x')));
    }
}
