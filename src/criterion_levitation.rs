use crate::criterion_distance_predicate::{DistancePredicateModel, DoubleBoundsModel};
use crate::registry::Identifier;

#[derive(Debug, Clone, PartialEq)]
pub struct LevitationTriggerInstanceModel {
    pub player_predicate_present: bool,
    pub distance: Option<DistancePredicateModel>,
    pub duration: IntBoundsModel,
}

impl LevitationTriggerInstanceModel {
    pub fn new(distance: Option<DistancePredicateModel>, duration: IntBoundsModel) -> Self {
        Self {
            player_predicate_present: false,
            distance,
            duration,
        }
    }

    pub fn levitated(distance: DistancePredicateModel) -> LevitationCriterionModel {
        LevitationCriterionModel {
            trigger_id: trigger_id(),
            instance: Self::new(Some(distance), IntBoundsModel::ANY),
        }
    }

    pub fn matches(&self, player: &PlayerPositionModel, start: Vec3Model, duration: i32) -> bool {
        if self.distance.as_ref().is_some_and(|distance| {
            !distance.matches(start.x, start.y, start.z, player.x, player.y, player.z)
        }) {
            return false;
        }

        self.duration.matches(duration)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LevitationCriterionModel {
    pub trigger_id: Identifier,
    pub instance: LevitationTriggerInstanceModel,
}

fn trigger_id() -> Identifier {
    Identifier::parse("minecraft:levitation").unwrap()
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec3Model {
    x: f64,
    y: f64,
    z: f64,
}

impl Vec3Model {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlayerPositionModel {
    x: f64,
    y: f64,
    z: f64,
}

impl PlayerPositionModel {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IntBoundsModel {
    min: Option<i32>,
    max: Option<i32>,
}

impl IntBoundsModel {
    pub const ANY: Self = Self {
        min: None,
        max: None,
    };

    pub fn exactly(value: i32) -> Self {
        Self {
            min: Some(value),
            max: Some(value),
        }
    }

    pub fn between(min: i32, max: i32) -> Self {
        Self {
            min: Some(min),
            max: Some(max),
        }
    }

    pub fn matches(&self, value: i32) -> bool {
        self.min.is_none_or(|min| min <= value) && self.max.is_none_or(|max| max >= value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(value: &str) -> Identifier {
        Identifier::parse(value).unwrap()
    }

    #[test]
    fn omitted_distance_and_any_duration_match_any_levitation_event() {
        let instance = LevitationTriggerInstanceModel::new(None, IntBoundsModel::ANY);

        assert!(instance.matches(
            &PlayerPositionModel::new(100.0, 80.0, -5.0),
            Vec3Model::new(0.0, 64.0, 0.0),
            240,
        ));
    }

    #[test]
    fn distance_is_checked_before_duration() {
        let instance = LevitationTriggerInstanceModel::new(
            Some(DistancePredicateModel::vertical(
                DoubleBoundsModel::between(10.0, 20.0),
            )),
            IntBoundsModel::between(100, 200),
        );

        assert!(instance.matches(
            &PlayerPositionModel::new(0.0, 80.0, 0.0),
            Vec3Model::new(0.0, 64.0, 0.0),
            150,
        ));
        assert!(!instance.matches(
            &PlayerPositionModel::new(0.0, 90.0, 0.0),
            Vec3Model::new(0.0, 64.0, 0.0),
            150,
        ));
        assert!(!instance.matches(
            &PlayerPositionModel::new(0.0, 80.0, 0.0),
            Vec3Model::new(0.0, 64.0, 0.0),
            99,
        ));
    }

    #[test]
    fn distance_uses_start_position_to_current_player_position() {
        let instance = LevitationTriggerInstanceModel::new(
            Some(DistancePredicateModel::horizontal(
                DoubleBoundsModel::between(5.0, 6.0),
            )),
            IntBoundsModel::ANY,
        );

        assert!(instance.matches(
            &PlayerPositionModel::new(3.0, 200.0, 4.0),
            Vec3Model::new(0.0, 64.0, 0.0),
            1,
        ));
        assert!(!instance.matches(
            &PlayerPositionModel::new(2.0, 200.0, 2.0),
            Vec3Model::new(0.0, 64.0, 0.0),
            1,
        ));
    }

    #[test]
    fn levitated_factory_uses_java_trigger_id_distance_and_any_duration() {
        let distance = DistancePredicateModel::absolute(DoubleBoundsModel::at_least(8.0));
        let criterion = LevitationTriggerInstanceModel::levitated(distance);

        assert_eq!(criterion.trigger_id, id("minecraft:levitation"));
        assert!(!criterion.instance.player_predicate_present);
        assert_eq!(criterion.instance.distance, Some(distance));
        assert_eq!(criterion.instance.duration, IntBoundsModel::ANY);
    }

    #[test]
    fn duration_exact_bound_matches_java_int_bounds() {
        let instance = LevitationTriggerInstanceModel::new(None, IntBoundsModel::exactly(40));

        assert!(instance.matches(
            &PlayerPositionModel::new(0.0, 0.0, 0.0),
            Vec3Model::new(0.0, 0.0, 0.0),
            40,
        ));
        assert!(!instance.matches(
            &PlayerPositionModel::new(0.0, 0.0, 0.0),
            Vec3Model::new(0.0, 0.0, 0.0),
            39,
        ));
    }
}
