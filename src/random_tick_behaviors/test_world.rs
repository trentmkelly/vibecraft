//! In-memory [`RandomTickWorld`] for the behavior tests.

use std::collections::BTreeMap;

use super::RandomTickWorld;
use crate::block_behavior::BlockStateModel;
use crate::block_survival::SurvivalWorld;
use crate::block_update::BlockPos;

/// A sparse world: unset positions are air.
pub(super) struct TestWorld {
    blocks: BTreeMap<BlockPos, BlockStateModel>,
    /// Value returned for every light query.
    pub light: i32,
    pub raining: bool,
    pub spread_vines: bool,
    /// Every `set_block` call, in order: position, state name, flags.
    pub writes: Vec<(BlockPos, String, i32)>,
}

impl Default for TestWorld {
    fn default() -> Self {
        Self {
            blocks: BTreeMap::new(),
            light: 15,
            raining: false,
            spread_vines: true,
            writes: Vec::new(),
        }
    }
}

/// `BlockPos` shorthand.
pub(super) fn p(x: i32, y: i32, z: i32) -> BlockPos {
    BlockPos { x, y, z }
}

/// A block's default state (`Block.defaultBlockState()`).
pub(super) fn block(registry_id: &str) -> BlockStateModel {
    BlockStateModel::default_for(registry_id)
        .unwrap_or_else(|| panic!("unknown block {registry_id}"))
}

impl TestWorld {
    pub fn with(mut self, pos: BlockPos, state: BlockStateModel) -> Self {
        self.blocks.insert(pos, state);
        self
    }

    pub fn put(&mut self, pos: BlockPos, state: BlockStateModel) {
        self.blocks.insert(pos, state);
    }

    /// Fills the inclusive box with `state`.
    pub fn fill(mut self, from: BlockPos, to: BlockPos, state: &BlockStateModel) -> Self {
        for x in from.x..=to.x {
            for y in from.y..=to.y {
                for z in from.z..=to.z {
                    self.blocks.insert(p(x, y, z), state.clone());
                }
            }
        }
        self
    }

    pub fn at(&self, pos: BlockPos) -> BlockStateModel {
        self.state_at(pos)
    }
}

impl SurvivalWorld for TestWorld {
    fn state_at(&self, pos: BlockPos) -> BlockStateModel {
        self.blocks
            .get(&pos)
            .cloned()
            .unwrap_or_else(BlockStateModel::air)
    }

    fn raw_brightness(&self, _pos: BlockPos) -> i32 {
        self.light
    }
}

impl RandomTickWorld for TestWorld {
    fn set_block(&mut self, pos: BlockPos, state: BlockStateModel, flags: i32) {
        self.writes.push((pos, state.state_name(), flags));
        self.blocks.insert(pos, state);
    }

    fn max_local_raw_brightness(&self, _pos: BlockPos) -> i32 {
        self.light
    }

    fn is_raining_at(&self, _pos: BlockPos) -> bool {
        self.raining
    }

    fn spread_vines(&self) -> bool {
        self.spread_vines
    }

    fn min_y(&self) -> i32 {
        -64
    }

    fn max_y(&self) -> i32 {
        319
    }
}
