//! Java `BlockBehaviour.randomTick` ports for plants and spreading blocks,
//! driven by the shared random-tick pass (`ServerLevel.tickChunk`).
//!
//! Every function here mirrors one Java `randomTick` override and consumes the
//! level `RandomSource` in exactly the order the Java method does, because the
//! sequence of `nextInt`/`nextFloat`/`nextDouble` calls decides which ticks
//! succeed. The world is reached only through [`RandomTickWorld`], so the same
//! code runs against the live chunk cache and against the in-memory test
//! worlds in `tests`.
//!
//! [`random_tick`] is the dispatcher (`BlockState.randomTick`): it selects the
//! Java class by block registry id. Blocks without a port are listed in the
//! `TODO` on that function so the seam is greppable.

mod bonemeal;
mod crops;
mod growing_plants;
mod spreading;
mod vines;

#[cfg(test)]
mod bonemeal_tests;
#[cfg(test)]
mod growth_tests;
#[cfg(test)]
mod spreading_tests;
#[cfg(test)]
mod test_world;
#[cfg(test)]
mod tests;

#[allow(unused_imports)] // consumed by the bone-meal item path
pub(crate) use bonemeal::{bonemeal_block, BonemealOutcome};

use crate::block_behavior::BlockStateModel;
use crate::block_properties::{state_physics_by_name, StateFluid};
use crate::block_survival::SurvivalWorld;
use crate::block_update::{BlockPos, Direction};
use crate::random_source::LegacyRandom;

/// `Block.UPDATE_NEIGHBORS`: notify neighbours of the change.
pub(crate) const UPDATE_NEIGHBORS: i32 = 1;
/// `Block.UPDATE_CLIENTS`: send the change to clients.
pub(crate) const UPDATE_CLIENTS: i32 = 2;
/// `Block.UPDATE_ALL` (`Level.setBlockAndUpdate`).
pub(crate) const UPDATE_ALL: i32 = UPDATE_NEIGHBORS | UPDATE_CLIENTS;
/// `Block.UPDATE_INVISIBLE | Block.UPDATE_SKIP_BLOCK_ENTITY_SIDEEFFECTS`
/// (260): store the state without a client update or neighbour notification.
pub(crate) const UPDATE_QUIET: i32 = 4 | 256;

/// The level view and mutation surface a `randomTick` needs.
pub(crate) trait RandomTickWorld: SurvivalWorld {
    /// `Level.setBlock(pos, state, flags)`.
    fn set_block(&mut self, pos: BlockPos, state: BlockStateModel, flags: i32);
    /// `LevelReader.getMaxLocalRawBrightness(pos)`.
    fn max_local_raw_brightness(&self, pos: BlockPos) -> i32;
    /// `Level.isRainingAt(pos)`.
    fn is_raining_at(&self, pos: BlockPos) -> bool;
    /// `GameRules.SPREAD_VINES`.
    fn spread_vines(&self) -> bool;
    /// `LevelHeightAccessor.getMinY()`.
    fn min_y(&self) -> i32;
    /// `LevelHeightAccessor.getMaxY()`.
    fn max_y(&self) -> i32;
}

/// `BlockState.randomTick(level, pos, random)` for the ported block classes.
///
/// TODO(random-tick-behaviors): `PitcherCropBlock`, `MangrovePropaguleBlock`,
/// `ChorusFlowerBlock`, `SnowLayerBlock`/`IceBlock` (both need the live block
/// light query), `SculkSensorBlock`, `WeatheringCopper` oxidation,
/// `PointedDripstoneBlock`, `LiquidBlock`, lava fire spread and the remaining
/// `randomTick` overrides are not ported.
pub(crate) fn random_tick(
    world: &mut impl RandomTickWorld,
    state: &BlockStateModel,
    pos: BlockPos,
    random: &mut LegacyRandom,
) {
    match state.registry_id.as_str() {
        "minecraft:wheat" | "minecraft:carrots" | "minecraft:potatoes" => {
            crops::crop_random_tick(world, state, pos, random);
        }
        "minecraft:beetroots" | "minecraft:torchflower_crop" => {
            // BeetrootBlock/TorchflowerCropBlock.randomTick: a one-in-three
            // skip before the CropBlock body.
            if random.next_i32_bound(3) != 0 {
                crops::crop_random_tick(world, state, pos, random);
            }
        }
        "minecraft:melon_stem" | "minecraft:pumpkin_stem" => {
            crops::stem_random_tick(world, state, pos, random);
        }
        "minecraft:nether_wart" => crops::nether_wart_random_tick(world, state, pos, random),
        "minecraft:sweet_berry_bush" => crops::sweet_berry_random_tick(world, state, pos, random),
        "minecraft:cocoa" => crops::cocoa_random_tick(world, state, pos, random),
        "minecraft:farmland" => crops::farmland_random_tick(world, state, pos),
        "minecraft:sugar_cane" => growing_plants::sugar_cane_random_tick(world, state, pos),
        "minecraft:cactus" => growing_plants::cactus_random_tick(world, state, pos, random),
        "minecraft:bamboo" => growing_plants::bamboo_stalk_random_tick(world, state, pos, random),
        "minecraft:bamboo_sapling" => {
            growing_plants::bamboo_sapling_random_tick(world, pos, random);
        }
        "minecraft:kelp"
        | "minecraft:twisting_vines"
        | "minecraft:weeping_vines"
        | "minecraft:cave_vines" => {
            growing_plants::growing_plant_head_random_tick(world, state, pos, random);
        }
        "minecraft:vine" => vines::vine_random_tick(world, state, pos, random),
        "minecraft:grass_block" | "minecraft:mycelium" => {
            spreading::spreading_snowy_random_tick(world, state, pos, random);
        }
        "minecraft:crimson_nylium" | "minecraft:warped_nylium" => {
            spreading::nylium_random_tick(world, state, pos);
        }
        id if is_sapling(id) => spreading::sapling_random_tick(world, state, pos, random),
        _ => {}
    }
}

/// `SaplingBlock` instances (the block-state table type `sapling`).
fn is_sapling(registry_id: &str) -> bool {
    crate::block_states::block_state_entry(registry_id)
        .is_some_and(|entry| entry.block_type == "sapling")
}

/// The horizontal directions in `Direction.Plane.HORIZONTAL` iteration order.
pub(crate) const HORIZONTAL: [Direction; 4] = [
    Direction::North,
    Direction::East,
    Direction::South,
    Direction::West,
];

/// `Direction.getName()` (the lower-case name used for block-state values).
pub(crate) fn direction_name(direction: Direction) -> &'static str {
    match direction {
        Direction::Down => "down",
        Direction::Up => "up",
        Direction::North => "north",
        Direction::South => "south",
        Direction::West => "west",
        Direction::East => "east",
    }
}

/// `Direction.Plane.HORIZONTAL.getRandomDirection(random)`:
/// `Util.getRandom(faces, random)` is `faces[random.nextInt(4)]`.
pub(crate) fn random_horizontal(random: &mut LegacyRandom) -> Direction {
    HORIZONTAL[random.next_i32_bound(4) as usize]
}

/// `pos.offset(dx, dy, dz)`.
pub(crate) fn offset(pos: BlockPos, dx: i32, dy: i32, dz: i32) -> BlockPos {
    BlockPos {
        x: pos.x + dx,
        y: pos.y + dy,
        z: pos.z + dz,
    }
}

/// `pos.above(dy)` (`pos.below(n)` is `vertical(pos, -n)`).
pub(crate) fn vertical(pos: BlockPos, dy: i32) -> BlockPos {
    offset(pos, 0, dy, 0)
}

/// `state.getValue(IntegerProperty)`; missing properties read as 0.
pub(crate) fn int_property(state: &BlockStateModel, name: &str) -> i32 {
    state
        .property(name)
        .and_then(|value| value.parse().ok())
        .unwrap_or(0)
}

/// `state.setValue(IntegerProperty, value)`.
pub(crate) fn with_int(state: &BlockStateModel, name: &str, value: i32) -> BlockStateModel {
    state.clone().try_set_property(name, value.to_string())
}

/// `state.getValue(BooleanProperty)`.
pub(crate) fn bool_property(state: &BlockStateModel, name: &str) -> bool {
    state.property(name) == Some("true")
}

/// `state.setValue(BooleanProperty, value)`.
pub(crate) fn with_bool(state: &BlockStateModel, name: &str, value: bool) -> BlockStateModel {
    state
        .clone()
        .try_set_property(name, if value { "true" } else { "false" })
}

/// `state.is(block)`.
pub(crate) fn is_block(state: &BlockStateModel, registry_id: &str) -> bool {
    state.registry_id == registry_id
}

/// `state.is(TagKey<Block>)`.
pub(crate) fn is_tag(state: &BlockStateModel, tag: &str) -> bool {
    crate::block_tags::block_tag_contains(tag, &state.registry_id)
}

/// `state.isAir()`.
pub(crate) fn is_air(state: &BlockStateModel) -> bool {
    state_physics_by_name(&state.state_name()).is_some_and(|physics| physics.is_air)
}

/// `state.getFluidState()`.
pub(crate) fn fluid_of(state: &BlockStateModel) -> StateFluid {
    state_physics_by_name(&state.state_name()).map_or(StateFluid::Empty, |physics| physics.fluid)
}

/// `Level.isEmptyBlock(pos)`: `getBlockState(pos).isAir()`.
pub(crate) fn is_empty_block(world: &impl SurvivalWorld, pos: BlockPos) -> bool {
    is_air(&world.state_at(pos))
}

/// `Level.isInsideBuildHeight(pos)` for a position of the current level.
pub(crate) fn is_inside_build_height(world: &impl RandomTickWorld, pos: BlockPos) -> bool {
    pos.y >= world.min_y() && pos.y <= world.max_y()
}

/// `Block.defaultBlockState()` of a registered block.
pub(crate) fn default_state(registry_id: &str) -> BlockStateModel {
    crate::block_placement::default_state(registry_id)
}
