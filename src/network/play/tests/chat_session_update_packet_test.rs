use super::*;

fn chat_session_prefix(session_id: Uuid, expires_at_epoch_millis: i64) -> Vec<u8> {
    let mut payload = Vec::new();
    write_uuid(&mut payload, session_id).unwrap();
    payload.extend_from_slice(&expires_at_epoch_millis.to_be_bytes());
    payload
}

#[test]
fn serverbound_chat_session_update_packet_matches_java_remote_chat_session_data_codec() {
    assert_eq!(SERVERBOUND_CHAT_SESSION_UPDATE_PACKET_ID, 10);
    let registry = PlayProtocolRegistry::new();
    assert_eq!(
        registry.serverbound_name(SERVERBOUND_CHAT_SESSION_UPDATE_PACKET_ID),
        Some("chat_session_update")
    );

    let packet = ServerboundChatSessionUpdatePacket {
        session_id: Uuid([
            0x10, 0x32, 0x54, 0x76, 0x98, 0xba, 0xdc, 0xfe, 0x0f, 0xed, 0xcb, 0xa9, 0x87, 0x65,
            0x43, 0x21,
        ]),
        expires_at_epoch_millis: 1_717_171_717_171,
        public_key: vec![0x30, 0x82, 0x01, 0x0a],
        key_signature: vec![0xaa, 0xbb, 0xcc],
    };

    let mut expected = chat_session_prefix(packet.session_id, packet.expires_at_epoch_millis);
    write_var_i32(&mut expected, packet.public_key.len() as i32).unwrap();
    expected.extend_from_slice(&packet.public_key);
    write_var_i32(&mut expected, packet.key_signature.len() as i32).unwrap();
    expected.extend_from_slice(&packet.key_signature);

    let mut payload = Vec::new();
    packet.write(&mut payload).unwrap();
    assert_eq!(payload, expected);
    assert_eq!(
        ServerboundChatSessionUpdatePacket::read(&mut cursor(payload.clone())).unwrap(),
        packet
    );

    let mut session = PlaySession::new(1, 0);
    session.state = PlayState::Playing;
    assert_eq!(
        session.handle_decoded(decoded(SERVERBOUND_CHAT_SESSION_UPDATE_PACKET_ID, payload)),
        DispatchOutcome::Handled
    );
    assert_eq!(session.last_chat_session_update, Some(packet));
}

#[test]
fn serverbound_chat_session_update_packet_enforces_java_public_key_and_signature_limits() {
    let session_id = Uuid([9; 16]);

    assert!(ServerboundChatSessionUpdatePacket {
        session_id,
        expires_at_epoch_millis: 0,
        public_key: vec![0; ServerboundChatSessionUpdatePacket::MAX_PUBLIC_KEY_BYTES + 1],
        key_signature: Vec::new(),
    }
    .write(&mut Vec::new())
    .is_err());

    assert!(ServerboundChatSessionUpdatePacket {
        session_id,
        expires_at_epoch_millis: 0,
        public_key: Vec::new(),
        key_signature: vec![0; ServerboundChatSessionUpdatePacket::MAX_SIGNATURE_BYTES + 1],
    }
    .write(&mut Vec::new())
    .is_err());

    let mut overlong_public_key = chat_session_prefix(session_id, 0);
    write_var_i32(
        &mut overlong_public_key,
        (ServerboundChatSessionUpdatePacket::MAX_PUBLIC_KEY_BYTES + 1) as i32,
    )
    .unwrap();
    assert!(ServerboundChatSessionUpdatePacket::read(&mut cursor(overlong_public_key)).is_err());

    let mut overlong_signature = chat_session_prefix(session_id, 0);
    write_var_i32(&mut overlong_signature, 0).unwrap();
    write_var_i32(
        &mut overlong_signature,
        (ServerboundChatSessionUpdatePacket::MAX_SIGNATURE_BYTES + 1) as i32,
    )
    .unwrap();
    assert!(ServerboundChatSessionUpdatePacket::read(&mut cursor(overlong_signature.clone()))
        .is_err());

    let mut session = PlaySession::new(1, 0);
    session.state = PlayState::Playing;
    assert!(matches!(
        session.handle_decoded(decoded(
            SERVERBOUND_CHAT_SESSION_UPDATE_PACKET_ID,
            overlong_signature
        )),
        DispatchOutcome::Disconnect(reason)
            if reason.starts_with("bad chat session update packet:")
    ));
}
