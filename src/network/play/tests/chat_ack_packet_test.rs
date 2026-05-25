use super::*;

#[test]
fn serverbound_chat_ack_packet_matches_java_offset_varint_codec() {
    assert_eq!(SERVERBOUND_CHAT_ACK_PACKET_ID, 6);
    let registry = PlayProtocolRegistry::new();
    assert_eq!(
        registry.serverbound_name(SERVERBOUND_CHAT_ACK_PACKET_ID),
        Some("chat_ack")
    );

    let packet = ServerboundChatAckPacket { offset: 128 };
    let mut payload = Vec::new();
    packet.write(&mut payload).unwrap();
    assert_eq!(payload, vec![0x80, 0x01]);
    assert_eq!(
        ServerboundChatAckPacket::read(&mut cursor(payload.clone())).unwrap(),
        packet
    );

    let mut session = PlaySession::new(1, 0);
    session.state = PlayState::Playing;
    assert_eq!(
        session.handle_decoded(decoded(SERVERBOUND_CHAT_ACK_PACKET_ID, payload)),
        DispatchOutcome::Handled
    );
    assert_eq!(session.last_chat_ack, Some(packet));
}
