#![allow(dead_code)]

use std::collections::BTreeMap;
use std::fmt;
use std::marker::PhantomData;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Identifier {
    namespace: String,
    path: String,
}

impl Identifier {
    pub fn parse(value: &str) -> Result<Self, String> {
        let (namespace, path) = value.split_once(':').unwrap_or(("minecraft", value));
        Self::new(namespace, path)
    }

    pub fn new(namespace: &str, path: &str) -> Result<Self, String> {
        if namespace.is_empty() || path.is_empty() {
            return Err("identifier namespace and path must be non-empty".to_string());
        }
        if !namespace
            .chars()
            .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '_' || ch == '-')
        {
            return Err(format!("invalid identifier namespace: {namespace}"));
        }
        if !path.chars().all(|ch| {
            ch.is_ascii_lowercase() || ch.is_ascii_digit() || matches!(ch, '_' | '-' | '.' | '/')
        }) {
            return Err(format!("invalid identifier path: {path}"));
        }

        Ok(Self {
            namespace: namespace.to_string(),
            path: path.to_string(),
        })
    }

    pub fn namespace(&self) -> &str {
        &self.namespace
    }

    pub fn path(&self) -> &str {
        &self.path
    }
}

impl fmt::Display for Identifier {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}:{}", self.namespace, self.path)
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ResourceKey<T> {
    registry: Identifier,
    location: Identifier,
    _marker: PhantomData<T>,
}

impl<T> Clone for ResourceKey<T> {
    fn clone(&self) -> Self {
        Self {
            registry: self.registry.clone(),
            location: self.location.clone(),
            _marker: PhantomData,
        }
    }
}

impl<T> ResourceKey<T> {
    pub fn new(registry: Identifier, location: Identifier) -> Self {
        Self {
            registry,
            location,
            _marker: PhantomData,
        }
    }

    pub fn registry(&self) -> &Identifier {
        &self.registry
    }

    pub fn location(&self) -> &Identifier {
        &self.location
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lifecycle {
    Stable,
    Experimental,
    Deprecated,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegistryEntry<T> {
    id: u32,
    key: ResourceKey<T>,
    value: T,
    lifecycle: Lifecycle,
}

impl<T> RegistryEntry<T> {
    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn key(&self) -> &ResourceKey<T> {
        &self.key
    }

    pub fn value(&self) -> &T {
        &self.value
    }

    pub fn lifecycle(&self) -> Lifecycle {
        self.lifecycle
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SerializedRegistryEntry {
    pub id: u32,
    pub location: Identifier,
    pub value: String,
    pub lifecycle: Lifecycle,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SerializedRegistry {
    pub registry: Identifier,
    pub entries: Vec<SerializedRegistryEntry>,
}

#[derive(Debug, Clone)]
pub struct Registry<T> {
    registry_id: Identifier,
    entries: Vec<RegistryEntry<T>>,
    by_location: BTreeMap<Identifier, usize>,
    frozen: bool,
}

impl<T> Registry<T> {
    pub fn new(registry_id: Identifier) -> Self {
        Self {
            registry_id,
            entries: Vec::new(),
            by_location: BTreeMap::new(),
            frozen: false,
        }
    }

    pub fn register(
        &mut self,
        location: Identifier,
        value: T,
        lifecycle: Lifecycle,
    ) -> Result<ResourceKey<T>, String> {
        if self.frozen {
            return Err(format!("registry {} is frozen", self.registry_id));
        }
        if self.by_location.contains_key(&location) {
            return Err(format!("duplicate registry key: {location}"));
        }

        let key = ResourceKey::new(self.registry_id.clone(), location.clone());
        let id = self.entries.len() as u32;
        self.by_location.insert(location, self.entries.len());
        self.entries.push(RegistryEntry {
            id,
            key: key.clone(),
            value,
            lifecycle,
        });
        Ok(key)
    }

    pub fn get(&self, location: &Identifier) -> Option<&RegistryEntry<T>> {
        self.by_location
            .get(location)
            .and_then(|index| self.entries.get(*index))
    }

    pub fn get_by_id(&self, id: u32) -> Option<&RegistryEntry<T>> {
        self.entries.get(id as usize)
    }

    pub fn iter(&self) -> impl Iterator<Item = &RegistryEntry<T>> {
        self.entries.iter()
    }

    pub fn registry_id(&self) -> &Identifier {
        &self.registry_id
    }

    pub fn freeze(&mut self) {
        self.frozen = true;
    }

    pub fn is_frozen(&self) -> bool {
        self.frozen
    }

    pub fn serialize_with<F>(&self, mut encode: F) -> SerializedRegistry
    where
        F: FnMut(&T) -> String,
    {
        SerializedRegistry {
            registry: self.registry_id.clone(),
            entries: self
                .entries
                .iter()
                .map(|entry| SerializedRegistryEntry {
                    id: entry.id,
                    location: entry.key.location().clone(),
                    value: encode(&entry.value),
                    lifecycle: entry.lifecycle,
                })
                .collect(),
        }
    }

    pub fn deserialize_with<F>(snapshot: SerializedRegistry, mut decode: F) -> Result<Self, String>
    where
        F: FnMut(&str) -> Result<T, String>,
    {
        let mut registry = Self::new(snapshot.registry);
        for (expected_id, entry) in snapshot.entries.into_iter().enumerate() {
            if entry.id != expected_id as u32 {
                return Err(format!(
                    "registry snapshot id mismatch for {}: expected {}, got {}",
                    entry.location, expected_id, entry.id
                ));
            }
            registry.register(entry.location, decode(&entry.value)?, entry.lifecycle)?;
        }
        Ok(registry)
    }

    pub fn apply_data_pack_overrides(
        &mut self,
        overrides: impl IntoIterator<Item = (Identifier, T, Lifecycle)>,
    ) -> Result<(), String> {
        if self.frozen {
            return Err(format!("registry {} is frozen", self.registry_id));
        }
        for (location, value, lifecycle) in overrides {
            if let Some(index) = self.by_location.get(&location).copied() {
                self.entries[index].value = value;
                self.entries[index].lifecycle = lifecycle;
            } else {
                self.register(location, value, lifecycle)?;
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Holder<T> {
    Direct(T),
    Reference(ResourceKey<T>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TagKey<T> {
    registry: Identifier,
    location: Identifier,
    _marker: PhantomData<T>,
}

impl<T> TagKey<T> {
    pub fn new(registry: Identifier, location: Identifier) -> Self {
        Self {
            registry,
            location,
            _marker: PhantomData,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HolderSet<T> {
    Direct(Vec<Holder<T>>),
    Named(TagKey<T>),
}

pub mod registries {
    pub const BLOCK: &str = "minecraft:block";
    pub const ITEM: &str = "minecraft:item";
    pub const ENTITY_TYPE: &str = "minecraft:entity_type";
    pub const DIMENSION_TYPE: &str = "minecraft:dimension_type";
    pub const BIOME: &str = "minecraft:worldgen/biome";
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FeatureFlag {
    bit: u8,
}

impl FeatureFlag {
    const fn new(bit: u8) -> Self {
        Self { bit }
    }

    fn mask(self) -> u64 {
        1u64 << self.bit
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FeatureFlagSet {
    mask: u64,
}

impl FeatureFlagSet {
    pub const fn empty() -> Self {
        Self { mask: 0 }
    }

    pub fn of(flags: &[FeatureFlag]) -> Self {
        let mut mask = 0u64;
        for flag in flags {
            mask |= flag.mask();
        }
        Self { mask }
    }

    pub fn contains(self, flag: FeatureFlag) -> bool {
        self.mask & flag.mask() != 0
    }

    pub fn is_subset_of(self, other: Self) -> bool {
        self.mask & !other.mask == 0
    }

    pub fn join(self, other: Self) -> Self {
        Self {
            mask: self.mask | other.mask,
        }
    }

    pub fn subtract(self, other: Self) -> Self {
        Self {
            mask: self.mask & !other.mask,
        }
    }

    pub fn intersects(self, other: Self) -> bool {
        self.mask & other.mask != 0
    }
}

#[derive(Debug, Clone)]
pub struct FeatureFlagRegistry {
    names: Vec<(Identifier, FeatureFlag)>,
}

impl FeatureFlagRegistry {
    pub fn main_26_1_2() -> Self {
        Self {
            names: vec![
                (
                    Identifier::parse("vanilla").unwrap(),
                    feature_flags::VANILLA,
                ),
                (
                    Identifier::parse("trade_rebalance").unwrap(),
                    feature_flags::TRADE_REBALANCE,
                ),
                (
                    Identifier::parse("redstone_experiments").unwrap(),
                    feature_flags::REDSTONE_EXPERIMENTS,
                ),
                (
                    Identifier::parse("minecart_improvements").unwrap(),
                    feature_flags::MINECART_IMPROVEMENTS,
                ),
            ],
        }
    }

    pub fn to_names(&self, set: FeatureFlagSet) -> Vec<Identifier> {
        self.names
            .iter()
            .filter_map(|(id, flag)| set.contains(*flag).then(|| id.clone()))
            .collect()
    }

    pub fn from_names(&self, names: &[Identifier]) -> Result<FeatureFlagSet, Vec<Identifier>> {
        let mut unknown = Vec::new();
        let mut set = FeatureFlagSet::empty();
        for name in names {
            if let Some((_id, flag)) = self.names.iter().find(|(id, _flag)| id == name) {
                set = set.join(FeatureFlagSet::of(&[*flag]));
            } else {
                unknown.push(name.clone());
            }
        }

        if unknown.is_empty() {
            Ok(set)
        } else {
            Err(unknown)
        }
    }
}

pub mod feature_flags {
    use super::{FeatureFlag, FeatureFlagSet};

    pub const VANILLA: FeatureFlag = FeatureFlag::new(0);
    pub const TRADE_REBALANCE: FeatureFlag = FeatureFlag::new(1);
    pub const REDSTONE_EXPERIMENTS: FeatureFlag = FeatureFlag::new(2);
    pub const MINECART_IMPROVEMENTS: FeatureFlag = FeatureFlag::new(3);

    pub fn vanilla_set() -> FeatureFlagSet {
        FeatureFlagSet::of(&[VANILLA])
    }

    pub fn default_flags_26_1_2() -> FeatureFlagSet {
        vanilla_set()
    }
}

#[cfg(test)]
mod tests {
    use super::{
        feature_flags, registries, FeatureFlagRegistry, Holder, HolderSet, Identifier, Lifecycle,
        Registry, TagKey,
    };

    #[test]
    fn parses_default_namespace_identifiers() {
        let id = Identifier::parse("stone").unwrap();
        assert_eq!(id.namespace(), "minecraft");
        assert_eq!(id.path(), "stone");
        assert_eq!(id.to_string(), "minecraft:stone");
    }

    #[test]
    fn rejects_invalid_identifier_case() {
        assert!(Identifier::parse("Minecraft:Stone").is_err());
    }

    #[test]
    fn registry_assigns_stable_ids_and_freezes() {
        let mut registry = Registry::new(Identifier::parse(registries::BLOCK).unwrap());
        let stone = Identifier::parse("stone").unwrap();
        let dirt = Identifier::parse("dirt").unwrap();

        registry
            .register(stone.clone(), "stone block", Lifecycle::Stable)
            .unwrap();
        registry
            .register(dirt.clone(), "dirt block", Lifecycle::Stable)
            .unwrap();
        registry.freeze();

        assert_eq!(registry.get(&stone).unwrap().id(), 0);
        assert_eq!(registry.get_by_id(1).unwrap().value(), &"dirt block");
        assert!(registry
            .register(
                Identifier::parse("grass_block").unwrap(),
                "grass",
                Lifecycle::Stable
            )
            .is_err());
    }

    #[test]
    fn registry_snapshots_round_trip_in_numeric_id_order() {
        let mut registry = Registry::new(Identifier::parse(registries::ITEM).unwrap());
        registry
            .register(
                Identifier::parse("stick").unwrap(),
                "stick item".to_string(),
                Lifecycle::Stable,
            )
            .unwrap();
        registry
            .register(
                Identifier::parse("trial_key").unwrap(),
                "trial key item".to_string(),
                Lifecycle::Experimental,
            )
            .unwrap();

        let snapshot = registry.serialize_with(Clone::clone);
        assert_eq!(
            snapshot.registry,
            Identifier::parse(registries::ITEM).unwrap()
        );
        assert_eq!(snapshot.entries[0].id, 0);
        assert_eq!(
            snapshot.entries[1].location,
            Identifier::parse("trial_key").unwrap()
        );

        let decoded = Registry::deserialize_with(snapshot, |value| Ok(value.to_string())).unwrap();
        assert_eq!(
            decoded
                .get(&Identifier::parse("stick").unwrap())
                .unwrap()
                .value(),
            "stick item"
        );
        assert_eq!(
            decoded.get_by_id(1).unwrap().lifecycle(),
            Lifecycle::Experimental
        );
    }

    #[test]
    fn registry_data_pack_overrides_preserve_existing_ids_and_append_new_entries() {
        let mut registry = Registry::new(Identifier::parse(registries::ITEM).unwrap());
        registry
            .register(
                Identifier::parse("stick").unwrap(),
                "vanilla stick".to_string(),
                Lifecycle::Stable,
            )
            .unwrap();
        registry
            .register(
                Identifier::parse("apple").unwrap(),
                "vanilla apple".to_string(),
                Lifecycle::Stable,
            )
            .unwrap();

        registry
            .apply_data_pack_overrides(vec![
                (
                    Identifier::parse("stick").unwrap(),
                    "pack stick".to_string(),
                    Lifecycle::Experimental,
                ),
                (
                    Identifier::parse("custom").unwrap(),
                    "pack custom".to_string(),
                    Lifecycle::Stable,
                ),
            ])
            .unwrap();

        let stick = registry.get(&Identifier::parse("stick").unwrap()).unwrap();
        assert_eq!(stick.id(), 0);
        assert_eq!(stick.value(), "pack stick");
        assert_eq!(stick.lifecycle(), Lifecycle::Experimental);
        assert_eq!(
            registry
                .get(&Identifier::parse("custom").unwrap())
                .unwrap()
                .id(),
            2
        );
    }

    #[test]
    fn represents_holders_and_named_tag_sets() {
        let registry = Identifier::parse(registries::ITEM).unwrap();
        let tag = TagKey::<String>::new(registry.clone(), Identifier::parse("logs").unwrap());
        let set = HolderSet::<String>::Named(tag);
        assert!(matches!(set, HolderSet::Named(_)));

        let key = super::ResourceKey::new(registry, Identifier::parse("stick").unwrap());
        let holder = Holder::<String>::Reference(key);
        assert!(matches!(holder, Holder::Reference(_)));
    }

    #[test]
    fn feature_flags_match_26_1_2_defaults() {
        let registry = FeatureFlagRegistry::main_26_1_2();
        let defaults = feature_flags::default_flags_26_1_2();
        assert!(defaults.contains(feature_flags::VANILLA));
        assert!(!defaults.contains(feature_flags::TRADE_REBALANCE));
        assert_eq!(
            registry.to_names(defaults),
            vec![Identifier::parse("minecraft:vanilla").unwrap()]
        );

        let experimental = defaults.join(super::FeatureFlagSet::of(&[
            feature_flags::MINECART_IMPROVEMENTS,
        ]));
        assert!(defaults.is_subset_of(experimental));
        assert!(!experimental.is_subset_of(defaults));
    }
}
