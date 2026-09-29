//! `randomTick` of the age-driven crop blocks: `CropBlock` (and its
//! `BeetrootBlock`/`TorchflowerCropBlock` subclasses), `StemBlock`,
//! `NetherWartBlock`, `SweetBerryBushBlock`, `CocoaBlock` and the
//! `FarmlandBlock` moisture cycle they depend on.

use super::{
    default_state, fluid_of, int_property, is_air, is_block, is_tag, offset, random_horizontal,
    vertical, with_int, RandomTickWorld, UPDATE_ALL, UPDATE_CLIENTS,
};
use crate::block_behavior::BlockStateModel;
use crate::block_properties::StateFluid;
use crate::block_update::{BlockPos, Direction};
use crate::random_source::LegacyRandom;

/// `CropBlock.getMaxAge()` for the `CropBlock` subclasses.
pub(super) fn crop_max_age(registry_id: &str) -> i32 {
    match registry_id {
        "minecraft:beetroots" => 3,
        "minecraft:torchflower_crop" => 2,
        _ => 7,
    }
}

/// `CropBlock.getAge(state)`.
pub(super) fn crop_age(state: &BlockStateModel) -> i32 {
    int_property(state, "age")
}

/// `CropBlock.getStateForAge(age)`; `TorchflowerCropBlock` turns its final
/// stage into the `torchflower` block.
pub(super) fn crop_state_for_age(registry_id: &str, age: i32) -> BlockStateModel {
    if registry_id == "minecraft:torchflower_crop" && age == 2 {
        return default_state("minecraft:torchflower");
    }
    with_int(&default_state(registry_id), "age", age)
}

/// `CropBlock.randomTick`.
pub(super) fn crop_random_tick(
    world: &mut impl RandomTickWorld,
    state: &BlockStateModel,
    pos: BlockPos,
    random: &mut LegacyRandom,
) {
    if world.raw_brightness(pos) >= 9 {
        let age = crop_age(state);
        if age < crop_max_age(&state.registry_id) {
            let growth_speed = growth_speed(world, &state.registry_id, pos);
            if random.next_i32_bound((25.0_f32 / growth_speed) as i32 + 1) == 0 {
                world.set_block(
                    pos,
                    crop_state_for_age(&state.registry_id, age + 1),
                    UPDATE_CLIENTS,
                );
            }
        }
    }
}

/// `CropBlock.growCrops(level, pos, state)` with an explicit age increase
/// (`getBonemealAgeIncrease`).
pub(super) fn grow_crops(
    world: &mut impl RandomTickWorld,
    state: &BlockStateModel,
    pos: BlockPos,
    age_increase: i32,
) {
    let age = crop_max_age(&state.registry_id).min(crop_age(state) + age_increase);
    world.set_block(
        pos,
        crop_state_for_age(&state.registry_id, age),
        UPDATE_CLIENTS,
    );
}

/// `CropBlock.getGrowthSpeed(type, level, pos)`. `f32` arithmetic in Java's
/// evaluation order so the truncated `25 / speed` roll bound is identical.
pub(super) fn growth_speed(world: &impl RandomTickWorld, block_id: &str, pos: BlockPos) -> f32 {
    let mut speed = 1.0_f32;
    let below = vertical(pos, -1);
    for xx in -1..=1 {
        for zz in -1..=1 {
            let mut block_speed = 0.0_f32;
            let block_state = world.state_at(offset(below, xx, 0, zz));
            if is_tag(&block_state, "grows_crops") {
                block_speed = 1.0;
                if int_property(&block_state, "moisture") > 0 {
                    block_speed = 3.0;
                }
            }
            if xx != 0 || zz != 0 {
                block_speed /= 4.0;
            }
            speed += block_speed;
        }
    }

    let is_type = |at: BlockPos| is_block(&world.state_at(at), block_id);
    let north = pos.relative(Direction::North);
    let south = pos.relative(Direction::South);
    let west = pos.relative(Direction::West);
    let east = pos.relative(Direction::East);
    let horizontal = is_type(west) || is_type(east);
    let vertical_neighbours = is_type(north) || is_type(south);
    if horizontal && vertical_neighbours {
        speed /= 2.0;
    } else {
        let diagonal = is_type(west.relative(Direction::North))
            || is_type(east.relative(Direction::North))
            || is_type(east.relative(Direction::South))
            || is_type(west.relative(Direction::South));
        if diagonal {
            speed /= 2.0;
        }
    }
    speed
}

/// `(fruit, attached stem, fruit support tag)` of a `StemBlock`.
fn stem_blocks(registry_id: &str) -> (&'static str, &'static str, &'static str) {
    if registry_id == "minecraft:pumpkin_stem" {
        (
            "minecraft:pumpkin",
            "minecraft:attached_pumpkin_stem",
            "supports_pumpkin_stem_fruit",
        )
    } else {
        (
            "minecraft:melon",
            "minecraft:attached_melon_stem",
            "supports_melon_stem_fruit",
        )
    }
}

/// `StemBlock.randomTick`.
pub(super) fn stem_random_tick(
    world: &mut impl RandomTickWorld,
    state: &BlockStateModel,
    pos: BlockPos,
    random: &mut LegacyRandom,
) {
    if world.raw_brightness(pos) < 9 {
        return;
    }
    let growth_speed = growth_speed(world, &state.registry_id, pos);
    if random.next_i32_bound((25.0_f32 / growth_speed) as i32 + 1) != 0 {
        return;
    }
    let age = int_property(state, "age");
    if age < 7 {
        world.set_block(pos, with_int(state, "age", age + 1), UPDATE_CLIENTS);
        return;
    }
    let direction = random_horizontal(random);
    let relative = pos.relative(direction);
    let state_below = world.state_at(vertical(relative, -1));
    let (fruit, attached_stem, fruit_support) = stem_blocks(&state.registry_id);
    if is_air(&world.state_at(relative)) && is_tag(&state_below, fruit_support) {
        world.set_block(relative, default_state(fruit), UPDATE_ALL);
        let stem = default_state(attached_stem)
            .try_set_property("facing", super::direction_name(direction));
        world.set_block(pos, stem, UPDATE_ALL);
    }
}

/// `NetherWartBlock.randomTick`.
pub(super) fn nether_wart_random_tick(
    world: &mut impl RandomTickWorld,
    state: &BlockStateModel,
    pos: BlockPos,
    random: &mut LegacyRandom,
) {
    let age = int_property(state, "age");
    if age < 3 && random.next_i32_bound(10) == 0 {
        world.set_block(pos, with_int(state, "age", age + 1), UPDATE_CLIENTS);
    }
}

/// `SweetBerryBushBlock.randomTick`.
///
/// TODO(game-event-block-change): Java also emits `GameEvent.BLOCK_CHANGE`
/// (vibration/sculk listeners); no game-event bus is wired to the live world.
pub(super) fn sweet_berry_random_tick(
    world: &mut impl RandomTickWorld,
    state: &BlockStateModel,
    pos: BlockPos,
    random: &mut LegacyRandom,
) {
    let age = int_property(state, "age");
    if age < 3 && random.next_i32_bound(5) == 0 && world.raw_brightness(vertical(pos, 1)) >= 9 {
        world.set_block(pos, with_int(state, "age", age + 1), UPDATE_CLIENTS);
    }
}

/// `CocoaBlock.randomTick`.
pub(super) fn cocoa_random_tick(
    world: &mut impl RandomTickWorld,
    state: &BlockStateModel,
    pos: BlockPos,
    random: &mut LegacyRandom,
) {
    if random.next_i32_bound(5) == 0 {
        let age = int_property(state, "age");
        if age < 2 {
            world.set_block(pos, with_int(state, "age", age + 1), UPDATE_CLIENTS);
        }
    }
}

/// `FarmlandBlock.randomTick`.
pub(super) fn farmland_random_tick(
    world: &mut impl RandomTickWorld,
    state: &BlockStateModel,
    pos: BlockPos,
) {
    let moisture = int_property(state, "moisture");
    if !is_near_water(world, pos) && !world.is_raining_at(vertical(pos, 1)) {
        if moisture > 0 {
            world.set_block(
                pos,
                with_int(state, "moisture", moisture - 1),
                UPDATE_CLIENTS,
            );
        } else if !should_maintain_farmland(world, pos) {
            turn_to_dirt(world, pos);
        }
    } else if moisture < 7 {
        world.set_block(pos, with_int(state, "moisture", 7), UPDATE_CLIENTS);
    }
}

/// `FarmlandBlock.turnToDirt(null, state, level, pos)`.
///
/// TODO(farmland-push-entities): `pushEntitiesUp` (moving entities standing
/// on the lowered block) needs the live entity collision query; the block
/// change itself (`setBlockAndUpdate` to dirt) is exact.
pub(super) fn turn_to_dirt(world: &mut impl RandomTickWorld, pos: BlockPos) {
    world.set_block(pos, default_state("minecraft:dirt"), UPDATE_ALL);
}

/// `FarmlandBlock.shouldMaintainFarmland`.
fn should_maintain_farmland(world: &impl RandomTickWorld, pos: BlockPos) -> bool {
    is_tag(&world.state_at(vertical(pos, 1)), "maintains_farmland")
}

/// `FarmlandBlock.isNearWater`: any water fluid in the box
/// `pos + (-4, 0, -4) .. pos + (4, 1, 4)`.
fn is_near_water(world: &impl RandomTickWorld, pos: BlockPos) -> bool {
    for x in -4..=4 {
        for y in 0..=1 {
            for z in -4..=4 {
                let fluid = fluid_of(&world.state_at(offset(pos, x, y, z)));
                if matches!(fluid, StateFluid::Water { .. }) {
                    return true;
                }
            }
        }
    }
    false
}
