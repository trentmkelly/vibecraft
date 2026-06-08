use crate::registry::Identifier;

#[derive(Debug, Clone, PartialEq)]
pub struct DamagePredicateModel {
    pub dealt_damage: DoubleBoundsModel,
    pub taken_damage: DoubleBoundsModel,
    pub source_entity: Option<EntityPredicateModel>,
    pub blocked: Option<bool>,
    pub damage_type: Option<DamageSourcePredicateModel>,
}

impl DamagePredicateModel {
    pub fn new(
        dealt_damage: DoubleBoundsModel,
        taken_damage: DoubleBoundsModel,
        source_entity: Option<EntityPredicateModel>,
        blocked: Option<bool>,
        damage_type: Option<DamageSourcePredicateModel>,
    ) -> Self {
        Self {
            dealt_damage,
            taken_damage,
            source_entity,
            blocked,
            damage_type,
        }
    }

    pub fn builder() -> DamagePredicateBuilder {
        DamagePredicateBuilder::default()
    }

    pub fn matches(
        &self,
        source: &DamageSourceModel,
        original_damage: f64,
        actual_damage: f64,
        blocked: bool,
    ) -> bool {
        if !self.dealt_damage.matches(original_damage) {
            return false;
        }

        if !self.taken_damage.matches(actual_damage) {
            return false;
        }

        if self.source_entity.as_ref().is_some_and(|predicate| {
            !source
                .entity
                .as_ref()
                .is_some_and(|entity| predicate.matches(entity))
        }) {
            return false;
        }

        if self
            .blocked
            .is_some_and(|expected_blocked| expected_blocked != blocked)
        {
            return false;
        }

        self.damage_type
            .as_ref()
            .is_none_or(|damage_type| damage_type.matches(source))
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct DamagePredicateBuilder {
    dealt_damage: DoubleBoundsModel,
    taken_damage: DoubleBoundsModel,
    source_entity: Option<EntityPredicateModel>,
    blocked: Option<bool>,
    damage_type: Option<DamageSourcePredicateModel>,
}

impl DamagePredicateBuilder {
    pub fn damage_instance() -> Self {
        Self::default()
    }

    pub fn dealt_damage(mut self, dealt_damage: DoubleBoundsModel) -> Self {
        self.dealt_damage = dealt_damage;
        self
    }

    pub fn taken_damage(mut self, taken_damage: DoubleBoundsModel) -> Self {
        self.taken_damage = taken_damage;
        self
    }

    pub fn source_entity(mut self, source_entity: EntityPredicateModel) -> Self {
        self.source_entity = Some(source_entity);
        self
    }

    pub fn blocked(mut self, blocked: bool) -> Self {
        self.blocked = Some(blocked);
        self
    }

    pub fn damage_type(mut self, damage_type: DamageSourcePredicateModel) -> Self {
        self.damage_type = Some(damage_type);
        self
    }

    pub fn build(self) -> DamagePredicateModel {
        DamagePredicateModel::new(
            self.dealt_damage,
            self.taken_damage,
            self.source_entity,
            self.blocked,
            self.damage_type,
        )
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DamageSourceModel {
    pub damage_type: Identifier,
    pub direct_entity: Option<EntityContextModel>,
    pub entity: Option<EntityContextModel>,
    pub is_direct: bool,
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

#[derive(Debug, Clone, PartialEq)]
pub struct DamageSourcePredicateModel {
    pub required_type: Option<Identifier>,
    pub direct_entity: Option<EntityPredicateModel>,
    pub source_entity: Option<EntityPredicateModel>,
    pub is_direct: Option<bool>,
}

impl DamageSourcePredicateModel {
    pub fn new(
        required_type: Option<Identifier>,
        direct_entity: Option<EntityPredicateModel>,
        source_entity: Option<EntityPredicateModel>,
        is_direct: Option<bool>,
    ) -> Self {
        Self {
            required_type,
            direct_entity,
            source_entity,
            is_direct,
        }
    }

    fn matches(&self, source: &DamageSourceModel) -> bool {
        if self
            .required_type
            .as_ref()
            .is_some_and(|required_type| required_type != &source.damage_type)
        {
            return false;
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
                .entity
                .as_ref()
                .is_some_and(|entity| predicate.matches(entity))
        }) {
            return false;
        }

        self.is_direct
            .is_none_or(|expected_direct| expected_direct == source.is_direct)
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct DoubleBoundsModel {
    pub min: Option<f64>,
    pub max: Option<f64>,
}

impl DoubleBoundsModel {
    pub fn any() -> Self {
        Self {
            min: None,
            max: None,
        }
    }

    pub fn exactly(value: f64) -> Self {
        Self {
            min: Some(value),
            max: Some(value),
        }
    }

    pub fn between(min: f64, max: f64) -> Self {
        Self {
            min: Some(min),
            max: Some(max),
        }
    }

    pub fn at_least(value: f64) -> Self {
        Self {
            min: Some(value),
            max: None,
        }
    }

    pub fn matches(&self, value: f64) -> bool {
        self.min.is_none_or(|min| min <= value) && self.max.is_none_or(|max| max >= value)
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
            damage_type: id("minecraft:arrow"),
            direct_entity: Some(entity("minecraft:arrow", None)),
            entity: Some(entity("minecraft:skeleton", Some("Shooter"))),
            is_direct: false,
        }
    }

    #[test]
    fn damage_predicate_default_builder_matches_any_damage_like_java() {
        let predicate = DamagePredicateBuilder::damage_instance().build();

        assert!(DoubleBoundsModel::any().matches(99.0));
        assert!(predicate.matches(&source(), 0.0, 0.0, false));
        assert!(predicate.matches(&source(), 12.5, 3.0, true));
    }

    #[test]
    fn damage_predicate_checks_dealt_taken_source_blocked_and_type_like_java() {
        let predicate = DamagePredicateModel::builder()
            .dealt_damage(DoubleBoundsModel::between(5.0, 7.0))
            .taken_damage(DoubleBoundsModel::exactly(2.0))
            .source_entity(EntityPredicateModel::named("Shooter"))
            .blocked(false)
            .damage_type(DamageSourcePredicateModel::new(
                Some(id("minecraft:arrow")),
                Some(EntityPredicateModel::entity_type(id("minecraft:arrow"))),
                Some(EntityPredicateModel::entity_type(id("minecraft:skeleton"))),
                Some(false),
            ))
            .build();

        assert!(predicate.matches(&source(), 6.0, 2.0, false));
        assert!(!predicate.matches(&source(), 4.9, 2.0, false));
        assert!(!predicate.matches(&source(), 6.0, 2.5, false));
        assert!(!predicate.matches(&source(), 6.0, 2.0, true));

        let mut wrong_type = source();
        wrong_type.damage_type = id("minecraft:mob_attack");
        assert!(!predicate.matches(&wrong_type, 6.0, 2.0, false));
    }

    #[test]
    fn damage_predicate_requires_source_entity_when_configured_like_java() {
        let predicate = DamagePredicateModel::builder()
            .source_entity(EntityPredicateModel::entity_type(id("minecraft:skeleton")))
            .build();
        let mut no_source_entity = source();
        no_source_entity.entity = None;

        assert!(predicate.matches(&source(), 1.0, 1.0, false));
        assert!(!predicate.matches(&no_source_entity, 1.0, 1.0, false));
    }

    #[test]
    fn damage_predicate_builder_type_accepts_built_damage_source_predicate() {
        let damage_type =
            DamageSourcePredicateModel::new(Some(id("minecraft:arrow")), None, None, Some(false));
        let predicate = DamagePredicateModel::builder()
            .dealt_damage(DoubleBoundsModel::at_least(1.0))
            .damage_type(damage_type.clone())
            .build();

        assert_eq!(predicate.damage_type, Some(damage_type));
        assert!(predicate.matches(&source(), 1.0, 0.0, false));
        assert!(!predicate.matches(&source(), 0.5, 0.0, false));
    }
}
