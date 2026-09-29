//! `randomTick` of the plants that grow new blocks: `SugarCaneBlock`,
//! `CactusBlock`, `BambooStalkBlock`, `BambooSaplingBlock` and the
//! `GrowingPlantHeadBlock` family (kelp, twisting/weeping/cave vines).

use super::{
    default_state, int_property, is_air, is_block, is_empty_block, vertical, with_bool, with_int,
    RandomTickWorld, UPDATE_ALL, UPDATE_QUIET,
};
use crate::block_behavior::BlockStateModel;
use crate::block_survival::can_survive;
use crate::block_update::{BlockPos, Direction};
use crate::random_source::LegacyRandom;

/// `GrowingPlantHeadBlock.MAX_AGE`.
pub(super) const HEAD_MAX_AGE: i32 = 25;

/// `SugarCaneBlock.randomTick`.
pub(super) fn sugar_cane_random_tick(
    world: &mut impl RandomTickWorld,
    state: &BlockStateModel,
    pos: BlockPos,
) {
    if !is_empty_block(world, vertical(pos, 1)) {
        return;
    }
    let mut height = 1;
    while is_block(&world.state_at(vertical(pos, -height)), &state.registry_id) {
        height += 1;
    }
    if height < 3 {
        let age = int_property(state, "age");
        if age == 15 {
            world.set_block(
                vertical(pos, 1),
                default_state(&state.registry_id),
                UPDATE_ALL,
            );
            world.set_block(pos, with_int(state, "age", 0), UPDATE_QUIET);
        } else {
            world.set_block(pos, with_int(state, "age", age + 1), UPDATE_QUIET);
        }
    }
}

/// `CactusBlock.randomTick`.
///
/// The trailing `level.neighborChanged(aboveBlock, above, this, ...)` is a
/// no-op for the cactus (it does not override `neighborChanged`).
pub(super) fn cactus_random_tick(
    world: &mut impl RandomTickWorld,
    state: &BlockStateModel,
    pos: BlockPos,
    random: &mut LegacyRandom,
) {
    let above = vertical(pos, 1);
    if !is_empty_block(world, above) {
        return;
    }
    let mut height = 1;
    let age = int_property(state, "age");
    while is_block(&world.state_at(vertical(pos, -height)), &state.registry_id) {
        height += 1;
        if height == 3 && age == 15 {
            return;
        }
    }

    let cactus = default_state(&state.registry_id);
    if age == 8 && can_survive(&cactus, above, world) {
        let chance_to_grow_flower = if height >= 3 { 0.25 } else { 0.1 };
        if random.next_f64() <= chance_to_grow_flower {
            world.set_block(above, default_state("minecraft:cactus_flower"), UPDATE_ALL);
        }
    } else if age == 15 && height < 3 {
        world.set_block(above, cactus, UPDATE_ALL);
        world.set_block(pos, with_int(state, "age", 0), UPDATE_QUIET);
    }

    if age < 15 {
        world.set_block(pos, with_int(state, "age", age + 1), UPDATE_QUIET);
    }
}

const BAMBOO: &str = "minecraft:bamboo";

/// `BambooStalkBlock.getHeightAboveUpToMax`.
pub(super) fn bamboo_height_above(world: &impl RandomTickWorld, pos: BlockPos) -> i32 {
    let mut height = 0;
    while height < 16 && is_block(&world.state_at(vertical(pos, height + 1)), BAMBOO) {
        height += 1;
    }
    height
}

/// `BambooStalkBlock.getHeightBelowUpToMax`.
pub(super) fn bamboo_height_below(world: &impl RandomTickWorld, pos: BlockPos) -> i32 {
    let mut height = 0;
    while height < 16 && is_block(&world.state_at(vertical(pos, -(height + 1))), BAMBOO) {
        height += 1;
    }
    height
}

/// `BambooStalkBlock.randomTick`.
pub(super) fn bamboo_stalk_random_tick(
    world: &mut impl RandomTickWorld,
    state: &BlockStateModel,
    pos: BlockPos,
    random: &mut LegacyRandom,
) {
    if int_property(state, "stage") != 0 {
        return;
    }
    let above = vertical(pos, 1);
    if random.next_i32_bound(3) == 0
        && is_empty_block(world, above)
        && world.raw_brightness(above) >= 9
    {
        let height = bamboo_height_below(world, pos) + 1;
        if height < 16 {
            grow_bamboo(world, state, pos, random, height);
        }
    }
}

/// `BambooStalkBlock.growBamboo(state, level, pos, random, height)`.
pub(super) fn grow_bamboo(
    world: &mut impl RandomTickWorld,
    state: &BlockStateModel,
    pos: BlockPos,
    random: &mut LegacyRandom,
    height: i32,
) {
    let below_pos = vertical(pos, -1);
    let below_state = world.state_at(below_pos);
    let two_below_pos = vertical(pos, -2);
    let two_below_state = world.state_at(two_below_pos);
    let leaf = |state: &BlockStateModel| state.property("leaves").unwrap_or("none").to_string();
    let mut leaves = "none";
    if height >= 1 {
        if !is_block(&below_state, BAMBOO) || leaf(&below_state) == "none" {
            leaves = "small";
        } else if is_block(&below_state, BAMBOO) && leaf(&below_state) != "none" {
            leaves = "large";
            if is_block(&two_below_state, BAMBOO) {
                world.set_block(
                    below_pos,
                    below_state.clone().try_set_property("leaves", "small"),
                    UPDATE_ALL,
                );
                world.set_block(
                    two_below_pos,
                    two_below_state.clone().try_set_property("leaves", "none"),
                    UPDATE_ALL,
                );
            }
        }
    }

    let age = if int_property(state, "age") != 1 && !is_block(&two_below_state, BAMBOO) {
        0
    } else {
        1
    };
    // `(height < 11 || !(random.nextFloat() < 0.25F)) && height != 15 ? 0 : 1`
    let skips_stage = height < 11 || random.next_f32() >= 0.25;
    let stage = if skips_stage && height != 15 { 0 } else { 1 };
    let grown = with_int(
        &with_int(&default_state(BAMBOO), "age", age),
        "stage",
        stage,
    )
    .try_set_property("leaves", leaves);
    world.set_block(vertical(pos, 1), grown, UPDATE_ALL);
}

/// `BambooSaplingBlock.randomTick`.
pub(super) fn bamboo_sapling_random_tick(
    world: &mut impl RandomTickWorld,
    pos: BlockPos,
    random: &mut LegacyRandom,
) {
    let above = vertical(pos, 1);
    if random.next_i32_bound(3) == 0
        && is_empty_block(world, above)
        && world.raw_brightness(above) >= 9
    {
        grow_bamboo_sapling(world, pos);
    }
}

/// `BambooSaplingBlock.growBamboo(level, pos)`.
pub(super) fn grow_bamboo_sapling(world: &mut impl RandomTickWorld, pos: BlockPos) {
    let bamboo = default_state(BAMBOO).try_set_property("leaves", "small");
    world.set_block(vertical(pos, 1), bamboo, UPDATE_ALL);
}

/// The `GrowingPlantHeadBlock` subclass parameters (constructor arguments and
/// `canGrowInto`).
#[derive(Clone, Copy)]
pub(super) struct GrowingPlantHead {
    /// `growthDirection`.
    pub direction: Direction,
    /// `growPerTickProbability`.
    pub grow_per_tick_probability: f64,
}

impl GrowingPlantHead {
    /// Parameters of a head block, `None` for other blocks.
    pub fn of(registry_id: &str) -> Option<Self> {
        let (direction, grow_per_tick_probability) = match registry_id {
            "minecraft:kelp" => (Direction::Up, 0.14),
            "minecraft:twisting_vines" => (Direction::Up, 0.1),
            "minecraft:weeping_vines" | "minecraft:cave_vines" => (Direction::Down, 0.1),
            _ => return None,
        };
        Some(Self {
            direction,
            grow_per_tick_probability,
        })
    }
}

/// `GrowingPlantHeadBlock.canGrowInto(state)` of the given head block.
pub(super) fn head_can_grow_into(head_id: &str, state: &BlockStateModel) -> bool {
    if head_id == "minecraft:kelp" {
        is_block(state, "minecraft:water")
    } else {
        // NetherVines.isValidGrowthState / CaveVinesBlock.canGrowInto.
        is_air(state)
    }
}

/// `GrowingPlantHeadBlock.getGrowIntoState(state, random)`
/// (`CaveVinesBlock` additionally rolls the berries).
pub(super) fn head_grow_into_state(
    state: &BlockStateModel,
    random: &mut LegacyRandom,
) -> BlockStateModel {
    let grown = with_int(state, "age", int_property(state, "age") + 1);
    if state.registry_id == "minecraft:cave_vines" {
        with_bool(&grown, "berries", random.next_f32() < 0.11)
    } else {
        grown
    }
}

/// `GrowingPlantHeadBlock.randomTick`.
pub(super) fn growing_plant_head_random_tick(
    world: &mut impl RandomTickWorld,
    state: &BlockStateModel,
    pos: BlockPos,
    random: &mut LegacyRandom,
) {
    let Some(head) = GrowingPlantHead::of(&state.registry_id) else {
        return;
    };
    if int_property(state, "age") < HEAD_MAX_AGE
        && random.next_f64() < head.grow_per_tick_probability
    {
        let growth_pos = pos.relative(head.direction);
        if head_can_grow_into(&state.registry_id, &world.state_at(growth_pos)) {
            let grown = head_grow_into_state(state, random);
            world.set_block(growth_pos, grown, UPDATE_ALL);
        }
    }
}
