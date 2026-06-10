//! Live wiring of the block placement/survival/shape-update catalogs into the
//! play session: the Java `BlockItem.place` pipeline (placement state ->
//! canSurvive -> setBlock), second-half placement (doors, beds, double
//! plants), and the `Level.updateNeighborShapes` cascade with Java's
//! 512-update budget.

use std::io::{self, Write};

use super::chunk_b::{
    consume_placed_block_item, place_block_item_in_world, write_block_change_ack,
    BlockItemPlacementTarget, UseItemOnContext,
};
use super::*;
use crate::block_behavior::BlockStateModel;
use crate::block_placement::{direction_by_name, PlaceContext, PlacementOutcome, PlacementWorld};
use crate::block_properties::{state_physics_by_name, StateFluid};
use crate::block_survival::SurvivalWorld;
use crate::block_update::{BlockPos, Direction};
use crate::network::play::Direction3d;

/// Java `BlockItem.place` from `getPlacementState` onward, with neighbour
/// shape updates after the write.
#[allow(clippy::too_many_arguments)]
pub(super) fn place_block_item_live<W: Write>(
    stream: &mut W,
    compression: CompressionState,
    state: &mut PlaySessionState,
    context: &mut UseItemOnContext<'_, '_>,
    packet: &crate::network::play::ServerboundUseItemOnPacket,
    item_name: &str,
    clicked_pos: BlockPos,
    target: BlockItemPlacementTarget,
    held_slot: usize,
) -> io::Result<()> {
    let world = LiveBlockWorld {
        layout: context.world_layout,
        seed: context.world_seed,
        cache: context.chunk_cache,
    };
    let place_context = PlaceContext {
        clicked_pos: target.pos,
        clicked_face: direction3d_to_block(packet.block_hit.direction),
        click_location: [
            f64::from(packet.block_hit.x) + f64::from(packet.block_hit.click_x),
            f64::from(packet.block_hit.y) + f64::from(packet.block_hit.click_y),
            f64::from(packet.block_hit.z) + f64::from(packet.block_hit.click_z),
        ],
        replacing_clicked_on_block: target.pos == clicked_pos,
        player_yaw: state.yaw,
        player_pitch: state.pitch,
        // TODO(live-sneak-tracking): wire ServerboundPlayerCommand
        // PRESS_SHIFT_KEY state for isSecondaryUseActive.
        secondary_use_active: false,
        // Java samples level.getRandom(); derive a deterministic roll from
        // the game time + position until a live world RNG is threaded here.
        random_age_roll: ((context.game_time as i32) ^ target.pos.x ^ target.pos.z).rem_euclid(25),
    };

    let placed_state =
        match crate::block_placement::state_for_placement(item_name, &place_context, &world) {
            Some(PlacementOutcome::Place(placed)) => {
                // Java BlockPlaceContext.canPlace -> state.canSurvive(level, pos).
                if !crate::block_survival::can_survive(&placed, target.pos, &world) {
                    return write_block_change_ack(stream, compression, packet.sequence);
                }
                placed
            }
            Some(PlacementOutcome::Reject) => {
                return write_block_change_ack(stream, compression, packet.sequence);
            }
            // Unported placement override: keep the legacy default-state path.
            None => crate::block_placement::default_state(item_name),
        };

    let mut placements = vec![(target.pos, placed_state.clone())];
    placements.extend(second_half_placement(&placed_state, target.pos, &world));

    for (pos, placed) in &placements {
        place_block_item_in_world(context, *pos, &placed.state_name());
    }
    write_block_change_ack(stream, compression, packet.sequence)?;
    for (pos, placed) in &placements {
        let id = crate::block_states::network_id_for_block_state(&placed.state_name()).unwrap_or(0);
        write_block_update(stream, compression, *pos, id)?;
    }

    // Java Level.setBlock flag 3 -> updateShapeAtEdge on the six neighbours,
    // cascading through Java's 512-update budget.
    let changed: Vec<BlockPos> = placements.iter().map(|(pos, _)| *pos).collect();
    let mut cascade = LiveCascade {
        layout: context.world_layout,
        seed: context.world_seed,
        cache: context.chunk_cache,
        fluid_ticks: context.live_fluid_ticks,
        game_time: context.game_time,
        random_roll: place_context.random_age_roll,
    };
    run_live_shape_cascade(stream, compression, &mut cascade, changed)?;
    consume_placed_block_item(stream, compression, state, held_slot)
}

/// Live world view over the generated-chunk cache for the placement,
/// survival, and shape-update catalogs.
pub(crate) struct LiveBlockWorld<'a> {
    pub layout: &'a WorldLayout,
    pub seed: i64,
    pub cache: &'a GeneratedChunkCache,
}

impl SurvivalWorld for LiveBlockWorld<'_> {
    fn state_at(&self, pos: BlockPos) -> BlockStateModel {
        let chunk = self.cache.get_or_load(
            pos.x.div_euclid(16),
            pos.z.div_euclid(16),
            self.layout.root(),
            self.seed,
        );
        if let Some(entry) = chunk.get_block_state_model(pos.x, pos.y, pos.z) {
            let mut state = BlockStateModel::new(entry.name);
            for (key, value) in entry.properties {
                state = state.with_property(&key, value);
            }
            state
        } else {
            BlockStateModel::air()
        }
    }

    fn raw_brightness(&self, _pos: BlockPos) -> i32 {
        // TODO(live-light-query): route through the lighting engine for the
        // crop/mushroom light gates; full daylight matches the current
        // surface-only placement surface.
        15
    }
}

impl PlacementWorld for LiveBlockWorld<'_> {
    fn has_neighbor_signal(&self, _pos: BlockPos) -> bool {
        // TODO(live-redstone): no live signal graph yet.
        false
    }
}

fn direction3d_to_block(direction: Direction3d) -> Direction {
    match direction {
        Direction3d::Down => Direction::Down,
        Direction3d::Up => Direction::Up,
        Direction3d::North => Direction::North,
        Direction3d::South => Direction::South,
        Direction3d::West => Direction::West,
        Direction3d::East => Direction::East,
    }
}

/// Java `DoublePlantBlock.setPlacedBy` / `DoorBlock.setPlacedBy` /
/// `BedBlock.setPlacedBy`: the second half written right after placement.
fn second_half_placement(
    placed: &BlockStateModel,
    pos: BlockPos,
    world: &LiveBlockWorld<'_>,
) -> Option<(BlockPos, BlockStateModel)> {
    let block_type = crate::block_states::block_state_entry(&placed.registry_id)?.block_type;
    match block_type {
        "door" | "weathering_copper_door" => {
            let above = pos.relative(Direction::Up);
            Some((above, placed.clone().try_set_property("half", "upper")))
        }
        "double_plant" | "tall_flower" | "tall_seagrass" | "small_dripleaf" => {
            let above = pos.relative(Direction::Up);
            let mut upper = placed.clone().try_set_property("half", "upper");
            if upper.has_property("waterlogged") {
                // DoublePlantBlock.copyWaterloggedFrom at the upper position.
                let above_state = world.state_at(above);
                let water =
                    state_physics_by_name(&above_state.state_name()).is_some_and(|physics| {
                        matches!(physics.fluid, StateFluid::Water { source: true, .. })
                    });
                upper = upper.try_set_property("waterlogged", if water { "true" } else { "false" });
            }
            Some((above, upper))
        }
        "bed" => {
            let facing = placed
                .property("facing")
                .and_then(direction_by_name)
                .unwrap_or(Direction::North);
            let head = pos.relative(facing);
            Some((head, placed.clone().try_set_property("part", "head")))
        }
        _ => None,
    }
}

/// World handles needed by the live shape cascade (shared by the placement
/// and destruction paths).
pub(super) struct LiveCascade<'a, 'b> {
    pub layout: &'a WorldLayout,
    pub seed: i64,
    pub cache: &'a GeneratedChunkCache,
    pub fluid_ticks: &'b mut LiveFluidTicks,
    pub game_time: i64,
    /// Pre-rolled randomness for coral die ticks / growing-plant ages.
    pub random_roll: i32,
}

/// Applies `updateShape` to the neighbours of every changed position,
/// cascading like Java's `Level.updateNeighborShapes` under the 512 budget.
pub(super) fn run_live_shape_cascade<W: Write>(
    writer: &mut W,
    compression: CompressionState,
    context: &mut LiveCascade<'_, '_>,
    mut worklist: Vec<BlockPos>,
) -> io::Result<()> {
    let world = LiveBlockWorld {
        layout: context.layout,
        seed: context.seed,
        cache: context.cache,
    };
    let mut budget = 512;
    while let Some(changed_pos) = worklist.pop() {
        for direction in [
            Direction::West,
            Direction::East,
            Direction::Down,
            Direction::Up,
            Direction::North,
            Direction::South,
        ] {
            if budget == 0 {
                return Ok(());
            }
            let neighbour_pos = changed_pos.relative(direction);
            let neighbour_state = world.state_at(neighbour_pos);
            if neighbour_state.is_air() {
                continue;
            }
            let changed_state = world.state_at(changed_pos);
            let Some(update) = crate::block_shape_updates::update_shape(
                &neighbour_state,
                neighbour_pos,
                direction.opposite(),
                changed_pos,
                &changed_state,
                context.random_roll,
                &world,
            ) else {
                continue;
            };
            if update.schedule_fluid_tick {
                context
                    .fluid_ticks
                    .schedule(context.game_time, neighbour_pos, FluidKind::Water);
            }
            // TODO(live-block-ticks): update.schedule_block_tick needs the
            // scheduled block-tick engine (leaf decay, cactus pop, falling
            // blocks) once it exists.
            if update.state != neighbour_state {
                budget -= 1;
                context.cache.set_block(
                    context.layout.root(),
                    context.seed,
                    neighbour_pos,
                    &update.state.state_name(),
                );
                let id =
                    crate::block_states::network_id_for_block_state(&update.state.state_name())
                        .unwrap_or(0);
                write_block_update(writer, compression, neighbour_pos, id)?;
                worklist.push(neighbour_pos);
            }
        }
    }
    Ok(())
}

/// One `ClientboundBlockUpdatePacket`.
fn write_block_update<W: Write>(
    writer: &mut W,
    compression: CompressionState,
    pos: BlockPos,
    block_state_id: i32,
) -> io::Result<()> {
    let packed_pos = block_pos_as_long(pos.x, pos.y, pos.z);
    write_framed_packet_with_compression(
        writer,
        compression,
        CLIENTBOUND_BLOCK_UPDATE_PACKET_ID,
        |payload| {
            payload.write_all(&packed_pos.to_be_bytes())?;
            write_var_i32(payload, block_state_id)
        },
    )
}

/// Java `Block.playerWillDestroy` for double blocks (doors, beds, double
/// plants): removing one half removes the counterpart, then the neighbour
/// cascade runs for every cleared position.
pub(super) fn run_block_break_aftermath<W: Write>(
    writer: &mut W,
    compression: CompressionState,
    context: &mut LiveCascade<'_, '_>,
    broken_pos: BlockPos,
    broken_state: Option<&BlockStateModel>,
) -> io::Result<()> {
    let mut cleared = vec![broken_pos];
    if let Some(state) = broken_state {
        if let Some(counterpart) = double_block_counterpart(state, broken_pos) {
            context.cache.set_block(
                context.layout.root(),
                context.seed,
                counterpart,
                "minecraft:air",
            );
            write_block_update(writer, compression, counterpart, 0)?;
            cleared.push(counterpart);
        }
    }
    run_live_shape_cascade(writer, compression, context, cleared)
}

/// The other half of a two-block structure, if `state` is one.
fn double_block_counterpart(state: &BlockStateModel, pos: BlockPos) -> Option<BlockPos> {
    let block_type = crate::block_states::block_state_entry(&state.registry_id)?.block_type;
    match block_type {
        "door"
        | "weathering_copper_door"
        | "double_plant"
        | "tall_flower"
        | "tall_seagrass"
        | "small_dripleaf"
        | "pitcher_crop" => match state.property("half") {
            Some("lower") => Some(pos.relative(Direction::Up)),
            Some("upper") => Some(pos.relative(Direction::Down)),
            _ => None,
        },
        "bed" => {
            let facing = state.property("facing").and_then(direction_by_name)?;
            match state.property("part") {
                // The head sits along FACING from the foot.
                Some("foot") => Some(pos.relative(facing)),
                Some("head") => Some(pos.relative(facing.opposite())),
                _ => None,
            }
        }
        _ => None,
    }
}
