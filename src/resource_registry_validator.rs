#![allow(dead_code)]

use std::collections::BTreeMap;
use std::marker::PhantomData;

use crate::registry::{Identifier, Registry, ResourceKey};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegistryValidatorKind {
    None,
    NonEmpty,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegistryValidatorModel<T> {
    kind: RegistryValidatorKind,
    marker: PhantomData<fn(&T)>,
}

impl<T> RegistryValidatorModel<T> {
    pub fn none() -> Self {
        Self {
            kind: RegistryValidatorKind::None,
            marker: PhantomData,
        }
    }

    pub fn non_empty() -> Self {
        Self {
            kind: RegistryValidatorKind::NonEmpty,
            marker: PhantomData,
        }
    }

    pub fn kind(&self) -> RegistryValidatorKind {
        self.kind
    }

    pub fn validate(&self, registry: &Registry<T>, loading_errors: &mut BTreeMap<String, String>) {
        match self.kind {
            RegistryValidatorKind::None => {}
            RegistryValidatorKind::NonEmpty => {
                if registry.iter().next().is_none() {
                    let key = registry_key_for::<T>(registry.registry_id());
                    loading_errors.insert(
                        key.java_to_string(),
                        format!("Registry must be non-empty: {}", registry.registry_id()),
                    );
                }
            }
        }
    }
}

fn registry_key_for<T>(registry_id: &Identifier) -> ResourceKey<Registry<T>> {
    ResourceKey::create_registry_key(registry_id.clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::registry::Lifecycle;

    const REGISTRY_VALIDATOR_JAVA: &str =
        vibecraft_java_source!("/net/minecraft/resources/RegistryValidator.java");

    #[test]
    fn none_validator_is_singleton_noop_like_java() {
        assert_java_contains(
            REGISTRY_VALIDATOR_JAVA,
            &[
                "@FunctionalInterface",
                "RegistryValidator<?> NONE = (var0, var1) -> {};",
                "static <T> RegistryValidator<T> none()",
                "return (RegistryValidator<T>)NONE;",
                "void validate(Registry<T> registry, Map<ResourceKey<?>, Exception> loadingErrors);",
            ],
        );

        let registry = Registry::<String>::new(Identifier::parse("minecraft:wolf_variant").unwrap());
        let mut loading_errors = BTreeMap::new();
        RegistryValidatorModel::none().validate(&registry, &mut loading_errors);

        assert!(loading_errors.is_empty());
    }

    #[test]
    fn non_empty_validator_records_java_error_for_empty_registry() {
        assert_java_contains(
            REGISTRY_VALIDATOR_JAVA,
            &[
                "RegistryValidator<?> NON_EMPTY = (registry, loadingErrors) -> {",
                "if (registry.size() == 0)",
                "loadingErrors.put(registry.key(), new IllegalStateException(\"Registry must be non-empty: \" + registry.key().identifier()));",
                "static <T> RegistryValidator<T> nonEmpty()",
                "return (RegistryValidator<T>)NON_EMPTY;",
            ],
        );

        let registry = Registry::<String>::new(Identifier::parse("minecraft:wolf_variant").unwrap());
        let mut loading_errors = BTreeMap::new();
        RegistryValidatorModel::non_empty().validate(&registry, &mut loading_errors);

        assert_eq!(
            loading_errors,
            BTreeMap::from([(
                "ResourceKey[minecraft:root / minecraft:wolf_variant]".to_string(),
                "Registry must be non-empty: minecraft:wolf_variant".to_string(),
            )])
        );
    }

    #[test]
    fn non_empty_validator_accepts_populated_registry_without_mutating_errors() {
        let mut registry =
            Registry::<String>::new(Identifier::parse("minecraft:wolf_variant").unwrap());
        registry
            .register(
                Identifier::parse("minecraft:pale").unwrap(),
                "pale".to_string(),
                Lifecycle::Stable,
            )
            .unwrap();
        let mut loading_errors = BTreeMap::from([(
            "ResourceKey[minecraft:root / minecraft:other]".to_string(),
            "existing error".to_string(),
        )]);

        RegistryValidatorModel::non_empty().validate(&registry, &mut loading_errors);

        assert_eq!(
            loading_errors,
            BTreeMap::from([(
                "ResourceKey[minecraft:root / minecraft:other]".to_string(),
                "existing error".to_string(),
            )])
        );
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
