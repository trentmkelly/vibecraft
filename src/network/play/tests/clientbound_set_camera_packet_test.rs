use super::*;

const CLIENTBOUND_SET_CAMERA_JAVA: &str = vibecraft_java_source!("/net/minecraft/network/protocol/game/ClientboundSetCameraPacket.java");

#[test]
fn clientbound_set_camera_packet_matches_java_codec() {
    for sentinel in [
        "this.cameraId = camera.getId();",
        "this.cameraId = input.readVarInt();",
        "output.writeVarInt(this.cameraId);",
        "return GamePacketTypes.CLIENTBOUND_SET_CAMERA;",
        "listener.handleSetCamera(this);",
        "return level.getEntity(this.cameraId);",
    ] {
        assert!(
            CLIENTBOUND_SET_CAMERA_JAVA.contains(sentinel),
            "missing ClientboundSetCameraPacket sentinel {sentinel}"
        );
    }

    assert_eq!(CLIENTBOUND_SET_CAMERA_PACKET_ID, 93);
    let registry = PlayProtocolRegistry::new();
    assert_eq!(
        registry.clientbound_name(CLIENTBOUND_SET_CAMERA_PACKET_ID),
        Some("set_camera")
    );

    let packet = ClientboundSetCameraPacket { camera_id: 300 };
    let mut payload = Vec::new();
    packet.write(&mut payload).unwrap();
    assert_eq!(payload, vec![0xac, 0x02]);
    assert_eq!(
        ClientboundSetCameraPacket::read(&mut cursor(payload)).unwrap(),
        packet
    );
}

#[test]
fn clientbound_set_camera_packet_rejects_malformed_payloads() {
    assert!(ClientboundSetCameraPacket::read(&mut cursor(Vec::new())).is_err());
    assert!(ClientboundSetCameraPacket::read(&mut cursor(vec![1, 0])).is_err());
}
