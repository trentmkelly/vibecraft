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

pub mod registries {
    pub const BLOCK: &str = "minecraft:block";
    pub const ITEM: &str = "minecraft:item";
    pub const ENTITY_TYPE: &str = "minecraft:entity_type";
    pub const DIMENSION_TYPE: &str = "minecraft:dimension_type";
    pub const BIOME: &str = "minecraft:worldgen/biome";
}

#[derive(Debug, Clone)]
pub struct BuiltInRegistries {
    pub blocks: Registry<String>,
    pub items: Registry<String>,
    pub entity_types: Registry<String>,
    pub dimension_types: Registry<String>,
    pub biomes: Registry<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataPackRegistryEntry {
    pub registry: Identifier,
    pub location: Identifier,
    pub value: String,
    pub lifecycle: Lifecycle,
}

#[derive(Debug, Clone)]
pub struct DynamicRegistryAccess {
    registries: BTreeMap<Identifier, Registry<String>>,
    order: Vec<Identifier>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BuiltInRegistryDescriptor {
    pub java_field: &'static str,
    pub registry: &'static str,
    pub default_key: Option<&'static str>,
    pub intrusive_holders: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServerReloadStage {
    BuiltInRegistries,
    DataPackRegistries,
    Tags,
    FrozenRegistries,
    Recipes,
    LootTables,
    Advancements,
    Functions,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerResourceReloadRequest {
    pub registry_entries: Vec<DataPackRegistryEntry>,
    pub tag_files: Vec<TagFile>,
}

#[derive(Debug, Clone)]
pub struct ServerResourceReload {
    registries: DynamicRegistryAccess,
    tags: BTreeMap<Identifier, LoadedTags>,
    stages: Vec<ServerReloadStage>,
}

impl ServerResourceReload {
    pub fn registries(&self) -> &DynamicRegistryAccess {
        &self.registries
    }

    pub fn tags(&self, registry: &Identifier) -> Option<&LoadedTags> {
        self.tags.get(registry)
    }

    pub fn stages(&self) -> &[ServerReloadStage] {
        &self.stages
    }
}

#[derive(Debug, Clone)]
pub struct ReloadableServerRegistries {
    builtins: BuiltInRegistries,
    last_successful: DynamicRegistryAccess,
    stages: Vec<ServerReloadStage>,
}

impl ReloadableServerRegistries {
    pub fn new(builtins: BuiltInRegistries) -> Self {
        let last_successful = DynamicRegistryAccess::from_builtins(&builtins);
        Self {
            builtins,
            last_successful,
            stages: vanilla_reload_stages(),
        }
    }

    pub fn reload(
        &mut self,
        request: ServerResourceReloadRequest,
    ) -> Result<ServerResourceReload, String> {
        let mut registries = DynamicRegistryAccess::from_builtins(&self.builtins);
        registries.apply_data_pack_entries(request.registry_entries)?;

        let mut tags = BTreeMap::new();
        let mut tag_files_by_registry: BTreeMap<Identifier, Vec<TagFile>> = BTreeMap::new();
        for file in request.tag_files {
            tag_files_by_registry
                .entry(file.registry.clone())
                .or_default()
                .push(file);
        }

        for (registry_id, files) in tag_files_by_registry {
            let registry = registries
                .registry(&registry_id)
                .ok_or_else(|| format!("tag reload references unknown registry {registry_id}"))?;
            let loaded = LoadedTags::load(registry, files).map_err(|errors| errors.join("; "))?;
            tags.insert(registry_id, loaded);
        }

        registries.freeze_all();
        self.last_successful = registries.clone();

        Ok(ServerResourceReload {
            registries,
            tags,
            stages: self.stages.clone(),
        })
    }

    pub fn last_successful(&self) -> &DynamicRegistryAccess {
        &self.last_successful
    }

    pub fn stages(&self) -> &[ServerReloadStage] {
        &self.stages
    }
}

fn vanilla_reload_stages() -> Vec<ServerReloadStage> {
    vec![
        ServerReloadStage::BuiltInRegistries,
        ServerReloadStage::DataPackRegistries,
        ServerReloadStage::Tags,
        ServerReloadStage::FrozenRegistries,
        ServerReloadStage::Recipes,
        ServerReloadStage::LootTables,
        ServerReloadStage::Advancements,
        ServerReloadStage::Functions,
    ]
}

impl DynamicRegistryAccess {
    pub fn from_builtins(builtins: &BuiltInRegistries) -> Self {
        let mut registries = BTreeMap::new();
        let mut order = Vec::new();
        for registry in [
            builtins.blocks.clone(),
            builtins.items.clone(),
            builtins.entity_types.clone(),
            builtins.dimension_types.clone(),
            builtins.biomes.clone(),
        ] {
            order.push(registry.registry_id().clone());
            registries.insert(registry.registry_id().clone(), registry);
        }
        Self { registries, order }
    }

    pub fn apply_data_pack_entries(
        &mut self,
        entries: impl IntoIterator<Item = DataPackRegistryEntry>,
    ) -> Result<(), String> {
        for entry in entries {
            let registry_id = entry.registry.clone();
            if !self.registries.contains_key(&registry_id) {
                self.order.push(registry_id.clone());
            }
            let registry = self
                .registries
                .entry(registry_id.clone())
                .or_insert_with(|| Registry::new(registry_id));
            if registry.is_frozen() {
                let snapshot = registry.serialize_with(Clone::clone);
                *registry = Registry::deserialize_with(snapshot, |value| Ok(value.to_string()))?;
            }
            registry.apply_data_pack_overrides([(entry.location, entry.value, entry.lifecycle)])?;
        }
        Ok(())
    }

    pub fn freeze_all(&mut self) {
        for registry in self.registries.values_mut() {
            registry.freeze();
        }
    }

    pub fn registry(&self, id: &Identifier) -> Option<&Registry<String>> {
        self.registries.get(id)
    }

    pub fn registry_ids(&self) -> Vec<Identifier> {
        self.order.clone()
    }
}

pub fn builtin_registry_manifest_26_1_2() -> Vec<BuiltInRegistryDescriptor> {
    BUILTIN_REGISTRY_MANIFEST_26_1_2.to_vec()
}

static BUILTIN_REGISTRY_MANIFEST_26_1_2: &[BuiltInRegistryDescriptor] = &[
    descriptor("GAME_EVENT", "minecraft:game_event", Some("step"), false),
    descriptor("SOUND_EVENT", "minecraft:sound_event", None, false),
    descriptor("FLUID", "minecraft:fluid", Some("empty"), true),
    descriptor("MOB_EFFECT", "minecraft:mob_effect", None, false),
    descriptor("BLOCK", "minecraft:block", Some("air"), true),
    descriptor(
        "DEBUG_SUBSCRIPTION",
        "minecraft:debug_subscription",
        None,
        false,
    ),
    descriptor("ENTITY_TYPE", "minecraft:entity_type", Some("pig"), true),
    descriptor("ITEM", "minecraft:item", Some("air"), true),
    descriptor("POTION", "minecraft:potion", None, false),
    descriptor("PARTICLE_TYPE", "minecraft:particle_type", None, false),
    descriptor(
        "BLOCK_ENTITY_TYPE",
        "minecraft:block_entity_type",
        None,
        true,
    ),
    descriptor("CUSTOM_STAT", "minecraft:custom_stat", None, false),
    descriptor(
        "CHUNK_STATUS",
        "minecraft:chunk_status",
        Some("empty"),
        false,
    ),
    descriptor("RULE_TEST", "minecraft:rule_test", None, false),
    descriptor(
        "RULE_BLOCK_ENTITY_MODIFIER",
        "minecraft:rule_block_entity_modifier",
        None,
        false,
    ),
    descriptor("POS_RULE_TEST", "minecraft:pos_rule_test", None, false),
    descriptor("MENU", "minecraft:menu", None, false),
    descriptor("RECIPE_TYPE", "minecraft:recipe_type", None, false),
    descriptor(
        "RECIPE_SERIALIZER",
        "minecraft:recipe_serializer",
        None,
        false,
    ),
    descriptor("ATTRIBUTE", "minecraft:attribute", None, false),
    descriptor(
        "POSITION_SOURCE_TYPE",
        "minecraft:position_source_type",
        None,
        false,
    ),
    descriptor(
        "COMMAND_ARGUMENT_TYPE",
        "minecraft:command_argument_type",
        None,
        false,
    ),
    descriptor("STAT_TYPE", "minecraft:stat_type", None, false),
    descriptor(
        "VILLAGER_TYPE",
        "minecraft:villager_type",
        Some("plains"),
        false,
    ),
    descriptor(
        "VILLAGER_PROFESSION",
        "minecraft:villager_profession",
        Some("none"),
        false,
    ),
    descriptor(
        "POINT_OF_INTEREST_TYPE",
        "minecraft:point_of_interest_type",
        None,
        false,
    ),
    descriptor(
        "MEMORY_MODULE_TYPE",
        "minecraft:memory_module_type",
        Some("dummy"),
        false,
    ),
    descriptor("SENSOR_TYPE", "minecraft:sensor_type", Some("dummy"), false),
    descriptor("ACTIVITY", "minecraft:activity", None, false),
    descriptor(
        "LOOT_POOL_ENTRY_TYPE",
        "minecraft:loot_pool_entry_type",
        None,
        false,
    ),
    descriptor(
        "LOOT_FUNCTION_TYPE",
        "minecraft:loot_function_type",
        None,
        false,
    ),
    descriptor(
        "LOOT_CONDITION_TYPE",
        "minecraft:loot_condition_type",
        None,
        false,
    ),
    descriptor(
        "LOOT_NUMBER_PROVIDER_TYPE",
        "minecraft:loot_number_provider_type",
        None,
        false,
    ),
    descriptor(
        "LOOT_NBT_PROVIDER_TYPE",
        "minecraft:loot_nbt_provider_type",
        None,
        false,
    ),
    descriptor(
        "LOOT_SCORE_PROVIDER_TYPE",
        "minecraft:loot_score_provider_type",
        None,
        false,
    ),
    descriptor(
        "FLOAT_PROVIDER_TYPE",
        "minecraft:float_provider_type",
        None,
        false,
    ),
    descriptor(
        "INT_PROVIDER_TYPE",
        "minecraft:int_provider_type",
        None,
        false,
    ),
    descriptor(
        "HEIGHT_PROVIDER_TYPE",
        "minecraft:height_provider_type",
        None,
        false,
    ),
    descriptor(
        "BLOCK_PREDICATE_TYPE",
        "minecraft:block_predicate_type",
        None,
        false,
    ),
    descriptor("CARVER", "minecraft:worldgen/carver", None, false),
    descriptor("FEATURE", "minecraft:worldgen/feature", None, false),
    descriptor(
        "STRUCTURE_PLACEMENT",
        "minecraft:worldgen/structure_placement",
        None,
        false,
    ),
    descriptor(
        "STRUCTURE_PIECE",
        "minecraft:worldgen/structure_piece",
        None,
        false,
    ),
    descriptor(
        "STRUCTURE_TYPE",
        "minecraft:worldgen/structure_type",
        None,
        false,
    ),
    descriptor(
        "PLACEMENT_MODIFIER_TYPE",
        "minecraft:worldgen/placement_modifier_type",
        None,
        false,
    ),
    descriptor(
        "BLOCKSTATE_PROVIDER_TYPE",
        "minecraft:worldgen/block_state_provider_type",
        None,
        false,
    ),
    descriptor(
        "FOLIAGE_PLACER_TYPE",
        "minecraft:worldgen/foliage_placer_type",
        None,
        false,
    ),
    descriptor(
        "TRUNK_PLACER_TYPE",
        "minecraft:worldgen/trunk_placer_type",
        None,
        false,
    ),
    descriptor(
        "ROOT_PLACER_TYPE",
        "minecraft:worldgen/root_placer_type",
        None,
        false,
    ),
    descriptor(
        "TREE_DECORATOR_TYPE",
        "minecraft:worldgen/tree_decorator_type",
        None,
        false,
    ),
    descriptor(
        "FEATURE_SIZE_TYPE",
        "minecraft:worldgen/feature_size_type",
        None,
        false,
    ),
    descriptor(
        "BIOME_SOURCE",
        "minecraft:worldgen/biome_source",
        None,
        false,
    ),
    descriptor(
        "CHUNK_GENERATOR",
        "minecraft:worldgen/chunk_generator",
        None,
        false,
    ),
    descriptor(
        "MATERIAL_CONDITION",
        "minecraft:worldgen/material_condition",
        None,
        false,
    ),
    descriptor(
        "MATERIAL_RULE",
        "minecraft:worldgen/material_rule",
        None,
        false,
    ),
    descriptor(
        "DENSITY_FUNCTION_TYPE",
        "minecraft:worldgen/density_function_type",
        None,
        false,
    ),
    descriptor("BLOCK_TYPE", "minecraft:block_type", None, false),
    descriptor(
        "STRUCTURE_PROCESSOR",
        "minecraft:worldgen/structure_processor",
        None,
        false,
    ),
    descriptor(
        "STRUCTURE_POOL_ELEMENT",
        "minecraft:worldgen/structure_pool_element",
        None,
        false,
    ),
    descriptor(
        "POOL_ALIAS_BINDING_TYPE",
        "minecraft:worldgen/pool_alias_binding",
        None,
        false,
    ),
    descriptor(
        "DECORATED_POT_PATTERN",
        "minecraft:decorated_pot_pattern",
        None,
        false,
    ),
    descriptor(
        "CREATIVE_MODE_TAB",
        "minecraft:creative_mode_tab",
        None,
        false,
    ),
    descriptor("TRIGGER_TYPES", "minecraft:trigger_type", None, false),
    descriptor(
        "NUMBER_FORMAT_TYPE",
        "minecraft:number_format_type",
        None,
        false,
    ),
    descriptor(
        "DATA_COMPONENT_TYPE",
        "minecraft:data_component_type",
        None,
        false,
    ),
    descriptor("GAME_RULE", "minecraft:game_rule", None, false),
    descriptor(
        "ENTITY_SUB_PREDICATE_TYPE",
        "minecraft:entity_sub_predicate_type",
        None,
        false,
    ),
    descriptor(
        "DATA_COMPONENT_PREDICATE_TYPE",
        "minecraft:data_component_predicate_type",
        None,
        false,
    ),
    descriptor(
        "MAP_DECORATION_TYPE",
        "minecraft:map_decoration_type",
        None,
        false,
    ),
    descriptor(
        "ENCHANTMENT_EFFECT_COMPONENT_TYPE",
        "minecraft:enchantment_effect_component_type",
        None,
        false,
    ),
    descriptor(
        "ENCHANTMENT_LEVEL_BASED_VALUE_TYPE",
        "minecraft:enchantment_level_based_value_type",
        None,
        false,
    ),
    descriptor(
        "ENCHANTMENT_ENTITY_EFFECT_TYPE",
        "minecraft:enchantment_entity_effect_type",
        None,
        false,
    ),
    descriptor(
        "ENCHANTMENT_LOCATION_BASED_EFFECT_TYPE",
        "minecraft:enchantment_location_based_effect_type",
        None,
        false,
    ),
    descriptor(
        "ENCHANTMENT_VALUE_EFFECT_TYPE",
        "minecraft:enchantment_value_effect_type",
        None,
        false,
    ),
    descriptor(
        "ENCHANTMENT_PROVIDER_TYPE",
        "minecraft:enchantment_provider_type",
        None,
        false,
    ),
    descriptor(
        "CONSUME_EFFECT_TYPE",
        "minecraft:consume_effect_type",
        None,
        false,
    ),
    descriptor("RECIPE_DISPLAY", "minecraft:recipe_display", None, false),
    descriptor("SLOT_DISPLAY", "minecraft:slot_display", None, false),
    descriptor(
        "RECIPE_BOOK_CATEGORY",
        "minecraft:recipe_book_category",
        None,
        false,
    ),
    descriptor("TICKET_TYPE", "minecraft:ticket_type", None, false),
    descriptor(
        "INCOMING_RPC_METHOD",
        "minecraft:incoming_rpc_methods",
        None,
        false,
    ),
    descriptor(
        "OUTGOING_RPC_METHOD",
        "minecraft:outgoing_rpc_methods",
        None,
        false,
    ),
    descriptor(
        "TEST_ENVIRONMENT_DEFINITION_TYPE",
        "minecraft:test_environment_definition_type",
        None,
        false,
    ),
    descriptor(
        "TEST_INSTANCE_TYPE",
        "minecraft:test_instance_type",
        None,
        false,
    ),
    descriptor(
        "SPAWN_CONDITION_TYPE",
        "minecraft:spawn_condition_type",
        None,
        false,
    ),
    descriptor("DIALOG_TYPE", "minecraft:dialog_type", None, false),
    descriptor(
        "DIALOG_ACTION_TYPE",
        "minecraft:dialog_action_type",
        None,
        false,
    ),
    descriptor(
        "INPUT_CONTROL_TYPE",
        "minecraft:input_control_type",
        None,
        false,
    ),
    descriptor(
        "DIALOG_BODY_TYPE",
        "minecraft:dialog_body_type",
        None,
        false,
    ),
    descriptor("PERMISSION_TYPE", "minecraft:permission_type", None, false),
    descriptor(
        "PERMISSION_CHECK_TYPE",
        "minecraft:permission_check_type",
        None,
        false,
    ),
    descriptor(
        "ENVIRONMENT_ATTRIBUTE",
        "minecraft:environment_attribute",
        None,
        false,
    ),
    descriptor("ATTRIBUTE_TYPE", "minecraft:attribute_type", None, false),
    descriptor(
        "SLOT_SOURCE_TYPE",
        "minecraft:slot_source_type",
        None,
        false,
    ),
    descriptor("TEST_FUNCTION", "minecraft:test_function", None, false),
];

const fn descriptor(
    java_field: &'static str,
    registry: &'static str,
    default_key: Option<&'static str>,
    intrusive_holders: bool,
) -> BuiltInRegistryDescriptor {
    BuiltInRegistryDescriptor {
        java_field,
        registry,
        default_key,
        intrusive_holders,
    }
}

impl BuiltInRegistries {
    pub fn bootstrap_26_1_2() -> Self {
        let mut blocks = Registry::new(Identifier::parse(registries::BLOCK).unwrap());
        for id in [
            "minecraft:air",
            "minecraft:stone",
            "minecraft:dirt",
            "minecraft:grass_block",
            "minecraft:bedrock",
            "minecraft:water",
            "minecraft:lava",
        ] {
            blocks
                .register(
                    Identifier::parse(id).unwrap(),
                    id.to_string(),
                    Lifecycle::Stable,
                )
                .unwrap();
        }

        let mut items = Registry::new(Identifier::parse(registries::ITEM).unwrap());
        for id in [
            "minecraft:air",
            "minecraft:stick",
            "minecraft:apple",
            "minecraft:stone",
            "minecraft:dirt",
            "minecraft:diamond_sword",
            "minecraft:netherite_chestplate",
        ] {
            items
                .register(
                    Identifier::parse(id).unwrap(),
                    id.to_string(),
                    Lifecycle::Stable,
                )
                .unwrap();
        }

        let mut entity_types = Registry::new(Identifier::parse(registries::ENTITY_TYPE).unwrap());
        for id in [
            "minecraft:player",
            "minecraft:pig",
            "minecraft:cow",
            "minecraft:armor_stand",
            "minecraft:item",
        ] {
            entity_types
                .register(
                    Identifier::parse(id).unwrap(),
                    id.to_string(),
                    Lifecycle::Stable,
                )
                .unwrap();
        }

        let mut dimension_types =
            Registry::new(Identifier::parse(registries::DIMENSION_TYPE).unwrap());
        for id in [
            "minecraft:overworld",
            "minecraft:overworld_caves",
            "minecraft:the_nether",
            "minecraft:the_end",
        ] {
            dimension_types
                .register(
                    Identifier::parse(id).unwrap(),
                    id.to_string(),
                    Lifecycle::Stable,
                )
                .unwrap();
        }

        let mut biomes = Registry::new(Identifier::parse(registries::BIOME).unwrap());
        for id in [
            "minecraft:plains",
            "minecraft:forest",
            "minecraft:desert",
            "minecraft:nether_wastes",
            "minecraft:the_end",
        ] {
            biomes
                .register(
                    Identifier::parse(id).unwrap(),
                    id.to_string(),
                    Lifecycle::Stable,
                )
                .unwrap();
        }

        blocks.freeze();
        items.freeze();
        entity_types.freeze();
        dimension_types.freeze();
        biomes.freeze();

        Self {
            blocks,
            items,
            entity_types,
            dimension_types,
            biomes,
        }
    }

    pub fn registry_ids(&self) -> Vec<Identifier> {
        vec![
            self.blocks.registry_id().clone(),
            self.items.registry_id().clone(),
            self.entity_types.registry_id().clone(),
            self.dimension_types.registry_id().clone(),
            self.biomes.registry_id().clone(),
        ]
    }
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
        builtin_registry_manifest_26_1_2, feature_flags, registries, FeatureFlagRegistry, Holder,
        HolderSet, Identifier, Lifecycle, Registry, TagKey,
    };

    const INTENTIONALLY_OMITTED_REGISTRIES: &[(&str, &str)] = &[
        (
            "minecraft:advancement",
            "not referenced by BuiltInRegistries in 26.1.2",
        ),
        (
            "minecraft:banner_pattern",
            "not referenced by BuiltInRegistries in 26.1.2",
        ),
        (
            "minecraft:cat_sound_variant",
            "not referenced by BuiltInRegistries in 26.1.2",
        ),
        (
            "minecraft:cat_variant",
            "not referenced by BuiltInRegistries in 26.1.2",
        ),
        (
            "minecraft:chat_type",
            "not referenced by BuiltInRegistries in 26.1.2",
        ),
        (
            "minecraft:chicken_sound_variant",
            "not referenced by BuiltInRegistries in 26.1.2",
        ),
        (
            "minecraft:chicken_variant",
            "not referenced by BuiltInRegistries in 26.1.2",
        ),
        (
            "minecraft:cow_sound_variant",
            "not referenced by BuiltInRegistries in 26.1.2",
        ),
        (
            "minecraft:cow_variant",
            "not referenced by BuiltInRegistries in 26.1.2",
        ),
        (
            "minecraft:damage_type",
            "not referenced by BuiltInRegistries in 26.1.2",
        ),
        (
            "minecraft:dialog",
            "not referenced by BuiltInRegistries in 26.1.2",
        ),
        (
            "minecraft:dimension",
            "not referenced by BuiltInRegistries in 26.1.2",
        ),
        (
            "minecraft:dimension_type",
            "not referenced by BuiltInRegistries in 26.1.2",
        ),
        (
            "minecraft:enchantment",
            "not referenced by BuiltInRegistries in 26.1.2",
        ),
        (
            "minecraft:enchantment_provider",
            "not referenced by BuiltInRegistries in 26.1.2",
        ),
        (
            "minecraft:frog_variant",
            "not referenced by BuiltInRegistries in 26.1.2",
        ),
        (
            "minecraft:instrument",
            "not referenced by BuiltInRegistries in 26.1.2",
        ),
        (
            "minecraft:item_modifier",
            "not referenced by BuiltInRegistries in 26.1.2",
        ),
        (
            "minecraft:jukebox_song",
            "not referenced by BuiltInRegistries in 26.1.2",
        ),
        (
            "minecraft:loot_table",
            "not referenced by BuiltInRegistries in 26.1.2",
        ),
        (
            "minecraft:painting_variant",
            "not referenced by BuiltInRegistries in 26.1.2",
        ),
        (
            "minecraft:pig_sound_variant",
            "not referenced by BuiltInRegistries in 26.1.2",
        ),
        (
            "minecraft:pig_variant",
            "not referenced by BuiltInRegistries in 26.1.2",
        ),
        (
            "minecraft:predicate",
            "not referenced by BuiltInRegistries in 26.1.2",
        ),
        (
            "minecraft:recipe",
            "not referenced by BuiltInRegistries in 26.1.2",
        ),
        (
            "minecraft:test_environment",
            "not referenced by BuiltInRegistries in 26.1.2",
        ),
        (
            "minecraft:test_instance",
            "not referenced by BuiltInRegistries in 26.1.2",
        ),
        (
            "minecraft:timeline",
            "not referenced by BuiltInRegistries in 26.1.2",
        ),
        (
            "minecraft:trade_set",
            "not referenced by BuiltInRegistries in 26.1.2",
        ),
        (
            "minecraft:trial_spawner",
            "not referenced by BuiltInRegistries in 26.1.2",
        ),
        (
            "minecraft:trim_material",
            "not referenced by BuiltInRegistries in 26.1.2",
        ),
        (
            "minecraft:trim_pattern",
            "not referenced by BuiltInRegistries in 26.1.2",
        ),
        (
            "minecraft:villager_trade",
            "not referenced by BuiltInRegistries in 26.1.2",
        ),
        (
            "minecraft:wolf_sound_variant",
            "not referenced by BuiltInRegistries in 26.1.2",
        ),
        (
            "minecraft:wolf_variant",
            "not referenced by BuiltInRegistries in 26.1.2",
        ),
        (
            "minecraft:world_clock",
            "not referenced by BuiltInRegistries in 26.1.2",
        ),
        (
            "minecraft:worldgen/biome",
            "not referenced by BuiltInRegistries in 26.1.2",
        ),
        (
            "minecraft:worldgen/configured_carver",
            "not referenced by BuiltInRegistries in 26.1.2",
        ),
        (
            "minecraft:worldgen/configured_feature",
            "not referenced by BuiltInRegistries in 26.1.2",
        ),
        (
            "minecraft:worldgen/density_function",
            "not referenced by BuiltInRegistries in 26.1.2",
        ),
        (
            "minecraft:worldgen/flat_level_generator_preset",
            "not referenced by BuiltInRegistries in 26.1.2",
        ),
        (
            "minecraft:worldgen/multi_noise_biome_source_parameter_list",
            "not referenced by BuiltInRegistries in 26.1.2",
        ),
        (
            "minecraft:worldgen/noise",
            "not referenced by BuiltInRegistries in 26.1.2",
        ),
        (
            "minecraft:worldgen/noise_settings",
            "not referenced by BuiltInRegistries in 26.1.2",
        ),
        (
            "minecraft:worldgen/placed_feature",
            "not referenced by BuiltInRegistries in 26.1.2",
        ),
        (
            "minecraft:worldgen/processor_list",
            "not referenced by BuiltInRegistries in 26.1.2",
        ),
        (
            "minecraft:worldgen/structure",
            "not referenced by BuiltInRegistries in 26.1.2",
        ),
        (
            "minecraft:worldgen/structure_set",
            "not referenced by BuiltInRegistries in 26.1.2",
        ),
        (
            "minecraft:worldgen/template_pool",
            "not referenced by BuiltInRegistries in 26.1.2",
        ),
        (
            "minecraft:worldgen/world_preset",
            "not referenced by BuiltInRegistries in 26.1.2",
        ),
        (
            "minecraft:zombie_nautilus_variant",
            "not referenced by BuiltInRegistries in 26.1.2",
        ),
    ];

    fn parse_java_constant_to_path_mapping(
        source: &str,
    ) -> std::collections::BTreeMap<String, String> {
        let mut mapping = std::collections::BTreeMap::new();
        let create_call = "createRegistryKey(";
        let mut cursor = 0usize;

        while let Some(create_offset) = source[cursor..].find(create_call) {
            let create_pos = cursor + create_offset;
            let before_call = &source[..create_pos];
            let line_start = before_call.rfind('\n').map_or(0, |idx| idx + 1);
            let before_call_line = &before_call[line_start..];
            if let Some(eq_pos) = before_call_line.rfind('=') {
                let lhs = before_call_line[..eq_pos].trim();
                if let Some(constant) = lhs.split_whitespace().last() {
                    let after_call = &source[create_pos + create_call.len()..];
                    if let Some(start_quote) = after_call.find('\"') {
                        let quoted = &after_call[start_quote + 1..];
                        if let Some(end_quote) = quoted.find('\"') {
                            mapping.insert(constant.to_string(), quoted[..end_quote].to_string());
                        }
                    }
                }
            }
            cursor = create_pos + create_call.len();
        }

        assert!(
            !mapping.is_empty(),
            "could not parse java registry constant mapping"
        );
        mapping
    }

    fn is_java_all_caps_registry_identifier(candidate: &str) -> bool {
        !candidate.is_empty()
            && candidate
                .chars()
                .all(|ch| ch.is_ascii_uppercase() || ch.is_ascii_digit() || ch == '_')
    }

    fn parse_java_registry_refs_from_source(source: &str, prefix: &str) -> Vec<String> {
        let mut constants = Vec::new();
        let mut cursor = source;

        while let Some(index) = cursor.find(prefix) {
            let after = &cursor[index + prefix.len()..];
            let mut len = 0usize;
            for ch in after.bytes() {
                if ch.is_ascii_alphanumeric() || ch == b'_' {
                    len += 1;
                } else {
                    break;
                }
            }
            if len > 0 {
                let candidate = &after[..len];
                if is_java_all_caps_registry_identifier(candidate) {
                    constants.push(candidate.to_string());
                }
            }
            cursor = &after[len..];
        }

        constants
    }

    fn parse_java_registry_ids() -> std::collections::BTreeSet<String> {
        let source = include_str!(
            "../../../decompiled-server-26.1.2/net/minecraft/core/registries/Registries.java"
        );
        let mut registry_ids = std::collections::BTreeSet::new();
        for id in parse_java_constant_to_path_mapping(source).values() {
            registry_ids.insert(format!("minecraft:{id}", id = id));
        }
        if registry_ids.is_empty() {
            panic!("could not parse java registry ids");
        }
        registry_ids
    }

    fn parse_builtin_registry_ids() -> std::collections::BTreeSet<String> {
        let registries_java = include_str!(
            "../../../decompiled-server-26.1.2/net/minecraft/core/registries/Registries.java"
        );
        let mapping = parse_java_constant_to_path_mapping(registries_java);
        let source =
            include_str!("../../../decompiled-server-26.1.2/net/minecraft/core/registries/BuiltInRegistries.java");
        let mut registry_ids = std::collections::BTreeSet::new();
        for constant in parse_java_registry_refs_from_source(source, "Registries.") {
            if constant == "ROOT_REGISTRY_NAME" || constant == "REGISTRY" {
                continue;
            }
            if let Some(id) = mapping.get(&constant) {
                registry_ids.insert(format!("minecraft:{id}", id = id));
            } else {
                panic!("unmapped registry constant {constant} in BuiltInRegistries.java");
            }
        }
        if registry_ids.is_empty() {
            panic!("could not parse built-in registry ids");
        }
        registry_ids
    }

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
    fn registry_backed_keys_round_trip_json_nbt_and_network_forms() {
        let key = super::ResourceKey::<String>::new(
            Identifier::parse(registries::ITEM).unwrap(),
            Identifier::parse("minecraft:stick").unwrap(),
        );

        let decoded_json = super::ResourceKey::<String>::from_json_object(&key.to_json_object())
            .expect("json object should decode");
        assert_eq!(decoded_json, key);

        let decoded_nbt =
            super::ResourceKey::<String>::from_nbt(&key.to_nbt()).expect("nbt should decode");
        assert_eq!(decoded_nbt, key);

        let mut bytes = Vec::new();
        key.write_network(&mut bytes).unwrap();
        let mut input = crate::network::codec::cursor(bytes);
        assert_eq!(
            super::ResourceKey::<String>::read_network(&mut input).unwrap(),
            key
        );
    }

    #[test]
    fn registry_network_ids_resolve_to_stable_resource_keys() {
        let mut registry = Registry::new(Identifier::parse(registries::ITEM).unwrap());
        let stick = Identifier::parse("minecraft:stick").unwrap();
        let apple = Identifier::parse("minecraft:apple").unwrap();
        registry
            .register(stick.clone(), "stick".to_string(), Lifecycle::Stable)
            .unwrap();
        registry
            .register(apple.clone(), "apple".to_string(), Lifecycle::Stable)
            .unwrap();

        let mut bytes = Vec::new();
        registry.write_network_id(&mut bytes, &apple).unwrap();
        let mut input = crate::network::codec::cursor(bytes);
        let key = registry.read_network_key(&mut input).unwrap();

        assert_eq!(key.registry(), registry.registry_id());
        assert_eq!(key.location(), &apple);
        assert_eq!(registry.id_for_location(&stick), Some(0));
        assert_eq!(registry.id_for_location(&apple), Some(1));

        let mut invalid = Vec::new();
        crate::network::codec::write_registry_value_id(
            &mut invalid,
            crate::network::codec::RegistryValueId(99),
        )
        .unwrap();
        let err = registry
            .read_network_key(&mut crate::network::codec::cursor(invalid))
            .unwrap_err();
        assert_eq!(err.kind(), std::io::ErrorKind::InvalidData);
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
    fn builtin_registries_bootstrap_before_datapack_overrides() {
        let builtins = super::BuiltInRegistries::bootstrap_26_1_2();
        assert_eq!(
            builtins.registry_ids(),
            vec![
                Identifier::parse(registries::BLOCK).unwrap(),
                Identifier::parse(registries::ITEM).unwrap(),
                Identifier::parse(registries::ENTITY_TYPE).unwrap(),
                Identifier::parse(registries::DIMENSION_TYPE).unwrap(),
                Identifier::parse(registries::BIOME).unwrap(),
            ]
        );
        assert_eq!(
            builtins
                .blocks
                .get(&Identifier::parse("minecraft:air").unwrap())
                .unwrap()
                .id(),
            0
        );
        assert_eq!(
            builtins
                .dimension_types
                .get(&Identifier::parse("minecraft:the_nether").unwrap())
                .unwrap()
                .value(),
            "minecraft:the_nether"
        );
        assert!(builtins.blocks.is_frozen());
        assert!(builtins.items.is_frozen());
        assert!(builtins.entity_types.is_frozen());
        assert!(builtins.dimension_types.is_frozen());
        assert!(builtins.biomes.is_frozen());
    }

    #[test]
    fn builtin_registry_descriptors_cover_java_builtins_or_are_documented() {
        let java_registry_ids = parse_java_registry_ids();
        let built_in_registry_ids = parse_builtin_registry_ids();
        let manifest_registry_ids: std::collections::BTreeSet<String> =
            builtin_registry_manifest_26_1_2()
                .into_iter()
                .map(|entry| entry.registry.to_string())
                .collect();
        let intentional: std::collections::BTreeMap<&str, &str> =
            INTENTIONALLY_OMITTED_REGISTRIES.iter().copied().collect();

        assert!(
            built_in_registry_ids.is_subset(&java_registry_ids),
            "BuiltInRegistries.java references are not in sync with Registries.java",
        );
        assert!(
            built_in_registry_ids == manifest_registry_ids,
            "BuiltInRegistries.java references are not aligned with Rust manifest",
        );

        for registry_id in java_registry_ids.iter() {
            if built_in_registry_ids.contains(registry_id)
                || manifest_registry_ids.contains(registry_id)
            {
                continue;
            }
            if let Some(reason) = intentional.get(registry_id.as_str()) {
                assert!(
                    !reason.trim().is_empty(),
                    "omission reason missing for intentionally omitted {registry_id}"
                );
            } else {
                panic!("missing built-in registry descriptor for {registry_id} with no omission reason");
            }
        }

        for registry_id in manifest_registry_ids.iter() {
            assert!(
                java_registry_ids.contains(registry_id),
                "manifest includes unknown registry {registry_id} not present in Registries.java"
            );
        }
    }

    #[test]
    fn builtin_registry_manifest_matches_26_1_2_builtin_registration_order() {
        let manifest = builtin_registry_manifest_26_1_2();
        assert_eq!(manifest.len(), 95);
        assert_eq!(manifest[0].java_field, "GAME_EVENT");
        assert_eq!(manifest[0].registry, "minecraft:game_event");
        assert_eq!(manifest[0].default_key, Some("step"));
        assert_eq!(manifest[4].java_field, "BLOCK");
        assert_eq!(manifest[4].registry, "minecraft:block");
        assert_eq!(manifest[4].default_key, Some("air"));
        assert!(manifest[4].intrusive_holders);
        assert_eq!(manifest[6].java_field, "ENTITY_TYPE");
        assert_eq!(manifest[7].java_field, "ITEM");
        assert_eq!(manifest[56].java_field, "BLOCK_TYPE");
        assert_eq!(manifest[81].registry, "minecraft:outgoing_rpc_methods");
        assert_eq!(manifest.last().unwrap().java_field, "TEST_FUNCTION");

        let mut seen = std::collections::BTreeSet::new();
        for descriptor in &manifest {
            assert!(
                seen.insert(descriptor.registry),
                "duplicate registry {}",
                descriptor.registry
            );
            Identifier::parse(descriptor.registry).unwrap();
        }
    }

    #[test]
    fn dynamic_registry_access_applies_datapack_entries_over_builtins() {
        let builtins = super::BuiltInRegistries::bootstrap_26_1_2();
        let mut dynamic = super::DynamicRegistryAccess::from_builtins(&builtins);
        dynamic
            .apply_data_pack_entries(vec![
                super::DataPackRegistryEntry {
                    registry: Identifier::parse(registries::BIOME).unwrap(),
                    location: Identifier::parse("minecraft:plains").unwrap(),
                    value: "pack plains".to_string(),
                    lifecycle: Lifecycle::Experimental,
                },
                super::DataPackRegistryEntry {
                    registry: Identifier::parse("minecraft:chat_type").unwrap(),
                    location: Identifier::parse("minecraft:chat").unwrap(),
                    value: "chat codec".to_string(),
                    lifecycle: Lifecycle::Stable,
                },
            ])
            .unwrap();

        let biomes = dynamic
            .registry(&Identifier::parse(registries::BIOME).unwrap())
            .unwrap();
        assert_eq!(
            biomes
                .get(&Identifier::parse("minecraft:plains").unwrap())
                .unwrap()
                .value(),
            "pack plains"
        );
        assert_eq!(
            dynamic
                .registry(&Identifier::parse("minecraft:chat_type").unwrap())
                .unwrap()
                .get(&Identifier::parse("minecraft:chat").unwrap())
                .unwrap()
                .value(),
            "chat codec"
        );
        dynamic.freeze_all();
        assert!(dynamic
            .registry(&Identifier::parse(registries::BIOME).unwrap())
            .unwrap()
            .is_frozen());
    }

    #[test]
    fn dynamic_registry_access_preserves_builtin_then_datapack_registry_order() {
        let builtins = super::BuiltInRegistries::bootstrap_26_1_2();
        let mut dynamic = super::DynamicRegistryAccess::from_builtins(&builtins);
        dynamic
            .apply_data_pack_entries(vec![
                super::DataPackRegistryEntry {
                    registry: Identifier::parse("minecraft:chat_type").unwrap(),
                    location: Identifier::parse("minecraft:chat").unwrap(),
                    value: "chat codec".to_string(),
                    lifecycle: Lifecycle::Stable,
                },
                super::DataPackRegistryEntry {
                    registry: Identifier::parse(registries::BIOME).unwrap(),
                    location: Identifier::parse("minecraft:forest").unwrap(),
                    value: "pack forest".to_string(),
                    lifecycle: Lifecycle::Stable,
                },
                super::DataPackRegistryEntry {
                    registry: Identifier::parse("minecraft:damage_type").unwrap(),
                    location: Identifier::parse("minecraft:generic").unwrap(),
                    value: "damage codec".to_string(),
                    lifecycle: Lifecycle::Stable,
                },
            ])
            .unwrap();

        assert_eq!(
            dynamic.registry_ids(),
            vec![
                Identifier::parse(registries::BLOCK).unwrap(),
                Identifier::parse(registries::ITEM).unwrap(),
                Identifier::parse(registries::ENTITY_TYPE).unwrap(),
                Identifier::parse(registries::DIMENSION_TYPE).unwrap(),
                Identifier::parse(registries::BIOME).unwrap(),
                Identifier::parse("minecraft:chat_type").unwrap(),
                Identifier::parse("minecraft:damage_type").unwrap(),
            ]
        );
    }

    #[test]
    fn tag_loading_handles_replace_optional_entries_and_errors() {
        let mut registry = Registry::new(Identifier::parse(registries::ITEM).unwrap());
        registry
            .register(
                Identifier::parse("stick").unwrap(),
                "stick".to_string(),
                Lifecycle::Stable,
            )
            .unwrap();
        registry
            .register(
                Identifier::parse("apple").unwrap(),
                "apple".to_string(),
                Lifecycle::Stable,
            )
            .unwrap();
        let tag = Identifier::parse("test/items").unwrap();

        let loaded = super::LoadedTags::load(
            &registry,
            vec![
                super::TagFile {
                    registry: Identifier::parse(registries::ITEM).unwrap(),
                    tag: tag.clone(),
                    replace: false,
                    entries: vec![super::TagEntry {
                        id: Identifier::parse("stick").unwrap(),
                        required: true,
                    }],
                },
                super::TagFile {
                    registry: Identifier::parse(registries::ITEM).unwrap(),
                    tag: tag.clone(),
                    replace: true,
                    entries: vec![
                        super::TagEntry {
                            id: Identifier::parse("apple").unwrap(),
                            required: true,
                        },
                        super::TagEntry {
                            id: Identifier::parse("missing_optional").unwrap(),
                            required: false,
                        },
                    ],
                },
            ],
        )
        .unwrap();

        assert_eq!(
            loaded
                .values(&Identifier::parse(registries::ITEM).unwrap(), &tag)
                .unwrap(),
            &[Identifier::parse("apple").unwrap()]
        );

        let errors = super::LoadedTags::load(
            &registry,
            vec![super::TagFile {
                registry: Identifier::parse(registries::ITEM).unwrap(),
                tag,
                replace: false,
                entries: vec![super::TagEntry {
                    id: Identifier::parse("missing_required").unwrap(),
                    required: true,
                }],
            }],
        )
        .unwrap_err();
        assert_eq!(
            errors,
            vec!["missing required tag entry minecraft:missing_required"]
        );
    }

    #[test]
    fn reloadable_server_registries_order_datapack_registries_before_tags_and_freeze() {
        let builtins = super::BuiltInRegistries::bootstrap_26_1_2();
        let mut reloadable = super::ReloadableServerRegistries::new(builtins);
        let biome_registry = Identifier::parse(registries::BIOME).unwrap();
        let custom_biome = Identifier::parse("example:glade").unwrap();
        let tag = Identifier::parse("minecraft:is_overworld").unwrap();

        let reload = reloadable
            .reload(super::ServerResourceReloadRequest {
                registry_entries: vec![super::DataPackRegistryEntry {
                    registry: biome_registry.clone(),
                    location: custom_biome.clone(),
                    value: "glade codec".to_string(),
                    lifecycle: Lifecycle::Experimental,
                }],
                tag_files: vec![super::TagFile {
                    registry: biome_registry.clone(),
                    tag: tag.clone(),
                    replace: true,
                    entries: vec![super::TagEntry {
                        id: custom_biome.clone(),
                        required: true,
                    }],
                }],
            })
            .unwrap();

        assert_eq!(
            reload.stages(),
            &[
                super::ServerReloadStage::BuiltInRegistries,
                super::ServerReloadStage::DataPackRegistries,
                super::ServerReloadStage::Tags,
                super::ServerReloadStage::FrozenRegistries,
                super::ServerReloadStage::Recipes,
                super::ServerReloadStage::LootTables,
                super::ServerReloadStage::Advancements,
                super::ServerReloadStage::Functions,
            ]
        );
        assert!(reload
            .registries()
            .registry(&biome_registry)
            .unwrap()
            .is_frozen());
        assert_eq!(
            reload
                .registries()
                .registry(&biome_registry)
                .unwrap()
                .get(&custom_biome)
                .unwrap()
                .value(),
            "glade codec"
        );
        assert_eq!(
            reload
                .tags(&biome_registry)
                .unwrap()
                .values(&biome_registry, &tag),
            Some([custom_biome].as_slice())
        );
    }

    #[test]
    fn reloadable_server_registries_keep_last_successful_state_on_tag_failure() {
        let builtins = super::BuiltInRegistries::bootstrap_26_1_2();
        let mut reloadable = super::ReloadableServerRegistries::new(builtins);
        let item_registry = Identifier::parse(registries::ITEM).unwrap();
        let stick = Identifier::parse("minecraft:stick").unwrap();
        let tag = Identifier::parse("minecraft:test_items").unwrap();

        reloadable
            .reload(super::ServerResourceReloadRequest {
                registry_entries: Vec::new(),
                tag_files: vec![super::TagFile {
                    registry: item_registry.clone(),
                    tag: tag.clone(),
                    replace: true,
                    entries: vec![super::TagEntry {
                        id: stick.clone(),
                        required: true,
                    }],
                }],
            })
            .unwrap();

        let before = reloadable
            .last_successful()
            .registry(&item_registry)
            .unwrap()
            .get(&stick)
            .unwrap()
            .id();
        let err = reloadable
            .reload(super::ServerResourceReloadRequest {
                registry_entries: Vec::new(),
                tag_files: vec![super::TagFile {
                    registry: item_registry.clone(),
                    tag,
                    replace: true,
                    entries: vec![super::TagEntry {
                        id: Identifier::parse("minecraft:missing_required").unwrap(),
                        required: true,
                    }],
                }],
            })
            .unwrap_err();

        assert_eq!(err, "missing required tag entry minecraft:missing_required");
        assert_eq!(
            reloadable
                .last_successful()
                .registry(&item_registry)
                .unwrap()
                .get(&stick)
                .unwrap()
                .id(),
            before
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
