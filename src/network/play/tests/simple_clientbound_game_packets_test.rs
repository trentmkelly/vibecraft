use super::*;

const CLIENTBOUND_BLOCK_CHANGED_ACK_JAVA: &str = vibecraft_java_source!("/net/minecraft/network/protocol/game/ClientboundBlockChangedAckPacket.java");
const CLIENTBOUND_CHANGE_DIFFICULTY_JAVA: &str = vibecraft_java_source!("/net/minecraft/network/protocol/game/ClientboundChangeDifficultyPacket.java");
const CLIENTBOUND_CHUNK_BATCH_FINISHED_JAVA: &str = vibecraft_java_source!("/net/minecraft/network/protocol/game/ClientboundChunkBatchFinishedPacket.java");
const CLIENTBOUND_CHUNK_BATCH_START_JAVA: &str = vibecraft_java_source!("/net/minecraft/network/protocol/game/ClientboundChunkBatchStartPacket.java");
const CLIENTBOUND_CLEAR_TITLES_JAVA: &str = vibecraft_java_source!("/net/minecraft/network/protocol/game/ClientboundClearTitlesPacket.java");
const CLIENTBOUND_COMMAND_SUGGESTIONS_JAVA: &str = vibecraft_java_source!("/net/minecraft/network/protocol/game/ClientboundCommandSuggestionsPacket.java");
const CLIENTBOUND_CONTAINER_CLOSE_JAVA: &str = vibecraft_java_source!("/net/minecraft/network/protocol/game/ClientboundContainerClosePacket.java");
const CLIENTBOUND_CONTAINER_SET_CONTENT_JAVA: &str = vibecraft_java_source!("/net/minecraft/network/protocol/game/ClientboundContainerSetContentPacket.java");
const CLIENTBOUND_CONTAINER_SET_DATA_JAVA: &str = vibecraft_java_source!("/net/minecraft/network/protocol/game/ClientboundContainerSetDataPacket.java");
const CLIENTBOUND_CONTAINER_SET_SLOT_JAVA: &str = vibecraft_java_source!("/net/minecraft/network/protocol/game/ClientboundContainerSetSlotPacket.java");
const CLIENTBOUND_COOLDOWN_JAVA: &str = vibecraft_java_source!("/net/minecraft/network/protocol/game/ClientboundCooldownPacket.java");

#[test]
fn simple_clientbound_game_packet_codecs_match_java_sources() {
    assert_block_changed_ack();
    assert_change_difficulty();
    assert_chunk_batch_packets();
    assert_clear_titles();
    assert_container_packets();
    assert_cooldown();
    assert_command_suggestions();
}

fn assert_block_changed_ack() {
    assert_java_contains(
        CLIENTBOUND_BLOCK_CHANGED_ACK_JAVA,
        &[
            "this(input.readVarInt());",
            "output.writeVarInt(this.sequence);",
            "return GamePacketTypes.CLIENTBOUND_BLOCK_CHANGED_ACK;",
            "listener.handleBlockChangedAck(this);",
        ],
    );
    assert_registry(CLIENTBOUND_BLOCK_CHANGED_ACK_PACKET_ID, "block_changed_ack");

    let packet = ClientboundBlockChangedAckPacket { sequence: 300 };
    let mut payload = Vec::new();
    packet.write(&mut payload).unwrap();
    assert_eq!(payload, vec![0xac, 0x02]);
    assert_eq!(
        ClientboundBlockChangedAckPacket::read(&mut cursor(payload)).unwrap(),
        packet
    );
}

fn assert_change_difficulty() {
    assert_java_contains(
        CLIENTBOUND_CHANGE_DIFFICULTY_JAVA,
        &[
            "Difficulty.STREAM_CODEC",
            "ByteBufCodecs.BOOL",
            "return GamePacketTypes.CLIENTBOUND_CHANGE_DIFFICULTY;",
            "listener.handleChangeDifficulty(this);",
        ],
    );
    assert_registry(CLIENTBOUND_CHANGE_DIFFICULTY_PACKET_ID, "change_difficulty");

    let packet = ClientboundChangeDifficultyPacket {
        difficulty: GameDifficulty::Hard,
        locked: true,
    };
    let mut payload = Vec::new();
    packet.write(&mut payload).unwrap();
    assert_eq!(payload, vec![3, 1]);
    assert_eq!(
        ClientboundChangeDifficultyPacket::read(&mut cursor(payload)).unwrap(),
        packet
    );
}

fn assert_chunk_batch_packets() {
    assert_java_contains(
        CLIENTBOUND_CHUNK_BATCH_FINISHED_JAVA,
        &[
            "this(input.readVarInt());",
            "output.writeVarInt(this.batchSize);",
            "return GamePacketTypes.CLIENTBOUND_CHUNK_BATCH_FINISHED;",
            "listener.handleChunkBatchFinished(this);",
        ],
    );
    assert_java_contains(
        CLIENTBOUND_CHUNK_BATCH_START_JAVA,
        &[
            "public static final ClientboundChunkBatchStartPacket INSTANCE",
            "StreamCodec.unit(INSTANCE)",
            "return GamePacketTypes.CLIENTBOUND_CHUNK_BATCH_START;",
            "listener.handleChunkBatchStart(this);",
        ],
    );
    assert_registry(
        CLIENTBOUND_CHUNK_BATCH_FINISHED_PACKET_ID,
        "chunk_batch_finished",
    );
    assert_registry(CLIENTBOUND_CHUNK_BATCH_START_PACKET_ID, "chunk_batch_start");

    let finished = ClientboundChunkBatchFinishedPacket { batch_size: 300 };
    let mut payload = Vec::new();
    finished.write(&mut payload).unwrap();
    assert_eq!(payload, vec![0xac, 0x02]);
    assert_eq!(
        ClientboundChunkBatchFinishedPacket::read(&mut cursor(payload)).unwrap(),
        finished
    );

    let mut start_payload = Vec::new();
    ClientboundChunkBatchStartPacket
        .write(&mut start_payload)
        .unwrap();
    assert!(start_payload.is_empty());
    assert_eq!(
        ClientboundChunkBatchStartPacket::read(&mut cursor(start_payload)).unwrap(),
        ClientboundChunkBatchStartPacket
    );
}

fn assert_clear_titles() {
    assert_java_contains(
        CLIENTBOUND_CLEAR_TITLES_JAVA,
        &[
            "this.resetTimes = input.readBoolean();",
            "output.writeBoolean(this.resetTimes);",
            "return GamePacketTypes.CLIENTBOUND_CLEAR_TITLES;",
            "listener.handleTitlesClear(this);",
            "public boolean shouldResetTimes()",
        ],
    );
    assert_registry(CLIENTBOUND_CLEAR_TITLES_PACKET_ID, "clear_titles");

    let mut payload = Vec::new();
    ClientboundClearTitlesPacket { reset_times: true }
        .write(&mut payload)
        .unwrap();
    assert_eq!(payload, vec![1]);
}

fn assert_container_packets() {
    assert_java_contains(
        CLIENTBOUND_CONTAINER_CLOSE_JAVA,
        &[
            "this.containerId = input.readContainerId();",
            "output.writeContainerId(this.containerId);",
            "return GamePacketTypes.CLIENTBOUND_CONTAINER_CLOSE;",
            "listener.handleContainerClose(this);",
        ],
    );
    assert_java_contains(
        CLIENTBOUND_CONTAINER_SET_CONTENT_JAVA,
        &[
            "ByteBufCodecs.CONTAINER_ID",
            "ByteBufCodecs.VAR_INT",
            "ItemStack.OPTIONAL_LIST_STREAM_CODEC",
            "ItemStack.OPTIONAL_STREAM_CODEC",
            "listener.handleContainerContent(this);",
        ],
    );
    assert_java_contains(
        CLIENTBOUND_CONTAINER_SET_DATA_JAVA,
        &[
            "this.containerId = input.readContainerId();",
            "this.id = input.readShort();",
            "this.value = input.readShort();",
            "listener.handleContainerSetData(this);",
        ],
    );
    assert_java_contains(
        CLIENTBOUND_CONTAINER_SET_SLOT_JAVA,
        &[
            "this.containerId = input.readContainerId();",
            "this.stateId = input.readVarInt();",
            "this.slot = input.readShort();",
            "ItemStack.OPTIONAL_STREAM_CODEC.decode(input);",
            "listener.handleContainerSetSlot(this);",
        ],
    );
    assert_registry(CLIENTBOUND_CONTAINER_CLOSE_PACKET_ID, "container_close");
    assert_registry(CLIENTBOUND_CONTAINER_SET_CONTENT_PACKET_ID, "container_set_content");
    assert_registry(CLIENTBOUND_CONTAINER_SET_DATA_PACKET_ID, "container_set_data");
    assert_registry(CLIENTBOUND_CONTAINER_SET_SLOT_PACKET_ID, "container_set_slot");

    let mut close = Vec::new();
    ClientboundContainerClosePacket { container_id: 128 }
        .write(&mut close)
        .unwrap();
    assert_eq!(close, vec![0x80, 0x01]);

    let mut data = Vec::new();
    ClientboundContainerSetDataPacket {
        container_id: 2,
        id: 0x0102,
        value: -2,
    }
    .write(&mut data)
    .unwrap();
    assert_eq!(data, vec![2, 0x01, 0x02, 0xff, 0xfe]);
}

fn assert_cooldown() {
    assert_java_contains(
        CLIENTBOUND_COOLDOWN_JAVA,
        &[
            "Identifier.STREAM_CODEC",
            "ByteBufCodecs.VAR_INT",
            "return GamePacketTypes.CLIENTBOUND_COOLDOWN;",
            "listener.handleItemCooldown(this);",
        ],
    );
    assert_registry(CLIENTBOUND_COOLDOWN_PACKET_ID, "cooldown");

    let packet = ClientboundCooldownPacket {
        cooldown_group: Identifier::parse("minecraft:ender_pearl").unwrap(),
        duration: 300,
    };
    let mut payload = Vec::new();
    packet.write(&mut payload).unwrap();
    assert_eq!(
        payload,
        [
            vec![21],
            b"minecraft:ender_pearl".to_vec(),
            vec![0xac, 0x02]
        ]
        .concat()
    );
}

fn assert_command_suggestions() {
    assert_java_contains(
        CLIENTBOUND_COMMAND_SUGGESTIONS_JAVA,
        &[
            "ByteBufCodecs.VAR_INT",
            "ByteBufCodecs.STRING_UTF8",
            "ComponentSerialization.TRUSTED_OPTIONAL_STREAM_CODEC",
            "return GamePacketTypes.CLIENTBOUND_COMMAND_SUGGESTIONS;",
            "listener.handleCommandSuggestions(this);",
            "public Suggestions toSuggestions()",
        ],
    );
    assert_registry(
        CLIENTBOUND_COMMAND_SUGGESTIONS_PACKET_ID,
        "command_suggestions",
    );

    let mut payload = Vec::new();
    ClientboundCommandSuggestionsPacket {
        transaction_id: 4,
        start: 1,
        length: 2,
        suggestions: vec![CommandSuggestionEntry {
            text: "help".to_string(),
            tooltip: None,
        }],
    }
    .write(&mut payload)
    .unwrap();
    assert_eq!(payload, vec![4, 1, 2, 1, 4, b'h', b'e', b'l', b'p', 0]);
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
