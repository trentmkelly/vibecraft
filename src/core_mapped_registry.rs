use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LifecycleModel {
    Stable,
    Experimental,
}

impl LifecycleModel {
    fn add(self, other: Self) -> Self {
        if matches!(self, Self::Experimental) || matches!(other, Self::Experimental) {
            Self::Experimental
        } else {
            Self::Stable
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RegistrationInfoModel {
    lifecycle: LifecycleModel,
}

impl RegistrationInfoModel {
    fn stable() -> Self {
        Self {
            lifecycle: LifecycleModel::Stable,
        }
    }

    fn experimental() -> Self {
        Self {
            lifecycle: LifecycleModel::Experimental,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct IdentifierModel(String);

impl IdentifierModel {
    fn parse(value: &str) -> Self {
        Self(value.to_string())
    }
}

impl fmt::Display for IdentifierModel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct ResourceKeyModel {
    registry: IdentifierModel,
    location: IdentifierModel,
}

impl ResourceKeyModel {
    fn new(registry: &str, location: &str) -> Self {
        Self {
            registry: IdentifierModel::parse(registry),
            location: IdentifierModel::parse(location),
        }
    }

    fn identifier(&self) -> &IdentifierModel {
        &self.location
    }
}

impl fmt::Display for ResourceKeyModel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} / {}", self.registry, self.location)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct ValueModel {
    identity: usize,
    name: &'static str,
}

impl ValueModel {
    fn new(identity: usize, name: &'static str) -> Self {
        Self { identity, name }
    }
}

impl fmt::Display for ValueModel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}#{}", self.name, self.identity)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct TagKeyModel {
    registry: IdentifierModel,
    location: IdentifierModel,
}

impl TagKeyModel {
    fn new(registry: &IdentifierModel, location: &str) -> Self {
        Self {
            registry: registry.clone(),
            location: IdentifierModel::parse(location),
        }
    }

    fn location(&self) -> &IdentifierModel {
        &self.location
    }
}

impl fmt::Display for TagKeyModel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}#{}", self.registry, self.location)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum HolderKindModel {
    Reference,
    Direct,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct HolderReferenceModel {
    key: ResourceKeyModel,
    value: Option<ValueModel>,
    tags: Vec<TagKeyModel>,
    intrusive: bool,
}

impl HolderReferenceModel {
    fn stand_alone(key: ResourceKeyModel) -> Self {
        Self {
            key,
            value: None,
            tags: Vec::new(),
            intrusive: false,
        }
    }

    fn intrusive(value: ValueModel, registry_key: &IdentifierModel) -> Self {
        Self {
            key: ResourceKeyModel {
                registry: registry_key.clone(),
                location: IdentifierModel::parse("<intrusive>"),
            },
            value: Some(value),
            tags: Vec::new(),
            intrusive: true,
        }
    }

    fn bind_key(&mut self, key: ResourceKeyModel) {
        self.key = key;
    }

    fn bind_value(&mut self, value: ValueModel) {
        self.value = Some(value);
    }

    fn bind_tags(&mut self, tags: Vec<TagKeyModel>) {
        self.tags = tags;
    }

    fn is_bound(&self) -> bool {
        self.value.is_some()
    }

    fn value(&self) -> Result<&ValueModel, String> {
        self.value
            .as_ref()
            .ok_or_else(|| format!("Trying to access unbound value '{}'", self.key))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum HolderModel {
    Reference(usize),
    Direct(ValueModel),
}

impl HolderModel {
    fn kind(&self) -> HolderKindModel {
        match self {
            Self::Reference(_) => HolderKindModel::Reference,
            Self::Direct(_) => HolderKindModel::Direct,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct HolderSetNamedModel {
    tag: TagKeyModel,
    holder_ids: Option<Vec<usize>>,
}

impl HolderSetNamedModel {
    fn new(tag: TagKeyModel) -> Self {
        Self {
            tag,
            holder_ids: None,
        }
    }

    fn bind(&mut self, holder_ids: Vec<usize>) {
        self.holder_ids = Some(holder_ids);
    }

    fn is_bound(&self) -> bool {
        self.holder_ids.is_some()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum TagSetModel {
    Unbound,
    Bound(BTreeMap<TagKeyModel, HolderSetNamedModel>),
}

impl TagSetModel {
    fn is_bound(&self) -> bool {
        matches!(self, Self::Bound(_))
    }

    fn get(&self, id: &TagKeyModel) -> Result<Option<&HolderSetNamedModel>, String> {
        match self {
            Self::Unbound => Err(format!("Tags not bound, trying to access {id}")),
            Self::Bound(tags) => Ok(tags.get(id)),
        }
    }

    fn get_tags(&self) -> Result<Vec<HolderSetNamedModel>, String> {
        match self {
            Self::Unbound => Err("Tags not bound".to_string()),
            Self::Bound(tags) => Ok(tags.values().cloned().collect()),
        }
    }

    fn for_each(&self) -> Result<Vec<(TagKeyModel, HolderSetNamedModel)>, String> {
        match self {
            Self::Unbound => Err("Tags not bound".to_string()),
            Self::Bound(tags) => Ok(tags
                .iter()
                .map(|(key, value)| (key.clone(), value.clone()))
                .collect()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DataComponentLookupModel {
    holder_count: usize,
}

#[derive(Debug, Clone)]
struct MappedRegistryModel {
    key: IdentifierModel,
    by_id: Vec<usize>,
    holders: Vec<HolderReferenceModel>,
    to_id: BTreeMap<usize, usize>,
    by_location: BTreeMap<IdentifierModel, usize>,
    by_key: BTreeMap<ResourceKeyModel, usize>,
    by_value: BTreeMap<usize, usize>,
    values_by_identity: BTreeMap<usize, ValueModel>,
    registration_infos: BTreeMap<ResourceKeyModel, RegistrationInfoModel>,
    lifecycle: LifecycleModel,
    frozen_tags: BTreeMap<TagKeyModel, HolderSetNamedModel>,
    all_tags: TagSetModel,
    component_lookup: Option<DataComponentLookupModel>,
    frozen: bool,
    unregistered_intrusive_holders: Option<BTreeMap<usize, usize>>,
}

impl fmt::Display for MappedRegistryModel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Registry[{} ({:?})]", self.key, self.lifecycle)
    }
}

impl MappedRegistryModel {
    fn new(key: &str, lifecycle: LifecycleModel, intrusive_holders: bool) -> Self {
        Self {
            key: IdentifierModel::parse(key),
            by_id: Vec::new(),
            holders: Vec::new(),
            to_id: BTreeMap::new(),
            by_location: BTreeMap::new(),
            by_key: BTreeMap::new(),
            by_value: BTreeMap::new(),
            values_by_identity: BTreeMap::new(),
            registration_infos: BTreeMap::new(),
            lifecycle,
            frozen_tags: BTreeMap::new(),
            all_tags: TagSetModel::Unbound,
            component_lookup: None,
            frozen: false,
            unregistered_intrusive_holders: intrusive_holders.then(BTreeMap::new),
        }
    }

    fn validate_write(&self) -> Result<(), String> {
        if self.frozen {
            Err("Registry is already frozen".to_string())
        } else {
            Ok(())
        }
    }

    fn validate_write_key(&self, key: &ResourceKeyModel) -> Result<(), String> {
        if self.frozen {
            Err(format!(
                "Registry is already frozen (trying to add key {key})"
            ))
        } else {
            Ok(())
        }
    }

    fn register(
        &mut self,
        key: ResourceKeyModel,
        value: ValueModel,
        registration_info: RegistrationInfoModel,
    ) -> Result<usize, String> {
        self.validate_write_key(&key)?;
        if self.by_location.contains_key(key.identifier()) {
            return Err(format!("Adding duplicate key '{key}' to registry"));
        }
        if self.by_value.contains_key(&value.identity) {
            return Err(format!("Adding duplicate value '{value}' to registry"));
        }

        let holder_id = if let Some(intrusive) = &mut self.unregistered_intrusive_holders {
            let holder_id = intrusive
                .remove(&value.identity)
                .ok_or_else(|| format!("Missing intrusive holder for {key}:{value}"))?;
            self.holders[holder_id].bind_key(key.clone());
            holder_id
        } else if let Some(holder_id) = self.by_key.get(&key).copied() {
            holder_id
        } else {
            let holder_id = self.holders.len();
            self.holders
                .push(HolderReferenceModel::stand_alone(key.clone()));
            holder_id
        };

        self.by_key.insert(key.clone(), holder_id);
        self.by_location.insert(key.identifier().clone(), holder_id);
        self.by_value.insert(value.identity, holder_id);
        self.values_by_identity
            .insert(value.identity, value.clone());
        let new_id = self.by_id.len();
        self.by_id.push(holder_id);
        self.to_id.insert(value.identity, new_id);
        self.registration_infos
            .insert(key, registration_info.clone());
        self.lifecycle = self.lifecycle.add(registration_info.lifecycle);
        Ok(holder_id)
    }

    fn key(&self) -> &IdentifierModel {
        &self.key
    }

    fn get_key(&self, value: &ValueModel) -> Option<&IdentifierModel> {
        self.by_value
            .get(&value.identity)
            .map(|holder_id| self.holders[*holder_id].key.identifier())
    }

    fn get_resource_key(&self, value: &ValueModel) -> Option<&ResourceKeyModel> {
        self.by_value
            .get(&value.identity)
            .map(|holder_id| &self.holders[*holder_id].key)
    }

    fn get_id(&self, value: Option<&ValueModel>) -> i32 {
        value
            .and_then(|value| self.to_id.get(&value.identity).copied())
            .map_or(-1, |id| id as i32)
    }

    fn value_from_holder(&self, holder_id: usize) -> Result<&ValueModel, String> {
        self.holders[holder_id].value()
    }

    fn get_value_by_resource_key(
        &self,
        key: &ResourceKeyModel,
    ) -> Result<Option<&ValueModel>, String> {
        self.by_key
            .get(key)
            .map(|holder_id| self.value_from_holder(*holder_id))
            .transpose()
    }

    fn get_value_by_identifier(
        &self,
        key: &IdentifierModel,
    ) -> Result<Option<&ValueModel>, String> {
        self.by_location
            .get(key)
            .map(|holder_id| self.value_from_holder(*holder_id))
            .transpose()
    }

    fn by_id_value(&self, id: i32) -> Result<Option<&ValueModel>, String> {
        if id < 0 || id as usize >= self.by_id.len() {
            Ok(None)
        } else {
            self.value_from_holder(self.by_id[id as usize]).map(Some)
        }
    }

    fn get_holder_by_id(&self, id: i32) -> Option<usize> {
        if id < 0 || id as usize >= self.by_id.len() {
            None
        } else {
            Some(self.by_id[id as usize])
        }
    }

    fn get_holder_by_identifier(&self, id: &IdentifierModel) -> Option<usize> {
        self.by_location.get(id).copied()
    }

    fn get_holder_by_resource_key(&self, id: &ResourceKeyModel) -> Option<usize> {
        self.by_key.get(id).copied()
    }

    fn get_any(&self) -> Option<usize> {
        self.by_id.first().copied()
    }

    fn wrap_as_holder(&self, value: ValueModel) -> HolderModel {
        self.by_value
            .get(&value.identity)
            .copied()
            .map(HolderModel::Reference)
            .unwrap_or(HolderModel::Direct(value))
    }

    fn size(&self) -> usize {
        self.by_key.len()
    }

    fn registration_info(&self, key: &ResourceKeyModel) -> Option<&RegistrationInfoModel> {
        self.registration_infos.get(key)
    }

    fn registry_lifecycle(&self) -> LifecycleModel {
        self.lifecycle
    }

    fn iterator(&self) -> Result<Vec<ValueModel>, String> {
        self.by_id
            .iter()
            .map(|holder_id| self.value_from_holder(*holder_id).cloned())
            .collect()
    }

    fn key_set(&self) -> BTreeSet<IdentifierModel> {
        self.by_location.keys().cloned().collect()
    }

    fn registry_key_set(&self) -> BTreeSet<ResourceKeyModel> {
        self.by_key.keys().cloned().collect()
    }

    fn entry_set(&self) -> Result<BTreeSet<(ResourceKeyModel, ValueModel)>, String> {
        self.by_key
            .iter()
            .map(|(key, holder_id)| {
                self.value_from_holder(*holder_id)
                    .map(|value| (key.clone(), value.clone()))
            })
            .collect()
    }

    fn list_elements(&self) -> Vec<usize> {
        self.by_id.clone()
    }

    fn get_tags(&self) -> Result<Vec<HolderSetNamedModel>, String> {
        self.all_tags.get_tags()
    }

    fn list_tags(&self) -> Result<Vec<HolderSetNamedModel>, String> {
        self.get_tags()
    }

    fn get_or_create_tag_for_registration(&mut self, tag: TagKeyModel) -> HolderSetNamedModel {
        self.frozen_tags
            .entry(tag.clone())
            .or_insert_with(|| HolderSetNamedModel::new(tag))
            .clone()
    }

    fn is_empty(&self) -> bool {
        self.by_key.is_empty()
    }

    fn get_random(&self, random: &mut SequenceRandomModel) -> Option<usize> {
        if self.by_id.is_empty() {
            None
        } else {
            Some(self.by_id[random.next_usize(self.by_id.len())])
        }
    }

    fn contains_identifier(&self, key: &IdentifierModel) -> bool {
        self.by_location.contains_key(key)
    }

    fn contains_resource_key(&self, key: &ResourceKeyModel) -> bool {
        self.by_key.contains_key(key)
    }

    fn component_lookup(&self) -> Result<&DataComponentLookupModel, String> {
        self.component_lookup
            .as_ref()
            .ok_or_else(|| "Registry not frozen yet".to_string())
    }

    fn freeze(&mut self) -> Result<&mut Self, String> {
        if self.frozen {
            return Ok(self);
        }

        self.frozen = true;
        for (value_identity, holder_id) in self.by_value.clone() {
            let value = self.holders[holder_id]
                .value
                .clone()
                .or_else(|| self.value_by_identity(value_identity));
            if let Some(value) = value {
                self.holders[holder_id].bind_value(value);
            }
        }

        let unbound_entries = self
            .by_key
            .iter()
            .filter(|(_, holder_id)| !self.holders[**holder_id].is_bound())
            .map(|(key, _)| key.identifier().clone())
            .collect::<Vec<_>>();
        if !unbound_entries.is_empty() {
            return Err(format!(
                "Unbound values in registry {}: {:?}",
                self.key, unbound_entries
            ));
        }

        if let Some(intrusive) = &self.unregistered_intrusive_holders {
            if !intrusive.is_empty() {
                let holders = intrusive
                    .values()
                    .map(|holder_id| format!("{:?}", self.holders[*holder_id]))
                    .collect::<Vec<_>>();
                return Err(format!(
                    "Some intrusive holders were not registered: {holders:?}"
                ));
            }
        }
        self.unregistered_intrusive_holders = None;

        if self.all_tags.is_bound() {
            return Err("Tags already present before freezing".to_string());
        }

        let unbound_tags = self
            .frozen_tags
            .iter()
            .filter(|(_, tag)| !tag.is_bound())
            .map(|(key, _)| key.location().clone())
            .collect::<Vec<_>>();
        if !unbound_tags.is_empty() {
            return Err(format!(
                "Unbound tags in registry {}: {:?}",
                self.key, unbound_tags
            ));
        }

        self.component_lookup = Some(DataComponentLookupModel {
            holder_count: self.by_id.len(),
        });
        self.all_tags = TagSetModel::Bound(self.frozen_tags.clone());
        self.refresh_tags_in_holders()?;
        Ok(self)
    }

    fn value_by_identity(&self, identity: usize) -> Option<ValueModel> {
        self.values_by_identity.get(&identity).cloned()
    }

    fn create_intrusive_holder(&mut self, value: ValueModel) -> Result<usize, String> {
        self.unregistered_intrusive_holders
            .as_ref()
            .ok_or_else(|| "This registry can't create intrusive holders".to_string())?;
        self.validate_write()?;
        if let Some(existing) = self
            .unregistered_intrusive_holders
            .as_ref()
            .and_then(|holders| holders.get(&value.identity).copied())
        {
            return Ok(existing);
        }
        let holder_id = self.holders.len();
        self.holders
            .push(HolderReferenceModel::intrusive(value.clone(), &self.key));
        self.unregistered_intrusive_holders
            .as_mut()
            .expect("checked above")
            .insert(value.identity, holder_id);
        Ok(holder_id)
    }

    fn get_tag(&self, id: &TagKeyModel) -> Result<Option<&HolderSetNamedModel>, String> {
        self.all_tags.get(id)
    }

    fn validate_and_unwrap_tag_element(
        &self,
        id: &TagKeyModel,
        value: &HolderModel,
    ) -> Result<usize, String> {
        match value {
            HolderModel::Reference(holder_id) if self.by_id.contains(holder_id) => Ok(*holder_id),
            HolderModel::Reference(holder_id) => Err(format!(
                "Can't create named set {id} containing value {:?} from outside registry {self}",
                self.holders.get(*holder_id)
            )),
            HolderModel::Direct(value) => Err(format!(
                "Found direct holder Direct{{{value}}} value in tag {id}"
            )),
        }
    }

    fn bind_tags(
        &mut self,
        pending_tags: BTreeMap<TagKeyModel, Vec<HolderModel>>,
    ) -> Result<(), String> {
        self.validate_write()?;
        for (id, values) in pending_tags {
            let holder_ids = values
                .iter()
                .map(|value| self.validate_and_unwrap_tag_element(&id, value))
                .collect::<Result<Vec<_>, _>>()?;
            self.get_or_create_tag_for_registration(id.clone());
            self.frozen_tags
                .get_mut(&id)
                .expect("created above")
                .bind(holder_ids);
        }
        Ok(())
    }

    fn refresh_tags_in_holders(&mut self) -> Result<(), String> {
        let mut tags_for_element: BTreeMap<usize, Vec<TagKeyModel>> = self
            .by_key
            .values()
            .map(|holder_id| (*holder_id, Vec::new()))
            .collect();
        for (id, tag) in self.all_tags.for_each()? {
            for holder_id in tag.holder_ids.unwrap_or_default() {
                tags_for_element
                    .entry(holder_id)
                    .or_default()
                    .push(id.clone());
            }
        }
        for (holder_id, tags) in tags_for_element {
            self.holders[holder_id].bind_tags(tags);
        }
        Ok(())
    }

    fn bind_all_tags_to_empty(&mut self) -> Result<(), String> {
        self.validate_write()?;
        for tag in self.frozen_tags.values_mut() {
            tag.bind(Vec::new());
        }
        Ok(())
    }

    fn create_registration_lookup(&mut self) -> Result<RegistrationLookupModel<'_>, String> {
        self.validate_write()?;
        Ok(RegistrationLookupModel { registry: self })
    }

    fn prepare_tag_reload(
        &self,
        tags: BTreeMap<TagKeyModel, Vec<HolderModel>>,
    ) -> Result<PendingTagsModel, String> {
        if !self.frozen {
            return Err("Invalid method used for tag loading".to_string());
        }
        let pending_tags = tags
            .into_iter()
            .map(|(id, values)| {
                let holder_ids = values
                    .iter()
                    .map(|value| self.validate_and_unwrap_tag_element(&id, value))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok((
                    id.clone(),
                    HolderSetNamedModel {
                        tag: id,
                        holder_ids: Some(holder_ids),
                    },
                ))
            })
            .collect::<Result<BTreeMap<_, _>, String>>()?;
        Ok(PendingTagsModel {
            key: self.key.clone(),
            pending_tags,
            applied: false,
        })
    }
}

#[derive(Debug)]
struct RegistrationLookupModel<'a> {
    registry: &'a mut MappedRegistryModel,
}

impl RegistrationLookupModel<'_> {
    fn get_or_throw(&mut self, id: ResourceKeyModel) -> Result<usize, String> {
        if let Some(holder_id) = self.registry.by_key.get(&id).copied() {
            return Ok(holder_id);
        }
        if self.registry.unregistered_intrusive_holders.is_some() {
            return Err("This registry can't create new holders without value".to_string());
        }
        self.registry.validate_write_key(&id)?;
        let holder_id = self.registry.holders.len();
        self.registry
            .holders
            .push(HolderReferenceModel::stand_alone(id.clone()));
        self.registry.by_key.insert(id, holder_id);
        Ok(holder_id)
    }

    fn get_tag_or_throw(&mut self, id: TagKeyModel) -> HolderSetNamedModel {
        self.registry.get_or_create_tag_for_registration(id)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PendingTagsModel {
    key: IdentifierModel,
    pending_tags: BTreeMap<TagKeyModel, HolderSetNamedModel>,
    applied: bool,
}

impl PendingTagsModel {
    fn key(&self) -> &IdentifierModel {
        &self.key
    }

    fn size(&self) -> usize {
        self.pending_tags.len()
    }

    fn lookup(&self, id: &TagKeyModel) -> Option<&HolderSetNamedModel> {
        self.pending_tags.get(id)
    }

    fn apply(&mut self, registry: &mut MappedRegistryModel) -> Result<(), String> {
        registry.all_tags = TagSetModel::Bound(self.pending_tags.clone());
        registry.refresh_tags_in_holders()?;
        self.applied = true;
        Ok(())
    }
}

#[derive(Debug, Clone)]
struct SequenceRandomModel {
    values: Vec<usize>,
    index: usize,
}

impl SequenceRandomModel {
    fn new(values: impl Into<Vec<usize>>) -> Self {
        Self {
            values: values.into(),
            index: 0,
        }
    }

    fn next_usize(&mut self, bound: usize) -> usize {
        let value = self.values[self.index % self.values.len()] % bound;
        self.index += 1;
        value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const REGISTRY: &str = "minecraft:test_registry";

    fn key(location: &str) -> ResourceKeyModel {
        ResourceKeyModel::new(REGISTRY, location)
    }

    fn tag(location: &str) -> TagKeyModel {
        TagKeyModel::new(&IdentifierModel::parse(REGISTRY), location)
    }

    fn value(identity: usize, name: &'static str) -> ValueModel {
        ValueModel::new(identity, name)
    }

    fn registered_registry() -> MappedRegistryModel {
        let mut registry = MappedRegistryModel::new(REGISTRY, LifecycleModel::Stable, false);
        registry
            .register(
                key("minecraft:stone"),
                value(1, "stone"),
                RegistrationInfoModel::stable(),
            )
            .unwrap();
        registry
            .register(
                key("minecraft:dirt"),
                value(2, "dirt"),
                RegistrationInfoModel::experimental(),
            )
            .unwrap();
        registry
    }

    #[test]
    fn construction_registration_identity_maps_and_unbound_value_surface_match_java() {
        let registry = registered_registry();
        assert_eq!(registry.key().to_string(), REGISTRY);
        assert_eq!(
            registry.to_string(),
            "Registry[minecraft:test_registry (Experimental)]"
        );
        assert_eq!(registry.size(), 2);
        assert!(!registry.is_empty());
        assert_eq!(registry.registry_lifecycle(), LifecycleModel::Experimental);
        assert_eq!(
            registry.registration_info(&key("minecraft:dirt")).unwrap(),
            &RegistrationInfoModel::experimental()
        );
        assert_eq!(registry.get_id(Some(&value(1, "stone"))), 0);
        assert_eq!(registry.get_id(Some(&value(9, "missing"))), -1);
        assert_eq!(
            registry.get_key(&value(1, "stone")).unwrap().to_string(),
            "minecraft:stone"
        );
        assert_eq!(
            registry.get_resource_key(&value(2, "dirt")).unwrap(),
            &key("minecraft:dirt")
        );
        assert_eq!(registry.get_any(), Some(0));
        assert_eq!(registry.list_elements(), vec![0, 1]);
        assert_eq!(registry.get_holder_by_id(1), Some(1));
        assert_eq!(
            registry.get_holder_by_identifier(&IdentifierModel::parse("minecraft:dirt")),
            Some(1)
        );
        assert_eq!(
            registry.get_holder_by_resource_key(&key("minecraft:stone")),
            Some(0)
        );
        assert_eq!(
            registry
                .get_value_by_resource_key(&key("minecraft:stone"))
                .unwrap_err(),
            "Trying to access unbound value 'minecraft:test_registry / minecraft:stone'"
        );
        assert_eq!(
            registry.by_id_value(99).unwrap(),
            None,
            "out-of-range ids return null before touching holder values"
        );
    }

    #[test]
    fn freeze_binds_values_component_lookup_tags_and_iteration_surfaces() {
        let mut registry = registered_registry();
        assert_eq!(
            registry.component_lookup().unwrap_err(),
            "Registry not frozen yet"
        );
        assert_eq!(registry.get_tags().unwrap_err(), "Tags not bound");
        let pickaxe = tag("minecraft:mineable/pickaxe");
        registry
            .bind_tags(BTreeMap::from([(
                pickaxe.clone(),
                vec![HolderModel::Reference(0), HolderModel::Reference(1)],
            )]))
            .unwrap();
        registry.freeze().unwrap();
        assert_eq!(
            registry
                .get_value_by_identifier(&IdentifierModel::parse("minecraft:stone"))
                .unwrap(),
            Some(&value(1, "stone"))
        );
        assert_eq!(
            registry.iterator().unwrap(),
            vec![value(1, "stone"), value(2, "dirt")]
        );
        assert_eq!(
            registry.entry_set().unwrap(),
            BTreeSet::from([
                (key("minecraft:dirt"), value(2, "dirt")),
                (key("minecraft:stone"), value(1, "stone")),
            ])
        );
        assert_eq!(registry.component_lookup().unwrap().holder_count, 2);
        assert_eq!(registry.list_tags().unwrap().len(), 1);
        assert_eq!(
            registry
                .get_tag(&pickaxe)
                .unwrap()
                .unwrap()
                .holder_ids
                .as_ref()
                .unwrap(),
            &vec![0, 1]
        );
        assert_eq!(registry.holders[0].tags, vec![pickaxe.clone()]);
        assert_eq!(
            registry.freeze().unwrap().size(),
            2,
            "freezing twice returns this"
        );
    }

    #[test]
    fn duplicate_frozen_random_contains_wrap_and_sets_match_java_mapped_registry() {
        let mut registry = registered_registry();
        assert_eq!(
            registry
                .register(
                    key("minecraft:stone"),
                    value(3, "duplicate-key"),
                    RegistrationInfoModel::stable(),
                )
                .unwrap_err(),
            "Adding duplicate key 'minecraft:test_registry / minecraft:stone' to registry"
        );
        assert_eq!(
            registry
                .register(
                    key("minecraft:new"),
                    value(1, "duplicate-value"),
                    RegistrationInfoModel::stable(),
                )
                .unwrap_err(),
            "Adding duplicate value 'duplicate-value#1' to registry"
        );
        assert_eq!(
            registry.key_set(),
            BTreeSet::from([
                IdentifierModel::parse("minecraft:dirt"),
                IdentifierModel::parse("minecraft:stone"),
            ])
        );
        assert_eq!(
            registry.registry_key_set(),
            BTreeSet::from([key("minecraft:dirt"), key("minecraft:stone"),])
        );
        assert!(registry.contains_identifier(&IdentifierModel::parse("minecraft:stone")));
        assert!(registry.contains_resource_key(&key("minecraft:dirt")));
        assert_eq!(
            registry.wrap_as_holder(value(1, "stone")),
            HolderModel::Reference(0)
        );
        assert_eq!(
            registry.wrap_as_holder(value(9, "loose")).kind(),
            HolderKindModel::Direct
        );
        assert_eq!(
            registry.get_random(&mut SequenceRandomModel::new([1])),
            Some(1)
        );
        registry.freeze().unwrap();
        assert_eq!(
            registry
                .register(
                    key("minecraft:late"),
                    value(9, "late"),
                    RegistrationInfoModel::stable(),
                )
                .unwrap_err(),
            "Registry is already frozen (trying to add key minecraft:test_registry / minecraft:late)"
        );
    }

    #[test]
    fn intrusive_holder_path_and_registration_lookup_match_java_write_guards() {
        let mut intrusive = MappedRegistryModel::new(REGISTRY, LifecycleModel::Stable, true);
        assert_eq!(
            intrusive
                .register(
                    key("minecraft:stone"),
                    value(1, "stone"),
                    RegistrationInfoModel::stable(),
                )
                .unwrap_err(),
            "Missing intrusive holder for minecraft:test_registry / minecraft:stone:stone#1"
        );
        let holder_id = intrusive
            .create_intrusive_holder(value(1, "stone"))
            .unwrap();
        assert_eq!(
            intrusive
                .create_intrusive_holder(value(1, "stone"))
                .unwrap(),
            holder_id
        );
        intrusive
            .register(
                key("minecraft:stone"),
                value(1, "stone"),
                RegistrationInfoModel::stable(),
            )
            .unwrap();
        assert!(intrusive.holders[holder_id].intrusive);
        assert_eq!(intrusive.holders[holder_id].key, key("minecraft:stone"));
        assert_eq!(
            intrusive
                .create_registration_lookup()
                .unwrap()
                .get_or_throw(key("minecraft:new"))
                .unwrap_err(),
            "This registry can't create new holders without value"
        );

        let mut standalone = MappedRegistryModel::new(REGISTRY, LifecycleModel::Stable, false);
        let mut lookup = standalone.create_registration_lookup().unwrap();
        let future_holder = lookup.get_or_throw(key("minecraft:future")).unwrap();
        assert_eq!(future_holder, 0);
        assert_eq!(
            lookup.get_tag_or_throw(tag("minecraft:test")).tag,
            tag("minecraft:test")
        );
        standalone
            .register(
                key("minecraft:future"),
                value(2, "future"),
                RegistrationInfoModel::stable(),
            )
            .unwrap();
        assert_eq!(
            standalone.get_holder_by_resource_key(&key("minecraft:future")),
            Some(0)
        );
    }

    #[test]
    fn tag_errors_bind_all_empty_prepare_reload_and_pending_apply_match_java() {
        let mut registry = registered_registry();
        let pickaxe = tag("minecraft:mineable/pickaxe");
        assert_eq!(
            registry.get_tag(&pickaxe).unwrap_err(),
            "Tags not bound, trying to access minecraft:test_registry#minecraft:mineable/pickaxe"
        );
        assert_eq!(
            registry
                .bind_tags(BTreeMap::from([(pickaxe.clone(), vec![HolderModel::Direct(value(9, "direct"))])]))
                .unwrap_err(),
            "Found direct holder Direct{direct#9} value in tag minecraft:test_registry#minecraft:mineable/pickaxe"
        );
        registry.get_or_create_tag_for_registration(pickaxe.clone());
        assert_eq!(
            registry.freeze().unwrap_err(),
            "Unbound tags in registry minecraft:test_registry: [IdentifierModel(\"minecraft:mineable/pickaxe\")]"
        );

        let mut registry = registered_registry();
        registry.get_or_create_tag_for_registration(pickaxe.clone());
        registry.bind_all_tags_to_empty().unwrap();
        registry.freeze().unwrap();
        assert_eq!(
            registry
                .prepare_tag_reload(BTreeMap::from([(
                    pickaxe.clone(),
                    vec![HolderModel::Reference(0)]
                )]))
                .unwrap()
                .size(),
            1
        );
        let mut pending = registry
            .prepare_tag_reload(BTreeMap::from([(
                pickaxe.clone(),
                vec![HolderModel::Reference(0)],
            )]))
            .unwrap();
        assert_eq!(pending.key().to_string(), REGISTRY);
        assert_eq!(
            pending
                .lookup(&pickaxe)
                .unwrap()
                .holder_ids
                .as_ref()
                .unwrap(),
            &vec![0]
        );
        pending.apply(&mut registry).unwrap();
        assert!(pending.applied);
        assert_eq!(registry.holders[0].tags, vec![pickaxe]);

        let unfrozen = registered_registry();
        assert_eq!(
            unfrozen.prepare_tag_reload(BTreeMap::new()).unwrap_err(),
            "Invalid method used for tag loading"
        );
    }

    #[test]
    fn freeze_detects_unbound_registration_lookup_values_and_intrusive_holders() {
        let mut registry = MappedRegistryModel::new(REGISTRY, LifecycleModel::Stable, false);
        registry
            .create_registration_lookup()
            .unwrap()
            .get_or_throw(key("minecraft:future"))
            .unwrap();
        assert_eq!(
            registry.freeze().unwrap_err(),
            "Unbound values in registry minecraft:test_registry: [IdentifierModel(\"minecraft:future\")]"
        );

        let mut intrusive = MappedRegistryModel::new(REGISTRY, LifecycleModel::Stable, true);
        intrusive
            .create_intrusive_holder(value(1, "stone"))
            .unwrap();
        assert!(intrusive
            .freeze()
            .unwrap_err()
            .starts_with("Some intrusive holders were not registered:"));
    }
}
