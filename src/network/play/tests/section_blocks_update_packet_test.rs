use super::*;
use std::io;

#[test]
fn clientbound_section_blocks_update_packet_matches_java_codec() {
    assert_eq!(CLIENTBOUND_SECTION_BLOCKS_UPDATE_PACKET_ID, 84);
    let registry = PlayProtocolRegistry::new();
    assert_eq!(
        registry.clientbound_name(CLIENTBOUND_SECTION_BLOCKS_UPDATE_PACKET_ID),
        Some("section_blocks_update")
    );

    let packet = ClientboundSectionBlocksUpdatePacket {
        section_pos: SectionPos { x: 1, y: -2, z: 3 },
        updates: vec![
            SectionBlockUpdate {
                packed_pos: 0x0abc,
                block_state_id: 118,
            },
            SectionBlockUpdate {
                packed_pos: 0x0123,
                block_state_id: 4096,
            },
        ],
    };

    let mut payload = Vec::new();
    packet.write(&mut payload).unwrap();

    assert_eq!(&payload[..8], &0x0000_0400_003f_fffe_i64.to_be_bytes());
    assert_eq!(
        &payload[8..],
        &[
            2, // update count
            0xbc, 0xd5, 0x1d, // (118 << 12) | 0x0abc
            0xa3, 0x82, 0x80, 0x08, // (4096 << 12) | 0x0123
        ]
    );
}

#[test]
fn clientbound_section_blocks_update_packet_rejects_out_of_range_section_position() {
    let packet = ClientboundSectionBlocksUpdatePacket {
        section_pos: SectionPos { x: 0, y: 0, z: 0 },
        updates: vec![SectionBlockUpdate {
            packed_pos: 0x1000,
            block_state_id: 1,
        }],
    };

    let err = packet.write(&mut Vec::new()).unwrap_err();
    assert_eq!(err.kind(), io::ErrorKind::InvalidInput);
}
