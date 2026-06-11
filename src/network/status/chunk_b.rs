use super::block_menu_open::{
    block_menu_open_for_state, next_open_container_id, write_open_block_menu,
};
use super::ActiveBlockMenu;
use super::*;

pub fn cache_login_profile(
    player_access: &Arc<Mutex<PlayerAccess>>,
    profile: &NameAndId,
) -> io::Result<()> {
    let mut access = player_access
        .lock()
        .map_err(|_| io::Error::other("player access lock poisoned"))?;
    access.cache_user(profile.clone());
    access.save_user_cache(Path::new("."))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PlaySessionUpdate {
    pub position_changed: bool,
    pub health_changed: bool,
    pub respawn_requested: bool,
}

pub fn apply_player_fall_movement(
    state: &mut PlaySessionState,
    delta_y: f64,
    position_changed: bool,
) -> PlaySessionUpdate {
    apply_player_movement(state, 0.0, delta_y, 0.0, position_changed)
}

pub fn apply_player_movement(
    state: &mut PlaySessionState,
    delta_x: f64,
    delta_y: f64,
    delta_z: f64,
    position_changed: bool,
) -> PlaySessionUpdate {
    if position_changed && !state.in_water {
        state.water_velocity_x = delta_x;
        state.water_velocity_y = delta_y;
        state.water_velocity_z = delta_z;
    }
    state.fall_distance = update_fall_distance(state.fall_distance, delta_y, state.in_water);
    if state.in_water {
        state.fall_distance = 0.0;
    }
    if state.eye_in_water {
        let distance_cm = ((delta_x * delta_x + delta_y * delta_y + delta_z * delta_z).sqrt()
            * 100.0)
            .round() as i32;
        if distance_cm > 0 {
            add_player_food_exhaustion(
                state,
                movement_exhaustion(SWIM_EXHAUSTION_PER_METER, distance_cm),
            );
        }
    } else if state.in_water {
        let horizontal_distance_cm =
            ((delta_x * delta_x + delta_z * delta_z).sqrt() * 100.0).round() as i32;
        if horizontal_distance_cm > 0 {
            add_player_food_exhaustion(
                state,
                movement_exhaustion(SWIM_EXHAUSTION_PER_METER, horizontal_distance_cm),
            );
        }
    } else if state.on_ground && state.input_sprinting {
        let horizontal_distance_cm =
            ((delta_x * delta_x + delta_z * delta_z).sqrt() * 100.0).round() as i32;
        if horizontal_distance_cm > 0 {
            add_player_food_exhaustion(
                state,
                movement_exhaustion(SPRINT_EXHAUSTION_PER_METER, horizontal_distance_cm),
            );
        }
    }

    let mut health_changed = false;
    if state.on_ground && state.fall_distance > 0.0 {
        let damage = calculate_fall_damage(FallDamageInput {
            fall_distance: state.fall_distance,
            damage_modifier: 1.0,
            safe_fall_distance: DEFAULT_SAFE_FALL_DISTANCE,
            fall_damage_multiplier: DEFAULT_FALL_DAMAGE_MULTIPLIER,
            fall_damage_enabled: true,
            may_fly: state.abilities.mayfly,
        });
        state.fall_distance = 0.0;
        if damage > 0 && state.health > 0.0 {
            state.health = (state.health - damage as f32).max(0.0);
            health_changed = true;
        }
    }

    PlaySessionUpdate {
        position_changed,
        health_changed,
        respawn_requested: false,
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlayerFluidState {
    pub in_water: bool,
    pub eye_in_water: bool,
    pub water_height: f64,
}

impl PlayerFluidState {
    pub const DRY: Self = Self {
        in_water: false,
        eye_in_water: false,
        water_height: 0.0,
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PlayerWaterTickUpdate {
    pub air_changed: bool,
    pub health_changed: bool,
    pub motion_changed: bool,
}

pub fn detect_play_session_fluid_state(
    state: &PlaySessionState,
    world_root: &Path,
    world_seed: i64,
    chunk_cache: &GeneratedChunkCache,
) -> PlayerFluidState {
    detect_play_session_fluid_state_with_lookup(state, |x, y, z| {
        let chunk =
            chunk_cache.get_or_load(x.div_euclid(16), z.div_euclid(16), world_root, world_seed);
        chunk.get_block_state_name(x, y, z).map(str::to_string)
    })
}

pub fn detect_play_session_fluid_state_with_lookup<F>(
    state: &PlaySessionState,
    mut block_at: F,
) -> PlayerFluidState
where
    F: FnMut(i32, i32, i32) -> Option<String>,
{
    let half_width = PLAYER_WIDTH / 2.0;
    let min_x = state.x - half_width;
    let max_x = state.x + half_width;
    let min_y = state.y;
    let max_y = state.y + PLAYER_HEIGHT;
    let min_z = state.z - half_width;
    let max_z = state.z + half_width;
    let x0 = min_x.floor() as i32;
    let y0 = min_y.floor() as i32;
    let z0 = min_z.floor() as i32;
    let x1 = max_x.ceil() as i32 - 1;
    let y1 = max_y.ceil() as i32 - 1;
    let z1 = max_z.ceil() as i32 - 1;
    let eye_block_x = state.x.floor() as i32;
    let eye_y = state.y + PLAYER_EYE_HEIGHT;
    let eye_block_z = state.z.floor() as i32;

    let mut water_height = 0.0_f64;
    let mut eye_in_water = false;
    for x in x0..=x1 {
        for y in y0..=y1 {
            for z in z0..=z1 {
                let Some(block) = block_at(x, y, z) else {
                    continue;
                };
                let Some(fluid_height) = water_fluid_height_for_block(&block) else {
                    continue;
                };
                let fluid_bottom = f64::from(y);
                let fluid_top = fluid_bottom + fluid_height;
                if fluid_top < min_y {
                    continue;
                }
                water_height = water_height.max(fluid_top - min_y);
                if x == eye_block_x
                    && z == eye_block_z
                    && eye_y >= fluid_bottom
                    && eye_y <= fluid_top
                {
                    eye_in_water = true;
                }
            }
        }
    }

    if water_height > 0.0 {
        PlayerFluidState {
            in_water: true,
            eye_in_water,
            water_height,
        }
    } else {
        PlayerFluidState::DRY
    }
}

pub fn water_fluid_height_for_block(block: &str) -> Option<f64> {
    if block.contains("waterlogged=true") {
        return Some(1.0);
    }
    let (base, properties) = block.split_once('[').map_or((block, ""), |(base, rest)| {
        (base, rest.trim_end_matches(']'))
    });
    if base != "minecraft:water" && base != "minecraft:flowing_water" {
        return None;
    }
    let level = properties
        .split(',')
        .find_map(|property| property.strip_prefix("level="))
        .and_then(|value| value.parse::<i32>().ok())
        .unwrap_or(0);
    Some(match level {
        1..=7 => f64::from(8 - level) / 9.0,
        _ => 1.0,
    })
}

pub fn play_session_water_input_vector(state: &PlaySessionState) -> (f64, f64) {
    let left_intent = if state.input_left == state.input_right {
        0.0
    } else if state.input_left {
        1.0
    } else {
        -1.0
    };
    let forward_intent = if state.input_forward == state.input_backward {
        0.0
    } else if state.input_forward {
        1.0
    } else {
        -1.0
    };
    (left_intent, forward_intent)
}

pub fn rotate_player_input_to_world(strafe: f64, forward: f64, speed: f64, yaw: f32) -> (f64, f64) {
    let length_sqr = strafe * strafe + forward * forward;
    if length_sqr < 1.0e-7 {
        return (0.0, 0.0);
    }
    let scale = if length_sqr > 1.0 {
        speed / length_sqr.sqrt()
    } else {
        speed
    };
    let strafe = strafe * scale;
    let forward = forward * scale;
    let yaw = f64::from(yaw).to_radians();
    let sin = yaw.sin();
    let cos = yaw.cos();
    (strafe * cos - forward * sin, forward * cos + strafe * sin)
}

pub fn tick_play_session_water(
    state: &mut PlaySessionState,
    fluid_state: PlayerFluidState,
) -> PlayerWaterTickUpdate {
    state.in_water = fluid_state.in_water;
    state.eye_in_water = fluid_state.eye_in_water;
    state.water_fluid_height = fluid_state.water_height;
    if state.in_water {
        state.fall_distance = 0.0;
    }

    let old_air = state.air_supply;
    let old_health = state.health;
    let old_water_velocity_x = state.water_velocity_x;
    let old_water_velocity_y = state.water_velocity_y;
    let old_water_velocity_z = state.water_velocity_z;
    if state.in_water {
        let (strafe, forward) = play_session_water_input_vector(state);
        let (input_x, input_z) =
            rotate_player_input_to_world(strafe, forward, WATER_MOVE_RELATIVE_SPEED, state.yaw);
        state.water_velocity_x += input_x;
        state.water_velocity_z += input_z;

        if state.input_shift {
            state.water_velocity_y -= WATER_JUMP_IMPULSE;
        }
        if state.input_jumping && state.water_fluid_height > 0.0 {
            state.water_velocity_y += WATER_JUMP_IMPULSE;
        }

        let should_apply_vertical_fluid_drag =
            !state.on_ground || state.water_velocity_y.abs() > f64::EPSILON || state.eye_in_water;
        if should_apply_vertical_fluid_drag {
            state.water_velocity_y *= WATER_VERTICAL_SLOWDOWN;
            if !state.input_sprinting {
                state.water_velocity_y -= WATER_FALLING_GRAVITY;
            }
        }

        let horizontal_slowdown = if state.input_sprinting {
            WATER_SPRINTING_HORIZONTAL_SLOWDOWN
        } else {
            WATER_HORIZONTAL_SLOWDOWN
        };
        state.water_velocity_x *= horizontal_slowdown;
        state.water_velocity_z *= horizontal_slowdown;
    } else {
        state.water_velocity_x = 0.0;
        state.water_velocity_y = 0.0;
        state.water_velocity_z = 0.0;
    }

    if state.health > 0.0 {
        if state.eye_in_water {
            if !state.abilities.invulnerable {
                state.air_supply -= 1;
                if state.air_supply <= DROWN_AIR_SUPPLY_THRESHOLD {
                    state.air_supply = 0;
                    state.health = (state.health - DROWN_DAMAGE).max(0.0);
                }
            } else if state.air_supply < MAX_AIR_SUPPLY {
                state.air_supply = (state.air_supply + 4).min(MAX_AIR_SUPPLY);
            }
        } else if state.air_supply < MAX_AIR_SUPPLY {
            state.air_supply = (state.air_supply + 4).min(MAX_AIR_SUPPLY);
        }
    }

    PlayerWaterTickUpdate {
        air_changed: state.air_supply != old_air,
        health_changed: state.health != old_health,
        motion_changed: state.in_water
            && ((state.water_velocity_x - old_water_velocity_x).abs() > f64::EPSILON
                || (state.water_velocity_y - old_water_velocity_y).abs() > f64::EPSILON
                || (state.water_velocity_z - old_water_velocity_z).abs() > f64::EPSILON),
    }
}

pub fn food_state_from_play_session(state: &PlaySessionState) -> FoodState {
    FoodState {
        food_level: state.food_level,
        saturation: state.food_saturation,
        exhaustion: state.food_exhaustion,
        tick_timer: state.food_tick_timer,
    }
}

pub fn apply_food_state_to_play_session(state: &mut PlaySessionState, food: FoodState) {
    state.food_level = food.food_level;
    state.food_saturation = food.saturation;
    state.food_exhaustion = food.exhaustion;
    state.food_tick_timer = food.tick_timer;
}

pub fn add_player_food_exhaustion(state: &mut PlaySessionState, amount: f32) {
    if state.abilities.invulnerable {
        return;
    }
    let mut food = food_state_from_play_session(state);
    food.add_exhaustion(amount);
    apply_food_state_to_play_session(state, food);
}

pub fn tick_play_session_food(
    state: &mut PlaySessionState,
    difficulty: FoodDifficulty,
    natural_regen: bool,
    tick_count: u64,
) -> bool {
    if state.health <= 0.0 {
        // TODO(player-death-event-flow): there is currently no live death handler.
        // On the health<=0 transition Java fires ServerPlayer.die(): send
        // ClientboundPlayerCombatKillPacket (id 68), spawn the inventory as item
        // entities via Inventory::death_drops (player_inventory.rs) gated on the
        // keepInventory gamerule, spawn XP orbs totalling
        // player_xp_reward_on_death (player_entity.rs), record last_death_location,
        // then await the client's respawn request. The drop/orb logic exists and
        // is unit-tested but is not wired here because the server has no live
        // entity-simulation tick yet (item entities + XP orbs never spawn/tick/
        // get picked up — see experience_system.rs orb_pickup_in_range /
        // non_living_entity.rs merge, both currently test-only). Blocks PLAYER
        // checklist #38 (XP orb pickup/merge), #40 (death drops), #41 (respawn).
        return false;
    }

    let old_health = state.health;
    let old_food_level = state.food_level;
    let old_saturation_zero = state.food_saturation == 0.0;

    // Java: ServerPlayer.tickRegeneration() runs from LivingEntity.tick()
    // before ServerPlayer.doTick() calls FoodData.tick(this).
    if difficulty == FoodDifficulty::Peaceful && natural_regen {
        if tick_count.is_multiple_of(20) {
            if state.health < 20.0 {
                state.health = (state.health + 1.0).min(20.0);
            }
            if state.food_saturation < 20.0 {
                state.food_saturation += 1.0;
            }
        }
        if tick_count.is_multiple_of(10) && state.food_level < 20 {
            state.food_level += 1;
        }
    }

    let mut food = food_state_from_play_session(state);
    match food.tick_food(state.health < 20.0, natural_regen, difficulty) {
        FoodTickOutcome::None => {}
        FoodTickOutcome::FastHeal { amount, .. } => {
            state.health = (state.health + amount).min(20.0);
        }
        FoodTickOutcome::SlowHeal => {
            state.health = (state.health + 1.0).min(20.0);
        }
        FoodTickOutcome::StarveAttempt => {
            if starvation_damages(difficulty, state.health) {
                state.health = (state.health - 1.0).max(0.0);
            }
        }
    }
    apply_food_state_to_play_session(state, food);

    state.health != old_health
        || state.food_level != old_food_level
        || (state.food_saturation == 0.0) != old_saturation_zero
}

pub fn food_difficulty_from_properties(properties: &ServerProperties) -> FoodDifficulty {
    // Java DedicatedServer.initServer(): hardcore forces Hard difficulty.
    if properties.hardcore {
        return FoodDifficulty::Hard;
    }
    match properties.difficulty.as_str() {
        "0" | "peaceful" => FoodDifficulty::Peaceful,
        "2" | "normal" => FoodDifficulty::Normal,
        "3" | "hard" => FoodDifficulty::Hard,
        _ => FoodDifficulty::Easy,
    }
}

pub fn write_play_state_health_packet<W: Write>(
    writer: &mut W,
    compression: CompressionState,
    state: &PlaySessionState,
) -> io::Result<()> {
    write_framed_packet_with_compression(
        writer,
        compression,
        CLIENTBOUND_SET_HEALTH_PACKET_ID,
        |payload| {
            payload.write_all(&state.health.to_be_bytes())?;
            write_var_i32(payload, state.food_level)?;
            payload.write_all(&state.food_saturation.to_be_bytes())
        },
    )
}

pub fn play_state_air_supply_metadata_packet(
    state: &PlaySessionState,
) -> io::Result<ClientboundSetEntityDataPacket> {
    Ok(ClientboundSetEntityDataPacket {
        id: PLAYER_ENTITY_ID,
        packed_items: vec![EntityDataValue::typed(
            1,
            EntityMetadataValue::VarInt(state.air_supply),
        )?],
    })
}

pub fn write_play_state_air_supply_packet<W: Write>(
    writer: &mut W,
    compression: CompressionState,
    state: &PlaySessionState,
) -> io::Result<()> {
    let packet = play_state_air_supply_metadata_packet(state)?;
    write_framed_packet_with_compression(
        writer,
        compression,
        CLIENTBOUND_SET_ENTITY_DATA_PACKET_ID,
        |payload| packet.write(payload),
    )
}

pub fn write_play_state_motion_packet<W: Write>(
    writer: &mut W,
    compression: CompressionState,
    state: &PlaySessionState,
) -> io::Result<()> {
    let packet = ClientboundSetEntityMotionPacket::new(
        PLAYER_ENTITY_ID,
        Vec3 {
            x: state.water_velocity_x,
            y: state.water_velocity_y,
            z: state.water_velocity_z,
        },
    );
    write_framed_packet_with_compression(
        writer,
        compression,
        CLIENTBOUND_SET_ENTITY_MOTION_PACKET_ID,
        |payload| packet.write(payload),
    )
}

pub struct UseItemOnContext<'a, 'b> {
    pub world_layout: &'a WorldLayout,
    pub world_seed: i64,
    pub chunk_cache: &'a GeneratedChunkCache,
    pub recipe_manager: &'a RecipeManagerModel,
    pub live_fluid_ticks: &'b mut LiveFluidTicks,
    pub live_block_ticks: &'b mut LiveBlockTicks,
    pub game_time: i64,
    pub max_chained_neighbor_updates: i32,
    // Spawn-protection inputs (Java handleUseItemOn -> isUnderSpawnProtection),
    // mirroring PlayerActionContext on the block-break path.
    pub player_access: &'a Arc<Mutex<PlayerAccess>>,
    pub profile_uuid: &'a str,
    pub spawn_protection_radius: u32,
}

pub(super) fn write_block_change_ack<W: Write>(
    writer: &mut W,
    compression: CompressionState,
    sequence: i32,
) -> io::Result<()> {
    write_framed_packet_with_compression(
        writer,
        compression,
        CLIENTBOUND_BLOCK_CHANGED_ACK_PACKET_ID,
        |payload| write_var_i32(payload, sequence),
    )
}

fn held_item_slot_for_use_item_on(
    state: &PlaySessionState,
    packet: &ServerboundUseItemOnPacket,
) -> usize {
    match packet.hand {
        ServerboundSwingHand::MainHand => state.selected_slot as usize,
        ServerboundSwingHand::OffHand => SLOT_OFFHAND,
    }
}

pub(super) fn raw_stack_for_player_inventory(stack: &ItemStack) -> RawItemStack {
    if stack.is_empty() {
        return RawItemStack::empty();
    }
    item_protocol_id(stack.item_id()).map_or_else(RawItemStack::empty, |pid| RawItemStack {
        count: stack.count(),
        item_id: Some(pid),
        components: RawDataComponentPatch::empty(),
    })
}

pub(super) fn write_player_inventory_slot_update<W: Write>(
    writer: &mut W,
    compression: CompressionState,
    slot: usize,
    contents: RawItemStack,
) -> io::Result<()> {
    write_framed_packet_with_compression(
        writer,
        compression,
        CLIENTBOUND_SET_PLAYER_INVENTORY_PACKET_ID,
        |payload| {
            ClientboundSetPlayerInventoryPacket {
                slot: slot as i32,
                contents,
            }
            .write(payload)
        },
    )
}

/// Handles a block-placement request from the client.
///
/// Java: ServerPlayerGameMode.useItemOn() -> BlockItem.place() -> Level.setBlock()
/// Spawn-protection gate for block placement/interaction, mirroring the
/// block-break path's `block_break_is_spawn_protected`: cheap in-memory op guards
/// first, then read the world spawn (level.dat) only when a non-op interacts on a
/// server that has operators. 1:1 with Java `DedicatedServer.isUnderSpawnProtection`.
fn use_item_on_spawn_protected(
    context: &UseItemOnContext<'_, '_>,
    pos: crate::block_update::BlockPos,
) -> bool {
    if context.spawn_protection_radius == 0 {
        return false;
    }
    {
        let access = lock_status_mutex(context.player_access);
        if !access.has_ops() || access.is_op(context.profile_uuid) {
            return false;
        }
    }
    let spawn = world_spawn_suggestion(context.world_layout.root(), context.world_seed);
    let world_spawn = crate::block_update::BlockPos {
        x: spawn.0,
        y: spawn.1,
        z: spawn.2,
    };
    spawn_protection_break_denied(
        &lock_status_mutex(context.player_access),
        context.spawn_protection_radius,
        world_spawn,
        context.profile_uuid,
        pos,
    )
}

/// Java `handleUseItemOn` build-height check (lines 1351-1357,
/// sendBuildLimitMessage + skip) and spawn protection (line 1358,
/// sendSpawnProtectionMessage): writes the denial message and returns whether
/// the interaction is rejected (the caller acks the sequence).
fn use_item_on_denied_by_world_gates(
    stream: &mut TcpStream,
    compression: CompressionState,
    context: &UseItemOnContext<'_, '_>,
    clicked_pos: crate::block_update::BlockPos,
) -> io::Result<bool> {
    let max_y = crate::world::OVERWORLD_MIN_Y + crate::world::OVERWORLD_LEVEL_HEIGHT - 1;
    let min_y = crate::world::OVERWORLD_MIN_Y;
    if clicked_pos.y > max_y {
        write_build_limit_message(stream, compression, true, max_y)?;
        return Ok(true);
    }
    if clicked_pos.y < min_y {
        write_build_limit_message(stream, compression, false, min_y)?;
        return Ok(true);
    }
    if use_item_on_spawn_protected(context, clicked_pos) {
        write_spawn_protection_message(
            stream,
            compression,
            clicked_pos.x,
            clicked_pos.y,
            clicked_pos.z,
        )?;
        return Ok(true);
    }
    Ok(false)
}

pub fn handle_use_item_on(
    stream: &mut TcpStream,
    compression: CompressionState,
    state: &mut PlaySessionState,
    mut context: UseItemOnContext<'_, '_>,
    packet: &ServerboundUseItemOnPacket,
) -> io::Result<()> {
    if state.game_mode == GameMode::Spectator {
        return write_block_change_ack(stream, compression, packet.sequence);
    }

    // Java ServerGamePacketListenerImpl.handleUseItemOn (line 1345) gates the
    // interaction on isWithinBlockInteractionRange(pos, 1.0); an out-of-reach
    // use is ignored (the sequence is still acked). Same server-authoritative
    // reach check the block-break path uses.
    let clicked_pos = crate::block_update::BlockPos {
        x: packet.block_hit.x,
        y: packet.block_hit.y,
        z: packet.block_hit.z,
    };
    if !super::player_creative_packets::is_within_block_interaction_range(state, clicked_pos) {
        return write_block_change_ack(stream, compression, packet.sequence);
    }
    if use_item_on_denied_by_world_gates(stream, compression, &context, clicked_pos)? {
        return write_block_change_ack(stream, compression, packet.sequence);
    }

    // Java: ServerPlayerGameMode.useItemOn() calls player.getItemInHand(hand).
    let held_slot = held_item_slot_for_use_item_on(state, packet);
    let held_item = state
        .inventory_menu
        .player_inventory()
        .get(held_slot)
        .clone();
    let suppress_using_block = state.input_shift && !held_item.is_empty();
    let clicked_state = read_live_block_model_at(
        context.chunk_cache,
        context.world_layout,
        context.world_seed,
        clicked_pos,
    );
    if !suppress_using_block {
        if let Some(menu) = block_menu_open_for_state(&clicked_state) {
            let container_id = next_open_container_id(state);
            let active_menu = ActiveBlockMenu::open(
                container_id,
                clicked_pos,
                menu.live_kind,
                context.world_layout,
                context.world_seed,
                context.chunk_cache,
                context.recipe_manager.recipe_map(),
            );
            write_open_block_menu(stream, compression, container_id, menu, packet.sequence)?;
            active_menu.write_full_content(stream, compression, state)?;
            state.active_block_menu = Some(active_menu);
            return Ok(());
        }
    }
    if held_item.is_empty() {
        return write_block_change_ack(stream, compression, packet.sequence);
    }

    let item_name = held_item.item_id();
    if let Some(kind) = bucket_fluid_kind(item_name) {
        return handle_bucket_place_fluid(
            stream,
            compression,
            state,
            context,
            packet,
            held_slot,
            kind,
        );
    }

    // Java ServerPlayerGameMode.useItemOn second stage: itemStack.useOn(
    // context) dispatches on the item class. Behavioral (non-BlockItem)
    // useOn overrides are wired in item_use_live; flint and steel is the
    // first member (the rest of the family sits on
    // TODO(item-use-block-and-entity-behaviors) in item_family_behavior.rs).
    if item_name == "minecraft:flint_and_steel" {
        return super::item_use_live::use_flint_and_steel(
            stream,
            compression,
            state,
            &mut context,
            packet,
            held_slot,
        );
    }

    // Java: BlockItem.place() only proceeds for items backed by a Block.
    if block_state_name_network_id(item_name).is_none() {
        return write_block_change_ack(stream, compression, packet.sequence);
    }

    let target = super::block_placement_live::resolve_block_item_placement_target(
        context.world_layout,
        context.world_seed,
        context.chunk_cache,
        packet,
    );
    if !block_item_can_replace(&target.existing_state) {
        return write_block_change_ack(stream, compression, packet.sequence);
    }

    // Java BlockItem.place: BlockPlaceContext -> getPlacementState ->
    // canSurvive -> Level.setBlock + neighbour shape updates; the whole
    // pipeline lives in block_placement_live.
    super::block_placement_live::place_block_item_live(
        stream,
        compression,
        state,
        &mut context,
        packet,
        item_name,
        clicked_pos,
        target,
        held_slot,
    )
}

pub fn bucket_fluid_kind(item_name: &str) -> Option<FluidKind> {
    match item_name {
        "minecraft:water_bucket" => Some(FluidKind::Water),
        "minecraft:lava_bucket" => Some(FluidKind::Lava),
        _ => None,
    }
}

pub fn handle_bucket_place_fluid(
    stream: &mut TcpStream,
    compression: CompressionState,
    state: &mut PlaySessionState,
    context: UseItemOnContext<'_, '_>,
    packet: &ServerboundUseItemOnPacket,
    held_slot: usize,
    kind: FluidKind,
) -> io::Result<()> {
    let (dx, dy, dz) = direction_offset(packet.block_hit.direction);
    let clicked = crate::block_update::BlockPos {
        x: packet.block_hit.x,
        y: packet.block_hit.y,
        z: packet.block_hit.z,
    };
    let adjacent = crate::block_update::BlockPos {
        x: clicked.x + dx,
        y: clicked.y + dy,
        z: clicked.z + dz,
    };
    let clicked_state = read_live_block_model_at(
        context.chunk_cache,
        context.world_layout,
        context.world_seed,
        clicked,
    );
    let target = if kind == FluidKind::Water && clicked_state.property("waterlogged").is_some() {
        clicked
    } else {
        adjacent
    };
    let existing = if target == clicked {
        clicked_state
    } else {
        read_live_block_model_at(
            context.chunk_cache,
            context.world_layout,
            context.world_seed,
            target,
        )
    };
    let placed = match place_liquid(&existing, kind) {
        LiquidPlaceResult::Rejected(_) => None,
        LiquidPlaceResult::Replaced(state) | LiquidPlaceResult::Waterlogged(state) => Some(state),
    };
    let Some(fluid_state) = placed else {
        return write_block_change_ack(stream, compression, packet.sequence);
    };
    // Java mirror: BucketItem.emptyContents → Level.setBlock. In-memory
    // mutation only; the periodic flush thread persists.
    context.chunk_cache.set_block(
        context.world_layout.root(),
        context.world_seed,
        target,
        &block_state_model_name(&fluid_state),
    );
    context
        .live_fluid_ticks
        .schedule(context.game_time, target, kind);
    schedule_neighbor_fluids(
        context.live_fluid_ticks,
        context.game_time,
        context.world_layout,
        context.world_seed,
        context.chunk_cache,
        target,
    );

    write_block_change_ack(stream, compression, packet.sequence)?;
    write_single_block_update(stream, compression, target, &fluid_state)?;

    if state.game_mode != GameMode::Creative {
        state
            .inventory_menu
            .player_inventory_mut()
            .set(held_slot, ItemStack::new("minecraft:bucket", 1));
        let raw = item_protocol_id("minecraft:bucket").map_or_else(RawItemStack::empty, |pid| {
            RawItemStack {
                count: 1,
                item_id: Some(pid),
                components: RawDataComponentPatch::empty(),
            }
        });
        write_player_inventory_slot_update(stream, compression, held_slot, raw)?;
    }

    Ok(())
}

pub fn schedule_neighbor_fluids(
    live_fluid_ticks: &mut LiveFluidTicks,
    game_time: i64,
    world_layout: &WorldLayout,
    world_seed: i64,
    chunk_cache: &GeneratedChunkCache,
    pos: crate::block_update::BlockPos,
) {
    for direction in crate::fluid::fluid_neighbor_order() {
        let neighbor = pos.relative(direction);
        if let Some(fluid) = crate::fluid::fluid_state_for_block(&read_live_block_model_at(
            chunk_cache,
            world_layout,
            world_seed,
            neighbor,
        )) {
            live_fluid_ticks.schedule(game_time, neighbor, fluid.kind);
        }
    }
}

/// Java-parity chunk-load fluid restore. Mirrors
/// `LevelChunk.registerTickContainerInLevel` + `LevelChunkTicks.unpack`:
/// reads the chunk's saved `fluid_ticks` NBT (one compound per tick with
/// `i`/`x`/`y`/`z`/`t`/`p` fields per `SavedTick.codec`), and schedules
/// each into the live tick queue at `current_tick + delay`.
///
/// **Does NOT scan blocks** — Java never scans on load. Fluids that should
/// flow are scheduled either (a) by the worldgen feature that placed
/// them (via the chunk's `postProcessing` list, then unpacked here), or
/// (b) by neighbour block updates at gameplay time (e.g. a player breaks
/// a block next to water → `schedule_neighbor_fluids` fires). Freshly
/// generated chunks with no saved ticks restore zero ticks, identical to
/// vanilla.
pub fn unpack_chunk_fluid_ticks(
    live_fluid_ticks: &mut LiveFluidTicks,
    current_tick: i64,
    chunk: &LevelChunk,
) {
    let started = Instant::now();
    let mut scheduled = 0_usize;
    let mut skipped = 0_usize;
    for tag in &chunk.fluid_ticks {
        let crate::storage::nbt::Tag::Compound(fields) = tag else {
            skipped += 1;
            continue;
        };
        let mut x: Option<i32> = None;
        let mut y: Option<i32> = None;
        let mut z: Option<i32> = None;
        let mut ty: Option<&str> = None;
        let mut delay: i32 = 0;
        let mut prio: TickPriority = TickPriority::Normal;
        for (key, value) in fields {
            match (key.as_str(), value) {
                ("x", crate::storage::nbt::Tag::Int(v)) => x = Some(*v),
                ("y", crate::storage::nbt::Tag::Int(v)) => y = Some(*v),
                ("z", crate::storage::nbt::Tag::Int(v)) => z = Some(*v),
                ("i", crate::storage::nbt::Tag::String(s)) => ty = Some(s.as_str()),
                ("t", crate::storage::nbt::Tag::Int(v)) => delay = *v,
                // TickPriority.CODEC encodes the enum as its int ordinal:
                // EXTREMELY_HIGH=-3, VERY_HIGH=-2, HIGH=-1, NORMAL=0,
                // LOW=1, VERY_LOW=2, EXTREMELY_LOW=3.
                ("p", crate::storage::nbt::Tag::Int(v)) => {
                    prio = match v {
                        -3 => TickPriority::ExtremelyHigh,
                        -2 => TickPriority::VeryHigh,
                        -1 => TickPriority::High,
                        1 => TickPriority::Low,
                        2 => TickPriority::VeryLow,
                        3 => TickPriority::ExtremelyLow,
                        _ => TickPriority::Normal,
                    };
                }
                _ => {}
            }
        }
        let (Some(x), Some(y), Some(z), Some(ty)) = (x, y, z, ty) else {
            skipped += 1;
            continue;
        };
        let kind = match ty {
            "minecraft:water" | "minecraft:flowing_water" => FluidKind::Water,
            "minecraft:lava" | "minecraft:flowing_lava" => FluidKind::Lava,
            _ => {
                skipped += 1;
                continue;
            }
        };
        live_fluid_ticks.schedule_saved(
            current_tick,
            crate::block_update::BlockPos { x, y, z },
            kind,
            delay,
            prio,
        );
        scheduled += 1;
    }
    let elapsed = started.elapsed();
    if scheduled > 0 || skipped > 0 || elapsed >= Duration::from_millis(10) {
        eprintln!(
            "[fluid-timing] unpack chunk=({}, {}) scheduled={} skipped={} elapsed={}ms",
            chunk.pos.x,
            chunk.pos.z,
            scheduled,
            skipped,
            elapsed.as_millis()
        );
    }
}

pub fn write_single_block_update<W: Write>(
    writer: &mut W,
    compression: CompressionState,
    pos: crate::block_update::BlockPos,
    state: &crate::block_behavior::BlockStateModel,
) -> io::Result<()> {
    let block_name = block_state_model_name(state);
    let Some(block_state_id) = block_state_name_network_id(&block_name) else {
        return Ok(());
    };
    let packed_pos = block_pos_as_long(pos.x, pos.y, pos.z);
    write_framed_packet_with_compression(
        writer,
        compression,
        CLIENTBOUND_BLOCK_UPDATE_PACKET_ID,
        |p| {
            p.write_all(&packed_pos.to_be_bytes())?;
            write_var_i32(p, block_state_id)
        },
    )
}

pub fn process_live_fluid_ticks(
    stream: &mut TcpStream,
    compression: CompressionState,
    live_fluid_ticks: &mut LiveFluidTicks,
    game_time: i64,
    world_layout: &WorldLayout,
    world_seed: i64,
    chunk_cache: &GeneratedChunkCache,
) -> io::Result<()> {
    let total_started = Instant::now();
    let due = live_fluid_ticks.tick_due(game_time, 4096);
    let due_count = due.len();
    let mut read_current_us = 0_u128;
    let mut tick_fluid_us = 0_u128;
    let mut write_update_us = 0_u128;
    let mut neighbor_read_us = 0_u128;
    let mut changes_written = 0_usize;
    let mut result_schedules = 0_usize;
    let mut neighbor_schedules = 0_usize;
    let mut skipped_wrong_fluid = 0_usize;
    for tick in due {
        let kind = match tick.ty.as_str() {
            "minecraft:water" => FluidKind::Water,
            "minecraft:lava" => FluidKind::Lava,
            _ => continue,
        };
        let read_started = Instant::now();
        // Java parity: a fluid tick on an unloaded chunk simply doesn't fire
        // — its tick container was unregistered on chunk unload (see
        // LevelChunk.unregisterTickContainerFromLevel). Mirror that here by
        // skipping ticks whose chunk isn't ready in the cache; this also
        // stops a single tick from triggering a fresh worldgen of its own
        // chunk via the legacy `load_chunk` fallback.
        let current = match try_read_block_model_at(chunk_cache, world_layout, tick.pos) {
            Some(state) => state,
            None => {
                skipped_wrong_fluid += 1;
                read_current_us += read_started.elapsed().as_micros();
                continue;
            }
        };
        read_current_us += read_started.elapsed().as_micros();
        if crate::fluid::fluid_state_for_block(&current).is_none_or(|fluid| fluid.kind != kind) {
            skipped_wrong_fluid += 1;
            continue;
        }
        let tick_started = Instant::now();
        // Neighbour reads in the tick closure use the same non-generating
        // path. A missing neighbour is treated as air; Java reaches the
        // same conclusion via its simulation-distance ticket guarantee
        // (all neighbours are loaded before a tick fires), but where that
        // guarantee doesn't hold here we conservatively treat the
        // neighbour as air rather than recursively regenerating it.
        let result = tick_fluid(tick.pos, &current, |pos| {
            try_read_block_model_at(chunk_cache, world_layout, pos)
                .unwrap_or_else(crate::block_behavior::BlockStateModel::air)
        });
        tick_fluid_us += tick_started.elapsed().as_micros();
        for (pos, state) in result.changes {
            let write_started = Instant::now();
            // Java mirror: FlowingFluid.spreadTo / spread → Level.setBlock
            // → LevelChunk.setBlockState. In-memory mutation; the chunk
            // is marked unsaved and the periodic flush thread persists
            // later. Previously this path wrote the full 24-section
            // chunk NBT to disk on every fluid spread step, which
            // ate ~20 ms per change.
            chunk_cache.set_block(
                world_layout.root(),
                world_seed,
                pos,
                &block_state_model_name(&state),
            );
            write_update_us += write_started.elapsed().as_micros();
            changes_written += 1;
            write_single_block_update(stream, compression, pos, &state)?;
            if let Some(fluid) = crate::fluid::fluid_state_for_block(&state) {
                live_fluid_ticks.schedule(game_time, pos, fluid.kind);
            }
            for direction in crate::fluid::fluid_neighbor_order() {
                let neighbor = pos.relative(direction);
                let neighbor_started = Instant::now();
                let neighbor_state = try_read_block_model_at(chunk_cache, world_layout, neighbor)
                    .unwrap_or_else(crate::block_behavior::BlockStateModel::air);
                neighbor_read_us += neighbor_started.elapsed().as_micros();
                if let Some(fluid) = crate::fluid::fluid_state_for_block(&neighbor_state) {
                    live_fluid_ticks.schedule(game_time, neighbor, fluid.kind);
                    neighbor_schedules += 1;
                }
            }
        }
        for pos in result.schedule {
            live_fluid_ticks.schedule(game_time, pos, kind);
            result_schedules += 1;
        }
    }
    let total = total_started.elapsed();
    if due_count > 0 || total >= Duration::from_millis(10) {
        eprintln!(
            "[fluid-timing] tick game_time={} due={} skipped={} changes={} result_schedules={} neighbor_schedules={} total={}ms read_current={}us tick_fluid={}us write_update={}us neighbor_read={}us",
            game_time,
            due_count,
            skipped_wrong_fluid,
            changes_written,
            result_schedules,
            neighbor_schedules,
            total.as_millis(),
            read_current_us,
            tick_fluid_us,
            write_update_us,
            neighbor_read_us
        );
    }
    Ok(())
}
