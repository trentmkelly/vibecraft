//! Registry storage: `MappedRegistry`, `RegistrationInfo` and the layered
//! `RegistryAccess` that the server keeps after startup.

use std::collections::{BTreeMap, HashMap};

use serde_json::Value;

use crate::network::configuration::KnownPack;
use crate::registry::{Identifier, Lifecycle};

/// `RegistrationInfo`: where an element came from and how stable it is.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegistrationInfo {
    /// The known pack the element was loaded from, when the source pack has one.
    pub known_pack: Option<KnownPack>,
    /// `Lifecycle.stable()` for vanilla packs, `experimental()` otherwise.
    pub lifecycle: Lifecycle,
}

impl RegistrationInfo {
    /// `ResourceManagerRegistryLoadTask.REGISTRATION_INFO_CACHE`.
    pub fn for_pack(known_pack: Option<KnownPack>) -> Self {
        let lifecycle = match &known_pack {
            Some(pack) if pack.is_vanilla() => Lifecycle::Stable,
            _ => Lifecycle::Experimental,
        };
        Self {
            known_pack,
            lifecycle,
        }
    }

    /// Info for static (built-in) registry elements.
    pub fn built_in() -> Self {
        Self {
            known_pack: None,
            lifecycle: Lifecycle::Stable,
        }
    }
}

/// One registered element. Data-pack elements keep the JSON they were decoded
/// from so they can be re-encoded with the registry's network codec when they are
/// synchronised; static elements carry `Value::Null`.
#[derive(Debug, Clone)]
pub struct RegistryElement {
    /// The element's identifier within its registry.
    pub key: Identifier,
    /// The validated source JSON.
    pub json: Value,
    /// Registration metadata.
    pub info: RegistrationInfo,
}

/// `MappedRegistry`: elements in registration order, plus bound tags.
#[derive(Debug, Clone)]
pub struct MappedRegistry {
    key: Identifier,
    elements: Vec<RegistryElement>,
    by_key: HashMap<Identifier, usize>,
    tags: BTreeMap<Identifier, Vec<usize>>,
    frozen: bool,
}

impl MappedRegistry {
    /// Creates an empty, unfrozen registry.
    pub fn new(key: Identifier) -> Self {
        Self {
            key,
            elements: Vec::new(),
            by_key: HashMap::new(),
            tags: BTreeMap::new(),
            frozen: false,
        }
    }

    /// The registry key, e.g. `minecraft:worldgen/biome`.
    pub fn key(&self) -> &Identifier {
        &self.key
    }

    /// `WritableRegistry.register`; ids are assigned in registration order.
    pub fn register(
        &mut self,
        key: Identifier,
        json: Value,
        info: RegistrationInfo,
    ) -> Result<usize, String> {
        if self.frozen {
            return Err(format!(
                "Registry is already frozen (trying to add key {key})"
            ));
        }
        if self.by_key.contains_key(&key) {
            return Err(format!("Adding duplicate key '{key}' to registry"));
        }
        let id = self.elements.len();
        self.by_key.insert(key.clone(), id);
        self.elements.push(RegistryElement { key, json, info });
        Ok(id)
    }

    /// `WritableRegistry.bindTags`: binds a tag to already-registered elements.
    pub fn bind_tag(&mut self, tag: Identifier, elements: Vec<usize>) {
        self.tags.insert(tag, elements);
    }

    /// `MappedRegistry.prepareTagReload` + `PendingTags.apply`: replaces the whole tag
    /// set of a frozen registry (tags absent from `tags` disappear).
    pub fn replace_tags(&mut self, tags: BTreeMap<Identifier, Vec<usize>>) {
        self.tags = tags;
    }

    /// `Registry.freeze`.
    pub fn freeze(&mut self) {
        self.frozen = true;
    }

    /// `Registry.getId(key)`.
    pub fn id_of(&self, key: &Identifier) -> Option<usize> {
        self.by_key.get(key).copied()
    }

    /// Whether `key` is registered.
    pub fn contains(&self, key: &Identifier) -> bool {
        self.by_key.contains_key(key)
    }

    /// `Registry.listElements()` in id order.
    pub fn elements(&self) -> &[RegistryElement] {
        &self.elements
    }

    /// `Registry.size()`.
    pub fn len(&self) -> usize {
        self.elements.len()
    }

    /// `Registry.isEmpty()`.
    pub fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }

    /// `Registry.getTags()`: tag id to element ids.
    pub fn tags(&self) -> &BTreeMap<Identifier, Vec<usize>> {
        &self.tags
    }
}

/// `LayeredRegistryAccess<RegistryLayer>` reduced to the two layers that matter
/// for synchronisation: the static registries and the data-driven worldgen ones.
#[derive(Debug, Clone)]
pub struct Registries {
    static_layer: Vec<MappedRegistry>,
    worldgen_layer: Vec<MappedRegistry>,
}

impl Registries {
    /// Assembles the access from its layers.
    pub fn new(static_layer: Vec<MappedRegistry>, worldgen_layer: Vec<MappedRegistry>) -> Self {
        Self {
            static_layer,
            worldgen_layer,
        }
    }

    /// `getLayer(RegistryLayer.STATIC)`.
    pub fn static_layer(&self) -> &[MappedRegistry] {
        &self.static_layer
    }

    /// `getAccessFrom(RegistryLayer.WORLDGEN)`.
    pub fn worldgen_layer(&self) -> &[MappedRegistry] {
        &self.worldgen_layer
    }

    /// Mutable access to both layers, used by the tag reload.
    pub(crate) fn layers_mut(&mut self) -> impl Iterator<Item = &mut MappedRegistry> {
        self.worldgen_layer
            .iter_mut()
            .chain(self.static_layer.iter_mut())
    }

    /// `RegistryAccess.lookup(key)` across both layers.
    pub fn lookup(&self, key: &Identifier) -> Option<&MappedRegistry> {
        self.worldgen_layer
            .iter()
            .chain(self.static_layer.iter())
            .find(|registry| registry.key() == key)
    }
}
