use super::*;

const CLIENT_GAME_PACKET_LISTENER_JAVA: &str =
    vibecraft_java_source!("/net/minecraft/network/protocol/game/ClientGamePacketListener.java");

#[test]
fn client_game_packet_listener_matches_java_protocol_and_handlers() {
    for sentinel in [
        "public interface ClientGamePacketListener extends ClientCommonPacketListener, ClientPongPacketListener",
        "default ConnectionProtocol protocol()",
        "return ConnectionProtocol.PLAY;",
    ] {
        assert!(
            CLIENT_GAME_PACKET_LISTENER_JAVA.contains(sentinel),
            "missing ClientGamePacketListener sentinel {sentinel}"
        );
    }

    assert_eq!(CLIENT_GAME_PACKET_LISTENER_PROTOCOL, ProtocolState::Play);
    assert_eq!(CLIENT_GAME_PACKET_LISTENER_HANDLERS.len(), 124);
    assert_eq!(
        CLIENT_GAME_PACKET_LISTENER_JAVA.matches("void handle").count()
            + CLIENT_GAME_PACKET_LISTENER_JAVA.matches("void set").count(),
        CLIENT_GAME_PACKET_LISTENER_HANDLERS.len()
    );

    for handler in CLIENT_GAME_PACKET_LISTENER_HANDLERS {
        let signature = format!("void {}({} packet);", handler.method, handler.packet);
        assert!(
            CLIENT_GAME_PACKET_LISTENER_JAVA.contains(&signature),
            "missing ClientGamePacketListener handler signature {signature}"
        );
    }
}

#[test]
fn client_game_packet_listener_manifest_keeps_unusual_java_names() {
    assert!(CLIENT_GAME_PACKET_LISTENER_HANDLERS.contains(&ClientGamePacketListenerHandler {
        method: "handleTabListCustomisation",
        packet: "ClientboundTabListPacket",
    }));
    assert!(CLIENT_GAME_PACKET_LISTENER_HANDLERS.contains(&ClientGamePacketListenerHandler {
        method: "setActionBarText",
        packet: "ClientboundSetActionBarTextPacket",
    }));
    assert!(CLIENT_GAME_PACKET_LISTENER_HANDLERS.contains(&ClientGamePacketListenerHandler {
        method: "handleMinecartAlongTrack",
        packet: "ClientboundMoveMinecartPacket",
    }));
    assert!(CLIENT_GAME_PACKET_LISTENER_HANDLERS.contains(&ClientGamePacketListenerHandler {
        method: "handleLowDiskSpaceWarning",
        packet: "ClientboundLowDiskSpaceWarningPacket",
    }));
}
