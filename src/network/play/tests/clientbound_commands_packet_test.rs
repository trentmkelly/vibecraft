use super::*;

const CLIENTBOUND_COMMANDS_JAVA: &str =
    vibecraft_java_source!("/net/minecraft/network/protocol/game/ClientboundCommandsPacket.java");

#[test]
fn clientbound_commands_packet_matches_java_codec_surface() {
    for sentinel in [
        "private static final byte MASK_TYPE = 3;",
        "private static final byte FLAG_EXECUTABLE = 4;",
        "private static final byte FLAG_REDIRECT = 8;",
        "private static final byte FLAG_CUSTOM_SUGGESTIONS = 16;",
        "private static final byte FLAG_RESTRICTED = 32;",
        "this.entries = input.readList(ClientboundCommandsPacket::readNode);",
        "this.rootIndex = input.readVarInt();",
        "validateEntries(this.entries);",
        "output.writeCollection(this.entries, (buffer, entry) -> entry.write(buffer));",
        "output.writeVarInt(this.rootIndex);",
        "return GamePacketTypes.CLIENTBOUND_COMMANDS;",
        "listener.handleCommands(this);",
    ] {
        assert!(
            CLIENTBOUND_COMMANDS_JAVA.contains(sentinel),
            "missing ClientboundCommandsPacket sentinel {sentinel}"
        );
    }

    let registry = PlayProtocolRegistry::new();
    assert_eq!(CLIENTBOUND_COMMANDS_PACKET_ID, 16);
    assert_eq!(
        registry.clientbound_name(CLIENTBOUND_COMMANDS_PACKET_ID),
        Some("commands")
    );
}

#[test]
fn clientbound_commands_packet_reads_and_writes_java_node_layout() {
    let packet = ClientboundCommandsPacket {
        root_index: 0,
        entries: vec![
            CommandNodeEntryData {
                stub: CommandNodeStubData::Root,
                executable: false,
                restricted: false,
                redirect: None,
                children: vec![1, 2],
            },
            CommandNodeEntryData {
                stub: CommandNodeStubData::Literal {
                    name: "help".to_string(),
                },
                executable: true,
                restricted: false,
                redirect: None,
                children: Vec::new(),
            },
            CommandNodeEntryData {
                stub: CommandNodeStubData::Argument {
                    name: "target".to_string(),
                    parser_type_id: 5,
                    parser_payload: vec![0x02],
                    suggestion_id: Some(Identifier::parse("minecraft:ask_server").unwrap()),
                },
                executable: false,
                restricted: true,
                redirect: Some(1),
                children: Vec::new(),
            },
        ],
    };

    let mut payload = Vec::new();
    packet.write(&mut payload).unwrap();
    assert_eq!(
        payload,
        [
            vec![3, 0, 2, 1, 2, 5, 0, 4],
            b"help".to_vec(),
            vec![58, 0, 1, 6],
            b"target".to_vec(),
            vec![5, 0x02, 20],
            b"minecraft:ask_server".to_vec(),
            vec![0],
        ]
        .concat()
    );
    assert_eq!(
        ClientboundCommandsPacket::read(&mut cursor(payload)).unwrap(),
        packet
    );
}

#[test]
fn clientbound_commands_packet_rejects_impossible_command_trees() {
    let child_cycle = ClientboundCommandsPacket {
        root_index: 0,
        entries: vec![
            CommandNodeEntryData {
                stub: CommandNodeStubData::Root,
                executable: false,
                restricted: false,
                redirect: None,
                children: vec![1],
            },
            CommandNodeEntryData {
                stub: CommandNodeStubData::Literal {
                    name: "loop".to_string(),
                },
                executable: false,
                restricted: false,
                redirect: None,
                children: vec![0],
            },
        ],
    };
    let mut payload = Vec::new();
    child_cycle.write(&mut payload).unwrap();
    assert!(ClientboundCommandsPacket::read(&mut cursor(payload)).is_err());

    let redirect_cycle = ClientboundCommandsPacket {
        root_index: 0,
        entries: vec![
            CommandNodeEntryData {
                stub: CommandNodeStubData::Root,
                executable: false,
                restricted: false,
                redirect: Some(1),
                children: Vec::new(),
            },
            CommandNodeEntryData {
                stub: CommandNodeStubData::Root,
                executable: false,
                restricted: false,
                redirect: Some(0),
                children: Vec::new(),
            },
        ],
    };
    let mut payload = Vec::new();
    redirect_cycle.write(&mut payload).unwrap();
    assert!(ClientboundCommandsPacket::read(&mut cursor(payload)).is_err());
}
