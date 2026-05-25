use super::*;

#[test]
fn clientbound_set_border_warning_delay_packet_matches_java_codec() {
    assert_eq!(CLIENTBOUND_SET_BORDER_WARNING_DELAY_PACKET_ID, 91);
    let registry = PlayProtocolRegistry::new();
    assert_eq!(
        registry.clientbound_name(CLIENTBOUND_SET_BORDER_WARNING_DELAY_PACKET_ID),
        Some("set_border_warning_delay")
    );

    let mut payload = Vec::new();
    ClientboundSetBorderWarningDelayPacket { warning_delay: 300 }
        .write(&mut payload)
        .unwrap();

    assert_eq!(payload, [0xac, 0x02]);
}
