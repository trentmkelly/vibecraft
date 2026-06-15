use super::*;

const CLIENTBOUND_SET_BORDER_SIZE_JAVA: &str = vibecraft_java_source!("/net/minecraft/network/protocol/game/ClientboundSetBorderSizePacket.java");

#[test]
fn clientbound_set_border_size_packet_matches_java_codec() {
    for sentinel in [
        "this.size = input.readDouble();",
        "output.writeDouble(this.size);",
        "return GamePacketTypes.CLIENTBOUND_SET_BORDER_SIZE;",
        "listener.handleSetBorderSize(this);",
        "public double getSize()",
    ] {
        assert!(
            CLIENTBOUND_SET_BORDER_SIZE_JAVA.contains(sentinel),
            "missing ClientboundSetBorderSizePacket sentinel {sentinel}"
        );
    }

    assert_eq!(CLIENTBOUND_SET_BORDER_SIZE_PACKET_ID, 90);
    let registry = PlayProtocolRegistry::new();
    assert_eq!(
        registry.clientbound_name(CLIENTBOUND_SET_BORDER_SIZE_PACKET_ID),
        Some("set_border_size")
    );

    let packet = ClientboundSetBorderSizePacket { size: 1234.5 };
    let mut payload = Vec::new();
    packet.write(&mut payload).unwrap();

    assert_eq!(payload, 1234.5_f64.to_be_bytes());
    assert_eq!(
        ClientboundSetBorderSizePacket::read(&mut cursor(payload)).unwrap(),
        packet
    );
}

#[test]
fn clientbound_set_border_size_packet_rejects_malformed_payloads() {
    assert!(ClientboundSetBorderSizePacket::read(&mut cursor(vec![0; 7])).is_err());
    assert!(ClientboundSetBorderSizePacket::read(&mut cursor(vec![0; 9])).is_err());
}
