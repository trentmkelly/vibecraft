//! Callback contract that can stop an iteration, matching Minecraft's utility interface.

#![allow(dead_code)]

/// A callback which reports whether the caller should stop iterating.
pub trait AbortableIterationConsumer<T> {
    fn accept(&mut self, entry: T) -> Continuation;
}

impl<T, F> AbortableIterationConsumer<T> for F
where
    F: FnMut(T) -> Continuation,
{
    fn accept(&mut self, entry: T) -> Continuation {
        self(entry)
    }
}

pub fn for_consumer<T>(mut consumer: impl FnMut(T)) -> impl AbortableIterationConsumer<T> {
    move |entry| {
        consumer(entry);
        Continuation::Continue
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Continuation {
    Continue,
    Abort,
}

impl Continuation {
    pub const fn should_abort(self) -> bool {
        matches!(self, Self::Abort)
    }
}

#[cfg(test)]
mod tests {
    use super::{for_consumer, AbortableIterationConsumer, Continuation};

    #[cfg(vibecraft_has_decompiled_sources)]
    #[test]
    fn abortable_iteration_consumer_matches_java_source() {
        const JAVA: &str =
            vibecraft_java_source!("/net/minecraft/util/AbortableIterationConsumer.java");
        assert_eq!(JAVA.lines().count(), 24);
        for fragment in [
            "public interface AbortableIterationConsumer<T>",
            "AbortableIterationConsumer.Continuation accept(T entry)",
            "static <T> AbortableIterationConsumer<T> forConsumer(final Consumer<T> consumer)",
            "return AbortableIterationConsumer.Continuation.CONTINUE",
            "public boolean shouldAbort()",
        ] {
            assert!(
                JAVA.contains(fragment),
                "missing AbortableIterationConsumer source fragment: {fragment}"
            );
        }
    }

    #[test]
    fn callback_adapter_always_continues_and_enum_reports_abort() {
        let mut seen = Vec::new();
        let mut callback = for_consumer(|entry| seen.push(entry));
        assert_eq!(callback.accept(4), Continuation::Continue);
        assert_eq!(callback.accept(9), Continuation::Continue);
        drop(callback);
        assert_eq!(seen, [4, 9]);
        assert!(!Continuation::Continue.should_abort());
        assert!(Continuation::Abort.should_abort());
    }
}
