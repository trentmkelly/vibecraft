use super::super::*;
use super::*;


fn always_ready_chunk(pos: ChunkPos) -> Option<Arc<LevelChunk>> {
    Some(Arc::new(LevelChunk::empty(pos)))
}

#[test]
fn chunk_sender_starts_batches_sends_nearest_chunks_and_waits_for_first_ack() {
    let mut sender = PlayerChunkSender::new(false);
    for pos in [
        ChunkPos { x: 8, z: 0 },
        ChunkPos { x: 1, z: 0 },
        ChunkPos { x: -2, z: 0 },
        ChunkPos { x: 3, z: 4 },
        ChunkPos { x: 0, z: 2 },
        ChunkPos { x: 4, z: 4 },
        ChunkPos { x: -3, z: 3 },
        ChunkPos { x: 0, z: -1 },
        ChunkPos { x: 2, z: 2 },
        ChunkPos { x: 9, z: 9 },
    ] {
        sender.mark_chunk_pending_to_send(pos);
    }

    let batch = sender
        .send_next_chunks(ChunkPos { x: 0, z: 0 }, always_ready_chunk)
        .expect("ready batch");
    assert_eq!(sender.unacknowledged_batches(), 1);
    let sent: Vec<_> = batch.chunks.iter().map(|(pos, _)| *pos).collect();
    assert_eq!(sent.len(), 9);
    assert_eq!(
        sent,
        vec![
            ChunkPos { x: 0, z: -1 },
            ChunkPos { x: 1, z: 0 },
            ChunkPos { x: -2, z: 0 },
            ChunkPos { x: 0, z: 2 },
            ChunkPos { x: 2, z: 2 },
            ChunkPos { x: -3, z: 3 },
            ChunkPos { x: 3, z: 4 },
            ChunkPos { x: 4, z: 4 },
            ChunkPos { x: 8, z: 0 },
        ]
    );
    assert!(sender.is_pending(ChunkPos { x: 9, z: 9 }));
    assert!(sender
        .send_next_chunks(ChunkPos { x: 0, z: 0 }, always_ready_chunk)
        .is_none());
}

#[test]
fn chunk_sender_applies_client_feedback_clamp_and_allows_more_unacked_batches() {
    let mut sender = PlayerChunkSender::new(false);
    sender.mark_chunk_pending_to_send(ChunkPos { x: 0, z: 0 });
    assert!(sender
        .send_next_chunks(ChunkPos { x: 0, z: 0 }, always_ready_chunk)
        .is_some());

    sender.on_chunk_batch_received_by_client(f32::NAN);
    assert_eq!(
        sender.desired_chunks_per_tick(),
        PlayerChunkSender::MIN_CHUNKS_PER_TICK
    );
    sender.mark_chunk_pending_to_send(ChunkPos { x: 1, z: 0 });
    assert!(sender
        .send_next_chunks(ChunkPos { x: 0, z: 0 }, always_ready_chunk)
        .is_some());

    sender.on_chunk_batch_received_by_client(128.0);
    assert_eq!(
        sender.desired_chunks_per_tick(),
        PlayerChunkSender::MAX_CHUNKS_PER_TICK
    );

    let mut sender = PlayerChunkSender::new(false);
    sender.mark_chunk_pending_to_send(ChunkPos { x: 0, z: 0 });
    assert!(sender
        .send_next_chunks(ChunkPos { x: 0, z: 0 }, always_ready_chunk)
        .is_some());
    sender.on_chunk_batch_received_by_client(1.0);
    for x in 0..10 {
        sender.mark_chunk_pending_to_send(ChunkPos { x, z: 1 });
        assert!(sender
            .send_next_chunks(ChunkPos { x: 0, z: 0 }, always_ready_chunk)
            .is_some());
    }
    assert_eq!(sender.unacknowledged_batches(), 10);
    sender.mark_chunk_pending_to_send(ChunkPos { x: 10, z: 1 });
    assert!(sender
        .send_next_chunks(ChunkPos { x: 0, z: 0 }, always_ready_chunk)
        .is_none());
}

#[test]
fn chunk_sender_keeps_unready_chunks_pending_and_does_not_consume_ack_slot() {
    // Java parity: PlayerChunkSender.collectChunksToSend silently filters
    // pending positions through chunkMap::getChunkToSend, so unready
    // chunks stay pending without consuming an unacknowledged-batch slot.
    let mut sender = PlayerChunkSender::new(false);
    let ready_pos = ChunkPos { x: 1, z: 0 };
    let unready_pos = ChunkPos { x: 0, z: 1 };
    sender.mark_chunk_pending_to_send(ready_pos);
    sender.mark_chunk_pending_to_send(unready_pos);

    let batch = sender
        .send_next_chunks(ChunkPos { x: 0, z: 0 }, |pos| {
            if pos == ready_pos {
                Some(Arc::new(LevelChunk::empty(pos)))
            } else {
                None
            }
        })
        .expect("ready batch");
    assert_eq!(batch.chunks.len(), 1);
    assert_eq!(batch.chunks[0].0, ready_pos);
    assert!(sender.is_pending(unready_pos));
    assert_eq!(sender.unacknowledged_batches(), 1);

    // No ready chunks remain → no batch, no unacked-slot consumption.
    let none = sender.send_next_chunks(ChunkPos { x: 0, z: 0 }, |_| None);
    assert!(none.is_none());
    assert_eq!(sender.unacknowledged_batches(), 1);
    assert!(sender.is_pending(unready_pos));
}

#[test]
fn chunk_sender_nearest_position_rule_drops_unready_near_chunks_for_this_tick() {
    // Java's "pending > quota" path picks the nearest positions FIRST,
    // then filters readiness, so a not-ready near chunk blocks a ready
    // far chunk from being sent this tick (but the far chunk stays
    // pending and will be tried next tick).
    let mut sender = PlayerChunkSender::new(false);
    sender.on_chunk_batch_received_by_client(1.0); // quota = 1
    let near_unready = ChunkPos { x: 0, z: 1 };
    let far_ready = ChunkPos { x: 5, z: 5 };
    sender.mark_chunk_pending_to_send(near_unready);
    sender.mark_chunk_pending_to_send(far_ready);

    let batch = sender.send_next_chunks(ChunkPos { x: 0, z: 0 }, |pos| {
        if pos == far_ready {
            Some(Arc::new(LevelChunk::empty(pos)))
        } else {
            None
        }
    });
    assert!(batch.is_none(), "near unready blocks far ready this tick");
    assert!(sender.is_pending(near_unready));
    assert!(sender.is_pending(far_ready));
}

#[test]
fn chunk_sender_drop_chunk_emits_forget_only_after_chunk_was_sent() {
    let mut sender = PlayerChunkSender::new(false);
    let pending = ChunkPos { x: 4, z: -2 };
    sender.mark_chunk_pending_to_send(pending);
    assert_eq!(sender.drop_chunk(pending, true), None);

    let sent = ChunkPos { x: -3, z: 5 };
    sender.mark_chunk_pending_to_send(sent);
    assert!(sender
        .send_next_chunks(ChunkPos { x: 0, z: 0 }, always_ready_chunk)
        .is_some());
    assert_eq!(
        sender.drop_chunk(sent, true),
        Some(PlayInstruction::ForgetLevelChunk { pos: sent })
    );
    assert_eq!(sender.drop_chunk(sent, false), None);
}

#[test]
fn remove_entities_packet_uses_java_int_id_list_encoding() {
    let registry = PlayProtocolRegistry::new();
    assert_eq!(CLIENTBOUND_REMOVE_ENTITIES_PACKET_ID, 77);
    assert_eq!(
        registry.clientbound_name(CLIENTBOUND_REMOVE_ENTITIES_PACKET_ID),
        Some("remove_entities")
    );

    let mut payload = Vec::new();
    ClientboundRemoveEntitiesPacket {
        entity_ids: vec![0, 300, 16_383],
    }
    .write(&mut payload)
    .unwrap();

    assert_eq!(payload, vec![3, 0, 0xac, 0x02, 0xff, 0x7f]);
}

#[test]
fn rotate_head_packet_uses_java_entity_id_then_packed_head_yaw() {
    let registry = PlayProtocolRegistry::new();
    assert_eq!(CLIENTBOUND_ROTATE_HEAD_PACKET_ID, 83);
    assert_eq!(
        registry.clientbound_name(CLIENTBOUND_ROTATE_HEAD_PACKET_ID),
        Some("rotate_head")
    );

    let packet = ClientboundRotateHeadPacket::new(300, 180.0);
    assert_eq!(packet.y_head_rot, 128);

    let mut payload = Vec::new();
    packet.write(&mut payload).unwrap();
    assert_eq!(payload, vec![0xac, 0x02, 0x80]);
}

#[test]
fn animate_packet_uses_java_entity_id_then_unsigned_action_byte() {
    let registry = PlayProtocolRegistry::new();
    assert_eq!(CLIENTBOUND_ANIMATE_PACKET_ID, 2);
    assert_eq!(
        registry.clientbound_name(CLIENTBOUND_ANIMATE_PACKET_ID),
        Some("animate")
    );
    assert_eq!(EntityAnimation::SwingMainHand as u8, 0);
    assert_eq!(EntityAnimation::WakeUp as u8, 2);
    assert_eq!(EntityAnimation::SwingOffHand as u8, 3);
    assert_eq!(EntityAnimation::CriticalHit as u8, 4);
    assert_eq!(EntityAnimation::MagicCriticalHit as u8, 5);

    let mut payload = Vec::new();
    ClientboundAnimatePacket {
        id: 300,
        action: EntityAnimation::MagicCriticalHit,
    }
    .write(&mut payload)
    .unwrap();
    assert_eq!(payload, vec![0xac, 0x02, 5]);
}

#[test]
fn entity_event_packet_uses_java_fixed_int_entity_id_then_event_byte() {
    let registry = PlayProtocolRegistry::new();
    assert_eq!(CLIENTBOUND_ENTITY_EVENT_PACKET_ID, 34);
    assert_eq!(
        registry.clientbound_name(CLIENTBOUND_ENTITY_EVENT_PACKET_ID),
        Some("entity_event")
    );

    let mut payload = Vec::new();
    ClientboundEntityEventPacket {
        entity_id: 300,
        event_id: 3,
    }
    .write(&mut payload)
    .unwrap();
    assert_eq!(payload, vec![0x00, 0x00, 0x01, 0x2c, 0x03]);
}

#[test]
fn set_passengers_packet_uses_java_vehicle_then_varint_array() {
    let registry = PlayProtocolRegistry::new();
    assert_eq!(CLIENTBOUND_SET_PASSENGERS_PACKET_ID, 107);
    assert_eq!(
        registry.clientbound_name(CLIENTBOUND_SET_PASSENGERS_PACKET_ID),
        Some("set_passengers")
    );

    let mut payload = Vec::new();
    ClientboundSetPassengersPacket {
        vehicle: 300,
        passengers: vec![0, 301, 16_383],
    }
    .write(&mut payload)
    .unwrap();

    assert_eq!(
        payload,
        vec![0xac, 0x02, 3, 0, 0xad, 0x02, 0xff, 0x7f]
    );
}

#[test]
fn move_vehicle_packet_uses_java_vec3_then_yaw_pitch() {
    let registry = PlayProtocolRegistry::new();
    assert_eq!(CLIENTBOUND_MOVE_VEHICLE_PACKET_ID, 57);
    assert_eq!(
        registry.clientbound_name(CLIENTBOUND_MOVE_VEHICLE_PACKET_ID),
        Some("move_vehicle")
    );

    let mut payload = Vec::new();
    ClientboundMoveVehiclePacket {
        position: Vec3 {
            x: 1.25,
            y: 65.0,
            z: -2.5,
        },
        y_rot: 90.0,
        x_rot: -30.5,
    }
    .write(&mut payload)
    .unwrap();

    assert_eq!(payload.len(), 32);
    assert_eq!(&payload[0..8], &1.25_f64.to_be_bytes());
    assert_eq!(&payload[8..16], &65.0_f64.to_be_bytes());
    assert_eq!(&payload[16..24], &(-2.5_f64).to_be_bytes());
    assert_eq!(&payload[24..28], &90.0_f32.to_be_bytes());
    assert_eq!(&payload[28..32], &(-30.5_f32).to_be_bytes());
}

#[test]
fn move_entity_pos_packet_uses_java_varint_signed_shorts_then_on_ground() {
    let registry = PlayProtocolRegistry::new();
    assert_eq!(CLIENTBOUND_MOVE_ENTITY_POS_PACKET_ID, 53);
    assert_eq!(
        registry.clientbound_name(CLIENTBOUND_MOVE_ENTITY_POS_PACKET_ID),
        Some("move_entity_pos")
    );

    let mut payload = Vec::new();
    ClientboundMoveEntityPacket::pos(300, [32_767, -32_768, -2], false)
        .write_pos(&mut payload)
        .unwrap();

    assert_eq!(
        payload,
        vec![0xac, 0x02, 0x7f, 0xff, 0x80, 0x00, 0xff, 0xfe, 0x00]
    );
}

#[test]
fn move_entity_pos_rot_packet_uses_java_deltas_rotation_bytes_then_on_ground() {
    let registry = PlayProtocolRegistry::new();
    assert_eq!(CLIENTBOUND_MOVE_ENTITY_POS_ROT_PACKET_ID, 54);
    assert_eq!(
        registry.clientbound_name(CLIENTBOUND_MOVE_ENTITY_POS_ROT_PACKET_ID),
        Some("move_entity_pos_rot")
    );

    let mut payload = Vec::new();
    ClientboundMoveEntityPacket::pos_rot(300, [1, -2, 3], 90.0, -45.0, true)
        .write_pos_rot(&mut payload)
        .unwrap();

    assert_eq!(
        payload,
        vec![0xac, 0x02, 0x00, 0x01, 0xff, 0xfe, 0x00, 0x03, 0x40, 0xe0, 0x01]
    );
}

#[test]
fn move_entity_rot_packet_uses_java_rotation_bytes_then_on_ground() {
    let registry = PlayProtocolRegistry::new();
    assert_eq!(CLIENTBOUND_MOVE_ENTITY_ROT_PACKET_ID, 56);
    assert_eq!(
        registry.clientbound_name(CLIENTBOUND_MOVE_ENTITY_ROT_PACKET_ID),
        Some("move_entity_rot")
    );

    let mut payload = Vec::new();
    ClientboundMoveEntityPacket::rot(300, 180.0, -45.0, false)
        .write_rot(&mut payload)
        .unwrap();

    assert_eq!(payload, vec![0xac, 0x02, 0x80, 0xe0, 0x00]);
}

#[test]
fn set_entity_motion_packet_uses_java_varint_then_lp_vec3_without_legacy_clamp() {
    let registry = PlayProtocolRegistry::new();
    assert_eq!(CLIENTBOUND_SET_ENTITY_MOTION_PACKET_ID, 101);
    assert_eq!(
        registry.clientbound_name(CLIENTBOUND_SET_ENTITY_MOTION_PACKET_ID),
        Some("set_entity_motion")
    );

    let packet = ClientboundSetEntityMotionPacket::new(
        300,
        Vec3 {
            x: 4.5,
            y: -4.5,
            z: 0.0,
        },
    );
    assert_eq!(
        packet.movement,
        Vec3 {
            x: 4.5,
            y: -4.5,
            z: 0.0,
        }
    );

    let mut payload = Vec::new();
    packet.write(&mut payload).unwrap();

    assert_eq!(
        payload,
        vec![0xac, 0x02, 0xc5, 0xcc, 0x7f, 0xfe, 0x19, 0x9b, 0x01]
    );
}

#[test]
fn teleport_entity_packet_uses_java_position_move_rotation_then_relative_int_and_on_ground() {
    let registry = PlayProtocolRegistry::new();
    assert_eq!(CLIENTBOUND_TELEPORT_ENTITY_PACKET_ID, 125);
    assert_eq!(
        registry.clientbound_name(CLIENTBOUND_TELEPORT_ENTITY_PACKET_ID),
        Some("teleport_entity")
    );

    let mut payload = Vec::new();
    ClientboundTeleportEntityPacket {
        id: 300,
        position: Vec3 {
            x: 1.25,
            y: 64.0,
            z: -2.5,
        },
        movement: Vec3 {
            x: 0.1,
            y: -0.2,
            z: 0.3,
        },
        y_rot: 90.0,
        x_rot: -30.5,
        relative_flags: 0x1ff,
        on_ground: false,
    }
    .write(&mut payload)
    .unwrap();

    assert_eq!(payload.len(), 63);
    assert_eq!(&payload[0..2], &[0xac, 0x02]);
    assert_eq!(&payload[2..10], &1.25_f64.to_be_bytes());
    assert_eq!(&payload[10..18], &64.0_f64.to_be_bytes());
    assert_eq!(&payload[18..26], &(-2.5_f64).to_be_bytes());
    assert_eq!(&payload[26..34], &0.1_f64.to_be_bytes());
    assert_eq!(&payload[34..42], &(-0.2_f64).to_be_bytes());
    assert_eq!(&payload[42..50], &0.3_f64.to_be_bytes());
    assert_eq!(&payload[50..54], &90.0_f32.to_be_bytes());
    assert_eq!(&payload[54..58], &(-30.5_f32).to_be_bytes());
    assert_eq!(&payload[58..62], &0x1ff_i32.to_be_bytes());
    assert_eq!(payload[62], 0);
}

#[test]
fn experience_orbs_use_add_entity_packet_not_a_dedicated_game_packet() {
    let registry = PlayProtocolRegistry::new();
    assert_eq!(
        registry.clientbound_name(CLIENTBOUND_ADD_ENTITY_PACKET_ID),
        Some("add_entity")
    );
    assert!((0..CLIENTBOUND_PLAY_PACKET_COUNT_26_1_2 as i32)
        .all(|id| registry.clientbound_name(id) != Some("add_experience_orb")));
}

#[test]
fn remove_mob_effect_packet_uses_java_entity_then_effect_holder_varints() {
    let registry = PlayProtocolRegistry::new();
    assert_eq!(CLIENTBOUND_REMOVE_MOB_EFFECT_PACKET_ID, 78);
    assert_eq!(
        registry.clientbound_name(CLIENTBOUND_REMOVE_MOB_EFFECT_PACKET_ID),
        Some("remove_mob_effect")
    );

    let mut payload = Vec::new();
    ClientboundRemoveMobEffectPacket {
        entity_id: 300,
        effect_id: 129,
    }
    .write(&mut payload)
    .unwrap();

    assert_eq!(payload, vec![0xac, 0x02, 0x81, 0x01]);
}

#[test]
fn chunk_batch_received_packet_uses_big_endian_float_payload() {
    let packet = ServerboundChunkBatchReceivedPacket {
        desired_chunks_per_tick: 12.5,
    };
    let mut bytes = Vec::new();
    packet.write(&mut bytes).unwrap();
    assert_eq!(bytes, 12.5_f32.to_be_bytes());
    assert_eq!(
        ServerboundChunkBatchReceivedPacket::read(&mut cursor(bytes.clone())).unwrap(),
        packet
    );
    assert!(
        ServerboundChunkBatchReceivedPacket::read(&mut cursor(vec![0x41, 0x48, 0x00])).is_err()
    );
    let mut trailing = bytes;
    trailing.push(0);
    assert!(ServerboundChunkBatchReceivedPacket::read(&mut cursor(trailing)).is_err());
}

#[test]
fn game_event_packet_uses_java_event_ids_and_float_payload() {
    assert_eq!(CLIENTBOUND_GAME_EVENT_PACKET_ID, 38);

    let cases = [
        (ClientboundGameEventType::NoRespawnBlockAvailable, 0),
        (ClientboundGameEventType::StartRaining, 1),
        (ClientboundGameEventType::StopRaining, 2),
        (ClientboundGameEventType::ChangeGameMode, 3),
        (ClientboundGameEventType::WinGame, 4),
        (ClientboundGameEventType::DemoEvent, 5),
        (ClientboundGameEventType::PlayArrowHitSound, 6),
        (ClientboundGameEventType::RainLevelChange, 7),
        (ClientboundGameEventType::ThunderLevelChange, 8),
        (ClientboundGameEventType::PufferFishSting, 9),
        (ClientboundGameEventType::GuardianElderEffect, 10),
        (ClientboundGameEventType::ImmediateRespawn, 11),
        (ClientboundGameEventType::LimitedCrafting, 12),
        (ClientboundGameEventType::LevelChunksLoadStart, 13),
    ];

    for (event, id) in cases {
        let packet = ClientboundGameEventPacket { event, param: 0.5 };
        let mut bytes = Vec::new();
        packet.write(&mut bytes).unwrap();
        assert_eq!(bytes[0], id);
        assert_eq!(&bytes[1..5], &0.5_f32.to_be_bytes());
        assert_eq!(
            ClientboundGameEventPacket::read(&mut cursor(bytes)).unwrap(),
            packet
        );
    }

    assert!(ClientboundGameEventPacket::read(&mut cursor(vec![7, 0, 0, 0])).is_err());
    let mut trailing = vec![7];
    trailing.extend_from_slice(&0.5_f32.to_be_bytes());
    trailing.push(0);
    assert!(ClientboundGameEventPacket::read(&mut cursor(trailing)).is_err());
    assert_eq!(
        ClientboundGameEventPacket::read(&mut cursor(vec![250, 0, 0, 0, 0]))
            .unwrap()
            .event,
        ClientboundGameEventType::Unknown(250)
    );
}

#[test]
fn light_update_data_uses_vanilla_masks_and_2048_byte_layers() {
    let sections = vec![
        ChunkSection {
            y: 0,
            block_states: PalettedContainer::single(Tag::Int(0), 4096).to_nbt(),
            biomes: PalettedContainer::single(Tag::Int(0), 64).to_nbt(),
            block_light: Some(vec![0; 2048]),
            sky_light: Some(vec![-1; 2048]),
        },
        ChunkSection {
            y: 1,
            block_states: PalettedContainer::single(Tag::Int(0), 4096).to_nbt(),
            biomes: PalettedContainer::single(Tag::Int(0), 64).to_nbt(),
            block_light: Some(vec![1; 2048]),
            sky_light: None,
        },
    ];

    let data = ClientboundLightUpdatePacketData::from_chunk_sections(&sections);
    assert_eq!(data.sky_y_mask, vec![1]);
    assert_eq!(data.empty_block_y_mask, vec![1]);
    assert_eq!(data.block_y_mask, vec![2]);
    assert_eq!(data.sky_updates.len(), 1);
    assert_eq!(data.block_updates.len(), 1);

    let mut payload = Vec::new();
    data.write(&mut payload).unwrap();
    assert!(!payload.is_empty());
    assert!(
        payload.windows(3).any(|bytes| bytes == [0x80, 0x10, 0xff]),
        "sky light data layers use ByteBufCodecs.byteArray(2048): VarInt length then bytes"
    );
    assert!(
        payload.windows(3).any(|bytes| bytes == [0x80, 0x10, 0x01]),
        "block light data layers use ByteBufCodecs.byteArray(2048): VarInt length then bytes"
    );
}

#[test]
fn light_update_packet_writes_vanilla_varint_coordinates_then_light_data() {
    let packet = ClientboundLightUpdatePacket {
        pos: ChunkPos { x: 128, z: -2 },
        light_data: ClientboundLightUpdatePacketData {
            sky_y_mask: vec![0b10],
            block_y_mask: vec![0b100],
            empty_sky_y_mask: vec![0b1000],
            empty_block_y_mask: vec![0b1_0000],
            sky_updates: vec![vec![-1; ClientboundLightUpdatePacketData::DATA_LAYER_SIZE]],
            block_updates: vec![vec![1; ClientboundLightUpdatePacketData::DATA_LAYER_SIZE]],
        },
    };

    let mut payload = Vec::new();
    packet.write(&mut payload).unwrap();

    let expected_prefix = [
        0x80, 0x01, // x = 128 as Java VarInt
        0xfe, 0xff, 0xff, 0xff, 0x0f, // z = -2 as Java VarInt
        0x01, // skyYMask long-array length
        0, 0, 0, 0, 0, 0, 0, 0b10,
        0x01, // blockYMask long-array length
        0, 0, 0, 0, 0, 0, 0, 0b100,
        0x01, // emptySkyYMask long-array length
        0, 0, 0, 0, 0, 0, 0, 0b1000,
        0x01, // emptyBlockYMask long-array length
        0, 0, 0, 0, 0, 0, 0, 0b1_0000,
        0x01, // sky update list length
        0x80, 0x10, // ByteBufCodecs.byteArray(2048) length
        0xff,
    ];
    assert_eq!(&payload[..expected_prefix.len()], &expected_prefix);
    assert!(
        payload
            .windows(3)
            .any(|bytes| bytes == [0x80, 0x10, 0x01]),
        "block update list writes a second 2048-byte data layer"
    );
}

#[test]
fn chunk_section_serialization_matches_vanilla_section_field_order() {
    let section = NetworkChunkSection {
        non_empty_block_count: 2,
        fluid_count: 0,
        block_states: NetworkPalettedContainer::single(5),
        biomes: NetworkPalettedContainer::single(7),
    };
    let mut bytes = Vec::new();
    section.write(&mut bytes).unwrap();

    assert_eq!(&bytes[0..2], &2_i16.to_be_bytes());
    assert_eq!(&bytes[2..4], &0_i16.to_be_bytes());
    assert_eq!(bytes[4], 0);
    assert_eq!(bytes[5], 5);
    assert_eq!(bytes[6], 0);
    assert_eq!(bytes[7], 7);
    assert_eq!(bytes.len(), 8);
}

#[test]
fn network_chunk_sections_report_vanilla_fluid_counts() {
    let section = ChunkSection {
        y: 0,
        block_states: PalettedContainer::single(
            Tag::String("minecraft:water[level=0]".to_string()),
            4096,
        )
        .to_nbt(),
        biomes: PalettedContainer::single(Tag::Int(0), 64).to_nbt(),
        block_light: None,
        sky_light: None,
    };

    let network = NetworkChunkSection::from_storage_section(&section);

    assert_eq!(network.non_empty_block_count, 4096);
    assert_eq!(network.fluid_count, 4096);
    assert_eq!(network.block_states.palette_ids, vec![86]);
}

#[test]
fn section_fluid_counts_follow_palette_indices() {
    let mut waterlogged_fence =
        crate::storage::chunk::BlockStateEntry::new("minecraft:oak_fence");
    waterlogged_fence
        .properties
        .insert("waterlogged".to_string(), "true".to_string());
    let indices = (0..4096)
        .map(|index| if index % 2 == 0 { 0 } else { 1 })
        .collect::<Vec<_>>();
    let container = PalettedContainer {
        palette: vec![
            crate::storage::chunk::BlockStateEntry::new("minecraft:air").to_nbt(),
            waterlogged_fence.to_nbt(),
        ],
        data: Some(crate::storage::chunk::pack_palette_indices(&indices, 4)),
        expected_entries: 4096,
    };

    assert_eq!(section_non_empty_block_count(&container.to_nbt()), 2048);
    assert_eq!(section_fluid_count(&container.to_nbt()), 2048);
}

#[test]
fn generated_terrain_block_state_names_use_current_protocol_state_ids() {
    for (name, id) in [
        ("minecraft:water", 86),
        ("minecraft:sand", 118),
        ("minecraft:red_sand", 123),
        ("minecraft:gravel", 124),
        ("minecraft:sandstone", 578),
        ("minecraft:red_sandstone", 13247),
        ("minecraft:white_terracotta", 11444),
        ("minecraft:orange_terracotta", 11445),
        ("minecraft:terracotta", 12912),
        ("minecraft:yellow_terracotta", 11448),
        ("minecraft:brown_terracotta", 11456),
        ("minecraft:red_terracotta", 11458),
        ("minecraft:light_gray_terracotta", 11452),
        ("minecraft:short_grass", 2248),
        ("minecraft:dandelion", 2321),
        ("minecraft:poppy", 2324),
        ("minecraft:oak_log", 137),
        ("minecraft:birch_log", 143),
        ("minecraft:oak_leaves", 279),
        ("minecraft:birch_leaves", 335),
        ("minecraft:sunflower", 12916),
        ("minecraft:tuff", 23452),
        ("minecraft:deepslate", 27924),
        ("minecraft:copper_ore", 25313),
        ("minecraft:deepslate_copper_ore", 25314),
    ] {
        assert_eq!(block_state_name_network_id(name), Some(id), "{name}");
    }
}

#[test]
fn storage_palette_network_bits_match_packed_storage_width() {
    let palette = (0..17).map(Tag::Int).collect::<Vec<_>>();
    let container = PalettedContainer {
        palette,
        data: Some(vec![16]),
        expected_entries: 4096,
    };

    let network = NetworkPalettedContainer::from_storage_container(
        &container.to_nbt(),
        PaletteKind::BlockState,
    );

    assert_eq!(network.bits_per_entry, 5);
    assert_eq!(network.palette_ids.len(), 17);
    assert_eq!(network.data, vec![16]);
}

#[test]
fn large_block_palettes_use_global_palette_without_indirect_list() {
    let palette = (0..300).map(Tag::Int).collect::<Vec<_>>();
    let indices = vec![299_u64; 4096];
    let container = PalettedContainer {
        palette,
        data: Some(crate::storage::chunk::pack_palette_indices(&indices, 9)),
        expected_entries: 4096,
    };

    let network = NetworkPalettedContainer::from_storage_container(
        &container.to_nbt(),
        PaletteKind::BlockState,
    );

    assert!(network.uses_global_palette);
    assert_eq!(network.bits_per_entry, 15);
    assert!(network.palette_ids.is_empty());
    assert_eq!(
        crate::storage::chunk::unpack_palette_indices(
            &network.data,
            network.bits_per_entry as usize,
            1,
        )[0],
        299
    );

    let mut bytes = Vec::new();
    network.write(&mut bytes).unwrap();
    assert_eq!(bytes[0], 15);
    assert_ne!(
        bytes[1], 0xac,
        "global palette containers must not write an indirect palette length"
    );
}

#[test]
fn biome_palette_network_ids_follow_synchronized_biome_registry_order() {
    assert_eq!(biome_name_network_id("minecraft:plains"), Some(40));
    assert_eq!(biome_name_network_id("plains"), Some(40));
    assert_eq!(biome_name_network_id("minecraft:the_void"), Some(57));

    let network = NetworkPalettedContainer::from_storage_container(
        &PalettedContainer::single(Tag::String("minecraft:plains".to_string()), 64).to_nbt(),
        PaletteKind::Biome,
    );

    assert_eq!(network.bits_per_entry, 0);
    assert_eq!(network.palette_ids, vec![40]);
}

#[test]
fn level_chunk_with_light_packet_carries_chunk_buffer_then_light_payload_data() {
    let mut heightmaps = BTreeMap::new();
    heightmaps.insert("WORLD_SURFACE".to_string(), Tag::LongArray(vec![1, 2, 3]));
    let chunk = LevelChunk {
        pos: ChunkPos { x: 4, z: -2 },
        min_section_y: 0,
        last_update: 0,
        status: "minecraft:full".to_string(),
        inhabited_time: 0,
        sections: vec![ChunkSection {
            y: 0,
            block_states: PalettedContainer::single(Tag::Int(5), 4096).to_nbt(),
            biomes: PalettedContainer::single(Tag::Int(7), 64).to_nbt(),
            block_light: Some(vec![0; 2048]),
            sky_light: Some(vec![-1; 2048]),
        }],
        heightmaps,
        block_entities: vec![Tag::Compound(vec![
            ("id".to_string(), Tag::String("minecraft:chest".to_string())),
            ("x".to_string(), Tag::Int(65)),
            ("y".to_string(), Tag::Int(70)),
            ("z".to_string(), Tag::Int(-18)),
        ])],
        entities: Vec::new(),
        structures: Tag::Compound(Vec::new()),
        upgrade_data: None,
        blending_data: None,
        below_zero_retrogen: None,
        carving_mask: None,
        block_ticks: Vec::new(),
        fluid_ticks: Vec::new(),
        post_processing: Vec::new(),
        light_correct: false,
    };
    let light_data = ClientboundLightUpdatePacketData::from_chunk(&chunk);
    let packet = ClientboundLevelChunkWithLightPacket::from_chunk(&chunk, light_data.clone());

    assert_eq!(packet.pos, chunk.pos);
    let chunk_data = packet.chunk_data.as_ref().unwrap();
    assert_eq!(chunk_data.heightmaps["WORLD_SURFACE"], vec![1, 2, 3]);
    assert_eq!(chunk_data.block_entity_count, 1);
    assert_eq!(chunk_data.block_entities.len(), 1);
    assert_eq!(chunk_data.block_entities[0].packed_xz, 0x1e);
    assert_eq!(chunk_data.block_entities[0].y, 70);
    assert_eq!(chunk_data.block_entities[0].block_entity_type_id, 1);
    assert_eq!(chunk_data.buffer.len(), OVERWORLD_SECTION_COUNT * 8);
    assert_eq!(
        &chunk_data.buffer[0..8],
        &[0, 0, 0, 0, 0, 0, 0, 40],
        "missing sections before Y=0 are serialized as air/plains"
    );
    let y0_offset = (0 - OVERWORLD_MIN_SECTION_Y) as usize * 8;
    assert_eq!(
        &chunk_data.buffer[y0_offset..y0_offset + 8],
        &[0x10, 0, 0, 0, 0, 5, 0, 7],
        "storage section Y=0 must remain at network section index 4"
    );
    assert_eq!(packet.light_data, Some(light_data.clone()));

    let mut chunk_payload = Vec::new();
    packet.write(&mut chunk_payload).unwrap();
    assert_eq!(&chunk_payload[..4], &4_i32.to_be_bytes());
    assert_eq!(&chunk_payload[4..8], &(-2_i32).to_be_bytes());
    assert_eq!(chunk_payload[8], 1);
    assert_eq!(chunk_payload[9], 1);
    assert_eq!(chunk_payload[10], 3);
    assert_eq!(&chunk_payload[11..19], &1_i64.to_be_bytes());
    assert_eq!(&chunk_payload[19..27], &2_i64.to_be_bytes());
    assert_eq!(&chunk_payload[27..35], &3_i64.to_be_bytes());
    assert_eq!(
        read_var_i32(&mut cursor(chunk_payload[35..].to_vec())).unwrap(),
        (OVERWORLD_SECTION_COUNT * 8) as i32
    );
    assert!(
        chunk_payload
            .windows(4)
            .any(|bytes| bytes == [1, 0x1e, 0, 70]),
        "block entity list writes packed XZ, y short, type id and tag"
    );

    let mut light_payload = Vec::new();
    ClientboundLightUpdatePacket {
        pos: chunk.pos,
        light_data: light_data.clone(),
    }
    .write(&mut light_payload)
    .unwrap();
    assert_eq!(&light_payload[..2], &[4, 0xfe]);
    assert!(
        light_data
            .sky_y_mask
            .first()
            .is_some_and(|mask| mask & (1 << 4) != 0),
        "storage section Y=0 light must be mapped to overworld network section index 4"
    );
}

#[test]
fn sparse_chunk_sections_are_padded_to_vanilla_overworld_height() {
    let mut chunk = LevelChunk::empty(ChunkPos { x: 0, z: 0 });
    chunk.min_section_y = OVERWORLD_MIN_SECTION_Y;
    chunk.sections = vec![ChunkSection {
        y: 4,
        block_states: PalettedContainer::single(Tag::Int(1), 4096).to_nbt(),
        biomes: PalettedContainer::single(Tag::String("minecraft:plains".to_string()), 64)
            .to_nbt(),
        block_light: None,
        sky_light: Some(vec![-1; 2048]),
    }];

    let data = ClientboundLevelChunkPacketData::from_chunk(&chunk);
    assert_eq!(data.buffer.len(), OVERWORLD_SECTION_COUNT * 8);
    let section_y_4_offset = (4 - OVERWORLD_MIN_SECTION_Y) as usize * 8;
    assert_eq!(
        &data.buffer[section_y_4_offset..section_y_4_offset + 8],
        &[0x10, 0, 0, 0, 0, 1, 0, 40],
        "section Y=4 must serialize at index 8, not at the bottom of the packet"
    );
    assert_eq!(
        &data.buffer[0..8],
        &[0, 0, 0, 0, 0, 0, 0, 40],
        "lower missing sections must remain explicit air sections"
    );

    let light = ClientboundLightUpdatePacketData::from_chunk(&chunk);
    assert!(light
        .sky_y_mask
        .first()
        .is_some_and(|mask| mask & (1 << 8) != 0));
}

#[test]
fn map_item_data_packet_uses_java_optional_lists_patch_and_rotation_mask() {
    let mut with_patch = Vec::new();
    ClientboundMapItemDataPacket {
        map_id: 300,
        scale: 2,
        locked: true,
        decorations: Some(vec![MapDecorationData {
            decoration_type_id: 7,
            x: -1,
            y: 2,
            rotation: 19,
            name: None,
        }]),
        color_patch: Some(MapPatch {
            width: 2,
            height: 1,
            start_x: 4,
            start_y: 5,
            colors: vec![6, 7],
        }),
    }
    .write(&mut with_patch)
    .unwrap();

    assert_eq!(
        with_patch,
        vec![
            0xac, 0x02, // MapId.STREAM_CODEC delegates to VarInt.
            2, 1, // scale byte, locked bool.
            1, 1, // optional decorations present, list length.
            7, 0xff, 2, 3, 0, // type, x, y, rot & 15, optional name absent.
            2, 1, 4, 5, 2, 6, 7, // patch width/height/start/colors byte array.
        ]
    );

    let mut empty_update = Vec::new();
    ClientboundMapItemDataPacket {
        map_id: 1,
        scale: 0,
        locked: false,
        decorations: None,
        color_patch: None,
    }
    .write(&mut empty_update)
    .unwrap();
    assert_eq!(empty_update, vec![1, 0, 0, 0, 0]);
}

#[test]
fn update_attributes_packet_uses_java_snapshot_and_modifier_order() {
    let mut payload = Vec::new();
    ClientboundUpdateAttributesPacket {
        entity_id: 300,
        attributes: vec![
            AttributeSnapshot {
                attribute_id: 4,
                base: 20.0,
                modifiers: vec![AttributeModifierSnapshot {
                    id: Identifier::parse("minecraft:movement_speed").unwrap(),
                    amount: 0.5,
                    operation: AttributeModifierOperation::AddMultipliedTotal,
                }],
            },
            AttributeSnapshot {
                attribute_id: 22,
                base: 0.7,
                modifiers: Vec::new(),
            },
        ],
    }
    .write(&mut payload)
    .unwrap();

    let mut expected = vec![0xac, 0x02, 2, 4];
    expected.extend_from_slice(&20.0_f64.to_be_bytes());
    expected.extend_from_slice(&[1, 24]);
    expected.extend_from_slice(b"minecraft:movement_speed");
    expected.extend_from_slice(&0.5_f64.to_be_bytes());
    expected.extend_from_slice(&[2, 22]);
    expected.extend_from_slice(&0.7_f64.to_be_bytes());
    expected.push(0);
    assert_eq!(payload, expected);
}

#[test]
fn join_sequence_enters_play_with_login_held_slot_and_position_packets() {
    let mut session = PlaySession::new(42, 3);
    let login = ClientboundLoginPacket {
        player_id: 42,
        hardcore: false,
        levels: vec![Identifier::parse("minecraft:overworld").unwrap()],
        max_players: 20,
        chunk_radius: 10,
        simulation_distance: 10,
        reduced_debug_info: false,
        show_death_screen: true,
        do_limited_crafting: false,
        spawn_info: CommonPlayerSpawnInfo::default(),
        enforces_secure_chat: false,
    };

    let instructions = session.join_sequence(login.clone());
    assert_eq!(session.state, PlayState::WaitingForPlayerLoaded);
    assert_eq!(
        instructions,
        vec![
            PlayInstruction::Login(login),
            PlayInstruction::SetHeldSlot(ClientboundSetHeldSlotPacket { slot: 3 }),
            PlayInstruction::PlayerPosition { teleport_id: 0 }
        ]
    );
}

#[test]
fn login_and_respawn_packets_write_common_spawn_info_in_vanilla_order() {
    let spawn_info = CommonPlayerSpawnInfo {
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
    };
    let login = ClientboundLoginPacket {
        player_id: 42,
        hardcore: true,
        levels: vec![
            Identifier::parse("minecraft:overworld").unwrap(),
            Identifier::parse("minecraft:the_nether").unwrap(),
        ],
        max_players: 20,
        chunk_radius: 10,
        simulation_distance: 8,
        reduced_debug_info: false,
        show_death_screen: true,
        do_limited_crafting: false,
        spawn_info: spawn_info.clone(),
        enforces_secure_chat: true,
    };

    let mut login_payload = Vec::new();
    login.write(&mut login_payload).unwrap();
    assert_eq!(&login_payload[..5], &[0, 0, 0, 42, 1]);
    let mut input = cursor(login_payload);
    assert_eq!(read_i32(&mut input).unwrap(), 42);
    assert!(read_bool(&mut input).unwrap());
    assert_eq!(read_var_i32(&mut input).unwrap(), 2);
    assert_eq!(
        read_identifier(&mut input).unwrap(),
        Identifier::parse("minecraft:overworld").unwrap()
    );
    assert_eq!(
        read_identifier(&mut input).unwrap(),
        Identifier::parse("minecraft:the_nether").unwrap()
    );
    assert_eq!(read_var_i32(&mut input).unwrap(), 20);
    assert_eq!(read_var_i32(&mut input).unwrap(), 10);
    assert_eq!(read_var_i32(&mut input).unwrap(), 8);
    assert!(!read_bool(&mut input).unwrap());
    assert!(read_bool(&mut input).unwrap());
    assert!(!read_bool(&mut input).unwrap());
    assert_eq!(read_var_i32(&mut input).unwrap(), 3);
    assert_eq!(
        read_identifier(&mut input).unwrap(),
        Identifier::parse("minecraft:the_nether").unwrap()
    );
    assert_eq!(read_i64(&mut input).unwrap(), -7);
    assert_eq!(read_u8(&mut input).unwrap(), 1);
    assert_eq!(read_u8(&mut input).unwrap(), 0);
    assert!(!read_bool(&mut input).unwrap());
    assert!(read_bool(&mut input).unwrap());
    assert!(read_bool(&mut input).unwrap());
    assert_eq!(
        read_identifier(&mut input).unwrap(),
        Identifier::parse("minecraft:overworld").unwrap()
    );
    assert_eq!(read_block_position(&mut input).unwrap(), (1, 64, -2));
    assert_eq!(read_var_i32(&mut input).unwrap(), 20);
    assert_eq!(read_var_i32(&mut input).unwrap(), 32);
    assert!(read_bool(&mut input).unwrap());

    let mut respawn_payload = Vec::new();
    ClientboundRespawnPacket {
        spawn_info,
        data_to_keep: RespawnDataToKeep::KEEP_ALL_DATA,
    }
    .write(&mut respawn_payload)
    .unwrap();
    assert_eq!(*respawn_payload.last().unwrap(), 3);
}

#[test]
fn vanilla_join_sequence_matches_player_list_packet_and_side_effect_order() {
    let mut session = PlaySession::new(42, 3);
    session.container_state_id = 42;
    let login = ClientboundLoginPacket {
        player_id: 42,
        hardcore: true,
        levels: vec![
            Identifier::parse("minecraft:overworld").unwrap(),
            Identifier::parse("minecraft:the_nether").unwrap(),
            Identifier::parse("minecraft:the_end").unwrap(),
        ],
        max_players: 20,
        chunk_radius: 10,
        simulation_distance: 10,
        reduced_debug_info: false,
        show_death_screen: true,
        do_limited_crafting: false,
        spawn_info: CommonPlayerSpawnInfo::default(),
        enforces_secure_chat: true,
    };
    let abilities = PlayerAbilities {
        invulnerable: false,
        flying: false,
        may_fly: false,
        instabuild: false,
        flying_speed: 0.05,
        walking_speed: 0.1,
    };

    let instructions = session.vanilla_join_sequence(JoinGameSettings {
        login: login.clone(),
        difficulty: GameDifficulty::Hard,
        difficulty_locked: true,
        abilities,
        permission_level: 2,
        initial_recipes: true,
        initial_recipe_book: true,
        scoreboard: true,
        server_status: true,
        player_info_existing_count: 2,
        active_effect_count: 1,
    });

    assert_eq!(session.state, PlayState::WaitingForPlayerLoaded);
    assert_eq!(session.container_state_id, 0);
    assert_eq!(
        instructions,
        vec![
            PlayInstruction::Login(login),
            PlayInstruction::ChangeDifficulty {
                difficulty: GameDifficulty::Hard,
                locked: true,
            },
            PlayInstruction::PlayerAbilities(abilities),
            PlayInstruction::SetHeldSlot(ClientboundSetHeldSlotPacket { slot: 3 }),
            PlayInstruction::UpdateRecipes,
            PlayInstruction::UpdatePermissionLevel(2),
            PlayInstruction::SendInitialRecipeBook,
            PlayInstruction::UpdateScoreboard,
            PlayInstruction::TeleportToSpawn { teleport_id: 0 },
            PlayInstruction::ServerStatus,
            PlayInstruction::PlayerInfoUpdate {
                existing_players: 2,
            },
            PlayInstruction::BroadcastSelfPlayerInfo,
            PlayInstruction::SendLevelInfo,
            PlayInstruction::AddPlayerToLevel,
            PlayInstruction::BossEventsOnConnect,
            PlayInstruction::ActiveEffects { count: 1 },
            PlayInstruction::InitInventoryMenu,
        ]
    );
}
