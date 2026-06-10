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

    // Java FallingBlock.onPlace: a freshly placed gravity block schedules its
    // 2-tick fall check immediately.
    for (pos, placed) in &placements {
        if crate::gravity::falling_kind(placed).is_some() {
            context.live_block_ticks.schedule(
                context.game_time,
                *pos,
                &placed.registry_id,
                crate::gravity::FALLING_BLOCK_TICK_DELAY,
            );
        }
    }

    // Java Level.setBlock flag 3 -> updateShapeAtEdge on the six neighbours,
    // cascading through Java's 512-update budget.
    let changed: Vec<BlockPos> = placements.iter().map(|(pos, _)| *pos).collect();
    let mut cascade = LiveCascade {
        layout: context.world_layout,
        seed: context.world_seed,
        cache: context.chunk_cache,
        fluid_ticks: context.live_fluid_ticks,
        block_ticks: context.live_block_ticks,
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
    pub block_ticks: &'b mut LiveBlockTicks,
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
            if let Some(delay) = update.schedule_block_tick {
                context.block_ticks.schedule(
                    context.game_time,
                    neighbour_pos,
                    &neighbour_state.registry_id,
                    delay,
                );
            }
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

/// Drains the due scheduled block ticks and applies the
/// `block_scheduled_ticks` catalog: Java `LevelTicks.runCollectedTicks` ->
/// `BlockState.tick`.
#[allow(clippy::too_many_arguments)]
pub(super) fn process_live_block_ticks<W: Write>(
    writer: &mut W,
    compression: CompressionState,
    block_ticks: &mut LiveBlockTicks,
    game_time: i64,
    layout: &WorldLayout,
    seed: i64,
    cache: &GeneratedChunkCache,
    world_items: &std::sync::Arc<std::sync::Mutex<WorldItemEntities>>,
) -> io::Result<()> {
    let due = block_ticks.tick_due(game_time, 65536);
    if due.is_empty() {
        return Ok(());
    }
    let world = LiveBlockWorld {
        layout,
        seed,
        cache,
    };
    let mut changed: Vec<BlockPos> = Vec::new();
    for tick in due {
        let state = world.state_at(tick.pos);
        // Java: the tick fires against whatever block is there now; a
        // replaced block's stale tick is ignored by the type check.
        if state.registry_id != tick.ty {
            continue;
        }
        // FallingBlock.tick: spawn a falling entity when the support is gone.
        if crate::gravity::falling_kind(&state).is_some() {
            let below = world.state_at(tick.pos.relative(Direction::Down));
            if crate::gravity::is_free_for_falling(&below)
                && tick.pos.y >= crate::world::OVERWORLD_MIN_Y
            {
                start_block_fall(
                    writer,
                    compression,
                    cache,
                    layout,
                    seed,
                    world_items,
                    tick.pos,
                    &state,
                )?;
                changed.push(tick.pos);
            }
            continue;
        }
        let Some(outcome) = crate::block_scheduled_ticks::scheduled_tick(&state, tick.pos, &world)
        else {
            // TODO(live-scheduled-tick-gaps): scaffolding/dripstone falls,
            // bubble columns, and creaking hearts stay on their TODO items.
            continue;
        };
        match outcome {
            crate::block_scheduled_ticks::BlockTickOutcome::None => {}
            crate::block_scheduled_ticks::BlockTickOutcome::SetState { state, reschedule } => {
                cache.set_block(layout.root(), seed, tick.pos, &state.state_name());
                let id = crate::block_states::network_id_for_block_state(&state.state_name())
                    .unwrap_or(0);
                write_block_update(writer, compression, tick.pos, id)?;
                if let Some(delay) = reschedule {
                    block_ticks.schedule(game_time, tick.pos, &state.registry_id, delay);
                }
                changed.push(tick.pos);
            }
            crate::block_scheduled_ticks::BlockTickOutcome::Destroy { drop } => {
                cache.set_block(layout.root(), seed, tick.pos, "minecraft:air");
                write_block_update(writer, compression, tick.pos, 0)?;
                if drop {
                    spawn_scheduled_tick_drops(
                        writer,
                        compression,
                        world_items,
                        tick.pos,
                        &state.registry_id,
                    )?;
                }
                changed.push(tick.pos);
            }
        }
    }
    if !changed.is_empty() {
        let mut cascade = LiveCascade {
            layout,
            seed,
            cache,
            fluid_ticks: &mut LiveFluidTicks::new(),
            block_ticks,
            game_time,
            random_roll: (game_time as i32).rem_euclid(40),
        };
        // NOTE: fluid ticks raised by this cascade use a throwaway queue; the
        // neighbouring fluid blocks are already rescheduled by
        // schedule_neighbor_fluids on the next interaction. TODO(live-tick
        // -fluid-requeue): thread the real fluid queue once the borrow of the
        // session's LiveFluidTicks can be split from the block queue.
        run_live_shape_cascade(writer, compression, &mut cascade, changed)?;
    }
    Ok(())
}

/// Java `Block.dropResources` for scheduled-tick destruction.
fn spawn_scheduled_tick_drops<W: Write>(
    writer: &mut W,
    compression: CompressionState,
    world_items: &std::sync::Arc<std::sync::Mutex<WorldItemEntities>>,
    pos: BlockPos,
    block_name: &str,
) -> io::Result<()> {
    let loot_seed = (pos.x as u64).wrapping_mul(0x9E37_79B9)
        ^ (pos.y as u64).wrapping_mul(0x6C62_272E)
        ^ (pos.z as u64).wrapping_mul(0x517C_C1B7);
    let drops = evaluate_block_loot(block_name, loot_seed);
    for (item_name, count) in drops {
        let Some(item_pid) = item_protocol_id(item_name) else {
            continue;
        };
        let eid = lock_status_mutex(world_items).alloc_entity_id();
        // Java: ItemEntity constructor velocity (random*0.2-0.1, 0.2, ...).
        let item = DroppedItem {
            entity_id: eid,
            item: item_name,
            count,
            x: f64::from(pos.x) + 0.5,
            y: f64::from(pos.y) + 0.5,
            z: f64::from(pos.z) + 0.5,
            vel_x: f64::from(pseudo_rand_f32(eid, 0)) * 0.2 - 0.1,
            vel_y: 0.2,
            vel_z: f64::from(pseudo_rand_f32(eid, 1)) * 0.2 - 0.1,
            pickup_delay: DEFAULT_PICKUP_DELAY,
            age: 0,
            target_uuid: None,
        };
        write_item_entity_spawn_packets(writer, compression, &item, item_pid)?;
        lock_status_mutex(world_items).entities.push(item);
    }
    Ok(())
}

/// Java entity-type registry id for `minecraft:falling_block` (26.1.2).
const FALLING_BLOCK_ENTITY_TYPE_ID: i32 = 51;

/// Java `FallingBlockEntity.fall(level, pos, state)`: replace the block with
/// its fluid remnant, spawn the entity, and register it for the tick sim.
#[allow(clippy::too_many_arguments)]
fn start_block_fall<W: Write>(
    writer: &mut W,
    compression: CompressionState,
    cache: &GeneratedChunkCache,
    layout: &WorldLayout,
    seed: i64,
    world_items: &std::sync::Arc<std::sync::Mutex<WorldItemEntities>>,
    pos: BlockPos,
    state: &BlockStateModel,
) -> io::Result<()> {
    // state.getFluidState().createLegacyBlock(): waterlogged gravity blocks
    // leave water behind; the vanilla set all leave air.
    let remnant = match state_physics_by_name(&state.state_name())
        .map_or(StateFluid::Empty, |physics| physics.fluid)
    {
        StateFluid::Water { source: true, .. } => "minecraft:water",
        _ => "minecraft:air",
    };
    cache.set_block(layout.root(), seed, pos, remnant);
    let remnant_id = crate::block_states::network_id_for_block_state(remnant).unwrap_or(0);
    write_block_update(writer, compression, pos, remnant_id)?;

    let entity_id = lock_status_mutex(world_items).alloc_entity_id();
    let falling = crate::item_entity::FallingBlockEntity {
        entity_id,
        block_state: state.state_name(),
        x: f64::from(pos.x) + 0.5,
        y: f64::from(pos.y),
        z: f64::from(pos.z) + 0.5,
        vel_y: 0.0,
        start_y: f64::from(pos.y),
    };
    write_falling_block_spawn(writer, compression, &falling)?;
    lock_status_mutex(world_items).falling_blocks.push(falling);
    Ok(())
}

/// `ClientboundAddEntityPacket` for a falling block: the `data` VarInt carries
/// the block-state network id (Java `Block.getId(blockState)`).
fn write_falling_block_spawn<W: Write>(
    writer: &mut W,
    compression: CompressionState,
    falling: &crate::item_entity::FallingBlockEntity,
) -> io::Result<()> {
    let eid = falling.entity_id;
    let uuid_hi = (eid as u64).wrapping_mul(0x6C62_272E_07BB_0142);
    let uuid_lo = (eid as u64).wrapping_mul(0x62B8_2175_6295_C58D);
    let state_id =
        crate::block_states::network_id_for_block_state(&falling.block_state).unwrap_or(0);
    write_framed_packet_with_compression(
        writer,
        compression,
        crate::network::play::CLIENTBOUND_ADD_ENTITY_PACKET_ID,
        |p| {
            write_var_i32(p, eid)?;
            p.write_all(&uuid_hi.to_be_bytes())?;
            p.write_all(&uuid_lo.to_be_bytes())?;
            write_var_i32(p, FALLING_BLOCK_ENTITY_TYPE_ID)?;
            p.write_all(&falling.x.to_be_bytes())?;
            p.write_all(&falling.y.to_be_bytes())?;
            p.write_all(&falling.z.to_be_bytes())?;
            write_lp_vec3(p, 0.0, falling.vel_y, 0.0)?;
            p.write_all(&[0u8, 0u8, 0u8])?; // xRot, yRot, yHeadRot
            write_var_i32(p, state_id)
        },
    )
}

/// Java `FallingBlockEntity.tick` for every live falling block: gravity with
/// drag, landing via the gravity catalog (concrete solidification, anvil
/// damage, break-on-obstruction), and the post-landing shape cascade.
#[allow(clippy::too_many_arguments, clippy::too_many_lines)] // one coherent entity sim
pub(super) fn tick_falling_blocks<W: Write>(
    writer: &mut W,
    compression: CompressionState,
    cascade: &mut LiveCascade<'_, '_>,
    world_items: &std::sync::Arc<std::sync::Mutex<WorldItemEntities>>,
) -> io::Result<()> {
    let mut active = {
        let mut registry = lock_status_mutex(world_items);
        std::mem::take(&mut registry.falling_blocks)
    };
    if active.is_empty() {
        return Ok(());
    }
    let world = LiveBlockWorld {
        layout: cascade.layout,
        seed: cascade.seed,
        cache: cascade.cache,
    };
    let mut surviving = Vec::with_capacity(active.len());
    let mut removed: Vec<i32> = Vec::new();
    let mut landed: Vec<BlockPos> = Vec::new();
    for falling in active.drain(..) {
        let mut falling = falling;
        // Java: applyGravity (-0.04), move, then scale by 0.98.
        falling.vel_y -= 0.04;
        let next_y = falling.y + falling.vel_y;
        falling.vel_y *= 0.98;

        if next_y < f64::from(crate::world::OVERWORLD_MIN_Y) - 64.0 {
            // Java: discardः entities falling out of the world vanish.
            removed.push(falling.entity_id);
            continue;
        }

        let cell = BlockPos {
            x: falling.x.floor() as i32,
            y: next_y.floor() as i32,
            z: falling.z.floor() as i32,
        };
        let cell_state = world.state_at(cell);
        if crate::gravity::is_free_for_falling(&cell_state) {
            falling.y = next_y;
            write_entity_teleport(writer, compression, &falling, false)?;
            surviving.push(falling);
            continue;
        }

        // Landed: the entity rests in the lowest free cell above the
        // obstruction (its current cell).
        let landing_pos = BlockPos {
            x: cell.x,
            y: cell.y + 1,
            z: cell.z,
        };
        let replaced = world.state_at(landing_pos);
        let block_model = parse_state_name(&falling.block_state);
        let kind = crate::gravity::falling_kind(&block_model)
            .unwrap_or(crate::gravity::FallingKind::SandLike);
        let entity_model = crate::gravity::FallingBlockEntityModel {
            block: block_model,
            pos: landing_pos,
            kind,
            drop_item: true,
            cancel_drop: false,
            hurt_entities: matches!(
                kind,
                crate::gravity::FallingKind::Anvil | crate::gravity::FallingKind::PointedDripstone
            ),
        };
        let adjacent_to_water =
            crate::block_placement::connecting::concrete_should_solidify(&world, landing_pos);
        let fall_distance = (falling.start_y - next_y).max(0.0) as f32;
        let plan = crate::gravity::land_falling_block(
            &entity_model,
            landing_pos,
            &replaced,
            adjacent_to_water,
            fall_distance,
        );
        removed.push(falling.entity_id);
        match plan.action {
            crate::gravity::GravityAction::Land { state } => {
                cascade.cache.set_block(
                    cascade.layout.root(),
                    cascade.seed,
                    landing_pos,
                    &state.state_name(),
                );
                let id = crate::block_states::network_id_for_block_state(&state.state_name())
                    .unwrap_or(0);
                write_block_update(writer, compression, landing_pos, id)?;
                landed.push(landing_pos);
            }
            crate::gravity::GravityAction::Break { drops } => {
                for block_name in drops {
                    spawn_scheduled_tick_drops(
                        writer,
                        compression,
                        world_items,
                        landing_pos,
                        &block_name,
                    )?;
                }
            }
            _ => {}
        }
    }
    if !removed.is_empty() {
        write_framed_packet_with_compression(
            writer,
            compression,
            CLIENTBOUND_REMOVE_ENTITIES_PACKET_ID,
            |p| {
                write_var_i32(p, removed.len() as i32)?;
                for id in &removed {
                    write_var_i32(p, *id)?;
                }
                Ok(())
            },
        )?;
    }
    lock_status_mutex(world_items)
        .falling_blocks
        .extend(surviving);
    if !landed.is_empty() {
        run_live_shape_cascade(writer, compression, cascade, landed)?;
    }
    Ok(())
}

/// `ClientboundTeleportEntityPacket`: entity id, PositionMoveRotation
/// (position, delta movement, yaw, pitch), empty relatives bitmask, on-ground.
fn write_entity_teleport<W: Write>(
    writer: &mut W,
    compression: CompressionState,
    falling: &crate::item_entity::FallingBlockEntity,
    on_ground: bool,
) -> io::Result<()> {
    write_framed_packet_with_compression(
        writer,
        compression,
        crate::network::play::CLIENTBOUND_TELEPORT_ENTITY_PACKET_ID,
        |p| {
            write_var_i32(p, falling.entity_id)?;
            p.write_all(&falling.x.to_be_bytes())?;
            p.write_all(&falling.y.to_be_bytes())?;
            p.write_all(&falling.z.to_be_bytes())?;
            p.write_all(&0.0_f64.to_be_bytes())?;
            p.write_all(&falling.vel_y.to_be_bytes())?;
            p.write_all(&0.0_f64.to_be_bytes())?;
            p.write_all(&0.0_f32.to_be_bytes())?; // yaw
            p.write_all(&0.0_f32.to_be_bytes())?; // pitch
            p.write_all(&0_i32.to_be_bytes())?; // Relative bitmask: absolute
            p.write_all(&[u8::from(on_ground)])
        },
    )
}

/// Parses a `block[prop=value,...]` string into a model.
fn parse_state_name(name: &str) -> BlockStateModel {
    match name.split_once('[') {
        Some((base, raw_properties)) => {
            let mut parsed = BlockStateModel::new(base);
            for pair in raw_properties.trim_end_matches(']').split(',') {
                if let Some((key, value)) = pair.split_once('=') {
                    parsed = parsed.with_property(key.trim(), value.trim());
                }
            }
            parsed
        }
        None => BlockStateModel::new(name),
    }
}
