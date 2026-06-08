use std::collections::BTreeSet;
use std::io::{self, Read, Write};

pub trait PositionModel {
    fn x(&self) -> f64;
    fn y(&self) -> f64;
    fn z(&self) -> f64;
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec3PositionModel {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl PositionModel for Vec3PositionModel {
    fn x(&self) -> f64 {
        self.x
    }

    fn y(&self) -> f64 {
        self.y
    }

    fn z(&self) -> f64 {
        self.z
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CoreAxisModel {
    X,
    Y,
    Z,
}

impl CoreAxisModel {
    pub fn ordinal(self) -> i32 {
        match self {
            Self::X => 0,
            Self::Y => 1,
            Self::Z => 2,
        }
    }

    pub fn by_ordinal(ordinal: i32) -> Self {
        match ordinal.rem_euclid(3) {
            0 => Self::X,
            1 => Self::Y,
            _ => Self::Z,
        }
    }

    pub fn choose_i32(self, x: i32, y: i32, z: i32) -> i32 {
        match self {
            Self::X => x,
            Self::Y => y,
            Self::Z => z,
        }
    }

    pub fn choose_f64(self, x: f64, y: f64, z: f64) -> f64 {
        match self {
            Self::X => x,
            Self::Y => y,
            Self::Z => z,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CoreDirectionModel {
    Down,
    Up,
    North,
    South,
    West,
    East,
}

impl CoreDirectionModel {
    pub fn ordinal(self) -> usize {
        match self {
            Self::Down => 0,
            Self::Up => 1,
            Self::North => 2,
            Self::South => 3,
            Self::West => 4,
            Self::East => 5,
        }
    }

    pub fn step(self) -> (i32, i32, i32) {
        match self {
            Self::Down => (0, -1, 0),
            Self::Up => (0, 1, 0),
            Self::North => (0, 0, -1),
            Self::South => (0, 0, 1),
            Self::West => (-1, 0, 0),
            Self::East => (1, 0, 0),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AxisCycleModel {
    None,
    Forward,
    Backward,
}

impl AxisCycleModel {
    pub fn between(from: CoreAxisModel, to: CoreAxisModel) -> Self {
        match (to.ordinal() - from.ordinal()).rem_euclid(3) {
            0 => Self::None,
            1 => Self::Forward,
            _ => Self::Backward,
        }
    }

    pub fn cycle_i32(self, x: i32, y: i32, z: i32, axis: CoreAxisModel) -> i32 {
        match self {
            Self::None => axis.choose_i32(x, y, z),
            Self::Forward => axis.choose_i32(z, x, y),
            Self::Backward => axis.choose_i32(y, z, x),
        }
    }

    pub fn cycle_f64(self, x: f64, y: f64, z: f64, axis: CoreAxisModel) -> f64 {
        match self {
            Self::None => axis.choose_f64(x, y, z),
            Self::Forward => axis.choose_f64(z, x, y),
            Self::Backward => axis.choose_f64(y, z, x),
        }
    }

    pub fn cycle_axis(self, axis: CoreAxisModel) -> CoreAxisModel {
        match self {
            Self::None => axis,
            Self::Forward => CoreAxisModel::by_ordinal(axis.ordinal() + 1),
            Self::Backward => CoreAxisModel::by_ordinal(axis.ordinal() - 1),
        }
    }

    pub fn inverse(self) -> Self {
        match self {
            Self::None => Self::None,
            Self::Forward => Self::Backward,
            Self::Backward => Self::Forward,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction8Model {
    North,
    NorthEast,
    East,
    SouthEast,
    South,
    SouthWest,
    West,
    NorthWest,
}

impl Direction8Model {
    pub fn directions(self) -> BTreeSet<CoreDirectionModel> {
        use CoreDirectionModel::{East, North, South, West};
        match self {
            Self::North => BTreeSet::from([North]),
            Self::NorthEast => BTreeSet::from([North, East]),
            Self::East => BTreeSet::from([East]),
            Self::SouthEast => BTreeSet::from([South, East]),
            Self::South => BTreeSet::from([South]),
            Self::SouthWest => BTreeSet::from([South, West]),
            Self::West => BTreeSet::from([West]),
            Self::NorthWest => BTreeSet::from([North, West]),
        }
    }

    pub fn step_x(self) -> i32 {
        self.directions()
            .iter()
            .map(|direction| direction.step().0)
            .sum()
    }

    pub fn step_z(self) -> i32 {
        self.directions()
            .iter()
            .map(|direction| direction.step().2)
            .sum()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrontAndTopModel {
    DownEast,
    DownNorth,
    DownSouth,
    DownWest,
    UpEast,
    UpNorth,
    UpSouth,
    UpWest,
    WestUp,
    EastUp,
    NorthUp,
    SouthUp,
}

impl FrontAndTopModel {
    pub fn serialized_name(self) -> &'static str {
        match self {
            Self::DownEast => "down_east",
            Self::DownNorth => "down_north",
            Self::DownSouth => "down_south",
            Self::DownWest => "down_west",
            Self::UpEast => "up_east",
            Self::UpNorth => "up_north",
            Self::UpSouth => "up_south",
            Self::UpWest => "up_west",
            Self::WestUp => "west_up",
            Self::EastUp => "east_up",
            Self::NorthUp => "north_up",
            Self::SouthUp => "south_up",
        }
    }

    pub fn front(self) -> CoreDirectionModel {
        match self {
            Self::DownEast | Self::DownNorth | Self::DownSouth | Self::DownWest => {
                CoreDirectionModel::Down
            }
            Self::UpEast | Self::UpNorth | Self::UpSouth | Self::UpWest => CoreDirectionModel::Up,
            Self::WestUp => CoreDirectionModel::West,
            Self::EastUp => CoreDirectionModel::East,
            Self::NorthUp => CoreDirectionModel::North,
            Self::SouthUp => CoreDirectionModel::South,
        }
    }

    pub fn top(self) -> CoreDirectionModel {
        match self {
            Self::DownEast | Self::UpEast => CoreDirectionModel::East,
            Self::DownNorth | Self::UpNorth => CoreDirectionModel::North,
            Self::DownSouth | Self::UpSouth => CoreDirectionModel::South,
            Self::DownWest | Self::UpWest => CoreDirectionModel::West,
            Self::WestUp | Self::EastUp | Self::NorthUp | Self::SouthUp => CoreDirectionModel::Up,
        }
    }

    pub fn from_front_and_top(front: CoreDirectionModel, top: CoreDirectionModel) -> Option<Self> {
        Self::all()
            .iter()
            .copied()
            .find(|value| value.front() == front && value.top() == top)
    }

    fn all() -> &'static [Self; 12] {
        &[
            Self::DownEast,
            Self::DownNorth,
            Self::DownSouth,
            Self::DownWest,
            Self::UpEast,
            Self::UpNorth,
            Self::UpSouth,
            Self::UpWest,
            Self::WestUp,
            Self::EastUp,
            Self::NorthUp,
            Self::SouthUp,
        ]
    }
}

pub struct QuartPosModel;

impl QuartPosModel {
    pub const BITS: i32 = 2;
    pub const SIZE: i32 = 4;
    pub const MASK: i32 = 3;

    pub fn from_block(block_coord: i32) -> i32 {
        block_coord >> 2
    }

    pub fn quart_local(block_coord: i32) -> i32 {
        block_coord & 3
    }

    pub fn to_block(quart: i32) -> i32 {
        quart << 2
    }

    pub fn from_section(section: i32) -> i32 {
        section << 2
    }

    pub fn to_section(quart: i32) -> i32 {
        quart >> 2
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RotationsModel {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl RotationsModel {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self {
            x: normalize_rotation(x),
            y: normalize_rotation(y),
            z: normalize_rotation(z),
        }
    }

    pub fn encode_network<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        writer.write_all(&self.x.to_be_bytes())?;
        writer.write_all(&self.y.to_be_bytes())?;
        writer.write_all(&self.z.to_be_bytes())
    }

    pub fn decode_network<R: Read>(reader: &mut R) -> io::Result<Self> {
        Ok(Self::new(
            read_f32(reader)?,
            read_f32(reader)?,
            read_f32(reader)?,
        ))
    }

    pub fn as_codec_list(self) -> [f32; 3] {
        [self.x, self.y, self.z]
    }

    pub fn from_codec_list(values: &[f32]) -> Result<Self, String> {
        if values.len() == 3 {
            Ok(Self::new(values[0], values[1], values[2]))
        } else {
            Err(format!(
                "Rotations requires exactly 3 floats, got {}",
                values.len()
            ))
        }
    }
}

fn normalize_rotation(value: f32) -> f32 {
    if value.is_finite() {
        value % 360.0
    } else {
        0.0
    }
}

fn read_f32<R: Read>(reader: &mut R) -> io::Result<f32> {
    let mut bytes = [0u8; 4];
    reader.read_exact(&mut bytes)?;
    Ok(f32::from_be_bytes(bytes))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn axis_cycle_matches_java_permutation_and_inverse() {
        assert_eq!(
            AxisCycleModel::between(CoreAxisModel::X, CoreAxisModel::X),
            AxisCycleModel::None
        );
        assert_eq!(
            AxisCycleModel::between(CoreAxisModel::X, CoreAxisModel::Y),
            AxisCycleModel::Forward
        );
        assert_eq!(
            AxisCycleModel::between(CoreAxisModel::Z, CoreAxisModel::Y),
            AxisCycleModel::Backward
        );
        assert_eq!(
            AxisCycleModel::Forward.cycle_axis(CoreAxisModel::Z),
            CoreAxisModel::X
        );
        assert_eq!(
            AxisCycleModel::Backward.cycle_axis(CoreAxisModel::X),
            CoreAxisModel::Z
        );
        assert_eq!(AxisCycleModel::Forward.inverse(), AxisCycleModel::Backward);
        assert_eq!(AxisCycleModel::Backward.inverse(), AxisCycleModel::Forward);
        assert_eq!(
            AxisCycleModel::Forward.cycle_i32(10, 20, 30, CoreAxisModel::X),
            30
        );
        assert_eq!(
            AxisCycleModel::Backward.cycle_f64(1.5, 2.5, 3.5, CoreAxisModel::Z),
            1.5
        );
    }

    #[test]
    fn direction8_steps_are_sum_of_java_cardinal_directions() {
        assert_eq!(CoreDirectionModel::Down.ordinal(), 0);
        assert_eq!(CoreDirectionModel::East.ordinal(), 5);
        assert_eq!(
            Direction8Model::NorthEast.directions(),
            BTreeSet::from([CoreDirectionModel::North, CoreDirectionModel::East])
        );
        assert_eq!(Direction8Model::NorthEast.step_x(), 1);
        assert_eq!(Direction8Model::NorthEast.step_z(), -1);
        assert_eq!(Direction8Model::SouthWest.step_x(), -1);
        assert_eq!(Direction8Model::SouthWest.step_z(), 1);
        assert_eq!(Direction8Model::North.step_x(), 0);
        assert_eq!(Direction8Model::North.step_z(), -1);
        assert_eq!(Direction8Model::East.step_x(), 1);
        assert_eq!(Direction8Model::East.step_z(), 0);
        assert_eq!(Direction8Model::SouthEast.step_x(), 1);
        assert_eq!(Direction8Model::SouthEast.step_z(), 1);
        assert_eq!(Direction8Model::South.step_x(), 0);
        assert_eq!(Direction8Model::South.step_z(), 1);
        assert_eq!(Direction8Model::West.step_x(), -1);
        assert_eq!(Direction8Model::West.step_z(), 0);
        assert_eq!(Direction8Model::NorthWest.step_x(), -1);
        assert_eq!(Direction8Model::NorthWest.step_z(), -1);
    }

    #[test]
    fn front_and_top_lookup_uses_java_front_top_key_pairs() {
        let value = FrontAndTopModel::from_front_and_top(
            CoreDirectionModel::Down,
            CoreDirectionModel::East,
        )
        .unwrap();
        assert_eq!(value, FrontAndTopModel::DownEast);
        assert_eq!(value.serialized_name(), "down_east");
        assert_eq!(value.front(), CoreDirectionModel::Down);
        assert_eq!(value.top(), CoreDirectionModel::East);

        assert_eq!(
            FrontAndTopModel::from_front_and_top(CoreDirectionModel::North, CoreDirectionModel::Up),
            Some(FrontAndTopModel::NorthUp)
        );
        assert_eq!(
            FrontAndTopModel::from_front_and_top(
                CoreDirectionModel::North,
                CoreDirectionModel::East
            ),
            None
        );
    }

    #[test]
    fn quart_pos_uses_java_shift_and_mask_semantics() {
        assert_eq!(QuartPosModel::BITS, 2);
        assert_eq!(QuartPosModel::SIZE, 4);
        assert_eq!(QuartPosModel::MASK, 3);
        assert_eq!(QuartPosModel::from_block(15), 3);
        assert_eq!(QuartPosModel::from_block(-1), -1);
        assert_eq!(QuartPosModel::quart_local(15), 3);
        assert_eq!(QuartPosModel::quart_local(-1), 3);
        assert_eq!(QuartPosModel::to_block(-2), -8);
        assert_eq!(QuartPosModel::from_section(-2), -8);
        assert_eq!(QuartPosModel::to_section(-5), -2);
    }

    #[test]
    fn position_interface_and_rotations_match_java_records() {
        let position = Vec3PositionModel {
            x: 1.25,
            y: -2.5,
            z: 3.75,
        };
        assert_eq!(position.x(), 1.25);
        assert_eq!(position.y(), -2.5);
        assert_eq!(position.z(), 3.75);

        let rotations = RotationsModel::new(725.0, -725.0, f32::NAN);
        assert_eq!(rotations.as_codec_list(), [5.0, -5.0, 0.0]);
        assert_eq!(
            RotationsModel::from_codec_list(&[f32::INFINITY, 361.0, -361.0]).unwrap(),
            RotationsModel::new(0.0, 1.0, -1.0)
        );
        assert!(RotationsModel::from_codec_list(&[1.0, 2.0]).is_err());

        let mut bytes = Vec::new();
        rotations.encode_network(&mut bytes).unwrap();
        assert_eq!(
            RotationsModel::decode_network(&mut &bytes[..]).unwrap(),
            rotations
        );
    }
}
