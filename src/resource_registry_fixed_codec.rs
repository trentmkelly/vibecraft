#![allow(dead_code)]

use std::marker::PhantomData;

use crate::registry::{Identifier, Registry, ResourceKey};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FixedHolderModel<E> {
    Reference {
        id: Identifier,
        owner: String,
        marker: PhantomData<E>,
    },
    Direct {
        value: E,
        owner: String,
    },
}

impl<E> FixedHolderModel<E> {
    pub fn reference(id: Identifier, owner: impl Into<String>) -> Self {
        Self::Reference {
            id,
            owner: owner.into(),
            marker: PhantomData,
        }
    }

    pub fn direct(value: E, owner: impl Into<String>) -> Self {
        Self::Direct {
            value,
            owner: owner.into(),
        }
    }

    pub fn owner(&self) -> &str {
        match self {
            Self::Reference { owner, .. } | Self::Direct { owner, .. } => owner,
        }
    }

    pub fn can_serialize_in(&self, owner: &str) -> bool {
        self.owner() == owner
    }

    pub fn describe(&self) -> String
    where
        E: std::fmt::Debug,
    {
        match self {
            Self::Reference { id, .. } => format!("Reference({id})"),
            Self::Direct { value, .. } => format!("Direct({value:?})"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegistryFixedOps<E> {
    owner: Option<String>,
    values: Vec<(Identifier, FixedHolderModel<E>)>,
}

impl<E: Clone> RegistryFixedOps<E> {
    pub fn missing_registry() -> Self {
        Self {
            owner: None,
            values: Vec::new(),
        }
    }

    pub fn with_owner(owner: impl Into<String>, values: Vec<(Identifier, FixedHolderModel<E>)>) -> Self {
        Self {
            owner: Some(owner.into()),
            values,
        }
    }

    fn owner(&self) -> Option<&str> {
        self.owner.as_deref()
    }

    fn get(&self, key: &ResourceKey<E>) -> Option<FixedHolderModel<E>> {
        self.values
            .iter()
            .find(|(id, _holder)| id == key.location())
            .map(|(_id, holder)| holder.clone())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FixedCodecOutcome<T> {
    Success { value: T, lifecycle: Option<&'static str> },
    Error(String),
}

impl<T> FixedCodecOutcome<T> {
    pub fn success(value: T) -> Self {
        Self::Success {
            value,
            lifecycle: None,
        }
    }

    pub fn stable(value: T) -> Self {
        Self::Success {
            value,
            lifecycle: Some("stable"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct RegistryFixedCodecModel<E> {
    registry_key: ResourceKey<Registry<E>>,
}

impl<E> RegistryFixedCodecModel<E> {
    pub fn create(registry_key: ResourceKey<Registry<E>>) -> Self {
        Self { registry_key }
    }

    pub fn registry_key(&self) -> &ResourceKey<Registry<E>> {
        &self.registry_key
    }

    pub fn java_to_string(&self) -> String {
        format!("RegistryFixedCodec[{}]", self.registry_key)
    }
}

impl<E: Clone + std::fmt::Debug> RegistryFixedCodecModel<E> {
    pub fn encode(
        &self,
        input: &FixedHolderModel<E>,
        ops: Option<&RegistryFixedOps<E>>,
        prefix: impl Into<String>,
    ) -> FixedCodecOutcome<String> {
        let Some(ops) = ops else {
            return FixedCodecOutcome::Error(format!("Can't access registry {}", self.registry_key));
        };
        let Some(owner) = ops.owner() else {
            return FixedCodecOutcome::Error(format!("Can't access registry {}", self.registry_key));
        };
        if !input.can_serialize_in(owner) {
            return FixedCodecOutcome::Error(format!(
                "Element {} is not valid in current registry set",
                input.describe()
            ));
        }

        match input {
            FixedHolderModel::Reference { id, .. } => {
                FixedCodecOutcome::success(format!("{}+{}", prefix.into(), id))
            }
            FixedHolderModel::Direct { .. } => FixedCodecOutcome::Error(format!(
                "Elements from registry {} can't be serialized to a value",
                self.registry_key
            )),
        }
    }

    pub fn decode(
        &self,
        ops: Option<&RegistryFixedOps<E>>,
        input: &str,
        remainder: impl Into<String>,
    ) -> FixedCodecOutcome<(FixedHolderModel<E>, String)> {
        let Some(ops) = ops else {
            return FixedCodecOutcome::Error(format!("Can't access registry {}", self.registry_key));
        };
        if ops.owner().is_none() {
            return FixedCodecOutcome::Error(format!("Can't access registry {}", self.registry_key));
        }

        let id = match Identifier::parse(input) {
            Ok(id) => id,
            Err(err) => return FixedCodecOutcome::Error(err),
        };
        let element_key = ResourceKey::create(&self.registry_key, id.clone());
        match ops.get(&element_key) {
            Some(holder) => FixedCodecOutcome::stable((holder, remainder.into())),
            None => FixedCodecOutcome::Error(format!("Failed to get element {id}")),
        }
    }
}

impl<E> std::fmt::Display for RegistryFixedCodecModel<E> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.java_to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const REGISTRY_FIXED_CODEC_JAVA: &str =
        vibecraft_java_source!("/net/minecraft/resources/RegistryFixedCodec.java");

    fn wolf_registry_key() -> ResourceKey<Registry<String>> {
        ResourceKey::create_registry_key(Identifier::parse("minecraft:wolf_variant").unwrap())
    }

    #[test]
    fn encode_requires_registry_ops_owner_and_reference_holder_like_java() {
        assert_java_contains(
            REGISTRY_FIXED_CODEC_JAVA,
            &[
                "public final class RegistryFixedCodec<E> implements Codec<Holder<E>>",
                "public static <E> RegistryFixedCodec<E> create(final ResourceKey<? extends Registry<E>> registryKey)",
                "if (ops instanceof RegistryOps<?> registryOps)",
                "Optional<HolderOwner<E>> maybeOwner = registryOps.owner(this.registryKey);",
                "if (!input.canSerializeIn(maybeOwner.get()))",
                "return DataResult.error(() -> \"Element \" + input + \" is not valid in current registry set\");",
                "id -> Identifier.CODEC.encode(id.identifier(), ops, prefix)",
                "value -> DataResult.error(() -> \"Elements from registry \" + this.registryKey + \" can't be serialized to a value\")",
                "return DataResult.error(() -> \"Can't access registry \" + this.registryKey);",
            ],
        );

        let codec = RegistryFixedCodecModel::create(wolf_registry_key());
        let ops = RegistryFixedOps::with_owner(
            "owner",
            vec![(
                Identifier::parse("minecraft:pale").unwrap(),
                FixedHolderModel::reference(Identifier::parse("minecraft:pale").unwrap(), "owner"),
            )],
        );
        let holder = FixedHolderModel::<String>::reference(
            Identifier::parse("minecraft:pale").unwrap(),
            "owner",
        );
        assert_eq!(
            codec.encode(&holder, Some(&ops), "prefix"),
            FixedCodecOutcome::success("prefix+minecraft:pale".to_string())
        );

        assert_eq!(
            codec.encode(&holder, None, "prefix"),
            FixedCodecOutcome::Error(
                "Can't access registry ResourceKey[minecraft:root / minecraft:wolf_variant]"
                    .to_string(),
            )
        );
        assert_eq!(
            codec.encode(
                &FixedHolderModel::<String>::reference(
                    Identifier::parse("minecraft:pale").unwrap(),
                    "other",
                ),
                Some(&ops),
                "prefix",
            ),
            FixedCodecOutcome::Error(
                "Element Reference(minecraft:pale) is not valid in current registry set"
                    .to_string(),
            )
        );
        assert_eq!(
            codec.encode(
                &FixedHolderModel::direct("inline".to_string(), "owner"),
                Some(&ops),
                "prefix",
            ),
            FixedCodecOutcome::Error(
                "Elements from registry ResourceKey[minecraft:root / minecraft:wolf_variant] can't be serialized to a value"
                    .to_string(),
            )
        );
    }

    #[test]
    fn decode_requires_registry_ops_lookup_and_marks_success_stable_like_java() {
        assert_java_contains(
            REGISTRY_FIXED_CODEC_JAVA,
            &[
                "Optional<HolderGetter<E>> lookup = registryOps.getter(this.registryKey);",
                "Identifier.CODEC\n               .decode(ops, input)",
                "lookup.get()\n                        .get(ResourceKey.create(this.registryKey, id))",
                ".orElseGet(() -> DataResult.error(() -> \"Failed to get element \" + id))",
                ".map(h -> Pair.of(h, pair.getSecond()))",
                ".setLifecycle(Lifecycle.stable());",
            ],
        );

        let codec = RegistryFixedCodecModel::create(wolf_registry_key());
        let holder = FixedHolderModel::<String>::reference(
            Identifier::parse("minecraft:pale").unwrap(),
            "owner",
        );
        let ops = RegistryFixedOps::with_owner(
            "owner",
            vec![(Identifier::parse("minecraft:pale").unwrap(), holder.clone())],
        );

        assert_eq!(
            codec.decode(Some(&ops), "minecraft:pale", "tail"),
            FixedCodecOutcome::stable((holder, "tail".to_string()))
        );
        assert_eq!(
            codec.decode(Some(&ops), "minecraft:ashen", "tail"),
            FixedCodecOutcome::Error("Failed to get element minecraft:ashen".to_string())
        );
        assert_eq!(
            codec.decode(None, "minecraft:pale", "tail"),
            FixedCodecOutcome::Error(
                "Can't access registry ResourceKey[minecraft:root / minecraft:wolf_variant]"
                    .to_string(),
            )
        );
    }

    #[test]
    fn to_string_matches_java_registry_key_display() {
        assert_java_contains(
            REGISTRY_FIXED_CODEC_JAVA,
            &["return \"RegistryFixedCodec[\" + this.registryKey + \"]\";"],
        );
        let codec = RegistryFixedCodecModel::<String>::create(wolf_registry_key());
        assert_eq!(
            codec.to_string(),
            "RegistryFixedCodec[ResourceKey[minecraft:root / minecraft:wolf_variant]]"
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
