use super::*;

fn text_component(text: &str) -> Tag {
    Tag::Compound(vec![("text".to_string(), Tag::String(text.to_string()))])
}

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

fn red_team_parameters() -> TeamPacketParameters {
    TeamPacketParameters {
        display_name: text_component("Red"),
        options: 0b11,
        nametag_visibility: TeamVisibility::HideForOtherTeams,
        collision_rule: TeamCollisionRule::PushOwnTeam,
        color_id: 12,
        prefix: text_component("<"),
        suffix: text_component(">"),
    }
}

fn red_team_parameters_bytes() -> Vec<u8> {
    [
        text_component_bytes(b"Red"),
        vec![0b11, 2, 3, 12],
        text_component_bytes(b"<"),
        text_component_bytes(b">"),
    ]
    .concat()
}

#[test]
fn clientbound_set_player_team_packet_matches_java_codec() {
    assert_eq!(CLIENTBOUND_SET_PLAYER_TEAM_PACKET_ID, 109);
    let registry = PlayProtocolRegistry::new();
    assert_eq!(
        registry.clientbound_name(CLIENTBOUND_SET_PLAYER_TEAM_PACKET_ID),
        Some("set_player_team")
    );

    let mut create = Vec::new();
    ClientboundSetPlayerTeamPacket {
        name: "red".to_string(),
        method: TeamPacketMethod::Create {
            parameters: red_team_parameters(),
            players: vec!["Alex".to_string(), "Steve".to_string()],
        },
    }
    .write(&mut create)
    .unwrap();
    assert_eq!(
        create,
        [
            vec![3],
            b"red".to_vec(),
            vec![0],
            red_team_parameters_bytes(),
            vec![2, 4],
            b"Alex".to_vec(),
            vec![5],
            b"Steve".to_vec(),
        ]
        .concat()
    );

    let mut update = Vec::new();
    ClientboundSetPlayerTeamPacket {
        name: "red".to_string(),
        method: TeamPacketMethod::Update {
            parameters: red_team_parameters(),
        },
    }
    .write(&mut update)
    .unwrap();
    assert_eq!(
        update,
        [
            vec![3],
            b"red".to_vec(),
            vec![2],
            red_team_parameters_bytes()
        ]
        .concat()
    );

    let mut remove_team = Vec::new();
    ClientboundSetPlayerTeamPacket {
        name: "red".to_string(),
        method: TeamPacketMethod::Remove,
    }
    .write(&mut remove_team)
    .unwrap();
    assert_eq!(remove_team, [vec![3], b"red".to_vec(), vec![1]].concat());

    let mut add_players = Vec::new();
    ClientboundSetPlayerTeamPacket {
        name: "red".to_string(),
        method: TeamPacketMethod::AddPlayers {
            players: vec!["Alex".to_string()],
        },
    }
    .write(&mut add_players)
    .unwrap();
    assert_eq!(
        add_players,
        [vec![3], b"red".to_vec(), vec![3, 1, 4], b"Alex".to_vec()].concat()
    );

    let mut remove_players = Vec::new();
    ClientboundSetPlayerTeamPacket {
        name: "red".to_string(),
        method: TeamPacketMethod::RemovePlayers {
            players: vec!["Alex".to_string()],
        },
    }
    .write(&mut remove_players)
    .unwrap();
    assert_eq!(
        remove_players,
        [vec![3], b"red".to_vec(), vec![4, 1, 4], b"Alex".to_vec()].concat()
    );
}
