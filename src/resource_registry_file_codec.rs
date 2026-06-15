#![allow(dead_code)]

use std::marker::PhantomData;

use crate::registry::{Identifier, Registry, ResourceKey};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileHolderModel<E> {
    Reference {
        id: Identifier,
        value: E,
        owner: String,
        marker: PhantomData<E>,
    },
    Direct {
        value: E,
        owner: String,
    },
}

impl<E: Clone> FileHolderModel<E> {
    pub fn reference(id: Identifier, value: E, owner: impl Into<String>) -> Self {
        Self::Reference {
            id,
            value,
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

    pub fn value(&self) -> E {
        match self {
            Self::Reference { value, .. } | Self::Direct { value, .. } => value.clone(),
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
pub struct RegistryFileOps<E> {
    owner: Option<String>,
    values: Vec<(Identifier, FileHolderModel<E>)>,
}

impl<E: Clone> RegistryFileOps<E> {
    pub fn missing_registry() -> Self {
        Self {
            owner: None,
            values: Vec::new(),
        }
    }

    pub fn with_owner(owner: impl Into<String>, values: Vec<(Identifier, FileHolderModel<E>)>) -> Self {
        Self {
            owner: Some(owner.into()),
            values,
        }
    }

    fn owner(&self) -> Option<&str> {
        self.owner.as_deref()
    }

    fn get(&self, key: &ResourceKey<E>) -> Option<FileHolderModel<E>> {
        self.values
            .iter()
            .find(|(id, _holder)| id == key.location())
            .map(|(_id, holder)| holder.clone())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ElementCodecModel {
    name: &'static str,
}

impl ElementCodecModel {
    pub fn new(name: &'static str) -> Self {
        Self { name }
    }

    fn encode(&self, value: impl AsRef<str>, prefix: impl Into<String>) -> String {
        format!("{}:{}+{}", self.name, prefix.into(), value.as_ref())
    }

    fn decode(&self, input: &str, remainder: impl Into<String>) -> (String, String) {
        (format!("{}:{input}", self.name), remainder.into())
    }
}

impl std::fmt::Display for ElementCodecModel {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.name)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileCodecOutcome<T> {
    Success { value: T, lifecycle: Option<&'static str> },
    Error(String),
}

impl<T> FileCodecOutcome<T> {
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
pub struct RegistryFileCodecModel<E> {
    registry_key: ResourceKey<Registry<E>>,
    element_codec: ElementCodecModel,
    allow_inline: bool,
}

impl<E> RegistryFileCodecModel<E> {
    pub fn create(registry_key: ResourceKey<Registry<E>>, element_codec: ElementCodecModel) -> Self {
        Self::create_with_inline(registry_key, element_codec, true)
    }

    pub fn create_with_inline(
        registry_key: ResourceKey<Registry<E>>,
        element_codec: ElementCodecModel,
        allow_inline: bool,
    ) -> Self {
        Self {
            registry_key,
            element_codec,
            allow_inline,
        }
    }

    pub fn allow_inline(&self) -> bool {
        self.allow_inline
    }

    pub fn java_to_string(&self) -> String {
        format!(
            "RegistryFileCodec[{} {}]",
            self.registry_key, self.element_codec
        )
    }
}

impl<E: Clone + AsRef<str> + std::fmt::Debug> RegistryFileCodecModel<E> {
    pub fn encode(
        &self,
        input: &FileHolderModel<E>,
        ops: Option<&RegistryFileOps<E>>,
        prefix: impl Into<String>,
    ) -> FileCodecOutcome<String> {
        let prefix = prefix.into();
        if let Some(ops) = ops {
            if let Some(owner) = ops.owner() {
                if !input.can_serialize_in(owner) {
                    return FileCodecOutcome::Error(format!(
                        "Element {} is not valid in current registry set",
                        input.describe()
                    ));
                }

                return match input {
                    FileHolderModel::Reference { id, .. } => {
                        FileCodecOutcome::success(format!("{prefix}+{id}"))
                    }
                    FileHolderModel::Direct { value, .. } => {
                        FileCodecOutcome::success(self.element_codec.encode(value, prefix))
                    }
                };
            }
        }

        FileCodecOutcome::success(self.element_codec.encode(input.value(), prefix))
    }

    pub fn decode(
        &self,
        ops: Option<&RegistryFileOps<E>>,
        input: &str,
        remainder: impl Into<String>,
    ) -> FileCodecOutcome<(FileHolderModel<String>, String)> {
        let remainder = remainder.into();
        if let Some(ops) = ops {
            if ops.owner().is_none() {
                return FileCodecOutcome::Error(format!(
                    "Registry does not exist: {}",
                    self.registry_key
                ));
            }

            if let Ok(id) = Identifier::parse(input) {
                let element_key = ResourceKey::create(&self.registry_key, id.clone());
                return match ops.get(&element_key) {
                    Some(holder) => FileCodecOutcome::stable((
                        FileHolderModel::reference(id, holder.value().as_ref().to_string(), "owner"),
                        remainder,
                    )),
                    None => FileCodecOutcome::Error(format!("Failed to get element {element_key}")),
                };
            }

            if !self.allow_inline {
                return FileCodecOutcome::Error("Inline definitions not allowed here".to_string());
            }
        }

        let (value, remainder) = self.element_codec.decode(input, remainder);
        FileCodecOutcome::success((FileHolderModel::direct(value, "direct"), remainder))
    }
}

impl<E> std::fmt::Display for RegistryFileCodecModel<E> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.java_to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const REGISTRY_FILE_CODEC_JAVA: &str =
        vibecraft_java_source!("/net/minecraft/resources/RegistryFileCodec.java");

    fn wolf_registry_key() -> ResourceKey<Registry<String>> {
        ResourceKey::create_registry_key(Identifier::parse("minecraft:wolf_variant").unwrap())
    }

    #[test]
    fn factories_default_to_allowing_inline_definitions_like_java() {
        assert_java_contains(
            REGISTRY_FILE_CODEC_JAVA,
            &[
                "public static <E> RegistryFileCodec<E> create(final ResourceKey<? extends Registry<E>> registryKey, final Codec<E> elementCodec)",
                "return create(registryKey, elementCodec, true);",
                "public static <E> RegistryFileCodec<E> create(final ResourceKey<? extends Registry<E>> registryKey, final Codec<E> elementCodec, final boolean allowInline)",
                "return new RegistryFileCodec<>(registryKey, elementCodec, allowInline);",
            ],
        );

        let default_codec =
            RegistryFileCodecModel::<String>::create(wolf_registry_key(), ElementCodecModel::new("wolf"));
        assert!(default_codec.allow_inline());
        let strict_codec = RegistryFileCodecModel::<String>::create_with_inline(
            wolf_registry_key(),
            ElementCodecModel::new("wolf"),
            false,
        );
        assert!(!strict_codec.allow_inline());
    }

    #[test]
    fn encode_uses_registry_owner_when_available_and_falls_back_to_element_codec() {
        assert_java_contains(
            REGISTRY_FILE_CODEC_JAVA,
            &[
                "Optional<HolderOwner<E>> maybeOwner = registryOps.owner(this.registryKey);",
                "if (!input.canSerializeIn(maybeOwner.get()))",
                "return DataResult.error(() -> \"Element \" + input + \" is not valid in current registry set\");",
                "input.unwrap()\n               .map(id -> Identifier.CODEC.encode(id.identifier(), ops, prefix), value -> this.elementCodec.encode(value, ops, prefix));",
                "return this.elementCodec.encode(input.value(), ops, prefix);",
            ],
        );

        let codec =
            RegistryFileCodecModel::<String>::create(wolf_registry_key(), ElementCodecModel::new("wolf"));
        let ops = RegistryFileOps::with_owner("owner", Vec::new());
        let reference = FileHolderModel::reference(
            Identifier::parse("minecraft:pale").unwrap(),
            "pale-value".to_string(),
            "owner",
        );
        let direct = FileHolderModel::direct("inline-value".to_string(), "owner");

        assert_eq!(
            codec.encode(&reference, Some(&ops), "prefix"),
            FileCodecOutcome::success("prefix+minecraft:pale".to_string())
        );
        assert_eq!(
            codec.encode(&direct, Some(&ops), "prefix"),
            FileCodecOutcome::success("wolf:prefix+inline-value".to_string())
        );
        assert_eq!(
            codec.encode(&reference, None, "prefix"),
            FileCodecOutcome::success("wolf:prefix+pale-value".to_string())
        );
        assert_eq!(
            codec.encode(
                &FileHolderModel::reference(
                    Identifier::parse("minecraft:pale").unwrap(),
                    "pale-value".to_string(),
                    "other",
                ),
                Some(&ops),
                "prefix",
            ),
            FileCodecOutcome::Error(
                "Element Reference(minecraft:pale) is not valid in current registry set"
                    .to_string(),
            )
        );
    }

    #[test]
    fn decode_prefers_registry_ids_and_handles_inline_fallbacks_like_java() {
        assert_java_contains(
            REGISTRY_FILE_CODEC_JAVA,
            &[
                "Optional<HolderGetter<E>> maybeLookup = registryOps.getter(this.registryKey);",
                "return DataResult.error(() -> \"Registry does not exist: \" + this.registryKey);",
                "DataResult<Pair<Identifier, T>> decoded = Identifier.CODEC.decode(ops, input);",
                "return !this.allowInline\n               ? DataResult.error(() -> \"Inline definitions not allowed here\")",
                ": this.elementCodec.decode(ops, input).map(p -> p.mapFirst(Holder::direct));",
                "ResourceKey<E> elementKey = ResourceKey.create(this.registryKey, (Identifier)pair.getFirst());",
                ".orElseGet(() -> DataResult.error(() -> \"Failed to get element \" + elementKey))",
                ".setLifecycle(Lifecycle.stable());",
                "return this.elementCodec.decode(ops, input).map(p -> p.mapFirst(Holder::direct));",
            ],
        );

        let holder = FileHolderModel::reference(
            Identifier::parse("minecraft:pale").unwrap(),
            "pale-value".to_string(),
            "owner",
        );
        let ops = RegistryFileOps::with_owner(
            "owner",
            vec![(Identifier::parse("minecraft:pale").unwrap(), holder)],
        );
        let codec =
            RegistryFileCodecModel::<String>::create(wolf_registry_key(), ElementCodecModel::new("wolf"));

        assert_eq!(
            codec.decode(Some(&ops), "minecraft:pale", "tail"),
            FileCodecOutcome::stable((
                FileHolderModel::reference(
                    Identifier::parse("minecraft:pale").unwrap(),
                    "pale-value".to_string(),
                    "owner",
                ),
                "tail".to_string(),
            ))
        );
        assert_eq!(
            codec.decode(Some(&ops), "minecraft:ashen", "tail"),
            FileCodecOutcome::Error(
                "Failed to get element ResourceKey[minecraft:wolf_variant / minecraft:ashen]"
                    .to_string(),
            )
        );
        assert_eq!(
            codec.decode(Some(&RegistryFileOps::<String>::missing_registry()), "minecraft:pale", "tail"),
            FileCodecOutcome::Error(
                "Registry does not exist: ResourceKey[minecraft:root / minecraft:wolf_variant]"
                    .to_string(),
            )
        );
        assert_eq!(
            codec.decode(Some(&ops), "{inline}", "tail"),
            FileCodecOutcome::success((
                FileHolderModel::direct("wolf:{inline}".to_string(), "direct"),
                "tail".to_string(),
            ))
        );
        let strict_codec = RegistryFileCodecModel::<String>::create_with_inline(
            wolf_registry_key(),
            ElementCodecModel::new("wolf"),
            false,
        );
        assert_eq!(
            strict_codec.decode(Some(&ops), "{inline}", "tail"),
            FileCodecOutcome::Error("Inline definitions not allowed here".to_string())
        );
        assert_eq!(
            codec.decode(None, "{inline}", "tail"),
            FileCodecOutcome::success((
                FileHolderModel::direct("wolf:{inline}".to_string(), "direct"),
                "tail".to_string(),
            ))
        );
    }

    #[test]
    fn to_string_matches_java_registry_key_and_element_codec() {
        assert_java_contains(
            REGISTRY_FILE_CODEC_JAVA,
            &["return \"RegistryFileCodec[\" + this.registryKey + \" \" + this.elementCodec + \"]\";"],
        );
        let codec =
            RegistryFileCodecModel::<String>::create(wolf_registry_key(), ElementCodecModel::new("wolf"));
        assert_eq!(
            codec.to_string(),
            "RegistryFileCodec[ResourceKey[minecraft:root / minecraft:wolf_variant] wolf]"
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
