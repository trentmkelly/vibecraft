#![allow(dead_code)]

use std::marker::PhantomData;

pub trait DependantName<T, V> {
    fn get(&self, id: &T) -> V;
}

impl<T, V, F> DependantName<T, V> for F
where
    for<'a> F: Fn(&'a T) -> V,
{
    fn get(&self, id: &T) -> V {
        self(id)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FixedDependantName<T, V> {
    value: V,
    marker: PhantomData<fn(&T)>,
}

impl<T, V: Clone> DependantName<T, V> for FixedDependantName<T, V> {
    fn get(&self, _id: &T) -> V {
        self.value.clone()
    }
}

pub fn fixed<T, V: Clone>(value: V) -> FixedDependantName<T, V> {
    FixedDependantName {
        value,
        marker: PhantomData,
    }
}

#[cfg(test)]
mod tests {
    use super::{fixed, DependantName};

    const DEPENDANT_NAME_JAVA: &str =
        vibecraft_java_source!("/net/minecraft/resources/DependantName.java");

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct ResourceKeyFixture(&'static str);

    #[test]
    fn fixed_dependant_name_ignores_key_and_returns_cloned_value() {
        assert_java_contains(
            DEPENDANT_NAME_JAVA,
            &[
                "@FunctionalInterface",
                "public interface DependantName<T, V>",
                "V get(ResourceKey<T> id);",
                "static <T, V> DependantName<T, V> fixed(final V value)",
                "return id -> value;",
            ],
        );

        let name = fixed::<ResourceKeyFixture, String>("constant".to_string());
        assert_eq!(name.get(&ResourceKeyFixture("minecraft:stone")), "constant");
        assert_eq!(name.get(&ResourceKeyFixture("minecraft:dirt")), "constant");
    }

    #[test]
    fn closure_implementation_preserves_key_dependent_behavior() {
        let suffix = |key: &ResourceKeyFixture| format!("name:{}", key.0);
        assert_eq!(suffix.get(&ResourceKeyFixture("minecraft:stone")), "name:minecraft:stone");
        assert_eq!(suffix.get(&ResourceKeyFixture("minecraft:dirt")), "name:minecraft:dirt");
    }

    fn assert_java_contains(source: &str, sentinels: &[&str]) {
        if source.is_empty() {
            return;
        }
        for sentinel in sentinels {
            assert!(
                source.contains(sentinel),
                "missing Java source sentinel {sentinel}"
            );
        }
    }
}
