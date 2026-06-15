use super::*;

const SERVERBOUND_SET_CARRIED_ITEM_JAVA: &str = vibecraft_java_source!("/net/minecraft/network/protocol/game/ServerboundSetCarriedItemPacket.java");
const SERVERBOUND_SWING_JAVA: &str = vibecraft_java_source!("/net/minecraft/network/protocol/game/ServerboundSwingPacket.java");
const SERVERBOUND_USE_ITEM_JAVA: &str = vibecraft_java_source!("/net/minecraft/network/protocol/game/ServerboundUseItemPacket.java");
const SERVERBOUND_USE_ITEM_ON_JAVA: &str = vibecraft_java_source!("/net/minecraft/network/protocol/game/ServerboundUseItemOnPacket.java");

#[test]
fn serverbound_hand_and_item_packets_match_java_sources() {
    assert_java_contains(
        SERVERBOUND_SET_CARRIED_ITEM_JAVA,
        &[
            "Packet.codec(",
            "this.slot = input.readShort();",
            "output.writeShort(this.slot);",
            "return GamePacketTypes.SERVERBOUND_SET_CARRIED_ITEM;",
            "listener.handleSetCarriedItem(this);",
            "public int getSlot()",
        ],
    );
    assert_java_contains(
        SERVERBOUND_SWING_JAVA,
        &[
            "Packet.codec(",
            "this.hand = input.readEnum(InteractionHand.class);",
            "output.writeEnum(this.hand);",
            "return GamePacketTypes.SERVERBOUND_SWING;",
            "listener.handleAnimate(this);",
            "public InteractionHand getHand()",
        ],
    );
    assert_java_contains(
        SERVERBOUND_USE_ITEM_JAVA,
        &[
            "this.hand = input.readEnum(InteractionHand.class);",
            "this.sequence = input.readVarInt();",
            "this.yRot = input.readFloat();",
            "this.xRot = input.readFloat();",
            "output.writeEnum(this.hand);",
            "output.writeVarInt(this.sequence);",
            "output.writeFloat(this.yRot);",
            "output.writeFloat(this.xRot);",
            "return GamePacketTypes.SERVERBOUND_USE_ITEM;",
            "listener.handleUseItem(this);",
        ],
    );
    assert_java_contains(
        SERVERBOUND_USE_ITEM_ON_JAVA,
        &[
            "this.hand = input.readEnum(InteractionHand.class);",
            "this.blockHit = input.readBlockHitResult();",
            "this.sequence = input.readVarInt();",
            "output.writeEnum(this.hand);",
            "output.writeBlockHitResult(this.blockHit);",
            "output.writeVarInt(this.sequence);",
            "return GamePacketTypes.SERVERBOUND_USE_ITEM_ON;",
            "listener.handleUseItemOn(this);",
        ],
    );

    let registry = PlayProtocolRegistry::new();
    assert_packet_registry_names(&registry);

    let mut session = PlaySession::new(1, 0);
    session.state = PlayState::Playing;
    assert_set_carried_item_codec_and_dispatch(&mut session);
    assert_swing_codec_and_dispatch(&mut session);
    assert_use_item_codec_and_dispatch(&mut session);
    assert_use_item_on_codec_and_dispatch(&mut session);
}

fn assert_packet_registry_names(registry: &PlayProtocolRegistry) {
    assert_eq!(SERVERBOUND_SET_CARRIED_ITEM_PACKET_ID, 53);
    assert_eq!(
        registry.serverbound_name(SERVERBOUND_SET_CARRIED_ITEM_PACKET_ID),
        Some("set_carried_item")
    );
    assert_eq!(SERVERBOUND_SWING_PACKET_ID, 63);
    assert_eq!(
        registry.serverbound_name(SERVERBOUND_SWING_PACKET_ID),
        Some("swing")
    );
    assert_eq!(SERVERBOUND_USE_ITEM_ON_PACKET_ID, 66);
    assert_eq!(
        registry.serverbound_name(SERVERBOUND_USE_ITEM_ON_PACKET_ID),
        Some("use_item_on")
    );
    assert_eq!(SERVERBOUND_USE_ITEM_PACKET_ID, 67);
    assert_eq!(
        registry.serverbound_name(SERVERBOUND_USE_ITEM_PACKET_ID),
        Some("use_item")
    );
}

fn assert_set_carried_item_codec_and_dispatch(session: &mut PlaySession) {
    let packet = ServerboundSetCarriedItemPacket { slot: 8 };
    let mut payload = Vec::new();
    packet.write(&mut payload).unwrap();
    assert_eq!(payload, vec![0, 8]);
    assert_eq!(
        ServerboundSetCarriedItemPacket::read(&mut cursor(payload.clone())).unwrap(),
        packet
    );
    assert!(ServerboundSetCarriedItemPacket::read(&mut cursor(vec![0, 8, 0])).is_err());
    assert_eq!(
        session.handle_decoded(decoded(SERVERBOUND_SET_CARRIED_ITEM_PACKET_ID, payload)),
        DispatchOutcome::Handled
    );
    assert_eq!(session.selected_slot, 8);
}

fn assert_swing_codec_and_dispatch(session: &mut PlaySession) {
    let packet = ServerboundSwingPacket {
        hand: ServerboundSwingHand::OffHand,
    };
    let mut payload = Vec::new();
    packet.write(&mut payload).unwrap();
    assert_eq!(payload, vec![1]);
    assert_eq!(
        ServerboundSwingPacket::read(&mut cursor(payload.clone())).unwrap(),
        packet
    );
    assert_eq!(
        session.handle_decoded(decoded(SERVERBOUND_SWING_PACKET_ID, payload)),
        DispatchOutcome::Handled
    );
}

fn assert_use_item_codec_and_dispatch(session: &mut PlaySession) {
    let packet = ServerboundUseItemPacket {
        hand: ServerboundSwingHand::OffHand,
        sequence: 300,
        y_rot: 45.0,
        x_rot: -10.5,
    };
    let mut payload = Vec::new();
    packet.write(&mut payload).unwrap();
    assert_eq!(
        payload,
        vec![1, 0xac, 0x02, 0x42, 0x34, 0x00, 0x00, 0xc1, 0x28, 0x00, 0x00]
    );
    assert_eq!(
        ServerboundUseItemPacket::read(&mut cursor(payload.clone())).unwrap(),
        packet
    );
    assert_eq!(
        session.handle_decoded(decoded(SERVERBOUND_USE_ITEM_PACKET_ID, payload)),
        DispatchOutcome::Handled
    );
    assert_eq!(session.last_use_item, Some(packet));
}

fn assert_use_item_on_codec_and_dispatch(session: &mut PlaySession) {
    let packet = ServerboundUseItemOnPacket {
        hand: ServerboundSwingHand::MainHand,
        block_hit: BlockHitResultPacketData {
            x: -12,
            y: 64,
            z: 34,
            direction: Direction3d::Up,
            click_x: 0.25,
            click_y: 0.5,
            click_z: 0.75,
            inside: true,
            world_border_hit: false,
        },
        sequence: 301,
    };
    let mut payload = Vec::new();
    packet.write(&mut payload).unwrap();
    assert_eq!(
        payload,
        vec![
            0, 0xff, 0xff, 0xfd, 0x00, 0x00, 0x02, 0x20, 0x40, 1, 0x3e, 0x80, 0x00, 0x00, 0x3f,
            0x00, 0x00, 0x00, 0x3f, 0x40, 0x00, 0x00, 1, 0, 0xad, 0x02
        ]
    );
    assert_eq!(
        ServerboundUseItemOnPacket::read(&mut cursor(payload.clone())).unwrap(),
        packet
    );
    assert_eq!(
        session.handle_decoded(decoded(SERVERBOUND_USE_ITEM_ON_PACKET_ID, payload)),
        DispatchOutcome::Handled
    );
    assert_eq!(session.last_use_item_on, Some(packet));
}

fn assert_java_contains(source: &str, sentinels: &[&str]) {
    for sentinel in sentinels {
        assert!(
            source.contains(sentinel),
            "missing Java source sentinel: {sentinel}"
        );
    }
}
