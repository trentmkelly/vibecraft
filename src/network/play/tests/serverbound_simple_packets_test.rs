use super::*;

const SERVERBOUND_ACCEPT_TELEPORTATION_JAVA: &str = include_str!(
    "../../../../../decompiled-server-26.1.2/net/minecraft/network/protocol/game/ServerboundAcceptTeleportationPacket.java"
);
const SERVERBOUND_ATTACK_JAVA: &str = include_str!(
    "../../../../../decompiled-server-26.1.2/net/minecraft/network/protocol/game/ServerboundAttackPacket.java"
);
const SERVERBOUND_CHANGE_DIFFICULTY_JAVA: &str = include_str!(
    "../../../../../decompiled-server-26.1.2/net/minecraft/network/protocol/game/ServerboundChangeDifficultyPacket.java"
);
const SERVERBOUND_CHAT_ACK_JAVA: &str = include_str!(
    "../../../../../decompiled-server-26.1.2/net/minecraft/network/protocol/game/ServerboundChatAckPacket.java"
);
const SERVERBOUND_CHUNK_BATCH_RECEIVED_JAVA: &str = include_str!(
    "../../../../../decompiled-server-26.1.2/net/minecraft/network/protocol/game/ServerboundChunkBatchReceivedPacket.java"
);
const SERVERBOUND_CLIENT_COMMAND_JAVA: &str = include_str!(
    "../../../../../decompiled-server-26.1.2/net/minecraft/network/protocol/game/ServerboundClientCommandPacket.java"
);
const SERVERBOUND_CLIENT_TICK_END_JAVA: &str = include_str!(
    "../../../../../decompiled-server-26.1.2/net/minecraft/network/protocol/game/ServerboundClientTickEndPacket.java"
);

#[test]
fn serverbound_simple_packets_match_java_codecs() {
    assert_accept_teleportation();
    assert_attack();
    assert_change_difficulty();
    assert_chat_ack();
    assert_chunk_batch_received();
    assert_client_command();
    assert_client_tick_end();
}

fn assert_accept_teleportation() {
    assert_java_contains(
        SERVERBOUND_ACCEPT_TELEPORTATION_JAVA,
        &[
            "this.id = input.readVarInt();",
            "output.writeVarInt(this.id);",
            "return GamePacketTypes.SERVERBOUND_ACCEPT_TELEPORTATION;",
            "listener.handleAcceptTeleportPacket(this);",
        ],
    );
    assert_registry(0, "accept_teleportation");

    let packet = ServerboundAcceptTeleportationPacket { teleport_id: 300 };
    let mut payload = Vec::new();
    packet.write(&mut payload).unwrap();
    assert_eq!(payload, vec![0xac, 0x02]);
    assert_eq!(
        ServerboundAcceptTeleportationPacket::read(&mut cursor(payload)).unwrap(),
        packet
    );
}

fn assert_attack() {
    assert_java_contains(
        SERVERBOUND_ATTACK_JAVA,
        &[
            "ByteBufCodecs.VAR_INT",
            "return GamePacketTypes.SERVERBOUND_ATTACK;",
            "listener.handleAttack(this);",
        ],
    );
    assert_registry(1, "attack");

    let packet = ServerboundAttackPacket { entity_id: 128 };
    let mut payload = Vec::new();
    packet.write(&mut payload).unwrap();
    assert_eq!(payload, vec![0x80, 0x01]);
    assert_eq!(ServerboundAttackPacket::read(&mut cursor(payload)).unwrap(), packet);
}

fn assert_change_difficulty() {
    assert_java_contains(
        SERVERBOUND_CHANGE_DIFFICULTY_JAVA,
        &[
            "Difficulty.STREAM_CODEC",
            "return GamePacketTypes.SERVERBOUND_CHANGE_DIFFICULTY;",
            "listener.handleChangeDifficulty(this);",
        ],
    );
    assert_registry(4, "change_difficulty");

    let packet = ServerboundChangeDifficultyPacket {
        difficulty: GameDifficulty::Hard,
    };
    let mut payload = Vec::new();
    packet.write(&mut payload).unwrap();
    assert_eq!(payload, vec![3]);
    assert_eq!(
        ServerboundChangeDifficultyPacket::read(&mut cursor(payload)).unwrap(),
        packet
    );
}

fn assert_chat_ack() {
    assert_java_contains(
        SERVERBOUND_CHAT_ACK_JAVA,
        &[
            "this(input.readVarInt());",
            "output.writeVarInt(this.offset);",
            "return GamePacketTypes.SERVERBOUND_CHAT_ACK;",
            "listener.handleChatAck(this);",
        ],
    );
    assert_registry(6, "chat_ack");

    let packet = ServerboundChatAckPacket { offset: 128 };
    let mut payload = Vec::new();
    packet.write(&mut payload).unwrap();
    assert_eq!(payload, vec![0x80, 0x01]);
    assert_eq!(ServerboundChatAckPacket::read(&mut cursor(payload)).unwrap(), packet);
}

fn assert_chunk_batch_received() {
    assert_java_contains(
        SERVERBOUND_CHUNK_BATCH_RECEIVED_JAVA,
        &[
            "this(input.readFloat());",
            "output.writeFloat(this.desiredChunksPerTick);",
            "return GamePacketTypes.SERVERBOUND_CHUNK_BATCH_RECEIVED;",
            "listener.handleChunkBatchReceived(this);",
        ],
    );
    assert_registry(11, "chunk_batch_received");

    let packet = ServerboundChunkBatchReceivedPacket {
        desired_chunks_per_tick: 12.5,
    };
    let mut payload = Vec::new();
    packet.write(&mut payload).unwrap();
    assert_eq!(payload, 12.5_f32.to_be_bytes());
    assert_eq!(
        ServerboundChunkBatchReceivedPacket::read(&mut cursor(payload)).unwrap(),
        packet
    );
}

fn assert_client_command() {
    assert_java_contains(
        SERVERBOUND_CLIENT_COMMAND_JAVA,
        &[
            "this.action = input.readEnum(ServerboundClientCommandPacket.Action.class);",
            "output.writeEnum(this.action);",
            "return GamePacketTypes.SERVERBOUND_CLIENT_COMMAND;",
            "listener.handleClientCommand(this);",
            "REQUEST_GAMERULE_VALUES",
        ],
    );
    assert_registry(12, "client_command");

    let packet = ServerboundClientCommandPacket {
        action: ServerboundClientCommandAction::RequestGameruleValues,
    };
    let mut payload = Vec::new();
    packet.write(&mut payload).unwrap();
    assert_eq!(payload, vec![2]);
    assert_eq!(
        ServerboundClientCommandPacket::read(&mut cursor(payload)).unwrap(),
        packet
    );
}

fn assert_client_tick_end() {
    assert_java_contains(
        SERVERBOUND_CLIENT_TICK_END_JAVA,
        &[
            "public static final ServerboundClientTickEndPacket INSTANCE",
            "StreamCodec.unit(INSTANCE)",
            "return GamePacketTypes.SERVERBOUND_CLIENT_TICK_END;",
            "listener.handleClientTickEnd(this);",
        ],
    );
    assert_registry(13, "client_tick_end");

    let mut payload = Vec::new();
    ServerboundClientTickEndPacket.write(&mut payload).unwrap();
    assert!(payload.is_empty());
    assert_eq!(
        ServerboundClientTickEndPacket::read(&mut cursor(payload)).unwrap(),
        ServerboundClientTickEndPacket
    );
}

fn assert_registry(packet_id: i32, name: &str) {
    let registry = PlayProtocolRegistry::new();
    assert_eq!(registry.serverbound_name(packet_id), Some(name));
}

fn assert_java_contains(source: &str, sentinels: &[&str]) {
    for sentinel in sentinels {
        assert!(
            source.contains(sentinel),
            "Java source missing sentinel: {sentinel}"
        );
    }
}
