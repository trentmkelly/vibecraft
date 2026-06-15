use super::*;

const CLIENTBOUND_SET_BORDER_CENTER_JAVA: &str = vibecraft_java_source!("/net/minecraft/network/protocol/game/ClientboundSetBorderCenterPacket.java");

#[test]
fn clientbound_set_border_center_packet_matches_java_codec() {
    for sentinel in [
        "this.newCenterX = input.readDouble();",
        "this.newCenterZ = input.readDouble();",
        "output.writeDouble(this.newCenterX);",
        "output.writeDouble(this.newCenterZ);",
        "return GamePacketTypes.CLIENTBOUND_SET_BORDER_CENTER;",
        "listener.handleSetBorderCenter(this);",
        "public double getNewCenterX()",
        "public double getNewCenterZ()",
    ] {
        assert!(
            CLIENTBOUND_SET_BORDER_CENTER_JAVA.contains(sentinel),
            "missing ClientboundSetBorderCenterPacket sentinel {sentinel}"
        );
    }

    assert_eq!(CLIENTBOUND_SET_BORDER_CENTER_PACKET_ID, 88);
    let registry = PlayProtocolRegistry::new();
    assert_eq!(
        registry.clientbound_name(CLIENTBOUND_SET_BORDER_CENTER_PACKET_ID),
        Some("set_border_center")
    );

    let packet = ClientboundSetBorderCenterPacket {
        new_center_x: 12.25,
        new_center_z: -34.5,
    };
    let mut payload = Vec::new();
    packet.write(&mut payload).unwrap();

    assert_eq!(
        payload,
        [12.25_f64.to_be_bytes(), (-34.5_f64).to_be_bytes()].concat()
    );
    assert_eq!(
        ClientboundSetBorderCenterPacket::read(&mut cursor(payload)).unwrap(),
        packet
    );
}

#[test]
fn clientbound_set_border_center_packet_rejects_malformed_payloads() {
    assert!(ClientboundSetBorderCenterPacket::read(&mut cursor(vec![0; 15])).is_err());
    assert!(ClientboundSetBorderCenterPacket::read(&mut cursor(vec![0; 17])).is_err());
}
