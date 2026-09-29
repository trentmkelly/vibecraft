//! A sparse in-memory block world for the entity-physics and explosion tests.

use std::collections::BTreeMap;

use crate::block_behavior::BlockStateModel;
use crate::block_survival::SurvivalWorld;
use crate::block_update::BlockPos;

/// A sparse block world: everything not listed is air.
#[derive(Default)]
pub(crate) struct TestWorld {
    pub blocks: BTreeMap<BlockPos, BlockStateModel>,
}

impl TestWorld {
    pub(crate) fn set(&mut self, x: i32, y: i32, z: i32, block: &str) {
        self.blocks
            .insert(BlockPos { x, y, z }, crate::fluid::block_state_model_from_name(block));
    }

    /// A solid `stone` floor covering `-8..8` at height `y`.
    pub(crate) fn floor(y: i32) -> Self {
        let mut world = Self::default();
        for x in -8..8 {
            for z in -8..8 {
                world.set(x, y, z, "minecraft:stone");
            }
        }
        world
    }
}

impl SurvivalWorld for TestWorld {
    fn state_at(&self, pos: BlockPos) -> BlockStateModel {
        self.blocks.get(&pos).cloned().unwrap_or_else(BlockStateModel::air)
    }

    fn raw_brightness(&self, _pos: BlockPos) -> i32 {
        15
    }
}
