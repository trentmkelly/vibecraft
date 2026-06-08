use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LifecycleModel {
    Stable,
    Experimental,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RegistrationInfoModel {
    lifecycle: LifecycleModel,
    known_pack_info: Option<&'static str>,
}

impl RegistrationInfoModel {
    const BUILT_IN: Self = Self {
        lifecycle: LifecycleModel::Stable,
        known_pack_info: None,
    };

    fn experimental() -> Self {
        Self {
            lifecycle: LifecycleModel::Experimental,
            known_pack_info: None,
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
    fn create(registry: &IdentifierModel, location: IdentifierModel) -> Self {
        Self {
            registry: registry.clone(),
            location,
        }
    }

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

#[derive(Debug, Clone, PartialEq, Eq)]
enum HolderModel {
    Reference {
        key: ResourceKeyModel,
        value: ValueModel,
    },
    Direct(ValueModel),
}

impl HolderModel {
    fn value(&self) -> &ValueModel {
        match self {
            Self::Reference { value, .. } | Self::Direct(value) => value,
        }
    }

    fn key(&self) -> Option<&ResourceKeyModel> {
        match self {
            Self::Reference { key, .. } => Some(key),
            Self::Direct(_) => None,
        }
    }

    fn kind(&self) -> HolderKindModel {
        match self {
            Self::Reference { .. } => HolderKindModel::Reference,
            Self::Direct(_) => HolderKindModel::Direct,
        }
    }
}

impl fmt::Display for HolderModel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Reference { key, value } => write!(f, "Reference{{{key}={value}}}"),
            Self::Direct(value) => write!(f, "Direct{{{value}}}"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum HolderKindModel {
    Reference,
    Direct,
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
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct HolderSetNamedModel {
    tag: TagKeyModel,
    values: Vec<HolderModel>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PendingTagsModel {
    key: IdentifierModel,
    lookup_tags: BTreeMap<TagKeyModel, HolderSetNamedModel>,
    applied: bool,
}

impl PendingTagsModel {
    fn key(&self) -> &IdentifierModel {
        &self.key
    }

    fn lookup(&self, tag: &TagKeyModel) -> Option<&HolderSetNamedModel> {
        self.lookup_tags.get(tag)
    }

    fn size(&self) -> usize {
        self.lookup_tags.len()
    }

    fn apply(&mut self) {
        self.applied = true;
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DataComponentLookupModel {
    holder_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct HolderByNameCodecModel {
    registry_key: IdentifierModel,
    value_only: bool,
}

impl HolderByNameCodecModel {
    fn decode_reference(
        &self,
        registry: &WritableRegistryModel,
        name: &str,
    ) -> Result<(HolderModel, LifecycleModel), String> {
        let id = IdentifierModel::parse(name);
        let holder = registry
            .get_by_identifier(&id)
            .ok_or_else(|| format!("Unknown registry key in {}: {id}", registry.key()))?;
        let lifecycle = registry
            .registration_info(holder.key().expect("reference holder has key"))
            .map(|info| info.lifecycle)
            .unwrap_or(LifecycleModel::Experimental);
        Ok((holder.clone(), lifecycle))
    }

    fn encode_holder(
        &self,
        registry: &WritableRegistryModel,
        holder: &HolderModel,
    ) -> Result<String, String> {
        match holder.key() {
            Some(key) => {
                let _ = registry;
                Ok(key.identifier().to_string())
            }
            None => Err(format!(
                "Unregistered holder in {}: {holder}",
                registry.key()
            )),
        }
    }

    fn encode_value(
        &self,
        registry: &WritableRegistryModel,
        value: &ValueModel,
    ) -> Result<String, String> {
        self.encode_holder(registry, &registry.wrap_as_holder(value.clone()))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct WritableRegistryModel {
    key: IdentifierModel,
    by_id: Vec<HolderModel>,
    by_location: BTreeMap<IdentifierModel, usize>,
    by_key: BTreeMap<ResourceKeyModel, usize>,
    by_value_identity: BTreeMap<usize, usize>,
    registration_infos: BTreeMap<ResourceKeyModel, RegistrationInfoModel>,
    tags: BTreeMap<TagKeyModel, HolderSetNamedModel>,
    frozen: bool,
    intrusive_holders: BTreeMap<usize, HolderModel>,
}

impl WritableRegistryModel {
    fn new(registry_key: &str) -> Self {
        Self {
            key: IdentifierModel::parse(registry_key),
            by_id: Vec::new(),
            by_location: BTreeMap::new(),
            by_key: BTreeMap::new(),
            by_value_identity: BTreeMap::new(),
            registration_infos: BTreeMap::new(),
            tags: BTreeMap::new(),
            frozen: false,
            intrusive_holders: BTreeMap::new(),
        }
    }

    fn key(&self) -> &IdentifierModel {
        &self.key
    }

    fn by_name_codec(&self) -> HolderByNameCodecModel {
        HolderByNameCodecModel {
            registry_key: self.key.clone(),
            value_only: true,
        }
    }

    fn holder_by_name_codec(&self) -> HolderByNameCodecModel {
        HolderByNameCodecModel {
            registry_key: self.key.clone(),
            value_only: false,
        }
    }

    fn keys(&self) -> Vec<String> {
        self.key_set()
            .into_iter()
            .map(|key| key.to_string())
            .collect()
    }

    fn get_key(&self, value: &ValueModel) -> Option<IdentifierModel> {
        self.by_value_identity.get(&value.identity).map(|index| {
            self.by_id[*index]
                .key()
                .expect("registered holder")
                .identifier()
                .clone()
        })
    }

    fn get_resource_key(&self, value: &ValueModel) -> Option<ResourceKeyModel> {
        self.by_value_identity
            .get(&value.identity)
            .map(|index| self.by_id[*index].key().expect("registered holder").clone())
    }

    fn get_id(&self, value: Option<&ValueModel>) -> i32 {
        value
            .and_then(|value| self.by_value_identity.get(&value.identity).copied())
            .map_or(-1, |index| index as i32)
    }

    fn get_value_by_key(&self, key: &ResourceKeyModel) -> Option<&ValueModel> {
        self.by_key.get(key).map(|index| self.by_id[*index].value())
    }

    fn get_value_by_identifier(&self, key: &IdentifierModel) -> Option<&ValueModel> {
        self.by_location
            .get(key)
            .map(|index| self.by_id[*index].value())
    }

    fn get_optional_identifier(&self, key: Option<&IdentifierModel>) -> Option<&ValueModel> {
        key.and_then(|key| self.get_value_by_identifier(key))
    }

    fn get_optional_resource_key(&self, key: Option<&ResourceKeyModel>) -> Option<&ValueModel> {
        key.and_then(|key| self.get_value_by_key(key))
    }

    fn get_any(&self) -> Option<&HolderModel> {
        self.by_id.first()
    }

    fn get_value_or_throw(&self, key: &ResourceKeyModel) -> Result<&ValueModel, String> {
        self.get_value_by_key(key)
            .ok_or_else(|| format!("Missing key in {}: {key}", self.key()))
    }

    fn key_set(&self) -> BTreeSet<IdentifierModel> {
        self.by_location.keys().cloned().collect()
    }

    fn entry_set(&self) -> BTreeSet<(ResourceKeyModel, ValueModel)> {
        self.by_key
            .iter()
            .map(|(key, index)| (key.clone(), self.by_id[*index].value().clone()))
            .collect()
    }

    fn registry_key_set(&self) -> BTreeSet<ResourceKeyModel> {
        self.by_key.keys().cloned().collect()
    }

    fn get_random(&self, random: &mut SequenceRandomModel) -> Option<&HolderModel> {
        if self.by_id.is_empty() {
            None
        } else {
            self.by_id.get(random.next_usize(self.by_id.len()))
        }
    }

    fn stream(&self) -> Vec<&ValueModel> {
        self.by_id.iter().map(HolderModel::value).collect()
    }

    fn contains_identifier(&self, key: &IdentifierModel) -> bool {
        self.by_location.contains_key(key)
    }

    fn contains_resource_key(&self, key: &ResourceKeyModel) -> bool {
        self.by_key.contains_key(key)
    }

    fn register(
        &mut self,
        key: ResourceKeyModel,
        value: ValueModel,
        registration_info: RegistrationInfoModel,
    ) -> Result<HolderModel, String> {
        if self.frozen {
            return Err(format!(
                "Registry is already frozen (trying to add key {key})"
            ));
        }
        if self.by_location.contains_key(key.identifier()) {
            return Err(format!("Adding duplicate key '{key}' to registry"));
        }
        if self.by_value_identity.contains_key(&value.identity) {
            return Err(format!("Adding duplicate value '{value}' to registry"));
        }
        let holder = HolderModel::Reference {
            key: key.clone(),
            value: value.clone(),
        };
        let index = self.by_id.len();
        self.by_id.push(holder.clone());
        self.by_location.insert(key.identifier().clone(), index);
        self.by_key.insert(key.clone(), index);
        self.by_value_identity.insert(value.identity, index);
        self.registration_infos.insert(key, registration_info);
        Ok(holder)
    }

    fn register_from_string(
        &mut self,
        name: &str,
        value: ValueModel,
    ) -> Result<ValueModel, String> {
        self.register_from_identifier(IdentifierModel::parse(name), value)
    }

    fn register_from_identifier(
        &mut self,
        location: IdentifierModel,
        value: ValueModel,
    ) -> Result<ValueModel, String> {
        let key = ResourceKeyModel::create(&self.key, location);
        self.register_from_resource_key(key, value)
    }

    fn register_from_resource_key(
        &mut self,
        key: ResourceKeyModel,
        value: ValueModel,
    ) -> Result<ValueModel, String> {
        self.register(key, value.clone(), RegistrationInfoModel::BUILT_IN)?;
        Ok(value)
    }

    fn register_for_holder_from_identifier(
        &mut self,
        location: IdentifierModel,
        value: ValueModel,
    ) -> Result<HolderModel, String> {
        let key = ResourceKeyModel::create(&self.key, location);
        self.register_for_holder_from_resource_key(key, value)
    }

    fn register_for_holder_from_resource_key(
        &mut self,
        key: ResourceKeyModel,
        value: ValueModel,
    ) -> Result<HolderModel, String> {
        self.register(key, value, RegistrationInfoModel::BUILT_IN)
    }

    fn registration_info(&self, key: &ResourceKeyModel) -> Option<&RegistrationInfoModel> {
        self.registration_infos.get(key)
    }

    fn freeze(&mut self) -> &mut Self {
        self.frozen = true;
        self
    }

    fn create_intrusive_holder(&mut self, value: ValueModel) -> HolderModel {
        let holder = HolderModel::Reference {
            key: ResourceKeyModel::create(
                &self.key,
                IdentifierModel::parse(&format!("intrusive/{}", value.identity)),
            ),
            value: value.clone(),
        };
        self.intrusive_holders
            .insert(value.identity, holder.clone());
        holder
    }

    fn get_by_id(&self, id: i32) -> Option<&HolderModel> {
        if id < 0 {
            None
        } else {
            self.by_id.get(id as usize)
        }
    }

    fn get_by_identifier(&self, id: &IdentifierModel) -> Option<&HolderModel> {
        self.by_location.get(id).map(|index| &self.by_id[*index])
    }

    fn wrap_as_holder(&self, value: ValueModel) -> HolderModel {
        self.by_value_identity
            .get(&value.identity)
            .map(|index| self.by_id[*index].clone())
            .unwrap_or(HolderModel::Direct(value))
    }

    fn bind_tags(&mut self, pending_tags: BTreeMap<TagKeyModel, Vec<HolderModel>>) {
        for (tag, values) in pending_tags {
            self.tags
                .insert(tag.clone(), HolderSetNamedModel { tag, values });
        }
    }

    fn get_tag_or_empty(&self, id: &TagKeyModel) -> Vec<HolderModel> {
        self.tags
            .get(id)
            .map(|tag| tag.values.clone())
            .unwrap_or_default()
    }

    fn get_tags(&self) -> Vec<HolderSetNamedModel> {
        self.tags.values().cloned().collect()
    }

    fn is_empty(&self) -> bool {
        self.by_id.is_empty()
    }

    fn create_registration_lookup(&mut self) -> RegistrationLookupModel<'_> {
        RegistrationLookupModel { registry: self }
    }

    fn as_holder_id_map(&self) -> HolderIdMapModel<'_> {
        HolderIdMapModel { registry: self }
    }

    fn prepare_tag_reload(
        &self,
        tags: BTreeMap<TagKeyModel, Vec<HolderModel>>,
    ) -> PendingTagsModel {
        let lookup_tags = tags
            .into_iter()
            .map(|(tag, values)| (tag.clone(), HolderSetNamedModel { tag, values }))
            .collect();
        PendingTagsModel {
            key: self.key.clone(),
            lookup_tags,
            applied: false,
        }
    }

    fn component_lookup(&self) -> DataComponentLookupModel {
        DataComponentLookupModel {
            holder_count: self.by_id.len(),
        }
    }

    fn size(&self) -> usize {
        self.by_id.len()
    }
}

#[derive(Debug)]
struct RegistrationLookupModel<'a> {
    registry: &'a mut WritableRegistryModel,
}

impl RegistrationLookupModel<'_> {
    fn get_or_throw(&mut self, key: ResourceKeyModel) -> HolderModel {
        if let Some(holder) = self.registry.get_by_resource_key_holder(&key) {
            holder.clone()
        } else {
            HolderModel::Reference {
                key,
                value: ValueModel::new(usize::MAX, "unbound"),
            }
        }
    }

    fn get_tag_or_throw(&mut self, tag: TagKeyModel) -> HolderSetNamedModel {
        self.registry
            .tags
            .get(&tag)
            .cloned()
            .unwrap_or(HolderSetNamedModel {
                tag,
                values: Vec::new(),
            })
    }
}

impl WritableRegistryModel {
    fn get_by_resource_key_holder(&self, id: &ResourceKeyModel) -> Option<&HolderModel> {
        self.by_key.get(id).map(|index| &self.by_id[*index])
    }
}

#[derive(Debug, Clone, Copy)]
struct HolderIdMapModel<'a> {
    registry: &'a WritableRegistryModel,
}

impl HolderIdMapModel<'_> {
    fn get_id(&self, holder: &HolderModel) -> i32 {
        self.registry.get_id(Some(holder.value()))
    }

    fn by_id(&self, id: i32) -> Option<HolderModel> {
        self.registry.get_by_id(id).cloned()
    }

    fn size(&self) -> usize {
        self.registry.size()
    }

    fn iterator(&self) -> Vec<HolderModel> {
        self.registry.by_id.clone()
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

    fn value(identity: usize, name: &'static str) -> ValueModel {
        ValueModel::new(identity, name)
    }

    fn populated_registry() -> WritableRegistryModel {
        let mut registry = WritableRegistryModel::new(REGISTRY);
        registry
            .register(
                key("minecraft:stone"),
                value(1, "stone"),
                RegistrationInfoModel::BUILT_IN,
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
    fn registry_lookup_sets_and_optional_defaults_match_java_interface() {
        let registry = populated_registry();
        assert_eq!(registry.key().to_string(), REGISTRY);
        assert_eq!(
            registry.keys(),
            vec!["minecraft:dirt".to_string(), "minecraft:stone".to_string()]
        );
        assert_eq!(
            registry.entry_set(),
            BTreeSet::from([
                (key("minecraft:dirt"), value(2, "dirt")),
                (key("minecraft:stone"), value(1, "stone")),
            ])
        );
        assert_eq!(
            registry.registry_key_set(),
            BTreeSet::from([key("minecraft:dirt"), key("minecraft:stone")])
        );
        assert_eq!(
            registry.get_key(&value(1, "stone")).unwrap().to_string(),
            "minecraft:stone"
        );
        assert_eq!(
            registry.get_resource_key(&value(2, "dirt")).unwrap(),
            key("minecraft:dirt")
        );
        assert_eq!(registry.get_id(Some(&value(1, "stone"))), 0);
        assert_eq!(registry.get_id(Some(&value(99, "missing"))), -1);
        assert_eq!(registry.get_id(None), -1);
        assert_eq!(
            registry.get_optional_identifier(Some(&IdentifierModel::parse("minecraft:stone"))),
            Some(&value(1, "stone"))
        );
        assert_eq!(registry.get_optional_identifier(None), None);
        assert_eq!(
            registry.get_optional_resource_key(Some(&key("minecraft:dirt"))),
            Some(&value(2, "dirt"))
        );
        assert_eq!(registry.get_optional_resource_key(None), None);
        assert_eq!(registry.get_any().unwrap().value(), &value(1, "stone"));
        assert_eq!(
            registry.get_value_or_throw(&key("minecraft:dirt")).unwrap(),
            &value(2, "dirt")
        );
        assert_eq!(
            registry
                .get_value_or_throw(&key("minecraft:missing"))
                .unwrap_err(),
            "Missing key in minecraft:test_registry: minecraft:test_registry / minecraft:missing"
        );
        assert!(registry.contains_identifier(&IdentifierModel::parse("minecraft:stone")));
        assert!(registry.contains_resource_key(&key("minecraft:dirt")));
        assert_eq!(
            registry.stream(),
            vec![&value(1, "stone"), &value(2, "dirt")]
        );
    }

    #[test]
    fn registry_codecs_reference_lookup_lifecycle_and_direct_holder_errors_match_java() {
        let registry = populated_registry();
        let value_codec = registry.by_name_codec();
        let holder_codec = registry.holder_by_name_codec();
        assert!(value_codec.value_only);
        assert!(!holder_codec.value_only);
        assert_eq!(value_codec.registry_key.to_string(), REGISTRY);
        assert_eq!(
            value_codec
                .decode_reference(&registry, "minecraft:stone")
                .unwrap(),
            (
                HolderModel::Reference {
                    key: key("minecraft:stone"),
                    value: value(1, "stone"),
                },
                LifecycleModel::Stable,
            )
        );
        assert_eq!(
            holder_codec
                .decode_reference(&registry, "minecraft:dirt")
                .unwrap()
                .1,
            LifecycleModel::Experimental
        );
        assert_eq!(
            holder_codec
                .decode_reference(&registry, "minecraft:missing")
                .unwrap_err(),
            "Unknown registry key in minecraft:test_registry: minecraft:missing"
        );
        assert_eq!(
            value_codec
                .encode_value(&registry, &value(1, "stone"))
                .unwrap(),
            "minecraft:stone"
        );
        assert_eq!(
            holder_codec
                .encode_holder(&registry, &HolderModel::Direct(value(3, "direct")))
                .unwrap_err(),
            "Unregistered holder in minecraft:test_registry: Direct{direct#3}"
        );
    }

    #[test]
    fn static_registration_helpers_use_built_in_registration_info_and_return_java_values() {
        let mut registry = WritableRegistryModel::new(REGISTRY);
        let returned = registry
            .register_from_string("minecraft:stone", value(1, "stone"))
            .unwrap();
        assert_eq!(returned, value(1, "stone"));
        assert_eq!(
            registry.registration_info(&key("minecraft:stone")).unwrap(),
            &RegistrationInfoModel::BUILT_IN
        );
        let identifier_returned = registry
            .register_from_identifier(IdentifierModel::parse("minecraft:dirt"), value(2, "dirt"))
            .unwrap();
        assert_eq!(identifier_returned, value(2, "dirt"));
        let holder = registry
            .register_for_holder_from_resource_key(key("minecraft:grass"), value(3, "grass"))
            .unwrap();
        assert_eq!(holder.key(), Some(&key("minecraft:grass")));
        let holder_from_id = registry
            .register_for_holder_from_identifier(
                IdentifierModel::parse("minecraft:sand"),
                value(4, "sand"),
            )
            .unwrap();
        assert_eq!(holder_from_id.key(), Some(&key("minecraft:sand")));
    }

    #[test]
    fn writable_registry_methods_bind_tags_empty_registration_lookup_and_freeze() {
        let mut registry = WritableRegistryModel::new(REGISTRY);
        assert!(registry.is_empty());
        registry
            .register_from_string("minecraft:stone", value(1, "stone"))
            .unwrap();
        assert!(!registry.is_empty());
        let tag = TagKeyModel::new(registry.key(), "minecraft:mineable/pickaxe");
        registry.bind_tags(BTreeMap::from([(
            tag.clone(),
            vec![registry.wrap_as_holder(value(1, "stone"))],
        )]));
        assert_eq!(registry.get_tags().len(), 1);
        assert_eq!(registry.get_tag_or_empty(&tag).len(), 1);
        assert!(registry
            .get_tag_or_empty(&TagKeyModel::new(registry.key(), "minecraft:missing"))
            .is_empty());

        let mut lookup = registry.create_registration_lookup();
        assert_eq!(
            lookup.get_or_throw(key("minecraft:stone")).key(),
            Some(&key("minecraft:stone"))
        );
        assert_eq!(
            lookup.get_or_throw(key("minecraft:new")).key(),
            Some(&key("minecraft:new"))
        );
        assert_eq!(lookup.get_tag_or_throw(tag.clone()).tag, tag);

        registry.freeze();
        assert_eq!(
            registry
                .register_from_string("minecraft:late", value(9, "late"))
                .unwrap_err(),
            "Registry is already frozen (trying to add key minecraft:test_registry / minecraft:late)"
        );
    }

    #[test]
    fn holder_accessors_random_wrap_intrusive_component_and_pending_tags_match_java_surface() {
        let mut registry = populated_registry();
        assert_eq!(registry.get_by_id(0).unwrap().value(), &value(1, "stone"));
        assert!(registry.get_by_id(-1).is_none());
        assert_eq!(
            registry
                .get_by_identifier(&IdentifierModel::parse("minecraft:dirt"))
                .unwrap()
                .value(),
            &value(2, "dirt")
        );
        assert_eq!(
            registry.wrap_as_holder(value(1, "stone")).kind(),
            HolderKindModel::Reference
        );
        assert_eq!(
            registry.wrap_as_holder(value(9, "loose")).kind(),
            HolderKindModel::Direct
        );
        assert_eq!(
            registry
                .create_intrusive_holder(value(10, "intrusive"))
                .key()
                .unwrap()
                .identifier()
                .to_string(),
            "intrusive/10"
        );
        let mut random = SequenceRandomModel::new([1]);
        assert_eq!(
            registry.get_random(&mut random).unwrap().value(),
            &value(2, "dirt")
        );
        assert_eq!(
            registry.component_lookup(),
            DataComponentLookupModel { holder_count: 2 }
        );

        let tag = TagKeyModel::new(registry.key(), "minecraft:test");
        let mut pending = registry.prepare_tag_reload(BTreeMap::from([(
            tag.clone(),
            vec![registry.wrap_as_holder(value(1, "stone"))],
        )]));
        assert_eq!(pending.key().to_string(), REGISTRY);
        assert_eq!(pending.size(), 1);
        assert_eq!(pending.lookup(&tag).unwrap().values.len(), 1);
        pending.apply();
        assert!(pending.applied);
    }

    #[test]
    fn as_holder_id_map_delegates_to_registry_ids_by_id_size_and_iteration() {
        let registry = populated_registry();
        let holder_map = registry.as_holder_id_map();
        let stone_holder = registry.wrap_as_holder(value(1, "stone"));
        assert_eq!(holder_map.get_id(&stone_holder), 0);
        assert_eq!(holder_map.by_id(1).unwrap().value(), &value(2, "dirt"));
        assert_eq!(holder_map.by_id(99), None);
        assert_eq!(holder_map.size(), 2);
        assert_eq!(
            holder_map
                .iterator()
                .into_iter()
                .map(|holder| holder.value().clone())
                .collect::<Vec<_>>(),
            vec![value(1, "stone"), value(2, "dirt")]
        );
    }
}
