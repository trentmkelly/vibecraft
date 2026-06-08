use std::collections::{BTreeSet, VecDeque};

use crate::core_direction::{AxisModel, DirectionModel};
use crate::core_misc::Vec3iModel;
use crate::core_orientation::AxisCycleModel;
use crate::lighting::positions::{
    block_pos_as_long, block_pos_offset, block_pos_x, block_pos_y, block_pos_z,
};

pub const PACKED_HORIZONTAL_LENGTH: i32 = 26;
pub const PACKED_Y_LENGTH: i32 = 12;
pub const MAX_HORIZONTAL_COORDINATE: i32 = 33_554_431;
pub const BLOCK_POS_STREAM_CODEC_KIND: &str = "packed_long";

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec3dModel {
    x: f64,
    y: f64,
    z: f64,
}

impl Vec3dModel {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    fn get(self, axis: AxisModel) -> f64 {
        match axis {
            AxisModel::X => self.x,
            AxisModel::Y => self.y,
            AxisModel::Z => self.z,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AabbRangeModel {
    min: Vec3dModel,
    max: Vec3dModel,
}

impl AabbRangeModel {
    pub fn new(min: Vec3dModel, max: Vec3dModel) -> Self {
        Self { min, max }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RotationModel {
    None,
    Clockwise90,
    Clockwise180,
    Counterclockwise90,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TraversalNodeStatusModel {
    Accept,
    Skip,
    Stop,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct BlockPosModel {
    x: i32,
    y: i32,
    z: i32,
}

impl BlockPosModel {
    pub const ZERO: Self = Self { x: 0, y: 0, z: 0 };

    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    pub fn from_vec3i(vec: Vec3iModel) -> Self {
        Self::new(
            vec.get(crate::core_orientation::CoreAxisModel::X),
            vec.get(crate::core_orientation::CoreAxisModel::Y),
            vec.get(crate::core_orientation::CoreAxisModel::Z),
        )
    }

    pub fn containing(x: f64, y: f64, z: f64) -> Self {
        Self::new(x.floor() as i32, y.floor() as i32, z.floor() as i32)
    }

    pub fn min(a: Self, b: Self) -> Self {
        Self::new(a.x.min(b.x), a.y.min(b.y), a.z.min(b.z))
    }

    pub fn max(a: Self, b: Self) -> Self {
        Self::new(a.x.max(b.x), a.y.max(b.y), a.z.max(b.z))
    }

    pub fn as_long_coords(x: i32, y: i32, z: i32) -> i64 {
        block_pos_as_long(x, y, z)
    }

    pub fn as_long(self) -> i64 {
        block_pos_as_long(self.x, self.y, self.z)
    }

    pub fn of_long(block_node: i64) -> Self {
        Self::new(
            block_pos_x(block_node),
            block_pos_y(block_node),
            block_pos_z(block_node),
        )
    }

    pub fn x_from_long(block_node: i64) -> i32 {
        block_pos_x(block_node)
    }

    pub fn y_from_long(block_node: i64) -> i32 {
        block_pos_y(block_node)
    }

    pub fn z_from_long(block_node: i64) -> i32 {
        block_pos_z(block_node)
    }

    pub fn offset_long(block_node: i64, direction: DirectionModel) -> i64 {
        let (x, y, z) = direction.normal();
        block_pos_offset(block_node, x, y, z)
    }

    pub fn offset_long_by(block_node: i64, x: i32, y: i32, z: i32) -> i64 {
        block_pos_offset(block_node, x, y, z)
    }

    pub fn flat_index(block_node: i64) -> i64 {
        block_node & -16
    }

    pub fn x(self) -> i32 {
        self.x
    }

    pub fn y(self) -> i32 {
        self.y
    }

    pub fn z(self) -> i32 {
        self.z
    }

    pub fn to_vec3i(self) -> Vec3iModel {
        Vec3iModel::new(self.x, self.y, self.z)
    }

    pub fn offset(self, x: i32, y: i32, z: i32) -> Self {
        if x == 0 && y == 0 && z == 0 {
            self
        } else {
            Self::new(self.x + x, self.y + y, self.z + z)
        }
    }

    pub fn offset_vec(self, vec: Vec3iModel) -> Self {
        self.offset(
            vec.get(crate::core_orientation::CoreAxisModel::X),
            vec.get(crate::core_orientation::CoreAxisModel::Y),
            vec.get(crate::core_orientation::CoreAxisModel::Z),
        )
    }

    pub fn subtract(self, vec: Vec3iModel) -> Self {
        self.offset(
            -vec.get(crate::core_orientation::CoreAxisModel::X),
            -vec.get(crate::core_orientation::CoreAxisModel::Y),
            -vec.get(crate::core_orientation::CoreAxisModel::Z),
        )
    }

    pub fn multiply(self, scale: i32) -> Self {
        match scale {
            0 => Self::ZERO,
            1 => self,
            _ => Self::new(self.x * scale, self.y * scale, self.z * scale),
        }
    }

    pub fn center(self) -> Vec3dModel {
        Vec3dModel::new(
            self.x as f64 + 0.5,
            self.y as f64 + 0.5,
            self.z as f64 + 0.5,
        )
    }

    pub fn bottom_center(self) -> Vec3dModel {
        Vec3dModel::new(self.x as f64 + 0.5, self.y as f64, self.z as f64 + 0.5)
    }

    pub fn relative(self, direction: DirectionModel, steps: i32) -> Self {
        if steps == 0 {
            return self;
        }
        let (x, y, z) = direction.normal();
        Self::new(self.x + x * steps, self.y + y * steps, self.z + z * steps)
    }

    pub fn relative_axis(self, axis: AxisModel, steps: i32) -> Self {
        if steps == 0 {
            return self;
        }
        let x = if axis == AxisModel::X { steps } else { 0 };
        let y = if axis == AxisModel::Y { steps } else { 0 };
        let z = if axis == AxisModel::Z { steps } else { 0 };
        Self::new(self.x + x, self.y + y, self.z + z)
    }

    pub fn rotate(self, rotation: RotationModel) -> Self {
        match rotation {
            RotationModel::Clockwise90 => Self::new(-self.z, self.y, self.x),
            RotationModel::Clockwise180 => Self::new(-self.x, self.y, -self.z),
            RotationModel::Counterclockwise90 => Self::new(self.z, self.y, -self.x),
            RotationModel::None => self,
        }
    }

    pub fn cross(self, other: Vec3iModel) -> Self {
        Self::from_vec3i(self.to_vec3i().cross(other))
    }

    pub fn at_y(self, y: i32) -> Self {
        Self::new(self.x, y, self.z)
    }

    pub fn immutable(self) -> Self {
        self
    }

    pub fn mutable(self) -> MutableBlockPosModel {
        MutableBlockPosModel::new(self.x, self.y, self.z)
    }

    pub fn clamp_location_within(self, location: Vec3dModel) -> Vec3dModel {
        let min_x = self.x as f64 + 1.0e-5;
        let min_y = self.y as f64 + 1.0e-5;
        let min_z = self.z as f64 + 1.0e-5;
        let max_x = self.x as f64 + 1.0 - 1.0e-5;
        let max_y = self.y as f64 + 1.0 - 1.0e-5;
        let max_z = self.z as f64 + 1.0 - 1.0e-5;
        Vec3dModel::new(
            location.x.clamp(min_x, max_x),
            location.y.clamp(min_y, max_y),
            location.z.clamp(min_z, max_z),
        )
    }

    pub fn random_in_cube(
        random: &mut SequenceRandomModel,
        limit: i32,
        center: Self,
        size_to_scan_in_all_directions: i32,
    ) -> Vec<Self> {
        Self::random_between_closed(
            random,
            limit,
            center.x - size_to_scan_in_all_directions,
            center.y - size_to_scan_in_all_directions,
            center.z - size_to_scan_in_all_directions,
            center.x + size_to_scan_in_all_directions,
            center.y + size_to_scan_in_all_directions,
            center.z + size_to_scan_in_all_directions,
        )
    }

    pub fn square_out_south_east(from: Self) -> Vec<Self> {
        vec![
            from,
            from.relative(DirectionModel::South, 1),
            from.relative(DirectionModel::East, 1),
            from.relative(DirectionModel::South, 1)
                .relative(DirectionModel::East, 1),
        ]
    }

    #[allow(clippy::too_many_arguments)]
    pub fn random_between_closed(
        random: &mut SequenceRandomModel,
        limit: i32,
        min_x: i32,
        min_y: i32,
        min_z: i32,
        max_x: i32,
        max_y: i32,
        max_z: i32,
    ) -> Vec<Self> {
        let width = max_x - min_x + 1;
        let height = max_y - min_y + 1;
        let depth = max_z - min_z + 1;
        (0..limit)
            .map(|_| {
                Self::new(
                    min_x + random.next_int(width),
                    min_y + random.next_int(height),
                    min_z + random.next_int(depth),
                )
            })
            .collect()
    }

    pub fn within_manhattan(origin: Self, reach_x: i32, reach_y: i32, reach_z: i32) -> Vec<Self> {
        let max_depth = reach_x + reach_y + reach_z;
        let mut out = Vec::new();
        for current_depth in 0..=max_depth {
            let max_x = reach_x.min(current_depth);
            for x in -max_x..=max_x {
                let max_y = reach_y.min(current_depth - x.abs());
                for y in -max_y..=max_y {
                    let z = current_depth - x.abs() - y.abs();
                    if z <= reach_z {
                        out.push(Self::new(origin.x + x, origin.y + y, origin.z + z));
                        if z != 0 {
                            out.push(Self::new(origin.x + x, origin.y + y, origin.z - z));
                        }
                    }
                }
            }
        }
        out
    }

    pub fn find_closest_match(
        start_pos: Self,
        horizontal_search_radius: i32,
        vertical_search_radius: i32,
        predicate: impl Fn(Self) -> bool,
    ) -> Option<Self> {
        Self::within_manhattan(
            start_pos,
            horizontal_search_radius,
            vertical_search_radius,
            horizontal_search_radius,
        )
        .into_iter()
        .find(|pos| predicate(*pos))
    }

    pub fn between_closed(a: Self, b: Self) -> Vec<Self> {
        Self::between_closed_coords(
            a.x.min(b.x),
            a.y.min(b.y),
            a.z.min(b.z),
            a.x.max(b.x),
            a.y.max(b.y),
            a.z.max(b.z),
        )
    }

    pub fn between_closed_aabb(box_: AabbRangeModel) -> Vec<Self> {
        Self::between_closed(
            Self::containing(box_.min.x, box_.min.y, box_.min.z),
            Self::containing(box_.max.x, box_.max.y, box_.max.z),
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn between_closed_coords(
        min_x: i32,
        min_y: i32,
        min_z: i32,
        max_x: i32,
        max_y: i32,
        max_z: i32,
    ) -> Vec<Self> {
        let width = max_x - min_x + 1;
        let height = max_y - min_y + 1;
        let depth = max_z - min_z + 1;
        let end = width * height * depth;
        (0..end)
            .map(|index| {
                let x = index % width;
                let slice = index / width;
                let y = slice % height;
                let z = slice / height;
                Self::new(min_x + x, min_y + y, min_z + z)
            })
            .collect()
    }

    pub fn spiral_around(
        center: Self,
        radius: i32,
        first_direction: DirectionModel,
        second_direction: DirectionModel,
    ) -> Result<Vec<Self>, String> {
        if first_direction.axis() == second_direction.axis() {
            return Err("The two directions cannot be on the same axis".to_string());
        }
        let directions = [
            first_direction,
            second_direction,
            first_direction.opposite(),
            second_direction.opposite(),
        ];
        let mut cursor = center.mutable();
        cursor.move_direction(second_direction, 1);
        let legs = 4 * radius;
        let mut leg = -1;
        let mut leg_size = 0;
        let mut leg_index = 0;
        let mut last = cursor.to_block_pos();
        let mut out = Vec::new();
        loop {
            cursor
                .set_pos(last)
                .move_direction(directions[((leg + 4) % 4) as usize], 1);
            last = cursor.to_block_pos();
            if leg_index >= leg_size {
                if leg >= legs {
                    break;
                }
                leg += 1;
                leg_index = 0;
                leg_size = leg / 2 + 1;
            }
            leg_index += 1;
            out.push(cursor.to_block_pos());
        }
        Ok(out)
    }

    pub fn breadth_first_traversal(
        start_pos: Self,
        max_depth: i32,
        max_count: i32,
        neighbour_provider: impl Fn(Self) -> Vec<Self>,
        node_processor: impl Fn(Self) -> TraversalNodeStatusModel,
    ) -> i32 {
        let mut nodes = VecDeque::from([(start_pos, 0)]);
        let mut visited = BTreeSet::new();
        let mut count = 0;
        while let Some((current_pos, depth)) = nodes.pop_front() {
            if visited.insert(current_pos.as_long()) {
                let next = node_processor(current_pos);
                if next != TraversalNodeStatusModel::Skip {
                    if next == TraversalNodeStatusModel::Stop {
                        break;
                    }
                    count += 1;
                    if count >= max_count {
                        return count;
                    }
                    if depth < max_depth {
                        for pos in neighbour_provider(current_pos) {
                            nodes.push_back((pos, depth + 1));
                        }
                    }
                }
            }
        }
        count
    }

    pub fn between_corners_in_direction(a: Self, b: Self, direction: Vec3dModel) -> Vec<Self> {
        let min_x = a.x.min(b.x);
        let min_y = a.y.min(b.y);
        let min_z = a.z.min(b.z);
        let max_x = a.x.max(b.x);
        let max_y = a.y.max(b.y);
        let max_z = a.z.max(b.z);
        let diff_x = max_x - min_x;
        let diff_y = max_y - min_y;
        let diff_z = max_z - min_z;
        let start_x = if direction.x >= 0.0 { min_x } else { max_x };
        let start_y = if direction.y >= 0.0 { min_y } else { max_y };
        let start_z = if direction.z >= 0.0 { min_z } else { max_z };
        let axes = direction_axis_step_order(direction);
        let first_axis = axes[0];
        let second_axis = axes[1];
        let third_axis = axes[2];
        let first_dir = axis_dir_for_component(first_axis, direction);
        let second_dir = axis_dir_for_component(second_axis, direction);
        let third_dir = axis_dir_for_component(third_axis, direction);
        let first_max = axis_choose_i32(first_axis, diff_x, diff_y, diff_z);
        let second_max = axis_choose_i32(second_axis, diff_x, diff_y, diff_z);
        let third_max = axis_choose_i32(third_axis, diff_x, diff_y, diff_z);
        let mut out = Vec::new();
        for first_index in 0..=first_max {
            for second_index in 0..=second_max {
                for third_index in 0..=third_max {
                    out.push(Self::new(
                        start_x
                            + first_dir.step_x() * first_index
                            + second_dir.step_x() * second_index
                            + third_dir.step_x() * third_index,
                        start_y
                            + first_dir.step_y() * first_index
                            + second_dir.step_y() * second_index
                            + third_dir.step_y() * third_index,
                        start_z
                            + first_dir.step_z() * first_index
                            + second_dir.step_z() * second_index
                            + third_dir.step_z() * third_index,
                    ));
                }
            }
        }
        out
    }
}

fn direction_axis_step_order(movement: Vec3dModel) -> [AxisModel; 3] {
    if movement.x.abs() < movement.z.abs() {
        [AxisModel::Y, AxisModel::Z, AxisModel::X]
    } else {
        [AxisModel::Y, AxisModel::X, AxisModel::Z]
    }
}

fn axis_dir_for_component(axis: AxisModel, direction: Vec3dModel) -> DirectionModel {
    if direction.get(axis) >= 0.0 {
        axis.positive()
    } else {
        axis.negative()
    }
}

fn axis_choose_i32(axis: AxisModel, x: i32, y: i32, z: i32) -> i32 {
    match axis {
        AxisModel::X => x,
        AxisModel::Y => y,
        AxisModel::Z => z,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MutableBlockPosModel {
    x: i32,
    y: i32,
    z: i32,
}

impl MutableBlockPosModel {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    pub fn from_f64(x: f64, y: f64, z: f64) -> Self {
        Self::new(x.floor() as i32, y.floor() as i32, z.floor() as i32)
    }

    pub fn to_block_pos(&self) -> BlockPosModel {
        BlockPosModel::new(self.x, self.y, self.z)
    }

    pub fn set(&mut self, x: i32, y: i32, z: i32) -> &mut Self {
        self.x = x;
        self.y = y;
        self.z = z;
        self
    }

    pub fn set_f64(&mut self, x: f64, y: f64, z: f64) -> &mut Self {
        self.set(x.floor() as i32, y.floor() as i32, z.floor() as i32)
    }

    pub fn set_pos(&mut self, pos: BlockPosModel) -> &mut Self {
        self.set(pos.x, pos.y, pos.z)
    }

    pub fn set_long(&mut self, pos: i64) -> &mut Self {
        self.set(block_pos_x(pos), block_pos_y(pos), block_pos_z(pos))
    }

    pub fn set_axis_cycle(
        &mut self,
        transform: AxisCycleModel,
        x: i32,
        y: i32,
        z: i32,
    ) -> &mut Self {
        self.set(
            cycle_axis(transform, x, y, z, AxisModel::X),
            cycle_axis(transform, x, y, z, AxisModel::Y),
            cycle_axis(transform, x, y, z, AxisModel::Z),
        )
    }

    pub fn set_with_offset_direction(
        &mut self,
        pos: BlockPosModel,
        direction: DirectionModel,
    ) -> &mut Self {
        self.set(
            pos.x + direction.step_x(),
            pos.y + direction.step_y(),
            pos.z + direction.step_z(),
        )
    }

    pub fn set_with_offset(&mut self, pos: BlockPosModel, x: i32, y: i32, z: i32) -> &mut Self {
        self.set(pos.x + x, pos.y + y, pos.z + z)
    }

    pub fn set_with_offset_vec(&mut self, pos: BlockPosModel, offset: Vec3iModel) -> &mut Self {
        self.set(
            pos.x + offset.get(crate::core_orientation::CoreAxisModel::X),
            pos.y + offset.get(crate::core_orientation::CoreAxisModel::Y),
            pos.z + offset.get(crate::core_orientation::CoreAxisModel::Z),
        )
    }

    pub fn move_direction(&mut self, direction: DirectionModel, steps: i32) -> &mut Self {
        self.set(
            self.x + direction.step_x() * steps,
            self.y + direction.step_y() * steps,
            self.z + direction.step_z() * steps,
        )
    }

    pub fn move_by(&mut self, x: i32, y: i32, z: i32) -> &mut Self {
        self.set(self.x + x, self.y + y, self.z + z)
    }

    pub fn move_vec(&mut self, pos: Vec3iModel) -> &mut Self {
        self.move_by(
            pos.get(crate::core_orientation::CoreAxisModel::X),
            pos.get(crate::core_orientation::CoreAxisModel::Y),
            pos.get(crate::core_orientation::CoreAxisModel::Z),
        )
    }

    pub fn clamp(&mut self, axis: AxisModel, minimum: i32, maximum: i32) -> &mut Self {
        match axis {
            AxisModel::X => self.x = self.x.clamp(minimum, maximum),
            AxisModel::Y => self.y = self.y.clamp(minimum, maximum),
            AxisModel::Z => self.z = self.z.clamp(minimum, maximum),
        }
        self
    }

    pub fn set_x(&mut self, x: i32) -> &mut Self {
        self.x = x;
        self
    }

    pub fn set_y(&mut self, y: i32) -> &mut Self {
        self.y = y;
        self
    }

    pub fn set_z(&mut self, z: i32) -> &mut Self {
        self.z = z;
        self
    }
}

fn cycle_axis(transform: AxisCycleModel, x: i32, y: i32, z: i32, axis: AxisModel) -> i32 {
    let axis = match axis {
        AxisModel::X => crate::core_orientation::CoreAxisModel::X,
        AxisModel::Y => crate::core_orientation::CoreAxisModel::Y,
        AxisModel::Z => crate::core_orientation::CoreAxisModel::Z,
    };
    transform.cycle_i32(x, y, z, axis)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SequenceRandomModel {
    values: Vec<i32>,
    index: usize,
}

impl SequenceRandomModel {
    pub fn new(values: Vec<i32>) -> Self {
        Self { values, index: 0 }
    }

    pub fn next_int(&mut self, bound: i32) -> i32 {
        let value = self.values[self.index % self.values.len()];
        self.index += 1;
        value.rem_euclid(bound)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pos(x: i32, y: i32, z: i32) -> BlockPosModel {
        BlockPosModel::new(x, y, z)
    }

    #[test]
    fn block_pos_constants_packing_and_basic_constructors_match_java() {
        assert_eq!(PACKED_HORIZONTAL_LENGTH, 26);
        assert_eq!(PACKED_Y_LENGTH, 12);
        assert_eq!(MAX_HORIZONTAL_COORDINATE, 33_554_431);
        assert_eq!(BLOCK_POS_STREAM_CODEC_KIND, "packed_long");
        assert_eq!(BlockPosModel::ZERO, pos(0, 0, 0));
        assert_eq!(
            BlockPosModel::from_vec3i(Vec3iModel::new(1, 2, 3)),
            pos(1, 2, 3)
        );
        assert_eq!(BlockPosModel::containing(1.9, -0.1, -2.0), pos(1, -1, -2));
        assert_eq!(
            BlockPosModel::min(pos(5, -1, 2), pos(1, 3, 0)),
            pos(1, -1, 0)
        );
        assert_eq!(
            BlockPosModel::max(pos(5, -1, 2), pos(1, 3, 0)),
            pos(5, 3, 2)
        );
        assert_eq!(
            BlockPosModel::as_long_coords(1, 2, 3),
            pos(1, 2, 3).as_long()
        );

        for sample in [
            pos(0, 0, 0),
            pos(1, 2, 3),
            pos(-1, -2, -3),
            pos(30_000_000, 2047, -30_000_000),
        ] {
            let packed = sample.as_long();
            assert_eq!(BlockPosModel::of_long(packed), sample);
            assert_eq!(BlockPosModel::x_from_long(packed), sample.x());
            assert_eq!(BlockPosModel::y_from_long(packed), sample.y());
            assert_eq!(BlockPosModel::z_from_long(packed), sample.z());
        }
        assert_eq!(
            BlockPosModel::offset_long(pos(1, 2, 3).as_long(), DirectionModel::North),
            pos(1, 2, 2).as_long()
        );
        assert_eq!(
            BlockPosModel::offset_long_by(pos(1, 2, 3).as_long(), 4, -2, 1),
            pos(5, 0, 4).as_long()
        );
        assert_eq!(BlockPosModel::flat_index(pos(5, 9, 7).as_long()) & 15, 0);
    }

    #[test]
    fn block_pos_offsets_rotation_vectors_and_clamp_match_java() {
        let base = pos(1, 2, 3);
        assert_eq!(base.offset(0, 0, 0), base);
        assert_eq!(base.offset(4, -1, 2), pos(5, 1, 5));
        assert_eq!(base.offset_vec(Vec3iModel::new(2, 3, 4)), pos(3, 5, 7));
        assert_eq!(base.subtract(Vec3iModel::new(2, 3, 4)), pos(-1, -1, -1));
        assert_eq!(base.multiply(0), BlockPosModel::ZERO);
        assert_eq!(base.multiply(1), base);
        assert_eq!(base.multiply(3), pos(3, 6, 9));
        assert_eq!(base.center(), Vec3dModel::new(1.5, 2.5, 3.5));
        assert_eq!(base.bottom_center(), Vec3dModel::new(1.5, 2.0, 3.5));
        assert_eq!(base.relative(DirectionModel::Up, 2), pos(1, 4, 3));
        assert_eq!(base.relative(DirectionModel::North, 1), pos(1, 2, 2));
        assert_eq!(base.relative_axis(AxisModel::Z, -3), pos(1, 2, 0));
        assert_eq!(base.rotate(RotationModel::None), base);
        assert_eq!(base.rotate(RotationModel::Clockwise90), pos(-3, 2, 1));
        assert_eq!(base.rotate(RotationModel::Clockwise180), pos(-1, 2, -3));
        assert_eq!(
            base.rotate(RotationModel::Counterclockwise90),
            pos(3, 2, -1)
        );
        assert_eq!(base.cross(Vec3iModel::new(4, 5, 6)), pos(-3, 6, -3));
        assert_eq!(base.at_y(70), pos(1, 70, 3));
        assert_eq!(base.immutable(), base);
        assert_eq!(
            base.clamp_location_within(Vec3dModel::new(0.0, 2.5, 99.0)),
            Vec3dModel::new(1.00001, 2.5, 3.99999)
        );
    }

    #[test]
    fn block_pos_random_manhattan_and_between_closed_helpers_match_java() {
        let mut random = SequenceRandomModel::new(vec![0, 1, 2, 3, 4, 5]);
        assert_eq!(
            BlockPosModel::random_between_closed(&mut random, 2, 10, 20, 30, 12, 22, 32),
            vec![pos(10, 21, 32), pos(10, 21, 32)]
        );
        let mut random = SequenceRandomModel::new(vec![0, 1, 2]);
        assert_eq!(
            BlockPosModel::random_in_cube(&mut random, 1, pos(5, 6, 7), 1),
            vec![pos(4, 6, 8)]
        );
        assert_eq!(
            BlockPosModel::square_out_south_east(pos(1, 2, 3)),
            vec![pos(1, 2, 3), pos(1, 2, 4), pos(2, 2, 3), pos(2, 2, 4)]
        );
        assert_eq!(
            BlockPosModel::within_manhattan(pos(0, 0, 0), 1, 1, 1),
            vec![
                pos(0, 0, 0),
                pos(-1, 0, 0),
                pos(0, -1, 0),
                pos(0, 0, 1),
                pos(0, 0, -1),
                pos(0, 1, 0),
                pos(1, 0, 0),
                pos(-1, -1, 0),
                pos(-1, 0, 1),
                pos(-1, 0, -1),
                pos(-1, 1, 0),
                pos(0, -1, 1),
                pos(0, -1, -1),
                pos(0, 1, 1),
                pos(0, 1, -1),
                pos(1, -1, 0),
                pos(1, 0, 1),
                pos(1, 0, -1),
                pos(1, 1, 0),
                pos(-1, -1, 1),
                pos(-1, -1, -1),
                pos(-1, 1, 1),
                pos(-1, 1, -1),
                pos(1, -1, 1),
                pos(1, -1, -1),
                pos(1, 1, 1),
                pos(1, 1, -1),
            ]
        );
        assert_eq!(
            BlockPosModel::find_closest_match(pos(0, 0, 0), 2, 1, |candidate| candidate
                == pos(0, 0, -1)),
            Some(pos(0, 0, -1))
        );
        assert_eq!(
            BlockPosModel::between_closed(pos(1, 1, 1), pos(0, 0, 0)),
            vec![
                pos(0, 0, 0),
                pos(1, 0, 0),
                pos(0, 1, 0),
                pos(1, 1, 0),
                pos(0, 0, 1),
                pos(1, 0, 1),
                pos(0, 1, 1),
                pos(1, 1, 1),
            ]
        );
        assert_eq!(
            BlockPosModel::between_closed_aabb(AabbRangeModel::new(
                Vec3dModel::new(0.2, -0.1, 0.0),
                Vec3dModel::new(1.0, 0.0, 0.0),
            )),
            vec![pos(0, -1, 0), pos(1, -1, 0), pos(0, 0, 0), pos(1, 0, 0)]
        );
    }

    #[test]
    fn block_pos_spiral_breadth_first_and_directional_corner_iteration_match_java() {
        assert_eq!(
            BlockPosModel::spiral_around(
                pos(0, 0, 0),
                1,
                DirectionModel::East,
                DirectionModel::South,
            )
            .unwrap(),
            vec![
                pos(0, 0, 0),
                pos(1, 0, 0),
                pos(1, 0, 1),
                pos(0, 0, 1),
                pos(-1, 0, 1),
                pos(-1, 0, 0),
                pos(-1, 0, -1),
                pos(0, 0, -1),
                pos(1, 0, -1),
            ]
        );
        assert_eq!(
            BlockPosModel::spiral_around(
                pos(0, 0, 0),
                1,
                DirectionModel::East,
                DirectionModel::West,
            )
            .unwrap_err(),
            "The two directions cannot be on the same axis"
        );

        let count = BlockPosModel::breadth_first_traversal(
            pos(0, 0, 0),
            2,
            4,
            |candidate| vec![candidate.relative(DirectionModel::East, 1)],
            |candidate| {
                if candidate == pos(1, 0, 0) {
                    TraversalNodeStatusModel::Skip
                } else {
                    TraversalNodeStatusModel::Accept
                }
            },
        );
        assert_eq!(count, 1);
        let count = BlockPosModel::breadth_first_traversal(
            pos(0, 0, 0),
            3,
            10,
            |candidate| vec![candidate.relative(DirectionModel::East, 1)],
            |candidate| {
                if candidate == pos(2, 0, 0) {
                    TraversalNodeStatusModel::Stop
                } else {
                    TraversalNodeStatusModel::Accept
                }
            },
        );
        assert_eq!(count, 2);

        assert_eq!(
            BlockPosModel::between_corners_in_direction(
                pos(0, 0, 0),
                pos(1, 1, 1),
                Vec3dModel::new(-1.0, 2.0, 0.5),
            ),
            vec![
                pos(1, 0, 0),
                pos(1, 0, 1),
                pos(0, 0, 0),
                pos(0, 0, 1),
                pos(1, 1, 0),
                pos(1, 1, 1),
                pos(0, 1, 0),
                pos(0, 1, 1),
            ]
        );
    }

    #[test]
    fn mutable_block_pos_matches_java_mutation_and_immutable_overrides() {
        let mut mutable = MutableBlockPosModel::from_f64(1.9, -0.1, 3.0);
        assert_eq!(mutable.to_block_pos(), pos(1, -1, 3));
        mutable.set(4, 5, 6).set_f64(7.9, 8.1, -0.1);
        assert_eq!(mutable.to_block_pos(), pos(7, 8, -1));
        mutable.set_pos(pos(1, 2, 3));
        assert_eq!(mutable.to_block_pos(), pos(1, 2, 3));
        mutable.set_long(pos(-2, -3, -4).as_long());
        assert_eq!(mutable.to_block_pos(), pos(-2, -3, -4));
        mutable.set_axis_cycle(AxisCycleModel::Forward, 1, 2, 3);
        assert_eq!(mutable.to_block_pos(), pos(3, 1, 2));
        mutable.set_with_offset_direction(pos(1, 2, 3), DirectionModel::Up);
        assert_eq!(mutable.to_block_pos(), pos(1, 3, 3));
        mutable.set_with_offset(pos(1, 2, 3), 4, 5, 6);
        assert_eq!(mutable.to_block_pos(), pos(5, 7, 9));
        mutable.set_with_offset_vec(pos(1, 2, 3), Vec3iModel::new(-1, -2, -3));
        assert_eq!(mutable.to_block_pos(), pos(0, 0, 0));
        mutable
            .move_direction(DirectionModel::South, 2)
            .move_by(1, 2, 3);
        assert_eq!(mutable.to_block_pos(), pos(1, 2, 5));
        mutable.move_vec(Vec3iModel::new(4, 5, 6));
        assert_eq!(mutable.to_block_pos(), pos(5, 7, 11));
        mutable.clamp(AxisModel::Z, -2, 8);
        assert_eq!(mutable.to_block_pos(), pos(5, 7, 8));
        mutable.set_x(-1).set_y(-2).set_z(-3);
        assert_eq!(mutable.to_block_pos(), pos(-1, -2, -3));

        let immutable = mutable.to_block_pos();
        assert_eq!(immutable.offset(1, 0, 0), pos(0, -2, -3));
        assert_eq!(immutable.multiply(2), pos(-2, -4, -6));
        assert_eq!(immutable.relative(DirectionModel::Down, 1), pos(-1, -3, -3));
        assert_eq!(immutable.relative_axis(AxisModel::X, 3), pos(2, -2, -3));
        assert_eq!(immutable.rotate(RotationModel::Clockwise180), pos(1, -2, 3));
    }
}
