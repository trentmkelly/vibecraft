//! Java `FireBlock` (`net.minecraft.world.level.block.FireBlock`): the
//! flammability registry (`FireBlock.bootStrap`) and the scheduled
//! spread/age/burn-out tick (`FireBlock.tick`), ported statement for
//! statement against the [`FireWorld`] abstraction so the same code runs the
//! live server and the unit tests.
//!
//! The random-number consumption order matches Java exactly: every
//! `random.nextInt` / `nextFloat` call happens in the same order and under
//! the same short-circuit conditions as the decompiled source.

use std::collections::HashMap;
use std::sync::LazyLock;

use crate::block_behavior::BlockStateModel;
use crate::block_placement::PlacementWorld;
use crate::block_update::{BlockPos, Direction};

/// `Level.setBlock` flags used by `FireBlock`: `UPDATE_NEIGHBORS | UPDATE_CLIENTS`.
pub const SET_BLOCK_FLAGS_UPDATE_ALL: i32 = 3;
/// `260` = `UPDATE_INVISIBLE | UPDATE_SKIP_ON_PLACE`: the silent age write in `tick`.
pub const SET_BLOCK_FLAGS_AGE_ONLY: i32 = 260;
/// `FireBlock.MAX_AGE`.
pub const MAX_AGE: i32 = 15;

const ALL_DIRECTIONS: [Direction; 6] = [
    Direction::Down,
    Direction::Up,
    Direction::North,
    Direction::South,
    Direction::West,
    Direction::East,
];

/// `(block id, igniteOdds, burnOdds)` in `FireBlock.bootStrap` order.
static FLAMMABLE_BLOCKS: &[(&str, u8, u8)] = &[
    ("minecraft:oak_planks", 5, 20),
    ("minecraft:spruce_planks", 5, 20),
    ("minecraft:birch_planks", 5, 20),
    ("minecraft:jungle_planks", 5, 20),
    ("minecraft:acacia_planks", 5, 20),
    ("minecraft:cherry_planks", 5, 20),
    ("minecraft:dark_oak_planks", 5, 20),
    ("minecraft:pale_oak_planks", 5, 20),
    ("minecraft:mangrove_planks", 5, 20),
    ("minecraft:bamboo_planks", 5, 20),
    ("minecraft:bamboo_mosaic", 5, 20),
    ("minecraft:oak_slab", 5, 20),
    ("minecraft:spruce_slab", 5, 20),
    ("minecraft:birch_slab", 5, 20),
    ("minecraft:jungle_slab", 5, 20),
    ("minecraft:acacia_slab", 5, 20),
    ("minecraft:cherry_slab", 5, 20),
    ("minecraft:dark_oak_slab", 5, 20),
    ("minecraft:pale_oak_slab", 5, 20),
    ("minecraft:mangrove_slab", 5, 20),
    ("minecraft:bamboo_slab", 5, 20),
    ("minecraft:bamboo_mosaic_slab", 5, 20),
    ("minecraft:oak_fence_gate", 5, 20),
    ("minecraft:spruce_fence_gate", 5, 20),
    ("minecraft:birch_fence_gate", 5, 20),
    ("minecraft:jungle_fence_gate", 5, 20),
    ("minecraft:acacia_fence_gate", 5, 20),
    ("minecraft:cherry_fence_gate", 5, 20),
    ("minecraft:dark_oak_fence_gate", 5, 20),
    ("minecraft:pale_oak_fence_gate", 5, 20),
    ("minecraft:mangrove_fence_gate", 5, 20),
    ("minecraft:bamboo_fence_gate", 5, 20),
    ("minecraft:oak_fence", 5, 20),
    ("minecraft:spruce_fence", 5, 20),
    ("minecraft:birch_fence", 5, 20),
    ("minecraft:jungle_fence", 5, 20),
    ("minecraft:acacia_fence", 5, 20),
    ("minecraft:cherry_fence", 5, 20),
    ("minecraft:dark_oak_fence", 5, 20),
    ("minecraft:pale_oak_fence", 5, 20),
    ("minecraft:mangrove_fence", 5, 20),
    ("minecraft:bamboo_fence", 5, 20),
    ("minecraft:oak_stairs", 5, 20),
    ("minecraft:birch_stairs", 5, 20),
    ("minecraft:spruce_stairs", 5, 20),
    ("minecraft:jungle_stairs", 5, 20),
    ("minecraft:acacia_stairs", 5, 20),
    ("minecraft:cherry_stairs", 5, 20),
    ("minecraft:dark_oak_stairs", 5, 20),
    ("minecraft:pale_oak_stairs", 5, 20),
    ("minecraft:mangrove_stairs", 5, 20),
    ("minecraft:bamboo_stairs", 5, 20),
    ("minecraft:bamboo_mosaic_stairs", 5, 20),
    ("minecraft:oak_log", 5, 5),
    ("minecraft:spruce_log", 5, 5),
    ("minecraft:birch_log", 5, 5),
    ("minecraft:jungle_log", 5, 5),
    ("minecraft:acacia_log", 5, 5),
    ("minecraft:cherry_log", 5, 5),
    ("minecraft:pale_oak_log", 5, 5),
    ("minecraft:dark_oak_log", 5, 5),
    ("minecraft:mangrove_log", 5, 5),
    ("minecraft:bamboo_block", 5, 5),
    ("minecraft:stripped_oak_log", 5, 5),
    ("minecraft:stripped_spruce_log", 5, 5),
    ("minecraft:stripped_birch_log", 5, 5),
    ("minecraft:stripped_jungle_log", 5, 5),
    ("minecraft:stripped_acacia_log", 5, 5),
    ("minecraft:stripped_cherry_log", 5, 5),
    ("minecraft:stripped_dark_oak_log", 5, 5),
    ("minecraft:stripped_pale_oak_log", 5, 5),
    ("minecraft:stripped_mangrove_log", 5, 5),
    ("minecraft:stripped_bamboo_block", 5, 5),
    ("minecraft:stripped_oak_wood", 5, 5),
    ("minecraft:stripped_spruce_wood", 5, 5),
    ("minecraft:stripped_birch_wood", 5, 5),
    ("minecraft:stripped_jungle_wood", 5, 5),
    ("minecraft:stripped_acacia_wood", 5, 5),
    ("minecraft:stripped_cherry_wood", 5, 5),
    ("minecraft:stripped_dark_oak_wood", 5, 5),
    ("minecraft:stripped_pale_oak_wood", 5, 5),
    ("minecraft:stripped_mangrove_wood", 5, 5),
    ("minecraft:oak_wood", 5, 5),
    ("minecraft:spruce_wood", 5, 5),
    ("minecraft:birch_wood", 5, 5),
    ("minecraft:jungle_wood", 5, 5),
    ("minecraft:acacia_wood", 5, 5),
    ("minecraft:cherry_wood", 5, 5),
    ("minecraft:pale_oak_wood", 5, 5),
    ("minecraft:dark_oak_wood", 5, 5),
    ("minecraft:mangrove_wood", 5, 5),
    ("minecraft:mangrove_roots", 5, 20),
    ("minecraft:oak_leaves", 30, 60),
    ("minecraft:spruce_leaves", 30, 60),
    ("minecraft:birch_leaves", 30, 60),
    ("minecraft:jungle_leaves", 30, 60),
    ("minecraft:acacia_leaves", 30, 60),
    ("minecraft:cherry_leaves", 30, 60),
    ("minecraft:dark_oak_leaves", 30, 60),
    ("minecraft:pale_oak_leaves", 30, 60),
    ("minecraft:mangrove_leaves", 30, 60),
    ("minecraft:bookshelf", 30, 20),
    ("minecraft:tnt", 15, 100),
    ("minecraft:short_grass", 60, 100),
    ("minecraft:fern", 60, 100),
    ("minecraft:dead_bush", 60, 100),
    ("minecraft:short_dry_grass", 60, 100),
    ("minecraft:tall_dry_grass", 60, 100),
    ("minecraft:sunflower", 60, 100),
    ("minecraft:lilac", 60, 100),
    ("minecraft:rose_bush", 60, 100),
    ("minecraft:peony", 60, 100),
    ("minecraft:tall_grass", 60, 100),
    ("minecraft:large_fern", 60, 100),
    ("minecraft:dandelion", 60, 100),
    ("minecraft:golden_dandelion", 60, 100),
    ("minecraft:poppy", 60, 100),
    ("minecraft:open_eyeblossom", 60, 100),
    ("minecraft:closed_eyeblossom", 60, 100),
    ("minecraft:blue_orchid", 60, 100),
    ("minecraft:allium", 60, 100),
    ("minecraft:azure_bluet", 60, 100),
    ("minecraft:red_tulip", 60, 100),
    ("minecraft:orange_tulip", 60, 100),
    ("minecraft:white_tulip", 60, 100),
    ("minecraft:pink_tulip", 60, 100),
    ("minecraft:oxeye_daisy", 60, 100),
    ("minecraft:cornflower", 60, 100),
    ("minecraft:lily_of_the_valley", 60, 100),
    ("minecraft:torchflower", 60, 100),
    ("minecraft:pitcher_plant", 60, 100),
    ("minecraft:wither_rose", 60, 100),
    ("minecraft:pink_petals", 60, 100),
    ("minecraft:wildflowers", 60, 100),
    ("minecraft:leaf_litter", 60, 100),
    ("minecraft:cactus_flower", 60, 100),
    ("minecraft:white_wool", 30, 60),
    ("minecraft:orange_wool", 30, 60),
    ("minecraft:magenta_wool", 30, 60),
    ("minecraft:light_blue_wool", 30, 60),
    ("minecraft:yellow_wool", 30, 60),
    ("minecraft:lime_wool", 30, 60),
    ("minecraft:pink_wool", 30, 60),
    ("minecraft:gray_wool", 30, 60),
    ("minecraft:light_gray_wool", 30, 60),
    ("minecraft:cyan_wool", 30, 60),
    ("minecraft:purple_wool", 30, 60),
    ("minecraft:blue_wool", 30, 60),
    ("minecraft:brown_wool", 30, 60),
    ("minecraft:green_wool", 30, 60),
    ("minecraft:red_wool", 30, 60),
    ("minecraft:black_wool", 30, 60),
    ("minecraft:vine", 15, 100),
    ("minecraft:coal_block", 5, 5),
    ("minecraft:hay_block", 60, 20),
    ("minecraft:target", 15, 20),
    ("minecraft:white_carpet", 60, 20),
    ("minecraft:orange_carpet", 60, 20),
    ("minecraft:magenta_carpet", 60, 20),
    ("minecraft:light_blue_carpet", 60, 20),
    ("minecraft:yellow_carpet", 60, 20),
    ("minecraft:lime_carpet", 60, 20),
    ("minecraft:pink_carpet", 60, 20),
    ("minecraft:gray_carpet", 60, 20),
    ("minecraft:light_gray_carpet", 60, 20),
    ("minecraft:cyan_carpet", 60, 20),
    ("minecraft:purple_carpet", 60, 20),
    ("minecraft:blue_carpet", 60, 20),
    ("minecraft:brown_carpet", 60, 20),
    ("minecraft:green_carpet", 60, 20),
    ("minecraft:red_carpet", 60, 20),
    ("minecraft:black_carpet", 60, 20),
    ("minecraft:pale_moss_block", 5, 100),
    ("minecraft:pale_moss_carpet", 5, 100),
    ("minecraft:pale_hanging_moss", 5, 100),
    ("minecraft:dried_kelp_block", 30, 60),
    ("minecraft:bamboo", 60, 60),
    ("minecraft:scaffolding", 60, 60),
    ("minecraft:lectern", 30, 20),
    ("minecraft:composter", 5, 20),
    ("minecraft:sweet_berry_bush", 60, 100),
    ("minecraft:beehive", 5, 20),
    ("minecraft:bee_nest", 30, 20),
    ("minecraft:azalea_leaves", 30, 60),
    ("minecraft:flowering_azalea_leaves", 30, 60),
    ("minecraft:cave_vines", 15, 60),
    ("minecraft:cave_vines_plant", 15, 60),
    ("minecraft:spore_blossom", 60, 100),
    ("minecraft:azalea", 30, 60),
    ("minecraft:flowering_azalea", 30, 60),
    ("minecraft:big_dripleaf", 60, 100),
    ("minecraft:big_dripleaf_stem", 60, 100),
    ("minecraft:small_dripleaf", 60, 100),
    ("minecraft:hanging_roots", 30, 60),
    ("minecraft:glow_lichen", 15, 100),
    ("minecraft:firefly_bush", 60, 100),
    ("minecraft:bush", 60, 100),
    ("minecraft:acacia_shelf", 30, 20),
    ("minecraft:bamboo_shelf", 30, 20),
    ("minecraft:birch_shelf", 30, 20),
    ("minecraft:cherry_shelf", 30, 20),
    ("minecraft:dark_oak_shelf", 30, 20),
    ("minecraft:jungle_shelf", 30, 20),
    ("minecraft:mangrove_shelf", 30, 20),
    ("minecraft:oak_shelf", 30, 20),
    ("minecraft:pale_oak_shelf", 30, 20),
    ("minecraft:spruce_shelf", 30, 20),
];

static ODDS_BY_BLOCK: LazyLock<HashMap<&'static str, (u8, u8)>> = LazyLock::new(|| {
    FLAMMABLE_BLOCKS
        .iter()
        .map(|&(id, ignite, burn)| (id, (ignite, burn)))
        .collect()
});

/// Number of `setFlammable` registrations, exposed for the parity test.
#[cfg(test)]
pub const FLAMMABLE_BLOCK_COUNT_FOR_TESTS: usize = FLAMMABLE_BLOCKS.len();

/// `(igniteOdds, burnOdds)` of a block id, `None` when never registered via
/// `setFlammable` (Java's `Object2IntMap` default of 0 for both).
pub fn flammable_block_odds(block_id: &str) -> Option<(u8, u8)> {
    ODDS_BY_BLOCK.get(block_id).copied()
}

fn is_waterlogged(state: &BlockStateModel) -> bool {
    state.property("waterlogged") == Some("true")
}

/// `FireBlock.getIgniteOdds(BlockState)`.
pub fn state_ignite_odds(state: &BlockStateModel) -> i32 {
    if is_waterlogged(state) {
        return 0;
    }
    flammable_block_odds(&state.registry_id).map_or(0, |(ignite, _)| i32::from(ignite))
}

/// `FireBlock.getBurnOdds(BlockState)`.
pub fn state_burn_odds(state: &BlockStateModel) -> i32 {
    if is_waterlogged(state) {
        return 0;
    }
    flammable_block_odds(&state.registry_id).map_or(0, |(_, burn)| i32::from(burn))
}

/// `FireBlock.canBurn(BlockState)`: `getIgniteOdds(state) > 0`.
pub fn can_burn(state: &BlockStateModel) -> bool {
    state_ignite_odds(state) > 0
}

/// `Random` source consumed by the fire tick (`RandomSource`).
pub trait FireRandom {
    /// `RandomSource.nextInt(bound)`.
    fn next_int(&mut self, bound: i32) -> i32;
    /// `RandomSource.nextFloat()`.
    fn next_float(&mut self) -> f32;
}

/// The level view and mutations `FireBlock.tick` needs.
pub trait FireWorld: PlacementWorld {
    /// `ServerLevel.canSpreadFireAround(pos)`.
    fn can_spread_fire_around(&self, pos: BlockPos) -> bool;
    /// `Level.isRaining()`.
    fn is_raining(&self) -> bool;
    /// `Level.isRainingAt(pos)`.
    fn is_raining_at(&self, pos: BlockPos) -> bool;
    /// `state.is(level.dimensionType().infiniburn())`.
    fn is_infiniburn(&self, state: &BlockStateModel) -> bool;
    /// `EnvironmentAttributes.INCREASED_FIRE_BURNOUT` at `pos`.
    fn increased_fire_burnout(&self, pos: BlockPos) -> bool;
    /// `level.getDifficulty().getId()` (peaceful 0 .. hard 3).
    fn difficulty_id(&self) -> i32;
    /// `BlockState.isFaceSturdy(level, pos, Direction.UP)` for the block at `pos`.
    fn is_face_sturdy_up(&self, pos: BlockPos) -> bool;
    /// `level.setBlock(pos, state, flags)`, including `FireBlock.onPlace`
    /// scheduling when `flags` does not skip it.
    fn set_block(&mut self, pos: BlockPos, state: BlockStateModel, flags: i32);
    /// `level.removeBlock(pos, false)`.
    fn remove_block(&mut self, pos: BlockPos);
    /// `TntBlock.prime(level, pos)`.
    fn prime_tnt(&mut self, pos: BlockPos);
    /// `level.scheduleTick(pos, Blocks.FIRE, delay)`.
    fn schedule_fire_tick(&mut self, pos: BlockPos, delay: i32);
}

/// `FireBlock.getFireTickDelay`: `30 + random.nextInt(10)`.
pub fn fire_tick_delay(random: &mut impl FireRandom) -> i32 {
    30 + random.next_int(10)
}

fn is_air(state: &BlockStateModel) -> bool {
    crate::block_properties::state_physics_by_name(&state.state_name())
        .is_some_and(|physics| physics.is_air)
}

fn fire_age(state: &BlockStateModel) -> i32 {
    state
        .property("age")
        .and_then(|age| age.parse().ok())
        .unwrap_or(0)
}

/// `FireBlock.isValidFireLocation`: any of the six neighbours can burn.
fn is_valid_fire_location(world: &impl FireWorld, pos: BlockPos) -> bool {
    ALL_DIRECTIONS
        .iter()
        .any(|direction| can_burn(&world.state_at(pos.relative(*direction))))
}

/// `FireBlock.getIgniteOdds(LevelReader, BlockPos)`: 0 for non-empty
/// positions, else the best ignite odds among the six neighbours.
fn ignite_odds_at(world: &impl FireWorld, pos: BlockPos) -> i32 {
    if !is_air(&world.state_at(pos)) {
        return 0;
    }
    ALL_DIRECTIONS
        .iter()
        .map(|direction| state_ignite_odds(&world.state_at(pos.relative(*direction))))
        .max()
        .unwrap_or(0)
}

/// `FireBlock.isNearRain`.
fn is_near_rain(world: &impl FireWorld, pos: BlockPos) -> bool {
    world.is_raining_at(pos)
        || world.is_raining_at(pos.relative(Direction::West))
        || world.is_raining_at(pos.relative(Direction::East))
        || world.is_raining_at(pos.relative(Direction::North))
        || world.is_raining_at(pos.relative(Direction::South))
}

/// `FireBlock.getStateWithAge`: `BaseFireBlock.getState` (soul fire over soul
/// soil, else fire with the burnable-face set) with the age applied to fire.
fn state_with_age(world: &impl FireWorld, pos: BlockPos, age: i32) -> BlockStateModel {
    let placed = crate::block_placement::connecting::base_fire_state(world, pos);
    if placed.registry_id == "minecraft:fire" {
        placed.with_property("age", age.to_string())
    } else {
        placed
    }
}

/// `FireBlock.canSurvive`: sturdy ground or any burnable neighbour.
fn can_survive(world: &impl FireWorld, pos: BlockPos) -> bool {
    world.is_face_sturdy_up(pos.relative(Direction::Down)) || is_valid_fire_location(world, pos)
}

/// `FireBlock.checkBurnOut`.
fn check_burn_out(
    world: &mut impl FireWorld,
    pos: BlockPos,
    chance: i32,
    random: &mut impl FireRandom,
    age: i32,
) {
    let old_state = world.state_at(pos);
    let odds = state_burn_odds(&old_state);
    if random.next_int(chance) >= odds {
        return;
    }
    if random.next_int(age + 10) < 5 && !world.is_raining_at(pos) {
        let new_age = (age + random.next_int(5) / 4).min(MAX_AGE);
        let state = state_with_age(world, pos, new_age);
        world.set_block(pos, state, SET_BLOCK_FLAGS_UPDATE_ALL);
    } else {
        world.remove_block(pos);
    }
    if old_state.registry_id == "minecraft:tnt" {
        world.prime_tnt(pos);
    }
}

/// `FireBlock.tick(state, level, pos, random)`.
#[allow(clippy::too_many_lines)] // one Java method, kept 1:1
pub fn fire_tick(
    world: &mut impl FireWorld,
    mut state: BlockStateModel,
    pos: BlockPos,
    random: &mut impl FireRandom,
) {
    let delay = fire_tick_delay(random);
    world.schedule_fire_tick(pos, delay);
    if !world.can_spread_fire_around(pos) {
        return;
    }
    if !can_survive(world, pos) {
        world.remove_block(pos);
    }
    let below_state = world.state_at(pos.relative(Direction::Down));
    let infini_burn = world.is_infiniburn(&below_state);
    let age = fire_age(&state);
    if !infini_burn
        && world.is_raining()
        && is_near_rain(world, pos)
        && random.next_float() < 0.2 + age as f32 * 0.03
    {
        world.remove_block(pos);
        return;
    }
    let new_age = MAX_AGE.min(age + random.next_int(3) / 2);
    if age != new_age {
        state = state.with_property("age", new_age.to_string());
        world.set_block(pos, state, SET_BLOCK_FLAGS_AGE_ONLY);
    }
    if !infini_burn {
        if !is_valid_fire_location(world, pos) {
            if !world.is_face_sturdy_up(pos.relative(Direction::Down)) || age > 3 {
                world.remove_block(pos);
            }
            return;
        }
        if age == MAX_AGE
            && random.next_int(4) == 0
            && !can_burn(&world.state_at(pos.relative(Direction::Down)))
        {
            world.remove_block(pos);
            return;
        }
    }
    let increased_burnout = world.increased_fire_burnout(pos);
    let extra = if increased_burnout { -50 } else { 0 };
    for (direction, chance) in [
        (Direction::East, 300),
        (Direction::West, 300),
        (Direction::Down, 250),
        (Direction::Up, 250),
        (Direction::North, 300),
        (Direction::South, 300),
    ] {
        check_burn_out(world, pos.relative(direction), chance + extra, random, age);
    }
    spread_to_neighbours(world, pos, age, increased_burnout, random);
}

/// The 3x6x3 ignition scan at the end of `FireBlock.tick`.
fn spread_to_neighbours(
    world: &mut impl FireWorld,
    pos: BlockPos,
    age: i32,
    increased_burnout: bool,
    random: &mut impl FireRandom,
) {
    for dx in -1..=1 {
        for dz in -1..=1 {
            for dy in -1..=4 {
                if dx == 0 && dy == 0 && dz == 0 {
                    continue;
                }
                let mut rate = 100;
                if dy > 1 {
                    rate += (dy - 1) * 100;
                }
                let test_pos = BlockPos {
                    x: pos.x + dx,
                    y: pos.y + dy,
                    z: pos.z + dz,
                };
                let ignite_odds = ignite_odds_at(world, test_pos);
                if ignite_odds <= 0 {
                    continue;
                }
                let mut odds = (ignite_odds + 40 + world.difficulty_id() * 7) / (age + 30);
                if increased_burnout {
                    odds /= 2;
                }
                if odds > 0
                    && random.next_int(rate) <= odds
                    && (!world.is_raining() || !is_near_rain(world, test_pos))
                {
                    let spread_age = MAX_AGE.min(age + random.next_int(5) / 4);
                    let state = state_with_age(world, test_pos, spread_age);
                    world.set_block(test_pos, state, SET_BLOCK_FLAGS_UPDATE_ALL);
                }
            }
        }
    }
}
