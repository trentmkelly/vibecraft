//! `BonemealableBlock` hooks (`isValidBonemealTarget`, `isBonemealSuccess`,
//! `performBonemeal`) for the plant blocks whose growth does not need the
//! configured-feature placer, driven by `BoneMealItem.growCrop`.

use super::crops::{crop_age, crop_max_age, grow_crops, stem_random_tick};
use super::growing_plants::{
    bamboo_height_above, bamboo_height_below, grow_bamboo, grow_bamboo_sapling, head_can_grow_into,
    GrowingPlantHead, HEAD_MAX_AGE,
};
use super::spreading::advance_tree;
use super::{
    bool_property, int_property, is_empty_block, is_inside_build_height, vertical, with_bool,
    with_int, RandomTickWorld, UPDATE_ALL, UPDATE_CLIENTS,
};
use crate::block_behavior::BlockStateModel;
use crate::block_update::BlockPos;
use crate::random_source::LegacyRandom;

/// Result of applying bone meal to a block (`BoneMealItem.growCrop`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BonemealOutcome {
    /// The block is not a (ported) `BonemealableBlock`, or
    /// `isValidBonemealTarget` was false: nothing is consumed.
    NotApplicable,
    /// A valid target: the item is consumed. `grew` is
    /// `isBonemealSuccess` (saplings only fail probabilistically).
    Applied { grew: bool },
}

/// `Mth.nextInt(random, min, max)`.
fn next_int_between(random: &mut LegacyRandom, min: i32, max: i32) -> i32 {
    if min >= max {
        min
    } else {
        random.next_i32_bound(max - min + 1) + min
    }
}

/// `NetherVines.getBlocksToGrowWhenBonemealed`.
fn nether_vines_blocks_to_grow(random: &mut LegacyRandom) -> i32 {
    let mut grow_probability = 1.0_f64;
    let mut count = 0;
    while random.next_f64() < grow_probability {
        grow_probability *= 0.826;
        count += 1;
    }
    count
}

/// `BoneMealItem.growCrop` for the block at `pos`: validates the target and,
/// when valid, rolls `isBonemealSuccess` and runs `performBonemeal` with the
/// level random (the caller consumes the item for
/// [`BonemealOutcome::Applied`]).
///
/// TODO(bonemeal-feature-placement): blocks whose `performBonemeal` places a
/// configured/placed feature (grass block, nylium, moss, azalea, mushrooms,
/// seagrass, sea pickle, small dripleaf, pitcher crop, glow lichen and the
/// sapling tree growth itself) are not ported and report
/// [`BonemealOutcome::NotApplicable`].
pub(crate) fn bonemeal_block(
    world: &mut impl RandomTickWorld,
    pos: BlockPos,
    random: &mut LegacyRandom,
) -> BonemealOutcome {
    let state = world.state_at(pos);
    match state.registry_id.as_str() {
        "minecraft:wheat"
        | "minecraft:carrots"
        | "minecraft:potatoes"
        | "minecraft:beetroots"
        | "minecraft:torchflower_crop" => crop(world, &state, pos, random),
        "minecraft:melon_stem" | "minecraft:pumpkin_stem" => stem(world, &state, pos, random),
        "minecraft:cocoa" => cocoa(world, &state, pos),
        "minecraft:sweet_berry_bush" => sweet_berries(world, &state, pos),
        "minecraft:kelp" | "minecraft:twisting_vines" | "minecraft:weeping_vines" => {
            growing_head(world, &state, pos, random)
        }
        "minecraft:cave_vines" => cave_vines(world, &state, pos),
        "minecraft:bamboo" => bamboo(world, pos, random),
        "minecraft:bamboo_sapling" => bamboo_sapling(world, pos),
        id if super::is_sapling(id) => sapling(world, &state, pos, random),
        _ => BonemealOutcome::NotApplicable,
    }
}

const APPLIED: BonemealOutcome = BonemealOutcome::Applied { grew: true };

/// `CropBlock` (`isValidBonemealTarget`: not fully grown).
fn crop(
    world: &mut impl RandomTickWorld,
    state: &BlockStateModel,
    pos: BlockPos,
    random: &mut LegacyRandom,
) -> BonemealOutcome {
    if crop_age(state) >= crop_max_age(&state.registry_id) {
        return BonemealOutcome::NotApplicable;
    }
    // getBonemealAgeIncrease: Mth.nextInt(random, 2, 5); beetroots divide the
    // roll by three; torchflowers always add one (no roll).
    let increase = match state.registry_id.as_str() {
        "minecraft:torchflower_crop" => 1,
        "minecraft:beetroots" => next_int_between(random, 2, 5) / 3,
        _ => next_int_between(random, 2, 5),
    };
    grow_crops(world, state, pos, increase);
    APPLIED
}

/// `StemBlock.performBonemeal`.
fn stem(
    world: &mut impl RandomTickWorld,
    state: &BlockStateModel,
    pos: BlockPos,
    random: &mut LegacyRandom,
) -> BonemealOutcome {
    if int_property(state, "age") == 7 {
        return BonemealOutcome::NotApplicable;
    }
    let age = 7.min(int_property(state, "age") + next_int_between(random, 2, 5));
    let new_state = with_int(state, "age", age);
    world.set_block(pos, new_state.clone(), UPDATE_CLIENTS);
    if age == 7 {
        stem_random_tick(world, &new_state, pos, random);
    }
    APPLIED
}

/// `CocoaBlock.performBonemeal`.
fn cocoa(
    world: &mut impl RandomTickWorld,
    state: &BlockStateModel,
    pos: BlockPos,
) -> BonemealOutcome {
    let age = int_property(state, "age");
    if age >= 2 {
        return BonemealOutcome::NotApplicable;
    }
    world.set_block(pos, with_int(state, "age", age + 1), UPDATE_CLIENTS);
    APPLIED
}

/// `SweetBerryBushBlock.performBonemeal`.
fn sweet_berries(
    world: &mut impl RandomTickWorld,
    state: &BlockStateModel,
    pos: BlockPos,
) -> BonemealOutcome {
    let above = vertical(pos, 1);
    let age = int_property(state, "age");
    if age >= 3 || !is_empty_block(world, above) || !is_inside_build_height(world, above) {
        return BonemealOutcome::NotApplicable;
    }
    world.set_block(pos, with_int(state, "age", 3.min(age + 1)), UPDATE_CLIENTS);
    APPLIED
}

/// `GrowingPlantHeadBlock.performBonemeal` (kelp and nether vines).
fn growing_head(
    world: &mut impl RandomTickWorld,
    state: &BlockStateModel,
    pos: BlockPos,
    random: &mut LegacyRandom,
) -> BonemealOutcome {
    let Some(head) = GrowingPlantHead::of(&state.registry_id) else {
        return BonemealOutcome::NotApplicable;
    };
    let mut forward = pos.relative(head.direction);
    if !head_can_grow_into(&state.registry_id, &world.state_at(forward))
        || !is_inside_build_height(world, forward)
    {
        return BonemealOutcome::NotApplicable;
    }
    let mut next_age = HEAD_MAX_AGE.min(int_property(state, "age") + 1);
    let blocks_to_grow = if state.registry_id == "minecraft:kelp" {
        1
    } else {
        nether_vines_blocks_to_grow(random)
    };
    for _ in 0..blocks_to_grow {
        if !head_can_grow_into(&state.registry_id, &world.state_at(forward))
            || !is_inside_build_height(world, forward)
        {
            break;
        }
        world.set_block(forward, with_int(state, "age", next_age), UPDATE_ALL);
        forward = forward.relative(head.direction);
        next_age = HEAD_MAX_AGE.min(next_age + 1);
    }
    APPLIED
}

/// `CaveVinesBlock` (valid without berries; `performBonemeal` adds them).
fn cave_vines(
    world: &mut impl RandomTickWorld,
    state: &BlockStateModel,
    pos: BlockPos,
) -> BonemealOutcome {
    if bool_property(state, "berries") {
        return BonemealOutcome::NotApplicable;
    }
    world.set_block(pos, with_bool(state, "berries", true), UPDATE_CLIENTS);
    APPLIED
}

/// `BambooStalkBlock.isValidBonemealTarget` / `performBonemeal`.
fn bamboo(
    world: &mut impl RandomTickWorld,
    pos: BlockPos,
    random: &mut LegacyRandom,
) -> BonemealOutcome {
    let mut height_above = bamboo_height_above(world, pos);
    let height_below = bamboo_height_below(world, pos);
    let growth_pos = vertical(pos, height_above + 1);
    let top_stage = int_property(&world.state_at(vertical(pos, height_above)), "stage");
    let valid = height_above + height_below + 1 < 16
        && top_stage != 1
        && is_inside_build_height(world, growth_pos)
        && is_empty_block(world, growth_pos);
    if !valid {
        return BonemealOutcome::NotApplicable;
    }
    let mut total_height = height_above + height_below + 1;
    let new_bamboo = 1 + random.next_i32_bound(2);
    for _ in 0..new_bamboo {
        let top_pos = vertical(pos, height_above);
        let top_state = world.state_at(top_pos);
        let growth_pos = vertical(top_pos, 1);
        if total_height >= 16
            || int_property(&top_state, "stage") == 1
            || !is_empty_block(world, growth_pos)
            || !is_inside_build_height(world, growth_pos)
        {
            break;
        }
        grow_bamboo(world, &top_state, top_pos, random, total_height);
        height_above += 1;
        total_height += 1;
    }
    APPLIED
}

/// `BambooSaplingBlock` (valid with air above, inside the build height).
fn bamboo_sapling(world: &mut impl RandomTickWorld, pos: BlockPos) -> BonemealOutcome {
    let above = vertical(pos, 1);
    if !is_empty_block(world, above) || !is_inside_build_height(world, above) {
        return BonemealOutcome::NotApplicable;
    }
    grow_bamboo_sapling(world, pos);
    APPLIED
}

/// `SaplingBlock`: `isBonemealSuccess` is `level.getRandom().nextFloat() <
/// 0.45`; `performBonemeal` is `advanceTree`.
///
/// `isValidBonemealTarget` offsets by `treeGrower.getMinimumHeight`, which is
/// part of the unported tree-growth data (`TODO(sapling-tree-feature)`); an
/// offset of 0 (the `orElse(0)` fallback) is used.
fn sapling(
    world: &mut impl RandomTickWorld,
    state: &BlockStateModel,
    pos: BlockPos,
    random: &mut LegacyRandom,
) -> BonemealOutcome {
    if !is_inside_build_height(world, pos) {
        return BonemealOutcome::NotApplicable;
    }
    if f64::from(random.next_f32()) < 0.45 {
        advance_tree(world, state, pos);
        APPLIED
    } else {
        BonemealOutcome::Applied { grew: false }
    }
}
