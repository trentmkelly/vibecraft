use super::*;

const CLIENTBOUND_PLAYER_ROTATION_JAVA: &str = vibecraft_java_source!("/net/minecraft/network/protocol/game/ClientboundPlayerRotationPacket.java");

#[test]
fn clientbound_player_rotation_packet_matches_java_codec() {
    for sentinel in [
        "public record ClientboundPlayerRotationPacket(float yRot, boolean relativeY, float xRot, boolean relativeX)",
        "ByteBufCodecs.FLOAT",
        "ClientboundPlayerRotationPacket::yRot",
        "ByteBufCodecs.BOOL",
        "ClientboundPlayerRotationPacket::relativeY",
        "ClientboundPlayerRotationPacket::xRot",
        "ClientboundPlayerRotationPacket::relativeX",
        "return GamePacketTypes.CLIENTBOUND_PLAYER_ROTATION;",
        "listener.handleRotatePlayer(this);",
    ] {
        assert!(
            CLIENTBOUND_PLAYER_ROTATION_JAVA.contains(sentinel),
            "missing ClientboundPlayerRotationPacket sentinel {sentinel}"
        );
    }

    assert_eq!(CLIENTBOUND_PLAYER_ROTATION_PACKET_ID, 73);
    let registry = PlayProtocolRegistry::new();
    assert_eq!(
        registry.clientbound_name(CLIENTBOUND_PLAYER_ROTATION_PACKET_ID),
        Some("player_rotation")
    );

    let packet = ClientboundPlayerRotationPacket {
        y_rot: 90.0,
        relative_y: true,
        x_rot: -30.5,
        relative_x: false,
    };
    let mut payload = Vec::new();
    packet.write(&mut payload).unwrap();
    assert_eq!(
        payload,
        [
            90.0_f32.to_be_bytes().to_vec(),
            vec![1],
            (-30.5_f32).to_be_bytes().to_vec(),
            vec![0],
        ]
        .concat()
    );
    assert_eq!(
        ClientboundPlayerRotationPacket::read(&mut cursor(payload)).unwrap(),
        packet
    );
}

#[test]
fn clientbound_player_rotation_packet_rejects_malformed_payloads() {
    assert!(ClientboundPlayerRotationPacket::read(&mut cursor(vec![0; 9])).is_err());

    let mut payload = Vec::new();
    ClientboundPlayerRotationPacket {
        y_rot: 1.0,
        relative_y: false,
        x_rot: 2.0,
        relative_x: true,
    }
    .write(&mut payload)
    .unwrap();
    payload.push(0);
    assert!(ClientboundPlayerRotationPacket::read(&mut cursor(payload)).is_err());
}
