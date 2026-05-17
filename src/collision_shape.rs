#![allow(dead_code)]

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
}
