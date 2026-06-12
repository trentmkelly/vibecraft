use super::*;
use crate::network::codec::cursor;

const SERVERBOUND_ENTITY_TAG_QUERY_JAVA: &str = include_str!(
    "../../../../../decompiled-server-26.1.2/net/minecraft/network/protocol/game/ServerboundEntityTagQueryPacket.java"
);

#[test]
fn serverbound_entity_tag_query_packet_matches_java_codec() {
    for sentinel in [
        "Packet.codec(",
        "private final int transactionId;",
        "private final int entityId;",
        "this.transactionId = input.readVarInt();",
        "this.entityId = input.readVarInt();",
        "output.writeVarInt(this.transactionId);",
        "output.writeVarInt(this.entityId);",
        "return GamePacketTypes.SERVERBOUND_ENTITY_TAG_QUERY;",
        "listener.handleEntityTagQuery(this);",
        "return this.transactionId;",
        "return this.entityId;",
    ] {
        assert!(
            SERVERBOUND_ENTITY_TAG_QUERY_JAVA.contains(sentinel),
            "missing ServerboundEntityTagQueryPacket sentinel {sentinel}"
        );
    }

    assert_eq!(SERVERBOUND_ENTITY_TAG_QUERY_PACKET_ID, 25);
    let registry = PlayProtocolRegistry::new();
    assert_eq!(
        registry.serverbound_name(SERVERBOUND_ENTITY_TAG_QUERY_PACKET_ID),
        Some("entity_tag_query")
    );

    let packet = ServerboundEntityTagQueryPacket {
        transaction_id: 300,
        entity_id: 12_345,
    };
    let mut payload = Vec::new();
    packet.write(&mut payload).unwrap();
    assert_eq!(payload, vec![0xac, 0x02, 0xb9, 0x60]);
    assert_eq!(
        ServerboundEntityTagQueryPacket::read(&mut cursor(payload.clone())).unwrap(),
        packet
    );

    let mut session = PlaySession::new(1, 0);
    assert_eq!(
        session.handle_decoded(super::decoded(
            SERVERBOUND_ENTITY_TAG_QUERY_PACKET_ID,
            payload
        )),
        DispatchOutcome::Handled
    );
    assert_eq!(session.last_entity_tag_query, Some(packet));
}

#[test]
fn serverbound_entity_tag_query_packet_rejects_malformed_payloads() {
    assert!(ServerboundEntityTagQueryPacket::read(&mut cursor(Vec::new())).is_err());
    assert!(ServerboundEntityTagQueryPacket::read(&mut cursor(vec![1])).is_err());

    let mut session = PlaySession::new(1, 0);
    assert!(matches!(
        session.handle_decoded(super::decoded(
            SERVERBOUND_ENTITY_TAG_QUERY_PACKET_ID,
            vec![1]
        )),
        DispatchOutcome::Disconnect(reason) if reason.contains("bad entity tag query packet")
    ));
}
