use super::*;

fn chat_prefix(message: &str, timestamp_epoch_millis: i64, salt: i64) -> Vec<u8> {
    let mut payload = Vec::new();
    write_string(&mut payload, message, ServerboundChatPacket::MAX_MESSAGE_CHARS).unwrap();
    payload.extend_from_slice(&timestamp_epoch_millis.to_be_bytes());
    payload.extend_from_slice(&salt.to_be_bytes());
    payload
}

#[test]
fn serverbound_chat_packet_matches_java_codec() {
    assert_eq!(SERVERBOUND_CHAT_PACKET_ID, 9);
    let registry = PlayProtocolRegistry::new();
    assert_eq!(
        registry.serverbound_name(SERVERBOUND_CHAT_PACKET_ID),
        Some("chat")
    );

    let packet = ServerboundChatPacket {
        message: "hi".to_string(),
        timestamp_epoch_millis: 100,
        salt: -7,
        signature: Some(MessageSignature([7; MessageSignature::BYTES])),
        last_seen_messages: LastSeenMessagesUpdate {
            offset: 2,
            acknowledged: vec![0b1010_0001, 0, 0b0000_1000],
            checksum: 5,
        },
    };

    let mut expected = chat_prefix("hi", 100, -7);
    expected.push(1);
    expected.extend_from_slice(&[7; MessageSignature::BYTES]);
    write_var_i32(&mut expected, 2).unwrap();
    expected.extend_from_slice(&[0b1010_0001, 0, 0b0000_1000, 5]);

    let mut payload = Vec::new();
    packet.write(&mut payload).unwrap();
    assert_eq!(payload, expected);
    assert_eq!(
        ServerboundChatPacket::read(&mut cursor(payload.clone())).unwrap(),
        packet
    );

    let mut session = PlaySession::new(1, 0);
    session.state = PlayState::Playing;
    assert_eq!(
        session.handle_decoded(decoded(SERVERBOUND_CHAT_PACKET_ID, payload)),
        DispatchOutcome::Handled
    );
    assert_eq!(session.last_chat, Some(packet));
}

#[test]
fn serverbound_chat_packet_encodes_absent_signature_and_enforces_java_limits() {
    let packet = ServerboundChatPacket {
        message: "hi".to_string(),
        timestamp_epoch_millis: 100,
        salt: -7,
        signature: None,
        last_seen_messages: LastSeenMessagesUpdate {
            offset: 0,
            acknowledged: vec![0, 0, 0],
            checksum: 0,
        },
    };

    let mut expected = chat_prefix("hi", 100, -7);
    expected.extend_from_slice(&[0, 0, 0, 0, 0, 0]);
    let mut payload = Vec::new();
    packet.write(&mut payload).unwrap();
    assert_eq!(payload, expected);

    assert!(ServerboundChatPacket {
        message: "x".repeat(257),
        timestamp_epoch_millis: 0,
        salt: 0,
        signature: None,
        last_seen_messages: LastSeenMessagesUpdate {
            offset: 0,
            acknowledged: vec![0, 0, 0],
            checksum: 0,
        },
    }
    .write(&mut Vec::new())
    .is_err());

    let mut overlong_message = Vec::new();
    write_var_i32(&mut overlong_message, 257).unwrap();
    overlong_message.extend(std::iter::repeat_n(b'x', 257));
    assert!(ServerboundChatPacket::read(&mut cursor(overlong_message)).is_err());

    let mut short_signature = chat_prefix("hi", 100, -7);
    short_signature.push(1);
    short_signature.extend_from_slice(&[7; MessageSignature::BYTES - 1]);
    assert!(ServerboundChatPacket::read(&mut cursor(short_signature.clone())).is_err());

    let mut session = PlaySession::new(1, 0);
    session.state = PlayState::Playing;
    assert!(matches!(
        session.handle_decoded(decoded(SERVERBOUND_CHAT_PACKET_ID, short_signature)),
        DispatchOutcome::Disconnect(reason) if reason.starts_with("bad chat packet:")
    ));
}
