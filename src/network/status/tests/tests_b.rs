use super::super::*;
use super::*;

#[test]
pub fn live_login_writer_reuses_vanilla_common_spawn_codec() {
    let login = ClientboundLoginPacket {
        player_id: 42,
        hardcore: true,
        levels: BTreeSet::from([
            Identifier::parse("minecraft:overworld").unwrap(),
            Identifier::parse("minecraft:the_nether").unwrap(),
        ]),
        max_players: 20,
        chunk_radius: 10,
        simulation_distance: 8,
        reduced_debug_info: false,
        show_death_screen: true,
        do_limited_crafting: false,
        spawn_info: CommonPlayerSpawnInfo {
            dimension_type: Identifier::parse("minecraft:the_nether").unwrap(),
            dimension: Identifier::parse("minecraft:the_nether").unwrap(),
            seed: -7,
            game_mode: GameMode::Creative,
            previous_game_mode: Some(GameMode::Survival),
            is_debug: false,
            is_flat: true,
            last_death_location: Some((
                Identifier::parse("minecraft:overworld").unwrap(),
                [1, 64, -2],
            )),
            portal_cooldown: 20,
            sea_level: 32,
        },
        enforces_secure_chat: true,
    };

    let mut live_payload = Vec::new();
    write_clientbound_login_packet(&mut live_payload, &login).unwrap();
    let mut packet_payload = Vec::new();
    login.write(&mut packet_payload).unwrap();
    assert_eq!(live_payload, packet_payload);

    let packed_position = pack_block_position(1, 64, -2).to_be_bytes();
    assert!(live_payload
        .windows(packed_position.len())
        .any(|window| window == packed_position));

    let fixed_int_position = [
        1i32.to_be_bytes(),
        64i32.to_be_bytes(),
        (-2i32).to_be_bytes(),
    ]
    .concat();
    assert!(!live_payload
        .windows(fixed_int_position.len())
        .any(|window| window == fixed_int_position));
}

#[test]
pub fn inventory_menu_full_sync_writes_all_slots_and_carried_item() {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let handle = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let mut state = session_state_with_inventory(&[("minecraft:oak_log", 2, 0)]);
        state.carried_item = ItemStack::new("minecraft:oak_planks", 4);
        write_inventory_menu_full_sync(&mut stream, CompressionState::disabled(), &state).unwrap();
    });
    let mut client = std::net::TcpStream::connect(addr).unwrap();
    let frame = read_packet(&mut client).unwrap();
    let mut payload = &frame[..];
    assert_eq!(
        read_var_i32(&mut payload).unwrap(),
        crate::network::play::CLIENTBOUND_CONTAINER_SET_CONTENT_PACKET_ID
    );
    assert_eq!(read_var_i32(&mut payload).unwrap(), 0);
    assert_eq!(read_var_i32(&mut payload).unwrap(), 0);
    let slot_count = read_var_i32(&mut payload).unwrap();
    assert_eq!(slot_count, 46);
    let mut slots = Vec::new();
    for _ in 0..slot_count {
        slots.push(
            crate::network::play::RawItemStack::read_optional_untrusted(&mut payload).unwrap(),
        );
    }
    assert_eq!(slots[36].count, 2);
    assert_eq!(
        slots[36].item_id,
        crate::item_catalog::item_protocol_id("minecraft:oak_log")
    );
    let carried =
        crate::network::play::RawItemStack::read_optional_untrusted(&mut payload).unwrap();
    assert_eq!(carried.count, 4);
    assert_eq!(
        carried.item_id,
        crate::item_catalog::item_protocol_id("minecraft:oak_planks")
    );
    handle.join().unwrap();
}

#[test]
pub fn generated_chunk_entity_add_packets_reads_queued_chunk_mob_nbt() {
    let mut chunk = LevelChunk::empty(ChunkPos { x: 2, z: -3 });
    chunk.entities.push(Tag::Compound(vec![
        ("id".to_string(), Tag::String("minecraft:pig".to_string())),
        (
            "UUID".to_string(),
            Tag::String("00000000-0000-4000-8000-000000000123".to_string()),
        ),
        (
            "Pos".to_string(),
            Tag::List(vec![
                Tag::Double(32.9),
                Tag::Double(70.0),
                Tag::Double(-33.0),
            ]),
        ),
        (
            "Rotation".to_string(),
            Tag::List(vec![Tag::Float(90.0), Tag::Float(15.0)]),
        ),
    ]));

    let packets = super::super::generated_chunk_entity_add_packets(&chunk);

    assert_eq!(packets.len(), 1);
    let packet = &packets[0];
    assert_eq!(
        packet.id,
        super::super::generated_chunk_entity_runtime_id(chunk.pos, 0)
    );
    assert_eq!(
        packet.uuid,
        Uuid([0, 0, 0, 0, 0, 0, 64, 0, 128, 0, 0, 0, 0, 0, 1, 35])
    );
    assert_eq!(packet.entity_type, 100);
    assert_eq!(packet.position.x, 32.9);
    assert_eq!(packet.position.y, 70.0);
    assert_eq!(packet.position.z, -33.0);
    assert_eq!(packet.movement, Vec3::ZERO);
    assert_eq!(packet.x_rot, 10);
    assert_eq!(packet.y_rot, 64);
    assert_eq!(packet.y_head_rot, 64);
}

#[test]
pub fn generated_chunk_entity_spawn_packets_are_bundle_wrapped() {
    let plan = super::super::GeneratedChunkEntitySpawnPlan {
        add_entity: ClientboundAddEntityPacket::new(AddEntityPacketInput {
            id: 42,
            uuid: Uuid([1; 16]),
            entity_type: 100,
            position: Vec3 {
                x: 1.0,
                y: 65.0,
                z: 2.0,
            },
            movement: Vec3::ZERO,
            rotation: (0.0, 90.0),
            y_head_rot: 90.0,
            data: 0,
        }),
        metadata: None,
    };
    let mut output = Vec::new();

    super::super::write_generated_chunk_entity_spawn_packets(
        &mut output,
        CompressionState::disabled(),
        &plan,
    )
    .expect("generated mob pairing should serialize");

    let mut frames = Vec::new();
    let mut input = &output[..];
    while !input.is_empty() {
        let frame_len = read_var_i32(&mut input).unwrap() as usize;
        let mut frame = vec![0; frame_len];
        input.read_exact(&mut frame).unwrap();
        let mut payload = &frame[..];
        frames.push(read_var_i32(&mut payload).unwrap());
    }
    assert_eq!(
        frames,
        vec![
            CLIENTBOUND_BUNDLE_DELIMITER_PACKET_ID,
            CLIENTBOUND_ADD_ENTITY_PACKET_ID,
            CLIENTBOUND_BUNDLE_DELIMITER_PACKET_ID
        ]
    );
}

#[test]
pub fn generated_chunk_entity_spawn_plan_reads_non_default_pig_variant_metadata() {
    let mut chunk = LevelChunk::empty(ChunkPos { x: 2, z: -3 });
    chunk.entities.push(Tag::Compound(vec![
        ("id".to_string(), Tag::String("minecraft:pig".to_string())),
        (
            "UUID".to_string(),
            Tag::String("00000000-0000-4000-8000-000000000123".to_string()),
        ),
        (
            "Pos".to_string(),
            Tag::List(vec![
                Tag::Double(32.9),
                Tag::Double(70.0),
                Tag::Double(-33.0),
            ]),
        ),
        (
            "Rotation".to_string(),
            Tag::List(vec![Tag::Float(90.0), Tag::Float(0.0)]),
        ),
        (
            "variant".to_string(),
            Tag::String("minecraft:warm".to_string()),
        ),
        (
            "sound_variant".to_string(),
            Tag::String("minecraft:mini".to_string()),
        ),
    ]));

    let plans = super::super::generated_chunk_entity_spawn_plans(&chunk);

    assert_eq!(plans.len(), 1);
    let metadata = plans[0]
        .metadata
        .as_ref()
        .expect("non-default pig variant data should emit metadata");
    assert_eq!(metadata.id, plans[0].add_entity.id);
    assert_eq!(
        metadata.packed_items,
        vec![
            EntityDataValue::typed(19, EntityMetadataValue::PigVariant(2)).unwrap(),
            EntityDataValue::typed(20, EntityMetadataValue::PigSoundVariant(2)).unwrap(),
        ]
    );
}

#[test]
pub fn generated_chunk_entity_spawn_plan_reads_non_default_chicken_variant_metadata() {
    let mut chunk = LevelChunk::empty(ChunkPos { x: 2, z: -3 });
    chunk.entities.push(Tag::Compound(vec![
        (
            "id".to_string(),
            Tag::String("minecraft:chicken".to_string()),
        ),
        (
            "UUID".to_string(),
            Tag::String("00000000-0000-4000-8000-000000000124".to_string()),
        ),
        (
            "Pos".to_string(),
            Tag::List(vec![
                Tag::Double(32.9),
                Tag::Double(70.0),
                Tag::Double(-33.0),
            ]),
        ),
        (
            "Rotation".to_string(),
            Tag::List(vec![Tag::Float(90.0), Tag::Float(0.0)]),
        ),
        (
            "variant".to_string(),
            Tag::String("minecraft:cold".to_string()),
        ),
        (
            "sound_variant".to_string(),
            Tag::String("minecraft:picky".to_string()),
        ),
    ]));

    let plans = super::super::generated_chunk_entity_spawn_plans(&chunk);

    assert_eq!(plans.len(), 1);
    let metadata = plans[0]
        .metadata
        .as_ref()
        .expect("non-default chicken variant data should emit metadata");
    assert_eq!(
        metadata.packed_items,
        vec![
            EntityDataValue::typed(18, EntityMetadataValue::ChickenVariant(0)).unwrap(),
            EntityDataValue::typed(19, EntityMetadataValue::ChickenSoundVariant(1)).unwrap(),
        ]
    );
}

#[test]
pub fn generated_chunk_entity_spawn_plan_reads_non_default_zombie_nautilus_variant_metadata() {
    let mut chunk = LevelChunk::empty(ChunkPos { x: 2, z: -3 });
    chunk.entities.push(Tag::Compound(vec![
        (
            "id".to_string(),
            Tag::String("minecraft:zombie_nautilus".to_string()),
        ),
        (
            "UUID".to_string(),
            Tag::String("00000000-0000-4000-8000-000000000126".to_string()),
        ),
        (
            "Pos".to_string(),
            Tag::List(vec![
                Tag::Double(32.9),
                Tag::Double(62.0),
                Tag::Double(-33.0),
            ]),
        ),
        (
            "Rotation".to_string(),
            Tag::List(vec![Tag::Float(90.0), Tag::Float(0.0)]),
        ),
        (
            "variant".to_string(),
            Tag::String("minecraft:warm".to_string()),
        ),
    ]));

    let plans = super::super::generated_chunk_entity_spawn_plans(&chunk);

    assert_eq!(plans.len(), 1);
    let metadata = plans[0]
        .metadata
        .as_ref()
        .expect("non-default zombie nautilus variant data should emit metadata");
    assert_eq!(
        metadata.packed_items,
        vec![EntityDataValue::typed(21, EntityMetadataValue::ZombieNautilusVariant(1)).unwrap()]
    );
}

#[test]
pub fn generated_chunk_entity_spawn_plan_omits_default_animal_variant_metadata() {
    let mut chunk = LevelChunk::empty(ChunkPos { x: 2, z: -3 });
    chunk.entities.push(Tag::Compound(vec![
        (
            "id".to_string(),
            Tag::String("minecraft:chicken".to_string()),
        ),
        (
            "UUID".to_string(),
            Tag::String("00000000-0000-4000-8000-000000000125".to_string()),
        ),
        (
            "Pos".to_string(),
            Tag::List(vec![
                Tag::Double(32.9),
                Tag::Double(70.0),
                Tag::Double(-33.0),
            ]),
        ),
        (
            "Rotation".to_string(),
            Tag::List(vec![Tag::Float(90.0), Tag::Float(0.0)]),
        ),
        (
            "variant".to_string(),
            Tag::String("minecraft:temperate".to_string()),
        ),
        (
            "sound_variant".to_string(),
            Tag::String("minecraft:classic".to_string()),
        ),
    ]));

    let plans = super::super::generated_chunk_entity_spawn_plans(&chunk);

    assert_eq!(plans.len(), 1);
    assert_eq!(plans[0].metadata, None);
}

#[test]
pub fn generated_chunk_entity_spawn_packets_include_non_default_metadata_in_bundle() {
    let plan = super::super::GeneratedChunkEntitySpawnPlan {
        add_entity: ClientboundAddEntityPacket::new(AddEntityPacketInput {
            id: 42,
            uuid: Uuid([1; 16]),
            entity_type: 100,
            position: Vec3 {
                x: 1.0,
                y: 65.0,
                z: 2.0,
            },
            movement: Vec3::ZERO,
            rotation: (0.0, 90.0),
            y_head_rot: 90.0,
            data: 0,
        }),
        metadata: Some(ClientboundSetEntityDataPacket {
            id: 42,
            packed_items: vec![
                EntityDataValue::typed(19, EntityMetadataValue::PigVariant(2)).unwrap(),
            ],
        }),
    };
    let mut output = Vec::new();

    super::super::write_generated_chunk_entity_spawn_packets(
        &mut output,
        CompressionState::disabled(),
        &plan,
    )
    .expect("generated mob pairing with metadata should serialize");

    let mut frames = Vec::new();
    let mut input = &output[..];
    while !input.is_empty() {
        let frame_len = read_var_i32(&mut input).unwrap() as usize;
        let mut frame = vec![0; frame_len];
        input.read_exact(&mut frame).unwrap();
        let mut payload = &frame[..];
        frames.push(read_var_i32(&mut payload).unwrap());
    }
    assert_eq!(
        frames,
        vec![
            CLIENTBOUND_BUNDLE_DELIMITER_PACKET_ID,
            CLIENTBOUND_ADD_ENTITY_PACKET_ID,
            CLIENTBOUND_SET_ENTITY_DATA_PACKET_ID,
            CLIENTBOUND_BUNDLE_DELIMITER_PACKET_ID
        ]
    );
}

#[test]
pub fn generated_chunk_entity_remove_packets_use_deterministic_runtime_ids() {
    let mut chunk = LevelChunk::empty(ChunkPos { x: 2, z: -3 });
    chunk.entities.push(Tag::Compound(vec![
        ("id".to_string(), Tag::String("minecraft:pig".to_string())),
        (
            "UUID".to_string(),
            Tag::String("00000000-0000-4000-8000-000000000123".to_string()),
        ),
        (
            "Pos".to_string(),
            Tag::List(vec![
                Tag::Double(32.9),
                Tag::Double(70.0),
                Tag::Double(-33.0),
            ]),
        ),
        (
            "Rotation".to_string(),
            Tag::List(vec![Tag::Float(90.0), Tag::Float(0.0)]),
        ),
    ]));
    let mut output = Vec::new();

    super::super::write_generated_chunk_entity_remove_packets(
        &mut output,
        CompressionState::disabled(),
        &chunk,
    )
    .expect("generated mob removal should serialize");

    let mut frame = &output[..];
    let frame_len = read_var_i32(&mut frame).unwrap() as usize;
    assert_eq!(frame_len, frame.len());
    assert_eq!(
        read_var_i32(&mut frame).unwrap(),
        CLIENTBOUND_REMOVE_ENTITIES_PACKET_ID
    );
    assert_eq!(read_var_i32(&mut frame).unwrap(), 1);
    assert_eq!(
        read_var_i32(&mut frame).unwrap(),
        super::super::generated_chunk_entity_runtime_id(chunk.pos, 0)
    );
    assert!(frame.is_empty());
}

#[test]
pub fn generated_mob_entity_type_network_ids_cover_modeled_biome_spawns() {
    let modeled_spawn_ids = [
        ("minecraft:armadillo", 4),
        ("minecraft:axolotl", 7),
        ("minecraft:bat", 10),
        ("minecraft:bogged", 16),
        ("minecraft:camel", 19),
        ("minecraft:chicken", 26),
        ("minecraft:cod", 27),
        ("minecraft:cow", 30),
        ("minecraft:creeper", 32),
        ("minecraft:dolphin", 35),
        ("minecraft:donkey", 36),
        ("minecraft:drowned", 38),
        ("minecraft:enderman", 41),
        ("minecraft:fox", 54),
        ("minecraft:frog", 55),
        ("minecraft:ghast", 57),
        ("minecraft:glow_squid", 61),
        ("minecraft:goat", 62),
        ("minecraft:hoglin", 64),
        ("minecraft:horse", 66),
        ("minecraft:husk", 67),
        ("minecraft:llama", 78),
        ("minecraft:magma_cube", 80),
        ("minecraft:mooshroom", 86),
        ("minecraft:mule", 87),
        ("minecraft:ocelot", 91),
        ("minecraft:panda", 96),
        ("minecraft:parched", 97),
        ("minecraft:parrot", 98),
        ("minecraft:pig", 100),
        ("minecraft:piglin", 101),
        ("minecraft:polar_bear", 104),
        ("minecraft:pufferfish", 107),
        ("minecraft:rabbit", 108),
        ("minecraft:salmon", 110),
        ("minecraft:sheep", 111),
        ("minecraft:skeleton", 115),
        ("minecraft:slime", 117),
        ("minecraft:spider", 124),
        ("minecraft:squid", 127),
        ("minecraft:stray", 128),
        ("minecraft:strider", 129),
        ("minecraft:trader_llama", 134),
        ("minecraft:tropical_fish", 136),
        ("minecraft:turtle", 137),
        ("minecraft:witch", 144),
        ("minecraft:wolf", 148),
        ("minecraft:zombie", 150),
        ("minecraft:zombie_horse", 151),
        ("minecraft:zombie_nautilus", 152),
        ("minecraft:zombie_villager", 153),
        ("minecraft:zombified_piglin", 154),
    ];

    for (entity_type, network_id) in modeled_spawn_ids {
        assert_eq!(
            super::super::generated_mob_entity_type_network_id(entity_type),
            Some(network_id),
            "{entity_type}"
        );
    }
}

#[test]
pub fn spawn_chunk_window_can_represent_configured_server_view_distance_radius() {
    assert_eq!(chunk_batch_size(2), 25);
    assert_eq!(chunk_batch_size(10), 441);

    let chunks = chunk_window(4, -3, 10);
    assert_eq!(chunks.len(), 441);
    assert!(chunks.contains(&(4, -3)));
    assert!(chunks.contains(&(-6, -13)));
    assert!(chunks.contains(&(14, 7)));
    assert!(!chunks.contains(&(-7, -3)));
    assert!(!chunks.contains(&(4, 8)));
}

#[test]
pub fn spawn_chunk_window_keeps_minimum_five_by_five_terrain_patch() {
    let chunks = chunk_window(4, -3, 2);
    assert_eq!(chunks.len(), 25);
    assert!(chunks.contains(&(4, -3)));
    assert!(chunks.contains(&(2, -5)));
    assert!(chunks.contains(&(6, -1)));
    assert!(!chunks.contains(&(1, -3)));
    assert!(!chunks.contains(&(4, 0)));
}

#[test]
pub fn movement_chunk_window_sends_only_newly_visible_edge_chunks() {
    let previous = chunk_window(0, 0, 10);
    let next = chunk_window(1, 0, 10);
    let delta = newly_visible_chunks(&previous, &next);

    assert_eq!(previous.len(), 441);
    assert_eq!(next.len(), 441);
    assert_eq!(delta.len(), 21);
    assert!(delta.iter().all(|chunk| chunk.0 == 11));
    assert!(delta.contains(&(11, -10)));
    assert!(delta.contains(&(11, 0)));
    assert!(delta.contains(&(11, 10)));
}

#[test]
pub fn chunk_batch_start_packet_has_no_payload_after_packet_id() {
    let mut packet = Vec::new();
    write_framed_packet(
        &mut packet,
        CLIENTBOUND_PLAY_CHUNK_BATCH_START_PACKET_ID,
        |_| Ok(()),
    )
    .unwrap();

    let mut cursor = Cursor::new(packet);
    let frame_len = read_var_i32(&mut cursor).unwrap();
    assert_eq!(frame_len, 1);
    assert_eq!(
        read_var_i32(&mut cursor).unwrap(),
        CLIENTBOUND_PLAY_CHUNK_BATCH_START_PACKET_ID
    );
    assert_eq!(cursor.position(), cursor.get_ref().len() as u64);
}

#[test]
pub fn forget_level_chunk_packet_uses_packed_chunk_position() {
    assert_eq!(CLIENTBOUND_FORGET_LEVEL_CHUNK_PACKET_ID, 37);
    assert_eq!(packed_chunk_pos(4, -2), -8589934588);
    assert_eq!(packed_chunk_pos(-1, 0), 0xffff_ffff);

    let mut packet = Vec::new();
    write_framed_packet(
        &mut packet,
        CLIENTBOUND_FORGET_LEVEL_CHUNK_PACKET_ID,
        |payload| payload.write_all(&packed_chunk_pos(4, -2).to_be_bytes()),
    )
    .unwrap();

    let mut cursor = Cursor::new(packet);
    let frame_len = read_var_i32(&mut cursor).unwrap();
    assert_eq!(frame_len, 9);
    assert_eq!(
        read_var_i32(&mut cursor).unwrap(),
        CLIENTBOUND_FORGET_LEVEL_CHUNK_PACKET_ID
    );
    let mut packed = [0; 8];
    cursor.read_exact(&mut packed).unwrap();
    assert_eq!(i64::from_be_bytes(packed), packed_chunk_pos(4, -2));
    assert_eq!(cursor.position(), cursor.get_ref().len() as u64);
}

pub fn palette_index_at(words: &[u64], x: usize, y: usize, z: usize) -> u64 {
    const BITS_PER_ENTRY: usize = 4;
    const VALUES_PER_LONG: usize = 64 / BITS_PER_ENTRY;
    let block_index = (y << 8) | (z << 4) | x;
    let word_index = block_index / VALUES_PER_LONG;
    let bit_index = (block_index - word_index * VALUES_PER_LONG) * BITS_PER_ENTRY;
    (words[word_index] >> bit_index) & 0xf
}

#[test]
pub fn jukebox_song_registry_payloads_include_disc_13_component_data() {
    assert_eq!(
        registry_element_count(write_vanilla_jukebox_song_registry_packet),
        21
    );
    let thirteen = JUKEBOX_SONGS
        .iter()
        .find(|song| song.id == "13")
        .expect("music disc 13 should be sent");
    let tag = jukebox_song_nbt(thirteen);

    assert!(matches!(
        field_value(&tag, "sound_event"),
        Some(Tag::String(value)) if value == "minecraft:music_disc.13"
    ));
    let description = compound_field(&tag, "description");
    assert!(matches!(
        field_value(description, "translate"),
        Some(Tag::String(value)) if value == "jukebox_song.minecraft.13"
    ));
    assert!(matches!(
        field_value(&tag, "length_in_seconds"),
        Some(Tag::Float(value)) if (*value - 178.0).abs() < f32::EPSILON
    ));
    assert!(matches!(
        field_value(&tag, "comparator_output"),
        Some(Tag::Int(1))
    ));
}

#[test]
pub fn synced_tag_registries_include_required_names_and_indices() {
    let is_fire = damage_type_tag_entries("minecraft:is_fire");
    assert_eq!(is_fire, &[21, 3, 31, 24, 20, 46, 14]);

    let bypasses_shield = damage_type_tag_entries("minecraft:bypasses_shield");
    assert!(bypasses_shield.contains(&11));
    assert!(bypasses_shield.contains(&13));

    let flower = banner_pattern_tag_entries("minecraft:pattern_item/flower");
    assert_eq!(flower, vec![banner_pattern_index("flower")]);

    let field_masoned = banner_pattern_tag_entries("minecraft:pattern_item/field_masoned");
    assert_eq!(field_masoned, vec![banner_pattern_index("bricks")]);

    let bordure_indented = banner_pattern_tag_entries("minecraft:pattern_item/bordure_indented");
    assert_eq!(bordure_indented, vec![banner_pattern_index("curly_border")]);
}

#[test]
pub fn configuration_wait_ignores_vanilla_common_packets_before_known_packs() {
    let mut input = Vec::new();
    write_framed_packet(
        &mut input,
        SERVERBOUND_CONFIGURATION_CLIENT_INFORMATION_PACKET_ID,
        |payload| {
            crate::network::codec::write_string(payload, "en_us", 16)?;
            payload.write_all(&[12, 0, 0, 0, 1, 1, 0, 0])?;
            write_var_i32(payload, 127)?;
            Ok(())
        },
    )
    .unwrap();
    write_framed_packet(
        &mut input,
        SERVERBOUND_CONFIGURATION_CUSTOM_PAYLOAD_PACKET_ID,
        |payload| {
            write_identifier(payload, &Identifier::parse("minecraft:brand").unwrap())?;
            crate::network::codec::write_string(payload, "vanilla", 32767)
        },
    )
    .unwrap();
    write_framed_packet(
        &mut input,
        SERVERBOUND_CONFIGURATION_SELECT_KNOWN_PACKS_PACKET_ID,
        |payload| {
            write_var_i32(payload, 1)?;
            crate::network::codec::write_string(payload, "minecraft", 32767)?;
            crate::network::codec::write_string(payload, "core", 32767)?;
            crate::network::codec::write_string(payload, VERSION_NAME, 32767)
        },
    )
    .unwrap();

    let mut stream = Cursor::new(input);
    wait_for_configuration_packet(
        &mut stream,
        CompressionState::disabled(),
        SERVERBOUND_CONFIGURATION_SELECT_KNOWN_PACKS_PACKET_ID,
        "selected known packs",
    )
    .unwrap();
}

#[test]
pub fn raw_play_command_packets_reject_before_login_hello() {
    let command_like_play_packets = [
        SERVERBOUND_COMMAND_SUGGESTION_PACKET_ID,
        SERVERBOUND_CHAT_PACKET_ID,
        SERVERBOUND_CHAT_COMMAND_PACKET_ID,
        SERVERBOUND_CHAT_COMMAND_SIGNED_PACKET_ID,
    ];

    for packet_id in command_like_play_packets {
        let mut input = Vec::new();
        write_framed_packet(&mut input, packet_id, |payload| {
            write_command_like_play_payload(packet_id, payload)
        })
        .unwrap();

        let mut rate_limiter =
            crate::network::rate_limit::PacketRateLimiter::new(0, std::time::Instant::now());
        let err = read_expected_login_hello_packet(&mut Cursor::new(input), &mut rate_limiter)
            .unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidData);
        assert_eq!(err.to_string(), "expected login hello");
    }
}

#[test]
pub fn raw_play_command_packets_reject_during_configuration_wait() {
    let command_like_play_packets = [
        SERVERBOUND_COMMAND_SUGGESTION_PACKET_ID,
        SERVERBOUND_CHAT_PACKET_ID,
        SERVERBOUND_CHAT_COMMAND_PACKET_ID,
        SERVERBOUND_CHAT_COMMAND_SIGNED_PACKET_ID,
    ];

    for packet_id in command_like_play_packets {
        let mut input = Vec::new();
        write_framed_packet(&mut input, packet_id, |payload| {
            write_command_like_play_payload(packet_id, payload)
        })
        .unwrap();

        let err = wait_for_configuration_packet(
            &mut Cursor::new(input),
            CompressionState::disabled(),
            SERVERBOUND_CONFIGURATION_SELECT_KNOWN_PACKS_PACKET_ID,
            "selected known packs",
        )
        .unwrap_err();
        assert!(
            matches!(
                err.kind(),
                io::ErrorKind::InvalidData | io::ErrorKind::UnexpectedEof
            ),
            "packet {packet_id} was not rejected at the configuration boundary: {err}"
        );
        if packet_id == SERVERBOUND_COMMAND_SUGGESTION_PACKET_ID {
            assert!(err
                .to_string()
                .contains(&format!("configuration packet {packet_id}")));
        }
    }
}

fn write_command_like_play_payload<W: Write>(packet_id: i32, payload: &mut W) -> io::Result<()> {
    match packet_id {
        SERVERBOUND_COMMAND_SUGGESTION_PACKET_ID => {
            crate::network::play::ServerboundCommandSuggestionPacket {
                id: 12,
                command: "/list".to_string(),
            }
            .write(payload)
        }
        SERVERBOUND_CHAT_PACKET_ID => crate::network::play::ServerboundChatPacket {
            message: "hello".to_string(),
            timestamp_epoch_millis: 0,
            salt: 0,
            signature: None,
            last_seen_messages: empty_last_seen_messages(),
        }
        .write(payload),
        SERVERBOUND_CHAT_COMMAND_PACKET_ID => crate::network::play::ServerboundChatCommandPacket {
            command: "list".to_string(),
        }
        .write(payload),
        SERVERBOUND_CHAT_COMMAND_SIGNED_PACKET_ID => {
            crate::network::play::ServerboundChatCommandSignedPacket {
                command: "list".to_string(),
                timestamp_epoch_millis: 0,
                salt: 0,
                argument_signatures: Vec::new(),
                last_seen_messages: empty_last_seen_messages(),
            }
            .write(payload)
        }
        _ => unreachable!("test only supplies command-like play packet ids"),
    }
}

fn empty_last_seen_messages() -> crate::network::play::LastSeenMessagesUpdate {
    crate::network::play::LastSeenMessagesUpdate {
        offset: 0,
        acknowledged: vec![0; crate::network::play::LastSeenMessagesUpdate::ACKNOWLEDGED_BYTES],
        checksum: 0,
    }
}

#[test]
pub fn configuration_wait_rejects_unexpected_packets() {
    let mut input = Vec::new();
    write_framed_packet(&mut input, 42, |_payload| Ok(())).unwrap();

    let err = wait_for_configuration_packet(
        &mut Cursor::new(input),
        CompressionState::disabled(),
        SERVERBOUND_CONFIGURATION_SELECT_KNOWN_PACKS_PACKET_ID,
        "selected known packs",
    )
    .unwrap_err();
    assert_eq!(err.kind(), io::ErrorKind::InvalidData);
    assert!(err.to_string().contains("configuration packet 42"));
}

#[test]
pub fn configuration_wait_honors_connection_rate_limit() {
    let now = std::time::Instant::now();
    let mut rate_limiter = crate::network::rate_limit::PacketRateLimiter::new(1, now);
    for _ in 0..8 {
        assert_eq!(
            rate_limiter.record_packet(now),
            crate::network::rate_limit::PacketRateDecision::Allow
        );
    }
    assert!(matches!(
        rate_limiter.tick(now + std::time::Duration::from_secs(1)),
        crate::network::rate_limit::PacketRateDecision::Kick { .. }
    ));

    let mut input = Vec::new();
    write_framed_packet(
        &mut input,
        SERVERBOUND_CONFIGURATION_SELECT_KNOWN_PACKS_PACKET_ID,
        |payload| write_var_i32(payload, 0),
    )
    .unwrap();

    let err = wait_for_configuration_packet_with_rate_limit(
        &mut Cursor::new(input),
        CompressionState::disabled(),
        SERVERBOUND_CONFIGURATION_SELECT_KNOWN_PACKS_PACKET_ID,
        "selected known packs",
        &mut rate_limiter,
        None,
    )
    .unwrap_err();
    assert_eq!(err.kind(), io::ErrorKind::PermissionDenied);
    assert_eq!(err.to_string(), "disconnect.exceeded_packet_rate");
}

pub fn assert_nested_sound_variant_fields(tag: Tag, fields: &[&str]) {
    let adult = compound_field(&tag, "adult_sounds");
    let baby = compound_field(&tag, "baby_sounds");
    assert_sound_set_fields(adult, fields);
    assert_sound_set_fields(baby, fields);
}

pub fn assert_sound_variant_fields(tag: Tag, fields: &[&str]) {
    assert_sound_set_fields(&tag, fields);
}

pub fn assert_sound_set_fields(tag: &Tag, fields: &[&str]) {
    for field in fields {
        assert!(
            matches!(field_value(tag, field), Some(Tag::String(value)) if value.starts_with("minecraft:")),
            "missing sound field {field} in {tag:?}"
        );
    }
}

pub fn assert_string_list(value: Option<&Tag>, expected: &[&str]) {
    let Some(Tag::List(values)) = value else {
        panic!("expected string list, got {value:?}");
    };
    let actual: Vec<&str> = values
        .iter()
        .map(|value| match value {
            Tag::String(value) => value.as_str(),
            value => panic!("expected string list value, got {value:?}"),
        })
        .collect();
    assert_eq!(actual, expected);
}

pub fn damage_type_tag_entries(tag: &str) -> &'static [i32] {
    DAMAGE_TYPE_TAGS
        .iter()
        .find_map(|(name, entries)| (*name == tag).then_some(*entries))
        .unwrap_or_else(|| panic!("missing damage type tag {tag}"))
}

pub fn banner_pattern_tag_entries(tag: &str) -> Vec<usize> {
    BANNER_PATTERN_TAGS
        .iter()
        .find_map(|(name, entries)| {
            (*name == tag).then(|| {
                entries
                    .iter()
                    .map(|entry| banner_pattern_index(entry))
                    .collect()
            })
        })
        .unwrap_or_else(|| panic!("missing banner pattern tag {tag}"))
}

pub fn banner_pattern_index(pattern: &str) -> usize {
    BANNER_PATTERNS
        .iter()
        .position(|entry| *entry == pattern)
        .unwrap_or_else(|| panic!("missing banner pattern {pattern}"))
}

pub fn compound_field<'a>(tag: &'a Tag, field: &str) -> &'a Tag {
    match field_value(tag, field) {
        Some(value @ Tag::Compound(_)) => value,
        value => panic!("expected compound field {field}, got {value:?}"),
    }
}

pub fn field_value<'a>(tag: &'a Tag, field: &str) -> Option<&'a Tag> {
    let Tag::Compound(fields) = tag else {
        return None;
    };
    fields
        .iter()
        .find_map(|(name, value)| (name == field).then_some(value))
}

pub fn registry_element_count(write_packet: fn(&mut Vec<u8>) -> std::io::Result<()>) -> i32 {
    let mut payload = Vec::new();
    write_packet(&mut payload).unwrap();
    let mut cursor = Cursor::new(payload);
    let _registry = crate::network::codec::read_identifier(&mut cursor).unwrap();
    read_var_i32(&mut cursor).unwrap()
}

pub fn status_registry_id(write_packet: fn(&mut Vec<u8>) -> io::Result<()>) -> String {
    let mut payload = Vec::new();
    write_packet(&mut payload).unwrap();
    let mut cursor = Cursor::new(payload);
    crate::network::codec::read_identifier(&mut cursor)
        .unwrap()
        .to_string()
}

pub fn status_registry_entry_ids_ordered(
    write_packet: fn(&mut Vec<u8>) -> std::io::Result<()>,
) -> Vec<String> {
    let mut payload = Vec::new();
    write_packet(&mut payload).unwrap();
    let mut cursor = Cursor::new(payload);
    let _registry = crate::network::codec::read_identifier(&mut cursor).unwrap();
    let entry_count = read_var_i32(&mut cursor).unwrap();
    let mut entry_ids = Vec::with_capacity(entry_count as usize);
    for _ in 0..entry_count {
        let id = crate::network::codec::read_identifier(&mut cursor).unwrap();
        entry_ids.push(id.to_string());

        let mut _present = [0u8; 1];
        cursor.read_exact(&mut _present).unwrap();
        let mut tag_id = [0u8; 1];
        cursor.read_exact(&mut tag_id).unwrap();
        let _ = Tag::read_payload(tag_id[0], &mut cursor).unwrap();
    }
    entry_ids
}

pub fn assert_registry_order(
    write_packet: fn(&mut Vec<u8>) -> std::io::Result<()>,
    expected: &[&str],
) {
    let expected = expected
        .iter()
        .map(|id| format!("minecraft:{id}"))
        .collect::<Vec<_>>();
    assert_eq!(status_registry_entry_ids_ordered(write_packet), expected);
}

#[test]
pub fn level_chunk_packet_data_uses_vanilla_heightmap_stream_codec_not_nbt() {
    let mut heightmaps = std::collections::BTreeMap::new();
    for name in [
        "WORLD_SURFACE",
        "MOTION_BLOCKING",
        "MOTION_BLOCKING_NO_LEAVES",
    ] {
        heightmaps.insert(name.to_string(), vec![0x0102_0304_0506_0708]);
    }
    let data = super::super::ClientboundLevelChunkPacketData {
        heightmaps,
        buffer: Vec::new(),
        block_entity_count: 0,
        block_entities: Vec::new(),
    };

    let mut payload = Vec::new();
    super::super::write_level_chunk_packet_data(&mut payload, &data).unwrap();

    assert_eq!(
        payload[0], 3,
        "heightmap map count is a VarInt, not NBT TAG_Compound"
    );
    let mut cursor = Cursor::new(payload);
    assert_eq!(read_var_i32(&mut cursor).unwrap(), 3);
    for expected_id in [1, 4, 5] {
        assert_eq!(read_var_i32(&mut cursor).unwrap(), expected_id);
        assert_eq!(read_var_i32(&mut cursor).unwrap(), 1);
        let mut bytes = [0; 8];
        cursor.read_exact(&mut bytes).unwrap();
        assert_eq!(i64::from_be_bytes(bytes), 0x0102_0304_0506_0708);
    }
    assert_eq!(read_var_i32(&mut cursor).unwrap(), 0);
    assert_eq!(read_var_i32(&mut cursor).unwrap(), 0);
}

/// Builds a minimal PlaySessionState with only the fields needed for NBT round-trip
/// tests, seeding inventory with known items.
pub fn session_state_with_inventory(
    items: &[(&'static str, i32, usize)],
) -> super::super::PlaySessionState {
    let mut inventory = PlayerInventory::new();
    let loaded: Vec<(usize, crate::item_stack::ItemStack)> = items
        .iter()
        .map(|(id, count, slot)| (*slot, ItemStack::new(id, *count)))
        .collect();
    inventory.load_items(&loaded);
    super::super::PlaySessionState {
        x: 1.0,
        y: 64.0,
        z: -1.0,
        yaw: 0.0,
        pitch: 0.0,
        on_ground: true,
        fall_distance: 0.0,
        selected_slot: 0,
        health: 20.0,
        food_level: 20,
        food_saturation: 5.0,
        food_exhaustion: 0.0,
        food_tick_timer: 0,
        input_forward: false,
        input_backward: false,
        input_left: false,
        input_right: false,
        input_shift: false,
        input_sprinting: false,
        input_jumping: false,
        air_supply: super::super::MAX_AIR_SUPPLY,
        in_water: false,
        eye_in_water: false,
        water_fluid_height: 0.0,
        water_velocity_x: 0.0,
        water_velocity_y: 0.0,
        water_velocity_z: 0.0,
        xp_progress: 0.0,
        xp_level: 0,
        xp_total: 0,
        xp_seed: 0,
        score: 0,
        game_mode: GameMode::Survival,
        previous_game_mode: None,
        spawn: None,
        seen_credits: false,
        entered_nether_position: None,
        last_death_location: None,
        root_vehicle: None,
        active_effects: Vec::new(),
        ender_items: Vec::new(),
        abilities: PlayerNbtAbilities::default_survival(),
        inventory_menu: InventoryMenu::new(inventory, RecipeMap::default()),
        carried_item: ItemStack::empty(),
        container_state_id: 0,
        next_container_id: 1,
        active_block_menu: None,
        block_break_state: crate::player_game_mode::BlockBreakState::default(),
        recipe_book_settings: super::super::default_recipe_book_settings(),
    }
}

#[test]
pub fn region_feature_cache_keeps_only_fully_decorated_inner_chunks() {
    let center = ChunkPos { x: -41, z: -22 };
    assert_eq!(
        super::super::REGION_FEATURE_GENERATION_RADIUS,
        super::super::REGION_FEATURE_CACHEABLE_RADIUS + 2
    );

    assert!(super::super::region_generated_chunk_is_cacheable(
        center,
        ChunkPos { x: -41, z: -22 }
    ));
    assert!(super::super::region_generated_chunk_is_cacheable(
        center,
        ChunkPos { x: -40, z: -23 }
    ));
    assert!(!super::super::region_generated_chunk_is_cacheable(
        center,
        ChunkPos { x: -39, z: -22 }
    ));
    assert!(!super::super::region_generated_chunk_is_cacheable(
        center,
        ChunkPos { x: -41, z: -20 }
    ));
}

#[test]
pub fn play_session_state_nbt_round_trip_preserves_empty_inventory() {
    // An empty inventory should serialise as an empty Inventory list and
    // deserialise back without error.
    let state = session_state_with_inventory(&[]);
    let tag = play_session_state_to_nbt(&state);
    let restored =
        play_session_state_from_nbt(&tag, GameMode::Survival, &RecipeMap::default()).unwrap();
    assert!(
        restored
            .inventory_menu
            .player_inventory()
            .saved_items()
            .is_empty(),
        "expected empty inventory after round-trip"
    );
}
