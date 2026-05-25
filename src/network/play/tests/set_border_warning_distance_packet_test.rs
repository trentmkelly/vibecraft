use super::*;

#[test]
fn clientbound_set_border_warning_distance_packet_matches_java_codec() {
    assert_eq!(CLIENTBOUND_SET_BORDER_WARNING_DISTANCE_PACKET_ID, 92);
    let registry = PlayProtocolRegistry::new();
    assert_eq!(
        registry.clientbound_name(CLIENTBOUND_SET_BORDER_WARNING_DISTANCE_PACKET_ID),
        Some("set_border_warning_distance")
    );

    let mut payload = Vec::new();
    ClientboundSetBorderWarningDistancePacket {
        warning_blocks: 400,
    }
    .write(&mut payload)
    .unwrap();

    assert_eq!(payload, [0x90, 0x03]);
}
