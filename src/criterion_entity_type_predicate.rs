use std::collections::{BTreeMap, BTreeSet};

use crate::registry::Identifier;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntityTypePredicateModel {
    types: HolderSetModel,
}

impl EntityTypePredicateModel {
    pub fn new(types: HolderSetModel) -> Self {
        Self { types }
    }

    pub fn of_type(entity_type: Identifier) -> Self {
        Self::new(HolderSetModel::direct(entity_type))
    }

    pub fn of_tag(lookup: &EntityTypeLookupModel, tag: &Identifier) -> Result<Self, String> {
        Ok(Self::new(lookup.get_or_throw(tag)?))
    }

    pub fn matches(&self, entity_type: &Identifier) -> bool {
        self.types.contains(entity_type)
    }

    pub fn types(&self) -> &HolderSetModel {
        &self.types
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HolderSetModel {
    entries: BTreeSet<Identifier>,
}

impl HolderSetModel {
    pub fn direct(entity_type: Identifier) -> Self {
        Self {
            entries: BTreeSet::from([entity_type]),
        }
    }

    pub fn from_entries(entries: impl IntoIterator<Item = Identifier>) -> Self {
        Self {
            entries: entries.into_iter().collect(),
        }
    }

    pub fn contains(&self, entity_type: &Identifier) -> bool {
        self.entries.contains(entity_type)
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EntityTypeLookupModel {
    tags: BTreeMap<Identifier, HolderSetModel>,
}

impl EntityTypeLookupModel {
    pub fn with_tag(
        mut self,
        tag: Identifier,
        entries: impl IntoIterator<Item = Identifier>,
    ) -> Self {
        self.tags.insert(tag, HolderSetModel::from_entries(entries));
        self
    }

    pub fn get_or_throw(&self, tag: &Identifier) -> Result<HolderSetModel, String> {
        self.tags
            .get(tag)
            .cloned()
            .ok_or_else(|| format!("Missing entity type tag: {tag}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(value: &str) -> Identifier {
        Identifier::parse(value).unwrap()
    }

    fn lookup() -> EntityTypeLookupModel {
        EntityTypeLookupModel::default().with_tag(
            id("minecraft:undead"),
            [
                id("minecraft:zombie"),
                id("minecraft:skeleton"),
                id("minecraft:wither_skeleton"),
            ],
        )
    }

    #[test]
    fn direct_type_factory_matches_only_that_entity_type() {
        let predicate = EntityTypePredicateModel::of_type(id("minecraft:zombie"));

        assert!(predicate.matches(&id("minecraft:zombie")));
        assert!(!predicate.matches(&id("minecraft:skeleton")));
        assert_eq!(
            predicate.types(),
            &HolderSetModel::direct(id("minecraft:zombie"))
        );
    }

    #[test]
    fn tag_factory_uses_lookup_holder_set() {
        let predicate =
            EntityTypePredicateModel::of_tag(&lookup(), &id("minecraft:undead")).unwrap();

        assert!(predicate.matches(&id("minecraft:zombie")));
        assert!(predicate.matches(&id("minecraft:skeleton")));
        assert!(predicate.matches(&id("minecraft:wither_skeleton")));
        assert!(!predicate.matches(&id("minecraft:creeper")));
    }

    #[test]
    fn missing_tag_factory_reports_lookup_error_like_get_or_throw() {
        let missing = EntityTypePredicateModel::of_tag(&lookup(), &id("minecraft:missing"));

        assert_eq!(
            missing,
            Err("Missing entity type tag: minecraft:missing".to_string())
        );
    }

    #[test]
    fn homogeneous_holder_set_contains_any_registered_entry() {
        let holder_set = HolderSetModel::from_entries([
            id("minecraft:cow"),
            id("minecraft:sheep"),
            id("minecraft:pig"),
        ]);
        let predicate = EntityTypePredicateModel::new(holder_set);

        assert!(predicate.matches(&id("minecraft:cow")));
        assert!(predicate.matches(&id("minecraft:pig")));
        assert!(!predicate.matches(&id("minecraft:chicken")));
    }
}
