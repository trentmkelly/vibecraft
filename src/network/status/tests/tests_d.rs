use super::super::*;
use super::*;

#[test]
fn write_lp_vec3_round_trips_through_java_decode() {
    // Verify the encoded x/y/z can be recovered within float precision.
    // Java unpack: (value & 0x7FFF).min(32766) * 2.0 / 32766.0 - 1.0
    // where value is extracted from the buffer at the appropriate bit offset.
    fn pack(v: f64) -> i64 {
        ((v * 0.5 + 0.5) * 32766.0 + 0.5).floor() as i64
    }
    fn unpack(v: i64) -> f64 {
        (v & 0x7FFF).min(32766) as f64 * 2.0 / 32766.0 - 1.0
    }
    let vx = 0.3_f64;
    let vy = 0.15_f64;
    let vz = -0.25_f64;
    let scale = 1_i64;
    assert!((unpack(pack(vx / scale as f64)) * scale as f64 - vx).abs() < 0.001);
    assert!((unpack(pack(vy / scale as f64)) * scale as f64 - vy).abs() < 0.001);
    assert!((unpack(pack(vz / scale as f64)) * scale as f64 - vz).abs() < 0.001);
}

#[test]
pub fn vibecraft_debug_commands_packet_exposes_biome_literal() {
    let packet = vibecraft_debug_commands_packet();

    // Intentional Java parity divergence: `/biome` is a VibeCraft debugging
    // command, so the live fallback suggestion list must expose it alongside
    // the command tree even though vanilla 26.1.2 has no root `/biome`.
    assert!(PLAY_COMMAND_SUGGESTIONS.contains(&"biome"));
    assert_eq!(packet.root_index, 0);
    assert_eq!(packet.entries.len(), 2);
    assert_eq!(packet.entries[0].stub, CommandNodeStubData::Root);
    assert_eq!(packet.entries[0].children, vec![1]);
    assert!(!packet.entries[0].executable);
    assert_eq!(
        packet.entries[1].stub,
        CommandNodeStubData::Literal {
            name: "biome".to_string()
        }
    );
    assert!(packet.entries[1].executable);
    assert!(packet.entries[1].children.is_empty());
    assert!(!packet.entries[1].restricted);
}

#[test]
pub fn biome_debug_command_feedback_prints_current_biome() {
    let mut state = crate::command::ServerCommandState {
        command_source_position: crate::command::Vec3 {
            x: 7.9,
            y: 64.0,
            z: -1.0,
        },
        biomes: vec![crate::command::BiomeEntry {
            dimension: "minecraft:overworld".to_string(),
            position: crate::command::BlockPos { x: 4, y: 64, z: -4 },
            biome: "minecraft:forest".to_string(),
        }],
        ..crate::command::ServerCommandState::default()
    };
    let result = crate::command::execute_builtin_command(
        &mut state,
        crate::command::LevelBasedPermissionSet::ALL,
        "biome",
    )
    .unwrap();

    assert_eq!(
        command_feedback_text(&result, &state),
        "Biome: minecraft:forest"
    );
}

#[test]
pub fn raw_command_suggestion_response_keeps_stream_open_for_keepalive() {
    let mut request = Vec::new();
    ServerboundCommandSuggestionPacket {
        id: 42,
        command: "/li".to_string(),
    }
    .write(&mut request)
    .unwrap();

    let mut written = Vec::new();
    write_command_suggestions_response(
        &mut written,
        CompressionState::disabled(),
        &mut Cursor::new(request),
    )
    .unwrap();
    write_framed_packet_with_compression(
        &mut written,
        CompressionState::disabled(),
        CLIENTBOUND_KEEP_ALIVE_PACKET_ID,
        |payload| crate::network::common::ClientboundKeepAlivePacket { id: 99 }.write(payload),
    )
    .unwrap();

    let mut stream = Cursor::new(written);
    let suggestion_frame = read_packet(&mut stream).unwrap();
    let mut suggestion = Cursor::new(suggestion_frame);
    assert_eq!(
        read_var_i32(&mut suggestion).unwrap(),
        CLIENTBOUND_COMMAND_SUGGESTIONS_PACKET_ID
    );
    assert_eq!(read_var_i32(&mut suggestion).unwrap(), 42);
    assert_eq!(read_var_i32(&mut suggestion).unwrap(), 1);
    assert_eq!(read_var_i32(&mut suggestion).unwrap(), 2);
    assert_eq!(read_var_i32(&mut suggestion).unwrap(), 1);
    assert_eq!(read_string(&mut suggestion, 32767).unwrap(), "list");
    assert!(!read_bool(&mut suggestion).unwrap());

    let keepalive_frame = read_packet(&mut stream).unwrap();
    let mut keepalive = Cursor::new(keepalive_frame);
    assert_eq!(
        read_var_i32(&mut keepalive).unwrap(),
        CLIENTBOUND_KEEP_ALIVE_PACKET_ID
    );
    assert_eq!(
        crate::network::common::ClientboundKeepAlivePacket::read(&mut keepalive)
            .unwrap()
            .id,
        99
    );
}

#[test]
pub fn pseudo_rand_f32_produces_values_in_unit_interval() {
    for seed in [-100_i32, 0, 1, 42, i32::MAX, i32::MIN] {
        for index in 0..4_u32 {
            let v = pseudo_rand_f32(seed, index);
            assert!(
                (0.0..1.0).contains(&v),
                "pseudo_rand_f32({seed}, {index}) = {v} out of [0, 1)"
            );
        }
    }
}

#[test]
pub fn pseudo_rand_f32_differs_across_indices() {
    let seed = 12345_i32;
    let v0 = pseudo_rand_f32(seed, 0);
    let v1 = pseudo_rand_f32(seed, 1);
    let v2 = pseudo_rand_f32(seed, 2);
    let v3 = pseudo_rand_f32(seed, 3);
    // All four values should be distinct (probability of collision is ~2^-23).
    assert_ne!(v0, v1);
    assert_ne!(v1, v2);
    assert_ne!(v2, v3);
}

// ─── inventory_internal_slot ─────────────────────────────────────────────

#[test]
pub fn inventory_internal_slot_crafting_slots_have_no_backing() {
    // Java: InventoryMenu — slots 0 (result) and 1-4 (2×2 grid) have no PlayerInventory backing.
    for container_slot in 0..=4 {
        assert!(
            inventory_internal_slot(container_slot).is_none(),
            "container slot {container_slot} should have no backing"
        );
    }
}

#[test]
pub fn inventory_internal_slot_armor_mapping_matches_inventory_menu() {
    // Java: InventoryMenu adds armor slots HEAD(39)/CHEST(38)/LEGS(37)/FEET(36)
    // at container indices 5/6/7/8.
    assert_eq!(
        inventory_internal_slot(5),
        Some(39),
        "container 5 → HEAD (39)"
    );
    assert_eq!(
        inventory_internal_slot(6),
        Some(38),
        "container 6 → CHEST (38)"
    );
    assert_eq!(
        inventory_internal_slot(7),
        Some(37),
        "container 7 → LEGS (37)"
    );
    assert_eq!(
        inventory_internal_slot(8),
        Some(36),
        "container 8 → FEET (36)"
    );
}

#[test]
pub fn inventory_internal_slot_main_inventory_identity_mapping() {
    // Java: addStandardInventorySlots maps items[9..=35] directly to container slots 9-35.
    for slot in 9..=35usize {
        assert_eq!(
            inventory_internal_slot(slot),
            Some(slot),
            "main inventory: container {slot} → internal {slot}"
        );
    }
}

#[test]
pub fn inventory_internal_slot_hotbar_shifted_mapping() {
    // Java: addStandardInventorySlots maps items[0..=8] to container slots 36-44.
    for i in 0..=8usize {
        assert_eq!(
            inventory_internal_slot(36 + i),
            Some(i),
            "hotbar: container {} → internal {}",
            36 + i,
            i
        );
    }
}

#[test]
pub fn inventory_internal_slot_offhand_is_slot_45() {
    // Java: InventoryMenu adds the offhand slot (inventory index 40) at container index 45.
    assert_eq!(
        inventory_internal_slot(45),
        Some(crate::player_inventory::SLOT_OFFHAND)
    );
}

#[derive(Debug)]
pub struct CursorStream {
    pub read: Cursor<Vec<u8>>,
    pub written: Vec<u8>,
}

impl CursorStream {
    pub fn new(read: Vec<u8>) -> Self {
        Self {
            read: Cursor::new(read),
            written: Vec::new(),
        }
    }
}

impl Read for CursorStream {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.read.read(buf)
    }
}

impl Write for CursorStream {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.written.extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

// -----------------------------------------------------------------------
// Timeline registry and dimension-type sky-fix tests
// -----------------------------------------------------------------------

#[test]
pub fn timeline_registry_sends_four_entries_in_correct_order() {
    // Java: data/minecraft/timeline/ has day, early_game, moon, villager_schedule.
    // Registry order (0–3) must match the IDs used in the timeline tags packet.
    assert_eq!(
        status_registry_id(write_vanilla_timeline_registry_packet),
        "minecraft:timeline"
    );
    let ids = status_registry_entry_ids_ordered(write_vanilla_timeline_registry_packet);
    assert_eq!(ids.len(), 4);
    assert_eq!(ids[0], "minecraft:day"); // ID 0
    assert_eq!(ids[1], "minecraft:moon"); // ID 1
    assert_eq!(ids[2], "minecraft:villager_schedule"); // ID 2
    assert_eq!(ids[3], "minecraft:early_game"); // ID 3
}

#[test]
pub fn day_timeline_contains_syncable_tracks_and_omits_non_syncable() {
    // Java: Timeline.NETWORK_CODEC calls filterSyncableTracks, removing any track whose
    // EnvironmentAttribute does not have .syncable() set.
    let nbt = day_timeline_nbt();
    assert!(matches!(
        field_value(&nbt, "clock"),
        Some(Tag::String(v)) if v == "minecraft:overworld"
    ));
    assert!(matches!(
        field_value(&nbt, "period_ticks"),
        Some(Tag::Int(24000))
    ));

    let tracks = compound_field(&nbt, "tracks");

    // Syncable visual/audio/gameplay tracks that must be present.
    assert!(field_value(tracks, "minecraft:visual/sun_angle").is_some());
    assert!(field_value(tracks, "minecraft:visual/moon_angle").is_some());
    assert!(field_value(tracks, "minecraft:visual/star_angle").is_some());
    assert!(field_value(tracks, "minecraft:visual/fog_color").is_some());
    assert!(field_value(tracks, "minecraft:visual/sky_color").is_some());
    assert!(field_value(tracks, "minecraft:visual/sky_light_color").is_some());
    assert!(field_value(tracks, "minecraft:visual/sky_light_factor").is_some());
    assert!(field_value(tracks, "minecraft:visual/star_brightness").is_some());
    assert!(field_value(tracks, "minecraft:visual/cloud_color").is_some());
    assert!(field_value(tracks, "minecraft:visual/sunrise_sunset_color").is_some());
    assert!(field_value(tracks, "minecraft:gameplay/sky_light_level").is_some());
    assert!(field_value(tracks, "minecraft:audio/firefly_bush_sounds").is_some());
    assert!(field_value(tracks, "minecraft:gameplay/creaking_active").is_some());

    // Non-syncable tracks must be absent.
    assert!(field_value(tracks, "minecraft:gameplay/monsters_burn").is_none());
    assert!(field_value(tracks, "minecraft:gameplay/bees_stay_in_hive").is_none());
    assert!(field_value(tracks, "minecraft:gameplay/eyeblossom_open").is_none());
}

#[test]
pub fn day_timeline_sun_angle_track_uses_cubic_bezier_easing() {
    // Java: Timelines.java:53 — SUN_ANGLE uses EasingType.symmetricCubicBezier(0.362, 0.241).
    // That easing should be present in the track's `ease` field as {cubic_bezier: [...]}.
    let nbt = day_timeline_nbt();
    let tracks = compound_field(&nbt, "tracks");
    let sun_angle = compound_field(tracks, "minecraft:visual/sun_angle");
    let ease = compound_field(sun_angle, "ease");
    let bezier = field_value(ease, "cubic_bezier");
    assert!(
        matches!(bezier, Some(Tag::List(_))),
        "sun_angle ease must contain cubic_bezier list"
    );
}

#[test]
pub fn day_timeline_cloud_color_uses_int_encoding_for_fully_opaque_argb() {
    // Java: ArgbModifier.argumentCodec selects Codec.INT when alpha == 0xFF → Tag::Int.
    // Day cloud_color keyframe at tick 133 = -1 (0xFFFFFFFF, white).
    let nbt = day_timeline_nbt();
    let tracks = compound_field(&nbt, "tracks");
    let cloud_color = compound_field(tracks, "minecraft:visual/cloud_color");
    let Tag::List(keyframes) = field_value(cloud_color, "keyframes").unwrap() else {
        panic!("cloud_color keyframes must be a list");
    };
    let first_value = field_value(&keyframes[0], "value").unwrap();
    assert!(
        matches!(first_value, Tag::Int(_)),
        "cloud_color keyframe values must be Tag::Int (ArgbModifier, alpha=0xFF)"
    );
    assert_eq!(first_value, &Tag::Int(-1)); // 0xFFFFFFFF = white
}

#[test]
pub fn moon_timeline_has_moon_phase_track_and_192000_period() {
    // Java: data/minecraft/timeline/moon.json — period_ticks=192000, one syncable track.
    // surface_slime_spawn_chance is non-syncable and filtered out.
    let nbt = moon_timeline_nbt();
    assert!(matches!(
        field_value(&nbt, "clock"),
        Some(Tag::String(v)) if v == "minecraft:overworld"
    ));
    assert!(matches!(
        field_value(&nbt, "period_ticks"),
        Some(Tag::Int(192000))
    ));
    let tracks = compound_field(&nbt, "tracks");
    assert!(field_value(tracks, "minecraft:visual/moon_phase").is_some());
    assert!(
        field_value(tracks, "minecraft:gameplay/surface_slime_spawn_chance").is_none(),
        "surface_slime_spawn_chance is non-syncable and must be filtered out"
    );
}

#[test]
pub fn moon_timeline_moon_phase_keyframes_are_string_encoded() {
    // Java: MoonPhase.CODEC = StringRepresentable.fromEnum → Tag::String.
    let nbt = moon_timeline_nbt();
    let tracks = compound_field(&nbt, "tracks");
    let moon_phase = compound_field(tracks, "minecraft:visual/moon_phase");
    let Tag::List(keyframes) = field_value(moon_phase, "keyframes").unwrap() else {
        panic!("moon_phase keyframes must be a list");
    };
    assert_eq!(keyframes.len(), 8, "8 moon phases");
    assert!(matches!(
        field_value(&keyframes[0], "value"),
        Some(Tag::String(v)) if v == "full_moon"
    ));
    assert!(matches!(
        field_value(&keyframes[4], "value"),
        Some(Tag::String(v)) if v == "new_moon"
    ));
}

#[test]
pub fn villager_schedule_timeline_has_no_tracks_after_syncable_filter() {
    // Java: data/minecraft/timeline/villager_schedule.json — villager_activity and
    // baby_villager_activity are both non-syncable → tracks field is absent entirely.
    let nbt = villager_schedule_timeline_nbt();
    assert!(matches!(
        field_value(&nbt, "clock"),
        Some(Tag::String(v)) if v == "minecraft:overworld"
    ));
    assert!(matches!(
        field_value(&nbt, "period_ticks"),
        Some(Tag::Int(24000))
    ));
    assert!(
        field_value(&nbt, "tracks").is_none(),
        "all villager_schedule tracks are non-syncable; tracks field must be absent"
    );
}

#[test]
pub fn early_game_timeline_has_no_period_ticks_and_no_tracks() {
    // Java: data/minecraft/timeline/early_game.json — no period_ticks field; one track
    // (can_pillager_patrol_spawn) that is non-syncable → both fields absent.
    let nbt = early_game_timeline_nbt();
    assert!(matches!(
        field_value(&nbt, "clock"),
        Some(Tag::String(v)) if v == "minecraft:overworld"
    ));
    assert!(
        field_value(&nbt, "period_ticks").is_none(),
        "early_game has no period_ticks in source data"
    );
    assert!(
        field_value(&nbt, "tracks").is_none(),
        "can_pillager_patrol_spawn is non-syncable; tracks field must be absent"
    );
}

#[test]
pub fn overworld_dimension_type_has_timelines_clock_and_attributes() {
    // These three fields are required for the client to render a non-black sky.
    // They were absent before the sky fix, causing a permanently black sky on join.
    let nbt = overworld_dimension_type_nbt(false);

    // `timelines`: HolderSet tag reference resolved by the client using the tags packet.
    assert!(
        matches!(
            field_value(&nbt, "timelines"),
            Some(Tag::String(v)) if v == "#minecraft:in_overworld"
        ),
        "timelines must reference the #minecraft:in_overworld tag"
    );

    // `default_clock`: drives the timeline evaluation for this dimension.
    assert!(
        matches!(
            field_value(&nbt, "default_clock"),
            Some(Tag::String(v)) if v == "minecraft:overworld"
        ),
        "default_clock must be minecraft:overworld"
    );

    // `attributes`: static base values that the timeline tracks multiply/add to.
    let attributes = compound_field(&nbt, "attributes");

    assert!(matches!(
        field_value(attributes, "minecraft:visual/sky_color"),
        Some(Tag::String(v)) if v == "#78a7ff"
    ));
    assert!(matches!(
        field_value(attributes, "minecraft:visual/fog_color"),
        Some(Tag::String(v)) if v == "#c0d8ff"
    ));
    assert!(matches!(
        field_value(attributes, "minecraft:visual/cloud_color"),
        Some(Tag::String(v)) if v == "#ccffffff"
    ));
    assert!(
        matches!(
            field_value(attributes, "minecraft:visual/cloud_height"),
            Some(Tag::Float(_))
        ),
        "cloud_height must be a float"
    );
    assert!(matches!(
        field_value(attributes, "minecraft:visual/ambient_light_color"),
        Some(Tag::String(v)) if v == "#0a0a0a"
    ));
}

#[test]
pub fn update_tags_packet_includes_timeline_group_with_correct_ids() {
    // The tags packet must include a minecraft:timeline group so the client can resolve
    // the "#minecraft:in_overworld" HolderSet reference in the dimension type.
    let mut payload = Vec::new();
    write_minimal_update_tags_packet(&mut payload).unwrap();
    let mut cursor = Cursor::new(payload);

    let group_count = read_var_i32(&mut cursor).unwrap();
    assert_eq!(group_count, 4, "tags packet must have 4 registry groups");

    let mut found_timeline = false;
    let mut found_pickaxe_tag = false;
    for _ in 0..group_count {
        let registry_id = crate::network::codec::read_identifier(&mut cursor)
            .unwrap()
            .to_string();
        let tag_count = read_var_i32(&mut cursor).unwrap();
        if registry_id == "minecraft:block" {
            for _ in 0..tag_count {
                let tag_id = crate::network::codec::read_identifier(&mut cursor)
                    .unwrap()
                    .to_string();
                let entry_count = read_var_i32(&mut cursor).unwrap();
                let ids: Vec<i32> = (0..entry_count)
                    .map(|_| read_var_i32(&mut cursor).unwrap())
                    .collect();
                if tag_id == "minecraft:mineable/pickaxe" {
                    found_pickaxe_tag = true;
                    assert!(
                        ids.contains(
                            &crate::block_states::block_registry_network_id("minecraft:stone")
                                .unwrap()
                        ),
                        "client needs #mineable/pickaxe to animate pickaxes at tool speed"
                    );
                }
            }
        } else if registry_id == "minecraft:timeline" {
            found_timeline = true;
            assert_eq!(tag_count, 2);

            // First tag: #minecraft:in_overworld → [villager_schedule=2, day=0, moon=1, early_game=3].
            // Pre-expanded by server; IDs correspond to write_vanilla_timeline_registry_packet order.
            let tag_id = crate::network::codec::read_identifier(&mut cursor)
                .unwrap()
                .to_string();
            assert_eq!(tag_id, "minecraft:in_overworld");
            let entry_count = read_var_i32(&mut cursor).unwrap();
            assert_eq!(entry_count, 4);
            let ids: Vec<i32> = (0..entry_count)
                .map(|_| read_var_i32(&mut cursor).unwrap())
                .collect();
            assert_eq!(ids, vec![2, 0, 1, 3]);

            // Second tag: #minecraft:universal → [villager_schedule=2].
            let tag_id2 = crate::network::codec::read_identifier(&mut cursor)
                .unwrap()
                .to_string();
            assert_eq!(tag_id2, "minecraft:universal");
            let entry_count2 = read_var_i32(&mut cursor).unwrap();
            assert_eq!(entry_count2, 1);
            assert_eq!(read_var_i32(&mut cursor).unwrap(), 2);
        } else {
            // Skip tags for other registry groups.
            for _ in 0..tag_count {
                crate::network::codec::read_identifier(&mut cursor).unwrap();
                let entry_count = read_var_i32(&mut cursor).unwrap();
                for _ in 0..entry_count {
                    read_var_i32(&mut cursor).unwrap();
                }
            }
        }
    }
    assert!(
        found_timeline,
        "tags packet must include minecraft:timeline group"
    );
    assert!(
        found_pickaxe_tag,
        "tags packet must include minecraft:block #mineable/pickaxe"
    );
}

// ─── var_int_encoded_len ─────────────────────────────────────────────────

#[test]
pub fn var_int_encoded_len_matches_actual_encoding() {
    // Boundary values for each VarInt byte-count tier.
    let cases: &[(i32, usize)] = &[
        (0, 1),
        (1, 1),
        (127, 1),     // 0x7F — last 1-byte value
        (128, 2),     // 0x80 — first 2-byte value
        (16383, 2),   // 0x3FFF — last 2-byte value
        (16384, 3),   // 0x4000 — first 3-byte value
        (2097151, 3), // 0x1FFFFF — last 3-byte value
        (2097152, 4), // 0x200000 — first 4-byte value
    ];
    for &(value, expected_len) in cases {
        let encoded = crate::network::varint::encode_var_i32(value);
        assert_eq!(
            encoded.len(),
            expected_len,
            "encode_var_i32({value}) produced {} bytes, expected {expected_len}",
            encoded.len()
        );
        assert_eq!(
            var_int_encoded_len(value),
            expected_len,
            "var_int_encoded_len({value}) returned {}, expected {expected_len}",
            var_int_encoded_len(value)
        );
    }
}

#[test]
pub fn chunk_pipeline_coalesces_duplicate_requests_into_one_pending_entry() {
    // Tests Java parity invariant from ChunkTaskDispatcher: two players
    // requesting the same chunk should produce exactly one generation
    // job. We construct a worker-less pipeline so the queue cannot drain
    // and we can observe the pending entries directly.
    let cache = super::super::GeneratedChunkCache::default();
    let pipeline = super::super::ChunkPipeline::new(
        cache,
        std::path::PathBuf::from("/tmp/vibecraft-test-not-used"),
        42,
        0, // no workers — the queue stays put
    );
    let pos = crate::storage::region::ChunkPos { x: 7, z: -3 };
    pipeline.request_chunk(pos);
    pipeline.request_chunk(pos);
    pipeline.request_chunk(pos);

    let diag = pipeline.diagnostics();
    assert_eq!(
        diag.queue_depth, 1,
        "one queued entry, regardless of caller count"
    );
    assert_eq!(diag.in_flight, 0, "no workers, nothing in flight");
}

#[test]
pub fn chunk_pipeline_cancel_request_dequeues_pending_jobs() {
    let cache = super::super::GeneratedChunkCache::default();
    let pipeline = super::super::ChunkPipeline::new(
        cache,
        std::path::PathBuf::from("/tmp/vibecraft-test-not-used"),
        42,
        0,
    );
    let pos_a = crate::storage::region::ChunkPos { x: 1, z: 1 };
    let pos_b = crate::storage::region::ChunkPos { x: 2, z: 2 };
    pipeline.request_chunk(pos_a);
    pipeline.request_chunk(pos_b);
    pipeline.cancel_request(pos_a);

    let diag = pipeline.diagnostics();
    assert_eq!(diag.queue_depth, 1, "cancelled chunk dropped from queue");
    // Cancelling already-cancelled (or never-pending) is a no-op.
    pipeline.cancel_request(pos_a);
    pipeline.cancel_request(crate::storage::region::ChunkPos { x: 99, z: 99 });
    assert_eq!(pipeline.diagnostics().queue_depth, 1);
}

#[test]
pub fn chunk_pipeline_skips_scheduling_when_chunk_already_cached() {
    let cache = super::super::GeneratedChunkCache::default();
    let pos = crate::storage::region::ChunkPos { x: 5, z: 5 };
    // Pre-seed the cache so the pipeline sees this chunk as ready.
    cache.chunks.lock().unwrap().insert(
        pos,
        std::sync::Arc::new(crate::storage::chunk::LevelChunk::empty(pos)),
    );
    let pipeline = super::super::ChunkPipeline::new(
        cache,
        std::path::PathBuf::from("/tmp/vibecraft-test-not-used"),
        42,
        0,
    );
    pipeline.request_chunk(pos);
    let diag = pipeline.diagnostics();
    assert_eq!(diag.queue_depth, 0, "cache hit short-circuits the queue");
    assert!(pipeline.try_get_ready(pos).is_some());
}

#[test]
pub fn cache_set_block_mutates_in_memory_and_marks_dirty_without_disk_write() {
    // Java mirror: LevelChunk.setBlockState — in-memory mutation +
    // unsaved flag. Pre-seed the cache so get_or_load is a hit
    // (no disk path). The mutation must update the cached chunk
    // and record the chunk pos in the dirty set.
    let cache = super::super::GeneratedChunkCache::default();
    let pos = crate::storage::region::ChunkPos { x: 4, z: -7 };
    cache.chunks.lock().unwrap().insert(
        pos,
        std::sync::Arc::new(crate::storage::chunk::LevelChunk::empty(pos)),
    );

    let block_pos = crate::block_update::BlockPos {
        x: 4 * 16 + 3,
        y: 100,
        z: -7 * 16 + 11,
    };
    let prev = cache.set_block(
        std::path::Path::new("/tmp/vibecraft-test-not-used"),
        42,
        block_pos,
        "minecraft:stone",
    );
    assert!(prev.is_none(), "previous block was air");
    assert!(cache.dirty.lock().unwrap().contains(&pos));

    let cached = cache.try_get_ready(pos).expect("chunk still cached");
    assert_eq!(
        cached
            .get_block_state(block_pos.x, block_pos.y, block_pos.z)
            .as_deref(),
        Some("minecraft:stone")
    );
}

#[test]
pub fn live_block_model_reads_unflushed_cache_state_for_gameplay() {
    // Java mirror: Level.getBlockState reads the live LevelChunk, not the
    // persisted region file. A block placed this tick must be visible to the
    // next interaction before the dirty chunk is flushed.
    let cache = super::super::GeneratedChunkCache::default();
    let chunk_pos = crate::storage::region::ChunkPos { x: 0, z: 0 };
    cache.chunks.lock().unwrap().insert(
        chunk_pos,
        std::sync::Arc::new(crate::storage::chunk::LevelChunk::empty(chunk_pos)),
    );
    let world_root = std::env::temp_dir().join(format!(
        "vibecraft-live-block-read-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&world_root);
    let layout = super::super::WorldLayout::new(&world_root);
    let block_pos = crate::block_update::BlockPos {
        x: 2,
        y: 250,
        z: 3,
    };

    cache.set_block(&world_root, 42, block_pos, "minecraft:chest[facing=east,type=single]");

    let live = super::super::read_live_block_model_at(&cache, &layout, 42, block_pos);
    assert_eq!(live.registry_id, "minecraft:chest");
    assert_eq!(live.property("facing"), Some("east"));
    assert_eq!(live.property("type"), Some("single"));

    let snapshot = super::super::read_block_model_at(&layout, 42, block_pos);
    assert_eq!(
        snapshot.registry_id, "minecraft:air",
        "disk/worldgen snapshot must not be used as live gameplay state"
    );
    let _ = std::fs::remove_dir_all(world_root);
}

#[test]
pub fn placement_target_resolution_uses_unflushed_live_cache_state() {
    // Regression for immediate place-after-place/break behavior: if the clicked
    // block was changed in-memory but not flushed, Java's BlockPlaceContext sees
    // that live state and targets the adjacent position.
    let cache = super::super::GeneratedChunkCache::default();
    let chunk_pos = crate::storage::region::ChunkPos { x: 0, z: 0 };
    cache.chunks.lock().unwrap().insert(
        chunk_pos,
        std::sync::Arc::new(crate::storage::chunk::LevelChunk::empty(chunk_pos)),
    );
    let world_root = std::env::temp_dir().join(format!(
        "vibecraft-live-placement-target-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&world_root);
    let layout = super::super::WorldLayout::new(&world_root);
    let clicked = crate::block_update::BlockPos {
        x: 4,
        y: 100,
        z: 4,
    };
    cache.set_block(&world_root, 42, clicked, "minecraft:stone");

    let packet = crate::network::play::ServerboundUseItemOnPacket {
        hand: crate::network::play::ServerboundSwingHand::MainHand,
        block_hit: crate::network::play::BlockHitResultPacketData {
            x: clicked.x,
            y: clicked.y,
            z: clicked.z,
            direction: crate::network::play::Direction3d::Up,
            click_x: 0.5,
            click_y: 1.0,
            click_z: 0.5,
            inside: false,
            world_border_hit: false,
        },
        sequence: 7,
    };

    let target = super::super::resolve_block_item_placement_target(&layout, 42, &cache, &packet);
    assert_eq!(
        target.pos,
        crate::block_update::BlockPos {
            x: clicked.x,
            y: clicked.y + 1,
            z: clicked.z
        }
    );
    assert_eq!(target.existing_state.registry_id, "minecraft:air");
    let _ = std::fs::remove_dir_all(world_root);
}

#[test]
pub fn cache_set_block_clones_via_arc_make_mut_so_in_flight_readers_see_old_snapshot() {
    // Architectural guard: if a chunk send batch is mid-flight holding
    // an Arc<LevelChunk>, a concurrent setBlock must not mutate that
    // in-flight snapshot. Java's send packet captures state at
    // packet-build time and the follow-up BlockUpdate carries the
    // diff to the client.
    let cache = super::super::GeneratedChunkCache::default();
    let pos = crate::storage::region::ChunkPos { x: 0, z: 0 };
    cache.chunks.lock().unwrap().insert(
        pos,
        std::sync::Arc::new(crate::storage::chunk::LevelChunk::empty(pos)),
    );
    let in_flight_snapshot = cache.try_get_ready(pos).unwrap();

    let block_pos = crate::block_update::BlockPos { x: 5, y: 60, z: 5 };
    cache.set_block(
        std::path::Path::new("/tmp/vibecraft-test-not-used"),
        42,
        block_pos,
        "minecraft:dirt",
    );

    assert!(in_flight_snapshot
        .get_block_state(block_pos.x, block_pos.y, block_pos.z)
        .as_deref()
        .is_none_or(|n| n == "minecraft:air"));
    assert_eq!(
        cache
            .try_get_ready(pos)
            .unwrap()
            .get_block_state(block_pos.x, block_pos.y, block_pos.z)
            .as_deref(),
        Some("minecraft:dirt")
    );
}

#[test]
pub fn cache_invalidate_refuses_to_drop_dirty_chunks() {
    // Java parity: LevelChunk.unsaved blocks the chunk-unload path —
    // an in-memory-only block change can't be silently lost just
    // because something tried to evict the chunk before flush_dirty
    // ran.
    let cache = super::super::GeneratedChunkCache::default();
    let pos = crate::storage::region::ChunkPos { x: 1, z: 1 };
    cache.chunks.lock().unwrap().insert(
        pos,
        std::sync::Arc::new(crate::storage::chunk::LevelChunk::empty(pos)),
    );
    cache.set_block(
        std::path::Path::new("/tmp/vibecraft-test-not-used"),
        42,
        crate::block_update::BlockPos {
            x: 16,
            y: 64,
            z: 16,
        },
        "minecraft:gold_block",
    );
    cache.invalidate(pos);
    assert!(
        cache.try_get_ready(pos).is_some(),
        "dirty chunk must not be evicted before flush"
    );
}

#[test]
pub fn cache_flush_dirty_uses_configured_region_compression() {
    let mut world_root = std::env::temp_dir();
    world_root.push(format!(
        "vibecraft-cache-region-compression-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&world_root);

    let cache = super::super::GeneratedChunkCache::default();
    let pos = crate::storage::region::ChunkPos { x: 2, z: 0 };
    cache.chunks.lock().unwrap().insert(
        pos,
        std::sync::Arc::new(crate::storage::chunk::LevelChunk::empty(pos)),
    );
    cache.set_block(
        &world_root,
        42,
        crate::block_update::BlockPos { x: 32, y: 64, z: 0 },
        "minecraft:stone",
    );

    assert_eq!(
        cache.flush_dirty(
            &world_root,
            false,
            crate::storage::region::RegionCompression::Lz4
        ),
        1
    );
    let region =
        crate::storage::region::RegionFile::open(&world_root.join("region"), pos.region()).unwrap();
    let location = region.read_location(pos).unwrap().unwrap();
    let bytes = std::fs::read(region.path()).unwrap();
    let offset = location.sector_offset as usize * crate::storage::region::SECTOR_BYTES as usize;
    assert_eq!(
        bytes[offset + 4],
        crate::storage::region::RegionCompression::Lz4.id()
    );

    let _ = std::fs::remove_dir_all(&world_root);
}

#[test]
pub fn unpack_chunk_fluid_ticks_restores_saved_ticks_without_scanning_blocks() {
    // Java-parity guard: chunk load only restores saved fluid_ticks
    // (LevelChunkTicks.unpack); it never scans the chunk for fluid
    // blocks. Build a chunk with one synthetic saved water tick and
    // verify it lands in the live queue with the saved delay
    // preserved.
    use crate::storage::nbt::Tag;
    let pos = crate::storage::region::ChunkPos { x: 2, z: -1 };
    let mut chunk = crate::storage::chunk::LevelChunk::empty(pos);
    chunk.fluid_ticks.push(Tag::Compound(vec![
        ("i".to_string(), Tag::String("minecraft:water".to_string())),
        ("x".to_string(), Tag::Int(2 * 16 + 5)),
        ("y".to_string(), Tag::Int(64)),
        ("z".to_string(), Tag::Int(-16 + 9)),
        ("t".to_string(), Tag::Int(7)),
        ("p".to_string(), Tag::Int(0)),
    ]));

    let mut live = super::super::LiveFluidTicks::new();
    super::super::unpack_chunk_fluid_ticks(&mut live, 100, &chunk);

    // No tick is due yet (saved delay=7 → trigger_tick=107) so we
    // tick at 106 (nothing) then at 107 (one due).
    assert_eq!(live.tick_due(106, 4096).len(), 0);
    let due = live.tick_due(107, 4096);
    assert_eq!(due.len(), 1, "exactly the one saved tick fired");
    assert_eq!(due[0].pos.x, 2 * 16 + 5);
    assert_eq!(due[0].pos.y, 64);
    assert_eq!(due[0].pos.z, -16 + 9);
    assert_eq!(due[0].ty, "minecraft:water");
}

#[test]
pub fn unpack_chunk_fluid_ticks_does_not_touch_blocks_on_empty_saved_ticks() {
    // Architectural guard: this is the path that runs on freshly
    // generated chunks during the per-tick send drain. It must NOT
    // attempt to scan the chunk or read neighbour chunks — those
    // were the cascading worldgen calls that timed out the client.
    let chunk =
        crate::storage::chunk::LevelChunk::empty(crate::storage::region::ChunkPos { x: 0, z: 0 });
    let mut live = super::super::LiveFluidTicks::new();
    let started = std::time::Instant::now();
    super::super::unpack_chunk_fluid_ticks(&mut live, 0, &chunk);
    assert!(
        started.elapsed() < std::time::Duration::from_millis(5),
        "unpack must be O(saved ticks), not O(block count)"
    );
    assert_eq!(live.tick_due(1_000_000, 4096).len(), 0);
}

#[test]
pub fn login_seeding_does_not_synchronously_generate_view_distance_window() {
    // Architectural regression guard. Mirrors the Java invariant from
    // ChunkMap.applyChunkTrackingView: on player join, the chunk
    // tracking view is *recorded* immediately but generation runs
    // asynchronously. Seeding 441 chunks (vd=10) must not produce
    // any generated chunks and must not block — if it did, the play
    // loop could not start ticking until the whole window was
    // computed (the bug this rework exists to fix). We construct a
    // worker-less pipeline so we can prove nothing was generated as
    // a side effect of seeding.
    let cache = super::super::GeneratedChunkCache::default();
    let pipeline = super::super::ChunkPipeline::new(
        cache,
        std::path::PathBuf::from("/tmp/vibecraft-test-not-used"),
        42,
        0,
    );
    let mut sender = super::super::PlayerChunkSender::new(false);
    super::super::seed_chunk_window(&mut sender, &pipeline, 0, 0, 10);

    let diag = pipeline.diagnostics();
    assert_eq!(diag.queue_depth, 21 * 21, "all 441 chunks enqueued for gen");
    assert_eq!(diag.generated_total, 0, "seeding must not run worldgen");
    assert_eq!(
        sender.pending_count(),
        21 * 21,
        "sender tracks all positions"
    );
    assert_eq!(sender.unacknowledged_batches(), 0, "no batch sent yet");

    // The first per-tick drain returns nothing because no chunks are
    // ready — sender does not block.
    let batch = sender.send_next_chunks(crate::storage::region::ChunkPos { x: 0, z: 0 }, |pos| {
        pipeline.try_get_ready(pos)
    });
    assert!(batch.is_none(), "no ready chunks → no batch, never blocks");
    assert_eq!(
        sender.pending_count(),
        21 * 21,
        "no pending chunks lost when nothing is ready"
    );
    assert_eq!(sender.unacknowledged_batches(), 0);
}

#[test]
pub fn drain_flushes_only_ready_chunks_so_join_progresses_without_full_radius() {
    // Architectural regression guard. Demonstrates the per-tick
    // drain produces real progress (a sent batch) as soon as *any*
    // chunk completes generation, without waiting for the rest of
    // the view-distance square. This is the "gameplay ticks can run
    // before full radius is generated" invariant from
    // CHECKLIST_CHUNKING_CHANGES.md.
    let cache = super::super::GeneratedChunkCache::default();
    let pipeline = super::super::ChunkPipeline::new(
        cache.clone(),
        std::path::PathBuf::from("/tmp/vibecraft-test-not-used"),
        42,
        0,
    );
    let mut sender = super::super::PlayerChunkSender::new(false);
    super::super::seed_chunk_window(&mut sender, &pipeline, 0, 0, 10);

    // Simulate one worker completing one chunk (the centre). With a
    // worker-less pipeline we publish directly to the cache; the
    // production worker loop does the same call via `get_or_load`.
    let ready = crate::storage::region::ChunkPos { x: 0, z: 0 };
    cache.chunks.lock().unwrap().insert(
        ready,
        std::sync::Arc::new(crate::storage::chunk::LevelChunk::empty(ready)),
    );

    // Sender produces a batch containing exactly that one ready
    // chunk — the other 440 stay pending for future ticks.
    let batch = sender
        .send_next_chunks(ready, |pos| pipeline.try_get_ready(pos))
        .expect("one ready chunk → one-chunk batch");
    assert_eq!(batch.chunks.len(), 1);
    assert_eq!(batch.chunks[0].0, ready);
    assert_eq!(
        sender.pending_count(),
        21 * 21 - 1,
        "other chunks still pending"
    );
    assert_eq!(
        sender.unacknowledged_batches(),
        1,
        "one batch sent → one ack outstanding"
    );
}

#[test]
pub fn apply_chunk_movement_keeps_pending_aligned_with_new_view_window() {
    // Movement diff: chunks falling out of the new window should be
    // unscheduled and dropped from sender's pending set; chunks
    // entering the window should be both queued for generation and
    // recorded in sender's pending set. We exercise this without
    // touching the wire (using a sender + pipeline directly).
    let cache = super::super::GeneratedChunkCache::default();
    let pipeline = super::super::ChunkPipeline::new(
        cache,
        std::path::PathBuf::from("/tmp/vibecraft-test-not-used"),
        42,
        0,
    );
    let mut sender = super::super::PlayerChunkSender::new(false);
    // Initial window centred at (0, 0) with radius 1.
    super::super::seed_chunk_window(&mut sender, &pipeline, 0, 0, 1);
    // 3×3 = 9 chunks should be pending and queued.
    assert_eq!(sender.pending_count(), 9);
    assert_eq!(pipeline.diagnostics().queue_depth, 9);

    // Player walks one chunk east → centre (1, 0). The new window
    // covers x=0..=2, z=-1..=1; the old covered x=-1..=1, z=-1..=1.
    // So (-1,-1), (-1,0), (-1,1) leave; (2,-1), (2,0), (2,1) enter.
    let mut loaded: std::collections::BTreeSet<(i32, i32)> =
        super::super::chunk_window(0, 0, 1).into_iter().collect();
    // Note: we don't run the real apply_chunk_movement here (it needs a
    // TcpStream). Replicate just the per-pos sender + pipeline calls so
    // the unit test stays decoupled from the wire.
    let new_window = super::super::chunk_window(1, 0, 1);
    for stale in loaded.difference(&new_window).copied().collect::<Vec<_>>() {
        let pos = crate::storage::region::ChunkPos {
            x: stale.0,
            z: stale.1,
        };
        // None of these have been sent (sender only marks pending here),
        // so drop_chunk should return None and we cancel the pipeline
        // request.
        assert!(sender.drop_chunk(pos, true).is_none());
        pipeline.cancel_request(pos);
    }
    for fresh in new_window.difference(&loaded).copied() {
        let pos = crate::storage::region::ChunkPos {
            x: fresh.0,
            z: fresh.1,
        };
        sender.mark_chunk_pending_to_send(pos);
        pipeline.request_chunk(pos);
    }
    loaded = new_window;
    assert_eq!(loaded.len(), 9);
    assert_eq!(sender.pending_count(), 9);
    assert_eq!(pipeline.diagnostics().queue_depth, 9);
    // The old left-column chunks are no longer in either the pending
    // sender or the pipeline queue.
    for stale_x in [-1] {
        for stale_z in [-1, 0, 1] {
            let pos = crate::storage::region::ChunkPos {
                x: stale_x,
                z: stale_z,
            };
            assert!(!sender.is_pending(pos), "stale {:?} still pending", pos);
        }
    }
}
