use super::*;

pub fn evaluate_block_loot(block_name: &str, seed: u64) -> Vec<(&'static str, i32)> {
    let Some(table) = block_loot_table(block_name) else {
        return Vec::new();
    };
    let mut context = LootContext::new(LootParamSet::Block, seed);
    table
        .evaluate(&mut context)
        .into_iter()
        .filter_map(|stack| {
            if stack.count <= 0 {
                return None;
            }
            let name = item_static_name(&stack.item)?;
            Some((name, stack.count))
        })
        .collect()
}

pub fn load_chunk(layout: &WorldLayout, world_seed: i64, chunk_pos: ChunkPos) -> LevelChunk {
    let region_dir = layout.region_dir();
    if let Ok(region) = RegionFile::open(&region_dir, chunk_pos.region()) {
        if let Ok(Some((_name, tag))) = region.read_chunk_nbt(chunk_pos) {
            if let Ok(chunk) = LevelChunk::from_nbt(chunk_pos, &tag) {
                return chunk;
            }
        }
    }
    generate_overworld_spawn_chunk_for_preset_with_mode(
        chunk_pos,
        "normal",
        live_chunk_generation_mode(),
        world_seed,
        true,
    )
    .unwrap_or_else(|_| LevelChunk::empty(chunk_pos))
}

/// Nonblocking chunk lookup for fluid tick / neighbour-probe code.
///
/// Returns `Some(chunk)` if the chunk is sitting in the ready cache or can
/// be read from the region file; returns `None` if neither has it.
/// Critically, **never** triggers a fresh worldgen.
///
/// Java mirror: `Level.getChunkSource().getChunkNow(x, z)` — the
/// non-loading variant used by paths that can tolerate a `null`
/// (`getBlockState` on the loading variant would block waiting for the
/// chunk to materialise; here we don't have Java's simulation-distance
/// ticket guarantee, so a synchronous load fallback would deadlock the
/// play loop on neighbour chunks that are still in the pipeline queue).
pub fn try_get_chunk_for_neighbour_probe(
    cache: &GeneratedChunkCache,
    layout: &WorldLayout,
    chunk_pos: ChunkPos,
) -> Option<Arc<LevelChunk>> {
    if let Some(chunk) = cache.try_get_ready(chunk_pos) {
        return Some(chunk);
    }
    let region_dir = layout.region_dir();
    if let Ok(region) = RegionFile::open(&region_dir, chunk_pos.region()) {
        if let Ok(Some((_name, tag))) = region.read_chunk_nbt(chunk_pos) {
            if let Ok(chunk) = LevelChunk::from_nbt(chunk_pos, &tag) {
                return Some(Arc::new(chunk));
            }
        }
    }
    None
}

/// Cache/region-only block read; mirrors `read_block_model_at` but skips
/// the worldgen fallback. Returns `None` when neither the in-memory cache
/// nor the region file has the chunk — callers decide how to handle a
/// missing neighbour (the fluid seed path conservatively treats `None` as
/// air so a real fluid edge is never silently dropped).
pub fn try_read_block_model_at(
    cache: &GeneratedChunkCache,
    layout: &WorldLayout,
    pos: crate::block_update::BlockPos,
) -> Option<crate::block_behavior::BlockStateModel> {
    let chunk_pos = ChunkPos {
        x: pos.x.div_euclid(16),
        z: pos.z.div_euclid(16),
    };
    let chunk = try_get_chunk_for_neighbour_probe(cache, layout, chunk_pos)?;
    let entry = chunk.get_block_state_model(pos.x, pos.y, pos.z)?;
    let mut state = crate::block_behavior::BlockStateModel::new(entry.name);
    for (key, value) in entry.properties {
        state = state.with_property(&key, value);
    }
    Some(state)
}

pub fn read_block_at(
    layout: &WorldLayout,
    world_seed: i64,
    chunk_pos: ChunkPos,
    bx: i32,
    by: i32,
    bz: i32,
) -> Option<String> {
    load_chunk(layout, world_seed, chunk_pos)
        .get_block_state(bx, by, bz)
        .filter(|n| n != "minecraft:air")
}

pub fn read_block_model_at(
    layout: &WorldLayout,
    world_seed: i64,
    pos: crate::block_update::BlockPos,
) -> crate::block_behavior::BlockStateModel {
    let chunk_pos = ChunkPos {
        x: pos.x.div_euclid(16),
        z: pos.z.div_euclid(16),
    };
    let chunk = load_chunk(layout, world_seed, chunk_pos);
    if let Some(entry) = chunk.get_block_state_model(pos.x, pos.y, pos.z) {
        let mut state = crate::block_behavior::BlockStateModel::new(entry.name);
        for (key, value) in entry.properties {
            state = state.with_property(&key, value);
        }
        state
    } else {
        crate::block_behavior::BlockStateModel::air()
    }
}

/// Legacy disk write-through helper (deprecated by `GeneratedChunkCache::set_block`).
/// Retained only for tests/tools that exercise raw region I/O.
#[allow(dead_code)]
pub fn write_block_model_at(
    layout: &WorldLayout,
    world_seed: i64,
    pos: crate::block_update::BlockPos,
    state: &crate::block_behavior::BlockStateModel,
) -> bool {
    let chunk_pos = ChunkPos {
        x: pos.x.div_euclid(16),
        z: pos.z.div_euclid(16),
    };
    place_block_in_region(
        layout,
        world_seed,
        chunk_pos,
        pos.x,
        pos.y,
        pos.z,
        &block_state_model_name(state),
    )
}

// Reads the old block name from the region, sets it to air, saves, and returns the old name.
/// Legacy disk write-through break helper (deprecated by
/// `GeneratedChunkCache::set_block` with `"minecraft:air"`).
#[allow(dead_code)]
pub fn break_block_in_region(
    layout: &WorldLayout,
    world_seed: i64,
    chunk_pos: ChunkPos,
    bx: i32,
    by: i32,
    bz: i32,
) -> Option<String> {
    let region_dir = layout.region_dir();
    let Ok(region) = RegionFile::open(&region_dir, chunk_pos.region()) else {
        return None;
    };
    let mut chunk = load_chunk(layout, world_seed, chunk_pos);
    let old_name = chunk
        .get_block_state(bx, by, bz)
        .filter(|n| n != "minecraft:air");
    chunk.set_block_state(bx, by, bz, "minecraft:air");
    let nbt = chunk.to_nbt(crate::storage::datafix::TARGET_DATA_VERSION);
    let _ = region.write_chunk_nbt(chunk_pos, "", &nbt);
    old_name
}

/// Places a block at (bx, by, bz) in the region file and saves the chunk.
///
/// Returns true on success, false if the region file could not be opened.
/// Java: Level.setBlock() → ChunkAccess.setBlockState()
///
/// Legacy disk write-through (deprecated by `GeneratedChunkCache::set_block`).
#[allow(dead_code)]
pub fn place_block_in_region(
    layout: &WorldLayout,
    world_seed: i64,
    chunk_pos: ChunkPos,
    bx: i32,
    by: i32,
    bz: i32,
    block_name: &str,
) -> bool {
    let region_dir = layout.region_dir();
    let Ok(region) = RegionFile::open(&region_dir, chunk_pos.region()) else {
        return false;
    };
    let mut chunk = load_chunk(layout, world_seed, chunk_pos);
    chunk.set_block_state(bx, by, bz, block_name);
    let nbt = chunk.to_nbt(crate::storage::datafix::TARGET_DATA_VERSION);
    let _ = region.write_chunk_nbt(chunk_pos, "", &nbt);
    true
}

/// Maps a `ContainerSetContent` container slot index (0-45) for container 0 (the player
/// inventory) to the corresponding `PlayerInventory` internal slot index, or `None` for
/// crafting/result slots which have no persistent inventory backing.
///
/// Java: `InventoryMenu` slot layout:
///   0        → crafting result  (no inventory backing)
///   1–4      → crafting grid    (no inventory backing)
///   5–8      → armor HEAD/CHEST/LEGS/FEET (inventory indices 39/38/37/36)
///   9–35     → main inventory rows (same index)
///   36–44    → hotbar           (inventory indices 0–8)
///   45       → offhand          (inventory index 40 = SLOT_OFFHAND)
#[cfg(test)]
pub fn inventory_internal_slot(container_slot: usize) -> Option<usize> {
    match container_slot {
        0..=4 => None,                  // crafting result + 2×2 grid — no persistent backing
        5 => Some(39),                  // HEAD armor
        6 => Some(38),                  // CHEST armor
        7 => Some(37),                  // LEGS armor
        8 => Some(36),                  // FEET armor
        9..=35 => Some(container_slot), // main inventory (indices match)
        36..=44 => Some(container_slot - 36), // hotbar → items[0..=8]
        45 => Some(40),                 // offhand (SLOT_OFFHAND)
        _ => None,
    }
}

pub fn direction_offset(dir: Direction3d) -> (i32, i32, i32) {
    match dir {
        Direction3d::Down => (0, -1, 0),
        Direction3d::Up => (0, 1, 0),
        Direction3d::North => (0, 0, -1),
        Direction3d::South => (0, 0, 1),
        Direction3d::West => (-1, 0, 0),
        Direction3d::East => (1, 0, 0),
    }
}

/// Encodes a velocity vector using the LP (Loss-Precision) Vec3 format used in
/// `ClientboundAddEntityPacket`.
///
/// Java: `LpVec3.write` — zero vector writes a single `0` byte; non-zero writes
/// 1 + 1 + 4 bytes (plus an optional VarInt for large-magnitude vectors).
pub fn write_lp_vec3<W: Write>(writer: &mut W, vx: f64, vy: f64, vz: f64) -> io::Result<()> {
    pub fn sanitize(v: f64) -> f64 {
        if v.is_nan() {
            0.0
        } else {
            v.clamp(-1.7179869183e10, 1.7179869183e10)
        }
    }
    // Java: Math.round((value * 0.5 + 0.5) * 32766.0)
    pub fn pack(v: f64) -> i64 {
        ((v * 0.5 + 0.5) * 32766.0 + 0.5).floor() as i64
    }
    let x = sanitize(vx);
    let y = sanitize(vy);
    let z = sanitize(vz);
    // Java: Mth.absMax(a, Mth.absMax(b, c))
    let chessboard = x.abs().max(y.abs()).max(z.abs());
    if chessboard < 3.051944088384301e-5 {
        return writer.write_all(&[0u8]);
    }
    let scale = chessboard.ceil() as i64;
    let is_partial = (scale & 3) != scale;
    let markers = if is_partial { (scale & 3) | 4 } else { scale };
    let xn = pack(x / scale as f64) << 3;
    let yn = pack(y / scale as f64) << 18;
    let zn = pack(z / scale as f64) << 33;
    let buffer = markers | xn | yn | zn;
    writer.write_all(&[buffer as u8, (buffer >> 8) as u8])?;
    writer.write_all(&((buffer >> 16) as i32).to_be_bytes())?;
    if is_partial {
        write_var_i32(writer, (scale >> 2) as i32)?;
    }
    Ok(())
}

/// Returns a pseudo-random `f32` in `[0, 1)` from a 64-bit seed and a per-call index.
///
/// Used to reproduce Java's `Random.nextFloat()` scatter calls in `createItemStackToDrop`
/// without keeping a persistent RNG in game state.  The exact values don't need to match
/// Java's — they only affect cosmetic velocity scatter — but they must be uncorrelated
/// across different indices.
pub fn pseudo_rand_f32(seed: i32, index: u32) -> f32 {
    let mut x = (seed as u64)
        .wrapping_mul(0x517CC1B727220A95)
        .wrapping_add((index as u64).wrapping_mul(0x6C62272E07BB0142));
    x ^= x >> 30;
    x = x.wrapping_mul(0xBF58476D1CE4E5B9);
    x ^= x >> 27;
    x = x.wrapping_mul(0x94D049BB133111EB);
    x ^= x >> 31;
    (x >> 33) as f32 / u32::MAX as f32
}

/// Sends a bundle-wrapped `ADD_ENTITY + SET_ENTITY_DATA` pair for a single item entity.
///
/// The two packets are enclosed in `ClientboundBundlePacket` delimiters so the client
/// processes them atomically in one game tick — without this, `ADD_ENTITY` may be
/// rendered for one tick with no item stack, making the entity invisible.
///
/// Java: `ServerEntity.addPairing()` — wraps `ADD_ENTITY + SET_ENTITY_DATA` in
///       `ClientboundBundlePacket` for any entity that needs metadata at spawn.
pub fn write_item_entity_spawn_packets<W: Write>(
    stream: &mut W,
    compression: CompressionState,
    item: &DroppedItem,
    item_pid: i32,
) -> io::Result<()> {
    let eid = item.entity_id;
    let uuid_hi = (eid as u64).wrapping_mul(0x6C62_272E_07BB_0142);
    let uuid_lo = (eid as u64).wrapping_mul(0x62B8_2175_6295_C58D);
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_BUNDLE_DELIMITER_PACKET_ID,
        |_| Ok(()),
    )?;
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_ADD_ENTITY_PACKET_ID,
        |p| {
            write_var_i32(p, eid)?;
            p.write_all(&uuid_hi.to_be_bytes())?;
            p.write_all(&uuid_lo.to_be_bytes())?;
            write_var_i32(p, ITEM_ENTITY_TYPE_ID)?;
            p.write_all(&item.x.to_be_bytes())?;
            p.write_all(&item.y.to_be_bytes())?;
            p.write_all(&item.z.to_be_bytes())?;
            write_lp_vec3(p, item.vel_x, item.vel_y, item.vel_z)?;
            p.write_all(&[0u8, 0u8, 0u8])?; // xRot, yRot, yHeadRot
            write_var_i32(p, 0)
        },
    )?;
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_SET_ENTITY_DATA_PACKET_ID,
        |p| {
            write_var_i32(p, eid)?;
            p.write_all(&[8u8])?; // index 8: ItemEntity.DATA_ITEM
            write_var_i32(p, 7)?; // serializer 7: EntityDataSerializers.ITEM_STACK
            write_var_i32(p, item.count)?;
            write_var_i32(p, item_pid)?;
            write_var_i32(p, 0)?; // component add count
            write_var_i32(p, 0)?; // component remove count
            p.write_all(&[0xFFu8]) // end of metadata
        },
    )?;
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_BUNDLE_DELIMITER_PACKET_ID,
        |_| Ok(()),
    )
}

/// Handles `DROP_ITEM` (action 4, Q) and `DROP_ALL_ITEMS` (action 3, Ctrl+Q) from
/// `ServerboundPlayerActionPacket`.
///
/// Java: `ServerGamePacketListenerImpl.handlePlayerAction` → `ServerPlayer.drop(boolean)` →
///       `Inventory.removeFromSelected` → `LivingEntity.createItemStackToDrop`.
///
/// Sends:
///   1. `ClientboundSetPlayerInventoryPacket` — updates the now-depleted held slot.
///   2. `ClientboundAddEntityPacket`          — spawns the item entity at eye height.
///   3. `ClientboundSetEntityDataPacket`      — sets the item stack metadata (index 8).
pub fn handle_drop_item(
    stream: &mut TcpStream,
    compression: CompressionState,
    state: &mut PlaySessionState,
    world_items: &Arc<Mutex<WorldItemEntities>>,
    drop_all: bool,
) -> io::Result<()> {
    // Java: ServerGamePacketListenerImpl — spectators cannot drop items.
    if state.game_mode == GameMode::Spectator {
        return Ok(());
    }

    let held_slot = state.selected_slot as usize;

    // Java: Inventory.removeFromSelected(all) — remove 1 or the full stack count.
    let count_to_remove = {
        let stack = state.inventory_menu.player_inventory().get(held_slot);
        if stack.is_empty() {
            return Ok(());
        }
        if drop_all {
            stack.count()
        } else {
            1
        }
    };
    let removed = state
        .inventory_menu
        .player_inventory_mut()
        .remove(held_slot, count_to_remove);
    if removed.is_empty() {
        return Ok(());
    }

    // Update the client's held slot after removal.
    // Java: ServerPlayer.drop() → containerMenu.setRemoteSlot()
    let raw_after = {
        let stack = state.inventory_menu.player_inventory().get(held_slot);
        if stack.is_empty() {
            RawItemStack::empty()
        } else if let Some(pid) = item_protocol_id(stack.item_id()) {
            RawItemStack {
                count: stack.count(),
                item_id: Some(pid),
                components: RawDataComponentPatch::empty(),
            }
        } else {
            RawItemStack::empty()
        }
    };
    // Use ContainerSetSlot (container_id=0, with state_id) rather than SetPlayerInventory
    // so the client learns the new state_id and won't reject subsequent ContainerClick packets.
    // Java: ServerPlayer.drop() → containerMenu.setRemoteSlot() + broadcastChanges()
    //       → ClientboundContainerSetSlotPacket(containerId, incrementStateId(), slot, item).
    // The hotbar slot in the InventoryMenu is at index 36 + held_slot (menu layout: result=0,
    // crafting=1-4, armour=5-8, storage=9-35, hotbar=36-44, offhand=45).
    state.container_state_id = state.container_state_id.wrapping_add(1);
    let new_state_id = state.container_state_id;
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_CONTAINER_SET_SLOT_PACKET_ID,
        |p| {
            ClientboundContainerSetSlotPacket {
                container_id: 0,
                state_id: new_state_id,
                slot: (36 + held_slot) as i16,
                item_stack: raw_after,
            }
            .write(p)
        },
    )?;

    let Some(item_pid) = item_protocol_id(removed.item_id()) else {
        return Ok(());
    };

    // Java: LivingEntity.createItemStackToDrop — spawn at eye height minus 0.3.
    // Player eye height is 1.62 (EntityType.java: sized(0.6, 1.8).eyeHeight(1.62)).
    let drop_x = state.x;
    let drop_y = state.y + 1.62 - 0.3; // getEyeY() - 0.3F
    let drop_z = state.z;

    // Java: LivingEntity.createItemStackToDrop — directional velocity based on view angles.
    // xRot = pitch, yRot = yaw (both stored in degrees in play_state).
    let pitch_rad = (state.pitch as f64) * (std::f64::consts::PI / 180.0);
    let yaw_rad = (state.yaw as f64) * (std::f64::consts::PI / 180.0);
    let sin_pitch = pitch_rad.sin();
    let cos_pitch = pitch_rad.cos();
    let sin_yaw = yaw_rad.sin();
    let cos_yaw = yaw_rad.cos();
    let eid = world_items.lock().unwrap().alloc_entity_id();
    // Java: LivingEntity.drop() scatter randomness uses the counter value before the entity
    // ID is assigned (i.e., eid - 1), matching ItemEntity constructor random offsets.
    let r0 = pseudo_rand_f32(eid.wrapping_sub(1), 0) as f64;
    let r1 = pseudo_rand_f32(eid.wrapping_sub(1), 1) as f64;
    let r2 = pseudo_rand_f32(eid.wrapping_sub(1), 2) as f64;
    let r3 = pseudo_rand_f32(eid.wrapping_sub(1), 3) as f64;
    let scatter_dir = r0 * std::f64::consts::TAU;
    let scatter_mag = 0.02 * r1;
    let vel_x = -sin_yaw * cos_pitch * 0.3 + scatter_dir.cos() * scatter_mag;
    let vel_y = -sin_pitch * 0.3 + 0.1 + (r2 - r3) * 0.1;
    let vel_z = cos_yaw * cos_pitch * 0.3 + scatter_dir.sin() * scatter_mag;

    let item = DroppedItem {
        entity_id: eid,
        item: removed.item_id(),
        count: removed.count(),
        x: drop_x,
        y: drop_y,
        z: drop_z,
        vel_x,
        vel_y,
        vel_z,
        // Java: ItemEntity.setPickUpDelay(40) — 2-second delay before anyone can pick it up,
        // including the player who dropped it.
        pickup_delay: 40,
        age: 0,
        target_uuid: None,
    };
    write_item_entity_spawn_packets(stream, compression, &item, item_pid)?;
    world_items.lock().unwrap().entities.push(item);

    Ok(())
}

pub fn section_min_y(section_index: usize) -> i32 {
    -64 + section_index as i32 * 16
}

pub fn visible_spawn_terrain_block_count(chunk_x: i32, chunk_z: i32, section_index: usize) -> i16 {
    let mut count = 0_i16;
    let section_min_y = section_min_y(section_index);
    let section_max_y = section_min_y + 15;
    for local_z in 0..16 {
        for local_x in 0..16 {
            let top_y = visible_spawn_terrain_height(chunk_x, chunk_z, local_x, local_z);
            let column_min_y = TERRAIN_BASE_Y.max(section_min_y);
            let column_max_y = top_y.min(section_max_y);
            if column_max_y >= column_min_y {
                count += (column_max_y - column_min_y + 1) as i16;
            }
            if visible_spawn_surface_top_block_id(chunk_x, chunk_z, local_x, local_z)
                == GRASS_BLOCK_STATE_ID
                && visible_spawn_surface_feature_id(chunk_x, chunk_z, local_x, local_z).is_some()
                && top_y + 1 >= section_min_y
                && top_y < section_max_y
            {
                count += 1;
            }
        }
    }
    count
}

pub fn write_visible_spawn_terrain_block_state_container<W: Write>(
    writer: &mut W,
    chunk_x: i32,
    chunk_z: i32,
    section_index: usize,
) -> io::Result<()> {
    const BITS_PER_ENTRY: u8 = 4;
    const BLOCKS_PER_SECTION: usize = 16 * 16 * 16;
    const VALUES_PER_LONG: usize = 64 / BITS_PER_ENTRY as usize;

    writer.write_all(&[BITS_PER_ENTRY])?;
    write_var_i32(writer, 11)?;
    write_var_i32(writer, AIR_BLOCK_STATE_ID)?;
    write_var_i32(writer, STONE_BLOCK_STATE_ID)?;
    write_var_i32(writer, GRANITE_BLOCK_STATE_ID)?;
    write_var_i32(writer, DIORITE_BLOCK_STATE_ID)?;
    write_var_i32(writer, ANDESITE_BLOCK_STATE_ID)?;
    write_var_i32(writer, BEDROCK_BLOCK_STATE_ID)?;
    write_var_i32(writer, DIRT_BLOCK_STATE_ID)?;
    write_var_i32(writer, GRASS_BLOCK_STATE_ID)?;
    write_var_i32(writer, SHORT_GRASS_BLOCK_STATE_ID)?;
    write_var_i32(writer, DANDELION_BLOCK_STATE_ID)?;
    write_var_i32(writer, POPPY_BLOCK_STATE_ID)?;

    let mut storage = vec![0_u64; BLOCKS_PER_SECTION / VALUES_PER_LONG];
    let section_min_y = section_min_y(section_index);
    let section_max_y = section_min_y + 15;
    for z in 0..16 {
        for x in 0..16 {
            let top_y = visible_spawn_terrain_height(chunk_x, chunk_z, x, z);
            let column_min_y = TERRAIN_BASE_Y.max(section_min_y);
            let column_max_y = top_y.min(section_max_y);
            for global_y in column_min_y..=column_max_y {
                let local_y = (global_y - section_min_y) as usize;
                let palette_index = if global_y == top_y {
                    match visible_spawn_surface_top_block_id(chunk_x, chunk_z, x, z) {
                        STONE_BLOCK_STATE_ID => 1_u64,
                        GRANITE_BLOCK_STATE_ID => 2_u64,
                        DIORITE_BLOCK_STATE_ID => 3_u64,
                        ANDESITE_BLOCK_STATE_ID => 4_u64,
                        DIRT_BLOCK_STATE_ID => 6_u64,
                        GRASS_BLOCK_STATE_ID => 7_u64,
                        _ => unreachable!("surface top palette id is registered above"),
                    }
                } else if global_y == TERRAIN_BASE_Y {
                    5_u64
                } else {
                    6_u64
                };
                let block_index = (local_y << 8) | (z << 4) | x;
                let word_index = block_index / VALUES_PER_LONG;
                let bit_index =
                    (block_index - word_index * VALUES_PER_LONG) * BITS_PER_ENTRY as usize;
                storage[word_index] |= palette_index << bit_index;
            }
            if visible_spawn_surface_top_block_id(chunk_x, chunk_z, x, z) == GRASS_BLOCK_STATE_ID {
                if let Some(feature_id) = visible_spawn_surface_feature_id(chunk_x, chunk_z, x, z)
                    .filter(|_| top_y + 1 >= section_min_y && top_y < section_max_y)
                {
                    let palette_index = match feature_id {
                        SHORT_GRASS_BLOCK_STATE_ID => 8_u64,
                        DANDELION_BLOCK_STATE_ID => 9_u64,
                        POPPY_BLOCK_STATE_ID => 10_u64,
                        _ => unreachable!("surface feature palette id is registered above"),
                    };
                    let local_y = (top_y + 1 - section_min_y) as usize;
                    let block_index = (local_y << 8) | (z << 4) | x;
                    let word_index = block_index / VALUES_PER_LONG;
                    let bit_index =
                        (block_index - word_index * VALUES_PER_LONG) * BITS_PER_ENTRY as usize;
                    storage[word_index] |= palette_index << bit_index;
                }
            }
        }
    }

    for word in storage {
        writer.write_all(&word.to_be_bytes())?;
    }
    Ok(())
}

#[allow(dead_code)]
pub fn write_empty_bitset<W: Write>(writer: &mut W) -> io::Result<()> {
    write_var_i32(writer, 0)
}

pub fn write_minimal_damage_type_registry_packet<W: Write>(writer: &mut W) -> io::Result<()> {
    write_identifier(writer, &Identifier::parse("minecraft:damage_type").unwrap())?;
    write_var_i32(writer, DAMAGE_TYPES.len() as i32)?;
    for damage_type in DAMAGE_TYPES {
        write_identifier(
            writer,
            &Identifier::parse(&format!("minecraft:{damage_type}")).unwrap(),
        )?;
        write_bool(writer, true)?;
        write_network_nbt(
            writer,
            &Tag::Compound(vec![
                (
                    "message_id".to_string(),
                    Tag::String((*damage_type).to_string()),
                ),
                (
                    "scaling".to_string(),
                    Tag::String("when_caused_by_living_non_player".to_string()),
                ),
                ("exhaustion".to_string(), Tag::Float(0.0)),
            ]),
        )?;
    }
    Ok(())
}

pub fn write_minimal_update_tags_packet<W: Write>(writer: &mut W) -> io::Result<()> {
    write_var_i32(writer, 3)?;
    write_identifier(writer, &Identifier::parse("minecraft:damage_type").unwrap())?;
    write_var_i32(writer, DAMAGE_TYPE_TAGS.len() as i32)?;
    for (tag, entries) in DAMAGE_TYPE_TAGS {
        write_identifier(writer, &Identifier::parse(tag).unwrap())?;
        write_var_i32(writer, entries.len() as i32)?;
        for entry in *entries {
            write_var_i32(writer, *entry)?;
        }
    }
    write_identifier(
        writer,
        &Identifier::parse("minecraft:banner_pattern").unwrap(),
    )?;
    write_var_i32(writer, BANNER_PATTERN_TAGS.len() as i32)?;
    for (tag, entries) in BANNER_PATTERN_TAGS {
        write_identifier(writer, &Identifier::parse(tag).unwrap())?;
        write_var_i32(writer, entries.len() as i32)?;
        for entry in *entries {
            let index = BANNER_PATTERNS
                .iter()
                .position(|pattern| pattern == entry)
                .ok_or_else(|| {
                    io::Error::new(
                        io::ErrorKind::InvalidInput,
                        format!("unknown banner pattern tag entry {entry}"),
                    )
                })?;
            write_var_i32(writer, index as i32)?;
        }
    }
    // Timeline tags: required for the client to resolve the `timelines` HolderSet in the
    // dimension type (which references "#minecraft:in_overworld") and the `#minecraft:universal`
    // nested tag.
    //
    // IDs match the order entries are sent in write_vanilla_timeline_registry_packet:
    //   day=0, moon=1, villager_schedule=2, early_game=3
    //
    // Java refs:
    //   data/minecraft/tags/timeline/in_overworld.json  → [#universal, day, moon, early_game]
    //   data/minecraft/tags/timeline/universal.json     → [villager_schedule]
    // Tags are pre-expanded by the server (nested tag #universal resolved to its elements).
    write_identifier(writer, &Identifier::parse("minecraft:timeline").unwrap())?;
    // Two tags: #minecraft:in_overworld and #minecraft:universal.
    write_var_i32(writer, 2)?;
    // #minecraft:in_overworld expands to [villager_schedule=2, day=0, moon=1, early_game=3].
    write_identifier(
        writer,
        &Identifier::parse("minecraft:in_overworld").unwrap(),
    )?;
    write_var_i32(writer, 4)?;
    for id in [2i32, 0, 1, 3] {
        write_var_i32(writer, id)?;
    }
    // #minecraft:universal expands to [villager_schedule=2].
    write_identifier(writer, &Identifier::parse("minecraft:universal").unwrap())?;
    write_var_i32(writer, 1)?;
    write_var_i32(writer, 2)?; // villager_schedule = ID 2
    Ok(())
}

pub fn write_vanilla_known_packs_packet<W: Write>(writer: &mut W) -> io::Result<()> {
    write_var_i32(writer, 1)?;
    write_string(writer, "minecraft")?;
    write_string(writer, "core")?;
    write_string(writer, VERSION_NAME)
}

pub fn write_minimal_dimension_type_registry_packet<W: Write>(writer: &mut W) -> io::Result<()> {
    write_identifier(
        writer,
        &Identifier::parse("minecraft:dimension_type").unwrap(),
    )?;
    write_var_i32(writer, DIMENSION_TYPES.len() as i32)?;
    for dimension_type in DIMENSION_TYPES {
        write_identifier(
            writer,
            &Identifier::parse(&format!("minecraft:{dimension_type}")).unwrap(),
        )?;
        write_bool(writer, true)?;
        write_network_nbt(writer, &dimension_type_nbt(dimension_type))?;
    }
    Ok(())
}

pub fn write_vanilla_chat_type_registry_packet<W: Write>(writer: &mut W) -> io::Result<()> {
    write_identifier(writer, &Identifier::parse("minecraft:chat_type").unwrap())?;
    write_var_i32(writer, CHAT_TYPES.len() as i32)?;
    for chat_type in CHAT_TYPES {
        write_identifier(
            writer,
            &Identifier::parse(&format!("minecraft:{}", chat_type.id)).unwrap(),
        )?;
        write_bool(writer, true)?;
        write_network_nbt(writer, &chat_type_nbt(chat_type))?;
    }
    Ok(())
}

pub fn write_minimal_trim_material_registry_packet<W: Write>(writer: &mut W) -> io::Result<()> {
    write_identifier(
        writer,
        &Identifier::parse("minecraft:trim_material").unwrap(),
    )?;
    write_var_i32(writer, TRIM_MATERIALS.len() as i32)?;
    for material in TRIM_MATERIALS {
        write_identifier(
            writer,
            &Identifier::parse(&format!("minecraft:{}", material.id)).unwrap(),
        )?;
        write_bool(writer, true)?;
        write_network_nbt(writer, &trim_material_nbt(material))?;
    }
    Ok(())
}

pub fn write_vanilla_jukebox_song_registry_packet<W: Write>(writer: &mut W) -> io::Result<()> {
    write_identifier(
        writer,
        &Identifier::parse("minecraft:jukebox_song").unwrap(),
    )?;
    write_var_i32(writer, JUKEBOX_SONGS.len() as i32)?;
    for song in JUKEBOX_SONGS {
        write_identifier(
            writer,
            &Identifier::parse(&format!("minecraft:{}", song.id)).unwrap(),
        )?;
        write_bool(writer, true)?;
        write_network_nbt(writer, &jukebox_song_nbt(song))?;
    }
    Ok(())
}

pub fn write_vanilla_banner_pattern_registry_packet<W: Write>(writer: &mut W) -> io::Result<()> {
    write_identifier(
        writer,
        &Identifier::parse("minecraft:banner_pattern").unwrap(),
    )?;
    write_var_i32(writer, BANNER_PATTERNS.len() as i32)?;
    for pattern in BANNER_PATTERNS {
        write_identifier(
            writer,
            &Identifier::parse(&format!("minecraft:{pattern}")).unwrap(),
        )?;
        write_bool(writer, true)?;
        write_network_nbt(writer, &banner_pattern_nbt(pattern))?;
    }
    Ok(())
}

pub fn write_vanilla_trim_pattern_registry_packet<W: Write>(writer: &mut W) -> io::Result<()> {
    write_identifier(
        writer,
        &Identifier::parse("minecraft:trim_pattern").unwrap(),
    )?;
    write_var_i32(writer, TRIM_PATTERNS.len() as i32)?;
    for pattern in TRIM_PATTERNS {
        write_identifier(
            writer,
            &Identifier::parse(&format!("minecraft:{pattern}")).unwrap(),
        )?;
        write_bool(writer, true)?;
        write_network_nbt(writer, &trim_pattern_nbt(pattern))?;
    }
    Ok(())
}

pub fn write_vanilla_instrument_registry_packet<W: Write>(writer: &mut W) -> io::Result<()> {
    write_identifier(writer, &Identifier::parse("minecraft:instrument").unwrap())?;
    write_var_i32(writer, INSTRUMENTS.len() as i32)?;
    for instrument in INSTRUMENTS {
        write_identifier(
            writer,
            &Identifier::parse(&format!("minecraft:{}", instrument.id)).unwrap(),
        )?;
        write_bool(writer, true)?;
        write_network_nbt(writer, &instrument_nbt(instrument))?;
    }
    Ok(())
}

/// Java: net/minecraft/world/clock/WorldClock.java — `record WorldClock()` with DIRECT_CODEC =
/// `MapCodec.unitCodec(...)`, which encodes as an empty NBT compound.
/// Java: net/minecraft/world/clock/WorldClocks.java:12–13 — overworld registered first (ID 0),
/// the_end second (ID 1). This order defines the VarInt IDs used in ClientboundSetTimePacket.
/// Java: net/minecraft/resources/RegistryDataLoader.java:125,160 — WORLD_CLOCK is a
/// datapack-loaded registry that must be synced to clients during the configuration phase.
pub fn write_world_clock_registry_packet<W: Write>(writer: &mut W) -> io::Result<()> {
    write_identifier(writer, &Identifier::parse("minecraft:world_clock").unwrap())?;
    write_var_i32(writer, 2)?; // minecraft:overworld (ID 0) and minecraft:the_end (ID 1)
    for name in ["overworld", "the_end"] {
        write_identifier(
            writer,
            &Identifier::parse(&format!("minecraft:{name}")).unwrap(),
        )?;
        write_bool(writer, true)?;
        // WorldClock is a zero-field record; its NBT codec encodes as an empty compound.
        write_network_nbt(writer, &Tag::Compound(vec![]))?;
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Timeline registry helpers
// ---------------------------------------------------------------------------
//
// Java refs:
//   net/minecraft/world/timeline/Timeline.java:49 — NETWORK_CODEC filters to syncable tracks
//   net/minecraft/util/Keyframe.java:8–11         — {ticks: int, value: T} compound
//   net/minecraft/util/KeyframeTrack.java:21–29   — {keyframes: [...], ease: EasingType}
//   net/minecraft/world/timeline/AttributeTrack.java:16–18 — modifier dispatch + KeyframeTrack
//   net/minecraft/util/EasingType.java             — linear omitted (default); cubic_bezier compound
//   net/minecraft/world/attribute/AttributeTypes.java — value codecs per type
//   net/minecraft/world/attribute/modifier/ColorModifier.java — ArgbModifier.argumentCodec
//   net/minecraft/world/attribute/modifier/FloatModifier.java — Simple.argumentCodec = FLOAT
//   net/minecraft/world/attribute/modifier/BooleanModifier.java — argumentCodec = BOOL
//   net/minecraft/world/attribute/EnvironmentAttributes.java — .syncable() marks network tracks
//   data/minecraft/timeline/*.json                 — authoritative keyframe data

/// Builds a keyframe compound for a 32-bit float value.
/// Java: Keyframe.codec(Codec.FLOAT) → RecordCodecBuilder {ticks: INT, value: FLOAT}
pub fn timeline_keyframe_f32(ticks: i32, value: f32) -> Tag {
    Tag::Compound(vec![
        ("ticks".to_string(), Tag::Int(ticks)),
        ("value".to_string(), Tag::Float(value)),
    ])
}

/// Builds a keyframe compound for a string value (hex colour, enum name).
/// Java: Keyframe.codec(STRING) → RecordCodecBuilder {ticks: INT, value: STRING}
pub fn timeline_keyframe_str(ticks: i32, value: &str) -> Tag {
    Tag::Compound(vec![
        ("ticks".to_string(), Tag::Int(ticks)),
        ("value".to_string(), Tag::String(value.to_string())),
    ])
}

/// Builds a keyframe compound for a boolean value.
/// Java: Codec.BOOL encodes as ByteTag (1 = true, 0 = false) in NbtOps.
pub fn timeline_keyframe_bool(ticks: i32, value: bool) -> Tag {
    Tag::Compound(vec![
        ("ticks".to_string(), Tag::Int(ticks)),
        ("value".to_string(), Tag::Byte(value as i8)),
    ])
}

/// Builds a keyframe compound for a raw 32-bit signed integer value.
/// Java: ARGB_COLOR type with multiply modifier — ArgbModifier.argumentCodec selects
/// Codec.INT when alpha == 0xFF (fully opaque). Keyframe value → Tag::Int.
pub fn timeline_keyframe_i32(ticks: i32, value: i32) -> Tag {
    Tag::Compound(vec![
        ("ticks".to_string(), Tag::Int(ticks)),
        ("value".to_string(), Tag::Int(value)),
    ])
}

/// Builds an `ease` compound for a cubic-bezier easing function.
/// Java: EasingType.CubicBezier.CODEC → {cubic_bezier: [x1, y1, x2, y2]}
pub fn cubic_bezier_ease(x1: f32, y1: f32, x2: f32, y2: f32) -> Tag {
    Tag::Compound(vec![(
        "cubic_bezier".to_string(),
        Tag::List(vec![
            Tag::Float(x1),
            Tag::Float(y1),
            Tag::Float(x2),
            Tag::Float(y2),
        ]),
    )])
}
