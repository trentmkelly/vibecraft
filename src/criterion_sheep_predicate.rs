use crate::criterion_entity_sub_predicates::EntitySubPredicateTypeModel;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SheepPredicateModel {
    pub sheared: Option<bool>,
}

impl SheepPredicateModel {
    pub const fn new(sheared: Option<bool>) -> Self {
        Self { sheared }
    }

    pub fn codec_field_names() -> [&'static str; 1] {
        ["sheared"]
    }

    pub fn codec(&self) -> EntitySubPredicateTypeModel {
        EntitySubPredicateTypeModel::Sheep
    }

    pub fn has_wool() -> Self {
        Self::new(Some(false))
    }

    pub fn matches(
        &self,
        entity: &EntityModel,
        _level: &ServerLevelModel,
        _position: Option<Vec3Model>,
    ) -> bool {
        let EntityModel::Sheep(sheep) = entity else {
            return false;
        };

        self.sheared
            .is_none_or(|expected| sheep.is_sheared == expected)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntityModel {
    Sheep(SheepEntityModel),
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SheepEntityModel {
    is_sheared: bool,
}

impl SheepEntityModel {
    pub const fn new(is_sheared: bool) -> Self {
        Self { is_sheared }
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

    fn sheep(is_sheared: bool) -> EntityModel {
        EntityModel::Sheep(SheepEntityModel::new(is_sheared))
    }

    #[test]
    fn codec_field_and_sub_predicate_type_match_java() {
        let predicate = SheepPredicateModel::new(None);

        assert_eq!(SheepPredicateModel::codec_field_names(), ["sheared"]);
        assert_eq!(predicate.codec(), EntitySubPredicateTypeModel::Sheep);
    }

    #[test]
    fn non_sheep_entities_never_match() {
        assert!(!SheepPredicateModel::new(None).matches(
            &EntityModel::Other,
            &ServerLevelModel::new("minecraft:overworld"),
            None
        ));
    }

    #[test]
    fn omitted_sheared_predicate_matches_any_sheep_state() {
        let predicate = SheepPredicateModel::new(None);
        let level = ServerLevelModel::new("minecraft:overworld");

        assert!(predicate.matches(&sheep(false), &level, None));
        assert!(predicate.matches(&sheep(true), &level, None));
    }

    #[test]
    fn present_sheared_predicate_must_equal_sheep_is_sheared() {
        let sheared = SheepPredicateModel::new(Some(true));
        let unsheared = SheepPredicateModel::new(Some(false));
        let level = ServerLevelModel::new("minecraft:overworld");

        assert!(sheared.matches(&sheep(true), &level, None));
        assert!(!sheared.matches(&sheep(false), &level, None));
        assert!(unsheared.matches(&sheep(false), &level, None));
        assert!(!unsheared.matches(&sheep(true), &level, None));
    }

    #[test]
    fn has_wool_factory_requires_unsheared_sheep() {
        let predicate = SheepPredicateModel::has_wool();

        assert_eq!(predicate, SheepPredicateModel::new(Some(false)));
        assert!(predicate.matches(
            &sheep(false),
            &ServerLevelModel::new("minecraft:overworld"),
            Some(Vec3Model::new(1, 64, 1))
        ));
        assert!(!predicate.matches(
            &sheep(true),
            &ServerLevelModel::new("minecraft:the_nether"),
            None
        ));
    }
}
