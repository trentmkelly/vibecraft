use super::{
    ChatVisibility, ClientInformation, ClientboundClearDialogPacket,
    ClientboundCustomPayloadPacket, ClientboundCustomReportDetailsPacket,
    ClientboundDisconnectPacket, ClientboundKeepAlivePacket, ClientboundPingPacket,
    ClientboundResourcePackPopPacket, ClientboundResourcePackPushPacket, CommonSession,
    CustomPayload, DialogState, HumanoidArm, KeepAliveState, KeepAliveTick, ParticleStatus,
    ResourcePackAction, ResourcePackState, ResourcePackStatus, ServerLinkEntry, ServerLinkLabel,
    ServerLinkType, ServerboundClientInformationPacket, ServerboundCustomClickActionPacket,
    ServerboundCustomPayloadPacket, ServerboundKeepAlivePacket, ServerboundPongPacket,
    ServerboundResourcePackPacket, TagNetworkPayload, MAX_SERVERBOUND_CUSTOM_PAYLOAD_SIZE,
};
use crate::network::codec::{ComponentJson, Uuid};
use crate::registry::Identifier;
use std::io::Cursor;

#[test]
fn round_trips_keepalive_ping_pong_and_disconnect() {
    let keepalive = ClientboundKeepAlivePacket { id: 123456789 };
    let mut bytes = Vec::new();
    keepalive.write(&mut bytes).unwrap();
    assert_eq!(bytes, 123456789_i64.to_be_bytes());
    assert_eq!(
        ClientboundKeepAlivePacket::read(&mut Cursor::new(bytes.clone())).unwrap(),
        keepalive
    );
    assert_eq!(
        ServerboundKeepAlivePacket::read(&mut Cursor::new(bytes)).unwrap(),
        ServerboundKeepAlivePacket { id: 123456789 }
    );

    let ping = ClientboundPingPacket { id: -7 };
    let mut bytes = Vec::new();
    ping.write(&mut bytes).unwrap();
    assert_eq!(bytes, (-7_i32).to_be_bytes());
    assert_eq!(
        ClientboundPingPacket::read(&mut Cursor::new(bytes.clone())).unwrap(),
        ping
    );
    assert_eq!(
        ServerboundPongPacket::read(&mut Cursor::new(bytes)).unwrap(),
        ServerboundPongPacket { id: -7 }
    );

    let disconnect = ClientboundDisconnectPacket {
        reason: ComponentJson("{\"text\":\"bye\"}".to_string()),
    };
    let mut bytes = Vec::new();
    disconnect.write(&mut bytes).unwrap();
    assert_eq!(
        bytes,
        vec![8, 0, 3, b'b', b'y', b'e'],
        "Java's component codec collapses plain literal text to a TAG_String"
    );
    assert_eq!(
        ClientboundDisconnectPacket::read(&mut Cursor::new(bytes)).unwrap(),
        disconnect
    );
}

#[test]
fn common_packet_types_match_java_common_registry_names() {
    const COMMON_PACKET_TYPES_JAVA: &str = vibecraft_java_source!("/net/minecraft/network/protocol/common/CommonPacketTypes.java");

    for expected in [
        "CLIENTBOUND_CLEAR_DIALOG = createClientbound(\"clear_dialog\")",
        "CLIENTBOUND_CUSTOM_PAYLOAD = createClientbound(\"custom_payload\")",
        "CLIENTBOUND_CUSTOM_REPORT_DETAILS = createClientbound(\"custom_report_details\")",
        "CLIENTBOUND_DISCONNECT = createClientbound(\"disconnect\")",
        "CLIENTBOUND_KEEP_ALIVE = createClientbound(\"keep_alive\")",
        "CLIENTBOUND_PING = createClientbound(\"ping\")",
        "CLIENTBOUND_RESOURCE_PACK_POP = createClientbound(\"resource_pack_pop\")",
        "CLIENTBOUND_RESOURCE_PACK_PUSH = createClientbound(\"resource_pack_push\")",
        "CLIENTBOUND_SERVER_LINKS = createClientbound(\"server_links\")",
        "CLIENTBOUND_SHOW_DIALOG = createClientbound(\"show_dialog\")",
        "CLIENTBOUND_STORE_COOKIE = createClientbound(\"store_cookie\")",
        "CLIENTBOUND_TRANSFER = createClientbound(\"transfer\")",
        "CLIENTBOUND_UPDATE_TAGS = createClientbound(\"update_tags\")",
        "SERVERBOUND_CLIENT_INFORMATION = createServerbound(\"client_information\")",
        "SERVERBOUND_CUSTOM_PAYLOAD = createServerbound(\"custom_payload\")",
        "SERVERBOUND_KEEP_ALIVE = createServerbound(\"keep_alive\")",
        "SERVERBOUND_PONG = createServerbound(\"pong\")",
        "SERVERBOUND_RESOURCE_PACK = createServerbound(\"resource_pack\")",
        "SERVERBOUND_CUSTOM_CLICK_ACTION = createServerbound(\"custom_click_action\")",
    ] {
        assert!(
            COMMON_PACKET_TYPES_JAVA.contains(expected),
            "missing CommonPacketTypes declaration: {expected}"
        );
    }
}

#[test]
fn common_packet_listener_surfaces_match_java_interfaces() {
    const CLIENT_COMMON_LISTENER_JAVA: &str = vibecraft_java_source!("/net/minecraft/network/protocol/common/ClientCommonPacketListener.java");
    const SERVER_COMMON_LISTENER_JAVA: &str = vibecraft_java_source!("/net/minecraft/network/protocol/common/ServerCommonPacketListener.java");

    for sentinel in [
        "public interface ClientCommonPacketListener extends ClientCookiePacketListener",
        "void handleKeepAlive(ClientboundKeepAlivePacket packet);",
        "void handlePing(ClientboundPingPacket packet);",
        "void handleCustomPayload(ClientboundCustomPayloadPacket packet);",
        "void handleDisconnect(ClientboundDisconnectPacket packet);",
        "void handleResourcePackPush(ClientboundResourcePackPushPacket packet);",
        "void handleResourcePackPop(ClientboundResourcePackPopPacket packet);",
        "void handleUpdateTags(ClientboundUpdateTagsPacket packet);",
        "void handleStoreCookie(ClientboundStoreCookiePacket packet);",
        "void handleTransfer(ClientboundTransferPacket packet);",
        "void handleCustomReportDetails(ClientboundCustomReportDetailsPacket packet);",
        "void handleServerLinks(ClientboundServerLinksPacket packet);",
        "void handleClearDialog(ClientboundClearDialogPacket packet);",
        "void handleShowDialog(ClientboundShowDialogPacket packet);",
    ] {
        assert!(
            CLIENT_COMMON_LISTENER_JAVA.contains(sentinel),
            "missing ClientCommonPacketListener sentinel {sentinel}"
        );
    }
    for sentinel in [
        "public interface ServerCommonPacketListener extends ServerCookiePacketListener",
        "void handleKeepAlive(ServerboundKeepAlivePacket packet);",
        "void handlePong(ServerboundPongPacket serverboundPongPacket);",
        "void handleCustomPayload(ServerboundCustomPayloadPacket packet);",
        "void handleResourcePackResponse(ServerboundResourcePackPacket packet);",
        "void handleClientInformation(ServerboundClientInformationPacket packet);",
        "void handleCustomClickAction(ServerboundCustomClickActionPacket packet);",
    ] {
        assert!(
            SERVER_COMMON_LISTENER_JAVA.contains(sentinel),
            "missing ServerCommonPacketListener sentinel {sentinel}"
        );
    }
}

#[test]
fn common_disconnect_uses_trusted_component_nbt_not_login_json() {
    let disconnect = ClientboundDisconnectPacket {
        reason: ComponentJson(
            "{\"translate\":\"multiplayer.disconnect.outdated_client\"}".to_string(),
        ),
    };
    let mut bytes = Vec::new();
    disconnect.write(&mut bytes).unwrap();

    assert_eq!(
        bytes[0], 10,
        "trusted component must start with an NBT compound tag"
    );
    assert!(
        !bytes.starts_with(b"{") && !bytes.starts_with(b" "),
        "common disconnect must not use login JSON/string component encoding"
    );
    assert_eq!(
        ClientboundDisconnectPacket::read(&mut Cursor::new(bytes)).unwrap(),
        disconnect
    );
}

#[test]
fn round_trips_known_brand_custom_payload() {
    let packet = ServerboundCustomPayloadPacket {
        payload: CustomPayload::Brand("vanilla".to_string()),
    };
    let mut bytes = Vec::new();
    packet.write(&mut bytes).unwrap();
    assert_eq!(
        bytes,
        [vec![15], b"minecraft:brand".to_vec(), vec![7], b"vanilla".to_vec()].concat()
    );
    assert_eq!(
        ServerboundCustomPayloadPacket::read(&mut Cursor::new(bytes)).unwrap(),
        packet
    );
}

#[test]
fn round_trips_clientbound_brand_custom_payload() {
    let packet = ClientboundCustomPayloadPacket {
        payload: CustomPayload::Brand("vibecraft".to_string()),
    };
    let mut bytes = Vec::new();
    packet.write(&mut bytes).unwrap();
    assert_eq!(
        bytes,
        [
            vec![15],
            b"minecraft:brand".to_vec(),
            vec![9],
            b"vibecraft".to_vec()
        ]
        .concat()
    );
    assert_eq!(
        ClientboundCustomPayloadPacket::read(&mut Cursor::new(bytes)).unwrap(),
        packet
    );
}

#[test]
fn round_trips_unknown_custom_payload_with_direction_limit() {
    let packet = ClientboundCustomPayloadPacket {
        payload: CustomPayload::Unknown {
            channel: Identifier::parse("vibecraft:debug").unwrap(),
            payload: vec![1, 2, 3],
        },
    };
    let mut bytes = Vec::new();
    packet.write(&mut bytes).unwrap();
    assert_eq!(
        ClientboundCustomPayloadPacket::read(&mut Cursor::new(bytes)).unwrap(),
        packet
    );
}

#[test]
fn rejects_oversized_serverbound_unknown_custom_payload() {
    let packet = ServerboundCustomPayloadPacket {
        payload: CustomPayload::Unknown {
            channel: Identifier::parse("vibecraft:debug").unwrap(),
            payload: vec![0; MAX_SERVERBOUND_CUSTOM_PAYLOAD_SIZE + 1],
        },
    };
    assert!(packet.write(&mut Vec::new()).is_err());
}

#[test]
fn read_rejects_oversized_serverbound_unknown_custom_payload() {
    // The decode-side limit is what the live config/play custom-payload handler
    // relies on (Java `DiscardedPayload.codec(id, 32767)` throws on read). Build a
    // wire frame with an unknown channel and a payload one byte over the limit.
    let channel = Identifier::parse("vibecraft:debug").unwrap();
    let mut bytes = Vec::new();
    crate::network::codec::write_identifier(&mut bytes, &channel).unwrap();
    bytes.extend(vec![0u8; MAX_SERVERBOUND_CUSTOM_PAYLOAD_SIZE + 1]);
    assert!(ServerboundCustomPayloadPacket::read(&mut Cursor::new(bytes)).is_err());

    // A payload exactly at the limit decodes to an Unknown payload.
    let mut at_limit = Vec::new();
    crate::network::codec::write_identifier(&mut at_limit, &channel).unwrap();
    at_limit.extend(vec![0u8; MAX_SERVERBOUND_CUSTOM_PAYLOAD_SIZE]);
    assert!(ServerboundCustomPayloadPacket::read(&mut Cursor::new(at_limit)).is_ok());
}

#[test]
fn round_trips_client_information_with_vanilla_defaults() {
    let packet = ServerboundClientInformationPacket {
        information: ClientInformation::default(),
    };
    let mut bytes = Vec::new();
    packet.write(&mut bytes).unwrap();
    assert_eq!(bytes, vec![5, b'e', b'n', b'_', b'u', b's', 2, 0, 1, 0, 1, 0, 0, 0]);
    assert_eq!(
        ServerboundClientInformationPacket::read(&mut Cursor::new(bytes)).unwrap(),
        packet
    );
    assert_eq!(packet.information.language, "en_us");
    assert_eq!(packet.information.view_distance, 2);
    assert_eq!(packet.information.chat_visibility, ChatVisibility::Full);
    assert_eq!(packet.information.main_hand, HumanoidArm::Right);
    assert_eq!(packet.information.particle_status, ParticleStatus::All);
}

#[test]
fn round_trips_resource_pack_packets() {
    let id = Uuid([3; 16]);
    let push = ClientboundResourcePackPushPacket {
        id,
        url: "https://example.invalid/pack.zip".to_string(),
        hash: "0123456789abcdef0123456789abcdef01234567".to_string(),
        required: true,
        prompt: Some(ComponentJson("{\"text\":\"Use pack?\"}".to_string())),
    };
    let mut bytes = Vec::new();
    push.write(&mut bytes).unwrap();
    let mut expected = id.0.to_vec();
    expected.push(push.url.len() as u8);
    expected.extend_from_slice(push.url.as_bytes());
    expected.push(push.hash.len() as u8);
    expected.extend_from_slice(push.hash.as_bytes());
    expected.push(1);
    expected.push(1);
    expected.extend_from_slice(&[8, 0, 9]);
    expected.extend_from_slice(b"Use pack?");
    assert_eq!(bytes, expected);
    assert_eq!(
        ClientboundResourcePackPushPacket::read(&mut Cursor::new(bytes)).unwrap(),
        push
    );

    let pop = ClientboundResourcePackPopPacket { id: Some(id) };
    let mut bytes = Vec::new();
    pop.write(&mut bytes).unwrap();
    let mut expected = vec![1];
    expected.extend_from_slice(&id.0);
    assert_eq!(bytes, expected);
    assert_eq!(
        ClientboundResourcePackPopPacket::read(&mut Cursor::new(bytes)).unwrap(),
        pop
    );
    let mut bytes = Vec::new();
    ClientboundResourcePackPopPacket { id: None }
        .write(&mut bytes)
        .unwrap();
    assert_eq!(bytes, vec![0]);

    let response = ServerboundResourcePackPacket {
        id,
        action: ResourcePackAction::Downloaded,
    };
    let mut bytes = Vec::new();
    response.write(&mut bytes).unwrap();
    let mut expected = id.0.to_vec();
    expected.push(4);
    assert_eq!(bytes, expected);
    assert_eq!(
        ServerboundResourcePackPacket::read(&mut Cursor::new(bytes)).unwrap(),
        response
    );
}

#[test]
fn validates_resource_pack_hash_and_terminal_actions() {
    let too_long = ClientboundResourcePackPushPacket {
        id: Uuid([0; 16]),
        url: "https://example.invalid/pack.zip".to_string(),
        hash: "x".repeat(ClientboundResourcePackPushPacket::MAX_HASH_LENGTH + 1),
        required: false,
        prompt: None,
    };
    assert!(too_long.write(&mut Vec::new()).is_err());

    assert!(!ResourcePackAction::Accepted.is_terminal());
    assert!(!ResourcePackAction::Downloaded.is_terminal());
    assert!(ResourcePackAction::Declined.is_terminal());
    assert!(ResourcePackAction::SuccessfullyLoaded.is_terminal());
    assert!(ResourcePackAction::FailedDownload.is_terminal());
    assert!(ResourcePackAction::InvalidUrl.is_terminal());
    assert!(ResourcePackAction::FailedReload.is_terminal());
    assert!(ResourcePackAction::Discarded.is_terminal());
}

#[test]
fn round_trips_clear_dialog_and_report_details() {
    let mut bytes = Vec::new();
    ClientboundClearDialogPacket.write(&mut bytes).unwrap();
    assert_eq!(bytes, Vec::<u8>::new());
    assert_eq!(
        ClientboundClearDialogPacket::read(&mut Cursor::new(bytes)).unwrap(),
        ClientboundClearDialogPacket
    );

    let packet = ClientboundCustomReportDetailsPacket {
        details: vec![
            ("server".to_string(), "VibeCraft".to_string()),
            ("build".to_string(), "clean-room".to_string()),
        ],
    };
    let mut bytes = Vec::new();
    packet.write(&mut bytes).unwrap();
    let expected = [
        &[2][..],
        &[6],
        b"server",
        &[9],
        b"VibeCraft",
        &[5],
        b"build",
        &[10],
        b"clean-room",
    ]
    .concat();
    assert_eq!(bytes, expected);
    assert_eq!(
        ClientboundCustomReportDetailsPacket::read(&mut Cursor::new(bytes)).unwrap(),
        packet
    );
}

#[test]
fn rejects_too_many_report_details() {
    let packet = ClientboundCustomReportDetailsPacket {
        details: (0..=ClientboundCustomReportDetailsPacket::MAX_DETAIL_COUNT)
            .map(|index| (format!("key{index}"), "value".to_string()))
            .collect(),
    };
    assert!(packet.write(&mut Vec::new()).is_err());
}

#[test]
fn round_trips_custom_click_action_payload() {
    const SERVERBOUND_CUSTOM_CLICK_ACTION_JAVA: &str = vibecraft_java_source!("/net/minecraft/network/protocol/common/ServerboundCustomClickActionPacket.java");
    for sentinel in [
        "public record ServerboundCustomClickActionPacket(Identifier id, Optional<Tag> payload)",
        "new NbtAccounter(32768L, 16)",
        "ByteBufCodecs.lengthPrefixed(65536)",
        "Identifier.STREAM_CODEC",
        "CommonPacketTypes.SERVERBOUND_CUSTOM_CLICK_ACTION",
        "listener.handleCustomClickAction(this);",
    ] {
        assert!(
            SERVERBOUND_CUSTOM_CLICK_ACTION_JAVA.contains(sentinel),
            "missing ServerboundCustomClickActionPacket sentinel {sentinel}"
        );
    }

    let packet = ServerboundCustomClickActionPacket {
        id: Identifier::parse("vibecraft:inspect").unwrap(),
        payload: Some(vec![10, 20, 30]),
    };
    let mut bytes = Vec::new();
    packet.write(&mut bytes).unwrap();
    assert_eq!(
        ServerboundCustomClickActionPacket::read(&mut Cursor::new(bytes)).unwrap(),
        packet
    );
}

#[test]
fn rejects_oversized_custom_click_action_payload() {
    let packet = ServerboundCustomClickActionPacket {
        id: Identifier::parse("vibecraft:inspect").unwrap(),
        payload: Some(vec![
            0;
            ServerboundCustomClickActionPacket::MAX_LENGTH_PREFIXED_PAYLOAD_SIZE
                + 1
        ]),
    };
    assert!(packet.write(&mut Vec::new()).is_err());
}

#[test]
fn round_trips_server_links_packet() {
    let packet = super::ClientboundServerLinksPacket {
        links: vec![
            ServerLinkEntry {
                label: ServerLinkLabel::Known(ServerLinkType::BugReport),
                link: "https://example.invalid/bugs".to_string(),
            },
            ServerLinkEntry {
                label: ServerLinkLabel::Custom(ComponentJson("{\"text\":\"Docs\"}".to_string())),
                link: "https://example.invalid/docs".to_string(),
            },
        ],
    };
    let mut bytes = Vec::new();
    packet.write(&mut bytes).unwrap();
    let mut expected = Vec::new();
    expected.push(2);
    expected.extend_from_slice(&[1, 0]);
    expected.push("https://example.invalid/bugs".len() as u8);
    expected.extend_from_slice(b"https://example.invalid/bugs");
    expected.push(0);
    expected.push("{\"text\":\"Docs\"}".len() as u8);
    expected.extend_from_slice(b"{\"text\":\"Docs\"}");
    expected.push("https://example.invalid/docs".len() as u8);
    expected.extend_from_slice(b"https://example.invalid/docs");
    assert_eq!(bytes, expected);
    assert_eq!(
        super::ClientboundServerLinksPacket::read(&mut Cursor::new(bytes)).unwrap(),
        packet
    );
}

#[test]
fn round_trips_update_tags_packet() {
    let packet = super::ClientboundUpdateTagsPacket {
        registries: vec![(
            Identifier::parse("minecraft:block").unwrap(),
            TagNetworkPayload {
                tags: vec![(
                    Identifier::parse("minecraft:mineable/pickaxe").unwrap(),
                    vec![1, 2, 3],
                )],
            },
        )],
    };
    let mut bytes = Vec::new();
    packet.write(&mut bytes).unwrap();
    let mut expected = Vec::new();
    expected.push(1);
    expected.push("minecraft:block".len() as u8);
    expected.extend_from_slice(b"minecraft:block");
    expected.push(1);
    expected.push("minecraft:mineable/pickaxe".len() as u8);
    expected.extend_from_slice(b"minecraft:mineable/pickaxe");
    expected.extend_from_slice(&[3, 1, 2, 3]);
    assert_eq!(bytes, expected);
    assert_eq!(
        super::ClientboundUpdateTagsPacket::read(&mut Cursor::new(bytes)).unwrap(),
        packet
    );
}

#[test]
fn round_trips_show_dialog_as_bounded_context_free_payload() {
    const CLIENTBOUND_SHOW_DIALOG_JAVA: &str = vibecraft_java_source!("/net/minecraft/network/protocol/common/ClientboundShowDialogPacket.java");
    for sentinel in [
        "public record ClientboundShowDialogPacket(Holder<Dialog> dialog)",
        "Dialog.STREAM_CODEC, ClientboundShowDialogPacket::dialog, ClientboundShowDialogPacket::new",
        "Dialog.CONTEXT_FREE_STREAM_CODEC.map(Holder::direct, Holder::value)",
        "CommonPacketTypes.CLIENTBOUND_SHOW_DIALOG",
        "listener.handleShowDialog(this);",
    ] {
        assert!(
            CLIENTBOUND_SHOW_DIALOG_JAVA.contains(sentinel),
            "missing ClientboundShowDialogPacket sentinel {sentinel}"
        );
    }

    let packet = super::ClientboundShowDialogPacket {
        payload: vec![1, 2, 3, 4],
    };
    let mut bytes = Vec::new();
    packet.write(&mut bytes).unwrap();
    assert_eq!(
        super::ClientboundShowDialogPacket::read(&mut Cursor::new(bytes)).unwrap(),
        packet
    );

    let oversized = super::ClientboundShowDialogPacket {
        payload: vec![
            0;
            super::ClientboundShowDialogPacket::MAX_CONTEXT_FREE_DIALOG_PAYLOAD_SIZE + 1
        ],
    };
    assert!(oversized.write(&mut Vec::new()).is_err());
}

#[test]
fn dialog_state_tracks_show_and_clear_packets() {
    let packet = super::ClientboundShowDialogPacket {
        payload: vec![7, 8, 9],
    };
    let mut state = DialogState::default();
    assert_eq!(state.current(), None);
    state.show(packet.clone());
    assert_eq!(state.current(), Some(&packet));
    state.clear(ClientboundClearDialogPacket);
    assert_eq!(state.current(), None);
}

#[test]
fn common_session_tracks_cross_common_protocol_state() {
    let mut session = CommonSession::new(0);
    session.handle_client_information(ServerboundClientInformationPacket {
        information: ClientInformation {
            language: "fr_fr".to_string(),
            ..ClientInformation::default()
        },
    });
    assert_eq!(session.client_information.language, "fr_fr");

    session.update_server_links(super::ClientboundServerLinksPacket {
        links: vec![ServerLinkEntry {
            label: ServerLinkLabel::Known(ServerLinkType::Website),
            link: "https://example.invalid".to_string(),
        }],
    });
    assert_eq!(session.server_links.len(), 1);

    session.dialogs.show(super::ClientboundShowDialogPacket {
        payload: vec![1, 2, 3],
    });
    assert!(session.dialogs.current().is_some());

    let id = Uuid([1; 16]);
    session
        .resource_packs
        .push(ClientboundResourcePackPushPacket {
            id,
            url: "https://example.invalid/pack.zip".to_string(),
            hash: "abc".to_string(),
            required: false,
            prompt: None,
        });
    assert!(session.resource_packs.contains(id));

    session.disconnect("{\"text\":\"bye\"}");
    assert!(session.is_disconnected());
}

#[test]
fn keepalive_state_sends_challenge_and_smooths_latency() {
    let mut state = KeepAliveState::new(1_000, 100);
    assert_eq!(state.tick(15_999, false), KeepAliveTick::Idle);
    let challenge = match state.tick(16_000, false) {
        KeepAliveTick::Send(packet) => packet.id,
        other => panic!("expected keepalive send, got {other:?}"),
    };
    assert!(state.is_pending());

    let result = state.handle_response(ServerboundKeepAlivePacket { id: challenge }, 16_200, false);
    assert_eq!(result, KeepAliveTick::Idle);
    assert!(!state.is_pending());
    assert_eq!(state.latency_ms(), 125);
}

#[test]
fn keepalive_state_disconnects_on_timeout_or_wrong_response() {
    let mut state = KeepAliveState::new(0, 0);
    assert!(matches!(state.tick(15_000, false), KeepAliveTick::Send(_)));
    assert_eq!(state.tick(30_000, false), KeepAliveTick::Disconnect);

    let mut state = KeepAliveState::new(0, 0);
    assert!(matches!(state.tick(15_000, false), KeepAliveTick::Send(_)));
    assert_eq!(
        state.handle_response(ServerboundKeepAlivePacket { id: 99 }, 15_100, false),
        KeepAliveTick::Disconnect
    );
}

#[test]
fn resource_pack_state_tracks_push_pop_and_terminal_status() {
    let id = Uuid([9; 16]);
    let mut state = ResourcePackState::default();
    state.push(ClientboundResourcePackPushPacket {
        id,
        url: "https://example.invalid/pack.zip".to_string(),
        hash: "abc".to_string(),
        required: false,
        prompt: None,
    });
    assert!(state.contains(id));
    assert_eq!(
        state.handle_response(ServerboundResourcePackPacket {
            id,
            action: ResourcePackAction::Accepted,
        }),
        ResourcePackStatus::Pending
    );
    assert!(state.contains(id));
    assert_eq!(
        state.handle_response(ServerboundResourcePackPacket {
            id,
            action: ResourcePackAction::SuccessfullyLoaded,
        }),
        ResourcePackStatus::Terminal(ResourcePackAction::SuccessfullyLoaded)
    );
    assert!(!state.contains(id));

    state.push(ClientboundResourcePackPushPacket {
        id,
        url: "https://example.invalid/pack.zip".to_string(),
        hash: "abc".to_string(),
        required: false,
        prompt: None,
    });
    state.pop(ClientboundResourcePackPopPacket { id: Some(id) });
    assert!(!state.contains(id));
}

#[test]
fn resource_pack_state_disconnects_when_required_pack_is_declined() {
    let id = Uuid([8; 16]);
    let mut state = ResourcePackState::default();
    state.push(ClientboundResourcePackPushPacket {
        id,
        url: "https://example.invalid/pack.zip".to_string(),
        hash: "abc".to_string(),
        required: true,
        prompt: None,
    });
    assert_eq!(
        state.handle_response(ServerboundResourcePackPacket {
            id,
            action: ResourcePackAction::Declined,
        }),
        ResourcePackStatus::DisconnectRequiredDeclined
    );
    assert!(!state.contains(id));
}

#[test]
fn resource_pack_state_treats_failed_statuses_as_terminal() {
    for action in [
        ResourcePackAction::FailedDownload,
        ResourcePackAction::InvalidUrl,
        ResourcePackAction::FailedReload,
        ResourcePackAction::Discarded,
    ] {
        let id = Uuid([action.index() as u8; 16]);
        let mut state = ResourcePackState::default();
        state.push(ClientboundResourcePackPushPacket {
            id,
            url: "https://example.invalid/pack.zip".to_string(),
            hash: "abc".to_string(),
            required: false,
            prompt: None,
        });

        assert_eq!(
            state.handle_response(ServerboundResourcePackPacket { id, action }),
            ResourcePackStatus::Terminal(action)
        );
        assert!(!state.contains(id));
    }
}
