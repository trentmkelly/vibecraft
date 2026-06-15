use super::*;

const CLIENTBOUND_SET_BORDER_WARNING_DISTANCE_JAVA: &str = vibecraft_java_source!("/net/minecraft/network/protocol/game/ClientboundSetBorderWarningDistancePacket.java");

#[test]
fn clientbound_set_border_warning_distance_packet_matches_java_codec() {
    for sentinel in [
        "this.warningBlocks = input.readVarInt();",
        "output.writeVarInt(this.warningBlocks);",
        "return GamePacketTypes.CLIENTBOUND_SET_BORDER_WARNING_DISTANCE;",
        "listener.handleSetBorderWarningDistance(this);",
        "public int getWarningBlocks()",
    ] {
        assert!(
            CLIENTBOUND_SET_BORDER_WARNING_DISTANCE_JAVA.contains(sentinel),
            "missing ClientboundSetBorderWarningDistancePacket sentinel {sentinel}"
        );
    }

    assert_eq!(CLIENTBOUND_SET_BORDER_WARNING_DISTANCE_PACKET_ID, 92);
    let registry = PlayProtocolRegistry::new();
    assert_eq!(
        registry.clientbound_name(CLIENTBOUND_SET_BORDER_WARNING_DISTANCE_PACKET_ID),
        Some("set_border_warning_distance")
    );

    let packet = ClientboundSetBorderWarningDistancePacket {
        warning_blocks: 400,
    };
    let mut payload = Vec::new();
    packet.write(&mut payload).unwrap();

    assert_eq!(payload, [0x90, 0x03]);
    assert_eq!(
        ClientboundSetBorderWarningDistancePacket::read(&mut cursor(payload)).unwrap(),
        packet
    );
}

#[test]
fn clientbound_set_border_warning_distance_packet_rejects_malformed_payloads() {
    assert!(ClientboundSetBorderWarningDistancePacket::read(&mut cursor(Vec::new())).is_err());
    assert!(ClientboundSetBorderWarningDistancePacket::read(&mut cursor(vec![1, 0])).is_err());
}
