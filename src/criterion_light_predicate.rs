use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LightPredicateModel {
    composite: IntBoundsModel,
}

impl LightPredicateModel {
    pub fn new(composite: IntBoundsModel) -> Self {
        Self { composite }
    }

    pub fn builder() -> LightPredicateBuilderModel {
        LightPredicateBuilderModel::light()
    }

    pub fn matches(&self, level: &ServerLevelLightModel, pos: BlockPosModel) -> bool {
        if !level.is_loaded(pos) {
            return false;
        }

        self.composite
            .matches(level.get_max_local_raw_brightness(pos))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LightPredicateBuilderModel {
    composite: IntBoundsModel,
}

impl LightPredicateBuilderModel {
    pub fn light() -> Self {
        Self {
            composite: IntBoundsModel::ANY,
        }
    }

    pub fn set_composite(mut self, composite: IntBoundsModel) -> Self {
        self.composite = composite;
        self
    }

    pub fn build(self) -> LightPredicateModel {
        LightPredicateModel::new(self.composite)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerLevelLightModel {
    loaded_positions: BTreeSet<BlockPosModel>,
    raw_brightness: BTreeMap<BlockPosModel, i32>,
}

impl ServerLevelLightModel {
    pub fn new(
        loaded_positions: impl IntoIterator<Item = BlockPosModel>,
        raw_brightness: impl IntoIterator<Item = (BlockPosModel, i32)>,
    ) -> Self {
        Self {
            loaded_positions: loaded_positions.into_iter().collect(),
            raw_brightness: raw_brightness.into_iter().collect(),
        }
    }

    fn is_loaded(&self, pos: BlockPosModel) -> bool {
        self.loaded_positions.contains(&pos)
    }

    fn get_max_local_raw_brightness(&self, pos: BlockPosModel) -> i32 {
        *self.raw_brightness.get(&pos).unwrap_or(&0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct BlockPosModel {
    x: i32,
    y: i32,
    z: i32,
}

impl BlockPosModel {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
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

    pub fn at_least(value: i32) -> Self {
        Self {
            min: Some(value),
            max: None,
        }
    }

    pub fn at_most(value: i32) -> Self {
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

    fn pos(x: i32, y: i32, z: i32) -> BlockPosModel {
        BlockPosModel::new(x, y, z)
    }

    fn level() -> ServerLevelLightModel {
        ServerLevelLightModel::new(
            [pos(0, 64, 0), pos(1, 64, 0), pos(2, 64, 0)],
            [(pos(0, 64, 0), 15), (pos(1, 64, 0), 7)],
        )
    }

    #[test]
    fn unloaded_position_never_matches_even_when_bounds_are_any() {
        let predicate = LightPredicateModel::builder().build();

        assert!(!predicate.matches(&level(), pos(99, 64, 0)));
    }

    #[test]
    fn loaded_position_matches_max_local_raw_brightness_against_bounds() {
        let predicate = LightPredicateModel::builder()
            .set_composite(IntBoundsModel::between(6, 8))
            .build();

        assert!(predicate.matches(&level(), pos(1, 64, 0)));
        assert!(!predicate.matches(&level(), pos(0, 64, 0)));
    }

    #[test]
    fn loaded_missing_brightness_defaults_to_zero_for_test_level_model() {
        let predicate = LightPredicateModel::builder()
            .set_composite(IntBoundsModel::exactly(0))
            .build();

        assert!(predicate.matches(&level(), pos(2, 64, 0)));
    }

    #[test]
    fn builder_defaults_composite_to_any_and_preserves_set_composite() {
        let default = LightPredicateBuilderModel::light().build();
        let constrained = LightPredicateBuilderModel::light()
            .set_composite(IntBoundsModel::at_least(12))
            .build();

        assert_eq!(default, LightPredicateModel::new(IntBoundsModel::ANY));
        assert_eq!(
            constrained,
            LightPredicateModel::new(IntBoundsModel::at_least(12))
        );
        assert!(default.matches(&level(), pos(1, 64, 0)));
        assert!(constrained.matches(&level(), pos(0, 64, 0)));
        assert!(!constrained.matches(&level(), pos(1, 64, 0)));
    }

    #[test]
    fn int_bounds_cover_lower_and_upper_shapes() {
        assert!(IntBoundsModel::at_least(10).matches(15));
        assert!(!IntBoundsModel::at_least(10).matches(9));
        assert!(IntBoundsModel::at_most(10).matches(9));
        assert!(!IntBoundsModel::at_most(10).matches(11));
    }
}
