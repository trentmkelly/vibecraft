#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DistancePredicateModel {
    pub x: DoubleBoundsModel,
    pub y: DoubleBoundsModel,
    pub z: DoubleBoundsModel,
    pub horizontal: DoubleBoundsModel,
    pub absolute: DoubleBoundsModel,
}

impl DistancePredicateModel {
    pub fn new(
        x: DoubleBoundsModel,
        y: DoubleBoundsModel,
        z: DoubleBoundsModel,
        horizontal: DoubleBoundsModel,
        absolute: DoubleBoundsModel,
    ) -> Self {
        Self {
            x,
            y,
            z,
            horizontal,
            absolute,
        }
    }

    pub fn horizontal(horizontal: DoubleBoundsModel) -> Self {
        Self::new(
            DoubleBoundsModel::any(),
            DoubleBoundsModel::any(),
            DoubleBoundsModel::any(),
            horizontal,
            DoubleBoundsModel::any(),
        )
    }

    pub fn vertical(y: DoubleBoundsModel) -> Self {
        Self::new(
            DoubleBoundsModel::any(),
            y,
            DoubleBoundsModel::any(),
            DoubleBoundsModel::any(),
            DoubleBoundsModel::any(),
        )
    }

    pub fn absolute(absolute: DoubleBoundsModel) -> Self {
        Self::new(
            DoubleBoundsModel::any(),
            DoubleBoundsModel::any(),
            DoubleBoundsModel::any(),
            DoubleBoundsModel::any(),
            absolute,
        )
    }

    pub fn matches(&self, x0: f64, y0: f64, z0: f64, x1: f64, y1: f64, z1: f64) -> bool {
        let xd = (x0 - x1) as f32;
        let yd = (y0 - y1) as f32;
        let zd = (z0 - z1) as f32;

        if !self.x.matches(f64::from(xd.abs()))
            || !self.y.matches(f64::from(yd.abs()))
            || !self.z.matches(f64::from(zd.abs()))
        {
            return false;
        }

        if !self
            .horizontal
            .matches_sqr(f64::from((xd * xd) + (zd * zd)))
        {
            return false;
        }

        self.absolute
            .matches_sqr(f64::from((xd * xd) + (yd * yd) + (zd * zd)))
    }
}

impl Default for DistancePredicateModel {
    fn default() -> Self {
        Self::new(
            DoubleBoundsModel::any(),
            DoubleBoundsModel::any(),
            DoubleBoundsModel::any(),
            DoubleBoundsModel::any(),
            DoubleBoundsModel::any(),
        )
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct DoubleBoundsModel {
    pub min: Option<f64>,
    pub max: Option<f64>,
}

impl DoubleBoundsModel {
    pub fn any() -> Self {
        Self {
            min: None,
            max: None,
        }
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

    pub fn matches_sqr(&self, value_sqr: f64) -> bool {
        self.min.is_none_or(|min| (min * min) <= value_sqr)
            && self.max.is_none_or(|max| (max * max) >= value_sqr)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_predicate_matches_any_distance() {
        let predicate = DistancePredicateModel::default();

        assert!(predicate.matches(0.0, 0.0, 0.0, 100.0, -32.0, 9.0));
    }

    #[test]
    fn double_bounds_support_lower_only_ranges() {
        let bounds = DoubleBoundsModel::at_least(3.0);

        assert!(!bounds.matches(2.0));
        assert!(bounds.matches(3.0));
        assert!(bounds.matches(4.0));
        assert!(!bounds.matches_sqr(8.0));
        assert!(bounds.matches_sqr(9.0));
    }

    #[test]
    fn axis_bounds_match_absolute_coordinate_deltas_before_distance_checks() {
        let predicate = DistancePredicateModel::new(
            DoubleBoundsModel::between(2.0, 4.0),
            DoubleBoundsModel::at_most(1.0),
            DoubleBoundsModel::exactly(3.0),
            DoubleBoundsModel::any(),
            DoubleBoundsModel::any(),
        );

        assert!(predicate.matches(10.0, 64.0, -4.0, 7.0, 65.0, -1.0));
        assert!(predicate.matches(7.0, 65.0, -1.0, 10.0, 64.0, -4.0));
        assert!(!predicate.matches(10.0, 64.0, -4.0, 5.0, 65.0, -1.0));
        assert!(!predicate.matches(10.0, 64.0, -4.0, 7.0, 66.0, -1.0));
        assert!(!predicate.matches(10.0, 64.0, -4.0, 7.0, 65.0, 0.0));
    }

    #[test]
    fn horizontal_factory_checks_squared_xz_distance() {
        let predicate = DistancePredicateModel::horizontal(DoubleBoundsModel::between(5.0, 6.0));

        assert!(predicate.matches(0.0, 64.0, 0.0, 3.0, 200.0, 4.0));
        assert!(!predicate.matches(0.0, 64.0, 0.0, 2.0, 64.0, 2.0));
        assert!(!predicate.matches(0.0, 64.0, 0.0, 6.0, 64.0, 1.0));
    }

    #[test]
    fn vertical_factory_only_checks_y_delta() {
        let predicate = DistancePredicateModel::vertical(DoubleBoundsModel::exactly(4.0));

        assert!(predicate.matches(0.0, 10.0, 0.0, 100.0, 6.0, -100.0));
        assert!(!predicate.matches(0.0, 10.0, 0.0, 0.0, 5.0, 0.0));
    }

    #[test]
    fn absolute_factory_checks_squared_xyz_distance() {
        let predicate = DistancePredicateModel::absolute(DoubleBoundsModel::between(13.0, 14.0));

        assert!(predicate.matches(0.0, 0.0, 0.0, 3.0, 4.0, 12.0));
        assert!(!predicate.matches(0.0, 0.0, 0.0, 3.0, 4.0, 11.0));
        assert!(!predicate.matches(0.0, 0.0, 0.0, 3.0, 4.0, 14.0));
    }
}
