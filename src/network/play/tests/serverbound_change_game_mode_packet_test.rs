use super::*;
use crate::network::codec::cursor;

const SERVERBOUND_CHANGE_GAME_MODE_PACKET_JAVA: &str = vibecraft_java_source!("/net/minecraft/network/protocol/game/ServerboundChangeGameModePacket.java");
const GAME_TYPE_JAVA: &str =
    vibecraft_java_source!("/net/minecraft/world/level/GameType.java");

#[test]
fn serverbound_change_game_mode_packet_matches_java_codec() {
    for sentinel in [
        "GameType.STREAM_CODEC",
        "ServerboundChangeGameModePacket::mode",
        "return GamePacketTypes.SERVERBOUND_CHANGE_GAME_MODE;",
        "listener.handleChangeGameMode(this);",
    ] {
        assert!(
            SERVERBOUND_CHANGE_GAME_MODE_PACKET_JAVA.contains(sentinel),
            "missing ServerboundChangeGameModePacket sentinel {sentinel}"
        );
    }
    for sentinel in [
        "SURVIVAL(0, \"survival\")",
        "CREATIVE(1, \"creative\")",
        "ADVENTURE(2, \"adventure\")",
        "SPECTATOR(3, \"spectator\")",
        "ByIdMap.OutOfBoundsStrategy.ZERO",
        "ByteBufCodecs.idMapper(BY_ID, GameType::getId)",
    ] {
        assert!(
            GAME_TYPE_JAVA.contains(sentinel),
            "missing GameType sentinel {sentinel}"
        );
    }

    assert_eq!(SERVERBOUND_CHANGE_GAME_MODE_PACKET_ID, 5);
    let registry = PlayProtocolRegistry::new();
    assert_eq!(
        registry.serverbound_name(SERVERBOUND_CHANGE_GAME_MODE_PACKET_ID),
        Some("change_game_mode")
    );

    for (mode, payload) in [
        (GameMode::Survival, vec![0]),
        (GameMode::Creative, vec![1]),
        (GameMode::Adventure, vec![2]),
        (GameMode::Spectator, vec![3]),
    ] {
        let packet = ServerboundChangeGameModePacket { mode };
        let mut encoded = Vec::new();
        packet.write(&mut encoded).unwrap();
        assert_eq!(encoded, payload);
        assert_eq!(
            ServerboundChangeGameModePacket::read(&mut cursor(encoded)).unwrap(),
            packet
        );
    }

    assert_eq!(
        ServerboundChangeGameModePacket::read(&mut cursor(vec![99])).unwrap(),
        ServerboundChangeGameModePacket {
            mode: GameMode::Survival
        },
        "Java GameType id mapper falls back to SURVIVAL for out-of-bounds ids"
    );

    let packet = ServerboundChangeGameModePacket {
        mode: GameMode::Creative,
    };
    let mut session = PlaySession::new(1, 0);
    assert_eq!(
        session.handle_decoded(super::decoded(
            SERVERBOUND_CHANGE_GAME_MODE_PACKET_ID,
            vec![1]
        )),
        DispatchOutcome::Handled
    );
    assert_eq!(session.last_change_game_mode, Some(packet));
}

#[test]
fn serverbound_change_game_mode_packet_rejects_malformed_payloads() {
    assert!(ServerboundChangeGameModePacket::read(&mut cursor(Vec::new())).is_err());
    assert!(ServerboundChangeGameModePacket::read(&mut cursor(vec![1, 0])).is_err());

    let mut session = PlaySession::new(1, 0);
    assert!(matches!(
        session.handle_decoded(super::decoded(SERVERBOUND_CHANGE_GAME_MODE_PACKET_ID, Vec::new())),
        DispatchOutcome::Disconnect(reason) if reason.contains("bad change game mode packet")
    ));

    let mut session = PlaySession::new(1, 0);
    assert!(matches!(
        session.handle_decoded(super::decoded(
            SERVERBOUND_CHANGE_GAME_MODE_PACKET_ID,
            vec![1, 0]
        )),
        DispatchOutcome::Disconnect(reason) if reason.contains("bad change game mode packet")
    ));
}
