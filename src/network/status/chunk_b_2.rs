use super::*;

pub fn loggable_remote_address(log_ips: bool, remote_address: &str) -> String {
    if log_ips {
        remote_address.to_string()
    } else {
        "IP hidden".to_string()
    }
}

pub fn player_login_log_message(
    player_name: &str,
    loggable_address: &str,
    entity_id: i32,
    x: f64,
    y: f64,
    z: f64,
) -> String {
    format!(
        "{player_name}[{loggable_address}] logged in with entity id {entity_id} at ({x}, {y}, {z})"
    )
}

pub fn play_packet_is_handled_after_state_update(packet_id: i32) -> bool {
    matches!(
        packet_id,
        SERVERBOUND_ATTACK_PACKET_ID
            | SERVERBOUND_KEEP_ALIVE_PACKET_ID
            | SERVERBOUND_ACCEPT_TELEPORTATION_PACKET_ID
            | SERVERBOUND_CHAT_ACK_PACKET_ID
            | SERVERBOUND_CLIENT_COMMAND_PACKET_ID
            | SERVERBOUND_CLIENT_INFORMATION_PACKET_ID
            | SERVERBOUND_CLIENT_TICK_END_PACKET_ID
            | SERVERBOUND_CONTAINER_CLOSE_PACKET_ID
            | SERVERBOUND_MOVE_PLAYER_POS_PACKET_ID
            | SERVERBOUND_MOVE_PLAYER_POS_ROT_PACKET_ID
            | SERVERBOUND_MOVE_PLAYER_ROT_PACKET_ID
            | SERVERBOUND_MOVE_PLAYER_STATUS_ONLY_PACKET_ID
            | SERVERBOUND_PLAYER_ABILITIES_PACKET_ID
            | SERVERBOUND_PLAYER_COMMAND_PACKET_ID
            | SERVERBOUND_PLAYER_INPUT_PACKET_ID
            | SERVERBOUND_PLAYER_LOADED_PACKET_ID
            | SERVERBOUND_SEEN_ADVANCEMENTS_PACKET_ID
            | SERVERBOUND_SELECT_BUNDLE_ITEM_PACKET_ID
            | SERVERBOUND_SET_CARRIED_ITEM_PACKET_ID
            | SERVERBOUND_SET_GAME_RULE_PACKET_ID
            | SERVERBOUND_SET_JIGSAW_BLOCK_PACKET_ID
            | SERVERBOUND_SET_TEST_BLOCK_PACKET_ID
            | SERVERBOUND_SPECTATE_ENTITY_PACKET_ID
            | SERVERBOUND_SWING_PACKET_ID
            | SERVERBOUND_TELEPORT_TO_ENTITY_PACKET_ID
            | SERVERBOUND_TEST_INSTANCE_BLOCK_ACTION_PACKET_ID
            | SERVERBOUND_USE_ITEM_PACKET_ID
    )
}

#[cfg(test)]
pub fn play_packet_has_live_status_handler(packet_id: i32) -> bool {
    play_packet_is_handled_after_state_update(packet_id)
        || matches!(
            packet_id,
            SERVERBOUND_COMMAND_SUGGESTION_PACKET_ID
                | SERVERBOUND_CHAT_PACKET_ID
                | SERVERBOUND_CHAT_COMMAND_PACKET_ID
                | SERVERBOUND_CHAT_COMMAND_SIGNED_PACKET_ID
                | SERVERBOUND_USE_ITEM_ON_PACKET_ID
                | SERVERBOUND_PLAYER_ACTION_PACKET_ID
                | SERVERBOUND_EDIT_BOOK_PACKET_ID
                | SERVERBOUND_INTERACT_PACKET_ID
                | SERVERBOUND_PICK_ITEM_FROM_BLOCK_PACKET_ID
                | SERVERBOUND_PICK_ITEM_FROM_ENTITY_PACKET_ID
                | SERVERBOUND_CONTAINER_CLICK_PACKET_ID
                | SERVERBOUND_SET_CREATIVE_MODE_SLOT_PACKET_ID
                | SERVERBOUND_RECIPE_BOOK_CHANGE_SETTINGS_PACKET_ID
                | SERVERBOUND_RECIPE_BOOK_SEEN_RECIPE_PACKET_ID
                | SERVERBOUND_PLACE_RECIPE_PACKET_ID
                | SERVERBOUND_CHUNK_BATCH_RECEIVED_PACKET_ID
                | SERVERBOUND_CUSTOM_PAYLOAD_PACKET_ID
        )
}

struct PickupEvent {
    entity_id: i32,
    item: &'static str,
    picked_up: i32,
    fully_consumed: bool,
}

fn collect_item_pickup_events(
    state: &mut PlaySessionState,
    player_uuid: &str,
    world_items: &Arc<Mutex<WorldItemEntities>>,
) -> io::Result<Vec<PickupEvent>> {
    let mut events: Vec<PickupEvent> = Vec::new();
    let mut items = world_items
        .lock()
        .map_err(|_| io::Error::other("world item entity lock poisoned"))?;
    let (px, py, pz) = (state.x, state.y, state.z);
    for entity in &mut items.entities {
        if !entity.can_be_picked_up_by(player_uuid) {
            continue;
        }
        if !item_entity::in_pickup_range(px, py, pz, entity.x, entity.y, entity.z) {
            continue;
        }
        let original_count = entity.count;
        let stack = ItemStack::new(entity.item, entity.count);
        let (picked_up, new_count) = match state.inventory_menu.player_inventory_mut().add(stack) {
            InventoryAddResult::FullyAdded => (original_count, 0),
            InventoryAddResult::PartiallyAdded { remaining } => {
                (original_count - remaining, remaining)
            }
            // Inventory rejected the item (e.g. full) -- skip.
            InventoryAddResult::Rejected | InventoryAddResult::Dropped { .. } => continue,
        };
        entity.count = new_count;
        events.push(PickupEvent {
            entity_id: entity.entity_id,
            item: entity.item,
            picked_up,
            fully_consumed: new_count <= 0,
        });
    }
    items.entities.retain(|entity| entity.count > 0);
    Ok(events)
}

fn write_item_pickup_packets<W: Write>(
    writer: &mut W,
    compression: CompressionState,
    events: &[PickupEvent],
) -> io::Result<()> {
    for event in events {
        write_take_item_entity_packet(writer, compression, event)?;
        if event.fully_consumed {
            write_remove_item_entity_packet(writer, compression, event.entity_id)?;
        }
    }
    Ok(())
}

fn write_take_item_entity_packet<W: Write>(
    writer: &mut W,
    compression: CompressionState,
    event: &PickupEvent,
) -> io::Result<()> {
    // Java: player.take(this, orgCount) sends TakeItemEntityPacket to all trackers.
    write_framed_packet_with_compression(
        writer,
        compression,
        CLIENTBOUND_TAKE_ITEM_ENTITY_PACKET_ID,
        |payload| {
            ClientboundTakeItemEntityPacket {
                item_entity_id: event.entity_id,
                collector_entity_id: 1, // player always has entity ID 1 in single-session setup
                amount: event.picked_up,
            }
            .write(payload)
        },
    )
}

fn write_remove_item_entity_packet<W: Write>(
    writer: &mut W,
    compression: CompressionState,
    entity_id: i32,
) -> io::Result<()> {
    // Java: if (itemStack.isEmpty()) this.discard() -> RemoveEntitiesPacket.
    write_framed_packet_with_compression(
        writer,
        compression,
        CLIENTBOUND_REMOVE_ENTITIES_PACKET_ID,
        |payload| {
            write_var_i32(payload, 1)?;
            write_var_i32(payload, entity_id)
        },
    )
}

fn raw_item_stack_for_inventory_sync(stack: &ItemStack) -> RawItemStack {
    if stack.is_empty() {
        return RawItemStack::empty();
    }
    item_protocol_id(stack.item_id()).map_or_else(RawItemStack::empty, |pid| RawItemStack {
        count: stack.count(),
        item_id: Some(pid),
        components: RawDataComponentPatch::empty(),
    })
}

fn write_pickup_inventory_sync<W: Write>(
    writer: &mut W,
    compression: CompressionState,
    state: &mut PlaySessionState,
    times_changed_before: u32,
) -> io::Result<()> {
    if state.inventory_menu.player_inventory().times_changed() == times_changed_before {
        return Ok(());
    }
    state.container_state_id = state.container_state_id.wrapping_add(1);
    let new_state_id = state.container_state_id;
    let slots = state.inventory_menu.all_slots();
    write_framed_packet_with_compression(
        writer,
        compression,
        CLIENTBOUND_CONTAINER_SET_CONTENT_PACKET_ID,
        |payload| {
            payload.write_all(&[0])?; // container ID 0 = player inventory menu
            write_var_i32(payload, new_state_id)?;
            write_var_i32(payload, slots.len() as i32)?;
            for stack in &slots {
                raw_item_stack_for_inventory_sync(stack).write_optional_trusted(payload)?;
            }
            // The carried item must reflect server state so a ground pickup cannot
            // wipe an item already held on the cursor by an earlier ContainerClick.
            raw_item_stack_for_inventory_sync(&state.carried_item).write_optional_trusted(payload)
        },
    )
}

pub fn process_item_pickups(
    stream: &mut TcpStream,
    compression: CompressionState,
    state: &mut PlaySessionState,
    player_uuid: &str,
    world_items: &Arc<Mutex<WorldItemEntities>>,
    recipe_manager: &RecipeManagerModel,
) -> io::Result<()> {
    // Java: Inventory.add() mutates slots; we detect changes via times_changed().
    let times_changed_before = state.inventory_menu.player_inventory().times_changed();
    let events = collect_item_pickup_events(state, player_uuid, world_items)?;
    write_item_pickup_packets(stream, compression, &events)?;
    unlock_recipes_for_pickups(state, recipe_manager, &events);
    // Java: AbstractContainerMenu.broadcastChanges() sends slot updates with an
    // incremented state ID. We send full ContainerSetContent for simplicity.
    write_pickup_inventory_sync(stream, compression, state, times_changed_before)?;
    write_pickup_recipe_unlocks(stream, compression, state, recipe_manager)
}

fn unlock_recipes_for_pickups(
    state: &mut PlaySessionState,
    recipe_manager: &RecipeManagerModel,
    events: &[PickupEvent],
) {
    for event in events {
        if event.picked_up <= 0 {
            continue;
        }
        for recipe_id in recipe_manager.recipes_unlocked_by_item(event.item) {
            state.inventory_menu.unlock_recipe(recipe_id);
        }
    }
}

fn write_pickup_recipe_unlocks<W: Write>(
    writer: &mut W,
    compression: CompressionState,
    state: &mut PlaySessionState,
    recipe_manager: &RecipeManagerModel,
) -> io::Result<()> {
    let unlock_events = state.inventory_menu.drain_recipe_unlock_events();
    if unlock_events.is_empty() {
        return Ok(());
    }
    let Some(packet) = build_recipe_book_add(&unlock_events, recipe_manager.recipe_map()) else {
        return Ok(());
    };
    write_framed_packet_with_compression(
        writer,
        compression,
        CLIENTBOUND_RECIPE_BOOK_ADD_PACKET_ID,
        |payload| packet.write(payload),
    )
}

pub fn load_play_session_state(
    world_root: &Path,
    uuid: &str,
    properties: &ServerProperties,
    recipes: &RecipeMap,
    world_seed: i64,
) -> PlaySessionState {
    let layout = WorldLayout::new(world_root);
    let default_game_mode = game_mode_from_name(&properties.game_mode);
    let mut state = layout
        .load_player_data(uuid)
        .ok()
        .and_then(|tag| play_session_state_from_nbt(&tag, default_game_mode, recipes))
        .unwrap_or_else(|| {
            let mut state = PlaySessionState {
                game_mode: default_game_mode,
                abilities: PlayerNbtAbilities::for_game_mode(default_game_mode),
                ..PlaySessionState::default()
            };
            let spawn = find_default_player_spawn(world_root, world_seed, default_game_mode);
            apply_spawn_placement_to_state(&mut state, spawn);
            state
        });
    if properties.force_game_mode {
        state.game_mode = default_game_mode;
        state.abilities.apply_game_mode(default_game_mode);
    }
    state
}

pub fn save_play_session_state(
    world_root: &Path,
    uuid: &str,
    state: &PlaySessionState,
) -> io::Result<()> {
    WorldLayout::new(world_root).save_player_data(uuid, &play_session_state_to_nbt(state))
}

fn respawn_last_death_location(pos: &PlayerGlobalPosData) -> io::Result<(Identifier, [i32; 3])> {
    let dimension = Identifier::parse(&pos.dimension)
        .or_else(|_| Identifier::new("minecraft", "overworld"))
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;
    Ok((dimension, [pos.x, pos.y, pos.z]))
}

fn respawn_spawn_info(
    state: &PlaySessionState,
    world_seed: i64,
) -> io::Result<CommonPlayerSpawnInfo> {
    Ok(CommonPlayerSpawnInfo {
        seed: world_seed,
        game_mode: state.game_mode,
        previous_game_mode: state.previous_game_mode,
        last_death_location: state
            .last_death_location
            .as_ref()
            .map(respawn_last_death_location)
            .transpose()?,
        ..CommonPlayerSpawnInfo::default()
    })
}

fn write_respawn_packet<W: Write>(
    writer: &mut W,
    compression: CompressionState,
    state: &PlaySessionState,
    world_seed: i64,
) -> io::Result<()> {
    let spawn_info = respawn_spawn_info(state, world_seed)?;
    write_framed_packet_with_compression(
        writer,
        compression,
        CLIENTBOUND_RESPAWN_PACKET_ID,
        |payload| {
            ClientboundRespawnPacket {
                spawn_info,
                data_to_keep: RespawnDataToKeep::NONE,
            }
            .write(payload)
        },
    )
}

fn write_respawn_chunk_cache_packets<W: Write>(
    writer: &mut W,
    compression: CompressionState,
    properties: &ServerProperties,
    state: &PlaySessionState,
) -> io::Result<()> {
    let center_chunk_x = chunk_coordinate(state.x);
    let center_chunk_z = chunk_coordinate(state.z);
    write_framed_packet_with_compression(
        writer,
        compression,
        CLIENTBOUND_SET_CHUNK_CACHE_CENTER_PACKET_ID,
        |payload| {
            write_var_i32(payload, center_chunk_x)?;
            write_var_i32(payload, center_chunk_z)
        },
    )?;
    write_framed_packet_with_compression(
        writer,
        compression,
        CLIENTBOUND_SET_CHUNK_CACHE_RADIUS_PACKET_ID,
        |payload| write_var_i32(payload, properties.view_distance as i32),
    )
}

fn write_respawn_position_packet<W: Write>(
    writer: &mut W,
    compression: CompressionState,
    state: &PlaySessionState,
) -> io::Result<()> {
    write_framed_packet_with_compression(
        writer,
        compression,
        CLIENTBOUND_PLAYER_POSITION_PACKET_ID,
        |payload| {
            write_var_i32(payload, 0)?;
            write_vec3(payload, state.x, state.y, state.z)?;
            write_vec3(payload, 0.0, 0.0, 0.0)?;
            payload.write_all(&state.yaw.to_be_bytes())?;
            payload.write_all(&state.pitch.to_be_bytes())?;
            payload.write_all(&0_i32.to_be_bytes())
        },
    )
}

fn write_respawn_default_spawn_packet<W: Write>(
    writer: &mut W,
    compression: CompressionState,
    world_root: &Path,
    world_seed: i64,
) -> io::Result<()> {
    let default_spawn = world_spawn_suggestion(world_root, world_seed);
    write_framed_packet_with_compression(
        writer,
        compression,
        CLIENTBOUND_SET_DEFAULT_SPAWN_POSITION_PACKET_ID,
        |payload| {
            write_default_spawn_position_packet(
                payload,
                default_spawn.0,
                default_spawn.1,
                default_spawn.2,
            )
        },
    )
}

fn write_respawn_status_packets<W: Write>(
    writer: &mut W,
    compression: CompressionState,
    state: &PlaySessionState,
) -> io::Result<()> {
    write_framed_packet_with_compression(
        writer,
        compression,
        CLIENTBOUND_CHANGE_DIFFICULTY_PACKET_ID,
        |payload| {
            payload.write_all(&[1])?;
            write_bool(payload, false)
        },
    )?;
    write_framed_packet_with_compression(
        writer,
        compression,
        CLIENTBOUND_SET_EXPERIENCE_PACKET_ID,
        |payload| {
            payload.write_all(&state.xp_progress.to_be_bytes())?;
            write_var_i32(payload, state.xp_level)?;
            write_var_i32(payload, state.xp_total)
        },
    )
}

fn write_respawn_game_events<W: Write>(
    writer: &mut W,
    compression: CompressionState,
    state: &PlaySessionState,
) -> io::Result<()> {
    write_game_event_to_writer(writer, compression, 2, 0.0)?;
    write_play_state_health_packet(writer, compression, state)?;
    write_play_state_air_supply_packet(writer, compression, state)?;
    write_framed_packet_with_compression(
        writer,
        compression,
        CLIENTBOUND_GAME_EVENT_PACKET_ID,
        |payload| {
            payload.write_all(&[LEVEL_CHUNKS_LOAD_START_GAME_EVENT_ID])?;
            payload.write_all(&0.0f32.to_be_bytes())
        },
    )
}

pub fn handle_play_respawn_request(
    stream: &mut TcpStream,
    compression: CompressionState,
    state: &mut PlaySessionState,
    properties: &ServerProperties,
    world_root: &Path,
    world_seed: i64,
) -> io::Result<()> {
    // TODO(respawn-config-spawn-block): consume the player's stored respawn
    // position before falling back to world default. Java
    // ServerPlayer.findRespawnPositionAndUseSpawnBlock (ServerPlayer.java:995)
    // reads RespawnConfig (the 26.1.2 `respawn` NBT compound via
    // RespawnConfig.CODEC, NOT the legacy SpawnX/SpawnY/SpawnZ this session
    // still serializes in play_session_state_nbt.rs), then resolves the spawn
    // block: respawn anchor (consume 1 charge unless forced), else bed (gated
    // on EnvironmentAttributes BED_RULE), else missingRespawnBlock -> default.
    // The primitives exist in respawn.rs (use_bed/use_respawn_anchor/
    // consume_respawn_anchor_charge/validate_spawn_point) but are not wired in
    // here, and the legacy NBT shape must be replaced with the RespawnConfig
    // codec first. Until then we always respawn at the world spawn.
    let spawn = find_default_player_spawn(world_root, world_seed, state.game_mode);
    apply_spawn_placement_to_state(state, spawn);
    reset_play_state_after_death_respawn(state);

    write_respawn_packet(stream, compression, state, world_seed)?;
    write_respawn_chunk_cache_packets(stream, compression, properties, state)?;
    // Chunk payloads after respawn are flushed by the per-tick
    // drain_chunk_sender call in the play loop — the caller is responsible
    // for re-seeding the per-session PlayerChunkSender with the new
    // visible window. See handle_login_connection's respawn handling.
    write_respawn_position_packet(stream, compression, state)?;
    write_respawn_default_spawn_packet(stream, compression, world_root, world_seed)?;
    write_respawn_status_packets(stream, compression, state)?;
    write_respawn_game_events(stream, compression, state)?;
    delay_initial_chunk_batch_for_probe(stream, compression)
}

pub fn apply_spawn_placement_to_state(state: &mut PlaySessionState, spawn: PlayerSpawnPlacement) {
    state.x = spawn.x;
    state.y = spawn.y;
    state.z = spawn.z;
    state.yaw = spawn.yaw;
    state.pitch = spawn.pitch;
}

pub fn reset_play_state_after_death_respawn(state: &mut PlaySessionState) {
    state.health = 20.0;
    state.food_level = 20;
    state.food_saturation = 5.0;
    state.food_exhaustion = 0.0;
    state.food_tick_timer = 0;
    state.input_forward = false;
    state.input_backward = false;
    state.input_left = false;
    state.input_right = false;
    state.input_shift = false;
    state.input_sprinting = false;
    state.input_jumping = false;
    state.air_supply = MAX_AIR_SUPPLY;
    state.in_water = false;
    state.eye_in_water = false;
    state.water_fluid_height = 0.0;
    state.water_velocity_x = 0.0;
    state.water_velocity_y = 0.0;
    state.water_velocity_z = 0.0;
    state.fall_distance = 0.0;
    state.on_ground = true;
    state.xp_progress = 0.0;
    state.xp_level = 0;
    state.xp_total = 0;
    state.score = 0;
}

pub fn find_default_player_spawn(
    world_root: &Path,
    world_seed: i64,
    game_mode: GameMode,
) -> PlayerSpawnPlacement {
    let suggestion = world_spawn_suggestion(world_root, world_seed);
    find_player_spawn_near(world_root, world_seed, suggestion, game_mode).unwrap_or({
        PlayerSpawnPlacement {
            x: suggestion.0 as f64 + 0.5,
            y: suggestion.1 as f64,
            z: suggestion.2 as f64 + 0.5,
            yaw: suggestion.3,
            pitch: 0.0,
        }
    })
}

pub fn world_spawn_suggestion(world_root: &Path, world_seed: i64) -> (i32, i32, i32, f32) {
    let layout = WorldLayout::new(world_root);
    if let Ok(tag) = layout.load_level_dat_with_backup() {
        if let Some(level) = PrimaryLevelData::from_level_dat(&tag) {
            return (
                level.spawn.x,
                level.spawn.y,
                level.spawn.z,
                level.spawn.angle,
            );
        }
    }

    resolve_world_preset("normal")
        .and_then(|preset| generator_find_spawn_position_for_stem(&preset.overworld, world_seed))
        .map(|pos| (pos.x, pos.y, pos.z, 0.0))
        .unwrap_or((0, SPAWN_Y as i32, 0, 0.0))
}

pub fn find_player_spawn_near(
    world_root: &Path,
    world_seed: i64,
    suggestion: (i32, i32, i32, f32),
    game_mode: GameMode,
) -> Option<PlayerSpawnPlacement> {
    let layout = WorldLayout::new(world_root);
    if game_mode != GameMode::Adventure {
        let radius = spawn_search_radius(
            SPAWN_SELECTION_CONSTANTS.default_respawn_radius,
            SPAWN_SELECTION_CONSTANTS.default_respawn_radius,
        );
        let candidate_count = spawn_search_candidate_count(radius);
        let random_offset =
            spawn_search_offset(world_seed, suggestion.0, suggestion.2, candidate_count);
        for candidate_index in 0..candidate_count {
            let Some((x, z)) = spawn_search_candidate(
                suggestion.0,
                suggestion.2,
                radius,
                random_offset,
                candidate_index,
            ) else {
                continue;
            };
            let chunk = load_chunk(
                &layout,
                world_seed,
                ChunkPos {
                    x: x.div_euclid(16),
                    z: z.div_euclid(16),
                },
            );
            let Some((spawn_x, spawn_y, spawn_z)) = overworld_respawn_pos_in_chunk(&chunk, x, z)
            else {
                continue;
            };
            if no_collision_no_liquid_in_chunk(&chunk, spawn_x, spawn_y, spawn_z) {
                return Some(PlayerSpawnPlacement {
                    x: spawn_x as f64 + 0.5,
                    y: spawn_y as f64,
                    z: spawn_z as f64 + 0.5,
                    yaw: suggestion.3,
                    pitch: 0.0,
                });
            }
        }
    }

    let chunk = load_chunk(
        &layout,
        world_seed,
        ChunkPos {
            x: suggestion.0.div_euclid(16),
            z: suggestion.2.div_euclid(16),
        },
    );
    let y = fixup_spawn_height(suggestion.1, -64, 320, |y| {
        no_collision_no_liquid_in_chunk(&chunk, suggestion.0, y, suggestion.2)
    });
    Some(PlayerSpawnPlacement {
        x: suggestion.0 as f64 + 0.5,
        y: y as f64,
        z: suggestion.2 as f64 + 0.5,
        yaw: suggestion.3,
        pitch: 0.0,
    })
}

pub fn spawn_search_offset(world_seed: i64, x: i32, z: i32, candidate_count: i32) -> i32 {
    if candidate_count <= 0 {
        return 0;
    }
    let mixed = (world_seed as u64)
        ^ (x as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15)
        ^ (z as u64).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    (mixed % candidate_count as u64) as i32
}

pub fn overworld_respawn_pos_in_chunk(
    chunk: &LevelChunk,
    x: i32,
    z: i32,
) -> Option<(i32, i32, i32)> {
    let local_x = x.rem_euclid(16) as usize;
    let local_z = z.rem_euclid(16) as usize;
    let index = local_z * 16 + local_x;
    let motion_blocking = chunk.compute_heightmap_values(HeightmapKind::MotionBlocking);
    let world_surface = chunk.compute_heightmap_values(HeightmapKind::WorldSurface);
    let ocean_floor = chunk.compute_heightmap_values(HeightmapKind::OceanFloor);
    let top_y = motion_blocking[index];
    if top_y < -64 {
        return None;
    }
    let surface_y = world_surface[index];
    let ocean_floor_y = ocean_floor[index];
    if surface_y <= top_y && surface_y > ocean_floor_y {
        return None;
    }

    for y in (-64..=top_y + 1).rev() {
        match block_kind_at(chunk, x, y, z) {
            SpawnBlockKind::Fluid => break,
            SpawnBlockKind::Solid => return Some((x, y + 1, z)),
            SpawnBlockKind::Air | SpawnBlockKind::NonSolid => {}
        }
    }
    None
}

pub fn no_collision_no_liquid_in_chunk(chunk: &LevelChunk, x: i32, y: i32, z: i32) -> bool {
    matches!(
        block_kind_at(chunk, x, y, z),
        SpawnBlockKind::Air | SpawnBlockKind::NonSolid
    ) && matches!(
        block_kind_at(chunk, x, y + 1, z),
        SpawnBlockKind::Air | SpawnBlockKind::NonSolid
    )
}

pub fn block_kind_at(chunk: &LevelChunk, x: i32, y: i32, z: i32) -> SpawnBlockKind {
    let block = chunk
        .get_block_state_name(x, y, z)
        .unwrap_or("minecraft:air");
    spawn_block_kind(block)
}
