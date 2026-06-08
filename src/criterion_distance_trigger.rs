use crate::criterion_distance_predicate::{DistancePredicateModel, DoubleBoundsModel};
use crate::registry::Identifier;

#[derive(Debug, Clone, PartialEq)]
pub struct DistanceTriggerInstance {
    pub player_predicate_present: bool,
    pub start_position: Option<LocationPredicateModel>,
    pub distance: Option<DistancePredicateModel>,
}

impl DistanceTriggerInstance {
    pub fn new(
        player_predicate_present: bool,
        start_position: Option<LocationPredicateModel>,
        distance: Option<DistancePredicateModel>,
    ) -> Self {
        Self {
            player_predicate_present,
            start_position,
            distance,
        }
    }

    pub fn matches(&self, entered_position: Vec3Model, player_position: Vec3Model) -> bool {
        if self
            .start_position
            .as_ref()
            .is_some_and(|predicate| !predicate.matches(entered_position))
        {
            return false;
        }

        self.distance.as_ref().is_none_or(|distance| {
            distance.matches(
                entered_position.x,
                entered_position.y,
                entered_position.z,
                player_position.x,
                player_position.y,
                player_position.z,
            )
        })
    }

    pub fn fall_from_height(
        distance: DistancePredicateModel,
        start_position: LocationPredicateBuilder,
    ) -> DistanceCriterion {
        DistanceCriterion {
            trigger_id: trigger_id("fall_from_height"),
            instance: Self::new(true, Some(start_position.build()), Some(distance)),
        }
    }

    pub fn ride_entity_in_lava(distance: DistancePredicateModel) -> DistanceCriterion {
        DistanceCriterion {
            trigger_id: trigger_id("ride_entity_in_lava"),
            instance: Self::new(true, None, Some(distance)),
        }
    }

    pub fn travelled_through_nether(distance: DistancePredicateModel) -> DistanceCriterion {
        DistanceCriterion {
            trigger_id: trigger_id("nether_travel"),
            instance: Self::new(false, None, Some(distance)),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DistanceCriterion {
    pub trigger_id: Identifier,
    pub instance: DistanceTriggerInstance,
}

fn trigger_id(path: &str) -> Identifier {
    Identifier::parse(path).unwrap()
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
    pub x: DoubleBoundsModel,
    pub y: DoubleBoundsModel,
    pub z: DoubleBoundsModel,
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

    pub fn at_y_location(y: DoubleBoundsModel) -> Self {
        Self::location().set_y(y)
    }

    pub fn set_x(mut self, x: DoubleBoundsModel) -> Self {
        self.x = x;
        self
    }

    pub fn set_y(mut self, y: DoubleBoundsModel) -> Self {
        self.y = y;
        self
    }

    pub fn set_z(mut self, z: DoubleBoundsModel) -> Self {
        self.z = z;
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
    fn omitted_start_and_distance_predicates_match_any_positions() {
        let instance = DistanceTriggerInstance::new(false, None, None);

        assert!(instance.matches(vec3(0.0, 64.0, 0.0), vec3(100.0, -20.0, 5.0)));
    }

    #[test]
    fn start_position_is_checked_before_distance_like_java() {
        let instance = DistanceTriggerInstance::new(
            false,
            Some(
                LocationPredicateBuilder::location()
                    .set_x(DoubleBoundsModel::between(0.0, 10.0))
                    .set_y(DoubleBoundsModel::at_least(60.0))
                    .set_z(DoubleBoundsModel::at_most(5.0))
                    .build(),
            ),
            Some(DistancePredicateModel::horizontal(
                DoubleBoundsModel::at_most(4.0),
            )),
        );

        assert!(instance.matches(vec3(3.0, 64.0, 2.0), vec3(6.0, 100.0, 2.0)));
        assert!(!instance.matches(vec3(11.0, 64.0, 2.0), vec3(11.0, 64.0, 2.0)));
        assert!(!instance.matches(vec3(3.0, 64.0, 2.0), vec3(8.0, 64.0, 2.0)));
    }

    #[test]
    fn fall_from_height_factory_sets_trigger_id_player_distance_and_start_position() {
        let distance = DistancePredicateModel::vertical(DoubleBoundsModel::at_least(5.0));
        let criterion = DistanceTriggerInstance::fall_from_height(
            distance,
            LocationPredicateBuilder::at_y_location(DoubleBoundsModel::at_least(70.0)),
        );

        assert_eq!(criterion.trigger_id, id("minecraft:fall_from_height"));
        assert!(criterion.instance.player_predicate_present);
        assert_eq!(criterion.instance.distance, Some(distance));
        assert!(criterion
            .instance
            .start_position
            .as_ref()
            .is_some_and(|predicate| predicate.matches(vec3(0.0, 70.0, 0.0))));
        assert!(!criterion
            .instance
            .matches(vec3(0.0, 69.0, 0.0), vec3(0.0, 60.0, 0.0)));
    }

    #[test]
    fn ride_entity_in_lava_factory_sets_java_trigger_shape() {
        let distance = DistancePredicateModel::absolute(DoubleBoundsModel::between(2.0, 3.0));
        let criterion = DistanceTriggerInstance::ride_entity_in_lava(distance);

        assert_eq!(criterion.trigger_id, id("minecraft:ride_entity_in_lava"));
        assert!(criterion.instance.player_predicate_present);
        assert_eq!(criterion.instance.start_position, None);
        assert_eq!(criterion.instance.distance, Some(distance));
    }

    #[test]
    fn travelled_through_nether_factory_has_no_player_or_start_predicate() {
        let distance = DistancePredicateModel::horizontal(DoubleBoundsModel::at_least(100.0));
        let criterion = DistanceTriggerInstance::travelled_through_nether(distance);

        assert_eq!(criterion.trigger_id, id("minecraft:nether_travel"));
        assert!(!criterion.instance.player_predicate_present);
        assert_eq!(criterion.instance.start_position, None);
        assert_eq!(criterion.instance.distance, Some(distance));
    }
}
