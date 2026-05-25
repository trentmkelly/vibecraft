#![allow(dead_code)]

use std::collections::BTreeMap;
use std::fmt;
use std::io::{self, Read, Write};
use std::marker::PhantomData;

use crate::network::codec::{
    read_identifier, read_registry_value_id, write_identifier, write_registry_value_id,
    RegistryValueId,
};
use crate::storage::nbt::Tag;

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

    pub fn to_json_object(&self) -> String {
        format!(
            "{{\"registry\":\"{}\",\"value\":\"{}\"}}",
            self.registry, self.location
        )
    }

    pub fn from_json_object(raw: &str) -> Result<Self, String> {
        let registry = parse_json_object_string_field(raw, "registry")?;
        let value = parse_json_object_string_field(raw, "value")?;
        Ok(Self::new(
            Identifier::parse(&registry)?,
            Identifier::parse(&value)?,
        ))
    }

    pub fn to_nbt(&self) -> Tag {
        Tag::Compound(vec![
            (
                "registry".to_string(),
                Tag::String(self.registry.to_string()),
            ),
            ("value".to_string(), Tag::String(self.location.to_string())),
        ])
    }

    pub fn from_nbt(tag: &Tag) -> Result<Self, String> {
        let compound = match tag {
            Tag::Compound(values) => values,
            _ => return Err("registry key NBT must be a compound".to_string()),
        };
        let registry = compound_string(compound, "registry")?;
        let value = compound_string(compound, "value")?;
        Ok(Self::new(
            Identifier::parse(registry)?,
            Identifier::parse(value)?,
        ))
    }

    pub fn write_network<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_identifier(writer, &self.registry)?;
        write_identifier(writer, &self.location)
    }

    pub fn read_network<R: Read>(reader: &mut R) -> io::Result<Self> {
        let registry = read_identifier(reader)?;
        let location = read_identifier(reader)?;
        Ok(Self::new(registry, location))
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

    pub fn key_by_id(&self, id: u32) -> Option<ResourceKey<T>> {
        self.get_by_id(id).map(|entry| entry.key().clone())
    }

    pub fn id_for_location(&self, location: &Identifier) -> Option<u32> {
        self.get(location).map(RegistryEntry::id)
    }

    pub fn write_network_id<W: Write>(
        &self,
        writer: &mut W,
        location: &Identifier,
    ) -> io::Result<()> {
        let id = self.id_for_location(location).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("unknown registry key {location}"),
            )
        })?;
        write_registry_value_id(writer, RegistryValueId(id as i32))
    }

    pub fn read_network_key<R: Read>(&self, reader: &mut R) -> io::Result<ResourceKey<T>> {
        let id = read_registry_value_id(reader)?.0;
        if id < 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "negative registry value id",
            ));
        }
        self.key_by_id(id as u32).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("unknown registry value id {id}"),
            )
        })
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

fn parse_json_object_string_field(raw: &str, field: &str) -> Result<String, String> {
    let trimmed = raw.trim();
    if !(trimmed.starts_with('{') && trimmed.ends_with('}')) {
        return Err("registry key JSON must be an object".to_string());
    }
    let needle = format!("\"{field}\":\"");
    let start = trimmed
        .find(&needle)
        .ok_or_else(|| format!("missing JSON field {field}"))?
        + needle.len();
    let tail = &trimmed[start..];
    let end = tail
        .find('"')
        .ok_or_else(|| format!("unterminated JSON field {field}"))?;
    Ok(tail[..end].to_string())
}

fn compound_string<'a>(compound: &'a [(String, Tag)], field: &str) -> Result<&'a str, String> {
    compound
        .iter()
        .find_map(|(name, value)| match (name.as_str(), value) {
            (name, Tag::String(value)) if name == field => Some(value.as_str()),
            _ => None,
        })
        .ok_or_else(|| format!("missing NBT string field {field}"))
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TagFile {
    pub registry: Identifier,
    pub tag: Identifier,
    pub replace: bool,
    pub entries: Vec<TagEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TagEntry {
    pub id: Identifier,
    pub required: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadedTags {
    tags: BTreeMap<(Identifier, Identifier), Vec<Identifier>>,
}

impl LoadedTags {
    pub fn load<T>(
        registry: &Registry<T>,
        files: impl IntoIterator<Item = TagFile>,
    ) -> Result<Self, Vec<String>> {
        let mut tags = BTreeMap::new();
        let mut errors = Vec::new();
        for file in files {
            if &file.registry != registry.registry_id() {
                errors.push(format!(
                    "tag {} targets registry {}, expected {}",
                    file.tag,
                    file.registry,
                    registry.registry_id()
                ));
                continue;
            }
            let key = (file.registry.clone(), file.tag.clone());
            if file.replace {
                tags.insert(key.clone(), Vec::new());
            }
            let values = tags.entry(key).or_insert_with(Vec::new);
            for entry in file.entries {
                if registry.get(&entry.id).is_some() {
                    if !values.contains(&entry.id) {
                        values.push(entry.id);
                    }
                } else if entry.required {
                    errors.push(format!("missing required tag entry {}", entry.id));
                }
            }
        }
        if errors.is_empty() {
            Ok(Self { tags })
        } else {
            Err(errors)
        }
    }

    pub fn values(&self, registry: &Identifier, tag: &Identifier) -> Option<&[Identifier]> {
        self.tags
            .get(&(registry.clone(), tag.clone()))
            .map(Vec::as_slice)
    }
}

mod builtin;
#[cfg(test)]
use builtin::*;

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
    pub fn main_26_1_2() -> Result<Self, String> {
        Ok(Self {
            names: vec![
                (Identifier::parse("vanilla")?, feature_flags::VANILLA),
                (
                    Identifier::parse("trade_rebalance")?,
                    feature_flags::TRADE_REBALANCE,
                ),
                (
                    Identifier::parse("redstone_experiments")?,
                    feature_flags::REDSTONE_EXPERIMENTS,
                ),
                (
                    Identifier::parse("minecart_improvements")?,
                    feature_flags::MINECART_IMPROVEMENTS,
                ),
            ],
        })
    }

    pub fn to_names(&self, set: FeatureFlagSet) -> Vec<Identifier> {
        self.names
            .iter()
            .filter(|(_id, flag)| set.contains(*flag))
            .map(|(id, _flag)| id.clone())
            .collect()
    }

    pub fn resolve_names(&self, names: &[Identifier]) -> Result<FeatureFlagSet, Vec<Identifier>> {
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
mod tests;
