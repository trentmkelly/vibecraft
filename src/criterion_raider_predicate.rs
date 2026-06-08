use crate::criterion_entity_sub_predicates::EntitySubPredicateTypeModel;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RaiderPredicateModel {
    pub has_raid: bool,
    pub is_captain: bool,
}

impl RaiderPredicateModel {
    pub const CAPTAIN_WITHOUT_RAID: Self = Self::new(false, true);

    pub const fn new(has_raid: bool, is_captain: bool) -> Self {
        Self {
            has_raid,
            is_captain,
        }
    }

    pub fn codec_field_defaults() -> [(&'static str, bool); 2] {
        [("has_raid", false), ("is_captain", false)]
    }

    pub fn codec(&self) -> EntitySubPredicateTypeModel {
        EntitySubPredicateTypeModel::Raider
    }

    pub fn matches(
        &self,
        entity: &EntityModel,
        _level: &ServerLevelModel,
        _position: Option<Vec3Model>,
    ) -> bool {
        let EntityModel::Raider(raider) = entity else {
            return false;
        };

        raider.has_raid == self.has_raid && raider.is_captain == self.is_captain
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntityModel {
    Raider(RaiderEntityModel),
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RaiderEntityModel {
    has_raid: bool,
    is_captain: bool,
}

impl RaiderEntityModel {
    pub const fn new(has_raid: bool, is_captain: bool) -> Self {
        Self {
            has_raid,
            is_captain,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ServerLevelModel {
    dimension: &'static str,
}

impl ServerLevelModel {
    pub const fn new(dimension: &'static str) -> Self {
        Self { dimension }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Vec3Model {
    x: i32,
    y: i32,
    z: i32,
}

impl Vec3Model {
    pub const fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn raider(has_raid: bool, is_captain: bool) -> EntityModel {
        EntityModel::Raider(RaiderEntityModel::new(has_raid, is_captain))
    }

    #[test]
    fn codec_defaults_match_java_record_codec() {
        assert_eq!(
            RaiderPredicateModel::codec_field_defaults(),
            [("has_raid", false), ("is_captain", false)]
        );
        assert_eq!(
            RaiderPredicateModel::new(false, false).codec(),
            EntitySubPredicateTypeModel::Raider
        );
    }

    #[test]
    fn captain_without_raid_constant_matches_java_constant() {
        assert_eq!(
            RaiderPredicateModel::CAPTAIN_WITHOUT_RAID,
            RaiderPredicateModel::new(false, true)
        );
    }

    #[test]
    fn non_raider_entities_never_match() {
        assert!(!RaiderPredicateModel::new(false, false).matches(
            &EntityModel::Other,
            &ServerLevelModel::new("minecraft:overworld"),
            None
        ));
    }

    #[test]
    fn raider_must_match_has_raid_and_captain_flags_exactly() {
        let predicate = RaiderPredicateModel::new(true, false);
        let level = ServerLevelModel::new("minecraft:overworld");

        assert!(predicate.matches(&raider(true, false), &level, None));
        assert!(!predicate.matches(&raider(false, false), &level, None));
        assert!(!predicate.matches(&raider(true, true), &level, None));
        assert!(!predicate.matches(&raider(false, true), &level, None));
    }

    #[test]
    fn level_and_position_are_ignored_like_java_signature_parameters() {
        let predicate = RaiderPredicateModel::CAPTAIN_WITHOUT_RAID;
        let entity = raider(false, true);

        assert!(predicate.matches(&entity, &ServerLevelModel::new("minecraft:overworld"), None));
        assert!(predicate.matches(
            &entity,
            &ServerLevelModel::new("minecraft:the_nether"),
            Some(Vec3Model::new(10, 64, -4))
        ));
    }
}
