use super::*;

const SERVERBOUND_COMMAND_SUGGESTION_JAVA: &str = include_str!(
    "../../../../../decompiled-server-26.1.2/net/minecraft/network/protocol/game/ServerboundCommandSuggestionPacket.java"
);

#[test]
fn clientbound_command_suggestions_packet_matches_java_entry_codec() {
    assert_eq!(CLIENTBOUND_COMMAND_SUGGESTIONS_PACKET_ID, 15);
    let registry = PlayProtocolRegistry::new();
    assert_eq!(
        registry.clientbound_name(CLIENTBOUND_COMMAND_SUGGESTIONS_PACKET_ID),
        Some("command_suggestions")
    );

    let mut payload = Vec::new();
    ClientboundCommandSuggestionsPacket {
        transaction_id: 4,
        start: 1,
        length: 2,
        suggestions: vec![
            CommandSuggestionEntry {
                text: "help".to_string(),
                tooltip: None,
            },
            CommandSuggestionEntry {
                text: "hello".to_string(),
                tooltip: Some(Tag::Compound(vec![(
                    "text".to_string(),
                    Tag::String("tooltip".to_string()),
                )])),
            },
        ],
    }
    .write(&mut payload)
    .unwrap();

    assert_eq!(
        payload,
        vec![
            4, 1, 2, 2, 4, b'h', b'e', b'l', b'p', 0, 5, b'h', b'e', b'l', b'l', b'o', 1, 10, 8, 0,
            4, b't', b'e', b'x', b't', 0, 7, b't', b'o', b'o', b'l', b't', b'i', b'p', 0,
        ]
    );
}

#[test]
fn serverbound_command_suggestion_packet_matches_java_utf_bound() {
    for sentinel in [
        "this.id = input.readVarInt();",
        "this.command = input.readUtf(32500);",
        "output.writeVarInt(this.id);",
        "output.writeUtf(this.command, 32500);",
        "return GamePacketTypes.SERVERBOUND_COMMAND_SUGGESTION;",
        "listener.handleCustomCommandSuggestions(this);",
    ] {
        assert!(
            SERVERBOUND_COMMAND_SUGGESTION_JAVA.contains(sentinel),
            "Java source missing sentinel: {sentinel}"
        );
    }

    assert_eq!(SERVERBOUND_COMMAND_SUGGESTION_PACKET_ID, 15);
    let registry = PlayProtocolRegistry::new();
    assert_eq!(
        registry.serverbound_name(SERVERBOUND_COMMAND_SUGGESTION_PACKET_ID),
        Some("command_suggestion")
    );

    let packet = ServerboundCommandSuggestionPacket {
        id: 128,
        command: "/time set day".to_string(),
    };
    let mut payload = Vec::new();
    packet.write(&mut payload).unwrap();
    assert_eq!(
        payload,
        vec![
            0x80, 0x01, 13, b'/', b't', b'i', b'm', b'e', b' ', b's', b'e', b't', b' ', b'd', b'a',
            b'y'
        ]
    );
    assert_eq!(
        ServerboundCommandSuggestionPacket::read(&mut cursor(payload.clone())).unwrap(),
        packet
    );

    let mut session = PlaySession::new(1, 0);
    session.state = PlayState::Playing;
    assert_eq!(
        session.handle_decoded(decoded(SERVERBOUND_COMMAND_SUGGESTION_PACKET_ID, payload)),
        DispatchOutcome::Handled
    );
    assert_eq!(session.last_command_suggestion, Some(packet));
}

#[test]
fn serverbound_command_suggestion_packet_rejects_java_overlong_command() {
    assert!(ServerboundCommandSuggestionPacket {
        id: 1,
        command: "x".repeat(ServerboundCommandSuggestionPacket::MAX_COMMAND_CHARS + 1),
    }
    .write(&mut Vec::new())
    .is_err());

    let mut overlong_payload = Vec::new();
    write_var_i32(&mut overlong_payload, 1).unwrap();
    write_var_i32(
        &mut overlong_payload,
        (ServerboundCommandSuggestionPacket::MAX_COMMAND_CHARS + 1) as i32,
    )
    .unwrap();
    overlong_payload.extend(std::iter::repeat_n(
        b'x',
        ServerboundCommandSuggestionPacket::MAX_COMMAND_CHARS + 1,
    ));
    assert!(
        ServerboundCommandSuggestionPacket::read(&mut cursor(overlong_payload.clone())).is_err()
    );

    let mut session = PlaySession::new(1, 0);
    session.state = PlayState::Playing;
    assert!(matches!(
        session.handle_decoded(decoded(
            SERVERBOUND_COMMAND_SUGGESTION_PACKET_ID,
            overlong_payload
        )),
        DispatchOutcome::Disconnect(reason)
            if reason.starts_with("bad command suggestion packet:")
    ));
}
