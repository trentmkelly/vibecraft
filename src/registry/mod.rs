#![allow(dead_code)]
#![cfg_attr(
    all(test, not(vibecraft_has_decompiled_sources)),
    allow(unused_imports)
)]

use std::collections::BTreeMap;
use std::cmp::Ordering;
use std::fmt;
use std::io::{self, Read, Write};
use std::marker::PhantomData;
use std::path::{Path, PathBuf};

use crate::network::codec::{
    read_identifier, read_registry_value_id, write_identifier, write_registry_value_id,
    RegistryValueId,
};
use crate::storage::nbt::Tag;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Identifier {
    namespace: String,
    path: String,
}

impl Identifier {
    pub const NAMESPACE_SEPARATOR: char = ':';
    pub const DEFAULT_NAMESPACE: &'static str = "minecraft";
    pub const REALMS_NAMESPACE: &'static str = "realms";
    pub const ALLOWED_NAMESPACE_CHARACTERS: &'static str = "[a-z0-9_.-]";

    pub fn parse(value: &str) -> Result<Self, String> {
        Self::by_separator(value, Self::NAMESPACE_SEPARATOR)
    }

    pub fn new(namespace: &str, path: &str) -> Result<Self, String> {
        Self::from_namespace_and_path(namespace, path)
    }

    pub fn from_namespace_and_path(namespace: &str, path: &str) -> Result<Self, String> {
        Self::create_untrusted(namespace, path)
    }

    pub fn with_default_namespace(path: &str) -> Result<Self, String> {
        Ok(Self {
            namespace: Self::DEFAULT_NAMESPACE.to_string(),
            path: assert_valid_path(Self::DEFAULT_NAMESPACE, path)?.to_string(),
        })
    }

    pub fn try_parse(value: &str) -> Option<Self> {
        Self::try_by_separator(value, Self::NAMESPACE_SEPARATOR)
    }

    pub fn try_build(namespace: &str, path: &str) -> Option<Self> {
        (Self::is_valid_namespace(namespace) && Self::is_valid_path(path)).then(|| Self {
            namespace: namespace.to_string(),
            path: path.to_string(),
        })
    }

    pub fn by_separator(identifier: &str, separator: char) -> Result<Self, String> {
        match identifier.find(separator) {
            Some(0) => Self::with_default_namespace(&identifier[separator.len_utf8()..]),
            Some(separator_index) => {
                let path = &identifier[separator_index + separator.len_utf8()..];
                let namespace = &identifier[..separator_index];
                Self::create_untrusted(namespace, path)
            }
            None => Self::with_default_namespace(identifier),
        }
    }

    pub fn try_by_separator(identifier: &str, separator: char) -> Option<Self> {
        match identifier.find(separator) {
            Some(0) => {
                let path = &identifier[separator.len_utf8()..];
                Self::is_valid_path(path).then(|| Self {
                    namespace: Self::DEFAULT_NAMESPACE.to_string(),
                    path: path.to_string(),
                })
            }
            Some(separator_index) => {
                let path = &identifier[separator_index + separator.len_utf8()..];
                if !Self::is_valid_path(path) {
                    return None;
                }
                let namespace = &identifier[..separator_index];
                Self::is_valid_namespace(namespace).then(|| Self {
                    namespace: namespace.to_string(),
                    path: path.to_string(),
                })
            }
            None => Self::is_valid_path(identifier).then(|| Self {
                namespace: Self::DEFAULT_NAMESPACE.to_string(),
                path: identifier.to_string(),
            }),
        }
    }

    pub fn read(input: &str) -> Result<Self, String> {
        Self::parse(input).map_err(|err| format!("Not a valid resource location: {input} {err}"))
    }

    pub fn namespace(&self) -> &str {
        &self.namespace
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn with_path(&self, new_path: &str) -> Result<Self, String> {
        Ok(Self {
            namespace: self.namespace.clone(),
            path: assert_valid_path(&self.namespace, new_path)?.to_string(),
        })
    }

    pub fn with_modified_path(&self, modifier: impl FnOnce(&str) -> String) -> Result<Self, String> {
        self.with_path(&modifier(&self.path))
    }

    pub fn with_prefix(&self, prefix: &str) -> Result<Self, String> {
        self.with_path(&format!("{prefix}{}", self.path))
    }

    pub fn with_suffix(&self, suffix: &str) -> Result<Self, String> {
        self.with_path(&format!("{}{suffix}", self.path))
    }

    pub fn resolve_against(&self, root: &Path) -> PathBuf {
        root.join(&self.namespace).join(&self.path)
    }

    pub fn to_debug_file_name(&self) -> String {
        self.to_string().replace(['/', ':'], "_")
    }

    pub fn to_language_key(&self) -> String {
        format!("{}.{}", self.namespace, self.path)
    }

    pub fn to_short_language_key(&self) -> String {
        if self.namespace == Self::DEFAULT_NAMESPACE {
            self.path.clone()
        } else {
            self.to_language_key()
        }
    }

    pub fn to_short_string(&self) -> String {
        if self.namespace == Self::DEFAULT_NAMESPACE {
            self.path.clone()
        } else {
            self.to_string()
        }
    }

    pub fn to_language_key_with_prefix(&self, prefix: &str) -> String {
        format!("{prefix}.{}", self.to_language_key())
    }

    pub fn to_language_key_with_prefix_and_suffix(&self, prefix: &str, suffix: &str) -> String {
        format!("{prefix}.{}.{suffix}", self.to_language_key())
    }

    pub fn is_allowed_in_identifier(character: char) -> bool {
        character.is_ascii_digit()
            || character.is_ascii_lowercase()
            || matches!(character, '_' | ':' | '/' | '.' | '-')
    }

    pub fn is_valid_path(path: &str) -> bool {
        path.chars().all(Self::valid_path_char)
    }

    pub fn is_valid_namespace(namespace: &str) -> bool {
        namespace != ".." && namespace.chars().all(valid_namespace_char)
    }

    pub fn valid_path_char(character: char) -> bool {
        character == '_'
            || character == '-'
            || character.is_ascii_lowercase()
            || character.is_ascii_digit()
            || character == '/'
            || character == '.'
    }

    fn create_untrusted(namespace: &str, path: &str) -> Result<Self, String> {
        Ok(Self {
            namespace: assert_valid_namespace(namespace, path)?.to_string(),
            path: assert_valid_path(namespace, path)?.to_string(),
        })
    }
}

impl PartialOrd for Identifier {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Identifier {
    fn cmp(&self, other: &Self) -> Ordering {
        self.path
            .cmp(&other.path)
            .then_with(|| self.namespace.cmp(&other.namespace))
    }
}

impl fmt::Display for Identifier {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}:{}", self.namespace, self.path)
    }
}

fn valid_namespace_char(character: char) -> bool {
    character == '_'
        || character == '-'
        || character.is_ascii_lowercase()
        || character.is_ascii_digit()
        || character == '.'
}

fn assert_valid_namespace<'a>(namespace: &'a str, path: &str) -> Result<&'a str, String> {
    if Identifier::is_valid_namespace(namespace) {
        Ok(namespace)
    } else {
        Err(format!(
            "Non [a-z0-9_.-] character in namespace of identifier: {namespace}:{path}"
        ))
    }
}

fn assert_valid_path<'a>(namespace: &str, path: &'a str) -> Result<&'a str, String> {
    if Identifier::is_valid_path(path) {
        Ok(path)
    } else {
        Err(format!(
            "Non [a-z0-9/._-] character in path of location: {namespace}:{path}"
        ))
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

    pub fn create<U>(registry_name: &ResourceKey<U>, location: Identifier) -> Self {
        Self::new(registry_name.location.clone(), location)
    }

    pub fn create_registry_key(identifier: Identifier) -> ResourceKey<Registry<T>> {
        ResourceKey::new(
            Identifier {
                namespace: "minecraft".to_string(),
                path: "root".to_string(),
            },
            identifier,
        )
    }

    pub fn registry(&self) -> &Identifier {
        &self.registry
    }

    pub fn location(&self) -> &Identifier {
        &self.location
    }

    pub fn is_for<U>(&self, registry: &ResourceKey<Registry<U>>) -> bool {
        self.registry == *registry.location()
    }

    pub fn cast<U>(&self, registry: &ResourceKey<Registry<U>>) -> Option<ResourceKey<U>> {
        self.is_for(registry)
            .then(|| ResourceKey::new(self.registry.clone(), self.location.clone()))
    }

    pub fn registry_key(&self) -> ResourceKey<Registry<T>> {
        ResourceKey::create_registry_key(self.registry.clone())
    }

    pub fn java_to_string(&self) -> String {
        format!("ResourceKey[{} / {}]", self.registry, self.location)
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

    pub fn write_bound_network<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_identifier(writer, &self.location)
    }

    pub fn read_bound_network<R: Read, U>(
        registry_name: &ResourceKey<U>,
        reader: &mut R,
    ) -> io::Result<Self> {
        let location = read_identifier(reader)?;
        Ok(Self::create(registry_name, location))
    }

    pub fn to_bound_json(&self) -> String {
        format!("\"{}\"", self.location)
    }

    pub fn from_bound_json<U>(registry_name: &ResourceKey<U>, raw: &str) -> Result<Self, String> {
        let raw = raw.trim();
        let identifier = raw
            .strip_prefix('"')
            .and_then(|value| value.strip_suffix('"'))
            .ok_or_else(|| "resource key codec JSON must be an identifier string".to_string())?;
        Ok(Self::create(registry_name, Identifier::parse(identifier)?))
    }
}

impl<T> fmt::Display for ResourceKey<T> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.java_to_string())
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

    /// Java `TagKey.codec(registryName)` decodes an identifier and associates
    /// it with the requested registry.
    pub fn codec(registry: Identifier, location: &str) -> Result<Self, String> {
        Ok(Self::new(registry, Identifier::parse(location)?))
    }

    /// Java `TagKey.hashedCodec(registryName)` requires the serialized value
    /// to carry the leading `#` marker.
    pub fn hashed_codec(registry: Identifier, value: &str) -> Result<Self, String> {
        let location = value
            .strip_prefix('#')
            .ok_or_else(|| "Not a tag id".to_string())?;
        Self::codec(registry, location)
    }

    pub fn registry(&self) -> &Identifier {
        &self.registry
    }

    pub fn location(&self) -> &Identifier {
        &self.location
    }

    pub fn is_for(&self, registry: &Identifier) -> bool {
        self.registry == *registry
    }

    pub fn cast<E>(&self, registry: &Identifier) -> Option<TagKey<E>> {
        self.is_for(registry)
            .then(|| TagKey::new(self.registry.clone(), self.location.clone()))
    }

    pub fn hashed_string(&self) -> String {
        format!("#{}", self.location)
    }
}

impl<T> fmt::Display for TagKey<T> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "TagKey[{} / {}]",
            self.registry, self.location
        )
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

impl TagFile {
    /// Constructs a tag file with the Java codec's `replace` default.
    pub fn new(registry: Identifier, tag: Identifier, entries: Vec<TagEntry>, replace: bool) -> Self {
        Self {
            registry,
            tag,
            replace,
            entries,
        }
    }

    /// Encodes the Java `TagFile.CODEC` shape (`values` plus optional
    /// `replace`). Registry and tag identify the resource file and are not
    /// serialized inside the file itself.
    pub fn to_json(&self) -> serde_json::Value {
        let mut object = serde_json::Map::new();
        object.insert(
            "values".to_string(),
            serde_json::Value::Array(self.entries.iter().map(TagEntry::to_json).collect()),
        );
        if self.replace {
            object.insert("replace".to_string(), serde_json::Value::Bool(true));
        }
        serde_json::Value::Object(object)
    }

    pub fn from_json(
        registry: Identifier,
        tag: Identifier,
        value: &serde_json::Value,
    ) -> Result<Self, String> {
        let object = value
            .as_object()
            .ok_or_else(|| "tag file must be a JSON object".to_string())?;
        let values = object
            .get("values")
            .and_then(serde_json::Value::as_array)
            .ok_or_else(|| "tag file is missing array field values".to_string())?
            .iter()
            .map(TagEntry::from_json)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self::new(
            registry,
            tag,
            values,
            object
                .get("replace")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false),
        ))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TagEntry {
    pub id: Identifier,
    pub tag: bool,
    pub required: bool,
}

impl TagEntry {
    pub fn element(id: Identifier) -> Self {
        Self {
            id,
            tag: false,
            required: true,
        }
    }

    pub fn optional_element(id: Identifier) -> Self {
        Self {
            id,
            tag: false,
            required: false,
        }
    }

    pub fn tag(id: Identifier) -> Self {
        Self {
            id,
            tag: true,
            required: true,
        }
    }

    pub fn optional_tag(id: Identifier) -> Self {
        Self {
            id,
            tag: true,
            required: false,
        }
    }

    pub fn to_json(&self) -> serde_json::Value {
        let id = if self.tag {
            format!("#{}", self.id)
        } else {
            self.id.to_string()
        };
        if self.required {
            serde_json::Value::String(id)
        } else {
            serde_json::json!({"id": id, "required": false})
        }
    }

    pub fn from_json(value: &serde_json::Value) -> Result<Self, String> {
        if let Some(value) = value.as_str() {
            return Self::from_id(value, true);
        }
        let object = value
            .as_object()
            .ok_or_else(|| "tag entry must be a string or object".to_string())?;
        let id = object
            .get("id")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| "tag entry object is missing string field id".to_string())?;
        Self::from_id(
            id,
            object
                .get("required")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(true),
        )
    }

    fn from_id(value: &str, required: bool) -> Result<Self, String> {
        let (tag, id) = value
            .strip_prefix('#')
            .map_or((false, value), |id| (true, id));
        let id = Identifier::parse(id)?;
        Ok(Self {
            id,
            tag,
            required,
        })
    }

    pub fn visit_required_dependencies(&self, output: &mut Vec<Identifier>) {
        if self.tag && self.required {
            output.push(self.id.clone());
        }
    }

    pub fn visit_optional_dependencies(&self, output: &mut Vec<Identifier>) {
        if self.tag && !self.required {
            output.push(self.id.clone());
        }
    }

    pub fn verify_if_present<F, G>(&self, element_check: F, tag_check: G) -> bool
    where
        F: FnOnce(&Identifier) -> bool,
        G: FnOnce(&Identifier) -> bool,
    {
        !self.required || if self.tag { tag_check(&self.id) } else { element_check(&self.id) }
    }
}

impl fmt::Display for TagEntry {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.tag {
            formatter.write_str("#")?;
        }
        write!(formatter, "{}", self.id)?;
        if !self.required {
            formatter.write_str("?")?;
        }
        Ok(())
    }
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
        let mut definitions: BTreeMap<(Identifier, Identifier), Vec<TagEntry>> = BTreeMap::new();
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
                definitions.insert(key.clone(), Vec::new());
            }
            definitions.entry(key).or_default().extend(file.entries);
        }

        let mut tags = BTreeMap::new();
        let mut resolving = Vec::new();
        let keys = definitions.keys().cloned().collect::<Vec<_>>();
        for key in keys {
            resolve_tag_entries(
                &key,
                &definitions,
                registry,
                &mut tags,
                &mut resolving,
                &mut errors,
            );
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

fn resolve_tag_entries<T>(
    key: &(Identifier, Identifier),
    definitions: &BTreeMap<(Identifier, Identifier), Vec<TagEntry>>,
    registry: &Registry<T>,
    resolved: &mut BTreeMap<(Identifier, Identifier), Vec<Identifier>>,
    resolving: &mut Vec<(Identifier, Identifier)>,
    errors: &mut Vec<String>,
) -> Vec<Identifier> {
    if let Some(values) = resolved.get(key) {
        return values.clone();
    }
    if resolving.iter().any(|active| active == key) {
        errors.push(format!("cyclic tag reference {}", key.1));
        return Vec::new();
    }
    resolving.push(key.clone());
    let mut values = Vec::new();
    if let Some(entries) = definitions.get(key) {
        for entry in entries {
            if entry.tag {
                let dependency = (key.0.clone(), entry.id.clone());
                if !definitions.contains_key(&dependency) {
                    if entry.required {
                        errors.push(format!("missing required tag entry #{}", entry.id));
                    }
                    continue;
                }
                for value in resolve_tag_entries(
                    &dependency,
                    definitions,
                    registry,
                    resolved,
                    resolving,
                    errors,
                ) {
                    if !values.contains(&value) {
                        values.push(value);
                    }
                }
            } else if registry.get(&entry.id).is_some() {
                if !values.contains(&entry.id) {
                    values.push(entry.id.clone());
                }
            } else if entry.required {
                errors.push(format!("missing required tag entry {}", entry.id));
            }
        }
    }
    resolving.pop();
    resolved.insert(key.clone(), values.clone());
    values
}

mod builtin;
#[cfg(test)]
pub use builtin::*;

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
mod identifier_parity_tests;

#[cfg(all(test, vibecraft_has_decompiled_sources))]
mod tests;
