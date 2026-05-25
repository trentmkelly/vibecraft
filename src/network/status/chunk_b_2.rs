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
    format!("{player_name}[{loggable_address}] logged in with entity id {entity_id} at ({x}, {y}, {z})")
}

pub fn play_packet_is_handled_after_state_update(packet_id: i32) -> bool {
    matches!(
        packet_id,
        SERVERBOUND_KEEP_ALIVE_PACKET_ID
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
            | SERVERBOUND_SET_CARRIED_ITEM_PACKET_ID
            | SERVERBOUND_SWING_PACKET_ID
            | SERVERBOUND_USE_ITEM_PACKET_ID
    )
}

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
                | SERVERBOUND_PICK_ITEM_FROM_BLOCK_PACKET_ID
                | SERVERBOUND_PICK_ITEM_FROM_ENTITY_PACKET_ID
                | SERVERBOUND_CONTAINER_CLICK_PACKET_ID
                | SERVERBOUND_SET_CREATIVE_MODE_SLOT_PACKET_ID
                | SERVERBOUND_RECIPE_BOOK_CHANGE_SETTINGS_PACKET_ID
                | SERVERBOUND_RECIPE_BOOK_SEEN_RECIPE_PACKET_ID
                | SERVERBOUND_PLACE_RECIPE_PACKET_ID
                | SERVERBOUND_CHUNK_BATCH_RECEIVED_PACKET_ID
        )
}

pub fn process_item_pickups(
    stream: &mut TcpStream,
    compression: CompressionState,
    state: &mut PlaySessionState,
    player_uuid: &str,
    world_items: &Arc<Mutex<WorldItemEntities>>,
) -> io::Result<()> {
    // Snapshot which slots exist before any mutation so we can send only dirty ones.
    // Java: Inventory.add() mutates slots; we detect changes via PlayerInventory.times_changed().
    let times_changed_before = state.inventory_menu.player_inventory().times_changed();

    // Phase 1: Under the lock, compute all pickups, mutate entity counts and inventory,
    // then remove fully-consumed entities.  Packet sends are deferred to Phase 2 so the
    // Mutex is not held during network I/O.
    struct PickupEvent {
        entity_id: i32,
        picked_up: i32,
        fully_consumed: bool,
    }
    let mut events: Vec<PickupEvent> = Vec::new();
    {
        let mut items = world_items.lock().unwrap();
        let (px, py, pz) = (state.x, state.y, state.z);
        for entity in items.entities.iter_mut() {
            if !entity.can_be_picked_up_by(player_uuid) {
                continue;
            }
            if !item_entity::in_pickup_range(px, py, pz, entity.x, entity.y, entity.z) {
                continue;
            }
            let original_count = entity.count;
            let stack = ItemStack::new(entity.item, entity.count);
            let (picked_up, new_count) =
                match state.inventory_menu.player_inventory_mut().add(stack) {
                    InventoryAddResult::FullyAdded => (original_count, 0),
                    InventoryAddResult::PartiallyAdded { remaining } => {
                        (original_count - remaining, remaining)
                    }
                    // Inventory rejected the item (e.g. full) — skip.
                    InventoryAddResult::Rejected | InventoryAddResult::Dropped { .. } => continue,
                };
            entity.count = new_count;
            events.push(PickupEvent {
                entity_id: entity.entity_id,
                picked_up,
                fully_consumed: new_count <= 0,
            });
        }
        // Remove fully-consumed entities from the world store.
        items.entities.retain(|e| e.count > 0);
    }

    // Phase 2: Send packets — lock is released, safe to block on network I/O.
    for event in &events {
        // 1. TakeItemEntity — triggers the client-side pickup animation and sound.
        //    Java: player.take(this, orgCount) → sends TakeItemEntityPacket to all trackers.
        write_framed_packet_with_compression(
            stream,
            compression,
            CLIENTBOUND_TAKE_ITEM_ENTITY_PACKET_ID,
            |p| {
                ClientboundTakeItemEntityPacket {
                    item_entity_id: event.entity_id,
                    collector_entity_id: 1, // player always has entity ID 1 in single-session setup
                    amount: event.picked_up,
                }
                .write(p)
            },
        )?;
        // 2. RemoveEntities — only once the entire stack has been consumed.
        //    Java: if (itemStack.isEmpty()) this.discard() → RemoveEntitiesPacket.
        if event.fully_consumed {
            write_framed_packet_with_compression(
                stream,
                compression,
                CLIENTBOUND_REMOVE_ENTITIES_PACKET_ID,
                |p| {
                    write_var_i32(p, 1)?;
                    write_var_i32(p, event.entity_id)
                },
            )?;
        }
    }

    // 3. ContainerSetContent — re-sync all 46 InventoryMenu slots so the client sees the
    //    newly picked-up items AND receives the updated container_state_id it must echo in
    //    its next ContainerClick.  Using SetPlayerInventory here would be wrong: that packet
    //    carries no state_id, so incrementing container_state_id on the server while sending
    //    it leaves the client tracking the old value, causing every subsequent crafting click
    //    to be rejected as stale and the crafting result slot to remain empty.
    //
    //    Java: AbstractContainerMenu.broadcastChanges() → synchronizer.sendSlotChange()
    //          → ClientboundContainerSetSlotPacket(containerId, incrementStateId(), slot, item).
    //    We send the full ContainerSetContent (equivalent to broadcastFullState) rather than
    //    per-slot ContainerSetSlot packets for simplicity.
    if state.inventory_menu.player_inventory().times_changed() != times_changed_before {
        state.container_state_id = state.container_state_id.wrapping_add(1);
        let new_state_id = state.container_state_id;
        let slots = state.inventory_menu.all_slots();
        write_framed_packet_with_compression(
            stream,
            compression,
            CLIENTBOUND_CONTAINER_SET_CONTENT_PACKET_ID,
            |payload| {
                payload.write_all(&[0])?; // container ID 0 = player inventory menu
                write_var_i32(payload, new_state_id)?;
                write_var_i32(payload, slots.len() as i32)?;
                for stack in &slots {
                    let raw = if stack.is_empty() {
                        RawItemStack::empty()
                    } else if let Some(pid) = item_protocol_id(stack.item_id()) {
                        RawItemStack {
                            count: stack.count(),
                            item_id: Some(pid),
                            components: RawDataComponentPatch::empty(),
                        }
                    } else {
                        RawItemStack::empty()
                    };
                    raw.write_optional_trusted(payload)?;
                }
                // Cursor (carried) item — must reflect the actual server state.
                // The player may have an item on their cursor (picked up via an earlier
                // ContainerClick) at the same time a ground pickup fires; sending empty
                // here would wipe the cursor on the client and make the held item vanish.
                let carried = &state.carried_item;
                let raw_carried = if carried.is_empty() {
                    RawItemStack::empty()
                } else if let Some(pid) = item_protocol_id(carried.item_id()) {
                    RawItemStack {
                        count: carried.count(),
                        item_id: Some(pid),
                        components: RawDataComponentPatch::empty(),
                    }
                } else {
                    RawItemStack::empty()
                };
                raw_carried.write_optional_trusted(payload)
            },
        )?;
    }

    Ok(())
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

pub fn handle_play_respawn_request(
    stream: &mut TcpStream,
    compression: CompressionState,
    state: &mut PlaySessionState,
    properties: &ServerProperties,
    world_root: &Path,
    world_seed: i64,
) -> io::Result<()> {
    let spawn = find_default_player_spawn(world_root, world_seed, state.game_mode);
    apply_spawn_placement_to_state(state, spawn);
    reset_play_state_after_death_respawn(state);

    let spawn_info = CommonPlayerSpawnInfo {
        seed: world_seed,
        game_mode: state.game_mode,
        previous_game_mode: state.previous_game_mode,
        last_death_location: state.last_death_location.as_ref().map(|pos| {
            (
                Identifier::parse(&pos.dimension).unwrap_or_else(|_| {
                    Identifier::parse("minecraft:overworld").expect("valid fallback identifier")
                }),
                [pos.x, pos.y, pos.z],
            )
        }),
        ..CommonPlayerSpawnInfo::default()
    };
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_RESPAWN_PACKET_ID,
        |payload| {
            ClientboundRespawnPacket {
                spawn_info,
                data_to_keep: RespawnDataToKeep::NONE,
            }
            .write(payload)
        },
    )?;

    let center_chunk_x = chunk_coordinate(state.x);
    let center_chunk_z = chunk_coordinate(state.z);
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_SET_CHUNK_CACHE_CENTER_PACKET_ID,
        |payload| {
            write_var_i32(payload, center_chunk_x)?;
            write_var_i32(payload, center_chunk_z)
        },
    )?;
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_SET_CHUNK_CACHE_RADIUS_PACKET_ID,
        |payload| write_var_i32(payload, properties.view_distance as i32),
    )?;
    // Chunk payloads after respawn are flushed by the per-tick
    // drain_chunk_sender call in the play loop — the caller is responsible
    // for re-seeding the per-session PlayerChunkSender with the new
    // visible window. See handle_login_connection's respawn handling.

    write_framed_packet_with_compression(
        stream,
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
    )?;
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_SET_DEFAULT_SPAWN_POSITION_PACKET_ID,
        |payload| {
            let default_spawn = world_spawn_suggestion(world_root, world_seed);
            write_default_spawn_position_packet(
                payload,
                default_spawn.0,
                default_spawn.1,
                default_spawn.2,
            )
        },
    )?;
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_CHANGE_DIFFICULTY_PACKET_ID,
        |payload| {
            payload.write_all(&[1])?;
            write_bool(payload, false)
        },
    )?;
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_SET_EXPERIENCE_PACKET_ID,
        |payload| {
            payload.write_all(&state.xp_progress.to_be_bytes())?;
            write_var_i32(payload, state.xp_level)?;
            write_var_i32(payload, state.xp_total)
        },
    )?;
    write_game_event_to_writer(stream, compression, 2, 0.0)?;
    write_play_state_health_packet(stream, compression, state)?;
    write_play_state_air_supply_packet(stream, compression, state)?;
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_GAME_EVENT_PACKET_ID,
        |payload| {
            payload.write_all(&[LEVEL_CHUNKS_LOAD_START_GAME_EVENT_ID])?;
            payload.write_all(&0.0f32.to_be_bytes())
        },
    )?;
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
    find_player_spawn_near(world_root, world_seed, suggestion, game_mode).unwrap_or_else(|| {
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

pub fn overworld_respawn_pos_in_chunk(chunk: &LevelChunk, x: i32, z: i32) -> Option<(i32, i32, i32)> {
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

pub fn play_session_state_to_nbt(state: &PlaySessionState) -> Tag {
    let mut values = vec![
        ("DataVersion".to_string(), Tag::Int(4791)),
        (
            "Pos".to_string(),
            Tag::List(vec![
                Tag::Double(state.x),
                Tag::Double(state.y),
                Tag::Double(state.z),
            ]),
        ),
        (
            "Rotation".to_string(),
            Tag::List(vec![Tag::Float(state.yaw), Tag::Float(state.pitch)]),
        ),
        (
            "Motion".to_string(),
            Tag::List(vec![Tag::Double(0.0), Tag::Double(0.0), Tag::Double(0.0)]),
        ),
        ("OnGround".to_string(), Tag::Byte(i8::from(state.on_ground))),
        ("Air".to_string(), Tag::Short(state.air_supply as i16)),
        (
            "fall_distance".to_string(),
            Tag::Double(state.fall_distance as f64),
        ),
        ("Health".to_string(), Tag::Float(state.health)),
        ("foodLevel".to_string(), Tag::Int(state.food_level)),
        (
            "foodSaturationLevel".to_string(),
            Tag::Float(state.food_saturation),
        ),
        (
            "foodExhaustionLevel".to_string(),
            Tag::Float(state.food_exhaustion),
        ),
        ("foodTickTimer".to_string(), Tag::Int(state.food_tick_timer)),
        ("XpLevel".to_string(), Tag::Int(state.xp_level)),
        ("XpP".to_string(), Tag::Float(state.xp_progress)),
        ("XpTotal".to_string(), Tag::Int(state.xp_total)),
        ("XpSeed".to_string(), Tag::Int(state.xp_seed)),
        ("Score".to_string(), Tag::Int(state.score)),
        (
            "SelectedItemSlot".to_string(),
            Tag::Int(state.selected_slot),
        ),
        (
            "playerGameType".to_string(),
            Tag::Int(game_mode_legacy_id(state.game_mode)),
        ),
        (
            "Dimension".to_string(),
            Tag::String("minecraft:overworld".to_string()),
        ),
        (
            "seenCredits".to_string(),
            Tag::Byte(i8::from(state.seen_credits)),
        ),
        (
            "recipeBook".to_string(),
            Tag::Compound(vec![
                (
                    "recipes".to_string(),
                    Tag::List(
                        state
                            .inventory_menu
                            .recipe_book_known_recipes()
                            .into_iter()
                            .map(|id| Tag::String(id.to_string()))
                            .collect(),
                    ),
                ),
                (
                    "toBeDisplayed".to_string(),
                    Tag::List(
                        state
                            .inventory_menu
                            .recipe_book_highlighted_recipes()
                            .into_iter()
                            .map(|id| Tag::String(id.to_string()))
                            .collect(),
                    ),
                ),
                (
                    "isGuiOpen".to_string(),
                    Tag::Byte(i8::from(state.recipe_book_settings.crafting.open)),
                ),
                (
                    "isFilteringCraftable".to_string(),
                    Tag::Byte(i8::from(state.recipe_book_settings.crafting.filtering)),
                ),
                (
                    "isFurnaceGuiOpen".to_string(),
                    Tag::Byte(i8::from(state.recipe_book_settings.furnace.open)),
                ),
                (
                    "isFurnaceFilteringCraftable".to_string(),
                    Tag::Byte(i8::from(state.recipe_book_settings.furnace.filtering)),
                ),
                (
                    "isBlastingFurnaceGuiOpen".to_string(),
                    Tag::Byte(i8::from(state.recipe_book_settings.blast_furnace.open)),
                ),
                (
                    "isBlastingFurnaceFilteringCraftable".to_string(),
                    Tag::Byte(i8::from(state.recipe_book_settings.blast_furnace.filtering)),
                ),
                (
                    "isSmokerGuiOpen".to_string(),
                    Tag::Byte(i8::from(state.recipe_book_settings.smoker.open)),
                ),
                (
                    "isSmokerFilteringCraftable".to_string(),
                    Tag::Byte(i8::from(state.recipe_book_settings.smoker.filtering)),
                ),
            ]),
        ),
        (
            "abilities".to_string(),
            Tag::Compound(vec![
                (
                    "invulnerable".to_string(),
                    Tag::Byte(i8::from(state.abilities.invulnerable)),
                ),
                (
                    "flying".to_string(),
                    Tag::Byte(i8::from(state.abilities.flying)),
                ),
                (
                    "mayfly".to_string(),
                    Tag::Byte(i8::from(state.abilities.mayfly)),
                ),
                (
                    "instabuild".to_string(),
                    Tag::Byte(i8::from(state.abilities.instabuild)),
                ),
                (
                    "mayBuild".to_string(),
                    Tag::Byte(i8::from(state.abilities.may_build)),
                ),
                (
                    "flySpeed".to_string(),
                    Tag::Float(state.abilities.fly_speed),
                ),
                (
                    "walkSpeed".to_string(),
                    Tag::Float(state.abilities.walk_speed),
                ),
            ]),
        ),
        (
            "EnderItems".to_string(),
            Tag::List(state.ender_items.clone()),
        ),
        (
            "active_effects".to_string(),
            Tag::List(state.active_effects.clone()),
        ),
    ];
    if let Some(mode) = state.previous_game_mode {
        values.push((
            "previousPlayerGameType".to_string(),
            Tag::Int(game_mode_legacy_id(mode)),
        ));
    }
    if let Some(spawn) = &state.spawn {
        values.push(("SpawnX".to_string(), Tag::Int(spawn.x)));
        values.push(("SpawnY".to_string(), Tag::Int(spawn.y)));
        values.push(("SpawnZ".to_string(), Tag::Int(spawn.z)));
        values.push(("SpawnForced".to_string(), Tag::Byte(i8::from(spawn.forced))));
        values.push((
            "SpawnDimension".to_string(),
            Tag::String(spawn.dimension.clone()),
        ));
    }
    if let Some((x, y, z)) = state.entered_nether_position {
        values.push((
            "enteredNetherPosition".to_string(),
            Tag::Compound(vec![
                ("x".to_string(), Tag::Double(x)),
                ("y".to_string(), Tag::Double(y)),
                ("z".to_string(), Tag::Double(z)),
            ]),
        ));
    }
    if let Some(last_death) = &state.last_death_location {
        values.push((
            "LastDeathLocation".to_string(),
            Tag::Compound(vec![
                (
                    "dimension".to_string(),
                    Tag::String(last_death.dimension.clone()),
                ),
                (
                    "pos".to_string(),
                    Tag::List(vec![
                        Tag::Int(last_death.x),
                        Tag::Int(last_death.y),
                        Tag::Int(last_death.z),
                    ]),
                ),
            ]),
        ));
    }
    if let Some(root_vehicle) = &state.root_vehicle {
        values.push(("RootVehicle".to_string(), root_vehicle.clone()));
    }
    // Serialize the hotbar and main inventory (slots 0-35) as a TAG_List of TAG_Compound
    // entries, matching vanilla's player NBT format.
    // Java: ServerPlayer.addAdditionalSaveData() → Inventory.save()
    let inventory_items: Vec<Tag> = state
        .inventory_menu
        .player_inventory()
        .saved_items()
        .into_iter()
        .map(|(slot, stack)| {
            Tag::Compound(vec![
                ("Slot".to_string(), Tag::Byte(slot as i8)),
                ("id".to_string(), Tag::String(stack.item_id().to_string())),
                ("count".to_string(), Tag::Int(stack.count())),
            ])
        })
        .collect();
    values.push(("Inventory".to_string(), Tag::List(inventory_items)));
    Tag::Compound(values)
}

pub fn play_session_state_from_nbt(
    tag: &Tag,
    default_game_mode: GameMode,
    recipes: &RecipeMap,
) -> Option<PlaySessionState> {
    let compound = match tag {
        Tag::Compound(values) => values,
        _ => return None,
    };
    let pos = compound_list(compound, "Pos")?;
    let rotation = compound_list(compound, "Rotation")?;
    let [Tag::Double(x), Tag::Double(y), Tag::Double(z)] = pos else {
        return None;
    };
    let [Tag::Float(yaw), Tag::Float(pitch)] = rotation else {
        return None;
    };
    let on_ground = match compound_tag(compound, "OnGround") {
        Some(Tag::Byte(value)) => *value != 0,
        _ => true,
    };
    let fall_distance = match compound_tag(compound, "fall_distance") {
        Some(Tag::Double(value)) => (*value as f32).max(0.0),
        Some(Tag::Float(value)) => value.max(0.0),
        _ => 0.0,
    };
    let air_supply = match compound_tag(compound, "Air") {
        Some(Tag::Short(value)) => {
            i32::from(*value).clamp(DROWN_AIR_SUPPLY_THRESHOLD, MAX_AIR_SUPPLY)
        }
        Some(Tag::Int(value)) => (*value).clamp(DROWN_AIR_SUPPLY_THRESHOLD, MAX_AIR_SUPPLY),
        _ => MAX_AIR_SUPPLY,
    };
    let selected_slot = match compound_tag(compound, "SelectedItemSlot") {
        Some(Tag::Int(value)) if (0..9).contains(value) => *value,
        _ => 0,
    };
    let health = match compound_tag(compound, "Health") {
        Some(Tag::Float(value)) => value.clamp(0.0, 20.0),
        _ => 20.0,
    };
    let food_level = match compound_tag(compound, "foodLevel") {
        Some(Tag::Int(value)) => (*value).clamp(0, 20),
        _ => 20,
    };
    let food_saturation = match compound_tag(compound, "foodSaturationLevel") {
        Some(Tag::Float(value)) => value.clamp(0.0, food_level as f32),
        _ => 5.0,
    };
    let food_exhaustion = match compound_tag(compound, "foodExhaustionLevel") {
        Some(Tag::Float(value)) => value.max(0.0),
        _ => 0.0,
    };
    let food_tick_timer = match compound_tag(compound, "foodTickTimer") {
        Some(Tag::Int(value)) => (*value).max(0),
        _ => 0,
    };
    let xp_progress = match compound_tag(compound, "XpP") {
        Some(Tag::Float(value)) => value.clamp(0.0, 1.0),
        _ => 0.0,
    };
    let xp_level = match compound_tag(compound, "XpLevel") {
        Some(Tag::Int(value)) => (*value).max(0),
        _ => 0,
    };
    let xp_total = match compound_tag(compound, "XpTotal") {
        Some(Tag::Int(value)) => (*value).max(0),
        _ => 0,
    };
    let xp_seed = match compound_tag(compound, "XpSeed") {
        Some(Tag::Int(value)) => *value,
        _ => 0,
    };
    let score = match compound_tag(compound, "Score") {
        Some(Tag::Int(value)) => *value,
        _ => 0,
    };
    let game_mode = match compound_tag(compound, "playerGameType") {
        Some(Tag::Int(value)) => game_mode_from_legacy_id(*value),
        _ => default_game_mode,
    };
    let previous_game_mode = match compound_tag(compound, "previousPlayerGameType") {
        Some(Tag::Int(value)) if *value == -1 => None,
        Some(Tag::Int(value)) => Some(game_mode_from_legacy_id(*value)),
        _ => None,
    };
    let spawn = match (
        compound_tag(compound, "SpawnX"),
        compound_tag(compound, "SpawnY"),
        compound_tag(compound, "SpawnZ"),
    ) {
        (Some(Tag::Int(x)), Some(Tag::Int(y)), Some(Tag::Int(z))) => Some(PlayerSpawnData {
            dimension: match compound_tag(compound, "SpawnDimension") {
                Some(Tag::String(value)) => value.clone(),
                _ => "minecraft:overworld".to_string(),
            },
            x: *x,
            y: *y,
            z: *z,
            forced: matches!(compound_tag(compound, "SpawnForced"), Some(Tag::Byte(value)) if *value != 0),
        }),
        _ => None,
    };
    let seen_credits =
        matches!(compound_tag(compound, "seenCredits"), Some(Tag::Byte(value)) if *value != 0);
    let entered_nether_position = match compound_tag(compound, "enteredNetherPosition") {
        Some(Tag::Compound(fields)) => match (
            compound_tag(fields, "x"),
            compound_tag(fields, "y"),
            compound_tag(fields, "z"),
        ) {
            (Some(Tag::Double(x)), Some(Tag::Double(y)), Some(Tag::Double(z))) => {
                Some((*x, *y, *z))
            }
            _ => None,
        },
        _ => None,
    };
    let last_death_location = match compound_tag(compound, "LastDeathLocation") {
        Some(Tag::Compound(fields)) => match (
            compound_tag(fields, "dimension"),
            compound_list(fields, "pos"),
        ) {
            (Some(Tag::String(dimension)), Some([Tag::Int(x), Tag::Int(y), Tag::Int(z)])) => {
                Some(PlayerGlobalPosData {
                    dimension: dimension.clone(),
                    x: *x,
                    y: *y,
                    z: *z,
                })
            }
            _ => None,
        },
        _ => None,
    };
    let root_vehicle = compound_tag(compound, "RootVehicle").cloned();
    let active_effects = match compound_tag(compound, "active_effects") {
        Some(Tag::List(values)) => values.clone(),
        _ => Vec::new(),
    };
    let ender_items = match compound_tag(compound, "EnderItems") {
        Some(Tag::List(values)) => values.clone(),
        _ => Vec::new(),
    };
    let mut abilities = match compound_tag(compound, "abilities") {
        Some(Tag::Compound(fields)) => PlayerNbtAbilities {
            invulnerable: compound_bool_byte(fields, "invulnerable", false),
            flying: compound_bool_byte(fields, "flying", false),
            mayfly: compound_bool_byte(fields, "mayfly", false),
            instabuild: compound_bool_byte(fields, "instabuild", false),
            may_build: compound_bool_byte(fields, "mayBuild", true),
            fly_speed: compound_float(fields, "flySpeed", 0.05),
            walk_speed: compound_float(fields, "walkSpeed", 0.1),
        },
        _ => PlayerNbtAbilities::default_survival(),
    };
    abilities.apply_game_mode(game_mode);
    // Restore hotbar and main inventory (slots 0-35) from the TAG_List written by
    // play_session_state_to_nbt.
    // Java: ServerPlayer.readAdditionalSaveData() → Inventory.load()
    let mut inventory = PlayerInventory::new();
    if let Some(Tag::List(items)) = compound_tag(compound, "Inventory") {
        let mut loaded: Vec<(usize, ItemStack)> = Vec::new();
        for item_tag in items {
            if let Tag::Compound(fields) = item_tag {
                let slot = match compound_tag(fields, "Slot") {
                    Some(Tag::Byte(b)) => *b as u8 as usize,
                    _ => continue,
                };
                let id = match compound_tag(fields, "id") {
                    Some(Tag::String(s)) => s.as_str(),
                    _ => continue,
                };
                let count = match compound_tag(fields, "count") {
                    Some(Tag::Int(c)) => *c,
                    _ => 1,
                };
                if let Some(static_name) = item_static_name(id) {
                    if slot < 36 && count > 0 {
                        loaded.push((slot, ItemStack::new(static_name, count)));
                    }
                }
            }
        }
        if !loaded.is_empty() {
            inventory.load_items(&loaded);
        }
    }
    let (recipe_book_settings, known_recipes, highlighted_recipes) =
        load_recipe_book_from_nbt(compound, recipes);
    let mut inventory_menu = InventoryMenu::new(inventory, recipes.clone());
    inventory_menu.load_recipe_book(known_recipes, highlighted_recipes);

    Some(PlaySessionState {
        x: *x,
        y: *y,
        z: *z,
        yaw: *yaw,
        pitch: *pitch,
        on_ground,
        fall_distance,
        selected_slot,
        health,
        food_level,
        food_saturation,
        food_exhaustion,
        food_tick_timer,
        input_forward: false,
        input_backward: false,
        input_left: false,
        input_right: false,
        input_shift: false,
        input_sprinting: false,
        input_jumping: false,
        air_supply,
        in_water: false,
        eye_in_water: false,
        water_fluid_height: 0.0,
        water_velocity_x: 0.0,
        water_velocity_y: 0.0,
        water_velocity_z: 0.0,
        xp_progress,
        xp_level,
        xp_total,
        xp_seed,
        score,
        game_mode,
        previous_game_mode,
        spawn,
        seen_credits,
        entered_nether_position,
        last_death_location,
        root_vehicle,
        active_effects,
        ender_items,
        abilities,
        inventory_menu,
        carried_item: ItemStack::empty(),
        container_state_id: 0,
        recipe_book_settings,
    })
}
