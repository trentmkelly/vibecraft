use super::*;

const SERVERBOUND_PLAYER_LOADED_JAVA: &str = vibecraft_java_source!("/net/minecraft/network/protocol/game/ServerboundPlayerLoadedPacket.java");

#[test]
fn serverbound_player_loaded_packet_matches_java_unit_codec_and_state_transition() {
    for sentinel in [
        "StreamCodec.unit(new ServerboundPlayerLoadedPacket())",
        "return GamePacketTypes.SERVERBOUND_PLAYER_LOADED;",
        "listener.handleAcceptPlayerLoad(this);",
    ] {
        assert!(
            SERVERBOUND_PLAYER_LOADED_JAVA.contains(sentinel),
            "Java source missing sentinel: {sentinel}"
        );
    }

    assert_eq!(SERVERBOUND_PLAYER_LOADED_PACKET_ID, 44);
    let registry = PlayProtocolRegistry::new();
    assert_eq!(
        registry.serverbound_name(SERVERBOUND_PLAYER_LOADED_PACKET_ID),
        Some("player_loaded")
    );

    let mut payload = Vec::new();
    ServerboundPlayerLoadedPacket.write(&mut payload).unwrap();
    assert!(payload.is_empty());
    assert_eq!(
        ServerboundPlayerLoadedPacket::read(&mut cursor(payload.clone())).unwrap(),
        ServerboundPlayerLoadedPacket
    );
    assert!(ServerboundPlayerLoadedPacket::read(&mut cursor(vec![0])).is_err());

    let mut session = PlaySession::new(1, 0);
    session.state = PlayState::WaitingForPlayerLoaded;
    assert_eq!(
        session.handle_decoded(decoded(SERVERBOUND_PLAYER_LOADED_PACKET_ID, payload)),
        DispatchOutcome::Handled
    );
    assert_eq!(session.state, PlayState::Playing);
    assert!(session.loaded);
}
