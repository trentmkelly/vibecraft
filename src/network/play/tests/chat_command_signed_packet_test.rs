use super::*;

fn signed_command_prefix(command: &str) -> Vec<u8> {
    let mut payload = Vec::new();
    write_string(&mut payload, command, 32767).unwrap();
    payload.extend_from_slice(&101_i64.to_be_bytes());
    payload.extend_from_slice(&9_i64.to_be_bytes());
    payload
}

#[test]
fn serverbound_chat_command_signed_packet_matches_java_codec() {
    assert_eq!(SERVERBOUND_CHAT_COMMAND_SIGNED_PACKET_ID, 8);
    let registry = PlayProtocolRegistry::new();
    assert_eq!(
        registry.serverbound_name(SERVERBOUND_CHAT_COMMAND_SIGNED_PACKET_ID),
        Some("chat_command_signed")
    );

    let packet = ServerboundChatCommandSignedPacket {
        command: "msg Notch hello".to_string(),
        timestamp_epoch_millis: 101,
        salt: 9,
        argument_signatures: vec![ArgumentSignature {
            name: "message".to_string(),
            signature: MessageSignature([8; MessageSignature::BYTES]),
        }],
        last_seen_messages: LastSeenMessagesUpdate {
            offset: 2,
            acknowledged: vec![0b1010_0001, 0, 0b0000_1000],
            checksum: 5,
        },
    };

    let mut expected = signed_command_prefix("msg Notch hello");
    write_var_i32(&mut expected, 1).unwrap();
    write_string(&mut expected, "message", ArgumentSignature::MAX_ARGUMENT_NAME_CHARS).unwrap();
    expected.extend_from_slice(&[8; MessageSignature::BYTES]);
    write_var_i32(&mut expected, 2).unwrap();
    expected.extend_from_slice(&[0b1010_0001, 0, 0b0000_1000, 5]);

    let mut payload = Vec::new();
    packet.write(&mut payload).unwrap();
    assert_eq!(payload, expected);
    assert_eq!(
        ServerboundChatCommandSignedPacket::read(&mut cursor(payload.clone())).unwrap(),
        packet
    );

    let mut session = PlaySession::new(1, 0);
    session.state = PlayState::Playing;
    assert_eq!(
        session.handle_decoded(decoded(SERVERBOUND_CHAT_COMMAND_SIGNED_PACKET_ID, payload)),
        DispatchOutcome::Handled
    );
    assert_eq!(session.last_signed_chat_command, Some(packet));
}

#[test]
fn serverbound_chat_command_signed_packet_enforces_java_limits() {
    let last_seen_messages = LastSeenMessagesUpdate {
        offset: 0,
        acknowledged: vec![0, 0, 0],
        checksum: 0,
    };

    assert!(ServerboundChatCommandSignedPacket {
        command: "x".repeat(32768),
        timestamp_epoch_millis: 0,
        salt: 0,
        argument_signatures: Vec::new(),
        last_seen_messages: last_seen_messages.clone(),
    }
    .write(&mut Vec::new())
    .is_err());

    let mut payload = signed_command_prefix("");
    write_var_i32(&mut payload, 9).unwrap();
    assert!(ServerboundChatCommandSignedPacket::read(&mut cursor(payload)).is_err());

    let mut payload = signed_command_prefix("");
    write_var_i32(&mut payload, 1).unwrap();
    write_string(&mut payload, &"x".repeat(17), 32767).unwrap();
    assert!(ServerboundChatCommandSignedPacket::read(&mut cursor(payload)).is_err());

    assert!(ServerboundChatCommandSignedPacket {
        command: String::new(),
        timestamp_epoch_millis: 0,
        salt: 0,
        argument_signatures: vec![
            ArgumentSignature {
                name: String::new(),
                signature: MessageSignature([0; MessageSignature::BYTES]),
            };
            9
        ],
        last_seen_messages,
    }
    .write(&mut Vec::new())
    .is_err());

    assert!(LastSeenMessagesUpdate {
        offset: 0,
        acknowledged: vec![0, 0, 0x10],
        checksum: 0,
    }
    .write(&mut Vec::new())
    .is_err());
}
