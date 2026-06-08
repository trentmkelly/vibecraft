use crate::registry::Identifier;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LightningStrikeTriggerInstanceModel {
    pub player_predicate_present: bool,
    pub lightning: Option<ContextAwareEntityPredicateModel>,
    pub bystander: Option<ContextAwareEntityPredicateModel>,
}

impl LightningStrikeTriggerInstanceModel {
    pub fn new(
        lightning: Option<ContextAwareEntityPredicateModel>,
        bystander: Option<ContextAwareEntityPredicateModel>,
    ) -> Self {
        Self {
            player_predicate_present: false,
            lightning,
            bystander,
        }
    }

    pub fn lightning_strike(
        lightning: Option<EntityPredicateModel>,
        bystander: Option<EntityPredicateModel>,
    ) -> LightningStrikeCriterionModel {
        LightningStrikeCriterionModel {
            trigger_id: trigger_id(),
            instance: Self::new(
                lightning.map(ContextAwareEntityPredicateModel::wrap),
                bystander.map(ContextAwareEntityPredicateModel::wrap),
            ),
        }
    }

    pub fn matches(
        &self,
        bolt: &EntityLootContextModel,
        entities_around: &[EntityLootContextModel],
    ) -> bool {
        if self
            .lightning
            .as_ref()
            .is_some_and(|predicate| !predicate.matches(bolt))
        {
            return false;
        }

        self.bystander.as_ref().is_none_or(|predicate| {
            entities_around
                .iter()
                .any(|entity| predicate.matches(entity))
        })
    }

    pub fn validate(&self) -> Vec<String> {
        let mut problems = Vec::new();
        if let Some(problem) = self
            .lightning
            .as_ref()
            .and_then(ContextAwareEntityPredicateModel::validation_problem)
        {
            problems.push(format!("lightning: {problem}"));
        }
        if let Some(problem) = self
            .bystander
            .as_ref()
            .and_then(ContextAwareEntityPredicateModel::validation_problem)
        {
            problems.push(format!("bystander: {problem}"));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LightningStrikeCriterionModel {
    pub trigger_id: Identifier,
    pub instance: LightningStrikeTriggerInstanceModel,
}

fn trigger_id() -> Identifier {
    id("minecraft:lightning_strike")
}

fn id(value: &str) -> Identifier {
    Identifier::parse(value).unwrap()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntityLootContextModel {
    entity_type: Identifier,
    name: Option<String>,
}

impl EntityLootContextModel {
    pub fn new(entity_type: Identifier, name: Option<String>) -> Self {
        Self { entity_type, name }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntityPredicateModel {
    required_type: Option<Identifier>,
    required_name: Option<String>,
    validation_problem: Option<String>,
}

impl EntityPredicateModel {
    pub fn any() -> Self {
        Self {
            required_type: None,
            required_name: None,
            validation_problem: None,
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

    pub fn invalid(problem: &str) -> Self {
        Self {
            validation_problem: Some(problem.to_string()),
            ..Self::any()
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextAwareEntityPredicateModel {
    required_type: Option<Identifier>,
    required_name: Option<String>,
    validation_problem: Option<String>,
}

impl ContextAwareEntityPredicateModel {
    pub fn wrap(predicate: EntityPredicateModel) -> Self {
        Self {
            required_type: predicate.required_type,
            required_name: predicate.required_name,
            validation_problem: predicate.validation_problem,
        }
    }

    fn matches(&self, entity: &EntityLootContextModel) -> bool {
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

    fn validation_problem(&self) -> Option<&str> {
        self.validation_problem.as_deref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn context(entity_type: &str, name: Option<&str>) -> EntityLootContextModel {
        EntityLootContextModel::new(id(entity_type), name.map(str::to_string))
    }

    fn bolt() -> EntityLootContextModel {
        context("minecraft:lightning_bolt", None)
    }

    #[test]
    fn omitted_lightning_and_bystander_predicates_match_any_strike() {
        let instance = LightningStrikeTriggerInstanceModel::new(None, None);

        assert!(instance.matches(&bolt(), &[]));
        assert!(instance.matches(&bolt(), &[context("minecraft:villager", None)]));
    }

    #[test]
    fn lightning_predicate_is_checked_before_bystander_predicate() {
        let instance = LightningStrikeTriggerInstanceModel::new(
            Some(ContextAwareEntityPredicateModel::wrap(
                EntityPredicateModel::entity_type(id("minecraft:lightning_bolt")),
            )),
            Some(ContextAwareEntityPredicateModel::wrap(
                EntityPredicateModel::entity_type(id("minecraft:villager")),
            )),
        );

        assert!(instance.matches(&bolt(), &[context("minecraft:villager", None)]));
        assert!(!instance.matches(
            &context("minecraft:zombie", None),
            &[context("minecraft:villager", None)]
        ));
        assert!(!instance.matches(&bolt(), &[context("minecraft:pig", None)]));
    }

    #[test]
    fn bystander_predicate_requires_any_nearby_entity_match() {
        let instance = LightningStrikeTriggerInstanceModel::new(
            None,
            Some(ContextAwareEntityPredicateModel::wrap(
                EntityPredicateModel::named("Target"),
            )),
        );

        assert!(instance.matches(
            &bolt(),
            &[
                context("minecraft:pig", None),
                context("minecraft:villager", Some("Target")),
            ],
        ));
        assert!(!instance.matches(&bolt(), &[context("minecraft:villager", Some("Other"))],));
        assert!(!instance.matches(&bolt(), &[]));
    }

    #[test]
    fn lightning_strike_factory_wraps_optional_predicates_and_uses_trigger_id() {
        let criterion = LightningStrikeTriggerInstanceModel::lightning_strike(
            Some(EntityPredicateModel::entity_type(id(
                "minecraft:lightning_bolt",
            ))),
            Some(EntityPredicateModel::entity_type(id("minecraft:villager"))),
        );

        assert_eq!(criterion.trigger_id, id("minecraft:lightning_strike"));
        assert!(!criterion.instance.player_predicate_present);
        assert!(criterion.instance.lightning.is_some());
        assert!(criterion.instance.bystander.is_some());
        assert!(criterion
            .instance
            .matches(&bolt(), &[context("minecraft:villager", None)]));
    }

    #[test]
    fn validation_reports_lightning_and_bystander_labels() {
        let instance = LightningStrikeTriggerInstanceModel::new(
            Some(ContextAwareEntityPredicateModel::wrap(
                EntityPredicateModel::invalid("bad lightning"),
            )),
            Some(ContextAwareEntityPredicateModel::wrap(
                EntityPredicateModel::invalid("bad bystander"),
            )),
        );

        assert_eq!(
            instance.validate(),
            vec![
                "lightning: bad lightning".to_string(),
                "bystander: bad bystander".to_string(),
            ]
        );
    }
}
