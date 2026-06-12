use super::*;

const CLIENTBOUND_SET_BORDER_WARNING_DELAY_JAVA: &str = include_str!(
    "../../../../../decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundSetBorderWarningDelayPacket.java"
);

#[test]
fn clientbound_set_border_warning_delay_packet_matches_java_codec() {
    for sentinel in [
        "this.warningDelay = input.readVarInt();",
        "output.writeVarInt(this.warningDelay);",
        "return GamePacketTypes.CLIENTBOUND_SET_BORDER_WARNING_DELAY;",
        "listener.handleSetBorderWarningDelay(this);",
        "public int getWarningDelay()",
    ] {
        assert!(
            CLIENTBOUND_SET_BORDER_WARNING_DELAY_JAVA.contains(sentinel),
            "missing ClientboundSetBorderWarningDelayPacket sentinel {sentinel}"
        );
    }

    assert_eq!(CLIENTBOUND_SET_BORDER_WARNING_DELAY_PACKET_ID, 91);
    let registry = PlayProtocolRegistry::new();
    assert_eq!(
        registry.clientbound_name(CLIENTBOUND_SET_BORDER_WARNING_DELAY_PACKET_ID),
        Some("set_border_warning_delay")
    );

    let packet = ClientboundSetBorderWarningDelayPacket { warning_delay: 300 };
    let mut payload = Vec::new();
    packet.write(&mut payload).unwrap();

    assert_eq!(payload, [0xac, 0x02]);
    assert_eq!(
        ClientboundSetBorderWarningDelayPacket::read(&mut cursor(payload)).unwrap(),
        packet
    );
}

#[test]
fn clientbound_set_border_warning_delay_packet_rejects_malformed_payloads() {
    assert!(ClientboundSetBorderWarningDelayPacket::read(&mut cursor(Vec::new())).is_err());
    assert!(ClientboundSetBorderWarningDelayPacket::read(&mut cursor(vec![1, 0])).is_err());
}
