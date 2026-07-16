//! Fixed-point angular conversion matching Minecraft's `SegmentedAnglePrecision`.

#![allow(dead_code)]

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Down,
    Up,
    North,
    South,
    West,
    East,
}

impl Direction {
    const fn is_vertical(self) -> bool {
        matches!(self, Self::Down | Self::Up)
    }

    const fn data_2d(self) -> i32 {
        match self {
            Self::South => 0,
            Self::West => 1,
            Self::North => 2,
            Self::East => 3,
            Self::Down | Self::Up => -1,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SegmentedAnglePrecision {
    mask: i32,
    precision: u32,
    degree_to_angle: f32,
    angle_to_degree: f32,
}

#[allow(clippy::wrong_self_convention)]
impl SegmentedAnglePrecision {
    pub fn new(bit_precision: u32) -> Result<Self, String> {
        if bit_precision < 2 {
            return Err("Precision cannot be less than 2 bits".to_string());
        }
        if bit_precision > 30 {
            return Err("Precision cannot be greater than 30 bits".to_string());
        }

        let two_pi = 1_i32 << bit_precision;
        Ok(Self {
            mask: two_pi - 1,
            precision: bit_precision,
            degree_to_angle: two_pi as f32 / 360.0,
            angle_to_degree: 360.0 / two_pi as f32,
        })
    }

    pub fn is_same_axis(self, binary_angle_a: i32, binary_angle_b: i32) -> bool {
        let semicircle_mask = self.get_mask() >> 1;
        (binary_angle_a & semicircle_mask) == (binary_angle_b & semicircle_mask)
    }

    pub fn from_direction(self, direction: Direction) -> i32 {
        if direction.is_vertical() {
            0
        } else {
            direction.data_2d() << (self.precision - 2)
        }
    }

    pub fn from_degrees_with_turns(self, degrees: f32) -> i32 {
        // Java Math.round(float) is floor(value + 0.5), including for
        // negative values; this is intentionally not Rust's round().
        (degrees * self.degree_to_angle + 0.5).floor() as i32
    }

    pub fn from_degrees(self, degrees: f32) -> i32 {
        self.normalize(self.from_degrees_with_turns(degrees))
    }

    pub fn to_degrees_with_turns(self, binary_angle: i32) -> f32 {
        binary_angle as f32 * self.angle_to_degree
    }

    pub fn to_degrees(self, binary_angle: i32) -> f32 {
        let degrees = self.to_degrees_with_turns(self.normalize(binary_angle));
        if degrees >= 180.0 {
            degrees - 360.0
        } else {
            degrees
        }
    }

    pub fn normalize(self, binary_angle: i32) -> i32 {
        binary_angle & self.mask
    }

    pub fn get_mask(self) -> i32 {
        self.mask
    }
}

#[cfg(test)]
mod tests {
    use super::{Direction, SegmentedAnglePrecision};

    #[cfg(vibecraft_has_decompiled_sources)]
    #[test]
    fn segmented_angle_precision_matches_java_source() {
        const JAVA: &str =
            vibecraft_java_source!("/net/minecraft/util/SegmentedAnglePrecision.java");
        assert_eq!(JAVA.lines().count(), 65);
        for fragment in [
            "if (bitPrecision < 2)",
            "Precision cannot be less than 2 bits",
            "if (bitPrecision > 30)",
            "int twoPi = 1 << bitPrecision",
            "return (binaryAngleA & semicircleMask) == (binaryAngleB & semicircleMask)",
            "return Math.round(degrees * this.degreeToAngle)",
            "return this.normalize(this.fromDegreesWithTurns(degrees))",
            "return binaryAngle & this.mask",
        ] {
            assert!(
                JAVA.contains(fragment),
                "missing SegmentedAnglePrecision source fragment: {fragment}"
            );
        }
    }

    #[test]
    fn segmented_angle_precision_converts_directions_and_degrees() {
        let precision = SegmentedAnglePrecision::new(4).expect("valid precision");
        assert_eq!(precision.get_mask(), 15);
        assert_eq!(precision.from_direction(Direction::Down), 0);
        assert_eq!(precision.from_direction(Direction::South), 0);
        assert_eq!(precision.from_direction(Direction::West), 4);
        assert_eq!(precision.from_direction(Direction::North), 8);
        assert_eq!(precision.from_direction(Direction::East), 12);
        assert_eq!(precision.from_degrees(90.0), 4);
        assert_eq!(precision.normalize(20), 4);
        assert_eq!(precision.to_degrees(12), -90.0);
        assert!(!precision.is_same_axis(0, 4));
        assert!(precision.is_same_axis(0, 8));
        assert!(SegmentedAnglePrecision::new(1).is_err());
        assert!(SegmentedAnglePrecision::new(31).is_err());
    }
}
