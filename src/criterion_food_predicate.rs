#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FoodPredicateModel {
    level: IntBoundsModel,
    saturation: DoubleBoundsModel,
}

impl FoodPredicateModel {
    pub const ANY: Self = Self {
        level: IntBoundsModel::ANY,
        saturation: DoubleBoundsModel::ANY,
    };

    pub fn new(level: IntBoundsModel, saturation: DoubleBoundsModel) -> Self {
        Self { level, saturation }
    }

    pub fn builder() -> FoodPredicateBuilderModel {
        FoodPredicateBuilderModel::food()
    }

    pub fn matches(&self, food: &FoodDataModel) -> bool {
        if !self.level.matches(food.food_level) {
            return false;
        }

        self.saturation.matches(f64::from(food.saturation_level))
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FoodPredicateBuilderModel {
    level: IntBoundsModel,
    saturation: DoubleBoundsModel,
}

impl FoodPredicateBuilderModel {
    pub const fn food() -> Self {
        Self {
            level: IntBoundsModel::ANY,
            saturation: DoubleBoundsModel::ANY,
        }
    }

    pub fn with_level(mut self, level: IntBoundsModel) -> Self {
        self.level = level;
        self
    }

    pub fn with_saturation(mut self, saturation: DoubleBoundsModel) -> Self {
        self.saturation = saturation;
        self
    }

    pub fn build(self) -> FoodPredicateModel {
        FoodPredicateModel::new(self.level, self.saturation)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FoodDataModel {
    food_level: i32,
    saturation_level: f32,
}

impl FoodDataModel {
    pub fn new(food_level: i32, saturation_level: f32) -> Self {
        Self {
            food_level,
            saturation_level,
        }
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

    pub fn any() -> Self {
        Self::ANY
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DoubleBoundsModel {
    min: Option<f64>,
    max: Option<f64>,
}

impl DoubleBoundsModel {
    pub const ANY: Self = Self {
        min: None,
        max: None,
    };

    pub fn any() -> Self {
        Self::ANY
    }

    pub fn exactly(value: f64) -> Self {
        Self {
            min: Some(value),
            max: Some(value),
        }
    }

    pub fn between(min: f64, max: f64) -> Self {
        Self {
            min: Some(min),
            max: Some(max),
        }
    }

    pub fn at_least(value: f64) -> Self {
        Self {
            min: Some(value),
            max: None,
        }
    }

    pub fn at_most(value: f64) -> Self {
        Self {
            min: None,
            max: Some(value),
        }
    }

    pub fn matches(&self, value: f64) -> bool {
        self.min.is_none_or(|min| min <= value) && self.max.is_none_or(|max| max >= value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn food(level: i32, saturation: f32) -> FoodDataModel {
        FoodDataModel::new(level, saturation)
    }

    #[test]
    fn any_matches_all_food_data() {
        assert!(FoodPredicateModel::ANY.matches(&food(0, 0.0)));
        assert!(FoodPredicateModel::ANY.matches(&food(20, 5.0)));
        assert!(FoodPredicateModel::builder()
            .build()
            .matches(&food(12, 1.25)));
    }

    #[test]
    fn level_is_checked_before_saturation_like_java() {
        let predicate = FoodPredicateModel::new(
            IntBoundsModel::between(10, 20),
            DoubleBoundsModel::between(1.0, 3.0),
        );

        assert!(predicate.matches(&food(12, 2.5)));
        assert!(!predicate.matches(&food(8, 2.5)));
        assert!(!predicate.matches(&food(12, 3.5)));
    }

    #[test]
    fn integer_bounds_support_exact_lower_and_upper_shapes() {
        assert!(IntBoundsModel::exactly(20).matches(20));
        assert!(!IntBoundsModel::exactly(20).matches(19));
        assert!(IntBoundsModel::at_least(6).matches(7));
        assert!(!IntBoundsModel::at_least(6).matches(5));
        assert!(IntBoundsModel::at_most(6).matches(6));
        assert!(!IntBoundsModel::at_most(6).matches(7));
        assert!(IntBoundsModel::any().matches(i32::MIN));
    }

    #[test]
    fn double_bounds_support_exact_lower_and_upper_shapes() {
        assert!(DoubleBoundsModel::exactly(5.0).matches(5.0));
        assert!(!DoubleBoundsModel::exactly(5.0).matches(5.1));
        assert!(DoubleBoundsModel::at_least(1.5).matches(2.0));
        assert!(!DoubleBoundsModel::at_least(1.5).matches(1.0));
        assert!(DoubleBoundsModel::at_most(1.5).matches(1.5));
        assert!(!DoubleBoundsModel::at_most(1.5).matches(2.0));
        assert!(DoubleBoundsModel::any().matches(f64::MAX));
    }

    #[test]
    fn builder_preserves_level_and_saturation_bounds() {
        let predicate = FoodPredicateBuilderModel::food()
            .with_level(IntBoundsModel::at_least(18))
            .with_saturation(DoubleBoundsModel::at_most(5.0))
            .build();

        assert_eq!(predicate.level, IntBoundsModel::at_least(18));
        assert_eq!(predicate.saturation, DoubleBoundsModel::at_most(5.0));
        assert!(predicate.matches(&food(20, 5.0)));
        assert!(!predicate.matches(&food(17, 5.0)));
    }
}
