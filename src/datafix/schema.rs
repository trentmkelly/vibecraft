//! Schemas: the per-version type registry (Java `com.mojang.datafixers.schemas.Schema`).
//!
//! A Java `Schema` re-runs its parent's `registerTypes` / `registerEntities` /
//! `registerBlockEntities` against itself (the default "SAME" schema does nothing
//! else), and version classes such as `V100` call `super` first and then add or
//! override entries. Because templates here refer to other types by reference
//! name and are resolved late, that whole mechanism is equivalent to cloning the
//! parent's registries and applying the version's modifications, which is what
//! [`Schema::derive`] does.

use std::collections::{BTreeMap, HashMap};

use super::references::TypeReference;
use super::template::{dsl, ChoiceSet, Tmpl};

/// Ordering key of a schema/fix (Java `DataFixUtils.makeKey(version, subVersion)`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VersionKey {
    /// The `DataVersion`.
    pub version: i32,
    /// Sub version used to order several schemas sharing one data version.
    pub sub_version: i32,
}

impl VersionKey {
    /// `DataFixUtils.makeKey(version)`.
    pub const fn new(version: i32) -> Self {
        Self {
            version,
            sub_version: 0,
        }
    }

    /// `DataFixUtils.makeKey(version, subVersion)`.
    pub const fn with_sub(version: i32, sub_version: i32) -> Self {
        Self {
            version,
            sub_version,
        }
    }
}

/// One version's type registry.
#[derive(Debug, Clone)]
pub struct Schema {
    /// This schema's key.
    pub key: VersionKey,
    types: HashMap<TypeReference, Tmpl>,
    entities: BTreeMap<String, Tmpl>,
    block_entities: BTreeMap<String, Tmpl>,
    named_choices: BTreeMap<&'static str, BTreeMap<String, Tmpl>>,
}

impl Schema {
    /// The first schema of a chain (Java `V99`): empty until its registration runs.
    pub fn root(key: VersionKey) -> Self {
        Self {
            key,
            types: HashMap::new(),
            entities: BTreeMap::new(),
            block_entities: BTreeMap::new(),
            named_choices: BTreeMap::new(),
        }
    }

    /// A child schema: a copy of `parent`'s registries under a new key. The
    /// caller then applies the version's `registerTypes` / `registerEntities` /
    /// `registerBlockEntities` overrides.
    pub fn derive(parent: &Schema, key: VersionKey) -> Self {
        Self {
            key,
            ..parent.clone()
        }
    }

    /// `Schema.registerType(recursive, reference, template)` (recursion is implicit).
    pub fn register_type(&mut self, reference: TypeReference, tmpl: Tmpl) {
        self.types.insert(reference, tmpl);
    }

    /// `Schema.register(entityTypes, name, template)`.
    pub fn register_entity(&mut self, name: &str, tmpl: Tmpl) {
        self.entities.insert(name.to_string(), tmpl);
    }

    /// `Schema.registerSimple(entityTypes, name)`.
    pub fn register_simple_entity(&mut self, name: &str) {
        self.register_entity(name, dsl::remainder());
    }

    /// `Schema.register(blockEntityTypes, name, template)`.
    pub fn register_block_entity(&mut self, name: &str, tmpl: Tmpl) {
        self.block_entities.insert(name.to_string(), tmpl);
    }

    /// `Schema.registerSimple(blockEntityTypes, name)`.
    pub fn register_simple_block_entity(&mut self, name: &str) {
        self.register_block_entity(name, dsl::remainder());
    }

    /// Registers a branch of a schema-local choice map (see [`ChoiceSet::Named`]).
    pub fn register_named_choice(&mut self, map: &'static str, name: &str, tmpl: Tmpl) {
        self.named_choices
            .entry(map)
            .or_default()
            .insert(name.to_string(), tmpl);
    }

    /// Forgets every registered type (`registerTypes` implementations that do not
    /// call `super`).
    pub fn clear_types(&mut self) {
        self.types.clear();
    }

    /// `map.put(new, map.remove(old))` on the entity registry; panics when `old`
    /// is not registered (the Java lookup would leave a null template behind).
    pub fn rename_entity(&mut self, old: &str, new: &str) {
        let template = self
            .entities
            .remove(old)
            .unwrap_or_else(|| panic!("Didn't find {old} in schema"));
        self.entities.insert(new.to_string(), template);
    }

    /// `map.put(new, map.remove(old))` on the block entity registry.
    pub fn rename_block_entity(&mut self, old: &str, new: &str) {
        let template = self
            .block_entities
            .remove(old)
            .unwrap_or_else(|| panic!("Didn't find {old} in schema"));
        self.block_entities.insert(new.to_string(), template);
    }

    /// Replaces the whole entity registry (`Map map = Maps.newHashMap()` in
    /// `registerEntities` implementations that do not call `super`).
    pub fn clear_entities(&mut self) {
        self.entities.clear();
    }

    /// Replaces the whole block entity registry.
    pub fn clear_block_entities(&mut self) {
        self.block_entities.clear();
    }

    /// `map.remove(name)` on the entity registry.
    pub fn remove_entity(&mut self, name: &str) -> Option<Tmpl> {
        self.entities.remove(name)
    }

    /// `map.remove(name)` on the block entity registry.
    pub fn remove_block_entity(&mut self, name: &str) -> Option<Tmpl> {
        self.block_entities.remove(name)
    }

    /// The template registered for a reference (Java `getTypeRaw`).
    pub fn type_of(&self, reference: TypeReference) -> Option<&Tmpl> {
        self.types.get(reference)
    }

    /// Whether the entity registry contains `name`.
    pub fn has_entity(&self, name: &str) -> bool {
        self.entities.contains_key(name)
    }

    /// The template of a choice branch (Java `Schema.getChoiceType`).
    pub fn choice(&self, set: ChoiceSet, name: &str) -> Option<&Tmpl> {
        match set {
            ChoiceSet::Entities => self.entities.get(name),
            ChoiceSet::BlockEntities => self.block_entities.get(name),
            ChoiceSet::Named(map) => self.named_choices.get(map)?.get(name),
        }
    }

    /// All branch templates of a choice registry.
    pub fn choices(&self, set: ChoiceSet) -> Vec<(&String, &Tmpl)> {
        match set {
            ChoiceSet::Entities => self.entities.iter().collect(),
            ChoiceSet::BlockEntities => self.block_entities.iter().collect(),
            ChoiceSet::Named(map) => self
                .named_choices
                .get(map)
                .map(|choices| choices.iter().collect())
                .unwrap_or_default(),
        }
    }
}
