use super::*;

#[test]
fn clientbound_set_entity_data_packet_matches_java_packed_items_codec() {
    assert_eq!(CLIENTBOUND_SET_ENTITY_DATA_PACKET_ID, 99);
    let registry = PlayProtocolRegistry::new();
    assert_eq!(
        registry.clientbound_name(CLIENTBOUND_SET_ENTITY_DATA_PACKET_ID),
        Some("set_entity_data")
    );

    let mut payload = Vec::new();
    ClientboundSetEntityDataPacket {
        id: 300,
        packed_items: vec![
            EntityDataValue::typed(0, EntityMetadataValue::Byte(-1)).unwrap(),
            EntityDataValue::typed(1, EntityMetadataValue::VarInt(300)).unwrap(),
            EntityDataValue::typed(8, EntityMetadataValue::Boolean(true)).unwrap(),
            EntityDataValue::typed(19, EntityMetadataValue::OptionalUnsignedInt(Some(4))).unwrap(),
            EntityDataValue::raw(254, 127, vec![0xaa, 0xbb]),
        ],
    }
    .write(&mut payload)
    .unwrap();

    assert_eq!(
        payload,
        vec![
            0xac, 0x02, // entity id
            0, 0, 0xff, // byte metadata
            1, 1, 0xac, 0x02, // VarInt metadata
            8, 8, 1, // boolean metadata
            19, 19, 5, // OptionalInt encodes value + 1, absent as zero
            254, 127, 0xaa, 0xbb, // highest legal metadata id before EOF
            0xff, // EOF marker
        ]
    );
}

#[test]
fn clientbound_set_entity_data_packet_writes_empty_list_eof_marker() {
    let mut payload = Vec::new();
    ClientboundSetEntityDataPacket {
        id: 1,
        packed_items: Vec::new(),
    }
    .write(&mut payload)
    .unwrap();

    assert_eq!(payload, vec![1, 0xff]);
}

#[test]
fn clientbound_set_entity_data_packet_rejects_reserved_eof_index() {
    let err = ClientboundSetEntityDataPacket {
        id: 1,
        packed_items: vec![EntityDataValue::raw(0xff, 0, vec![0])],
    }
    .write(&mut Vec::new())
    .unwrap_err();

    assert_eq!(err.kind(), io::ErrorKind::InvalidInput);
}
