use crate::criterion_damage_predicate::{
    DamagePredicateBuilder, DamagePredicateModel, DamageSourceModel, DoubleBoundsModel,
};
use crate::registry::Identifier;

#[derive(Debug, Clone, PartialEq)]
pub struct EntityHurtPlayerTriggerInstance {
    pub player_predicate_present: bool,
    pub damage: Option<DamagePredicateModel>,
}

impl EntityHurtPlayerTriggerInstance {
    pub fn new(damage: Option<DamagePredicateModel>) -> Self {
        Self {
            player_predicate_present: false,
            damage,
        }
    }

    pub fn matches(
        &self,
        source: &DamageSourceModel,
        original_damage: f64,
        actual_damage: f64,
        blocked: bool,
    ) -> bool {
        self.damage
            .as_ref()
            .is_none_or(|damage| damage.matches(source, original_damage, actual_damage, blocked))
    }

    pub fn entity_hurt_player() -> EntityHurtPlayerCriterion {
        EntityHurtPlayerCriterion {
            trigger_id: trigger_id(),
            instance: Self::new(None),
        }
    }

    pub fn entity_hurt_player_damage(damage: DamagePredicateModel) -> EntityHurtPlayerCriterion {
        EntityHurtPlayerCriterion {
            trigger_id: trigger_id(),
            instance: Self::new(Some(damage)),
        }
    }

    pub fn entity_hurt_player_builder(damage: DamagePredicateBuilder) -> EntityHurtPlayerCriterion {
        Self::entity_hurt_player_damage(damage.build())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EntityHurtPlayerCriterion {
    pub trigger_id: Identifier,
    pub instance: EntityHurtPlayerTriggerInstance,
}

fn trigger_id() -> Identifier {
    Identifier::parse("minecraft:entity_hurt_player").unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(value: &str) -> Identifier {
        Identifier::parse(value).unwrap()
    }

    fn damage_source() -> DamageSourceModel {
        DamageSourceModel {
            damage_type: id("minecraft:mob_attack"),
            direct_entity: None,
            entity: None,
            is_direct: true,
        }
    }

    #[test]
    fn omitted_damage_predicate_matches_any_hurt_player_event() {
        let instance = EntityHurtPlayerTriggerInstance::new(None);

        assert!(instance.matches(&damage_source(), 1.0, 0.0, false));
        assert!(instance.matches(&damage_source(), 20.0, 10.0, true));
    }

    #[test]
    fn present_damage_predicate_delegates_to_damage_matches() {
        let instance = EntityHurtPlayerTriggerInstance::new(Some(
            DamagePredicateModel::builder()
                .dealt_damage(DoubleBoundsModel::between(5.0, 10.0))
                .taken_damage(DoubleBoundsModel::exactly(2.0))
                .blocked(false)
                .build(),
        ));

        assert!(instance.matches(&damage_source(), 6.0, 2.0, false));
        assert!(!instance.matches(&damage_source(), 4.0, 2.0, false));
        assert!(!instance.matches(&damage_source(), 6.0, 3.0, false));
        assert!(!instance.matches(&damage_source(), 6.0, 2.0, true));
    }

    #[test]
    fn default_factory_uses_java_trigger_id_and_empty_damage() {
        let criterion = EntityHurtPlayerTriggerInstance::entity_hurt_player();

        assert_eq!(criterion.trigger_id, id("minecraft:entity_hurt_player"));
        assert_eq!(
            criterion.instance,
            EntityHurtPlayerTriggerInstance::new(None)
        );
        assert!(!criterion.instance.player_predicate_present);
    }

    #[test]
    fn damage_instance_factory_preserves_supplied_predicate() {
        let damage = DamagePredicateModel::builder()
            .dealt_damage(DoubleBoundsModel::at_least(3.0))
            .build();
        let criterion = EntityHurtPlayerTriggerInstance::entity_hurt_player_damage(damage.clone());

        assert_eq!(criterion.trigger_id, id("minecraft:entity_hurt_player"));
        assert_eq!(criterion.instance.damage, Some(damage));
    }

    #[test]
    fn damage_builder_factory_builds_before_storing_predicate() {
        let criterion = EntityHurtPlayerTriggerInstance::entity_hurt_player_builder(
            DamagePredicateModel::builder().taken_damage(DoubleBoundsModel::exactly(1.0)),
        );

        assert!(criterion
            .instance
            .matches(&damage_source(), 99.0, 1.0, false));
        assert!(!criterion
            .instance
            .matches(&damage_source(), 99.0, 2.0, false));
    }
}
