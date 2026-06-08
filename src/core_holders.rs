use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LifecycleModel {
    Stable,
    Experimental,
}

impl LifecycleModel {
    pub fn add(self, other: Self) -> Self {
        if matches!(self, Self::Experimental) || matches!(other, Self::Experimental) {
            Self::Experimental
        } else {
            Self::Stable
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ResourceKeyModel {
    id: usize,
    registry: String,
    location: String,
}

impl ResourceKeyModel {
    pub fn new(id: usize, registry: impl Into<String>, location: impl Into<String>) -> Self {
        Self {
            id,
            registry: registry.into(),
            location: location.into(),
        }
    }

    pub fn identifier(&self) -> &str {
        &self.location
    }

    pub fn registry_key(&self) -> &str {
        &self.registry
    }
}

impl std::fmt::Display for ResourceKeyModel {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}:{}", self.registry, self.location)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct TagKeyModel {
    registry: String,
    location: String,
}

impl TagKeyModel {
    pub fn new(registry: impl Into<String>, location: impl Into<String>) -> Self {
        Self {
            registry: registry.into(),
            location: location.into(),
        }
    }

    pub fn registry(&self) -> &str {
        &self.registry
    }
}

impl std::fmt::Display for TagKeyModel {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}#{}", self.registry, self.location)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EitherModel<L, R> {
    Left(L),
    Right(R),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HolderOwnerModel {
    identity: usize,
}

impl HolderOwnerModel {
    pub fn new(identity: usize) -> Self {
        Self { identity }
    }

    pub fn can_serialize_in(self, context: Self) -> bool {
        self == context
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HolderKindModel {
    Reference,
    Direct,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReferenceTypeModel {
    StandAlone,
    Intrusive,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HolderModel {
    Direct {
        value: String,
        components: String,
    },
    Reference {
        id: usize,
        owner: HolderOwnerModel,
        reference_type: ReferenceTypeModel,
        key: Option<ResourceKeyModel>,
        value: Option<String>,
        tags: Option<BTreeSet<TagKeyModel>>,
        components: Option<String>,
    },
}

impl HolderModel {
    pub fn direct(value: impl Into<String>) -> Self {
        Self::Direct {
            value: value.into(),
            components: "EMPTY".to_string(),
        }
    }

    pub fn direct_with_components(value: impl Into<String>, components: impl Into<String>) -> Self {
        Self::Direct {
            value: value.into(),
            components: components.into(),
        }
    }

    pub fn create_stand_alone(id: usize, owner: HolderOwnerModel, key: ResourceKeyModel) -> Self {
        Self::Reference {
            id,
            owner,
            reference_type: ReferenceTypeModel::StandAlone,
            key: Some(key),
            value: None,
            tags: None,
            components: None,
        }
    }

    pub fn create_intrusive(id: usize, owner: HolderOwnerModel, value: Option<String>) -> Self {
        Self::Reference {
            id,
            owner,
            reference_type: ReferenceTypeModel::Intrusive,
            key: None,
            value,
            tags: None,
            components: None,
        }
    }

    pub fn value(&self) -> Result<&str, String> {
        match self {
            Self::Direct { value, .. } => Ok(value),
            Self::Reference {
                key, value, owner, ..
            } => value.as_deref().ok_or_else(|| {
                format!(
                    "Trying to access unbound value '{}' from registry {:?}",
                    key.as_ref()
                        .map(ToString::to_string)
                        .unwrap_or_else(|| "null".to_string()),
                    owner
                )
            }),
        }
    }

    pub fn key(&self) -> Result<&ResourceKeyModel, String> {
        match self {
            Self::Direct { .. } => Err("Direct holder has no key".to_string()),
            Self::Reference {
                key, value, owner, ..
            } => key.as_ref().ok_or_else(|| {
                format!(
                    "Trying to access unbound value '{}' from registry {:?}",
                    value.as_deref().unwrap_or("null"),
                    owner
                )
            }),
        }
    }

    pub fn is_bound(&self) -> bool {
        match self {
            Self::Direct { .. } => true,
            Self::Reference { key, value, .. } => key.is_some() && value.is_some(),
        }
    }

    pub fn are_components_bound(&self) -> bool {
        match self {
            Self::Direct { .. } => true,
            Self::Reference { components, .. } => components.is_some(),
        }
    }

    pub fn is_identifier(&self, key: &str) -> Result<bool, String> {
        match self {
            Self::Direct { .. } => Ok(false),
            Self::Reference { .. } => Ok(self.key()?.identifier() == key),
        }
    }

    pub fn is_resource_key(&self, key: &ResourceKeyModel) -> Result<bool, String> {
        match self {
            Self::Direct { .. } => Ok(false),
            Self::Reference { .. } => Ok(self.key()?.id == key.id),
        }
    }

    pub fn is_predicate(
        &self,
        predicate: impl Fn(&ResourceKeyModel) -> bool,
    ) -> Result<bool, String> {
        match self {
            Self::Direct { .. } => Ok(false),
            Self::Reference { .. } => Ok(predicate(self.key()?)),
        }
    }

    pub fn is_tag(&self, tag: &TagKeyModel) -> Result<bool, String> {
        match self {
            Self::Direct { .. } => Ok(false),
            Self::Reference { tags, .. } => tags
                .as_ref()
                .ok_or_else(|| "Tags not bound".to_string())
                .map(|tags| tags.contains(tag)),
        }
    }

    pub fn is_holder(&self, holder: &HolderModel) -> Result<bool, String> {
        match self {
            Self::Direct { value, .. } => Ok(value == holder.value()?),
            Self::Reference { .. } => holder.is_resource_key(self.key()?),
        }
    }

    pub fn tags(&self) -> Result<Vec<TagKeyModel>, String> {
        match self {
            Self::Direct { .. } => Ok(Vec::new()),
            Self::Reference { tags, .. } => tags
                .as_ref()
                .ok_or_else(|| "Tags not bound".to_string())
                .map(|tags| tags.iter().cloned().collect()),
        }
    }

    pub fn components(&self) -> Result<&str, String> {
        match self {
            Self::Direct { components, .. } => Ok(components),
            Self::Reference { components, .. } => components
                .as_deref()
                .ok_or_else(|| "Components not bound yet".to_string()),
        }
    }

    pub fn unwrap(&self) -> Result<EitherModel<ResourceKeyModel, String>, String> {
        match self {
            Self::Direct { value, .. } => Ok(EitherModel::Right(value.clone())),
            Self::Reference { .. } => Ok(EitherModel::Left(self.key()?.clone())),
        }
    }

    pub fn unwrap_key(&self) -> Result<Option<ResourceKeyModel>, String> {
        match self {
            Self::Direct { .. } => Ok(None),
            Self::Reference { .. } => Ok(Some(self.key()?.clone())),
        }
    }

    pub fn kind(&self) -> HolderKindModel {
        match self {
            Self::Direct { .. } => HolderKindModel::Direct,
            Self::Reference { .. } => HolderKindModel::Reference,
        }
    }

    pub fn can_serialize_in(&self, context: HolderOwnerModel) -> bool {
        match self {
            Self::Direct { .. } => true,
            Self::Reference { owner, .. } => owner.can_serialize_in(context),
        }
    }

    pub fn registered_name(&self) -> String {
        self.unwrap_key()
            .ok()
            .flatten()
            .map(|key| key.identifier().to_string())
            .unwrap_or_else(|| "[unregistered]".to_string())
    }

    pub fn bind_key(&mut self, key: ResourceKeyModel) -> Result<(), String> {
        match self {
            Self::Reference {
                key: existing_key, ..
            } => {
                if existing_key
                    .as_ref()
                    .is_some_and(|existing| existing.id != key.id)
                {
                    return Err(format!(
                        "Can't change holder key: existing={}, new={}",
                        existing_key.as_ref().unwrap(),
                        key
                    ));
                }
                *existing_key = Some(key);
                Ok(())
            }
            Self::Direct { .. } => Err("Direct holder cannot bind key".to_string()),
        }
    }

    pub fn bind_value(&mut self, new_value: impl Into<String>) -> Result<(), String> {
        let new_value = new_value.into();
        match self {
            Self::Reference {
                reference_type,
                key,
                value,
                ..
            } => {
                if *reference_type == ReferenceTypeModel::Intrusive
                    && value.as_ref().is_some_and(|value| value != &new_value)
                {
                    return Err(format!(
                        "Can't change holder {} value: existing={}, new={}",
                        key.as_ref()
                            .map(ToString::to_string)
                            .unwrap_or_else(|| "null".to_string()),
                        value.as_ref().unwrap(),
                        new_value
                    ));
                }
                *value = Some(new_value);
                Ok(())
            }
            Self::Direct { .. } => Err("Direct holder cannot bind value".to_string()),
        }
    }

    pub fn bind_tags(&mut self, tags: impl IntoIterator<Item = TagKeyModel>) -> Result<(), String> {
        match self {
            Self::Reference {
                tags: existing_tags,
                ..
            } => {
                *existing_tags = Some(tags.into_iter().collect());
                Ok(())
            }
            Self::Direct { .. } => Err("Direct holder cannot bind tags".to_string()),
        }
    }

    pub fn bind_components(&mut self, components: impl Into<String>) -> Result<(), String> {
        match self {
            Self::Reference {
                components: existing,
                ..
            } => {
                *existing = Some(components.into());
                Ok(())
            }
            Self::Direct { .. } => Err("Direct holder cannot bind components".to_string()),
        }
    }
}

impl std::fmt::Display for HolderModel {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Direct { value, .. } => write!(formatter, "Direct{{{value}}}"),
            Self::Reference { key, value, .. } => {
                write!(formatter, "Reference{{{key:?}={value:?}}}")
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HolderSetModel {
    Direct {
        contents: Vec<HolderModel>,
    },
    Named {
        owner: HolderOwnerModel,
        key: TagKeyModel,
        contents: Option<Vec<HolderModel>>,
    },
}

impl HolderSetModel {
    pub fn empty() -> Self {
        Self::Direct {
            contents: Vec::new(),
        }
    }

    pub fn direct(contents: Vec<HolderModel>) -> Self {
        Self::Direct { contents }
    }

    pub fn named(owner: HolderOwnerModel, key: TagKeyModel) -> Self {
        Self::Named {
            owner,
            key,
            contents: None,
        }
    }

    pub fn bind(&mut self, new_contents: Vec<HolderModel>) -> Result<(), String> {
        match self {
            Self::Named { contents, .. } => {
                *contents = Some(new_contents);
                Ok(())
            }
            Self::Direct { .. } => Err("Direct holder set cannot bind".to_string()),
        }
    }

    fn contents(&self) -> Result<&[HolderModel], String> {
        match self {
            Self::Direct { contents } => Ok(contents),
            Self::Named {
                owner,
                key,
                contents,
            } => contents.as_deref().ok_or_else(|| {
                format!(
                    "Trying to access unbound tag '{}' from registry {:?}",
                    key, owner
                )
            }),
        }
    }

    pub fn size(&self) -> Result<usize, String> {
        Ok(self.contents()?.len())
    }

    pub fn is_bound(&self) -> bool {
        match self {
            Self::Direct { .. } => true,
            Self::Named { contents, .. } => contents.is_some(),
        }
    }

    pub fn unwrap(&self) -> Result<EitherModel<TagKeyModel, Vec<HolderModel>>, String> {
        match self {
            Self::Direct { contents } => Ok(EitherModel::Right(contents.clone())),
            Self::Named { key, .. } => Ok(EitherModel::Left(key.clone())),
        }
    }

    pub fn unwrap_key(&self) -> Option<TagKeyModel> {
        match self {
            Self::Direct { .. } => None,
            Self::Named { key, .. } => Some(key.clone()),
        }
    }

    pub fn get_random_element(
        &self,
        random: &mut SequenceRandomModel,
    ) -> Result<Option<HolderModel>, String> {
        let contents = self.contents()?;
        if contents.is_empty() {
            Ok(None)
        } else {
            Ok(Some(
                contents[random.next_int(contents.len() as i32) as usize].clone(),
            ))
        }
    }

    pub fn get(&self, index: usize) -> Result<HolderModel, String> {
        Ok(self.contents()?[index].clone())
    }

    pub fn stream(&self) -> Result<Vec<HolderModel>, String> {
        Ok(self.contents()?.to_vec())
    }

    pub fn contains(&self, value: &HolderModel) -> Result<bool, String> {
        match self {
            Self::Direct { contents } => Ok(contents.contains(value)),
            Self::Named { key, .. } => value.is_tag(key),
        }
    }

    pub fn can_serialize_in(&self, owner: HolderOwnerModel) -> bool {
        match self {
            Self::Direct { .. } => true,
            Self::Named {
                owner: set_owner, ..
            } => set_owner.can_serialize_in(owner),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HolderGetterModel {
    elements: BTreeMap<usize, HolderModel>,
    tags: BTreeMap<TagKeyModel, HolderSetModel>,
}

impl HolderGetterModel {
    pub fn new(
        elements: Vec<(ResourceKeyModel, HolderModel)>,
        tags: Vec<(TagKeyModel, HolderSetModel)>,
    ) -> Self {
        Self {
            elements: elements
                .into_iter()
                .map(|(key, holder)| (key.id, holder))
                .collect(),
            tags: tags.into_iter().collect(),
        }
    }

    pub fn get(&self, id: &ResourceKeyModel) -> Option<HolderModel> {
        self.elements.get(&id.id).cloned()
    }

    pub fn get_or_throw(&self, id: &ResourceKeyModel) -> Result<HolderModel, String> {
        self.get(id).ok_or_else(|| format!("Missing element {id}"))
    }

    pub fn get_tag(&self, id: &TagKeyModel) -> Option<HolderSetModel> {
        self.tags.get(id).cloned()
    }

    pub fn get_tag_or_throw(&self, id: &TagKeyModel) -> Result<HolderSetModel, String> {
        self.get_tag(id).ok_or_else(|| format!("Missing tag {id}"))
    }

    pub fn get_random_element_of(
        &self,
        tag: &TagKeyModel,
        random: &mut SequenceRandomModel,
    ) -> Result<Option<HolderModel>, String> {
        match self.get_tag(tag) {
            Some(set) => set.get_random_element(random),
            None => Ok(None),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegistryLookupModel {
    registry_key: String,
    lifecycle: LifecycleModel,
    getter: HolderGetterModel,
}

impl RegistryLookupModel {
    pub fn new(
        registry_key: impl Into<String>,
        lifecycle: LifecycleModel,
        getter: HolderGetterModel,
    ) -> Self {
        Self {
            registry_key: registry_key.into(),
            lifecycle,
            getter,
        }
    }

    pub fn key(&self) -> &str {
        &self.registry_key
    }

    pub fn registry_lifecycle(&self) -> LifecycleModel {
        self.lifecycle
    }

    pub fn get(&self, id: &ResourceKeyModel) -> Option<HolderModel> {
        self.getter.get(id)
    }

    pub fn get_tag(&self, id: &TagKeyModel) -> Option<HolderSetModel> {
        self.getter.get_tag(id)
    }

    pub fn list_elements(&self) -> Vec<HolderModel> {
        self.getter.elements.values().cloned().collect()
    }

    pub fn list_element_ids(&self) -> Vec<ResourceKeyModel> {
        self.list_elements()
            .into_iter()
            .filter_map(|holder| holder.unwrap_key().ok().flatten())
            .collect()
    }

    pub fn list_tags(&self) -> Vec<HolderSetModel> {
        self.getter.tags.values().cloned().collect()
    }

    pub fn list_tag_ids(&self) -> Vec<TagKeyModel> {
        self.getter.tags.keys().cloned().collect()
    }

    pub fn filter_elements(&self, filter: impl Fn(&str) -> bool) -> Self {
        let elements = self
            .getter
            .elements
            .iter()
            .filter_map(|(id, holder)| {
                holder
                    .value()
                    .ok()
                    .filter(|value| filter(value))
                    .map(|_| (*id, holder.clone()))
            })
            .collect();
        Self {
            registry_key: self.registry_key.clone(),
            lifecycle: self.lifecycle,
            getter: HolderGetterModel {
                elements,
                tags: self.getter.tags.clone(),
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HolderProviderModel {
    registries: BTreeMap<String, RegistryLookupModel>,
}

impl HolderProviderModel {
    pub fn create(lookups: Vec<RegistryLookupModel>) -> Self {
        Self {
            registries: lookups
                .into_iter()
                .map(|lookup| (lookup.registry_key.clone(), lookup))
                .collect(),
        }
    }

    pub fn list_registry_keys(&self) -> Vec<String> {
        self.registries.keys().cloned().collect()
    }

    pub fn list_registries(&self) -> Vec<RegistryLookupModel> {
        self.registries.values().cloned().collect()
    }

    pub fn lookup(&self, key: &str) -> Option<RegistryLookupModel> {
        self.registries.get(key).cloned()
    }

    pub fn lookup_or_throw(&self, key: &str) -> Result<RegistryLookupModel, String> {
        self.lookup(key)
            .ok_or_else(|| format!("Registry {key} not found"))
    }

    pub fn get(&self, id: &ResourceKeyModel) -> Option<HolderModel> {
        self.lookup(id.registry_key())
            .and_then(|lookup| lookup.get(id))
    }

    pub fn get_or_throw(&self, id: &ResourceKeyModel) -> Result<HolderModel, String> {
        self.get(id).ok_or_else(|| format!("Missing element {id}"))
    }

    pub fn get_tag(&self, id: &TagKeyModel) -> Option<HolderSetModel> {
        self.lookup(id.registry())
            .and_then(|lookup| lookup.get_tag(id))
    }

    pub fn get_tag_or_throw(&self, id: &TagKeyModel) -> Result<HolderSetModel, String> {
        self.get_tag(id).ok_or_else(|| format!("Missing tag {id}"))
    }

    pub fn serialization_context_parent_marker(&self, parent_marker: &str) -> String {
        format!("RegistryOps({parent_marker})")
    }

    pub fn all_registries_lifecycle(&self) -> LifecycleModel {
        self.registries
            .values()
            .map(RegistryLookupModel::registry_lifecycle)
            .fold(LifecycleModel::Stable, LifecycleModel::add)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SequenceRandomModel {
    values: Vec<i32>,
    index: usize,
}

impl SequenceRandomModel {
    pub fn new(values: Vec<i32>) -> Self {
        Self { values, index: 0 }
    }

    pub fn next_int(&mut self, bound: i32) -> i32 {
        let value = self.values[self.index % self.values.len()];
        self.index += 1;
        value.rem_euclid(bound)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(id: usize, location: &str) -> ResourceKeyModel {
        ResourceKeyModel::new(id, "minecraft:block", location)
    }

    fn tag(location: &str) -> TagKeyModel {
        TagKeyModel::new("minecraft:block", location)
    }

    fn bound_ref(
        id: usize,
        owner: HolderOwnerModel,
        key: ResourceKeyModel,
        value: &str,
        tags: Vec<TagKeyModel>,
    ) -> HolderModel {
        let mut holder = HolderModel::create_stand_alone(id, owner, key);
        holder.bind_value(value).unwrap();
        holder.bind_tags(tags).unwrap();
        holder.bind_components("components").unwrap();
        holder
    }

    #[test]
    fn direct_holder_matches_java_direct_contract() {
        let direct = HolderModel::direct_with_components("stone", "hardness=1.5");
        assert_eq!(direct.value().unwrap(), "stone");
        assert!(direct.is_bound());
        assert!(direct.are_components_bound());
        assert!(!direct.is_identifier("minecraft:stone").unwrap());
        assert!(!direct.is_resource_key(&key(1, "stone")).unwrap());
        assert!(!direct.is_predicate(|_| true).unwrap());
        assert!(!direct.is_tag(&tag("mineable/pickaxe")).unwrap());
        assert!(direct.is_holder(&HolderModel::direct("stone")).unwrap());
        assert_eq!(direct.tags().unwrap(), Vec::<TagKeyModel>::new());
        assert_eq!(direct.components().unwrap(), "hardness=1.5");
        assert_eq!(direct.unwrap(), Ok(EitherModel::Right("stone".to_string())));
        assert_eq!(direct.unwrap_key().unwrap(), None);
        assert_eq!(direct.kind(), HolderKindModel::Direct);
        assert!(direct.can_serialize_in(HolderOwnerModel::new(99)));
        assert_eq!(direct.registered_name(), "[unregistered]");
        assert_eq!(direct.to_string(), "Direct{stone}");
    }

    #[test]
    fn reference_holder_membership_and_unwrap_behavior_match_java() {
        let owner = HolderOwnerModel::new(1);
        let stone_key = key(10, "stone");
        let mineable = tag("mineable/pickaxe");
        let mut holder = HolderModel::create_stand_alone(7, owner, stone_key.clone());
        assert!(!holder.is_bound());
        assert!(!holder.are_components_bound());
        assert_eq!(
            holder.value().unwrap_err(),
            "Trying to access unbound value 'minecraft:block:stone' from registry HolderOwnerModel { identity: 1 }"
        );
        holder.bind_value("stone").unwrap();
        holder.bind_tags([mineable.clone()]).unwrap();
        holder.bind_components("stone-components").unwrap();
        assert!(holder.is_bound());
        assert!(holder.are_components_bound());
        assert_eq!(holder.value().unwrap(), "stone");
        assert_eq!(holder.key().unwrap(), &stone_key);
        assert!(holder.is_identifier("stone").unwrap());
        assert!(holder.is_resource_key(&stone_key).unwrap());
        assert!(!holder
            .is_resource_key(&ResourceKeyModel::new(99, "minecraft:block", "stone_clone"))
            .unwrap());
        assert!(holder
            .is_predicate(|key| key.identifier() == "stone")
            .unwrap());
        assert!(holder.is_tag(&mineable).unwrap());
        assert!(holder
            .is_holder(&HolderModel::create_stand_alone(
                8,
                owner,
                stone_key.clone()
            ))
            .unwrap());
        assert_eq!(holder.tags().unwrap(), vec![mineable.clone()]);
        assert_eq!(holder.components().unwrap(), "stone-components");
        assert_eq!(holder.unwrap(), Ok(EitherModel::Left(stone_key.clone())));
        assert_eq!(holder.unwrap_key().unwrap(), Some(stone_key.clone()));
        assert_eq!(holder.kind(), HolderKindModel::Reference);
        assert!(holder.can_serialize_in(owner));
        assert!(!holder.can_serialize_in(HolderOwnerModel::new(2)));
        assert_eq!(holder.registered_name(), "stone");
    }

    #[test]
    fn reference_holder_binding_failures_match_java() {
        let owner = HolderOwnerModel::new(1);
        let stone_key = key(10, "stone");
        let dirt_key = key(11, "dirt");
        let mineable = tag("mineable/pickaxe");
        let mut holder = HolderModel::create_stand_alone(7, owner, stone_key);
        assert_eq!(
            holder.bind_key(dirt_key.clone()).unwrap_err(),
            "Can't change holder key: existing=minecraft:block:stone, new=minecraft:block:dirt"
        );

        let mut intrusive = HolderModel::create_intrusive(9, owner, Some("granite".to_string()));
        assert_eq!(
            intrusive.bind_value("andesite").unwrap_err(),
            "Can't change holder null value: existing=granite, new=andesite"
        );
        assert_eq!(intrusive.key().unwrap_err(), "Trying to access unbound value 'granite' from registry HolderOwnerModel { identity: 1 }");
        assert_eq!(
            HolderModel::create_stand_alone(10, owner, key(12, "gold"))
                .is_tag(&mineable)
                .unwrap_err(),
            "Tags not bound"
        );
        assert_eq!(
            HolderModel::create_stand_alone(11, owner, key(13, "iron"))
                .components()
                .unwrap_err(),
            "Components not bound yet"
        );
    }

    #[test]
    fn holder_set_direct_and_named_match_java_list_backed_contract() {
        let owner = HolderOwnerModel::new(1);
        let mineable = tag("mineable/pickaxe");
        let stone = bound_ref(1, owner, key(1, "stone"), "stone", vec![mineable.clone()]);
        let dirt = bound_ref(2, owner, key(2, "dirt"), "dirt", vec![]);
        let direct = HolderSetModel::direct(vec![stone.clone(), dirt.clone()]);
        assert!(HolderSetModel::empty().is_bound());
        assert!(direct.is_bound());
        assert_eq!(direct.size().unwrap(), 2);
        assert_eq!(direct.stream().unwrap(), vec![stone.clone(), dirt.clone()]);
        assert_eq!(direct.get(1).unwrap(), dirt);
        assert!(direct.contains(&stone).unwrap());
        assert_eq!(
            direct.unwrap(),
            Ok(EitherModel::Right(vec![
                stone.clone(),
                bound_ref(2, owner, key(2, "dirt"), "dirt", vec![])
            ]))
        );
        assert_eq!(direct.unwrap_key(), None);
        assert!(direct.can_serialize_in(HolderOwnerModel::new(99)));
        assert_eq!(
            direct
                .get_random_element(&mut SequenceRandomModel::new(vec![1]))
                .unwrap()
                .unwrap()
                .value()
                .unwrap(),
            "dirt"
        );

        let mut named = HolderSetModel::named(owner, mineable.clone());
        assert!(!named.is_bound());
        assert_eq!(
            named.size().unwrap_err(),
            "Trying to access unbound tag 'minecraft:block#mineable/pickaxe' from registry HolderOwnerModel { identity: 1 }"
        );
        named.bind(vec![stone.clone()]).unwrap();
        assert!(named.is_bound());
        assert_eq!(named.unwrap(), Ok(EitherModel::Left(mineable.clone())));
        assert_eq!(named.unwrap_key(), Some(mineable.clone()));
        assert!(named.contains(&stone).unwrap());
        assert!(!named.contains(&HolderModel::direct("stone")).unwrap());
        assert!(named.can_serialize_in(owner));
        assert!(!named.can_serialize_in(HolderOwnerModel::new(2)));
    }

    #[test]
    fn holder_getter_and_provider_defaults_match_java_errors_and_random_tag_lookup() {
        let owner = HolderOwnerModel::new(1);
        let stone_key = key(1, "stone");
        let dirt_key = key(2, "dirt");
        let mineable = tag("mineable/pickaxe");
        let stone = bound_ref(1, owner, stone_key.clone(), "stone", vec![mineable.clone()]);
        let mut named = HolderSetModel::named(owner, mineable.clone());
        named.bind(vec![stone.clone()]).unwrap();
        let getter = HolderGetterModel::new(
            vec![(stone_key.clone(), stone.clone())],
            vec![(mineable.clone(), named.clone())],
        );
        assert_eq!(getter.get(&stone_key), Some(stone.clone()));
        assert_eq!(getter.get_or_throw(&stone_key).unwrap(), stone.clone());
        assert_eq!(
            getter.get_or_throw(&dirt_key).unwrap_err(),
            "Missing element minecraft:block:dirt"
        );
        assert_eq!(getter.get_tag(&mineable), Some(named.clone()));
        assert_eq!(getter.get_tag_or_throw(&mineable).unwrap(), named.clone());
        assert_eq!(
            getter.get_tag_or_throw(&tag("missing")).unwrap_err(),
            "Missing tag minecraft:block#missing"
        );
        assert_eq!(
            getter
                .get_random_element_of(&mineable, &mut SequenceRandomModel::new(vec![0]))
                .unwrap(),
            Some(stone.clone())
        );

        let lookup = RegistryLookupModel::new("minecraft:block", LifecycleModel::Stable, getter);
        let provider = HolderProviderModel::create(vec![lookup.clone()]);
        assert_eq!(
            provider.list_registry_keys(),
            vec!["minecraft:block".to_string()]
        );
        assert_eq!(provider.lookup("minecraft:block"), Some(lookup.clone()));
        assert_eq!(
            provider.lookup_or_throw("minecraft:item").unwrap_err(),
            "Registry minecraft:item not found"
        );
        assert_eq!(provider.get(&stone_key), Some(stone.clone()));
        assert_eq!(
            provider.get_or_throw(&dirt_key).unwrap_err(),
            "Missing element minecraft:block:dirt"
        );
        assert_eq!(provider.get_tag(&mineable), Some(named));
        assert_eq!(
            provider.get_tag_or_throw(&tag("missing")).unwrap_err(),
            "Missing tag minecraft:block#missing"
        );
        assert_eq!(
            provider.serialization_context_parent_marker("json"),
            "RegistryOps(json)"
        );
        assert_eq!(provider.all_registries_lifecycle(), LifecycleModel::Stable);
    }

    #[test]
    fn holder_lookup_listing_lifecycle_and_filtering_match_java_defaults() {
        let owner = HolderOwnerModel::new(1);
        let stone_key = key(1, "stone");
        let dirt_key = key(2, "dirt");
        let mineable = tag("mineable/pickaxe");
        let stone = bound_ref(1, owner, stone_key.clone(), "stone", vec![mineable.clone()]);
        let dirt = bound_ref(2, owner, dirt_key.clone(), "dirt", vec![]);
        let mut named = HolderSetModel::named(owner, mineable.clone());
        named.bind(vec![stone.clone()]).unwrap();
        let getter = HolderGetterModel::new(
            vec![
                (stone_key.clone(), stone.clone()),
                (dirt_key.clone(), dirt.clone()),
            ],
            vec![(mineable.clone(), named.clone())],
        );
        let lookup =
            RegistryLookupModel::new("minecraft:block", LifecycleModel::Experimental, getter);
        assert_eq!(lookup.key(), "minecraft:block");
        assert_eq!(lookup.registry_lifecycle(), LifecycleModel::Experimental);
        assert_eq!(lookup.list_elements(), vec![stone.clone(), dirt.clone()]);
        assert_eq!(
            lookup.list_element_ids(),
            vec![stone_key.clone(), dirt_key.clone()]
        );
        assert_eq!(lookup.list_tags(), vec![named.clone()]);
        assert_eq!(lookup.list_tag_ids(), vec![mineable]);
        let filtered = lookup.filter_elements(|value| value.starts_with('s'));
        assert_eq!(filtered.get(&stone_key), Some(stone));
        assert_eq!(filtered.get(&dirt_key), None);
        assert_eq!(filtered.list_elements().len(), 1);

        let provider = HolderProviderModel::create(vec![
            lookup,
            RegistryLookupModel::new(
                "minecraft:item",
                LifecycleModel::Stable,
                HolderGetterModel::new(Vec::new(), Vec::new()),
            ),
        ]);
        assert_eq!(provider.list_registries().len(), 2);
        assert_eq!(
            provider.all_registries_lifecycle(),
            LifecycleModel::Experimental
        );
    }
}
