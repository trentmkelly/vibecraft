use crate::registry::Identifier;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KilledTriggerInstanceModel {
    pub player_predicate_present: bool,
    pub entity: Option<ContextAwareEntityPredicateModel>,
    pub killing_blow: Option<DamageSourcePredicateModel>,
}

impl KilledTriggerInstanceModel {
    pub fn new(
        player_predicate_present: bool,
        entity: Option<ContextAwareEntityPredicateModel>,
        killing_blow: Option<DamageSourcePredicateModel>,
    ) -> Self {
        Self {
            player_predicate_present,
            entity,
            killing_blow,
        }
    }

    pub fn player_killed_entity(entity: Option<EntityPredicateModel>) -> KilledCriterionModel {
        Self::player_killed_entity_with_damage(entity, None)
    }

    pub fn player_killed_entity_builder(
        entity: EntityPredicateBuilderModel,
    ) -> KilledCriterionModel {
        Self::player_killed_entity(Some(entity.build()))
    }

    pub fn player_killed_entity_any() -> KilledCriterionModel {
        Self::criterion("minecraft:player_killed_entity", None, None)
    }

    pub fn player_killed_entity_with_damage(
        entity: Option<EntityPredicateModel>,
        killing_blow: Option<DamageSourcePredicateModel>,
    ) -> KilledCriterionModel {
        Self::criterion(
            "minecraft:player_killed_entity",
            entity.map(ContextAwareEntityPredicateModel::wrap),
            killing_blow,
        )
    }

    pub fn player_killed_entity_builder_with_damage(
        entity: EntityPredicateBuilderModel,
        killing_blow: Option<DamageSourcePredicateModel>,
    ) -> KilledCriterionModel {
        Self::player_killed_entity_with_damage(Some(entity.build()), killing_blow)
    }

    pub fn player_killed_entity_with_damage_builder(
        entity: Option<EntityPredicateModel>,
        killing_blow: DamageSourcePredicateBuilderModel,
    ) -> KilledCriterionModel {
        Self::player_killed_entity_with_damage(entity, Some(killing_blow.build()))
    }

    pub fn player_killed_entity_builder_with_damage_builder(
        entity: EntityPredicateBuilderModel,
        killing_blow: DamageSourcePredicateBuilderModel,
    ) -> KilledCriterionModel {
        Self::player_killed_entity_with_damage(Some(entity.build()), Some(killing_blow.build()))
    }

    pub fn player_killed_entity_near_sculk_catalyst() -> KilledCriterionModel {
        Self::criterion("minecraft:kill_mob_near_sculk_catalyst", None, None)
    }

    pub fn entity_killed_player(entity: Option<EntityPredicateModel>) -> KilledCriterionModel {
        Self::entity_killed_player_with_damage(entity, None)
    }

    pub fn entity_killed_player_builder(
        entity: EntityPredicateBuilderModel,
    ) -> KilledCriterionModel {
        Self::entity_killed_player(Some(entity.build()))
    }

    pub fn entity_killed_player_any() -> KilledCriterionModel {
        Self::criterion("minecraft:entity_killed_player", None, None)
    }

    pub fn entity_killed_player_with_damage(
        entity: Option<EntityPredicateModel>,
        killing_blow: Option<DamageSourcePredicateModel>,
    ) -> KilledCriterionModel {
        Self::criterion(
            "minecraft:entity_killed_player",
            entity.map(ContextAwareEntityPredicateModel::wrap),
            killing_blow,
        )
    }

    pub fn entity_killed_player_builder_with_damage(
        entity: EntityPredicateBuilderModel,
        killing_blow: Option<DamageSourcePredicateModel>,
    ) -> KilledCriterionModel {
        Self::entity_killed_player_with_damage(Some(entity.build()), killing_blow)
    }

    pub fn entity_killed_player_with_damage_builder(
        entity: Option<EntityPredicateModel>,
        killing_blow: DamageSourcePredicateBuilderModel,
    ) -> KilledCriterionModel {
        Self::entity_killed_player_with_damage(entity, Some(killing_blow.build()))
    }

    pub fn entity_killed_player_builder_with_damage_builder(
        entity: EntityPredicateBuilderModel,
        killing_blow: DamageSourcePredicateBuilderModel,
    ) -> KilledCriterionModel {
        Self::entity_killed_player_with_damage(Some(entity.build()), Some(killing_blow.build()))
    }

    fn criterion(
        trigger: &str,
        entity: Option<ContextAwareEntityPredicateModel>,
        killing_blow: Option<DamageSourcePredicateModel>,
    ) -> KilledCriterionModel {
        KilledCriterionModel {
            trigger_id: id(trigger),
            instance: Self::new(false, entity, killing_blow),
        }
    }

    pub fn matches(
        &self,
        player: &PlayerModel,
        entity: &EntityLootContextModel,
        killing_blow: &DamageSourceModel,
    ) -> bool {
        if self
            .killing_blow
            .as_ref()
            .is_some_and(|predicate| !predicate.matches(player, killing_blow))
        {
            return false;
        }

        self.entity
            .as_ref()
            .is_none_or(|predicate| predicate.matches(entity))
    }

    pub fn validate(&self) -> Vec<String> {
        self.entity
            .as_ref()
            .and_then(ContextAwareEntityPredicateModel::validation_problem)
            .map(|problem| vec![format!("entity: {problem}")])
            .unwrap_or_default()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KilledCriterionModel {
    pub trigger_id: Identifier,
    pub instance: KilledTriggerInstanceModel,
}

fn id(value: &str) -> Identifier {
    Identifier::parse(value).unwrap()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerModel {
    entity_type: Identifier,
}

impl PlayerModel {
    pub fn server_player() -> Self {
        Self {
            entity_type: id("minecraft:player"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntityLootContextModel {
    entity_type: Identifier,
}

impl EntityLootContextModel {
    pub fn entity_type(entity_type: Identifier) -> Self {
        Self { entity_type }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntityPredicateModel {
    required_type: Option<Identifier>,
    validation_problem: Option<String>,
}

impl EntityPredicateModel {
    pub fn any() -> Self {
        Self {
            required_type: None,
            validation_problem: None,
        }
    }

    pub fn entity_type(entity_type: Identifier) -> Self {
        Self {
            required_type: Some(entity_type),
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
pub struct EntityPredicateBuilderModel {
    predicate: EntityPredicateModel,
}

impl EntityPredicateBuilderModel {
    pub fn entity_type(entity_type: Identifier) -> Self {
        Self {
            predicate: EntityPredicateModel::entity_type(entity_type),
        }
    }

    pub fn build(self) -> EntityPredicateModel {
        self.predicate
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextAwareEntityPredicateModel {
    required_type: Option<Identifier>,
    validation_problem: Option<String>,
}

impl ContextAwareEntityPredicateModel {
    pub fn wrap(predicate: EntityPredicateModel) -> Self {
        Self {
            required_type: predicate.required_type,
            validation_problem: predicate.validation_problem,
        }
    }

    fn matches(&self, entity: &EntityLootContextModel) -> bool {
        self.required_type
            .as_ref()
            .is_none_or(|entity_type| entity_type == &entity.entity_type)
    }

    fn validation_problem(&self) -> Option<&str> {
        self.validation_problem.as_deref()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DamageSourceModel {
    msg_id: String,
    direct_entity_type: Option<Identifier>,
}

impl DamageSourceModel {
    pub fn new(msg_id: &str, direct_entity_type: Option<Identifier>) -> Self {
        Self {
            msg_id: msg_id.to_string(),
            direct_entity_type,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DamageSourcePredicateModel {
    msg_id: Option<String>,
    direct_entity_type: Option<Identifier>,
    require_player_source: bool,
}

impl DamageSourcePredicateModel {
    pub fn matches(&self, player: &PlayerModel, source: &DamageSourceModel) -> bool {
        self.msg_id
            .as_ref()
            .is_none_or(|msg_id| msg_id == &source.msg_id)
            && self.direct_entity_type.as_ref().is_none_or(|entity_type| {
                source
                    .direct_entity_type
                    .as_ref()
                    .is_some_and(|source_type| source_type == entity_type)
            })
            && (!self.require_player_source
                || source.direct_entity_type.as_ref() == Some(&player.entity_type))
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DamageSourcePredicateBuilderModel {
    msg_id: Option<String>,
    direct_entity_type: Option<Identifier>,
    require_player_source: bool,
}

impl DamageSourcePredicateBuilderModel {
    pub fn msg_id(mut self, msg_id: &str) -> Self {
        self.msg_id = Some(msg_id.to_string());
        self
    }

    pub fn direct_entity_type(mut self, entity_type: Identifier) -> Self {
        self.direct_entity_type = Some(entity_type);
        self
    }

    pub fn player_source(mut self) -> Self {
        self.require_player_source = true;
        self
    }

    pub fn build(self) -> DamageSourcePredicateModel {
        DamageSourcePredicateModel {
            msg_id: self.msg_id,
            direct_entity_type: self.direct_entity_type,
            require_player_source: self.require_player_source,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entity(entity_type: &str) -> EntityLootContextModel {
        EntityLootContextModel::entity_type(id(entity_type))
    }

    fn damage(msg_id: &str, direct_entity_type: Option<&str>) -> DamageSourceModel {
        DamageSourceModel::new(msg_id, direct_entity_type.map(id))
    }

    #[test]
    fn omitted_predicates_match_any_kill_event() {
        let instance = KilledTriggerInstanceModel::new(false, None, None);

        assert!(instance.matches(
            &PlayerModel::server_player(),
            &entity("minecraft:zombie"),
            &damage("player", Some("minecraft:player")),
        ));
    }

    #[test]
    fn killing_blow_predicate_is_checked_before_entity_predicate() {
        let instance = KilledTriggerInstanceModel::new(
            false,
            Some(ContextAwareEntityPredicateModel::wrap(
                EntityPredicateModel::entity_type(id("minecraft:zombie")),
            )),
            Some(
                DamageSourcePredicateBuilderModel::default()
                    .msg_id("arrow")
                    .build(),
            ),
        );

        assert!(instance.matches(
            &PlayerModel::server_player(),
            &entity("minecraft:zombie"),
            &damage("arrow", Some("minecraft:skeleton")),
        ));
        assert!(!instance.matches(
            &PlayerModel::server_player(),
            &entity("minecraft:zombie"),
            &damage("player", Some("minecraft:player")),
        ));
        assert!(!instance.matches(
            &PlayerModel::server_player(),
            &entity("minecraft:skeleton"),
            &damage("arrow", Some("minecraft:skeleton")),
        ));
    }

    #[test]
    fn damage_source_predicate_receives_player_context() {
        let instance = KilledTriggerInstanceModel::new(
            false,
            None,
            Some(
                DamageSourcePredicateBuilderModel::default()
                    .player_source()
                    .build(),
            ),
        );

        assert!(instance.matches(
            &PlayerModel::server_player(),
            &entity("minecraft:zombie"),
            &damage("player", Some("minecraft:player")),
        ));
        assert!(!instance.matches(
            &PlayerModel::server_player(),
            &entity("minecraft:zombie"),
            &damage("mob", Some("minecraft:zombie")),
        ));
    }

    #[test]
    fn player_killed_entity_factories_use_java_trigger_id_and_wrapped_entity() {
        let any = KilledTriggerInstanceModel::player_killed_entity_any();
        assert_eq!(any.trigger_id, id("minecraft:player_killed_entity"));
        assert_eq!(
            any.instance,
            KilledTriggerInstanceModel::new(false, None, None)
        );

        let optional = KilledTriggerInstanceModel::player_killed_entity(Some(
            EntityPredicateModel::entity_type(id("minecraft:creeper")),
        ));
        assert!(optional.instance.matches(
            &PlayerModel::server_player(),
            &entity("minecraft:creeper"),
            &damage("player", Some("minecraft:player")),
        ));

        let builder = KilledTriggerInstanceModel::player_killed_entity_builder(
            EntityPredicateBuilderModel::entity_type(id("minecraft:skeleton")),
        );
        assert!(builder.instance.matches(
            &PlayerModel::server_player(),
            &entity("minecraft:skeleton"),
            &damage("player", Some("minecraft:player")),
        ));
    }

    #[test]
    fn player_killed_entity_damage_overloads_build_and_store_damage_predicates() {
        let optional_damage = KilledTriggerInstanceModel::player_killed_entity_with_damage(
            Some(EntityPredicateModel::entity_type(id("minecraft:zombie"))),
            Some(
                DamageSourcePredicateBuilderModel::default()
                    .direct_entity_type(id("minecraft:player"))
                    .build(),
            ),
        );
        assert!(optional_damage.instance.matches(
            &PlayerModel::server_player(),
            &entity("minecraft:zombie"),
            &damage("player", Some("minecraft:player")),
        ));

        let builder_damage =
            KilledTriggerInstanceModel::player_killed_entity_builder_with_damage_builder(
                EntityPredicateBuilderModel::entity_type(id("minecraft:zombie")),
                DamageSourcePredicateBuilderModel::default().msg_id("arrow"),
            );
        assert!(builder_damage.instance.matches(
            &PlayerModel::server_player(),
            &entity("minecraft:zombie"),
            &damage("arrow", Some("minecraft:skeleton")),
        ));

        let entity_builder_optional_damage =
            KilledTriggerInstanceModel::player_killed_entity_builder_with_damage(
                EntityPredicateBuilderModel::entity_type(id("minecraft:zombie")),
                Some(
                    DamageSourcePredicateBuilderModel::default()
                        .msg_id("player")
                        .build(),
                ),
            );
        assert!(entity_builder_optional_damage.instance.matches(
            &PlayerModel::server_player(),
            &entity("minecraft:zombie"),
            &damage("player", Some("minecraft:player")),
        ));

        let optional_entity_damage_builder =
            KilledTriggerInstanceModel::player_killed_entity_with_damage_builder(
                Some(EntityPredicateModel::entity_type(id("minecraft:zombie"))),
                DamageSourcePredicateBuilderModel::default().msg_id("trident"),
            );
        assert!(optional_entity_damage_builder.instance.matches(
            &PlayerModel::server_player(),
            &entity("minecraft:zombie"),
            &damage("trident", Some("minecraft:player")),
        ));
    }

    #[test]
    fn sculk_catalyst_factory_uses_dedicated_trigger_id_and_empty_instance() {
        let criterion = KilledTriggerInstanceModel::player_killed_entity_near_sculk_catalyst();

        assert_eq!(
            criterion.trigger_id,
            id("minecraft:kill_mob_near_sculk_catalyst")
        );
        assert_eq!(
            criterion.instance,
            KilledTriggerInstanceModel::new(false, None, None)
        );
    }

    #[test]
    fn entity_killed_player_factories_use_java_trigger_id_and_overloads() {
        let any = KilledTriggerInstanceModel::entity_killed_player_any();
        assert_eq!(any.trigger_id, id("minecraft:entity_killed_player"));

        let optional = KilledTriggerInstanceModel::entity_killed_player(Some(
            EntityPredicateModel::entity_type(id("minecraft:zombie")),
        ));
        assert!(optional.instance.matches(
            &PlayerModel::server_player(),
            &entity("minecraft:zombie"),
            &damage("mob", Some("minecraft:zombie")),
        ));

        let builder_damage =
            KilledTriggerInstanceModel::entity_killed_player_builder_with_damage_builder(
                EntityPredicateBuilderModel::entity_type(id("minecraft:skeleton")),
                DamageSourcePredicateBuilderModel::default().msg_id("arrow"),
            );
        assert!(builder_damage.instance.matches(
            &PlayerModel::server_player(),
            &entity("minecraft:skeleton"),
            &damage("arrow", Some("minecraft:skeleton")),
        ));

        let builder = KilledTriggerInstanceModel::entity_killed_player_builder(
            EntityPredicateBuilderModel::entity_type(id("minecraft:warden")),
        );
        assert!(builder.instance.matches(
            &PlayerModel::server_player(),
            &entity("minecraft:warden"),
            &damage("mob", Some("minecraft:warden")),
        ));

        let builder_optional_damage =
            KilledTriggerInstanceModel::entity_killed_player_builder_with_damage(
                EntityPredicateBuilderModel::entity_type(id("minecraft:zombie")),
                Some(
                    DamageSourcePredicateBuilderModel::default()
                        .msg_id("mob")
                        .build(),
                ),
            );
        assert!(builder_optional_damage.instance.matches(
            &PlayerModel::server_player(),
            &entity("minecraft:zombie"),
            &damage("mob", Some("minecraft:zombie")),
        ));

        let optional_entity_damage_builder =
            KilledTriggerInstanceModel::entity_killed_player_with_damage_builder(
                Some(EntityPredicateModel::entity_type(id("minecraft:zombie"))),
                DamageSourcePredicateBuilderModel::default().msg_id("mob"),
            );
        assert!(optional_entity_damage_builder.instance.matches(
            &PlayerModel::server_player(),
            &entity("minecraft:zombie"),
            &damage("mob", Some("minecraft:zombie")),
        ));
    }

    #[test]
    fn validation_reports_entity_predicate_path() {
        let instance = KilledTriggerInstanceModel::new(
            false,
            Some(ContextAwareEntityPredicateModel::wrap(
                EntityPredicateModel::invalid("bad entity"),
            )),
            None,
        );

        assert_eq!(instance.validate(), vec!["entity: bad entity".to_string()]);
    }
}
