#![allow(dead_code)]

use std::sync::Arc;

#[derive(Clone)]
pub enum ChunkResultModel<T> {
    Success(T),
    Fail(Arc<dyn Fn() -> String + Send + Sync>),
}

impl<T> ChunkResultModel<T> {
    pub fn of(value: T) -> Self {
        Self::Success(value)
    }

    pub fn error(error: impl Into<String>) -> Self {
        let error = error.into();
        Self::error_supplier(move || error.clone())
    }

    pub fn error_supplier(error: impl Fn() -> String + Send + Sync + 'static) -> Self {
        Self::Fail(Arc::new(error))
    }

    pub fn is_success(&self) -> bool {
        matches!(self, Self::Success(_))
    }

    pub fn or_else(&self, fallback: T) -> T
    where
        T: Clone,
    {
        match self {
            Self::Success(value) => value.clone(),
            Self::Fail(_) => fallback,
        }
    }

    pub fn get_error(&self) -> Option<String> {
        match self {
            Self::Success(_) => None,
            Self::Fail(error) => Some(error()),
        }
    }

    pub fn if_success(&self, mut consumer: impl FnMut(&T)) -> &Self {
        if let Self::Success(value) = self {
            consumer(value);
        }
        self
    }

    pub fn map<R>(&self, map: impl FnOnce(&T) -> R) -> ChunkResultModel<R> {
        match self {
            Self::Success(value) => ChunkResultModel::Success(map(value)),
            Self::Fail(error) => ChunkResultModel::Fail(Arc::clone(error)),
        }
    }

    pub fn or_else_throw<E>(&self, exception_supplier: impl FnOnce() -> E) -> Result<T, E>
    where
        T: Clone,
    {
        match self {
            Self::Success(value) => Ok(value.clone()),
            Self::Fail(_) => Err(exception_supplier()),
        }
    }
}

impl ChunkResultModel<()> {
    pub fn static_or_else<R>(chunk_result: &ChunkResultModel<R>, fallback: R) -> R
    where
        R: Clone,
    {
        chunk_result.or_else(fallback)
    }
}

impl<T: std::fmt::Debug> std::fmt::Debug for ChunkResultModel<T> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Success(value) => formatter.debug_tuple("Success").field(value).finish(),
            Self::Fail(_) => formatter.write_str("Fail(<supplier>)"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    #[test]
    fn success_result_matches_java_success_methods() {
        let success = ChunkResultModel::of(7);
        let mut observed = Vec::new();

        assert!(success.is_success());
        assert_eq!(success.or_else(99), 7);
        assert_eq!(ChunkResultModel::static_or_else(&success, 99), 7);
        assert_eq!(success.get_error(), None);
        assert!(std::ptr::eq(success.if_success(|value| observed.push(*value)), &success));
        assert_eq!(observed, vec![7]);
        assert_eq!(success.map(|value| value * 3).or_else(0), 21);
        assert_eq!(
            success.or_else_throw(|| "should not be used".to_string()),
            Ok(7)
        );
    }

    #[test]
    fn failure_result_matches_java_fail_methods_and_keeps_lazy_error_supplier() {
        let calls = Arc::new(AtomicUsize::new(0));
        let calls_for_supplier = Arc::clone(&calls);
        let failure: ChunkResultModel<i32> = ChunkResultModel::error_supplier(move || {
            calls_for_supplier.fetch_add(1, Ordering::SeqCst);
            "chunk failed".to_string()
        });
        let mut observed = Vec::new();

        assert!(!failure.is_success());
        assert_eq!(failure.or_else(99), 99);
        assert_eq!(ChunkResultModel::static_or_else(&failure, 42), 42);
        assert_eq!(calls.load(Ordering::SeqCst), 0);
        assert_eq!(failure.get_error(), Some("chunk failed".to_string()));
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        assert!(std::ptr::eq(failure.if_success(|value| observed.push(*value)), &failure));
        assert!(observed.is_empty());
        assert_eq!(failure.or_else_throw(|| "boom".to_string()), Err("boom".to_string()));

        let mapped: ChunkResultModel<String> = failure.map(|value| value.to_string());
        assert!(!mapped.is_success());
        assert_eq!(mapped.get_error(), Some("chunk failed".to_string()));
        assert_eq!(calls.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn string_error_factory_matches_java_error_string_overload() {
        let failure: ChunkResultModel<&str> = ChunkResultModel::error("bad chunk");

        assert!(!failure.is_success());
        assert_eq!(failure.get_error(), Some("bad chunk".to_string()));
    }

    #[test]
    #[cfg(vibecraft_has_decompiled_sources)]
    fn chunk_result_source_matches_java_26_1_2() {
        const CHUNK_RESULT: &str =
            vibecraft_java_source!("/net/minecraft/server/level/ChunkResult.java");

        for sentinel in [
            "public interface ChunkResult<T>",
            "static <T> ChunkResult<T> of(final T value)",
            "return new ChunkResult.Success<>(value);",
            "static <T> ChunkResult<T> error(final String error)",
            "return error(() -> error);",
            "static <T> ChunkResult<T> error(final Supplier<String> errorSupplier)",
            "return new ChunkResult.Fail<>(errorSupplier);",
            "boolean isSuccess();",
            "@Nullable T orElse(@Nullable T orElse);",
            "static <R> @Nullable R orElse(final ChunkResult<? extends R> chunkResult, final @Nullable R orElse)",
            "R result = (R)chunkResult.orElse(null);",
            "return result != null ? result : orElse;",
            "@Nullable String getError();",
            "ChunkResult<T> ifSuccess(Consumer<T> consumer);",
            "<R> ChunkResult<R> map(Function<T, R> map);",
            "<E extends Throwable> T orElseThrow(Supplier<E> exceptionSupplier) throws E;",
            "record Fail<T>(Supplier<String> error) implements ChunkResult<T>",
            "return false;",
            "return orElse;",
            "return this.error.get();",
            "return this;",
            "return new ChunkResult.Fail(this.error);",
            "throw exceptionSupplier.get();",
            "record Success<T>(T value) implements ChunkResult<T>",
            "return true;",
            "return this.value;",
            "return null;",
            "consumer.accept(this.value);",
            "return new ChunkResult.Success<>(map.apply(this.value));",
        ] {
            assert!(
                CHUNK_RESULT.contains(sentinel),
                "ChunkResult.java is missing sentinel: {sentinel}"
            );
        }
    }
}
