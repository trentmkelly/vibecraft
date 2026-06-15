use super::*;

const SERVERBOUND_PICK_ITEM_FROM_BLOCK_JAVA: &str = vibecraft_java_source!("/net/minecraft/network/protocol/game/ServerboundPickItemFromBlockPacket.java");
const SERVERBOUND_PICK_ITEM_FROM_ENTITY_JAVA: &str = vibecraft_java_source!("/net/minecraft/network/protocol/game/ServerboundPickItemFromEntityPacket.java");
const SERVERBOUND_RENAME_ITEM_JAVA: &str = vibecraft_java_source!("/net/minecraft/network/protocol/game/ServerboundRenameItemPacket.java");
const SERVERBOUND_SELECT_TRADE_JAVA: &str = vibecraft_java_source!("/net/minecraft/network/protocol/game/ServerboundSelectTradePacket.java");

#[test]
fn serverbound_pick_rename_and_trade_packets_match_java_sources() {
    assert_java_contains(
        SERVERBOUND_PICK_ITEM_FROM_BLOCK_JAVA,
        &[
            "BlockPos.STREAM_CODEC",
            "ServerboundPickItemFromBlockPacket::pos",
            "ByteBufCodecs.BOOL",
            "ServerboundPickItemFromBlockPacket::includeData",
            "return GamePacketTypes.SERVERBOUND_PICK_ITEM_FROM_BLOCK;",
            "listener.handlePickItemFromBlock(this);",
        ],
    );
    assert_java_contains(
        SERVERBOUND_PICK_ITEM_FROM_ENTITY_JAVA,
        &[
            "ByteBufCodecs.VAR_INT",
            "ServerboundPickItemFromEntityPacket::id",
            "ByteBufCodecs.BOOL",
            "ServerboundPickItemFromEntityPacket::includeData",
            "return GamePacketTypes.SERVERBOUND_PICK_ITEM_FROM_ENTITY;",
            "listener.handlePickItemFromEntity(this);",
        ],
    );
    assert_java_contains(
        SERVERBOUND_RENAME_ITEM_JAVA,
        &[
            "Packet.codec(",
            "this.name = input.readUtf();",
            "output.writeUtf(this.name);",
            "return GamePacketTypes.SERVERBOUND_RENAME_ITEM;",
            "listener.handleRenameItem(this);",
            "public String getName()",
        ],
    );
    assert_java_contains(
        SERVERBOUND_SELECT_TRADE_JAVA,
        &[
            "Packet.codec(",
            "this.item = input.readVarInt();",
            "output.writeVarInt(this.item);",
            "return GamePacketTypes.SERVERBOUND_SELECT_TRADE;",
            "listener.handleSelectTrade(this);",
            "public int getItem()",
        ],
    );

    let registry = PlayProtocolRegistry::new();
    assert_packet_registry_names(&registry);

    let mut session = PlaySession::new(1, 0);
    session.state = PlayState::Playing;
    assert_pick_item_from_block_codec_and_dispatch(&mut session);
    assert_pick_item_from_entity_codec_and_dispatch(&mut session);
    assert_rename_item_codec_and_dispatch(&mut session);
    assert_select_trade_codec_and_dispatch(&mut session);
}

fn assert_packet_registry_names(registry: &PlayProtocolRegistry) {
    assert_eq!(SERVERBOUND_PICK_ITEM_FROM_BLOCK_PACKET_ID, 36);
    assert_eq!(
        registry.serverbound_name(SERVERBOUND_PICK_ITEM_FROM_BLOCK_PACKET_ID),
        Some("pick_item_from_block")
    );
    assert_eq!(SERVERBOUND_PICK_ITEM_FROM_ENTITY_PACKET_ID, 37);
    assert_eq!(
        registry.serverbound_name(SERVERBOUND_PICK_ITEM_FROM_ENTITY_PACKET_ID),
        Some("pick_item_from_entity")
    );
    assert_eq!(SERVERBOUND_RENAME_ITEM_PACKET_ID, 48);
    assert_eq!(
        registry.serverbound_name(SERVERBOUND_RENAME_ITEM_PACKET_ID),
        Some("rename_item")
    );
    assert_eq!(SERVERBOUND_SELECT_TRADE_PACKET_ID, 51);
    assert_eq!(
        registry.serverbound_name(SERVERBOUND_SELECT_TRADE_PACKET_ID),
        Some("select_trade")
    );
}

fn assert_pick_item_from_block_codec_and_dispatch(session: &mut PlaySession) {
    let packet = ServerboundPickItemFromBlockPacket {
        x: -12,
        y: 64,
        z: 34,
        include_data: true,
    };
    let mut payload = Vec::new();
    packet.write(&mut payload).unwrap();
    assert_eq!(payload.len(), 9);
    assert_eq!(payload[8], 1);
    assert_eq!(
        ServerboundPickItemFromBlockPacket::read(&mut cursor(payload.clone())).unwrap(),
        packet
    );
    assert_eq!(
        session.handle_decoded(decoded(
            SERVERBOUND_PICK_ITEM_FROM_BLOCK_PACKET_ID,
            payload
        )),
        DispatchOutcome::Handled
    );
    assert_eq!(session.last_pick_item_from_block, Some(packet));
}

fn assert_pick_item_from_entity_codec_and_dispatch(session: &mut PlaySession) {
    let packet = ServerboundPickItemFromEntityPacket {
        entity_id: 128,
        include_data: false,
    };
    let mut payload = Vec::new();
    packet.write(&mut payload).unwrap();
    assert_eq!(payload, vec![0x80, 0x01, 0]);
    assert_eq!(
        ServerboundPickItemFromEntityPacket::read(&mut cursor(payload.clone())).unwrap(),
        packet
    );
    assert_eq!(
        session.handle_decoded(decoded(
            SERVERBOUND_PICK_ITEM_FROM_ENTITY_PACKET_ID,
            payload
        )),
        DispatchOutcome::Handled
    );
    assert_eq!(session.last_pick_item_from_entity, Some(packet));
}

fn assert_rename_item_codec_and_dispatch(session: &mut PlaySession) {
    let packet = ServerboundRenameItemPacket {
        name: "Sharp Thing".to_string(),
    };
    let mut payload = Vec::new();
    packet.write(&mut payload).unwrap();
    assert_eq!(
        ServerboundRenameItemPacket::read(&mut cursor(payload.clone())).unwrap(),
        packet
    );
    assert_eq!(
        session.handle_decoded(decoded(SERVERBOUND_RENAME_ITEM_PACKET_ID, payload)),
        DispatchOutcome::Handled
    );
    assert_eq!(session.last_rename_item, Some(packet));
}

fn assert_select_trade_codec_and_dispatch(session: &mut PlaySession) {
    let packet = ServerboundSelectTradePacket { item: 128 };
    let mut payload = Vec::new();
    packet.write(&mut payload).unwrap();
    assert_eq!(payload, vec![0x80, 0x01]);
    assert_eq!(
        ServerboundSelectTradePacket::read(&mut cursor(payload.clone())).unwrap(),
        packet
    );
    assert_eq!(
        session.handle_decoded(decoded(SERVERBOUND_SELECT_TRADE_PACKET_ID, payload)),
        DispatchOutcome::Handled
    );
    assert_eq!(session.last_select_trade, Some(packet));
}

fn assert_java_contains(source: &str, sentinels: &[&str]) {
    for sentinel in sentinels {
        assert!(
            source.contains(sentinel),
            "missing Java source sentinel: {sentinel}"
        );
    }
}
