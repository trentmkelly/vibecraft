use super::*;
use std::io;

#[test]
fn clientbound_block_entity_data_packet_matches_java_codec() {
    assert_eq!(CLIENTBOUND_BLOCK_ENTITY_DATA_PACKET_ID, 6);
    let registry = PlayProtocolRegistry::new();
    assert_eq!(
        registry.clientbound_name(CLIENTBOUND_BLOCK_ENTITY_DATA_PACKET_ID),
        Some("block_entity_data")
    );

    let packet = ClientboundBlockEntityDataPacket {
        x: 1,
        y: 64,
        z: -2,
        block_entity_type_id: 129,
        tag: Tag::Compound(Vec::new()),
    };
    let mut payload = Vec::new();
    packet.write(&mut payload).unwrap();

    assert_eq!(
        payload,
        [
            pack_block_position(1, 64, -2).to_be_bytes().to_vec(),
            vec![0x81, 0x01, 10, 0],
        ]
        .concat()
    );
}

#[test]
fn clientbound_block_entity_data_packet_writes_trusted_compound_without_root_name() {
    let packet = ClientboundBlockEntityDataPacket {
        x: 0,
        y: 0,
        z: 0,
        block_entity_type_id: 1,
        tag: Tag::Compound(vec![(
            "id".to_string(),
            Tag::String("minecraft:chest".to_string()),
        )]),
    };
    let mut payload = Vec::new();
    packet.write(&mut payload).unwrap();

    assert_eq!(
        payload,
        [
            pack_block_position(0, 0, 0).to_be_bytes().to_vec(),
            vec![
                1, 10, // block entity type id, compound tag id
                8, 0, 2, b'i', b'd', // string field named "id"
                0, 15, b'm', b'i', b'n', b'e', b'c', b'r', b'a', b'f', b't', b':', b'c', b'h',
                b'e', b's', b't', 0, // compound end
            ],
        ]
        .concat()
    );
}

#[test]
fn clientbound_block_entity_data_packet_rejects_non_compound_tags() {
    let packet = ClientboundBlockEntityDataPacket {
        x: 0,
        y: 0,
        z: 0,
        block_entity_type_id: 1,
        tag: Tag::String("not a compound".to_string()),
    };

    let err = packet.write(&mut Vec::new()).unwrap_err();
    assert_eq!(err.kind(), io::ErrorKind::InvalidInput);
}
