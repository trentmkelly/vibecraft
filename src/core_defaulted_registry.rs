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

#[derive(Debug, Clone, PartialEq, Eq)]
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
struct HolderReferenceModel {
    key: ResourceKeyModel,
    value: Option<ValueModel>,
}

impl HolderReferenceModel {
    fn stand_alone(key: ResourceKeyModel) -> Self {
        Self { key, value: None }
    }

    fn bind_value(&mut self, value: ValueModel) {
        self.value = Some(value);
    }

    fn value(&self) -> Result<&ValueModel, String> {
        self.value
            .as_ref()
            .ok_or_else(|| format!("Trying to access unbound value '{}'", self.key))
    }
}

#[derive(Debug, Clone)]
struct MappedRegistryModel {
    key: IdentifierModel,
    by_id: Vec<HolderReferenceModel>,
    to_id: BTreeMap<usize, usize>,
    by_location: BTreeMap<IdentifierModel, usize>,
    by_key: BTreeMap<ResourceKeyModel, usize>,
    by_value: BTreeMap<usize, usize>,
    registration_infos: BTreeMap<ResourceKeyModel, RegistrationInfoModel>,
    lifecycle: LifecycleModel,
    frozen: bool,
}

impl MappedRegistryModel {
    fn new(registry_key: &str, lifecycle: LifecycleModel) -> Self {
        Self {
            key: IdentifierModel::parse(registry_key),
            by_id: Vec::new(),
            to_id: BTreeMap::new(),
            by_location: BTreeMap::new(),
            by_key: BTreeMap::new(),
            by_value: BTreeMap::new(),
            registration_infos: BTreeMap::new(),
            lifecycle,
            frozen: false,
        }
    }

    fn register(
        &mut self,
        key: ResourceKeyModel,
        value: ValueModel,
        registration_info: RegistrationInfoModel,
    ) -> Result<HolderReferenceModel, String> {
        self.validate_write_for_key(&key)?;
        if self.by_location.contains_key(key.identifier()) {
            return Err(format!("Adding duplicate key '{key}' to registry"));
        }
        if self.by_value.contains_key(&value.identity) {
            return Err(format!("Adding duplicate value '{value}' to registry"));
        }

        let mut holder = HolderReferenceModel::stand_alone(key.clone());
        holder.bind_value(value.clone());
        let new_id = self.by_id.len();
        self.by_id.push(holder.clone());
        self.to_id.insert(value.identity, new_id);
        self.by_location.insert(key.identifier().clone(), new_id);
        self.by_key.insert(key.clone(), new_id);
        self.by_value.insert(value.identity, new_id);
        self.lifecycle = self.lifecycle.add(registration_info.lifecycle);
        self.registration_infos.insert(key, registration_info);
        Ok(holder)
    }

    fn validate_write_for_key(&self, key: &ResourceKeyModel) -> Result<(), String> {
        if self.frozen {
            Err(format!(
                "Registry is already frozen (trying to add key {key})"
            ))
        } else {
            Ok(())
        }
    }

    fn get_id(&self, value: Option<&ValueModel>) -> i32 {
        value
            .and_then(|value| self.to_id.get(&value.identity).copied())
            .map_or(-1, |id| id as i32)
    }

    fn get_key(&self, value: &ValueModel) -> Option<&IdentifierModel> {
        self.by_value
            .get(&value.identity)
            .map(|index| self.by_id[*index].key.identifier())
    }

    fn get_value_by_identifier(&self, key: Option<&IdentifierModel>) -> Option<&ValueModel> {
        key.and_then(|key| self.by_location.get(key))
            .and_then(|index| self.by_id[*index].value.as_ref())
    }

    fn get_value_by_resource_key(&self, key: &ResourceKeyModel) -> Option<&ValueModel> {
        self.by_key
            .get(key)
            .and_then(|index| self.by_id[*index].value.as_ref())
    }

    fn by_id(&self, id: i32) -> Option<&ValueModel> {
        if id < 0 {
            return None;
        }
        self.by_id
            .get(id as usize)
            .and_then(|holder| holder.value.as_ref())
    }

    fn get_holder_by_id(&self, id: i32) -> Option<&HolderReferenceModel> {
        if id < 0 {
            return None;
        }
        self.by_id.get(id as usize)
    }

    fn get_by_identifier(&self, id: &IdentifierModel) -> Option<&HolderReferenceModel> {
        self.by_location.get(id).map(|index| &self.by_id[*index])
    }

    fn get_by_resource_key(&self, id: &ResourceKeyModel) -> Option<&HolderReferenceModel> {
        self.by_key.get(id).map(|index| &self.by_id[*index])
    }

    fn get_any(&self) -> Option<&HolderReferenceModel> {
        self.by_id.first()
    }

    fn registry_key(&self) -> &IdentifierModel {
        &self.key
    }

    fn get_random(&self, random: &mut SequenceRandomModel) -> Option<&HolderReferenceModel> {
        if self.by_id.is_empty() {
            None
        } else {
            self.by_id.get(random.next_usize(self.by_id.len()))
        }
    }

    fn key_set(&self) -> BTreeSet<IdentifierModel> {
        self.by_location.keys().cloned().collect()
    }

    fn registry_key_set(&self) -> BTreeSet<ResourceKeyModel> {
        self.by_key.keys().cloned().collect()
    }

    fn list_values(&self) -> Vec<&ValueModel> {
        self.by_id
            .iter()
            .filter_map(|holder| holder.value.as_ref())
            .collect()
    }

    fn registration_info(&self, key: &ResourceKeyModel) -> Option<&RegistrationInfoModel> {
        self.registration_infos.get(key)
    }

    fn registry_lifecycle(&self) -> LifecycleModel {
        self.lifecycle
    }

    fn freeze(&mut self) {
        self.frozen = true;
    }
}

#[derive(Debug, Clone)]
struct DefaultedMappedRegistryModel {
    default_key: IdentifierModel,
    default_value_index: Option<usize>,
    mapped: MappedRegistryModel,
}

impl DefaultedMappedRegistryModel {
    fn new(
        default_key: &str,
        registry_key: &str,
        lifecycle: LifecycleModel,
        intrusive_holders: bool,
    ) -> Result<Self, String> {
        if intrusive_holders {
            return Err(
                "intrusive holder mode belongs to MappedRegistry and is outside this focused defaulted-registry parity model"
                    .to_string(),
            );
        }
        Ok(Self {
            default_key: IdentifierModel::parse(default_key),
            default_value_index: None,
            mapped: MappedRegistryModel::new(registry_key, lifecycle),
        })
    }

    fn register(
        &mut self,
        key: ResourceKeyModel,
        value: ValueModel,
        registration_info: RegistrationInfoModel,
    ) -> Result<HolderReferenceModel, String> {
        let result = self
            .mapped
            .register(key.clone(), value, registration_info)?;
        if self.default_key == *key.identifier() {
            self.default_value_index = self.mapped.by_key.get(&key).copied();
        }
        Ok(result)
    }

    fn default_holder(&self) -> Result<&HolderReferenceModel, String> {
        self.default_value_index
            .and_then(|index| self.mapped.by_id.get(index))
            .ok_or_else(|| format!("Default key {} has not been registered", self.default_key))
    }

    fn default_value(&self) -> Result<&ValueModel, String> {
        self.default_holder()?.value()
    }

    fn get_id(&self, value: Option<&ValueModel>) -> Result<i32, String> {
        let id = self.mapped.get_id(value);
        if id == -1 {
            Ok(self.mapped.get_id(Some(self.default_value()?)))
        } else {
            Ok(id)
        }
    }

    fn get_key(&self, value: &ValueModel) -> &IdentifierModel {
        self.mapped.get_key(value).unwrap_or(&self.default_key)
    }

    fn get_value(&self, key: Option<&IdentifierModel>) -> Result<&ValueModel, String> {
        self.mapped
            .get_value_by_identifier(key)
            .map_or_else(|| self.default_value(), Ok)
    }

    fn get_optional(&self, key: Option<&IdentifierModel>) -> Option<&ValueModel> {
        self.mapped.get_value_by_identifier(key)
    }

    fn get_any(&self) -> Option<&HolderReferenceModel> {
        self.default_value_index
            .and_then(|index| self.mapped.by_id.get(index))
    }

    fn by_id(&self, id: i32) -> Result<&ValueModel, String> {
        self.mapped
            .by_id(id)
            .map_or_else(|| self.default_value(), Ok)
    }

    fn get_random(&self, random: &mut SequenceRandomModel) -> Option<&HolderReferenceModel> {
        self.mapped.get_random(random).or_else(|| self.get_any())
    }

    fn get_default_key(&self) -> &IdentifierModel {
        &self.default_key
    }

    fn get_value_by_resource_key(&self, key: &ResourceKeyModel) -> Option<&ValueModel> {
        self.mapped.get_value_by_resource_key(key)
    }

    fn get_holder_by_id(&self, id: i32) -> Option<&HolderReferenceModel> {
        self.mapped.get_holder_by_id(id)
    }

    fn get_by_identifier(&self, id: &IdentifierModel) -> Option<&HolderReferenceModel> {
        self.mapped.get_by_identifier(id)
    }

    fn get_by_resource_key(&self, id: &ResourceKeyModel) -> Option<&HolderReferenceModel> {
        self.mapped.get_by_resource_key(id)
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

    fn registry_with_default() -> DefaultedMappedRegistryModel {
        let mut registry = DefaultedMappedRegistryModel::new(
            "minecraft:air",
            REGISTRY,
            LifecycleModel::Stable,
            false,
        )
        .unwrap();
        registry
            .register(
                key("minecraft:stone"),
                value(1, "stone"),
                RegistrationInfoModel::stable(),
            )
            .unwrap();
        registry
            .register(
                key("minecraft:air"),
                value(2, "air"),
                RegistrationInfoModel::stable(),
            )
            .unwrap();
        registry
            .register(
                key("minecraft:test"),
                value(3, "test"),
                RegistrationInfoModel::experimental(),
            )
            .unwrap();
        registry
    }

    #[test]
    fn defaulted_registry_interface_methods_are_non_null_fallbacks() {
        let registry = registry_with_default();
        let unknown = value(99, "unknown");

        assert_eq!(
            registry.get_default_key(),
            &IdentifierModel::parse("minecraft:air")
        );
        assert_eq!(registry.mapped.registry_key().to_string(), REGISTRY);
        assert_eq!(registry.get_id(Some(&value(1, "stone"))).unwrap(), 0);
        assert_eq!(registry.get_id(Some(&value(99, "unknown"))).unwrap(), 1);
        assert_eq!(registry.get_id(None).unwrap(), 1);
        assert_eq!(
            registry.get_key(&value(1, "stone")).to_string(),
            "minecraft:stone"
        );
        assert_eq!(registry.get_key(&unknown).to_string(), "minecraft:air");
        assert_eq!(
            registry
                .get_value(Some(&IdentifierModel::parse("minecraft:stone")))
                .unwrap(),
            &value(1, "stone")
        );
        assert_eq!(
            registry
                .get_value(Some(&IdentifierModel::parse("minecraft:missing")))
                .unwrap(),
            &value(2, "air")
        );
        assert_eq!(registry.get_value(None).unwrap(), &value(2, "air"));
        assert_eq!(registry.by_id(2).unwrap(), &value(3, "test"));
        assert_eq!(registry.by_id(-1).unwrap(), &value(2, "air"));
        assert_eq!(registry.by_id(99).unwrap(), &value(2, "air"));
    }

    #[test]
    fn optional_holder_and_resource_key_lookup_keep_mapped_registry_null_semantics() {
        let registry = registry_with_default();

        assert_eq!(
            registry.get_optional(Some(&IdentifierModel::parse("minecraft:stone"))),
            Some(&value(1, "stone"))
        );
        assert_eq!(
            registry.get_optional(Some(&IdentifierModel::parse("minecraft:missing"))),
            None
        );
        assert_eq!(registry.get_optional(None), None);
        assert_eq!(
            registry.get_value_by_resource_key(&key("minecraft:missing")),
            None
        );
        assert!(registry
            .get_by_identifier(&IdentifierModel::parse("minecraft:missing"))
            .is_none());
        assert!(registry
            .get_by_resource_key(&key("minecraft:missing"))
            .is_none());
        assert!(registry.get_holder_by_id(99).is_none());
    }

    #[test]
    fn registering_default_key_replaces_get_any_and_random_empty_fallback_target() {
        let mut registry = DefaultedMappedRegistryModel::new(
            "minecraft:air",
            REGISTRY,
            LifecycleModel::Stable,
            false,
        )
        .unwrap();
        assert!(registry.get_any().is_none());
        assert!(registry
            .get_random(&mut SequenceRandomModel::new([0]))
            .is_none());

        registry
            .register(
                key("minecraft:stone"),
                value(1, "stone"),
                RegistrationInfoModel::stable(),
            )
            .unwrap();
        assert_eq!(
            registry.mapped.get_any().unwrap().value().unwrap(),
            &value(1, "stone")
        );
        assert!(registry.get_any().is_none());

        registry
            .register(
                key("minecraft:air"),
                value(2, "air"),
                RegistrationInfoModel::stable(),
            )
            .unwrap();
        assert_eq!(
            registry.get_any().unwrap().value().unwrap(),
            &value(2, "air")
        );

        let mut empty_mapped_defaulted = registry.clone();
        empty_mapped_defaulted.mapped.by_id.clear();
        assert!(empty_mapped_defaulted
            .get_random(&mut SequenceRandomModel::new([0]))
            .is_none());

        let mut random = SequenceRandomModel::new([1]);
        assert_eq!(
            registry.get_random(&mut random).unwrap().value().unwrap(),
            &value(2, "air")
        );
    }

    #[test]
    fn parent_mapped_registry_registration_order_sets_ids_keys_and_lifecycle() {
        let registry = registry_with_default();

        assert_eq!(
            registry.mapped.list_values(),
            vec![&value(1, "stone"), &value(2, "air"), &value(3, "test"),]
        );
        assert_eq!(
            registry.mapped.key_set(),
            BTreeSet::from([
                IdentifierModel::parse("minecraft:air"),
                IdentifierModel::parse("minecraft:stone"),
                IdentifierModel::parse("minecraft:test"),
            ])
        );
        assert_eq!(
            registry.mapped.registry_key_set(),
            BTreeSet::from([
                key("minecraft:air"),
                key("minecraft:stone"),
                key("minecraft:test"),
            ])
        );
        assert_eq!(
            registry
                .mapped
                .registration_info(&key("minecraft:test"))
                .unwrap(),
            &RegistrationInfoModel::experimental()
        );
        assert_eq!(
            registry.mapped.registry_lifecycle(),
            LifecycleModel::Experimental
        );
    }

    #[test]
    fn duplicate_and_frozen_registration_failures_match_java_messages() {
        let mut registry = registry_with_default();
        assert_eq!(
            registry
                .register(
                    key("minecraft:stone"),
                    value(4, "duplicate-key"),
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

        registry.mapped.freeze();
        assert_eq!(
            registry
                .register(
                    key("minecraft:late"),
                    value(5, "late"),
                    RegistrationInfoModel::stable(),
                )
                .unwrap_err(),
            "Registry is already frozen (trying to add key minecraft:test_registry / minecraft:late)"
        );
    }

    #[test]
    fn missing_default_registration_matches_late_failure_surface() {
        let mut registry = DefaultedMappedRegistryModel::new(
            "minecraft:air",
            REGISTRY,
            LifecycleModel::Stable,
            false,
        )
        .unwrap();
        registry
            .register(
                key("minecraft:stone"),
                value(1, "stone"),
                RegistrationInfoModel::stable(),
            )
            .unwrap();

        assert_eq!(
            registry.get_id(Some(&value(99, "unknown"))).unwrap_err(),
            "Default key minecraft:air has not been registered"
        );
        assert_eq!(
            registry
                .get_value(Some(&IdentifierModel::parse("minecraft:missing")))
                .unwrap_err(),
            "Default key minecraft:air has not been registered"
        );
        assert_eq!(
            registry.by_id(99).unwrap_err(),
            "Default key minecraft:air has not been registered"
        );
    }
}
