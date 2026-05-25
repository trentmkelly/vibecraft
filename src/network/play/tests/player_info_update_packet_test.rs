use super::*;

fn text_component_bytes(text: &[u8]) -> Vec<u8> {
    [
        vec![10, 8, 0, 4],
        b"text".to_vec(),
        vec![0, text.len() as u8],
        text.to_vec(),
        vec![0],
    ]
    .concat()
}

fn profile_property(index: usize) -> GameProfileProperty {
    GameProfileProperty {
        name: format!("property{index}"),
        value: "value".to_string(),
        signature: None,
    }
}

#[test]
fn clientbound_player_info_update_packet_matches_java_action_set_order() {
    assert_eq!(CLIENTBOUND_PLAYER_INFO_UPDATE_PACKET_ID, 70);
    let registry = PlayProtocolRegistry::new();
    assert_eq!(
        registry.clientbound_name(CLIENTBOUND_PLAYER_INFO_UPDATE_PACKET_ID),
        Some("player_info_update")
    );

    let chat_session_payload = [
        vec![2; 16],                     // RemoteChatSession.Data.sessionId
        1000_i64.to_be_bytes().to_vec(), // ProfilePublicKey.Data.expiresAt
        vec![1, 0xaa],                   // public key byte array
        vec![2, 0xbb, 0xcc],             // key signature byte array
    ]
    .concat();
    let display_name = text_component_bytes(b"Tab");
    let packet = ClientboundPlayerInfoUpdatePacket {
        // Java stores these actions in an EnumSet, so the payload order must be
        // enum ordinal order even if construction order is different or duplicated.
        actions: vec![
            PlayerInfoUpdateAction::UpdateHat,
            PlayerInfoUpdateAction::UpdateDisplayName,
            PlayerInfoUpdateAction::UpdateLatency,
            PlayerInfoUpdateAction::InitializeChat,
            PlayerInfoUpdateAction::UpdateLatency,
        ],
        entries: vec![PlayerInfoUpdateEntry {
            profile_id: Uuid([1; 16]),
            profile: None,
            chat_session_payload: Some(chat_session_payload.clone()),
            game_mode: 0,
            listed: false,
            latency: 127,
            display_name_payload: Some(display_name.clone()),
            list_order: 0,
            show_hat: false,
        }],
    };

    let mut payload = Vec::new();
    packet.write(&mut payload).unwrap();

    assert_eq!(
        payload,
        [
            vec![0xb2, 1], // fixed 8-bit action set, one entry
            vec![1; 16],
            vec![1],
            chat_session_payload,
            vec![0x7f, 1],
            display_name,
            vec![0],
        ]
        .concat()
    );
}

#[test]
fn clientbound_player_info_update_initializing_uses_java_action_surface() {
    let mut payload = Vec::new();
    ClientboundPlayerInfoUpdatePacket::player_initializing(vec![PlayerInfoUpdateEntry {
        profile_id: Uuid([7; 16]),
        profile: Some(PlayerInfoProfile {
            name: "Steve".to_string(),
            properties: vec![GameProfileProperty {
                name: "textures".to_string(),
                value: "abc".to_string(),
                signature: Some("sig".to_string()),
            }],
        }),
        chat_session_payload: None,
        game_mode: 1,
        listed: true,
        latency: 20,
        display_name_payload: None,
        list_order: 3,
        show_hat: true,
    }])
    .write(&mut payload)
    .unwrap();

    assert_eq!(
        payload,
        [
            vec![0xff, 1],
            vec![7; 16],
            vec![5],
            b"Steve".to_vec(),
            vec![1, 8],
            b"textures".to_vec(),
            vec![3],
            b"abc".to_vec(),
            vec![1, 3],
            b"sig".to_vec(),
            vec![0, 1, 1, 20, 0, 3, 1],
        ]
        .concat()
    );
}

#[test]
fn clientbound_player_info_update_rejects_missing_or_oversized_profile_data() {
    let err = ClientboundPlayerInfoUpdatePacket {
        actions: vec![PlayerInfoUpdateAction::AddPlayer],
        entries: vec![PlayerInfoUpdateEntry {
            profile_id: Uuid([0; 16]),
            profile: None,
            chat_session_payload: None,
            game_mode: 0,
            listed: false,
            latency: 0,
            display_name_payload: None,
            list_order: 0,
            show_hat: false,
        }],
    }
    .write(&mut Vec::new())
    .unwrap_err();
    assert_eq!(err.kind(), io::ErrorKind::InvalidInput);

    let err = ClientboundPlayerInfoUpdatePacket {
        actions: vec![PlayerInfoUpdateAction::AddPlayer],
        entries: vec![PlayerInfoUpdateEntry {
            profile_id: Uuid([0; 16]),
            profile: Some(PlayerInfoProfile {
                name: "Steve".to_string(),
                properties: (0..17).map(profile_property).collect(),
            }),
            chat_session_payload: None,
            game_mode: 0,
            listed: false,
            latency: 0,
            display_name_payload: None,
            list_order: 0,
            show_hat: false,
        }],
    }
    .write(&mut Vec::new())
    .unwrap_err();
    assert_eq!(err.kind(), io::ErrorKind::InvalidInput);
}
