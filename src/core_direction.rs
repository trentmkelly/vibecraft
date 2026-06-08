#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec3fModel {
    x: f32,
    y: f32,
    z: f32,
}

impl Vec3fModel {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DirectionModel {
    Down,
    Up,
    North,
    South,
    West,
    East,
}

impl DirectionModel {
    pub const VALUES: [Self; 6] = [
        Self::Down,
        Self::Up,
        Self::North,
        Self::South,
        Self::West,
        Self::East,
    ];

    const BY_2D_DATA: [Self; 4] = [Self::South, Self::West, Self::North, Self::East];

    pub fn data_3d(self) -> i32 {
        match self {
            Self::Down => 0,
            Self::Up => 1,
            Self::North => 2,
            Self::South => 3,
            Self::West => 4,
            Self::East => 5,
        }
    }

    pub fn data_2d(self) -> i32 {
        match self {
            Self::South => 0,
            Self::West => 1,
            Self::North => 2,
            Self::East => 3,
            Self::Down | Self::Up => -1,
        }
    }

    pub fn axis(self) -> AxisModel {
        match self {
            Self::Down | Self::Up => AxisModel::Y,
            Self::North | Self::South => AxisModel::Z,
            Self::West | Self::East => AxisModel::X,
        }
    }

    pub fn axis_direction(self) -> AxisDirectionModel {
        match self {
            Self::Up | Self::South | Self::East => AxisDirectionModel::Positive,
            Self::Down | Self::North | Self::West => AxisDirectionModel::Negative,
        }
    }

    pub fn normal(self) -> (i32, i32, i32) {
        match self {
            Self::Down => (0, -1, 0),
            Self::Up => (0, 1, 0),
            Self::North => (0, 0, -1),
            Self::South => (0, 0, 1),
            Self::West => (-1, 0, 0),
            Self::East => (1, 0, 0),
        }
    }

    pub fn unit_vec3(self) -> Vec3fModel {
        let (x, y, z) = self.normal();
        Vec3fModel::new(x as f32, y as f32, z as f32)
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Down => "down",
            Self::Up => "up",
            Self::North => "north",
            Self::South => "south",
            Self::West => "west",
            Self::East => "east",
        }
    }

    pub fn by_name(name: &str) -> Option<Self> {
        Self::VALUES
            .into_iter()
            .find(|direction| direction.name() == name)
    }

    pub fn from_3d_data_value(data: i32) -> Self {
        Self::VALUES[(data % Self::VALUES.len() as i32).unsigned_abs() as usize]
    }

    pub fn from_2d_data_value(data: i32) -> Self {
        Self::BY_2D_DATA[(data % Self::BY_2D_DATA.len() as i32).unsigned_abs() as usize]
    }

    pub fn from_y_rot(y_rot: f64) -> Self {
        Self::from_2d_data_value(((y_rot / 90.0 + 0.5).floor() as i32) & 3)
    }

    pub fn from_axis_and_direction(axis: AxisModel, direction: AxisDirectionModel) -> Self {
        match (axis, direction) {
            (AxisModel::X, AxisDirectionModel::Positive) => Self::East,
            (AxisModel::X, AxisDirectionModel::Negative) => Self::West,
            (AxisModel::Y, AxisDirectionModel::Positive) => Self::Up,
            (AxisModel::Y, AxisDirectionModel::Negative) => Self::Down,
            (AxisModel::Z, AxisDirectionModel::Positive) => Self::South,
            (AxisModel::Z, AxisDirectionModel::Negative) => Self::North,
        }
    }

    pub fn get(axis_direction: AxisDirectionModel, axis: AxisModel) -> Self {
        Self::from_axis_and_direction(axis, axis_direction)
    }

    pub fn get_facing_axis(view_x_rot: f32, view_y_rot: f32, axis: AxisModel) -> Self {
        match axis {
            AxisModel::X => {
                if Self::East.is_facing_angle(view_y_rot) {
                    Self::East
                } else {
                    Self::West
                }
            }
            AxisModel::Y => {
                if view_x_rot < 0.0 {
                    Self::Up
                } else {
                    Self::Down
                }
            }
            AxisModel::Z => {
                if Self::South.is_facing_angle(view_y_rot) {
                    Self::South
                } else {
                    Self::North
                }
            }
        }
    }

    pub fn opposite(self) -> Self {
        match self {
            Self::Down => Self::Up,
            Self::Up => Self::Down,
            Self::North => Self::South,
            Self::South => Self::North,
            Self::West => Self::East,
            Self::East => Self::West,
        }
    }

    pub fn clockwise(self, axis: AxisModel) -> Self {
        match axis {
            AxisModel::X => {
                if matches!(self, Self::West | Self::East) {
                    self
                } else {
                    self.clockwise_x()
                }
            }
            AxisModel::Y => {
                if matches!(self, Self::Up | Self::Down) {
                    self
                } else {
                    self.clockwise_y()
                }
            }
            AxisModel::Z => {
                if matches!(self, Self::North | Self::South) {
                    self
                } else {
                    self.clockwise_z()
                }
            }
        }
    }

    pub fn counter_clockwise(self, axis: AxisModel) -> Self {
        match axis {
            AxisModel::X => {
                if matches!(self, Self::West | Self::East) {
                    self
                } else {
                    self.counter_clockwise_x()
                }
            }
            AxisModel::Y => {
                if matches!(self, Self::Up | Self::Down) {
                    self
                } else {
                    self.counter_clockwise_y()
                }
            }
            AxisModel::Z => {
                if matches!(self, Self::North | Self::South) {
                    self
                } else {
                    self.counter_clockwise_z()
                }
            }
        }
    }

    pub fn clockwise_y(self) -> Self {
        match self {
            Self::North => Self::East,
            Self::South => Self::West,
            Self::West => Self::North,
            Self::East => Self::South,
            Self::Down | Self::Up => panic!("Unable to get Y-rotated facing of {}", self.name()),
        }
    }

    pub fn counter_clockwise_y(self) -> Self {
        match self {
            Self::North => Self::West,
            Self::South => Self::East,
            Self::West => Self::South,
            Self::East => Self::North,
            Self::Down | Self::Up => panic!("Unable to get CCW facing of {}", self.name()),
        }
    }

    fn clockwise_x(self) -> Self {
        match self {
            Self::Down => Self::South,
            Self::Up => Self::North,
            Self::North => Self::Down,
            Self::South => Self::Up,
            Self::West | Self::East => panic!("Unable to get X-rotated facing of {}", self.name()),
        }
    }

    fn counter_clockwise_x(self) -> Self {
        match self {
            Self::Down => Self::North,
            Self::Up => Self::South,
            Self::North => Self::Up,
            Self::South => Self::Down,
            Self::West | Self::East => panic!("Unable to get X-rotated facing of {}", self.name()),
        }
    }

    fn clockwise_z(self) -> Self {
        match self {
            Self::Down => Self::West,
            Self::Up => Self::East,
            Self::West => Self::Up,
            Self::East => Self::Down,
            Self::North | Self::South => {
                panic!("Unable to get Z-rotated facing of {}", self.name())
            }
        }
    }

    fn counter_clockwise_z(self) -> Self {
        match self {
            Self::Down => Self::East,
            Self::Up => Self::West,
            Self::West => Self::Down,
            Self::East => Self::Up,
            Self::North | Self::South => {
                panic!("Unable to get Z-rotated facing of {}", self.name())
            }
        }
    }

    pub fn step_x(self) -> i32 {
        self.normal().0
    }

    pub fn step_y(self) -> i32 {
        self.normal().1
    }

    pub fn step_z(self) -> i32 {
        self.normal().2
    }

    pub fn y_rot(direction: Self) -> Result<f32, String> {
        match direction {
            Self::North => Ok(180.0),
            Self::South => Ok(0.0),
            Self::West => Ok(90.0),
            Self::East => Ok(-90.0),
            Self::Down | Self::Up => {
                Err(format!("No y-Rot for vertical axis: {}", direction.name()))
            }
        }
    }

    pub fn to_y_rot(self) -> f32 {
        ((self.data_2d() & 3) * 90) as f32
    }

    pub fn rotation(self) -> RotationModel {
        match self {
            Self::Down => RotationModel::RotationX180,
            Self::Up => RotationModel::Identity,
            Self::North => RotationModel::RotationX90Z180,
            Self::South => RotationModel::RotationX90,
            Self::West => RotationModel::RotationX90Z90,
            Self::East => RotationModel::RotationX90ZNegative90,
        }
    }

    pub fn rotate(matrix: Matrix3Model, facing: Self) -> Self {
        let (x, y, z) = facing.normal();
        let transformed = matrix.transform_direction(Vec3fModel::new(x as f32, y as f32, z as f32));
        Self::approximate_nearest(transformed.x, transformed.y, transformed.z)
    }

    pub fn ordered_by_nearest(view_x_rot: f32, view_y_rot: f32) -> [Self; 6] {
        let pitch = view_x_rot * std::f32::consts::PI / 180.0;
        let yaw = -view_y_rot * std::f32::consts::PI / 180.0;
        let pitch_sin = pitch.sin();
        let pitch_cos = pitch.cos();
        let yaw_sin = yaw.sin();
        let yaw_cos = yaw.cos();
        let x_pos = yaw_sin > 0.0;
        let y_pos = pitch_sin < 0.0;
        let z_pos = yaw_cos > 0.0;
        let x_yaw = if x_pos { yaw_sin } else { -yaw_sin };
        let y_mag = if y_pos { -pitch_sin } else { pitch_sin };
        let z_yaw = if z_pos { yaw_cos } else { -yaw_cos };
        let x_mag = x_yaw * pitch_cos;
        let z_mag = z_yaw * pitch_cos;
        let axis_x = if x_pos { Self::East } else { Self::West };
        let axis_y = if y_pos { Self::Up } else { Self::Down };
        let axis_z = if z_pos { Self::South } else { Self::North };

        if x_yaw > z_yaw {
            if y_mag > x_mag {
                Self::make_direction_array(axis_y, axis_x, axis_z)
            } else if z_mag > y_mag {
                Self::make_direction_array(axis_x, axis_z, axis_y)
            } else {
                Self::make_direction_array(axis_x, axis_y, axis_z)
            }
        } else if y_mag > z_mag {
            Self::make_direction_array(axis_y, axis_z, axis_x)
        } else if x_mag > y_mag {
            Self::make_direction_array(axis_z, axis_x, axis_y)
        } else {
            Self::make_direction_array(axis_z, axis_y, axis_x)
        }
    }

    fn make_direction_array(axis1: Self, axis2: Self, axis3: Self) -> [Self; 6] {
        [
            axis1,
            axis2,
            axis3,
            axis3.opposite(),
            axis2.opposite(),
            axis1.opposite(),
        ]
    }

    pub fn approximate_nearest(dx: f32, dy: f32, dz: f32) -> Self {
        let mut result = Self::North;
        let mut highest_dot = f32::MIN_POSITIVE;
        for direction in Self::VALUES {
            let (x, y, z) = direction.normal();
            let dot = dx * x as f32 + dy * y as f32 + dz * z as f32;
            if dot > highest_dot {
                highest_dot = dot;
                result = direction;
            }
        }
        result
    }

    pub fn nearest(x: i32, y: i32, z: i32, or_else: Option<Self>) -> Option<Self> {
        let abs_x = x.abs();
        let abs_y = y.abs();
        let abs_z = z.abs();
        if abs_x > abs_z && abs_x > abs_y {
            Some(if x < 0 { Self::West } else { Self::East })
        } else if abs_z > abs_x && abs_z > abs_y {
            Some(if z < 0 { Self::North } else { Self::South })
        } else if abs_y > abs_x && abs_y > abs_z {
            Some(if y < 0 { Self::Down } else { Self::Up })
        } else {
            or_else
        }
    }

    pub fn axis_step_order(movement: Vec3fModel) -> [AxisModel; 3] {
        if movement.x.abs() < movement.z.abs() {
            [AxisModel::Y, AxisModel::Z, AxisModel::X]
        } else {
            [AxisModel::Y, AxisModel::X, AxisModel::Z]
        }
    }

    pub fn is_facing_angle(self, y_angle: f32) -> bool {
        let radians = y_angle * std::f32::consts::PI / 180.0;
        let dx = -radians.sin();
        let dz = radians.cos();
        let (x, _, z) = self.normal();
        x as f32 * dx + z as f32 * dz > 0.0
    }

    pub fn stream() -> Vec<Self> {
        Self::VALUES.to_vec()
    }

    pub fn all_shuffled(seed: u64) -> Vec<Self> {
        shuffled(Self::VALUES, seed)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RotationModel {
    Identity,
    RotationX180,
    RotationX90,
    RotationX90Z180,
    RotationX90Z90,
    RotationX90ZNegative90,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Matrix3Model {
    rows: [[f32; 3]; 3],
}

impl Matrix3Model {
    pub fn identity() -> Self {
        Self {
            rows: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
        }
    }

    pub fn rotate_y_90() -> Self {
        Self {
            rows: [[0.0, 0.0, 1.0], [0.0, 1.0, 0.0], [-1.0, 0.0, 0.0]],
        }
    }

    pub fn transform_direction(self, vec: Vec3fModel) -> Vec3fModel {
        Vec3fModel::new(
            self.rows[0][0] * vec.x + self.rows[0][1] * vec.y + self.rows[0][2] * vec.z,
            self.rows[1][0] * vec.x + self.rows[1][1] * vec.y + self.rows[1][2] * vec.z,
            self.rows[2][0] * vec.x + self.rows[2][1] * vec.y + self.rows[2][2] * vec.z,
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AxisModel {
    X,
    Y,
    Z,
}

impl AxisModel {
    pub const VALUES: [Self; 3] = [Self::X, Self::Y, Self::Z];

    pub fn by_name(name: &str) -> Option<Self> {
        Self::VALUES.into_iter().find(|axis| axis.name() == name)
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::X => "x",
            Self::Y => "y",
            Self::Z => "z",
        }
    }

    pub fn is_vertical(self) -> bool {
        self == Self::Y
    }

    pub fn is_horizontal(self) -> bool {
        matches!(self, Self::X | Self::Z)
    }

    pub fn positive(self) -> DirectionModel {
        DirectionModel::from_axis_and_direction(self, AxisDirectionModel::Positive)
    }

    pub fn negative(self) -> DirectionModel {
        DirectionModel::from_axis_and_direction(self, AxisDirectionModel::Negative)
    }

    pub fn directions(self) -> [DirectionModel; 2] {
        [self.positive(), self.negative()]
    }

    pub fn plane(self) -> PlaneModel {
        match self {
            Self::X | Self::Z => PlaneModel::Horizontal,
            Self::Y => PlaneModel::Vertical,
        }
    }

    pub fn test(self, input: Option<DirectionModel>) -> bool {
        input.is_some_and(|direction| direction.axis() == self)
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

    pub fn choose_bool(self, x: bool, y: bool, z: bool) -> bool {
        match self {
            Self::X => x,
            Self::Y => y,
            Self::Z => z,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AxisDirectionModel {
    Positive,
    Negative,
}

impl AxisDirectionModel {
    pub fn step(self) -> i32 {
        match self {
            Self::Positive => 1,
            Self::Negative => -1,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Positive => "Towards positive",
            Self::Negative => "Towards negative",
        }
    }

    pub fn opposite(self) -> Self {
        match self {
            Self::Positive => Self::Negative,
            Self::Negative => Self::Positive,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaneModel {
    Horizontal,
    Vertical,
}

impl PlaneModel {
    pub fn faces(self) -> &'static [DirectionModel] {
        match self {
            Self::Horizontal => &[
                DirectionModel::North,
                DirectionModel::East,
                DirectionModel::South,
                DirectionModel::West,
            ],
            Self::Vertical => &[DirectionModel::Up, DirectionModel::Down],
        }
    }

    pub fn axes(self) -> &'static [AxisModel] {
        match self {
            Self::Horizontal => &[AxisModel::X, AxisModel::Z],
            Self::Vertical => &[AxisModel::Y],
        }
    }

    pub fn test(self, input: Option<DirectionModel>) -> bool {
        input.is_some_and(|direction| direction.axis().plane() == self)
    }

    pub fn stream(self) -> Vec<DirectionModel> {
        self.faces().to_vec()
    }

    pub fn length(self) -> usize {
        self.faces().len()
    }

    pub fn shuffled_copy(self, seed: u64) -> Vec<DirectionModel> {
        let mut faces = self.faces().to_vec();
        shuffle_vec(&mut faces, seed);
        faces
    }
}

fn shuffled<const N: usize>(values: [DirectionModel; N], seed: u64) -> Vec<DirectionModel> {
    let mut values = values.to_vec();
    shuffle_vec(&mut values, seed);
    values
}

fn shuffle_vec(values: &mut [DirectionModel], seed: u64) {
    let mut rng = LcgModel::new(seed);
    for index in (1..values.len()).rev() {
        let swap = rng.next_usize(index + 1);
        values.swap(index, swap);
    }
}

#[derive(Debug, Clone, Copy)]
struct LcgModel {
    state: u64,
}

impl LcgModel {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next_usize(&mut self, bound: usize) -> usize {
        self.state = self
            .state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1);
        ((self.state >> 32) as usize) % bound
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn direction_constants_ids_names_axes_and_normals_match_java() {
        assert_eq!(
            DirectionModel::stream(),
            vec![
                DirectionModel::Down,
                DirectionModel::Up,
                DirectionModel::North,
                DirectionModel::South,
                DirectionModel::West,
                DirectionModel::East,
            ]
        );
        assert_eq!(DirectionModel::Down.data_3d(), 0);
        assert_eq!(DirectionModel::Up.data_3d(), 1);
        assert_eq!(DirectionModel::North.data_2d(), 2);
        assert_eq!(DirectionModel::East.data_2d(), 3);
        assert_eq!(DirectionModel::Down.data_2d(), -1);
        assert_eq!(DirectionModel::North.name(), "north");
        assert_eq!(DirectionModel::by_name("west"), Some(DirectionModel::West));
        assert_eq!(DirectionModel::by_name("missing"), None);
        assert_eq!(DirectionModel::West.axis(), AxisModel::X);
        assert_eq!(DirectionModel::Down.axis(), AxisModel::Y);
        assert_eq!(
            DirectionModel::South.axis_direction(),
            AxisDirectionModel::Positive
        );
        assert_eq!(
            DirectionModel::North.axis_direction(),
            AxisDirectionModel::Negative
        );
        assert_eq!(DirectionModel::East.normal(), (1, 0, 0));
        assert_eq!(
            DirectionModel::Down.unit_vec3(),
            Vec3fModel::new(0.0, -1.0, 0.0)
        );
        assert_eq!(DirectionModel::West.step_x(), -1);
        assert_eq!(DirectionModel::Up.step_y(), 1);
        assert_eq!(DirectionModel::North.step_z(), -1);
    }

    #[test]
    fn direction_lookup_rotation_and_yaw_helpers_match_java() {
        assert_eq!(DirectionModel::from_3d_data_value(7), DirectionModel::Up);
        assert_eq!(DirectionModel::from_3d_data_value(-7), DirectionModel::Up);
        assert_eq!(DirectionModel::from_2d_data_value(0), DirectionModel::South);
        assert_eq!(DirectionModel::from_2d_data_value(-1), DirectionModel::West);
        assert_eq!(DirectionModel::from_y_rot(44.9), DirectionModel::South);
        assert_eq!(DirectionModel::from_y_rot(45.0), DirectionModel::West);
        assert_eq!(
            DirectionModel::from_axis_and_direction(AxisModel::Z, AxisDirectionModel::Negative),
            DirectionModel::North
        );
        assert_eq!(
            DirectionModel::get(AxisDirectionModel::Positive, AxisModel::Y),
            DirectionModel::Up
        );
        assert_eq!(DirectionModel::North.opposite(), DirectionModel::South);
        assert_eq!(DirectionModel::East.to_y_rot(), 270.0);
        assert_eq!(DirectionModel::y_rot(DirectionModel::West).unwrap(), 90.0);
        assert_eq!(
            DirectionModel::y_rot(DirectionModel::Up).unwrap_err(),
            "No y-Rot for vertical axis: up"
        );
        assert_eq!(DirectionModel::Down.rotation(), RotationModel::RotationX180);
        assert_eq!(
            DirectionModel::East.rotation(),
            RotationModel::RotationX90ZNegative90
        );
        assert_eq!(
            DirectionModel::rotate(Matrix3Model::identity(), DirectionModel::South),
            DirectionModel::South
        );
        assert_eq!(
            DirectionModel::rotate(Matrix3Model::rotate_y_90(), DirectionModel::South),
            DirectionModel::East
        );
    }

    #[test]
    fn direction_clockwise_counterclockwise_and_facing_axis_match_java() {
        assert_eq!(DirectionModel::North.clockwise_y(), DirectionModel::East);
        assert_eq!(
            DirectionModel::South.counter_clockwise_y(),
            DirectionModel::East
        );
        assert_eq!(
            DirectionModel::Down.clockwise(AxisModel::X),
            DirectionModel::South
        );
        assert_eq!(
            DirectionModel::Up.counter_clockwise(AxisModel::X),
            DirectionModel::South
        );
        assert_eq!(
            DirectionModel::Down.clockwise(AxisModel::Z),
            DirectionModel::West
        );
        assert_eq!(
            DirectionModel::East.counter_clockwise(AxisModel::Z),
            DirectionModel::Up
        );
        assert_eq!(
            DirectionModel::East.clockwise(AxisModel::X),
            DirectionModel::East
        );
        assert_eq!(
            DirectionModel::Up.counter_clockwise(AxisModel::Y),
            DirectionModel::Up
        );
        assert_eq!(
            DirectionModel::get_facing_axis(-10.0, -90.0, AxisModel::Y),
            DirectionModel::Up
        );
        assert_eq!(
            DirectionModel::get_facing_axis(20.0, -90.0, AxisModel::X),
            DirectionModel::East
        );
        assert_eq!(
            DirectionModel::get_facing_axis(20.0, 180.0, AxisModel::Z),
            DirectionModel::North
        );
    }

    #[test]
    fn direction_nearest_order_and_angle_helpers_match_java() {
        assert_eq!(
            DirectionModel::ordered_by_nearest(0.0, 0.0),
            [
                DirectionModel::South,
                DirectionModel::Down,
                DirectionModel::West,
                DirectionModel::East,
                DirectionModel::Up,
                DirectionModel::North,
            ]
        );
        assert_eq!(
            DirectionModel::ordered_by_nearest(-90.0, 45.0)[0],
            DirectionModel::Up
        );
        assert_eq!(
            DirectionModel::approximate_nearest(0.0, 0.0, 0.0),
            DirectionModel::North
        );
        assert_eq!(
            DirectionModel::approximate_nearest(-0.3, 0.9, 0.1),
            DirectionModel::Up
        );
        assert_eq!(
            DirectionModel::nearest(5, 2, 3, None),
            Some(DirectionModel::East)
        );
        assert_eq!(
            DirectionModel::nearest(5, 5, 0, Some(DirectionModel::North)),
            Some(DirectionModel::North)
        );
        assert_eq!(DirectionModel::nearest(0, 0, 0, None), None);
        assert!(DirectionModel::South.is_facing_angle(0.0));
        assert!(!DirectionModel::North.is_facing_angle(0.0));
        assert_eq!(
            DirectionModel::axis_step_order(Vec3fModel::new(2.0, 0.0, 3.0)),
            [AxisModel::Y, AxisModel::Z, AxisModel::X]
        );
        assert_eq!(
            DirectionModel::axis_step_order(Vec3fModel::new(3.0, 0.0, 2.0)),
            [AxisModel::Y, AxisModel::X, AxisModel::Z]
        );
    }

    #[test]
    fn direction_axis_nested_type_matches_java() {
        assert_eq!(AxisModel::by_name("x"), Some(AxisModel::X));
        assert_eq!(AxisModel::by_name("missing"), None);
        assert_eq!(AxisModel::Y.name(), "y");
        assert!(AxisModel::Y.is_vertical());
        assert!(!AxisModel::Y.is_horizontal());
        assert!(AxisModel::X.is_horizontal());
        assert_eq!(AxisModel::X.positive(), DirectionModel::East);
        assert_eq!(AxisModel::Z.negative(), DirectionModel::North);
        assert_eq!(
            AxisModel::Y.directions(),
            [DirectionModel::Up, DirectionModel::Down]
        );
        assert_eq!(AxisModel::X.plane(), PlaneModel::Horizontal);
        assert_eq!(AxisModel::Y.plane(), PlaneModel::Vertical);
        assert!(AxisModel::Z.test(Some(DirectionModel::South)));
        assert!(!AxisModel::Z.test(Some(DirectionModel::East)));
        assert!(!AxisModel::Z.test(None));
        assert_eq!(AxisModel::X.choose_i32(1, 2, 3), 1);
        assert_eq!(AxisModel::Y.choose_f64(1.0, 2.0, 3.0), 2.0);
        assert!(AxisModel::Z.choose_bool(false, false, true));
    }

    #[test]
    fn direction_axis_direction_and_plane_match_java() {
        assert_eq!(AxisDirectionModel::Positive.step(), 1);
        assert_eq!(AxisDirectionModel::Negative.step(), -1);
        assert_eq!(AxisDirectionModel::Positive.name(), "Towards positive");
        assert_eq!(
            AxisDirectionModel::Positive.opposite(),
            AxisDirectionModel::Negative
        );
        assert_eq!(
            PlaneModel::Horizontal.stream(),
            vec![
                DirectionModel::North,
                DirectionModel::East,
                DirectionModel::South,
                DirectionModel::West,
            ]
        );
        assert_eq!(
            PlaneModel::Vertical.stream(),
            vec![DirectionModel::Up, DirectionModel::Down]
        );
        assert_eq!(PlaneModel::Horizontal.axes(), &[AxisModel::X, AxisModel::Z]);
        assert!(PlaneModel::Horizontal.test(Some(DirectionModel::West)));
        assert!(!PlaneModel::Horizontal.test(Some(DirectionModel::Up)));
        assert!(!PlaneModel::Vertical.test(None));
        assert_eq!(PlaneModel::Horizontal.length(), 4);
        let shuffled = DirectionModel::all_shuffled(7);
        assert_eq!(shuffled.len(), 6);
        for direction in DirectionModel::VALUES {
            assert!(shuffled.contains(&direction));
        }
        let plane_shuffled = PlaneModel::Horizontal.shuffled_copy(11);
        assert_eq!(plane_shuffled.len(), 4);
        for direction in PlaneModel::Horizontal.faces() {
            assert!(plane_shuffled.contains(direction));
        }
    }
}
