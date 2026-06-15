use super::*;
use crate::block_update::BlockPos;
use crate::network::codec::cursor;

const SERVERBOUND_BLOCK_ENTITY_TAG_QUERY_JAVA: &str = vibecraft_java_source!("/net/minecraft/network/protocol/game/ServerboundBlockEntityTagQueryPacket.java");

#[test]
fn serverbound_block_entity_tag_query_packet_matches_java_codec() {
    for sentinel in [
        "Packet.codec(",
        "private final int transactionId;",
        "private final BlockPos pos;",
        "this.transactionId = input.readVarInt();",
        "this.pos = input.readBlockPos();",
        "output.writeVarInt(this.transactionId);",
        "output.writeBlockPos(this.pos);",
        "return GamePacketTypes.SERVERBOUND_BLOCK_ENTITY_TAG_QUERY;",
        "listener.handleBlockEntityTagQuery(this);",
        "return this.transactionId;",
        "return this.pos;",
    ] {
        assert!(
            SERVERBOUND_BLOCK_ENTITY_TAG_QUERY_JAVA.contains(sentinel),
            "missing ServerboundBlockEntityTagQueryPacket sentinel {sentinel}"
        );
    }

    assert_eq!(SERVERBOUND_BLOCK_ENTITY_TAG_QUERY_PACKET_ID, 2);
    let registry = PlayProtocolRegistry::new();
    assert_eq!(
        registry.serverbound_name(SERVERBOUND_BLOCK_ENTITY_TAG_QUERY_PACKET_ID),
        Some("block_entity_tag_query")
    );

    let packet = ServerboundBlockEntityTagQueryPacket {
        transaction_id: 300,
        pos: BlockPos {
            x: -12,
            y: 70,
            z: 34,
        },
    };
    let mut payload = Vec::new();
    packet.write(&mut payload).unwrap();
    let mut expected = vec![0xac, 0x02];
    expected.extend(pack_block_position(packet.pos.x, packet.pos.y, packet.pos.z).to_be_bytes());
    assert_eq!(payload, expected);
    assert_eq!(
        ServerboundBlockEntityTagQueryPacket::read(&mut cursor(payload.clone())).unwrap(),
        packet
    );

    let mut session = PlaySession::new(1, 0);
    assert_eq!(
        session.handle_decoded(super::decoded(
            SERVERBOUND_BLOCK_ENTITY_TAG_QUERY_PACKET_ID,
            payload
        )),
        DispatchOutcome::Handled
    );
    assert_eq!(session.last_block_entity_tag_query, Some(packet));
}

#[test]
fn serverbound_block_entity_tag_query_packet_rejects_malformed_payloads() {
    assert!(ServerboundBlockEntityTagQueryPacket::read(&mut cursor(Vec::new())).is_err());
    assert!(ServerboundBlockEntityTagQueryPacket::read(&mut cursor(vec![1])).is_err());

    let mut session = PlaySession::new(1, 0);
    assert!(matches!(
        session.handle_decoded(super::decoded(
            SERVERBOUND_BLOCK_ENTITY_TAG_QUERY_PACKET_ID,
            vec![1]
        )),
        DispatchOutcome::Disconnect(reason)
            if reason.contains("bad block entity tag query packet")
    ));

    let packet = ServerboundBlockEntityTagQueryPacket {
        transaction_id: 1,
        pos: BlockPos { x: 0, y: 64, z: 0 },
    };
    let mut payload = Vec::new();
    packet.write(&mut payload).unwrap();
    payload.push(0);
    let mut session = PlaySession::new(1, 0);
    assert!(matches!(
        session.handle_decoded(super::decoded(
            SERVERBOUND_BLOCK_ENTITY_TAG_QUERY_PACKET_ID,
            payload
        )),
        DispatchOutcome::Disconnect(reason)
            if reason.contains("bad block entity tag query packet")
    ));
}
