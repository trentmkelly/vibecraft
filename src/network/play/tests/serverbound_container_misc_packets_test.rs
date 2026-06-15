use super::*;

const SERVERBOUND_CONTAINER_BUTTON_CLICK_JAVA: &str = vibecraft_java_source!("/net/minecraft/network/protocol/game/ServerboundContainerButtonClickPacket.java");
const SERVERBOUND_CONTAINER_CLOSE_JAVA: &str = vibecraft_java_source!("/net/minecraft/network/protocol/game/ServerboundContainerClosePacket.java");
const SERVERBOUND_JIGSAW_GENERATE_JAVA: &str = vibecraft_java_source!("/net/minecraft/network/protocol/game/ServerboundJigsawGeneratePacket.java");
const SERVERBOUND_LOCK_DIFFICULTY_JAVA: &str = vibecraft_java_source!("/net/minecraft/network/protocol/game/ServerboundLockDifficultyPacket.java");

#[test]
fn serverbound_container_button_close_jigsaw_and_lock_packets_match_java_codecs() {
    assert_java_contains(
        SERVERBOUND_CONTAINER_BUTTON_CLICK_JAVA,
        &[
            "ByteBufCodecs.CONTAINER_ID",
            "ServerboundContainerButtonClickPacket::containerId",
            "ByteBufCodecs.VAR_INT",
            "ServerboundContainerButtonClickPacket::buttonId",
            "return GamePacketTypes.SERVERBOUND_CONTAINER_BUTTON_CLICK;",
            "listener.handleContainerButtonClick(this);",
        ],
    );
    assert_java_contains(
        SERVERBOUND_CONTAINER_CLOSE_JAVA,
        &[
            "this.containerId = input.readContainerId();",
            "output.writeContainerId(this.containerId);",
            "return GamePacketTypes.SERVERBOUND_CONTAINER_CLOSE;",
            "listener.handleContainerClose(this);",
            "public int getContainerId()",
        ],
    );
    assert_java_contains(
        SERVERBOUND_JIGSAW_GENERATE_JAVA,
        &[
            "this.pos = input.readBlockPos();",
            "this.levels = input.readVarInt();",
            "this.keepJigsaws = input.readBoolean();",
            "output.writeBlockPos(this.pos);",
            "output.writeVarInt(this.levels);",
            "output.writeBoolean(this.keepJigsaws);",
            "return GamePacketTypes.SERVERBOUND_JIGSAW_GENERATE;",
            "listener.handleJigsawGenerate(this);",
        ],
    );
    assert_java_contains(
        SERVERBOUND_LOCK_DIFFICULTY_JAVA,
        &[
            "this.locked = input.readBoolean();",
            "output.writeBoolean(this.locked);",
            "return GamePacketTypes.SERVERBOUND_LOCK_DIFFICULTY;",
            "listener.handleLockDifficulty(this);",
            "public boolean isLocked()",
        ],
    );

    let registry = PlayProtocolRegistry::new();
    assert_packet_registry_names(&registry);

    let mut session = PlaySession::new(1, 0);
    session.state = PlayState::Playing;
    assert_container_button_click_codec_and_dispatch(&mut session);
    assert_container_close_codec_and_dispatch(&mut session);
    assert_jigsaw_generate_codec_and_dispatch(&mut session);
    assert_lock_difficulty_codec_and_dispatch(&mut session);
}

fn assert_packet_registry_names(registry: &PlayProtocolRegistry) {
    assert_eq!(SERVERBOUND_CONTAINER_BUTTON_CLICK_PACKET_ID, 17);
    assert_eq!(
        registry.serverbound_name(SERVERBOUND_CONTAINER_BUTTON_CLICK_PACKET_ID),
        Some("container_button_click")
    );
    assert_eq!(SERVERBOUND_CONTAINER_CLOSE_PACKET_ID, 19);
    assert_eq!(
        registry.serverbound_name(SERVERBOUND_CONTAINER_CLOSE_PACKET_ID),
        Some("container_close")
    );
    assert_eq!(SERVERBOUND_JIGSAW_GENERATE_PACKET_ID, 27);
    assert_eq!(
        registry.serverbound_name(SERVERBOUND_JIGSAW_GENERATE_PACKET_ID),
        Some("jigsaw_generate")
    );
    assert_eq!(SERVERBOUND_LOCK_DIFFICULTY_PACKET_ID, 29);
    assert_eq!(
        registry.serverbound_name(SERVERBOUND_LOCK_DIFFICULTY_PACKET_ID),
        Some("lock_difficulty")
    );
}

fn assert_container_button_click_codec_and_dispatch(session: &mut PlaySession) {
    let packet = ServerboundContainerButtonClickPacket {
        container_id: 128,
        button_id: 7,
    };
    let mut payload = Vec::new();
    packet.write(&mut payload).unwrap();
    assert_eq!(payload, vec![0x80, 0x01, 7]);
    assert_eq!(
        ServerboundContainerButtonClickPacket::read(&mut cursor(payload.clone())).unwrap(),
        packet
    );
    assert_eq!(
        session.handle_decoded(decoded(
            SERVERBOUND_CONTAINER_BUTTON_CLICK_PACKET_ID,
            payload
        )),
        DispatchOutcome::Handled
    );
    assert_eq!(session.last_container_button_click, Some(packet));
}

fn assert_container_close_codec_and_dispatch(session: &mut PlaySession) {
    let packet = ServerboundContainerClosePacket { container_id: 128 };
    let mut payload = Vec::new();
    packet.write(&mut payload).unwrap();
    assert_eq!(payload, vec![0x80, 0x01]);
    assert_eq!(
        ServerboundContainerClosePacket::read(&mut cursor(payload.clone())).unwrap(),
        packet
    );
    assert_eq!(
        session.handle_decoded(decoded(SERVERBOUND_CONTAINER_CLOSE_PACKET_ID, payload)),
        DispatchOutcome::Handled
    );
    assert_eq!(session.last_container_close, Some(packet));
}

fn assert_jigsaw_generate_codec_and_dispatch(session: &mut PlaySession) {
    let packet = ServerboundJigsawGeneratePacket {
        x: -12,
        y: 64,
        z: 34,
        levels: 7,
        keep_jigsaws: true,
    };
    let mut payload = Vec::new();
    packet.write(&mut payload).unwrap();
    assert_eq!(payload.len(), 10);
    assert_eq!(&payload[8..], &[7, 1]);
    assert_eq!(
        ServerboundJigsawGeneratePacket::read(&mut cursor(payload.clone())).unwrap(),
        packet
    );
    assert_eq!(
        session.handle_decoded(decoded(SERVERBOUND_JIGSAW_GENERATE_PACKET_ID, payload)),
        DispatchOutcome::Handled
    );
    assert_eq!(session.last_jigsaw_generate, Some(packet));
}

fn assert_lock_difficulty_codec_and_dispatch(session: &mut PlaySession) {
    let packet = ServerboundLockDifficultyPacket { locked: true };
    let mut payload = Vec::new();
    packet.write(&mut payload).unwrap();
    assert_eq!(payload, vec![1]);
    assert_eq!(
        ServerboundLockDifficultyPacket::read(&mut cursor(payload.clone())).unwrap(),
        packet
    );
    assert!(ServerboundLockDifficultyPacket::read(&mut cursor(Vec::<u8>::new())).is_err());
    assert!(ServerboundLockDifficultyPacket::read(&mut cursor(vec![1, 0])).is_err());
    assert_eq!(
        session.handle_decoded(decoded(SERVERBOUND_LOCK_DIFFICULTY_PACKET_ID, payload)),
        DispatchOutcome::Handled
    );
}

fn assert_java_contains(source: &str, sentinels: &[&str]) {
    for sentinel in sentinels {
        assert!(
            source.contains(sentinel),
            "Java source missing sentinel: {sentinel}"
        );
    }
}
