use super::*;

#[test]
fn clientbound_set_border_lerp_size_packet_matches_java_codec() {
    assert_eq!(CLIENTBOUND_SET_BORDER_LERP_SIZE_PACKET_ID, 89);
    let registry = PlayProtocolRegistry::new();
    assert_eq!(
        registry.clientbound_name(CLIENTBOUND_SET_BORDER_LERP_SIZE_PACKET_ID),
        Some("set_border_lerp_size")
    );

    let mut payload = Vec::new();
    ClientboundSetBorderLerpSizePacket {
        old_size: 100.0,
        new_size: 200.0,
        lerp_time: 300,
    }
    .write(&mut payload)
    .unwrap();

    let mut expected = Vec::new();
    expected.extend_from_slice(&100.0_f64.to_be_bytes());
    expected.extend_from_slice(&200.0_f64.to_be_bytes());
    expected.extend_from_slice(&[0xac, 0x02]);
    assert_eq!(payload, expected);
}
