use crate::criterion_min_max_bounds::DoublesBoundsModel;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MovementPredicateModel {
    pub x: DoublesBoundsModel,
    pub y: DoublesBoundsModel,
    pub z: DoublesBoundsModel,
    pub speed: DoublesBoundsModel,
    pub horizontal_speed: DoublesBoundsModel,
    pub vertical_speed: DoublesBoundsModel,
    pub fall_distance: DoublesBoundsModel,
}

impl MovementPredicateModel {
    pub fn new(
        x: DoublesBoundsModel,
        y: DoublesBoundsModel,
        z: DoublesBoundsModel,
        speed: DoublesBoundsModel,
        horizontal_speed: DoublesBoundsModel,
        vertical_speed: DoublesBoundsModel,
        fall_distance: DoublesBoundsModel,
    ) -> Self {
        Self {
            x,
            y,
            z,
            speed,
            horizontal_speed,
            vertical_speed,
            fall_distance,
        }
    }

    pub fn speed(bounds: DoublesBoundsModel) -> Self {
        Self::new(
            DoublesBoundsModel::ANY,
            DoublesBoundsModel::ANY,
            DoublesBoundsModel::ANY,
            bounds,
            DoublesBoundsModel::ANY,
            DoublesBoundsModel::ANY,
            DoublesBoundsModel::ANY,
        )
    }

    pub fn horizontal_speed(bounds: DoublesBoundsModel) -> Self {
        Self::new(
            DoublesBoundsModel::ANY,
            DoublesBoundsModel::ANY,
            DoublesBoundsModel::ANY,
            DoublesBoundsModel::ANY,
            bounds,
            DoublesBoundsModel::ANY,
            DoublesBoundsModel::ANY,
        )
    }

    pub fn vertical_speed(bounds: DoublesBoundsModel) -> Self {
        Self::new(
            DoublesBoundsModel::ANY,
            DoublesBoundsModel::ANY,
            DoublesBoundsModel::ANY,
            DoublesBoundsModel::ANY,
            DoublesBoundsModel::ANY,
            bounds,
            DoublesBoundsModel::ANY,
        )
    }

    pub fn fall_distance(bounds: DoublesBoundsModel) -> Self {
        Self::new(
            DoublesBoundsModel::ANY,
            DoublesBoundsModel::ANY,
            DoublesBoundsModel::ANY,
            DoublesBoundsModel::ANY,
            DoublesBoundsModel::ANY,
            DoublesBoundsModel::ANY,
            bounds,
        )
    }

    pub fn matches(&self, x: f64, y: f64, z: f64, fall_distance: f64) -> bool {
        if !(self.x.matches(x) && self.y.matches(y) && self.z.matches(z)) {
            return false;
        }

        let speed_sqr = length_squared3(x, y, z);
        if !self.speed.matches_sqr(speed_sqr) {
            return false;
        }

        let horizontal_speed_sqr = length_squared2(x, z);
        if !self.horizontal_speed.matches_sqr(horizontal_speed_sqr) {
            return false;
        }

        if !self.vertical_speed.matches(y.abs()) {
            return false;
        }

        self.fall_distance.matches(fall_distance)
    }

    pub fn codec_field_names() -> [&'static str; 7] {
        [
            "x",
            "y",
            "z",
            "speed",
            "horizontal_speed",
            "vertical_speed",
            "fall_distance",
        ]
    }
}

fn length_squared3(x: f64, y: f64, z: f64) -> f64 {
    (x * x) + (y * y) + (z * z)
}

fn length_squared2(x: f64, z: f64) -> f64 {
    (x * x) + (z * z)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codec_fields_and_any_defaults_match_java_record_shape() {
        let predicate = MovementPredicateModel::new(
            DoublesBoundsModel::ANY,
            DoublesBoundsModel::ANY,
            DoublesBoundsModel::ANY,
            DoublesBoundsModel::ANY,
            DoublesBoundsModel::ANY,
            DoublesBoundsModel::ANY,
            DoublesBoundsModel::ANY,
        );

        assert_eq!(
            MovementPredicateModel::codec_field_names(),
            [
                "x",
                "y",
                "z",
                "speed",
                "horizontal_speed",
                "vertical_speed",
                "fall_distance"
            ]
        );
        assert!(predicate.matches(100.0, -4.0, 3.0, 80.0));
    }

    #[test]
    fn axis_bounds_match_raw_velocity_components_before_speed_checks() {
        let predicate = MovementPredicateModel::new(
            DoublesBoundsModel::between(1.0, 2.0),
            DoublesBoundsModel::exactly(-3.0),
            DoublesBoundsModel::at_most(4.0),
            DoublesBoundsModel::ANY,
            DoublesBoundsModel::ANY,
            DoublesBoundsModel::ANY,
            DoublesBoundsModel::ANY,
        );

        assert!(predicate.matches(1.5, -3.0, 4.0, 0.0));
        assert!(!predicate.matches(0.5, -3.0, 4.0, 0.0));
        assert!(!predicate.matches(1.5, 3.0, 4.0, 0.0));
        assert!(!predicate.matches(1.5, -3.0, 5.0, 0.0));
    }

    #[test]
    fn speed_checks_use_java_squared_vector_lengths() {
        let predicate = MovementPredicateModel::new(
            DoublesBoundsModel::ANY,
            DoublesBoundsModel::ANY,
            DoublesBoundsModel::ANY,
            DoublesBoundsModel::between(13.0, 14.0),
            DoublesBoundsModel::between(5.0, 6.0),
            DoublesBoundsModel::ANY,
            DoublesBoundsModel::ANY,
        );

        assert!(predicate.matches(3.0, 12.0, 4.0, 0.0));
        assert!(!predicate.matches(3.0, 11.0, 4.0, 0.0));
        assert!(!predicate.matches(6.0, 12.0, 1.0, 0.0));
    }

    #[test]
    fn vertical_speed_uses_absolute_y_while_y_axis_uses_signed_y() {
        let predicate = MovementPredicateModel::new(
            DoublesBoundsModel::ANY,
            DoublesBoundsModel::at_most(0.0),
            DoublesBoundsModel::ANY,
            DoublesBoundsModel::ANY,
            DoublesBoundsModel::ANY,
            DoublesBoundsModel::exactly(4.0),
            DoublesBoundsModel::ANY,
        );

        assert!(predicate.matches(0.0, -4.0, 0.0, 0.0));
        assert!(!predicate.matches(0.0, 4.0, 0.0, 0.0));
        assert!(!predicate.matches(0.0, -3.0, 0.0, 0.0));
    }

    #[test]
    fn fall_distance_matches_raw_fall_distance_after_speed_checks() {
        let predicate =
            MovementPredicateModel::fall_distance(DoublesBoundsModel::between(2.0, 5.0));

        assert!(predicate.matches(0.0, 0.0, 0.0, 2.0));
        assert!(predicate.matches(0.0, 0.0, 0.0, 5.0));
        assert!(!predicate.matches(0.0, 0.0, 0.0, 5.1));
    }

    #[test]
    fn static_factories_set_only_the_requested_bound() {
        assert!(
            MovementPredicateModel::speed(DoublesBoundsModel::exactly(5.0))
                .matches(3.0, 4.0, 0.0, 0.0)
        );
        assert!(
            MovementPredicateModel::horizontal_speed(DoublesBoundsModel::exactly(5.0))
                .matches(3.0, 99.0, 4.0, 0.0)
        );
        assert!(
            MovementPredicateModel::vertical_speed(DoublesBoundsModel::exactly(5.0))
                .matches(0.0, -5.0, 0.0, 0.0)
        );
        assert!(
            MovementPredicateModel::fall_distance(DoublesBoundsModel::exactly(5.0))
                .matches(0.0, 0.0, 0.0, 5.0)
        );
    }
}
