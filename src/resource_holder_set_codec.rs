#![allow(dead_code)]

use std::marker::PhantomData;

use crate::registry::{Identifier, Registry, ResourceKey};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HolderSetElementKind {
    Reference,
    Direct,
}

impl std::fmt::Display for HolderSetElementKind {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Reference => formatter.write_str("Reference"),
            Self::Direct => formatter.write_str("Direct"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HolderSetElement<E> {
    Reference {
        id: Identifier,
        owner: String,
        marker: PhantomData<E>,
    },
    Direct {
        value: E,
    },
}

impl<E> HolderSetElement<E> {
    pub fn reference(id: Identifier, owner: impl Into<String>) -> Self {
        Self::Reference {
            id,
            owner: owner.into(),
            marker: PhantomData,
        }
    }

    pub fn direct(value: E) -> Self {
        Self::Direct { value }
    }

    pub fn kind(&self) -> HolderSetElementKind {
        match self {
            Self::Reference { .. } => HolderSetElementKind::Reference,
            Self::Direct { .. } => HolderSetElementKind::Direct,
        }
    }

    pub fn describe(&self) -> String
    where
        E: std::fmt::Debug,
    {
        match self {
            Self::Reference { id, .. } => format!("Reference({id})"),
            Self::Direct { value } => format!("Direct({value:?})"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct HolderSetTagKey<E> {
    registry_key: ResourceKey<Registry<E>>,
    location: Identifier,
}

impl<E> HolderSetTagKey<E> {
    pub fn new(registry_key: ResourceKey<Registry<E>>, location: Identifier) -> Self {
        Self {
            registry_key,
            location,
        }
    }

    pub fn location(&self) -> &Identifier {
        &self.location
    }

    pub fn registry_identifier(&self) -> &Identifier {
        self.registry_key.location()
    }

    pub fn java_to_string(&self) -> String {
        format!(
            "TagKey[{} / {}]",
            self.registry_identifier(),
            self.location
        )
    }
}

impl<E> PartialEq for HolderSetTagKey<E> {
    fn eq(&self, other: &Self) -> bool {
        self.registry_key.registry() == other.registry_key.registry()
            && self.registry_key.location() == other.registry_key.location()
            && self.location == other.location
    }
}

impl<E> Eq for HolderSetTagKey<E> {}

impl<E> std::fmt::Display for HolderSetTagKey<E> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.java_to_string())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HolderSetModel<E> {
    Direct {
        contents: Vec<HolderSetElement<E>>,
    },
    Named {
        owner: String,
        key: HolderSetTagKey<E>,
        contents: Vec<HolderSetElement<E>>,
    },
}

impl<E: Clone + std::fmt::Debug> HolderSetModel<E> {
    pub fn direct(contents: Vec<HolderSetElement<E>>) -> Self {
        Self::Direct { contents }
    }

    pub fn named(
        owner: impl Into<String>,
        registry_key: ResourceKey<Registry<E>>,
        location: Identifier,
        contents: Vec<HolderSetElement<E>>,
    ) -> Self {
        Self::Named {
            owner: owner.into(),
            key: HolderSetTagKey::new(registry_key, location),
            contents,
        }
    }

    pub fn stream(&self) -> &[HolderSetElement<E>] {
        match self {
            Self::Direct { contents } | Self::Named { contents, .. } => contents,
        }
    }

    pub fn can_serialize_in(&self, owner: &str) -> bool {
        match self {
            Self::Direct { .. } => true,
            Self::Named {
                owner: set_owner, ..
            } => set_owner == owner,
        }
    }

    fn unwrap_for_registry(&self) -> HolderSetUnwrap<E> {
        match self {
            Self::Direct { contents } => HolderSetUnwrap::Elements(contents.clone()),
            Self::Named { key, .. } => HolderSetUnwrap::Tag(key.location().clone()),
        }
    }

    pub fn describe(&self) -> String {
        match self {
            Self::Direct { contents } => format!(
                "DirectSet[{}]",
                contents
                    .iter()
                    .map(HolderSetElement::describe)
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Self::Named { key, contents, .. } => format!(
                "NamedSet({key})[{}]",
                contents
                    .iter()
                    .map(HolderSetElement::describe)
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum HolderSetUnwrap<E> {
    Tag(Identifier),
    Elements(Vec<HolderSetElement<E>>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HolderSetCodecInput<E> {
    Tag(Identifier),
    List(Vec<HolderSetElement<E>>),
    Element(HolderSetElement<E>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EncodedHolderSet {
    Tag(Identifier),
    List(Vec<String>),
    Single(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HolderSetCodecOutcome<T> {
    Success {
        value: T,
        lifecycle: Option<&'static str>,
    },
    Error(String),
}

impl<T> HolderSetCodecOutcome<T> {
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HolderElementCodecModel {
    name: &'static str,
}

impl HolderElementCodecModel {
    pub fn new(name: &'static str) -> Self {
        Self { name }
    }

    fn encode<E: AsRef<str> + std::fmt::Debug>(
        &self,
        holder: &HolderSetElement<E>,
        prefix: &str,
    ) -> String {
        match holder {
            HolderSetElement::Reference { id, .. } => format!("{prefix}+{id}"),
            HolderSetElement::Direct { value } => format!("{}:{prefix}+{}", self.name, value.as_ref()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegistryHolderSetOps<E> {
    owner: Option<String>,
    tags: Vec<(Identifier, HolderSetModel<E>)>,
}

impl<E: Clone + std::fmt::Debug> RegistryHolderSetOps<E> {
    pub fn missing_registry() -> Self {
        Self {
            owner: None,
            tags: Vec::new(),
        }
    }

    pub fn with_owner(owner: impl Into<String>, tags: Vec<(Identifier, HolderSetModel<E>)>) -> Self {
        Self {
            owner: Some(owner.into()),
            tags,
        }
    }

    fn owner(&self) -> Option<&str> {
        self.owner.as_deref()
    }

    fn get_tag(&self, key: &Identifier) -> Option<HolderSetModel<E>> {
        self.tags
            .iter()
            .find(|(id, _holders)| id == key)
            .map(|(_id, holders)| holders.clone())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HomogenousListShape {
    CompactList,
    ListOnly,
}

#[derive(Debug, Clone)]
pub struct HolderSetCodecModel<E> {
    registry_key: ResourceKey<Registry<E>>,
    element_codec: HolderElementCodecModel,
    always_use_list: bool,
}

impl<E> HolderSetCodecModel<E> {
    pub fn create(
        registry_key: ResourceKey<Registry<E>>,
        element_codec: HolderElementCodecModel,
        always_use_list: bool,
    ) -> Self {
        Self {
            registry_key,
            element_codec,
            always_use_list,
        }
    }

    pub fn homogenous_list_shape(&self) -> HomogenousListShape {
        if self.always_use_list {
            HomogenousListShape::ListOnly
        } else {
            HomogenousListShape::CompactList
        }
    }
}

impl<E: Clone + AsRef<str> + std::fmt::Debug> HolderSetCodecModel<E> {
    pub fn decode(
        &self,
        ops: Option<&RegistryHolderSetOps<E>>,
        input: HolderSetCodecInput<E>,
        remainder: impl Into<String>,
    ) -> HolderSetCodecOutcome<(HolderSetModel<E>, String)> {
        let remainder = remainder.into();
        if let Some(ops) = ops {
            if ops.owner().is_some() {
                return self.decode_with_registry(ops, input, remainder);
            }
        }

        self.decode_without_registry(input, remainder)
    }

    fn decode_with_registry(
        &self,
        ops: &RegistryHolderSetOps<E>,
        input: HolderSetCodecInput<E>,
        remainder: String,
    ) -> HolderSetCodecOutcome<(HolderSetModel<E>, String)> {
        match input {
            HolderSetCodecInput::Tag(tag) => match ops.get_tag(&tag) {
                Some(holders) => HolderSetCodecOutcome::success((holders, remainder)),
                None => HolderSetCodecOutcome::Error(format!(
                    "Missing tag: '{tag}' in '{}'",
                    self.registry_key.location()
                )),
            },
            HolderSetCodecInput::List(values) => match validate_homogenous(&values) {
                Ok(()) => HolderSetCodecOutcome::stable((HolderSetModel::direct(values), remainder)),
                Err(err) => HolderSetCodecOutcome::Error(err),
            },
            HolderSetCodecInput::Element(value) => {
                if self.always_use_list {
                    HolderSetCodecOutcome::Error("Expected holder list".to_string())
                } else {
                    HolderSetCodecOutcome::stable((HolderSetModel::direct(vec![value]), remainder))
                }
            }
        }
    }

    fn decode_without_registry(
        &self,
        input: HolderSetCodecInput<E>,
        remainder: String,
    ) -> HolderSetCodecOutcome<(HolderSetModel<E>, String)> {
        let HolderSetCodecInput::List(values) = input else {
            return HolderSetCodecOutcome::Error("Expected holder list".to_string());
        };
        let mut direct_holders = Vec::new();
        for holder in values {
            if !matches!(holder, HolderSetElement::Direct { .. }) {
                return HolderSetCodecOutcome::Error(format!(
                    "Can't decode element {} without registry",
                    holder.describe()
                ));
            }
            direct_holders.push(holder);
        }

        HolderSetCodecOutcome::success((HolderSetModel::direct(direct_holders), remainder))
    }

    pub fn encode(
        &self,
        input: &HolderSetModel<E>,
        ops: Option<&RegistryHolderSetOps<E>>,
        prefix: impl Into<String>,
    ) -> HolderSetCodecOutcome<EncodedHolderSet> {
        let prefix = prefix.into();
        if let Some(ops) = ops {
            if let Some(owner) = ops.owner() {
                if !input.can_serialize_in(owner) {
                    return HolderSetCodecOutcome::Error(format!(
                        "HolderSet {} is not valid in current registry set",
                        input.describe()
                    ));
                }

                return match input.unwrap_for_registry() {
                    HolderSetUnwrap::Tag(tag) => HolderSetCodecOutcome::success(EncodedHolderSet::Tag(tag)),
                    HolderSetUnwrap::Elements(values) => self.encode_homogenous_list(&values, &prefix),
                };
            }
        }

        self.encode_homogenous_list(input.stream(), &prefix)
    }

    fn encode_homogenous_list(
        &self,
        values: &[HolderSetElement<E>],
        prefix: &str,
    ) -> HolderSetCodecOutcome<EncodedHolderSet> {
        if let Err(err) = validate_homogenous(values) {
            return HolderSetCodecOutcome::Error(err);
        }
        let encoded = values
            .iter()
            .map(|holder| self.element_codec.encode(holder, prefix))
            .collect::<Vec<_>>();
        if !self.always_use_list && encoded.len() == 1 {
            HolderSetCodecOutcome::stable(EncodedHolderSet::Single(encoded[0].clone()))
        } else {
            HolderSetCodecOutcome::stable(EncodedHolderSet::List(encoded))
        }
    }
}

fn validate_homogenous<E: std::fmt::Debug>(values: &[HolderSetElement<E>]) -> Result<(), String> {
    let Some(first) = values.first() else {
        return Ok(());
    };
    let first_kind = first.kind();
    for next in &values[1..] {
        let next_kind = next.kind();
        if next_kind != first_kind {
            return Err(format!(
                "Mixed type list: element {} had type {}, but list is of type {}",
                next.describe(),
                next_kind,
                first_kind
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const HOLDER_SET_CODEC_JAVA: &str =
        vibecraft_java_source!("/net/minecraft/resources/HolderSetCodec.java");

    fn wolf_registry_key() -> ResourceKey<Registry<String>> {
        ResourceKey::create_registry_key(Identifier::parse("minecraft:wolf_variant").unwrap())
    }

    fn codec(always_use_list: bool) -> HolderSetCodecModel<String> {
        HolderSetCodecModel::create(
            wolf_registry_key(),
            HolderElementCodecModel::new("wolf"),
            always_use_list,
        )
    }

    fn pale_ref() -> HolderSetElement<String> {
        HolderSetElement::reference(Identifier::parse("minecraft:pale").unwrap(), "owner")
    }

    fn snowy_ref() -> HolderSetElement<String> {
        HolderSetElement::reference(Identifier::parse("minecraft:snowy").unwrap(), "owner")
    }

    fn direct(value: &str) -> HolderSetElement<String> {
        HolderSetElement::direct(value.to_string())
    }

    #[test]
    fn factory_builds_homogenous_list_and_registry_aware_codecs_like_java() {
        assert_java_contains(
            HOLDER_SET_CODEC_JAVA,
            &[
                "private static <E> Codec<List<Holder<E>>> homogenousList(final Codec<Holder<E>> elementCodec, final boolean alwaysUseList)",
                "elementCodec.listOf().validate(ExtraCodecs.ensureHomogenous(Holder::kind))",
                "return alwaysUseList ? listCodec : ExtraCodecs.compactListCodec(elementCodec, listCodec);",
                "this.registryAwareCodec = Codec.either(TagKey.hashedCodec(registryKey), this.homogenousListCodec);",
            ],
        );

        assert_eq!(codec(false).homogenous_list_shape(), HomogenousListShape::CompactList);
        assert_eq!(codec(true).homogenous_list_shape(), HomogenousListShape::ListOnly);
    }

    #[test]
    fn decode_uses_registry_tags_or_homogenous_holder_lists_when_registry_ops_exist() {
        assert_java_contains(
            HOLDER_SET_CODEC_JAVA,
            &[
                "if (ops instanceof RegistryOps<T> registryOps)",
                "Optional<HolderGetter<E>> registryOptional = registryOps.getter(this.registryKey);",
                "this.registryAwareCodec\n               .decode(ops, input)",
                "tag -> lookupTag(registry, tag), values -> DataResult.success(HolderSet.direct(values))",
                "Missing tag: '\" + key.location() + \"' in '\" + key.registry().identifier() + \"'",
            ],
        );

        let tameable = Identifier::parse("minecraft:tameable").unwrap();
        let named = HolderSetModel::named("owner", wolf_registry_key(), tameable.clone(), vec![pale_ref()]);
        let ops = RegistryHolderSetOps::with_owner("owner", vec![(tameable.clone(), named.clone())]);

        assert_eq!(
            codec(false).decode(Some(&ops), HolderSetCodecInput::Tag(tameable), "tail"),
            HolderSetCodecOutcome::success((named, "tail".to_string()))
        );
        assert_eq!(
            codec(false).decode(
                Some(&ops),
                HolderSetCodecInput::Tag(Identifier::parse("minecraft:hostile").unwrap()),
                "tail",
            ),
            HolderSetCodecOutcome::Error(
                "Missing tag: 'minecraft:hostile' in 'minecraft:wolf_variant'".to_string(),
            )
        );
        assert_eq!(
            codec(false).decode(
                Some(&ops),
                HolderSetCodecInput::List(vec![pale_ref(), snowy_ref()]),
                "tail",
            ),
            HolderSetCodecOutcome::stable((
                HolderSetModel::direct(vec![pale_ref(), snowy_ref()]),
                "tail".to_string(),
            ))
        );
        assert_eq!(
            codec(false).decode(
                Some(&ops),
                HolderSetCodecInput::List(vec![pale_ref(), direct("inline")]),
                "tail",
            ),
            HolderSetCodecOutcome::Error(
                "Mixed type list: element Direct(\"inline\") had type Direct, but list is of type Reference"
                    .to_string(),
            )
        );
        assert_eq!(
            codec(false).decode(Some(&ops), HolderSetCodecInput::Element(pale_ref()), "tail"),
            HolderSetCodecOutcome::stable((HolderSetModel::direct(vec![pale_ref()]), "tail".to_string()))
        );
        assert_eq!(
            codec(true).decode(Some(&ops), HolderSetCodecInput::Element(pale_ref()), "tail"),
            HolderSetCodecOutcome::Error("Expected holder list".to_string())
        );
    }

    #[test]
    fn decode_without_registry_accepts_only_direct_holder_lists_like_java() {
        assert_java_contains(
            HOLDER_SET_CODEC_JAVA,
            &[
                "private <T> DataResult<Pair<HolderSet<E>, T>> decodeWithoutRegistry(final DynamicOps<T> ops, final T input)",
                "this.elementCodec.listOf().decode(ops, input).flatMap(p -> {",
                "if (!(holder instanceof Holder.Direct<E> direct))",
                "return DataResult.error(() -> \"Can't decode element \" + holder + \" without registry\");",
                "return DataResult.success(new Pair(HolderSet.direct(directHolders), p.getSecond()));",
            ],
        );

        assert_eq!(
            codec(false).decode(None, HolderSetCodecInput::List(vec![direct("inline")]), "tail"),
            HolderSetCodecOutcome::success((
                HolderSetModel::direct(vec![direct("inline")]),
                "tail".to_string(),
            ))
        );
        assert_eq!(
            codec(false).decode(
                Some(&RegistryHolderSetOps::<String>::missing_registry()),
                HolderSetCodecInput::List(vec![direct("inline")]),
                "tail",
            ),
            HolderSetCodecOutcome::success((
                HolderSetModel::direct(vec![direct("inline")]),
                "tail".to_string(),
            ))
        );
        assert_eq!(
            codec(false).decode(None, HolderSetCodecInput::List(vec![pale_ref()]), "tail"),
            HolderSetCodecOutcome::Error(
                "Can't decode element Reference(minecraft:pale) without registry".to_string(),
            )
        );
        assert_eq!(
            codec(false).decode(None, HolderSetCodecInput::Element(direct("inline")), "tail"),
            HolderSetCodecOutcome::Error("Expected holder list".to_string())
        );
    }

    #[test]
    fn encode_uses_registry_owner_validation_and_registry_aware_tag_or_list_shape() {
        assert_java_contains(
            HOLDER_SET_CODEC_JAVA,
            &[
                "Optional<HolderOwner<E>> maybeOwner = registryOps.owner(this.registryKey);",
                "if (!input.canSerializeIn(maybeOwner.get()))",
                "return DataResult.error(() -> \"HolderSet \" + input + \" is not valid in current registry set\");",
                "return this.registryAwareCodec.encode(input.unwrap().mapRight(List::copyOf), ops, prefix);",
                "private <T> DataResult<T> encodeWithoutRegistry(final HolderSet<E> input, final DynamicOps<T> ops, final T prefix)",
                "return this.homogenousListCodec.encode(input.stream().toList(), ops, prefix);",
            ],
        );

        let tameable = Identifier::parse("minecraft:tameable").unwrap();
        let named = HolderSetModel::named("owner", wolf_registry_key(), tameable.clone(), vec![pale_ref()]);
        let ops = RegistryHolderSetOps::with_owner("owner", Vec::new());

        assert_eq!(
            codec(false).encode(&named, Some(&ops), "prefix"),
            HolderSetCodecOutcome::success(EncodedHolderSet::Tag(tameable))
        );
        let wrong_owner = HolderSetModel::named(
            "other",
            wolf_registry_key(),
            Identifier::parse("minecraft:tameable").unwrap(),
            vec![pale_ref()],
        );
        assert_eq!(
            codec(false).encode(&wrong_owner, Some(&ops), "prefix"),
            HolderSetCodecOutcome::Error(
                "HolderSet NamedSet(TagKey[minecraft:wolf_variant / minecraft:tameable])[Reference(minecraft:pale)] is not valid in current registry set"
                    .to_string(),
            )
        );
        assert_eq!(
            codec(false).encode(&HolderSetModel::direct(vec![pale_ref()]), Some(&ops), "prefix"),
            HolderSetCodecOutcome::stable(EncodedHolderSet::Single(
                "prefix+minecraft:pale".to_string(),
            ))
        );
        assert_eq!(
            codec(true).encode(&HolderSetModel::direct(vec![pale_ref()]), Some(&ops), "prefix"),
            HolderSetCodecOutcome::stable(EncodedHolderSet::List(vec![
                "prefix+minecraft:pale".to_string(),
            ]))
        );
        assert_eq!(
            codec(false).encode(
                &HolderSetModel::direct(vec![direct("inline"), direct("other")]),
                None,
                "prefix",
            ),
            HolderSetCodecOutcome::stable(EncodedHolderSet::List(vec![
                "wolf:prefix+inline".to_string(),
                "wolf:prefix+other".to_string(),
            ]))
        );
        assert_eq!(
            codec(false).encode(
                &HolderSetModel::direct(vec![pale_ref(), direct("inline")]),
                Some(&ops),
                "prefix",
            ),
            HolderSetCodecOutcome::Error(
                "Mixed type list: element Direct(\"inline\") had type Direct, but list is of type Reference"
                    .to_string(),
            )
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
