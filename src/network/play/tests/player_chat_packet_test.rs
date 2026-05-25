use super::*;

fn text_component_bytes(text: &str) -> Vec<u8> {
    [
        vec![10, 8, 0, 4],
        b"text".to_vec(),
        (text.len() as u16).to_be_bytes().to_vec(),
        text.as_bytes().to_vec(),
        vec![0],
    ]
    .concat()
}

#[test]
fn clientbound_player_chat_packet_matches_java_byte_layout() {
    assert_eq!(CLIENTBOUND_PLAYER_CHAT_PACKET_ID, 65);
    let registry = PlayProtocolRegistry::new();
    assert_eq!(
        registry.clientbound_name(CLIENTBOUND_PLAYER_CHAT_PACKET_ID),
        Some("player_chat")
    );

    let unsigned = text_component_bytes("Unsigned");
    let sender_name = text_component_bytes("Steve");
    let target_name = text_component_bytes("Alex");
    let packet = ClientboundPlayerChatPacket {
        global_index: 300,
        sender: Uuid([1; 16]),
        index: 2,
        signature: Some(vec![8; MessageSignature::BYTES]),
        body: SignedMessageBodyPacked {
            content: "hello".to_string(),
            timestamp_epoch_millis: 1_700_000_000_123,
            salt: -42,
            last_seen: vec![
                MessageSignaturePackedData::Id(0),
                MessageSignaturePackedData::Full(vec![9; MessageSignature::BYTES]),
            ],
        },
        unsigned_content_payload: Some(unsigned.clone()),
        filter_mask: FilterMaskData::PartiallyFiltered(vec![0b101]),
        chat_type: BoundChatTypeData {
            chat_type_id: 0,
            name_payload: sender_name.clone(),
            target_name_payload: Some(target_name.clone()),
        },
    };

    let mut payload = Vec::new();
    packet.write(&mut payload).unwrap();

    assert_eq!(
        payload,
        [
            vec![0xac, 0x02],
            vec![1; 16],
            vec![2, 1],
            vec![8; MessageSignature::BYTES],
            vec![5],
            b"hello".to_vec(),
            1_700_000_000_123_i64.to_be_bytes().to_vec(),
            (-42_i64).to_be_bytes().to_vec(),
            vec![2, 1, 0],
            vec![9; MessageSignature::BYTES],
            vec![1],
            unsigned,
            vec![2, 1],
            0b101_u64.to_be_bytes().to_vec(),
            vec![1],
            sender_name,
            vec![1],
            target_name,
        ]
        .concat()
    );
}

#[test]
fn clientbound_player_chat_packet_encodes_absent_optional_data_and_filter_modes() {
    let mut pass_through_payload = Vec::new();
    ClientboundPlayerChatPacket {
        global_index: 0,
        sender: Uuid([2; 16]),
        index: 0,
        signature: None,
        body: SignedMessageBodyPacked {
            content: String::new(),
            timestamp_epoch_millis: 0,
            salt: 0,
            last_seen: Vec::new(),
        },
        unsigned_content_payload: None,
        filter_mask: FilterMaskData::PassThrough,
        chat_type: BoundChatTypeData {
            chat_type_id: 6,
            name_payload: text_component_bytes("Steve"),
            target_name_payload: None,
        },
    }
    .write(&mut pass_through_payload)
    .unwrap();

    assert_eq!(
        pass_through_payload,
        [
            vec![0],
            vec![2; 16],
            vec![0, 0, 0],
            0_i64.to_be_bytes().to_vec(),
            0_i64.to_be_bytes().to_vec(),
            vec![0, 0, 0, 7],
            text_component_bytes("Steve"),
            vec![0],
        ]
        .concat()
    );

    let mut fully_filtered_payload = Vec::new();
    ClientboundPlayerChatPacket {
        global_index: 0,
        sender: Uuid([3; 16]),
        index: 0,
        signature: None,
        body: SignedMessageBodyPacked {
            content: "x".to_string(),
            timestamp_epoch_millis: 0,
            salt: 0,
            last_seen: Vec::new(),
        },
        unsigned_content_payload: None,
        filter_mask: FilterMaskData::FullyFiltered,
        chat_type: BoundChatTypeData {
            chat_type_id: 1,
            name_payload: text_component_bytes("Steve"),
            target_name_payload: None,
        },
    }
    .write(&mut fully_filtered_payload)
    .unwrap();

    assert_eq!(
        fully_filtered_payload,
        [
            vec![0],
            vec![3; 16],
            vec![0, 0, 1],
            b"x".to_vec(),
            0_i64.to_be_bytes().to_vec(),
            0_i64.to_be_bytes().to_vec(),
            vec![0, 0, 1, 2],
            text_component_bytes("Steve"),
            vec![0],
        ]
        .concat()
    );
}

#[test]
fn clientbound_player_chat_packet_rejects_java_limited_fields() {
    let packet = ClientboundPlayerChatPacket {
        global_index: 0,
        sender: Uuid([0; 16]),
        index: 0,
        signature: Some(vec![1; MessageSignature::BYTES - 1]),
        body: SignedMessageBodyPacked {
            content: "ok".to_string(),
            timestamp_epoch_millis: 0,
            salt: 0,
            last_seen: Vec::new(),
        },
        unsigned_content_payload: None,
        filter_mask: FilterMaskData::PassThrough,
        chat_type: BoundChatTypeData {
            chat_type_id: 0,
            name_payload: text_component_bytes("Steve"),
            target_name_payload: None,
        },
    };
    let err = packet.write(&mut Vec::new()).unwrap_err();
    assert_eq!(err.kind(), io::ErrorKind::InvalidInput);

    let packet = ClientboundPlayerChatPacket {
        signature: None,
        body: SignedMessageBodyPacked {
            content: "a".repeat(257),
            timestamp_epoch_millis: 0,
            salt: 0,
            last_seen: Vec::new(),
        },
        ..packet
    };
    let err = packet.write(&mut Vec::new()).unwrap_err();
    assert_eq!(err.kind(), io::ErrorKind::InvalidInput);
}
