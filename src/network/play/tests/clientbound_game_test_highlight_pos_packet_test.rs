use super::*;
use crate::block_update::BlockPos;

const CLIENTBOUND_GAME_TEST_HIGHLIGHT_POS_JAVA: &str = include_str!(
    "../../../../../decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundGameTestHighlightPosPacket.java"
);

#[test]
fn clientbound_game_test_highlight_pos_packet_matches_java_codec() {
    for sentinel in [
        "ClientboundGameTestHighlightPosPacket(BlockPos absolutePos, BlockPos relativePos)",
        "BlockPos.STREAM_CODEC",
        "ClientboundGameTestHighlightPosPacket::absolutePos",
        "ClientboundGameTestHighlightPosPacket::relativePos",
        "return GamePacketTypes.CLIENTBOUND_GAME_TEST_HIGHLIGHT_POS;",
        "listener.handleGameTestHighlightPos(this);",
    ] {
        assert!(
            CLIENTBOUND_GAME_TEST_HIGHLIGHT_POS_JAVA.contains(sentinel),
            "missing ClientboundGameTestHighlightPosPacket sentinel {sentinel}"
        );
    }

    assert_eq!(CLIENTBOUND_GAME_TEST_HIGHLIGHT_POS_PACKET_ID, 40);
    let registry = PlayProtocolRegistry::new();
    assert_eq!(
        registry.clientbound_name(CLIENTBOUND_GAME_TEST_HIGHLIGHT_POS_PACKET_ID),
        Some("game_test_highlight_pos")
    );

    let packet = ClientboundGameTestHighlightPosPacket {
        absolute_pos: BlockPos {
            x: 12,
            y: 64,
            z: -34,
        },
        relative_pos: BlockPos {
            x: -3,
            y: 5,
            z: 9,
        },
    };
    let mut payload = Vec::new();
    packet.write(&mut payload).unwrap();
    assert_eq!(
        payload,
        vec![
            0x00, 0x00, 0x03, 0x3f, 0xff, 0xfd, 0xe0, 0x40, // absolute_pos
            0xff, 0xff, 0xff, 0x40, 0x00, 0x00, 0x90, 0x05, // relative_pos
        ]
    );
    assert_eq!(
        ClientboundGameTestHighlightPosPacket::read(&mut cursor(payload)).unwrap(),
        packet
    );
}

#[test]
fn clientbound_game_test_highlight_pos_packet_rejects_trailing_payload() {
    let mut payload = Vec::new();
    ClientboundGameTestHighlightPosPacket {
        absolute_pos: BlockPos { x: 0, y: 0, z: 0 },
        relative_pos: BlockPos { x: 0, y: 0, z: 0 },
    }
    .write(&mut payload)
    .unwrap();
    payload.push(0);

    assert!(ClientboundGameTestHighlightPosPacket::read(&mut cursor(payload)).is_err());
}
