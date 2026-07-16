//! A one-entry cache matching Java's `SingleKeyCache`.

#![allow(dead_code)]

/// Caches the last non-null value produced for a key.
pub struct SingleKeyCache<K, V, F> {
    compute_value: F,
    cache_key: Option<K>,
    cached_value: Option<V>,
}

impl<K, V, F> SingleKeyCache<K, V, F>
where
    K: PartialEq,
    V: Clone,
    F: Fn(&K) -> Option<V>,
{
    pub fn new(compute_value: F) -> Self {
        Self {
            compute_value,
            cache_key: None,
            cached_value: None,
        }
    }

    /// Returns the cached value for the same key, recomputing on a key change
    /// or whenever the computation produced Java's `null` (`None`).
    pub fn get_value(&mut self, cache_key: K) -> Option<V> {
        if self.cached_value.is_none()
            || self.cache_key.as_ref() != Some(&cache_key)
        {
            let value = (self.compute_value)(&cache_key);
            self.cache_key = Some(cache_key);
            self.cached_value = value.clone();
            value
        } else {
            self.cached_value.clone()
        }
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;
    use std::rc::Rc;

    use super::SingleKeyCache;

    #[cfg(vibecraft_has_decompiled_sources)]
    #[test]
    fn single_key_cache_matches_java_source() {
        const JAVA: &str = vibecraft_java_source!("/net/minecraft/util/SingleKeyCache.java");
        assert_eq!(JAVA.lines().count(), 24);
        for fragment in [
            "private final Function<K, V> computeValue",
            "private @Nullable K cacheKey",
            "private @Nullable V cachedValue",
            "if (this.cachedValue == null || !Objects.equals(this.cacheKey, cacheKey))",
            "this.cachedValue = this.computeValue.apply(cacheKey)",
            "return this.cachedValue",
        ] {
            assert!(JAVA.contains(fragment), "missing SingleKeyCache source fragment: {fragment}");
        }
    }

    #[test]
    fn single_key_cache_reuses_non_null_values_and_recomputes_nulls() {
        let calls = Rc::new(Cell::new(0));
        let calls_for_compute = Rc::clone(&calls);
        let mut cache = SingleKeyCache::new(move |key: &String| {
            calls_for_compute.set(calls_for_compute.get() + 1);
            if key == "missing" {
                None
            } else {
                Some(format!("value:{key}"))
            }
        });

        assert_eq!(cache.get_value("first".to_string()), Some("value:first".to_string()));
        assert_eq!(cache.get_value("first".to_string()), Some("value:first".to_string()));
        assert_eq!(calls.get(), 1);
        assert_eq!(cache.get_value("second".to_string()), Some("value:second".to_string()));
        assert_eq!(calls.get(), 2);
        assert_eq!(cache.get_value("missing".to_string()), None);
        assert_eq!(cache.get_value("missing".to_string()), None);
        assert_eq!(calls.get(), 4);
    }
}
