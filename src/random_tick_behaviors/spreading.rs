//! `randomTick` of the ground blocks that spread or die: grass and mycelium
//! (`SpreadingSnowyBlock`), the nylium blocks, and saplings (`SaplingBlock`).

use super::{
    default_state, fluid_of, int_property, is_block, is_tag, offset, vertical, with_bool, with_int,
    RandomTickWorld, UPDATE_ALL, UPDATE_QUIET,
};
use crate::block_behavior::BlockStateModel;
use crate::block_properties::StateFluid;
use crate::block_update::BlockPos;
use crate::lighting::block_light_properties::light_properties_for;
use crate::lighting::direction::Direction as LightDirection;
use crate::lighting::light_chunk::get_light_block_into;
use crate::random_source::LegacyRandom;

/// `LightEngine.getLightBlockInto(state, aboveState, Direction.UP,
/// aboveState.getLightDampening())`: how much light the block above lets
/// through into `state` (15 or more means fully blocked).
fn light_block_into_from_above(state: &BlockStateModel, above: &BlockStateModel) -> i32 {
    let above_properties = light_properties_for(&above.state_name());
    get_light_block_into(
        light_properties_for(&state.state_name()),
        above_properties,
        LightDirection::Up,
        i32::from(above_properties.opacity),
    )
}

/// `SpreadingSnowyBlock.canStayAlive`.
fn can_stay_alive(world: &impl RandomTickWorld, state: &BlockStateModel, pos: BlockPos) -> bool {
    let above_state = world.state_at(vertical(pos, 1));
    if is_block(&above_state, "minecraft:snow") && int_property(&above_state, "layers") == 1 {
        return true;
    }
    if matches!(
        fluid_of(&above_state),
        StateFluid::Water { amount: 8, .. } | StateFluid::Lava { amount: 8, .. }
    ) {
        return false;
    }
    light_block_into_from_above(state, &above_state) < 15
}

/// `SpreadingSnowyBlock.canPropagate`: alive and no water directly above.
fn can_propagate(world: &impl RandomTickWorld, state: &BlockStateModel, pos: BlockPos) -> bool {
    let above_fluid = fluid_of(&world.state_at(vertical(pos, 1)));
    can_stay_alive(world, state, pos) && !matches!(above_fluid, StateFluid::Water { .. })
}

/// `SnowyBlock.isSnowySetting(aboveState)`.
fn is_snowy_setting(above_state: &BlockStateModel) -> bool {
    is_tag(above_state, "snow")
}

/// `SpreadingSnowyBlock.randomTick` for grass (base block dirt) and mycelium.
pub(super) fn spreading_snowy_random_tick(
    world: &mut impl RandomTickWorld,
    state: &BlockStateModel,
    pos: BlockPos,
    random: &mut LegacyRandom,
) {
    const BASE_BLOCK: &str = "minecraft:dirt";
    if !can_stay_alive(world, state, pos) {
        world.set_block(pos, default_state(BASE_BLOCK), UPDATE_ALL);
        return;
    }
    if world.max_local_raw_brightness(vertical(pos, 1)) < 9 {
        return;
    }
    let default_block_state = default_state(&state.registry_id);
    for _ in 0..4 {
        let test_pos = offset(
            pos,
            random.next_i32_bound(3) - 1,
            random.next_i32_bound(5) - 3,
            random.next_i32_bound(3) - 1,
        );
        if is_block(&world.state_at(test_pos), BASE_BLOCK)
            && can_propagate(world, &default_block_state, test_pos)
        {
            let snowy = is_snowy_setting(&world.state_at(vertical(test_pos, 1)));
            world.set_block(
                test_pos,
                with_bool(&default_block_state, "snowy", snowy),
                UPDATE_ALL,
            );
        }
    }
}

/// `NyliumBlock.randomTick`: covered nylium reverts to netherrack.
pub(super) fn nylium_random_tick(
    world: &mut impl RandomTickWorld,
    state: &BlockStateModel,
    pos: BlockPos,
) {
    let above_state = world.state_at(vertical(pos, 1));
    if light_block_into_from_above(state, &above_state) >= 15 {
        world.set_block(pos, default_state("minecraft:netherrack"), UPDATE_ALL);
    }
}

/// `SaplingBlock.randomTick`.
pub(super) fn sapling_random_tick(
    world: &mut impl RandomTickWorld,
    state: &BlockStateModel,
    pos: BlockPos,
    random: &mut LegacyRandom,
) {
    if world.max_local_raw_brightness(vertical(pos, 1)) >= 9 && random.next_i32_bound(7) == 0 {
        advance_tree(world, state, pos);
    }
}

/// `SaplingBlock.advanceTree`: stage 0 becomes stage 1; a stage-1 sapling
/// asks its `TreeGrower` to grow a tree.
///
/// TODO(sapling-tree-feature): `TreeGrower.growTree` places a configured tree
/// feature (`feature.place(level, generator, random, pos)`, including the 2x2
/// mega variants and the flower-dependent bee-nest variants). The runtime
/// configured-feature placer is not available to live block ticks, so a
/// stage-1 sapling is left in place (and, unlike Java, consumes no feature
/// RNG). Everything up to that call is exact.
pub(super) fn advance_tree(
    world: &mut impl RandomTickWorld,
    state: &BlockStateModel,
    pos: BlockPos,
) {
    if int_property(state, "stage") == 0 {
        world.set_block(pos, with_int(state, "stage", 1), UPDATE_QUIET);
    }
}
