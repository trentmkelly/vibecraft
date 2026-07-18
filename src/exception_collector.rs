//! First-error aggregation with suppressed follow-up errors, matching Java's collector.

#![allow(dead_code)]

/// The primary failure Java would throw, plus later failures attached as suppressed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CollectedException<T> {
    primary: T,
    suppressed: Vec<T>,
}

impl<T> CollectedException<T> {
    pub fn primary(&self) -> &T {
        &self.primary
    }

    pub fn suppressed(&self) -> &[T] {
        &self.suppressed
    }
}

/// Collects failures, retaining the first as the eventual thrown error.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ExceptionCollector<T> {
    result: Option<CollectedException<T>>,
}

impl<T> ExceptionCollector<T> {
    pub const fn new() -> Self {
        Self { result: None }
    }

    /// Java `add`: first failure becomes the result; later failures are suppressed.
    pub fn add(&mut self, throwable: T) {
        if let Some(result) = &mut self.result {
            result.suppressed.push(throwable);
        } else {
            self.result = Some(CollectedException {
                primary: throwable,
                suppressed: Vec::new(),
            });
        }
    }

    /// Rust equivalent of Java `throwIfPresent`, which leaves the stored result intact.
    pub fn throw_if_present(&self) -> Result<(), &CollectedException<T>> {
        self.result.as_ref().map_or(Ok(()), Err)
    }

    pub fn result(&self) -> Option<&CollectedException<T>> {
        self.result.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::ExceptionCollector;

    #[cfg(vibecraft_has_decompiled_sources)]
    #[test]
    fn exception_collector_matches_java_source() {
        const JAVA: &str = vibecraft_java_source!("/net/minecraft/util/ExceptionCollector.java");
        assert_eq!(JAVA.lines().count(), 21);
        for fragment in [
            "public class ExceptionCollector<T extends Throwable>",
            "private @Nullable T result",
            "public void add(final T throwable)",
            "this.result.addSuppressed(throwable)",
            "public void throwIfPresent() throws T",
            "throw this.result",
        ] {
            assert!(
                JAVA.contains(fragment),
                "missing ExceptionCollector source fragment: {fragment}"
            );
        }
    }

    #[test]
    fn first_error_is_preserved_and_later_errors_are_suppressed() {
        let mut collector = ExceptionCollector::new();
        assert_eq!(collector.throw_if_present(), Ok(()));
        collector.add("first");
        collector.add("second");
        collector.add("third");
        let Err(error) = collector.throw_if_present() else {
            panic!("first error must be present");
        };
        assert_eq!(error.primary(), &"first");
        assert_eq!(error.suppressed(), ["second", "third"]);
        assert_eq!(collector.result(), Some(error));
    }
}
