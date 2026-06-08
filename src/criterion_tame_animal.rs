use crate::registry::Identifier;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TameAnimalTriggerInstanceModel {
    pub player: Option<ContextAwarePredicateModel>,
    pub entity: Option<ContextAwarePredicateModel>,
}

impl TameAnimalTriggerInstanceModel {
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

    pub fn tamed_animal() -> TameAnimalCriterionModel {
        TameAnimalCriterionModel {
            trigger_id: id("minecraft:tame_animal"),
            instance: Self::new(None, None),
        }
    }

    pub fn tamed_animal_entity(entity: EntityPredicateBuilderModel) -> TameAnimalCriterionModel {
        TameAnimalCriterionModel {
            trigger_id: id("minecraft:tame_animal"),
            instance: Self::new(None, Some(ContextAwarePredicateModel::wrap_builder(entity))),
        }
    }

    pub fn matches(&self, animal: &LootContextModel) -> bool {
        self.entity
            .as_ref()
            .is_none_or(|predicate| predicate.matches(animal))
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
pub struct TameAnimalTriggerModel {
    listeners: Vec<TameAnimalTriggerInstanceModel>,
}

impl TameAnimalTriggerModel {
    pub fn new(listeners: impl IntoIterator<Item = TameAnimalTriggerInstanceModel>) -> Self {
        Self {
            listeners: listeners.into_iter().collect(),
        }
    }

    pub fn codec_type() -> &'static str {
        "TameAnimalTrigger.TriggerInstance.CODEC"
    }

    pub fn trigger(
        &self,
        player: &ServerPlayerModel,
        animal: AnimalModel,
    ) -> Vec<&TameAnimalTriggerInstanceModel> {
        let animal_context = EntityPredicateModel::create_context(player, animal);
        self.listeners
            .iter()
            .filter(|listener| listener.matches(&animal_context))
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TameAnimalCriterionModel {
    pub trigger_id: Identifier,
    pub instance: TameAnimalTriggerInstanceModel,
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
pub struct AnimalModel {
    entity_type: Identifier,
}

impl AnimalModel {
    pub fn new(entity_type: Identifier) -> Self {
        Self { entity_type }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LootContextModel {
    player_type: Identifier,
    animal_type: Identifier,
}

pub struct EntityPredicateModel;

impl EntityPredicateModel {
    fn create_context(player: &ServerPlayerModel, animal: AnimalModel) -> LootContextModel {
        LootContextModel {
            player_type: player.entity_type.clone(),
            animal_type: animal.entity_type,
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
            .is_none_or(|required| &context.animal_type == required)
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

    fn animal(entity_type: &str) -> AnimalModel {
        AnimalModel::new(id(entity_type))
    }

    #[test]
    fn tame_animal_trigger_uses_trigger_instance_codec() {
        assert_eq!(
            TameAnimalTriggerModel::codec_type(),
            "TameAnimalTrigger.TriggerInstance.CODEC"
        );
    }

    #[test]
    fn tame_animal_codec_fields_match_java_record_codec() {
        assert_eq!(
            TameAnimalTriggerInstanceModel::codec_fields(),
            [
                CodecFieldModel::optional("player", "EntityPredicate.ADVANCEMENT_CODEC"),
                CodecFieldModel::optional("entity", "EntityPredicate.ADVANCEMENT_CODEC"),
            ]
        );
    }

    #[test]
    fn tamed_animal_no_arg_factory_uses_empty_player_and_entity() {
        let criterion = TameAnimalTriggerInstanceModel::tamed_animal();

        assert_eq!(criterion.trigger_id, id("minecraft:tame_animal"));
        assert!(criterion.instance.player.is_none());
        assert!(criterion.instance.entity.is_none());
    }

    #[test]
    fn tamed_animal_entity_factory_wraps_builder_under_same_trigger_id() {
        let criterion = TameAnimalTriggerInstanceModel::tamed_animal_entity(
            EntityPredicateBuilderModel::entity().of(id("minecraft:wolf")),
        );

        assert_eq!(criterion.trigger_id, id("minecraft:tame_animal"));
        assert!(criterion.instance.player.is_none());
        assert_eq!(
            criterion.instance.entity,
            Some(ContextAwarePredicateModel::entity_type(id(
                "minecraft:wolf"
            )))
        );
    }

    #[test]
    fn omitted_entity_predicate_matches_any_tamed_animal_context() {
        let instance = TameAnimalTriggerInstanceModel::new(None, None);
        let context = EntityPredicateModel::create_context(&player(), animal("minecraft:cat"));

        assert!(instance.matches(&context));
    }

    #[test]
    fn entity_predicate_matches_created_animal_loot_context() {
        let instance = TameAnimalTriggerInstanceModel::new(
            None,
            Some(ContextAwarePredicateModel::entity_type(id(
                "minecraft:wolf",
            ))),
        );
        let trigger = TameAnimalTriggerModel::new([instance.clone()]);

        assert_eq!(
            trigger.trigger(&player(), animal("minecraft:wolf")),
            vec![&instance]
        );
        assert!(trigger
            .trigger(&player(), animal("minecraft:cat"))
            .is_empty());
    }

    #[test]
    fn trigger_context_is_created_from_player_and_animal() {
        let context = EntityPredicateModel::create_context(&player(), animal("minecraft:parrot"));

        assert_eq!(context.player_type, id("minecraft:player"));
        assert_eq!(context.animal_type, id("minecraft:parrot"));
    }

    #[test]
    fn validation_runs_inherited_player_label_and_entity_label() {
        let instance = TameAnimalTriggerInstanceModel::new(
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
