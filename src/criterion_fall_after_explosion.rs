use crate::criterion_distance_predicate::{DistancePredicateModel, DoubleBoundsModel};
use crate::registry::Identifier;

#[derive(Debug, Clone, PartialEq)]
pub struct FallAfterExplosionTriggerInstance {
    pub player_predicate_present: bool,
    pub start_position: Option<LocationPredicateModel>,
    pub distance: Option<DistancePredicateModel>,
    pub cause: Option<ContextAwarePredicateModel>,
}

impl FallAfterExplosionTriggerInstance {
    pub fn new(
        player_predicate_present: bool,
        start_position: Option<LocationPredicateModel>,
        distance: Option<DistancePredicateModel>,
        cause: Option<ContextAwarePredicateModel>,
    ) -> Self {
        Self {
            player_predicate_present,
            start_position,
            distance,
            cause,
        }
    }

    pub fn matches(
        &self,
        entered_position: Vec3Model,
        player_position: Vec3Model,
        cause: Option<&LootContextModel>,
    ) -> bool {
        if self
            .start_position
            .as_ref()
            .is_some_and(|predicate| !predicate.matches(entered_position))
        {
            return false;
        }

        if self.distance.as_ref().is_some_and(|distance| {
            !distance.matches(
                entered_position.x,
                entered_position.y,
                entered_position.z,
                player_position.x,
                player_position.y,
                player_position.z,
            )
        }) {
            return false;
        }

        self.cause
            .as_ref()
            .is_none_or(|predicate| cause.is_some_and(|cause| predicate.matches(cause)))
    }

    pub fn validate(&self) -> Vec<String> {
        if self.cause.is_some() {
            vec!["cause".to_string()]
        } else {
            Vec::new()
        }
    }

    pub fn fall_after_explosion(
        distance: DistancePredicateModel,
        cause: EntityPredicateBuilder,
    ) -> FallAfterExplosionCriterion {
        FallAfterExplosionCriterion {
            trigger_id: trigger_id(),
            instance: Self::new(
                false,
                None,
                Some(distance),
                Some(ContextAwarePredicateModel::wrap(cause.build())),
            ),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallAfterExplosionCriterion {
    pub trigger_id: Identifier,
    pub instance: FallAfterExplosionTriggerInstance,
}

fn trigger_id() -> Identifier {
    Identifier::parse("minecraft:fall_after_explosion").unwrap()
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec3Model {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vec3Model {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LocationPredicateModel {
    x: DoubleBoundsModel,
    y: DoubleBoundsModel,
    z: DoubleBoundsModel,
}

impl LocationPredicateModel {
    pub fn matches(&self, position: Vec3Model) -> bool {
        self.x.matches(position.x) && self.y.matches(position.y) && self.z.matches(position.z)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LocationPredicateBuilder {
    x: DoubleBoundsModel,
    y: DoubleBoundsModel,
    z: DoubleBoundsModel,
}

impl LocationPredicateBuilder {
    pub fn location() -> Self {
        Self::default()
    }

    pub fn set_x(mut self, x: DoubleBoundsModel) -> Self {
        self.x = x;
        self
    }

    pub fn build(self) -> LocationPredicateModel {
        LocationPredicateModel {
            x: self.x,
            y: self.y,
            z: self.z,
        }
    }
}

impl Default for LocationPredicateBuilder {
    fn default() -> Self {
        Self {
            x: DoubleBoundsModel::any(),
            y: DoubleBoundsModel::any(),
            z: DoubleBoundsModel::any(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextAwarePredicateModel {
    entity_type: Identifier,
}

impl ContextAwarePredicateModel {
    pub fn wrap(predicate: EntityPredicateModel) -> Self {
        Self {
            entity_type: predicate.entity_type,
        }
    }

    pub fn matches(&self, context: &LootContextModel) -> bool {
        self.entity_type == context.entity_type
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntityPredicateModel {
    entity_type: Identifier,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntityPredicateBuilder {
    entity_type: Identifier,
}

impl EntityPredicateBuilder {
    pub fn entity_type(entity_type: Identifier) -> Self {
        Self { entity_type }
    }

    pub fn build(self) -> EntityPredicateModel {
        EntityPredicateModel {
            entity_type: self.entity_type,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LootContextModel {
    entity_type: Identifier,
}

impl LootContextModel {
    pub fn new(entity_type: Identifier) -> Self {
        Self { entity_type }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(value: &str) -> Identifier {
        Identifier::parse(value).unwrap()
    }

    fn vec3(x: f64, y: f64, z: f64) -> Vec3Model {
        Vec3Model::new(x, y, z)
    }

    #[test]
    fn omitted_predicates_match_any_fall_after_explosion_event() {
        let instance = FallAfterExplosionTriggerInstance::new(false, None, None, None);

        assert!(instance.matches(vec3(0.0, 64.0, 0.0), vec3(100.0, 64.0, 0.0), None));
    }

    #[test]
    fn start_position_distance_and_cause_match_in_java_order() {
        let instance = FallAfterExplosionTriggerInstance::new(
            false,
            Some(
                LocationPredicateBuilder::location()
                    .set_x(DoubleBoundsModel::between(0.0, 10.0))
                    .build(),
            ),
            Some(DistancePredicateModel::horizontal(
                DoubleBoundsModel::at_most(5.0),
            )),
            Some(ContextAwarePredicateModel::wrap(
                EntityPredicateBuilder::entity_type(id("minecraft:creeper")).build(),
            )),
        );

        assert!(instance.matches(
            vec3(2.0, 64.0, 0.0),
            vec3(5.0, 80.0, 4.0),
            Some(&LootContextModel::new(id("minecraft:creeper")))
        ));
        assert!(!instance.matches(
            vec3(11.0, 64.0, 0.0),
            vec3(11.0, 64.0, 0.0),
            Some(&LootContextModel::new(id("minecraft:creeper")))
        ));
        assert!(!instance.matches(
            vec3(2.0, 64.0, 0.0),
            vec3(8.0, 80.0, 0.0),
            Some(&LootContextModel::new(id("minecraft:creeper")))
        ));
        assert!(!instance.matches(vec3(2.0, 64.0, 0.0), vec3(5.0, 80.0, 4.0), None));
        assert!(!instance.matches(
            vec3(2.0, 64.0, 0.0),
            vec3(5.0, 80.0, 4.0),
            Some(&LootContextModel::new(id("minecraft:tnt")))
        ));
    }

    #[test]
    fn validation_reports_cause_label_only_when_cause_predicate_exists() {
        let without_cause = FallAfterExplosionTriggerInstance::new(false, None, None, None);
        let with_cause = FallAfterExplosionTriggerInstance::new(
            false,
            None,
            None,
            Some(ContextAwarePredicateModel::wrap(
                EntityPredicateBuilder::entity_type(id("minecraft:creeper")).build(),
            )),
        );

        assert!(without_cause.validate().is_empty());
        assert_eq!(with_cause.validate(), vec!["cause".to_string()]);
    }

    #[test]
    fn factory_sets_java_trigger_id_distance_and_wrapped_cause() {
        let distance = DistancePredicateModel::absolute(DoubleBoundsModel::at_least(4.0));
        let criterion = FallAfterExplosionTriggerInstance::fall_after_explosion(
            distance,
            EntityPredicateBuilder::entity_type(id("minecraft:creeper")),
        );

        assert_eq!(criterion.trigger_id, id("minecraft:fall_after_explosion"));
        assert!(!criterion.instance.player_predicate_present);
        assert_eq!(criterion.instance.start_position, None);
        assert_eq!(criterion.instance.distance, Some(distance));
        assert!(criterion.instance.cause.is_some());
    }
}
