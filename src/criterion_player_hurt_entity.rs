use crate::criterion_damage_predicate::{
    DamagePredicateBuilder, DamagePredicateModel, DamageSourceModel, DoubleBoundsModel,
};
use crate::registry::Identifier;

#[derive(Debug, Clone, PartialEq)]
pub struct PlayerHurtEntityTriggerInstanceModel {
    pub player: Option<ContextAwarePredicateModel>,
    pub damage: Option<DamagePredicateModel>,
    pub entity: Option<ContextAwarePredicateModel>,
}

impl PlayerHurtEntityTriggerInstanceModel {
    pub fn new(
        player: Option<ContextAwarePredicateModel>,
        damage: Option<DamagePredicateModel>,
        entity: Option<ContextAwarePredicateModel>,
    ) -> Self {
        Self {
            player,
            damage,
            entity,
        }
    }

    pub fn player_hurt_entity() -> PlayerHurtEntityCriterionModel {
        Self::criterion(None, None)
    }

    pub fn player_hurt_entity_with_damage(
        damage: Option<DamagePredicateModel>,
    ) -> PlayerHurtEntityCriterionModel {
        Self::criterion(damage, None)
    }

    pub fn player_hurt_entity_with_damage_builder(
        damage: DamagePredicateBuilder,
    ) -> PlayerHurtEntityCriterionModel {
        Self::criterion(Some(damage.build()), None)
    }

    pub fn player_hurt_entity_with_entity(
        entity: Option<EntityPredicateModel>,
    ) -> PlayerHurtEntityCriterionModel {
        Self::criterion(None, EntityPredicateModel::wrap(entity))
    }

    pub fn player_hurt_entity_with_damage_and_entity(
        damage: Option<DamagePredicateModel>,
        entity: Option<EntityPredicateModel>,
    ) -> PlayerHurtEntityCriterionModel {
        Self::criterion(damage, EntityPredicateModel::wrap(entity))
    }

    pub fn player_hurt_entity_with_damage_builder_and_entity(
        damage: DamagePredicateBuilder,
        entity: Option<EntityPredicateModel>,
    ) -> PlayerHurtEntityCriterionModel {
        Self::criterion(Some(damage.build()), EntityPredicateModel::wrap(entity))
    }

    fn criterion(
        damage: Option<DamagePredicateModel>,
        entity: Option<ContextAwarePredicateModel>,
    ) -> PlayerHurtEntityCriterionModel {
        PlayerHurtEntityCriterionModel {
            trigger_id: trigger_id(),
            instance: Self::new(None, damage, entity),
        }
    }

    pub fn matches(
        &self,
        player: &ServerPlayerModel,
        victim: &LootContextModel,
        source: &DamageSourceModel,
        original_damage: f64,
        actual_damage: f64,
        blocked: bool,
    ) -> bool {
        if self
            .damage
            .as_ref()
            .is_some_and(|damage| !damage.matches(source, original_damage, actual_damage, blocked))
        {
            return false;
        }

        self.entity
            .as_ref()
            .is_none_or(|entity| entity.matches(player, victim))
    }

    pub fn validate(&self) -> Vec<String> {
        self.entity
            .as_ref()
            .and_then(ContextAwarePredicateModel::validation_problem)
            .map(|problem| vec![format!("entity: {problem}")])
            .unwrap_or_default()
    }

    pub fn codec_field_names() -> [&'static str; 3] {
        ["player", "damage", "entity"]
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlayerHurtEntityCriterionModel {
    pub trigger_id: Identifier,
    pub instance: PlayerHurtEntityTriggerInstanceModel,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlayerHurtEntityTriggerModel {
    listeners: Vec<PlayerHurtEntityTriggerInstanceModel>,
}

impl PlayerHurtEntityTriggerModel {
    pub fn new(listeners: Vec<PlayerHurtEntityTriggerInstanceModel>) -> Self {
        Self { listeners }
    }

    pub fn trigger(
        &self,
        player: &ServerPlayerModel,
        victim: EntityContextModel,
        source: &DamageSourceModel,
        original_damage: f64,
        actual_damage: f64,
        blocked: bool,
    ) -> Vec<&PlayerHurtEntityTriggerInstanceModel> {
        let victim_context = EntityPredicateModel::create_context(player, victim);
        self.listeners
            .iter()
            .filter(|listener| {
                listener.matches(
                    player,
                    &victim_context,
                    source,
                    original_damage,
                    actual_damage,
                    blocked,
                )
            })
            .collect()
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
    name: Option<String>,
}

impl EntityContextModel {
    pub fn new(entity_type: Identifier, name: Option<&str>) -> Self {
        Self {
            entity_type,
            name: name.map(str::to_string),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LootContextModel {
    entity_type: Identifier,
    name: Option<String>,
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
            required_name: None,
        }
    }

    pub fn named(name: &str) -> Self {
        Self {
            required_type: None,
            required_name: Some(name.to_string()),
        }
    }

    fn wrap(predicate: Option<Self>) -> Option<ContextAwarePredicateModel> {
        predicate.map(ContextAwarePredicateModel::wrap)
    }

    fn create_context(_player: &ServerPlayerModel, entity: EntityContextModel) -> LootContextModel {
        LootContextModel {
            entity_type: entity.entity_type,
            name: entity.name,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextAwarePredicateModel {
    required_type: Option<Identifier>,
    required_name: Option<String>,
    validation_problem: Option<String>,
}

impl ContextAwarePredicateModel {
    fn wrap(predicate: EntityPredicateModel) -> Self {
        Self {
            required_type: predicate.required_type,
            required_name: predicate.required_name,
            validation_problem: None,
        }
    }

    pub fn invalid(problem: &str) -> Self {
        Self {
            required_type: None,
            required_name: None,
            validation_problem: Some(problem.to_string()),
        }
    }

    fn matches(&self, _player: &ServerPlayerModel, context: &LootContextModel) -> bool {
        self.required_type
            .as_ref()
            .is_none_or(|entity_type| entity_type == &context.entity_type)
            && self.required_name.as_ref().is_none_or(|name| {
                context
                    .name
                    .as_ref()
                    .is_some_and(|entity_name| entity_name == name)
            })
    }

    fn validation_problem(&self) -> Option<&str> {
        self.validation_problem.as_deref()
    }
}

fn trigger_id() -> Identifier {
    id("minecraft:player_hurt_entity")
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

    fn victim(entity_type: &str, name: Option<&str>) -> EntityContextModel {
        EntityContextModel::new(id(entity_type), name)
    }

    fn damage_source() -> DamageSourceModel {
        DamageSourceModel {
            damage_type: id("minecraft:player_attack"),
            direct_entity: None,
            entity: None,
            is_direct: true,
        }
    }

    #[test]
    fn codec_field_names_match_java_record_codec() {
        assert_eq!(
            PlayerHurtEntityTriggerInstanceModel::codec_field_names(),
            ["player", "damage", "entity"]
        );
    }

    #[test]
    fn omitted_damage_and_entity_predicates_match_any_player_hurt_entity_event() {
        let instance = PlayerHurtEntityTriggerInstanceModel::new(None, None, None);

        assert!(instance.matches(
            &player(),
            &EntityPredicateModel::create_context(&player(), victim("minecraft:zombie", None)),
            &damage_source(),
            1.0,
            0.0,
            false,
        ));
        let any_entity = PlayerHurtEntityTriggerInstanceModel::new(
            None,
            None,
            Some(ContextAwarePredicateModel::wrap(EntityPredicateModel::any())),
        );
        assert!(any_entity.matches(
            &player(),
            &EntityPredicateModel::create_context(&player(), victim("minecraft:zombie", None)),
            &damage_source(),
            1.0,
            0.0,
            false,
        ));
    }

    #[test]
    fn damage_predicate_is_checked_before_victim_entity_predicate() {
        let instance = PlayerHurtEntityTriggerInstanceModel::new(
            None,
            Some(
                DamagePredicateModel::builder()
                    .dealt_damage(DoubleBoundsModel::between(5.0, 7.0))
                    .taken_damage(DoubleBoundsModel::exactly(2.0))
                    .blocked(false)
                    .build(),
            ),
            Some(ContextAwarePredicateModel::wrap(
                EntityPredicateModel::entity_type(id("minecraft:zombie")),
            )),
        );
        let context =
            EntityPredicateModel::create_context(&player(), victim("minecraft:zombie", None));

        assert!(instance.matches(&player(), &context, &damage_source(), 6.0, 2.0, false));
        assert!(!instance.matches(&player(), &context, &damage_source(), 4.0, 2.0, false));
        assert!(!instance.matches(&player(), &context, &damage_source(), 6.0, 3.0, false));
        assert!(!instance.matches(&player(), &context, &damage_source(), 6.0, 2.0, true));
    }

    #[test]
    fn trigger_creates_victim_loot_context_for_entity_predicate_matching() {
        let listener = PlayerHurtEntityTriggerInstanceModel::new(
            None,
            None,
            Some(ContextAwarePredicateModel::wrap(
                EntityPredicateModel::named("Target"),
            )),
        );
        let trigger = PlayerHurtEntityTriggerModel::new(vec![listener.clone()]);

        assert_eq!(
            trigger.trigger(
                &player(),
                victim("minecraft:villager", Some("Target")),
                &damage_source(),
                1.0,
                1.0,
                false,
            ),
            vec![&listener]
        );
        assert!(trigger
            .trigger(
                &player(),
                victim("minecraft:villager", Some("Other")),
                &damage_source(),
                1.0,
                1.0,
                false,
            )
            .is_empty());
    }

    #[test]
    fn factory_overloads_use_java_trigger_id_and_field_shapes() {
        let default = PlayerHurtEntityTriggerInstanceModel::player_hurt_entity();
        let damage = DamagePredicateModel::builder()
            .dealt_damage(DoubleBoundsModel::at_least(3.0))
            .build();
        let with_optional_damage =
            PlayerHurtEntityTriggerInstanceModel::player_hurt_entity_with_damage(Some(
                damage.clone(),
            ));
        let with_builder =
            PlayerHurtEntityTriggerInstanceModel::player_hurt_entity_with_damage_builder(
                DamagePredicateModel::builder().taken_damage(DoubleBoundsModel::exactly(1.0)),
            );
        let with_entity = PlayerHurtEntityTriggerInstanceModel::player_hurt_entity_with_entity(
            Some(EntityPredicateModel::entity_type(id("minecraft:zombie"))),
        );
        let with_damage_and_entity =
            PlayerHurtEntityTriggerInstanceModel::player_hurt_entity_with_damage_and_entity(
                Some(damage.clone()),
                Some(EntityPredicateModel::entity_type(id("minecraft:zombie"))),
            );
        let with_builder_and_entity =
            PlayerHurtEntityTriggerInstanceModel::player_hurt_entity_with_damage_builder_and_entity(
                DamagePredicateModel::builder().dealt_damage(DoubleBoundsModel::exactly(2.0)),
                Some(EntityPredicateModel::entity_type(id("minecraft:zombie"))),
            );

        for criterion in [
            &default,
            &with_optional_damage,
            &with_builder,
            &with_entity,
            &with_damage_and_entity,
            &with_builder_and_entity,
        ] {
            assert_eq!(criterion.trigger_id, id("minecraft:player_hurt_entity"));
            assert!(criterion.instance.player.is_none());
        }

        assert_eq!(default.instance.damage, None);
        assert_eq!(with_optional_damage.instance.damage, Some(damage.clone()));
        assert!(with_builder.instance.damage.is_some());
        assert!(with_entity.instance.entity.is_some());
        assert_eq!(with_damage_and_entity.instance.damage, Some(damage));
        assert!(with_damage_and_entity.instance.entity.is_some());
        assert!(with_builder_and_entity.instance.damage.is_some());
        assert!(with_builder_and_entity.instance.entity.is_some());
    }

    #[test]
    fn validation_reports_entity_predicate_label_only() {
        let instance = PlayerHurtEntityTriggerInstanceModel::new(
            None,
            Some(DamagePredicateModel::builder().build()),
            Some(ContextAwarePredicateModel::invalid("bad entity")),
        );

        assert_eq!(instance.validate(), vec!["entity: bad entity".to_string()]);
    }
}
