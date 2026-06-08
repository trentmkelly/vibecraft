use crate::registry::Identifier;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StartRidingTriggerInstanceModel {
    pub player: Option<ContextAwarePredicateModel>,
}

impl StartRidingTriggerInstanceModel {
    pub fn new(player: Option<ContextAwarePredicateModel>) -> Self {
        Self { player }
    }

    pub fn codec_field_names() -> [&'static str; 1] {
        ["player"]
    }

    pub fn codec_field_codec() -> &'static str {
        "EntityPredicate.ADVANCEMENT_CODEC"
    }

    pub fn player_starts_riding(player: EntityPredicateBuilderModel) -> StartRidingCriterionModel {
        StartRidingCriterionModel {
            trigger_id: id("minecraft:started_riding"),
            instance: Self::new(Some(ContextAwarePredicateModel::wrap_builder(player))),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StartRidingTriggerModel {
    listeners: Vec<StartRidingTriggerInstanceModel>,
}

impl StartRidingTriggerModel {
    pub fn new(listeners: impl IntoIterator<Item = StartRidingTriggerInstanceModel>) -> Self {
        Self {
            listeners: listeners.into_iter().collect(),
        }
    }

    pub fn codec_type() -> &'static str {
        "StartRidingTrigger.TriggerInstance.CODEC"
    }

    pub fn trigger(&self, _player: &ServerPlayerModel) -> Vec<&StartRidingTriggerInstanceModel> {
        self.listeners.iter().collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StartRidingCriterionModel {
    pub trigger_id: Identifier,
    pub instance: StartRidingTriggerInstanceModel,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextAwarePredicateModel {
    entity: EntityPredicateModel,
}

impl ContextAwarePredicateModel {
    pub fn wrap_builder(builder: EntityPredicateBuilderModel) -> Self {
        Self {
            entity: builder.build(),
        }
    }

    pub fn entity(&self) -> &EntityPredicateModel {
        &self.entity
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EntityPredicateBuilderModel {
    entity_type: Option<Identifier>,
    team: Option<String>,
}

impl EntityPredicateBuilderModel {
    pub fn entity() -> Self {
        Self::default()
    }

    pub fn of(mut self, entity_type: Identifier) -> Self {
        self.entity_type = Some(entity_type);
        self
    }

    pub fn team(mut self, team: &str) -> Self {
        self.team = Some(team.to_string());
        self
    }

    fn build(self) -> EntityPredicateModel {
        EntityPredicateModel {
            entity_type: self.entity_type,
            team: self.team,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntityPredicateModel {
    entity_type: Option<Identifier>,
    team: Option<String>,
}

impl EntityPredicateModel {
    pub fn entity_type(&self) -> Option<&Identifier> {
        self.entity_type.as_ref()
    }

    pub fn team(&self) -> Option<&str> {
        self.team.as_deref()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerPlayerModel {
    pub name: String,
}

impl ServerPlayerModel {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
        }
    }
}

fn id(value: &str) -> Identifier {
    Identifier::parse(value).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn start_riding_trigger_uses_trigger_instance_codec() {
        assert_eq!(
            StartRidingTriggerModel::codec_type(),
            "StartRidingTrigger.TriggerInstance.CODEC"
        );
    }

    #[test]
    fn start_riding_codec_has_optional_player_advancement_predicate_field() {
        assert_eq!(
            StartRidingTriggerInstanceModel::codec_field_names(),
            ["player"]
        );
        assert_eq!(
            StartRidingTriggerInstanceModel::codec_field_codec(),
            "EntityPredicate.ADVANCEMENT_CODEC"
        );
    }

    #[test]
    fn player_starts_riding_factory_wraps_built_player_predicate() {
        let criterion = StartRidingTriggerInstanceModel::player_starts_riding(
            EntityPredicateBuilderModel::entity()
                .of(id("minecraft:player"))
                .team("red"),
        );

        assert_eq!(criterion.trigger_id, id("minecraft:started_riding"));
        let player = criterion.instance.player.as_ref().unwrap();
        assert_eq!(player.entity().entity_type(), Some(&id("minecraft:player")));
        assert_eq!(player.entity().team(), Some("red"));
    }

    #[test]
    fn trigger_forwards_all_instances_with_unconditional_matcher() {
        let without_player = StartRidingTriggerInstanceModel::new(None);
        let with_player =
            StartRidingTriggerInstanceModel::new(Some(ContextAwarePredicateModel::wrap_builder(
                EntityPredicateBuilderModel::entity().of(id("minecraft:player")),
            )));
        let trigger = StartRidingTriggerModel::new([without_player.clone(), with_player.clone()]);
        let player = ServerPlayerModel::new("Alex");

        assert_eq!(
            trigger.trigger(&player),
            vec![&without_player, &with_player]
        );
    }
}
