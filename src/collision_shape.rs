#![allow(dead_code)]

use crate::block_update::Direction;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Aabb {
    pub min_x: f64,
    pub min_y: f64,
    pub min_z: f64,
    pub max_x: f64,
    pub max_y: f64,
    pub max_z: f64,
}

impl Aabb {
    pub const fn new(
        min_x: f64,
        min_y: f64,
        min_z: f64,
        max_x: f64,
        max_y: f64,
        max_z: f64,
    ) -> Self {
        Self {
            min_x,
            min_y,
            min_z,
            max_x,
            max_y,
            max_z,
        }
    }
}

pub fn piston_movement_area(aabb: Aabb, direction: Direction, amount: f64) -> Aabb {
    let axis_step = match direction {
        Direction::West | Direction::Down | Direction::North => -1.0,
        Direction::East | Direction::Up | Direction::South => 1.0,
    };
    let delta = amount * axis_step;
    let min = delta.min(0.0);
    let max = delta.max(0.0);
    match direction {
        Direction::West => Aabb::new(
            aabb.min_x + min,
            aabb.min_y,
            aabb.min_z,
            aabb.min_x + max,
            aabb.max_y,
            aabb.max_z,
        ),
        Direction::East => Aabb::new(
            aabb.max_x + min,
            aabb.min_y,
            aabb.min_z,
            aabb.max_x + max,
            aabb.max_y,
            aabb.max_z,
        ),
        Direction::Down => Aabb::new(
            aabb.min_x,
            aabb.min_y + min,
            aabb.min_z,
            aabb.max_x,
            aabb.min_y + max,
            aabb.max_z,
        ),
        Direction::Up => Aabb::new(
            aabb.min_x,
            aabb.max_y + min,
            aabb.min_z,
            aabb.max_x,
            aabb.max_y + max,
            aabb.max_z,
        ),
        Direction::North => Aabb::new(
            aabb.min_x,
            aabb.min_y,
            aabb.min_z + min,
            aabb.max_x,
            aabb.max_y,
            aabb.min_z + max,
        ),
        Direction::South => Aabb::new(
            aabb.min_x,
            aabb.min_y,
            aabb.max_z + min,
            aabb.max_x,
            aabb.max_y,
            aabb.max_z + max,
        ),
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VoxelShape {
    boxes: Vec<Aabb>,
}

impl VoxelShape {
    pub fn empty() -> Self {
        Self { boxes: Vec::new() }
    }

    pub fn block() -> Self {
        Self {
            boxes: vec![Aabb::new(0.0, 0.0, 0.0, 1.0, 1.0, 1.0)],
        }
    }

    pub fn box_shape(
        min_x: f64,
        min_y: f64,
        min_z: f64,
        max_x: f64,
        max_y: f64,
        max_z: f64,
    ) -> Self {
        assert!(min_x <= max_x && min_y <= max_y && min_z <= max_z);
        if max_x - min_x < 1.0E-7 || max_y - min_y < 1.0E-7 || max_z - min_z < 1.0E-7 {
            Self::empty()
        } else if find_bits(min_x, max_x) == Some(0)
            && find_bits(min_y, max_y) == Some(0)
            && find_bits(min_z, max_z) == Some(0)
        {
            Self::block()
        } else {
            Self {
                boxes: vec![Aabb::new(min_x, min_y, min_z, max_x, max_y, max_z)],
            }
        }
    }

    pub fn is_empty(&self) -> bool {
        self.boxes.is_empty()
    }

    pub fn boxes(&self) -> &[Aabb] {
        &self.boxes
    }

    pub fn max_y(&self) -> Option<f64> {
        self.boxes.iter().map(|aabb| aabb.max_y).reduce(f64::max)
    }

    pub fn intersects(&self, other: &Self) -> bool {
        self.boxes
            .iter()
            .any(|first| other.boxes.iter().any(|second| intersects(*first, *second)))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BooleanOpKind {
    False,
    NotOr,
    OnlySecond,
    NotFirst,
    OnlyFirst,
    NotSecond,
    NotSame,
    NotAnd,
    And,
    Same,
    Second,
    Causes,
    First,
    CausedBy,
    Or,
    True,
}

impl BooleanOpKind {
    pub fn apply(self, first: bool, second: bool) -> bool {
        match self {
            Self::False => false,
            Self::NotOr => !first && !second,
            Self::OnlySecond => second && !first,
            Self::NotFirst => !first,
            Self::OnlyFirst => first && !second,
            Self::NotSecond => !second,
            Self::NotSame => first != second,
            Self::NotAnd => !first || !second,
            Self::And => first && second,
            Self::Same => first == second,
            Self::Second => second,
            Self::Causes => !first || second,
            Self::First => first,
            Self::CausedBy => first || !second,
            Self::Or => first || second,
            Self::True => true,
        }
    }
}

pub fn join(first: &VoxelShape, second: &VoxelShape, op: BooleanOpKind) -> VoxelShape {
    assert!(!op.apply(false, false));
    match op {
        BooleanOpKind::False => VoxelShape::empty(),
        BooleanOpKind::First => first.clone(),
        BooleanOpKind::Second => second.clone(),
        BooleanOpKind::Or => {
            let mut boxes = first.boxes.clone();
            boxes.extend_from_slice(&second.boxes);
            VoxelShape { boxes }.optimize()
        }
        BooleanOpKind::And => {
            let boxes = first
                .boxes
                .iter()
                .flat_map(|a| second.boxes.iter().filter_map(|b| intersection(*a, *b)))
                .collect();
            VoxelShape { boxes }.optimize()
        }
        BooleanOpKind::OnlyFirst => subtract(first, second),
        BooleanOpKind::OnlySecond => subtract(second, first),
        BooleanOpKind::NotSame => {
            let a = subtract(first, second);
            let b = subtract(second, first);
            join(&a, &b, BooleanOpKind::Or)
        }
        _ => {
            if join_is_not_empty(first, second, op) {
                join(first, second, BooleanOpKind::Or)
            } else {
                VoxelShape::empty()
            }
        }
    }
}

pub fn join_is_not_empty(first: &VoxelShape, second: &VoxelShape, op: BooleanOpKind) -> bool {
    assert!(!op.apply(false, false));
    let first_empty = first.is_empty();
    let second_empty = second.is_empty();
    if first_empty || second_empty {
        return op.apply(!first_empty, !second_empty);
    }
    match op {
        BooleanOpKind::False => false,
        BooleanOpKind::First => !first_empty,
        BooleanOpKind::Second => !second_empty,
        BooleanOpKind::Or | BooleanOpKind::NotSame => true,
        BooleanOpKind::And | BooleanOpKind::Same => first.intersects(second),
        BooleanOpKind::OnlyFirst => !first_all_covered(first, second),
        BooleanOpKind::OnlySecond => !first_all_covered(second, first),
        _ => op.apply(true, true) && first.intersects(second),
    }
}

impl VoxelShape {
    fn optimize(mut self) -> Self {
        self.boxes.retain(|aabb| {
            aabb.max_x - aabb.min_x >= 1.0E-7
                && aabb.max_y - aabb.min_y >= 1.0E-7
                && aabb.max_z - aabb.min_z >= 1.0E-7
        });
        self
    }
}

pub fn find_bits(min: f64, max: f64) -> Option<u8> {
    if min < -1.0E-7 || max > 1.0000001 {
        return None;
    }
    for bits in 0..=3 {
        let intervals = 1 << bits;
        let sh_min = min * f64::from(intervals);
        let sh_max = max * f64::from(intervals);
        let epsilon = 1.0E-7 * f64::from(intervals);
        if (sh_min - sh_min.round()).abs() < epsilon && (sh_max - sh_max.round()).abs() < epsilon {
            return Some(bits);
        }
    }
    None
}

fn intersects(first: Aabb, second: Aabb) -> bool {
    first.max_x > second.min_x + 1.0E-7
        && second.max_x > first.min_x + 1.0E-7
        && first.max_y > second.min_y + 1.0E-7
        && second.max_y > first.min_y + 1.0E-7
        && first.max_z > second.min_z + 1.0E-7
        && second.max_z > first.min_z + 1.0E-7
}

fn intersection(first: Aabb, second: Aabb) -> Option<Aabb> {
    if !intersects(first, second) {
        return None;
    }
    Some(Aabb::new(
        first.min_x.max(second.min_x),
        first.min_y.max(second.min_y),
        first.min_z.max(second.min_z),
        first.max_x.min(second.max_x),
        first.max_y.min(second.max_y),
        first.max_z.min(second.max_z),
    ))
}

fn contains(outer: Aabb, inner: Aabb) -> bool {
    outer.min_x <= inner.min_x + 1.0E-7
        && outer.min_y <= inner.min_y + 1.0E-7
        && outer.min_z <= inner.min_z + 1.0E-7
        && outer.max_x + 1.0E-7 >= inner.max_x
        && outer.max_y + 1.0E-7 >= inner.max_y
        && outer.max_z + 1.0E-7 >= inner.max_z
}

fn first_all_covered(first: &VoxelShape, second: &VoxelShape) -> bool {
    first
        .boxes
        .iter()
        .all(|a| second.boxes.iter().any(|b| contains(*b, *a)))
}

fn subtract(first: &VoxelShape, second: &VoxelShape) -> VoxelShape {
    let boxes = first
        .boxes
        .iter()
        .copied()
        .filter(|a| !second.boxes.iter().any(|b| contains(*b, *a)))
        .collect();
    VoxelShape { boxes }.optimize()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FluidKind {
    Empty,
    Water,
    Lava,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FluidCollisionState {
    pub kind: FluidKind,
    pub amount: u8,
    pub same_fluid_above: bool,
}

impl FluidCollisionState {
    pub const fn empty() -> Self {
        Self {
            kind: FluidKind::Empty,
            amount: 0,
            same_fluid_above: false,
        }
    }

    pub const fn source(kind: FluidKind, same_fluid_above: bool) -> Self {
        Self {
            kind,
            amount: 9,
            same_fluid_above,
        }
    }

    pub const fn flowing(kind: FluidKind, amount: u8, same_fluid_above: bool) -> Self {
        Self {
            kind,
            amount,
            same_fluid_above,
        }
    }
}

pub fn fluid_height(state: FluidCollisionState) -> f64 {
    if state.kind == FluidKind::Empty {
        0.0
    } else if state.same_fluid_above {
        1.0
    } else {
        f64::from(state.amount.min(9)) / 9.0
    }
}

pub fn fluid_shape(state: FluidCollisionState) -> VoxelShape {
    if state.kind == FluidKind::Empty || state.amount == 0 {
        VoxelShape::empty()
    } else if state.amount == 9 && state.same_fluid_above {
        VoxelShape::block()
    } else {
        VoxelShape::box_shape(0.0, 0.0, 0.0, 1.0, fluid_height(state), 1.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockCollisionKind {
    Air,
    FullBlock,
    SlabBottom,
    SlabTop,
    Carpet,
    SnowLayer(u8),
    FencePost,
    Pane,
    Ladder,
    WaterloggableThin,
}

pub fn block_collision_shape(kind: BlockCollisionKind) -> VoxelShape {
    match kind {
        BlockCollisionKind::Air => VoxelShape::empty(),
        BlockCollisionKind::FullBlock => VoxelShape::block(),
        BlockCollisionKind::SlabBottom => VoxelShape::box_shape(0.0, 0.0, 0.0, 1.0, 0.5, 1.0),
        BlockCollisionKind::SlabTop => VoxelShape::box_shape(0.0, 0.5, 0.0, 1.0, 1.0, 1.0),
        BlockCollisionKind::Carpet => VoxelShape::box_shape(0.0, 0.0, 0.0, 1.0, 1.0 / 16.0, 1.0),
        BlockCollisionKind::SnowLayer(layers) => {
            let layers = layers.clamp(1, 8);
            VoxelShape::box_shape(0.0, 0.0, 0.0, 1.0, f64::from(layers) / 8.0, 1.0)
        }
        BlockCollisionKind::FencePost => {
            VoxelShape::box_shape(0.375, 0.0, 0.375, 0.625, 1.5, 0.625)
        }
        BlockCollisionKind::Pane => VoxelShape::box_shape(0.4375, 0.0, 0.0, 0.5625, 1.0, 1.0),
        BlockCollisionKind::Ladder => VoxelShape::box_shape(0.0, 0.0, 0.875, 1.0, 1.0, 1.0),
        BlockCollisionKind::WaterloggableThin => {
            VoxelShape::box_shape(0.25, 0.0, 0.25, 0.75, 1.0, 0.75)
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CollisionPair {
    pub block: VoxelShape,
    pub fluid: VoxelShape,
}

pub fn combined_block_fluid_collision(
    block: BlockCollisionKind,
    fluid: FluidCollisionState,
) -> CollisionPair {
    CollisionPair {
        block: block_collision_shape(block),
        fluid: fluid_shape(fluid),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::block_update::Direction;

    #[test]
    fn vanilla_box_creation_collapses_empty_and_full_unit_shapes() {
        assert!(VoxelShape::box_shape(0.0, 0.0, 0.0, 1.0E-8, 1.0, 1.0).is_empty());
        assert_eq!(
            VoxelShape::box_shape(0.0, 0.0, 0.0, 1.0, 1.0, 1.0),
            VoxelShape::block()
        );
        assert_eq!(find_bits(0.0, 0.5), Some(1));
        assert_eq!(find_bits(0.125, 0.875), Some(3));
        assert_eq!(find_bits(-0.1, 1.0), None);
    }

    #[test]
    fn fluid_collision_height_matches_flowing_fluid_shape_contract() {
        let source_no_above = FluidCollisionState::source(FluidKind::Water, false);
        let source_with_above = FluidCollisionState::source(FluidKind::Water, true);
        let flowing = FluidCollisionState::flowing(FluidKind::Lava, 5, false);

        assert_eq!(fluid_height(source_no_above), 1.0);
        assert_eq!(fluid_shape(source_with_above), VoxelShape::block());
        assert_eq!(fluid_height(flowing), 5.0 / 9.0);
        assert_eq!(fluid_shape(flowing).max_y(), Some(5.0 / 9.0));
        assert!(fluid_shape(FluidCollisionState::empty()).is_empty());
    }

    #[test]
    fn representative_block_collision_shapes_cover_full_partial_thin_and_empty_blocks() {
        assert!(block_collision_shape(BlockCollisionKind::Air).is_empty());
        assert_eq!(
            block_collision_shape(BlockCollisionKind::FullBlock),
            VoxelShape::block()
        );
        assert_eq!(
            block_collision_shape(BlockCollisionKind::SlabBottom).max_y(),
            Some(0.5)
        );
        assert_eq!(
            block_collision_shape(BlockCollisionKind::SlabTop).boxes()[0].min_y,
            0.5
        );
        assert_eq!(
            block_collision_shape(BlockCollisionKind::Carpet).max_y(),
            Some(1.0 / 16.0)
        );
        assert_eq!(
            block_collision_shape(BlockCollisionKind::SnowLayer(3)).max_y(),
            Some(3.0 / 8.0)
        );
        assert_eq!(
            block_collision_shape(BlockCollisionKind::FencePost).max_y(),
            Some(1.5)
        );
        assert_eq!(
            block_collision_shape(BlockCollisionKind::Pane).boxes()[0].min_x,
            0.4375
        );
        assert_eq!(
            block_collision_shape(BlockCollisionKind::Ladder).boxes()[0].min_z,
            0.875
        );
    }

    #[test]
    fn block_and_fluid_collision_surfaces_remain_distinct_for_waterlogged_blocks() {
        let pair = combined_block_fluid_collision(
            BlockCollisionKind::WaterloggableThin,
            FluidCollisionState::source(FluidKind::Water, false),
        );
        assert_eq!(pair.block.max_y(), Some(1.0));
        assert_eq!(pair.block.boxes()[0].min_x, 0.25);
        assert_eq!(pair.fluid.max_y(), Some(1.0));
    }

    #[test]
    fn boolean_ops_match_vanilla_truth_tables_for_empty_and_full_shapes() {
        let empty = VoxelShape::empty();
        let block = VoxelShape::block();
        assert!(!BooleanOpKind::False.apply(true, true));
        assert!(BooleanOpKind::Or.apply(true, false));
        assert!(BooleanOpKind::OnlyFirst.apply(true, false));
        assert!(!BooleanOpKind::OnlyFirst.apply(true, true));
        assert!(join_is_not_empty(&block, &empty, BooleanOpKind::First));
        assert!(!join_is_not_empty(&block, &empty, BooleanOpKind::Second));
        assert_eq!(join(&block, &empty, BooleanOpKind::Or), block);
        assert_eq!(join(&block, &block, BooleanOpKind::And), block);
        assert!(join(&block, &block, BooleanOpKind::OnlyFirst).is_empty());
    }

    #[test]
    fn boolean_composition_handles_overlapping_separated_and_partial_boxes() {
        let lower = VoxelShape::box_shape(0.0, 0.0, 0.0, 1.0, 0.5, 1.0);
        let upper = VoxelShape::box_shape(0.0, 0.5, 0.0, 1.0, 1.0, 1.0);
        let middle = VoxelShape::box_shape(0.0, 0.25, 0.0, 1.0, 0.75, 1.0);

        assert!(!lower.intersects(&upper));
        assert!(lower.intersects(&middle));
        assert!(!join_is_not_empty(&lower, &upper, BooleanOpKind::And));
        assert!(join_is_not_empty(&lower, &upper, BooleanOpKind::OnlyFirst));

        let overlap = join(&lower, &middle, BooleanOpKind::And);
        assert_eq!(overlap.boxes()[0], Aabb::new(0.0, 0.25, 0.0, 1.0, 0.5, 1.0));

        let union = join(&lower, &upper, BooleanOpKind::Or);
        assert_eq!(union.boxes().len(), 2);
    }

    #[test]
    fn piston_movement_area_matches_java_directional_face_sweep() {
        let aabb = Aabb::new(1.0, 2.0, 3.0, 4.0, 6.0, 8.0);
        assert_eq!(
            piston_movement_area(aabb, Direction::West, 0.25),
            Aabb::new(0.75, 2.0, 3.0, 1.0, 6.0, 8.0)
        );
        assert_eq!(
            piston_movement_area(aabb, Direction::East, 0.25),
            Aabb::new(4.0, 2.0, 3.0, 4.25, 6.0, 8.0)
        );
        assert_eq!(
            piston_movement_area(aabb, Direction::Down, 0.5),
            Aabb::new(1.0, 1.5, 3.0, 4.0, 2.0, 8.0)
        );
        assert_eq!(
            piston_movement_area(aabb, Direction::Up, 0.5),
            Aabb::new(1.0, 6.0, 3.0, 4.0, 6.5, 8.0)
        );
        assert_eq!(
            piston_movement_area(aabb, Direction::North, 0.75),
            Aabb::new(1.0, 2.0, 2.25, 4.0, 6.0, 3.0)
        );
        assert_eq!(
            piston_movement_area(aabb, Direction::South, 0.75),
            Aabb::new(1.0, 2.0, 8.0, 4.0, 6.0, 8.75)
        );
        assert_eq!(
            piston_movement_area(aabb, Direction::East, -0.25),
            Aabb::new(3.75, 2.0, 3.0, 4.0, 6.0, 8.0)
        );
    }
}
