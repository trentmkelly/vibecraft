use crate::registry::Identifier;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SummonedEntityTriggerInstanceModel {
    pub player: Option<ContextAwarePredicateModel>,
    pub entity: Option<ContextAwarePredicateModel>,
}

impl SummonedEntityTriggerInstanceModel {
    pub fn new(
        player: Option<ContextAwarePredicateModel>,
        entity: Option<ContextAwarePredicateModel>,
    ) -> Self {
        Self { player, entity }
    }

    pub fn codec_fields() -> [CodecFieldModel; 2] {
        [
            CodecFieldModel::optional("player", "EntityPredicate.ADVANCEMENT_CODEC"),
            CodecFieldModel::optional("entity", "EntityPredicate.ADVANCEMENT_CODEC"),
        ]
    }

    pub fn summoned_entity(predicate: EntityPredicateBuilderModel) -> SummonedEntityCriterionModel {
        SummonedEntityCriterionModel {
            trigger_id: id("minecraft:summoned_entity"),
            instance: Self::new(
                None,
                Some(ContextAwarePredicateModel::wrap_builder(predicate)),
            ),
        }
    }

    pub fn matches(&self, entity: &LootContextModel) -> bool {
        self.entity
            .as_ref()
            .is_none_or(|predicate| predicate.matches(entity))
    }

    pub fn validate(&self) -> Vec<String> {
        let mut problems = Vec::new();
        if let Some(problem) = self
            .player
            .as_ref()
            .and_then(ContextAwarePredicateModel::validation_problem)
        {
            problems.push(format!("player: {problem}"));
        }
        if let Some(problem) = self
            .entity
            .as_ref()
            .and_then(ContextAwarePredicateModel::validation_problem)
        {
            problems.push(format!("entity: {problem}"));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SummonedEntityTriggerModel {
    listeners: Vec<SummonedEntityTriggerInstanceModel>,
}

impl SummonedEntityTriggerModel {
    pub fn new(listeners: impl IntoIterator<Item = SummonedEntityTriggerInstanceModel>) -> Self {
        Self {
            listeners: listeners.into_iter().collect(),
        }
    }

    pub fn codec_type() -> &'static str {
        "SummonedEntityTrigger.TriggerInstance.CODEC"
    }

    pub fn trigger(
        &self,
        player: &ServerPlayerModel,
        entity: EntityContextModel,
    ) -> Vec<&SummonedEntityTriggerInstanceModel> {
        let context = EntityPredicateModel::create_context(player, entity);
        self.listeners
            .iter()
            .filter(|listener| listener.matches(&context))
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SummonedEntityCriterionModel {
    pub trigger_id: Identifier,
    pub instance: SummonedEntityTriggerInstanceModel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CodecFieldModel {
    pub name: &'static str,
    pub codec: &'static str,
    pub optional: bool,
}

impl CodecFieldModel {
    pub fn optional(name: &'static str, codec: &'static str) -> Self {
        Self {
            name,
            codec,
            optional: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerPlayerModel {
    entity_type: Identifier,
}

impl ServerPlayerModel {
    pub fn new(entity_type: Identifier) -> Self {
        Self { entity_type }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntityContextModel {
    entity_type: Identifier,
}

impl EntityContextModel {
    pub fn new(entity_type: Identifier) -> Self {
        Self { entity_type }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LootContextModel {
    player_type: Identifier,
    entity_type: Identifier,
}

pub struct EntityPredicateModel;

impl EntityPredicateModel {
    fn create_context(player: &ServerPlayerModel, entity: EntityContextModel) -> LootContextModel {
        LootContextModel {
            player_type: player.entity_type.clone(),
            entity_type: entity.entity_type,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EntityPredicateBuilderModel {
    entity_type: Option<Identifier>,
}

impl EntityPredicateBuilderModel {
    pub fn entity() -> Self {
        Self::default()
    }

    pub fn of(mut self, entity_type: Identifier) -> Self {
        self.entity_type = Some(entity_type);
        self
    }

    fn build(self) -> EntityPredicateDataModel {
        EntityPredicateDataModel {
            entity_type: self.entity_type,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntityPredicateDataModel {
    entity_type: Option<Identifier>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextAwarePredicateModel {
    required_entity_type: Option<Identifier>,
    validation_problem: Option<String>,
}

impl ContextAwarePredicateModel {
    pub fn entity_type(entity_type: Identifier) -> Self {
        Self {
            required_entity_type: Some(entity_type),
            validation_problem: None,
        }
    }

    pub fn invalid(problem: &str) -> Self {
        Self {
            required_entity_type: None,
            validation_problem: Some(problem.to_string()),
        }
    }

    fn wrap_builder(builder: EntityPredicateBuilderModel) -> Self {
        let predicate = builder.build();
        Self {
            required_entity_type: predicate.entity_type,
            validation_problem: None,
        }
    }

    fn matches(&self, context: &LootContextModel) -> bool {
        self.required_entity_type
            .as_ref()
            .is_none_or(|required| &context.entity_type == required)
    }

    fn validation_problem(&self) -> Option<&str> {
        self.validation_problem.as_deref()
    }
}

fn id(value: &str) -> Identifier {
    Identifier::parse(value).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn player() -> ServerPlayerModel {
        ServerPlayerModel::new(id("minecraft:player"))
    }

    fn entity(entity_type: &str) -> EntityContextModel {
        EntityContextModel::new(id(entity_type))
    }

    #[test]
    fn summoned_entity_trigger_uses_trigger_instance_codec() {
        assert_eq!(
            SummonedEntityTriggerModel::codec_type(),
            "SummonedEntityTrigger.TriggerInstance.CODEC"
        );
    }

    #[test]
    fn summoned_entity_codec_fields_match_java_record_codec() {
        assert_eq!(
            SummonedEntityTriggerInstanceModel::codec_fields(),
            [
                CodecFieldModel::optional("player", "EntityPredicate.ADVANCEMENT_CODEC"),
                CodecFieldModel::optional("entity", "EntityPredicate.ADVANCEMENT_CODEC"),
            ]
        );
    }

    #[test]
    fn summoned_entity_factory_wraps_entity_predicate_and_uses_trigger_id() {
        let criterion = SummonedEntityTriggerInstanceModel::summoned_entity(
            EntityPredicateBuilderModel::entity().of(id("minecraft:iron_golem")),
        );

        assert_eq!(criterion.trigger_id, id("minecraft:summoned_entity"));
        assert!(criterion.instance.player.is_none());
        assert_eq!(
            criterion.instance.entity,
            Some(ContextAwarePredicateModel::entity_type(id(
                "minecraft:iron_golem"
            )))
        );
    }

    #[test]
    fn omitted_entity_predicate_matches_any_summoned_entity_context() {
        let instance = SummonedEntityTriggerInstanceModel::new(None, None);
        let context = EntityPredicateModel::create_context(&player(), entity("minecraft:wither"));

        assert!(instance.matches(&context));
    }

    #[test]
    fn entity_predicate_matches_created_loot_context() {
        let instance = SummonedEntityTriggerInstanceModel::new(
            None,
            Some(ContextAwarePredicateModel::entity_type(id(
                "minecraft:iron_golem",
            ))),
        );
        let trigger = SummonedEntityTriggerModel::new([instance.clone()]);

        assert_eq!(
            trigger.trigger(&player(), entity("minecraft:iron_golem")),
            vec![&instance]
        );
        assert!(trigger
            .trigger(&player(), entity("minecraft:snow_golem"))
            .is_empty());
    }

    #[test]
    fn trigger_context_is_created_from_player_and_summoned_entity() {
        let context = EntityPredicateModel::create_context(&player(), entity("minecraft:allay"));

        assert_eq!(context.player_type, id("minecraft:player"));
        assert_eq!(context.entity_type, id("minecraft:allay"));
    }

    #[test]
    fn validation_runs_inherited_player_label_and_entity_label() {
        let instance = SummonedEntityTriggerInstanceModel::new(
            Some(ContextAwarePredicateModel::invalid("bad player")),
            Some(ContextAwarePredicateModel::invalid("bad entity")),
        );

        assert_eq!(
            instance.validate(),
            vec![
                "player: bad player".to_string(),
                "entity: bad entity".to_string(),
            ]
        );
    }
}
