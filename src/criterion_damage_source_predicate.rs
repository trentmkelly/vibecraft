use std::collections::BTreeSet;

use crate::registry::Identifier;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DamageSourcePredicateModel {
    pub tags: Vec<TagPredicateModel>,
    pub direct_entity: Option<EntityPredicateModel>,
    pub source_entity: Option<EntityPredicateModel>,
    pub is_direct: Option<bool>,
}

impl DamageSourcePredicateModel {
    pub fn new(
        tags: Vec<TagPredicateModel>,
        direct_entity: Option<EntityPredicateModel>,
        source_entity: Option<EntityPredicateModel>,
        is_direct: Option<bool>,
    ) -> Self {
        Self {
            tags,
            direct_entity,
            source_entity,
            is_direct,
        }
    }

    pub fn builder() -> DamageSourcePredicateBuilder {
        DamageSourcePredicateBuilder::default()
    }

    pub fn matches(&self, source: &DamageSourceModel) -> bool {
        for tag in &self.tags {
            if !tag.matches(&source.type_tags) {
                return false;
            }
        }

        if self.direct_entity.as_ref().is_some_and(|predicate| {
            !source
                .direct_entity
                .as_ref()
                .is_some_and(|entity| predicate.matches(entity))
        }) {
            return false;
        }

        if self.source_entity.as_ref().is_some_and(|predicate| {
            !source
                .source_entity
                .as_ref()
                .is_some_and(|entity| predicate.matches(entity))
        }) {
            return false;
        }

        self.is_direct
            .is_none_or(|expected_direct| expected_direct == source.is_direct)
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DamageSourcePredicateBuilder {
    tags: Vec<TagPredicateModel>,
    direct_entity: Option<EntityPredicateModel>,
    source_entity: Option<EntityPredicateModel>,
    is_direct: Option<bool>,
}

impl DamageSourcePredicateBuilder {
    pub fn damage_type() -> Self {
        Self::default()
    }

    pub fn tag(mut self, tag: TagPredicateModel) -> Self {
        self.tags.push(tag);
        self
    }

    pub fn direct(mut self, direct_entity: EntityPredicateModel) -> Self {
        self.direct_entity = Some(direct_entity);
        self
    }

    pub fn source(mut self, source_entity: EntityPredicateModel) -> Self {
        self.source_entity = Some(source_entity);
        self
    }

    pub fn with_is_direct(mut self, direct: bool) -> Self {
        self.is_direct = Some(direct);
        self
    }

    pub fn build(self) -> DamageSourcePredicateModel {
        DamageSourcePredicateModel::new(
            self.tags,
            self.direct_entity,
            self.source_entity,
            self.is_direct,
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DamageSourceModel {
    pub type_tags: BTreeSet<Identifier>,
    pub direct_entity: Option<EntityContextModel>,
    pub source_entity: Option<EntityContextModel>,
    pub is_direct: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TagPredicateModel {
    pub tag: Identifier,
    pub expected: bool,
}

impl TagPredicateModel {
    pub fn is(tag: Identifier) -> Self {
        Self {
            tag,
            expected: true,
        }
    }

    pub fn is_not(tag: Identifier) -> Self {
        Self {
            tag,
            expected: false,
        }
    }

    fn matches(&self, holder_tags: &BTreeSet<Identifier>) -> bool {
        holder_tags.contains(&self.tag) == self.expected
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntityContextModel {
    pub entity_type: Identifier,
    pub name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntityPredicateModel {
    required_type: Option<Identifier>,
    required_name: Option<String>,
}

impl EntityPredicateModel {
    pub fn any() -> Self {
        Self {
            required_type: None,
            required_name: None,
        }
    }

    pub fn entity_type(entity_type: Identifier) -> Self {
        Self {
            required_type: Some(entity_type),
            ..Self::any()
        }
    }

    pub fn named(name: &str) -> Self {
        Self {
            required_name: Some(name.to_string()),
            ..Self::any()
        }
    }

    fn matches(&self, entity: &EntityContextModel) -> bool {
        self.required_type
            .as_ref()
            .is_none_or(|entity_type| entity_type == &entity.entity_type)
            && self.required_name.as_ref().is_none_or(|name| {
                entity
                    .name
                    .as_ref()
                    .is_some_and(|entity_name| entity_name == name)
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(value: &str) -> Identifier {
        Identifier::parse(value).unwrap()
    }

    fn entity(entity_type: &str, name: Option<&str>) -> EntityContextModel {
        EntityContextModel {
            entity_type: id(entity_type),
            name: name.map(str::to_string),
        }
    }

    fn source() -> DamageSourceModel {
        DamageSourceModel {
            type_tags: BTreeSet::from([
                id("minecraft:is_projectile"),
                id("minecraft:bypasses_armor"),
            ]),
            direct_entity: Some(entity("minecraft:arrow", None)),
            source_entity: Some(entity("minecraft:skeleton", Some("Shooter"))),
            is_direct: false,
        }
    }

    #[test]
    fn damage_source_predicate_default_matches_any_source_like_java() {
        let predicate = DamageSourcePredicateBuilder::damage_type().build();

        assert!(predicate.matches(&source()));
        assert!(predicate.tags.is_empty());
    }

    #[test]
    fn damage_source_predicate_checks_all_tag_expectations_like_java() {
        let predicate = DamageSourcePredicateModel::builder()
            .tag(TagPredicateModel::is(id("minecraft:is_projectile")))
            .tag(TagPredicateModel::is_not(id("minecraft:is_fire")))
            .build();

        assert!(predicate.matches(&source()));

        let mut missing_required = source();
        missing_required
            .type_tags
            .remove(&id("minecraft:is_projectile"));
        assert!(!predicate.matches(&missing_required));

        let mut present_forbidden = source();
        present_forbidden.type_tags.insert(id("minecraft:is_fire"));
        assert!(!predicate.matches(&present_forbidden));
    }

    #[test]
    fn damage_source_predicate_checks_direct_source_entities_and_is_direct_like_java() {
        let predicate = DamageSourcePredicateModel::builder()
            .direct(EntityPredicateModel::entity_type(id("minecraft:arrow")))
            .source(EntityPredicateModel::named("Shooter"))
            .with_is_direct(false)
            .build();

        assert!(predicate.matches(&source()));

        let mut no_direct = source();
        no_direct.direct_entity = None;
        assert!(!predicate.matches(&no_direct));

        let mut wrong_source = source();
        wrong_source.source_entity = Some(entity("minecraft:skeleton", Some("Other")));
        assert!(!predicate.matches(&wrong_source));

        let mut direct = source();
        direct.is_direct = true;
        assert!(!predicate.matches(&direct));
    }

    #[test]
    fn damage_source_predicate_builder_preserves_java_fields() {
        let predicate = DamageSourcePredicateModel::builder()
            .tag(TagPredicateModel::is(id("minecraft:is_explosion")))
            .direct(EntityPredicateModel::any())
            .source(EntityPredicateModel::entity_type(id("minecraft:creeper")))
            .with_is_direct(true)
            .build();

        assert_eq!(
            predicate.tags,
            vec![TagPredicateModel::is(id("minecraft:is_explosion"))]
        );
        assert_eq!(predicate.direct_entity, Some(EntityPredicateModel::any()));
        assert_eq!(
            predicate.source_entity,
            Some(EntityPredicateModel::entity_type(id("minecraft:creeper")))
        );
        assert_eq!(predicate.is_direct, Some(true));
    }
}
