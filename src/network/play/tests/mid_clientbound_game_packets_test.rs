use super::*;

const CLIENTBOUND_CUSTOM_CHAT_COMPLETIONS_JAVA: &str = vibecraft_java_source!("/net/minecraft/network/protocol/game/ClientboundCustomChatCompletionsPacket.java");
const CLIENTBOUND_DEBUG_SAMPLE_JAVA: &str = vibecraft_java_source!("/net/minecraft/network/protocol/game/ClientboundDebugSamplePacket.java");
const CLIENTBOUND_DELETE_CHAT_JAVA: &str = vibecraft_java_source!("/net/minecraft/network/protocol/game/ClientboundDeleteChatPacket.java");
const CLIENTBOUND_DISGUISED_CHAT_JAVA: &str = vibecraft_java_source!("/net/minecraft/network/protocol/game/ClientboundDisguisedChatPacket.java");
const CLIENTBOUND_ENTITY_EVENT_JAVA: &str = vibecraft_java_source!("/net/minecraft/network/protocol/game/ClientboundEntityEventPacket.java");
const CLIENTBOUND_ENTITY_POSITION_SYNC_JAVA: &str = vibecraft_java_source!("/net/minecraft/network/protocol/game/ClientboundEntityPositionSyncPacket.java");
const CLIENTBOUND_FORGET_LEVEL_CHUNK_JAVA: &str = vibecraft_java_source!("/net/minecraft/network/protocol/game/ClientboundForgetLevelChunkPacket.java");
const CLIENTBOUND_GAME_EVENT_JAVA: &str = vibecraft_java_source!("/net/minecraft/network/protocol/game/ClientboundGameEventPacket.java");
const CLIENTBOUND_GAME_RULE_VALUES_JAVA: &str = vibecraft_java_source!("/net/minecraft/network/protocol/game/ClientboundGameRuleValuesPacket.java");
const CLIENTBOUND_HURT_ANIMATION_JAVA: &str = vibecraft_java_source!("/net/minecraft/network/protocol/game/ClientboundHurtAnimationPacket.java");

#[test]
fn mid_clientbound_game_packet_codecs_match_java_sources() {
    assert_custom_chat_completions();
    assert_debug_sample();
    assert_delete_chat();
    assert_disguised_chat();
    assert_entity_event();
    assert_entity_position_sync();
    assert_forget_level_chunk();
    assert_game_event();
    assert_game_rule_values();
    assert_hurt_animation();
}

fn assert_custom_chat_completions() {
    assert_java_contains(
        CLIENTBOUND_CUSTOM_CHAT_COMPLETIONS_JAVA,
        &[
            "input.readEnum(ClientboundCustomChatCompletionsPacket.Action.class)",
            "input.readList(FriendlyByteBuf::readUtf)",
            "output.writeEnum(this.action);",
            "output.writeCollection(this.entries, FriendlyByteBuf::writeUtf);",
            "listener.handleCustomChatCompletions(this);",
        ],
    );
    assert_registry(
        CLIENTBOUND_CUSTOM_CHAT_COMPLETIONS_PACKET_ID,
        "custom_chat_completions",
    );

    let packet = ClientboundCustomChatCompletionsPacket {
        action: CustomChatCompletionsAction::Set,
        entries: vec!["home".to_string(), "spawn".to_string()],
    };
    let mut payload = Vec::new();
    packet.write(&mut payload).unwrap();
    assert_eq!(
        payload,
        vec![
            2, 2, 4, b'h', b'o', b'm', b'e', 5, b's', b'p', b'a', b'w', b'n',
        ]
    );
    assert_eq!(
        ClientboundCustomChatCompletionsPacket::read(&mut cursor(payload)).unwrap(),
        packet
    );
}

fn assert_debug_sample() {
    assert_java_contains(
        CLIENTBOUND_DEBUG_SAMPLE_JAVA,
        &[
            "input.readLongArray()",
            "input.readEnum(RemoteDebugSampleType.class)",
            "output.writeLongArray(this.sample);",
            "output.writeEnum(this.debugSampleType);",
            "listener.handleDebugSample(this);",
        ],
    );
    assert_registry(CLIENTBOUND_DEBUG_SAMPLE_PACKET_ID, "debug_sample");

    let mut payload = Vec::new();
    ClientboundDebugSamplePacket {
        sample: vec![10, -20],
        sample_type: RemoteDebugSampleType::TickTime,
    }
    .write(&mut payload)
    .unwrap();
    assert_eq!(payload[0], 2);
    assert_eq!(&payload[1..9], &10_i64.to_be_bytes());
    assert_eq!(&payload[9..17], &(-20_i64).to_be_bytes());
    assert_eq!(payload[17], 0);
}

fn assert_delete_chat() {
    assert_java_contains(
        CLIENTBOUND_DELETE_CHAT_JAVA,
        &[
            "MessageSignature.Packed.read(input)",
            "MessageSignature.Packed.write(output, this.messageSignature);",
            "return GamePacketTypes.CLIENTBOUND_DELETE_CHAT;",
            "listener.handleDeleteChat(this);",
        ],
    );
    assert_registry(CLIENTBOUND_DELETE_CHAT_PACKET_ID, "delete_chat");

    let mut cached = Vec::new();
    ClientboundDeleteChatPacket {
        message_signature: PackedMessageSignature::CacheId(127),
    }
    .write(&mut cached)
    .unwrap();
    assert_eq!(cached, vec![0x80, 0x01]);
}

fn assert_disguised_chat() {
    assert_java_contains(
        CLIENTBOUND_DISGUISED_CHAT_JAVA,
        &[
            "ComponentSerialization.TRUSTED_STREAM_CODEC",
            "ChatType.Bound.STREAM_CODEC",
            "return GamePacketTypes.CLIENTBOUND_DISGUISED_CHAT;",
            "listener.handleDisguisedChat(this);",
            "public boolean isSkippable()",
        ],
    );
    assert_registry(CLIENTBOUND_DISGUISED_CHAT_PACKET_ID, "disguised_chat");

    let mut payload = Vec::new();
    ClientboundDisguisedChatPacket {
        message: text_component("Hello"),
        chat_type: ChatTypeBound {
            chat_type_id: 0,
            name: text_component("Steve"),
            target_name: None,
        },
    }
    .write(&mut payload)
    .unwrap();
    assert_eq!(&payload[..text_component_bytes("Hello").len()], text_component_bytes("Hello"));
    assert!(payload.ends_with(&[0]));
}

fn assert_entity_event() {
    assert_java_contains(
        CLIENTBOUND_ENTITY_EVENT_JAVA,
        &[
            "this.entityId = input.readInt();",
            "this.eventId = input.readByte();",
            "output.writeInt(this.entityId);",
            "output.writeByte(this.eventId);",
            "listener.handleEntityEvent(this);",
        ],
    );
    assert_registry(CLIENTBOUND_ENTITY_EVENT_PACKET_ID, "entity_event");

    let mut payload = Vec::new();
    ClientboundEntityEventPacket {
        entity_id: 300,
        event_id: 3,
    }
    .write(&mut payload)
    .unwrap();
    assert_eq!(payload, vec![0x00, 0x00, 0x01, 0x2c, 0x03]);
}

fn assert_entity_position_sync() {
    assert_java_contains(
        CLIENTBOUND_ENTITY_POSITION_SYNC_JAVA,
        &[
            "ByteBufCodecs.VAR_INT",
            "PositionMoveRotation.STREAM_CODEC",
            "ByteBufCodecs.BOOL",
            "return GamePacketTypes.CLIENTBOUND_ENTITY_POSITION_SYNC;",
            "listener.handleEntityPositionSync(this);",
        ],
    );
    assert_registry(
        CLIENTBOUND_ENTITY_POSITION_SYNC_PACKET_ID,
        "entity_position_sync",
    );

    let packet = ClientboundEntityPositionSyncPacket {
        id: 300,
        position: Vec3 {
            x: 1.25,
            y: 64.0,
            z: -2.5,
        },
        movement: Vec3 {
            x: 0.125,
            y: -0.25,
            z: 0.5,
        },
        y_rot: 90.0,
        x_rot: -30.0,
        on_ground: false,
    };
    let mut payload = Vec::new();
    packet.write(&mut payload).unwrap();
    assert_eq!(&payload[..2], &[0xac, 0x02]);
    assert_eq!(payload.last(), Some(&0));
}

fn assert_forget_level_chunk() {
    assert_java_contains(
        CLIENTBOUND_FORGET_LEVEL_CHUNK_JAVA,
        &[
            "this(input.readChunkPos());",
            "output.writeChunkPos(this.pos);",
            "return GamePacketTypes.CLIENTBOUND_FORGET_LEVEL_CHUNK;",
            "listener.handleForgetLevelChunk(this);",
        ],
    );
    assert_registry(CLIENTBOUND_FORGET_LEVEL_CHUNK_PACKET_ID, "forget_level_chunk");

    let packet = ClientboundForgetLevelChunkPacket {
        pos: ChunkPos { x: -2, z: 3 },
    };
    let mut payload = Vec::new();
    packet.write(&mut payload).unwrap();
    assert_eq!(payload, 0x0000_0003_ffff_fffe_i64.to_be_bytes());
    assert_eq!(
        ClientboundForgetLevelChunkPacket::read(&mut cursor(payload.to_vec())).unwrap(),
        packet
    );
}

fn assert_game_event() {
    assert_java_contains(
        CLIENTBOUND_GAME_EVENT_JAVA,
        &[
            "input.readUnsignedByte()",
            "this.param = input.readFloat();",
            "output.writeByte(this.event.id);",
            "output.writeFloat(this.param);",
            "listener.handleGameEvent(this);",
            "LEVEL_CHUNKS_LOAD_START = new ClientboundGameEventPacket.Type(13)",
        ],
    );
    assert_registry(CLIENTBOUND_GAME_EVENT_PACKET_ID, "game_event");

    let packet = ClientboundGameEventPacket {
        event: ClientboundGameEventType::RainLevelChange,
        param: 0.5,
    };
    let mut payload = Vec::new();
    packet.write(&mut payload).unwrap();
    assert_eq!(payload[0], 7);
    assert_eq!(&payload[1..], &0.5_f32.to_be_bytes());
    assert_eq!(
        ClientboundGameEventPacket::read(&mut cursor(payload)).unwrap(),
        packet
    );
}

fn assert_game_rule_values() {
    assert_java_contains(
        CLIENTBOUND_GAME_RULE_VALUES_JAVA,
        &[
            "ByteBufCodecs.map(",
            "ResourceKey.streamCodec(Registries.GAME_RULE)",
            "ByteBufCodecs.STRING_UTF8",
            "return GamePacketTypes.CLIENTBOUND_GAME_RULE_VALUES;",
            "listener.handleGameRuleValues(this);",
        ],
    );
    assert_registry(CLIENTBOUND_GAME_RULE_VALUES_PACKET_ID, "game_rule_values");

    let mut payload = Vec::new();
    ClientboundGameRuleValuesPacket {
        values: BTreeMap::from([(
            Identifier::parse("minecraft:keep_inventory").unwrap(),
            "true".to_string(),
        )]),
    }
    .write(&mut payload)
    .unwrap();
    assert_eq!(payload[0], 1);
    assert_eq!(
        read_identifier(&mut cursor(payload[1..].to_vec())).unwrap(),
        Identifier::parse("minecraft:keep_inventory").unwrap()
    );
}

fn assert_hurt_animation() {
    assert_java_contains(
        CLIENTBOUND_HURT_ANIMATION_JAVA,
        &[
            "this(input.readVarInt(), input.readFloat());",
            "output.writeVarInt(this.id);",
            "output.writeFloat(this.yaw);",
            "return GamePacketTypes.CLIENTBOUND_HURT_ANIMATION;",
            "listener.handleHurtAnimation(this);",
        ],
    );
    assert_registry(CLIENTBOUND_HURT_ANIMATION_PACKET_ID, "hurt_animation");

    let packet = ClientboundHurtAnimationPacket { id: 300, yaw: -30.0 };
    let mut payload = Vec::new();
    packet.write(&mut payload).unwrap();
    assert_eq!(&payload[..2], &[0xac, 0x02]);
    assert_eq!(&payload[2..], &(-30.0_f32).to_be_bytes());
    assert_eq!(
        ClientboundHurtAnimationPacket::read(&mut cursor(payload)).unwrap(),
        packet
    );
}

fn text_component(text: &str) -> Tag {
    Tag::Compound(vec![("text".to_string(), Tag::String(text.to_string()))])
}

fn text_component_bytes(text: &str) -> Vec<u8> {
    [
        vec![10, 8, 0, 4],
        b"text".to_vec(),
        (text.len() as u16).to_be_bytes().to_vec(),
        text.as_bytes().to_vec(),
        vec![0],
    ]
    .concat()
}

fn assert_registry(packet_id: i32, name: &str) {
    let registry = PlayProtocolRegistry::new();
    assert_eq!(registry.clientbound_name(packet_id), Some(name));
}

fn assert_java_contains(source: &str, sentinels: &[&str]) {
    for sentinel in sentinels {
        assert!(
            source.contains(sentinel),
            "missing Java source sentinel {sentinel}"
        );
    }
}
