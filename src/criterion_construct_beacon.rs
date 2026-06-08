use crate::registry::Identifier;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConstructBeaconTriggerInstance {
    pub player_predicate_present: bool,
    pub level: IntBoundsModel,
}

impl ConstructBeaconTriggerInstance {
    pub fn new(level: IntBoundsModel) -> Self {
        Self {
            player_predicate_present: false,
            level,
        }
    }

    pub fn matches(&self, levels: i32) -> bool {
        self.level.matches(levels)
    }

    pub fn constructed_beacon() -> ConstructBeaconCriterion {
        ConstructBeaconCriterion {
            trigger_id: trigger_id(),
            instance: Self::new(IntBoundsModel::any()),
        }
    }

    pub fn constructed_beacon_with_level(level: IntBoundsModel) -> ConstructBeaconCriterion {
        ConstructBeaconCriterion {
            trigger_id: trigger_id(),
            instance: Self::new(level),
        }
    }
}

fn trigger_id() -> Identifier {
    Identifier::parse("minecraft:construct_beacon").unwrap()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConstructBeaconCriterion {
    pub trigger_id: Identifier,
    pub instance: ConstructBeaconTriggerInstance,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IntBoundsModel {
    pub min: Option<i32>,
    pub max: Option<i32>,
}

impl IntBoundsModel {
    pub fn any() -> Self {
        Self {
            min: None,
            max: None,
        }
    }

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
        self.min.is_none_or(|min| value >= min) && self.max.is_none_or(|max| value <= max)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(value: &str) -> Identifier {
        Identifier::parse(value).unwrap()
    }

    #[test]
    fn construct_beacon_default_any_level_matches_java_codec_default() {
        let instance = ConstructBeaconTriggerInstance::new(IntBoundsModel::any());

        assert!(instance.matches(0));
        assert!(instance.matches(1));
        assert!(instance.matches(4));
    }

    #[test]
    fn construct_beacon_level_bounds_match_java_min_max_bounds() {
        let exactly_four = ConstructBeaconTriggerInstance::new(IntBoundsModel::exactly(4));
        assert!(exactly_four.matches(4));
        assert!(!exactly_four.matches(3));
        assert!(!exactly_four.matches(5));

        let mid_levels = ConstructBeaconTriggerInstance::new(IntBoundsModel::between(2, 3));
        assert!(!mid_levels.matches(1));
        assert!(mid_levels.matches(2));
        assert!(mid_levels.matches(3));
        assert!(!mid_levels.matches(4));
    }

    #[test]
    fn construct_beacon_factories_use_java_trigger_id_and_fields() {
        let any = ConstructBeaconTriggerInstance::constructed_beacon();
        assert_eq!(any.trigger_id, id("minecraft:construct_beacon"));
        assert_eq!(
            any.instance,
            ConstructBeaconTriggerInstance::new(IntBoundsModel::any())
        );

        let full = ConstructBeaconTriggerInstance::constructed_beacon_with_level(
            IntBoundsModel::exactly(4),
        );
        assert_eq!(full.trigger_id, id("minecraft:construct_beacon"));
        assert_eq!(full.instance.level, IntBoundsModel::exactly(4));
    }
}
