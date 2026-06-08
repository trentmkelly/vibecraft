use crate::criterion_entity_sub_predicates::EntitySubPredicateTypeModel;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SlimePredicateModel {
    pub size: IntBoundsModel,
}

impl SlimePredicateModel {
    pub const fn new(size: IntBoundsModel) -> Self {
        Self { size }
    }

    pub const fn codec_default_size() -> IntBoundsModel {
        IntBoundsModel::ANY
    }

    pub fn codec_field_names() -> [&'static str; 1] {
        ["size"]
    }

    pub const fn sized(size: IntBoundsModel) -> Self {
        Self::new(size)
    }

    pub fn matches(
        &self,
        entity: &EntityModel,
        _level: &ServerLevelModel,
        _position: Option<Vec3Model>,
    ) -> bool {
        let EntityModel::Slime(slime) = entity else {
            return false;
        };

        self.size.matches(slime.size)
    }

    pub fn codec(&self) -> EntitySubPredicateTypeModel {
        EntitySubPredicateTypeModel::Slime
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntityModel {
    Slime(SlimeEntityModel),
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SlimeEntityModel {
    size: i32,
}

impl SlimeEntityModel {
    pub const fn new(size: i32) -> Self {
        Self { size }
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

    pub const fn exactly(value: i32) -> Self {
        Self {
            min: Some(value),
            max: Some(value),
        }
    }

    pub const fn between(min: i32, max: i32) -> Self {
        Self {
            min: Some(min),
            max: Some(max),
        }
    }

    pub const fn at_least(value: i32) -> Self {
        Self {
            min: Some(value),
            max: None,
        }
    }

    pub const fn at_most(value: i32) -> Self {
        Self {
            min: None,
            max: Some(value),
        }
    }

    pub fn matches(&self, value: i32) -> bool {
        self.min.is_none_or(|min| min <= value) && self.max.is_none_or(|max| max >= value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn slime(size: i32) -> EntityModel {
        EntityModel::Slime(SlimeEntityModel::new(size))
    }

    #[test]
    fn codec_field_and_default_size_match_java_record_codec() {
        let predicate = SlimePredicateModel::new(SlimePredicateModel::codec_default_size());

        assert_eq!(SlimePredicateModel::codec_field_names(), ["size"]);
        assert_eq!(predicate.size, IntBoundsModel::ANY);
        assert_eq!(predicate.codec(), EntitySubPredicateTypeModel::Slime);
    }

    #[test]
    fn sized_factory_preserves_provided_bounds() {
        let size = IntBoundsModel::between(2, 4);

        assert_eq!(
            SlimePredicateModel::sized(size),
            SlimePredicateModel::new(size)
        );
    }

    #[test]
    fn non_slime_entities_never_match() {
        assert!(!SlimePredicateModel::new(IntBoundsModel::ANY).matches(
            &EntityModel::Other,
            &ServerLevelModel::new("minecraft:overworld"),
            None
        ));
    }

    #[test]
    fn any_size_matches_all_slime_sizes() {
        let predicate = SlimePredicateModel::new(IntBoundsModel::ANY);
        let level = ServerLevelModel::new("minecraft:overworld");

        assert!(predicate.matches(&slime(1), &level, None));
        assert!(predicate.matches(&slime(4), &level, Some(Vec3Model::new(0, 64, 0))));
    }

    #[test]
    fn size_bounds_are_checked_against_slime_get_size() {
        let exactly_two = SlimePredicateModel::sized(IntBoundsModel::exactly(2));
        let at_least_three = SlimePredicateModel::sized(IntBoundsModel::at_least(3));
        let at_most_two = SlimePredicateModel::sized(IntBoundsModel::at_most(2));
        let level = ServerLevelModel::new("minecraft:overworld");

        assert!(exactly_two.matches(&slime(2), &level, None));
        assert!(!exactly_two.matches(&slime(3), &level, None));
        assert!(at_least_three.matches(&slime(4), &level, None));
        assert!(!at_least_three.matches(&slime(2), &level, None));
        assert!(at_most_two.matches(&slime(1), &level, None));
        assert!(!at_most_two.matches(&slime(3), &level, None));
    }
}
