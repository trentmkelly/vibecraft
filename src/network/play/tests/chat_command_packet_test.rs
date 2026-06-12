use super::*;

const SERVERBOUND_CHAT_COMMAND_JAVA: &str = include_str!(
    "../../../../../decompiled-server-26.1.2/net/minecraft/network/protocol/game/ServerboundChatCommandPacket.java"
);

#[test]
fn serverbound_chat_command_packet_matches_java_utf_codec() {
    for sentinel in [
        "this(input.readUtf());",
        "output.writeUtf(this.command);",
        "return GamePacketTypes.SERVERBOUND_CHAT_COMMAND;",
        "listener.handleChatCommand(this);",
    ] {
        assert!(
            SERVERBOUND_CHAT_COMMAND_JAVA.contains(sentinel),
            "Java source missing sentinel: {sentinel}"
        );
    }

    assert_eq!(SERVERBOUND_CHAT_COMMAND_PACKET_ID, 7);
    let registry = PlayProtocolRegistry::new();
    assert_eq!(
        registry.serverbound_name(SERVERBOUND_CHAT_COMMAND_PACKET_ID),
        Some("chat_command")
    );

    let packet = ServerboundChatCommandPacket {
        command: "seed".to_string(),
    };
    let mut payload = Vec::new();
    packet.write(&mut payload).unwrap();
    assert_eq!(payload, vec![4, b's', b'e', b'e', b'd']);
    assert_eq!(
        ServerboundChatCommandPacket::read(&mut cursor(payload.clone())).unwrap(),
        packet
    );

    let mut session = PlaySession::new(1, 0);
    session.state = PlayState::Playing;
    assert_eq!(
        session.handle_decoded(decoded(SERVERBOUND_CHAT_COMMAND_PACKET_ID, payload)),
        DispatchOutcome::Handled
    );
    assert_eq!(session.last_chat_command, Some(packet));
}

#[test]
fn serverbound_chat_command_packet_rejects_java_overlong_command() {
    assert!(ServerboundChatCommandPacket {
        command: "x".repeat(32768),
    }
    .write(&mut Vec::new())
    .is_err());

    let mut payload = Vec::new();
    write_var_i32(&mut payload, 32768).unwrap();
    payload.extend(std::iter::repeat_n(b'x', 32768));
    assert!(ServerboundChatCommandPacket::read(&mut cursor(payload.clone())).is_err());

    let mut session = PlaySession::new(1, 0);
    session.state = PlayState::Playing;
    assert!(matches!(
        session.handle_decoded(decoded(SERVERBOUND_CHAT_COMMAND_PACKET_ID, payload)),
        DispatchOutcome::Disconnect(reason) if reason.starts_with("bad chat command packet:")
    ));
}
