use super::*;

#[test]
fn clientbound_initialize_border_packet_matches_java_codec() {
    assert_eq!(CLIENTBOUND_INITIALIZE_BORDER_PACKET_ID, 43);
    let registry = PlayProtocolRegistry::new();
    assert_eq!(
        registry.clientbound_name(CLIENTBOUND_INITIALIZE_BORDER_PACKET_ID),
        Some("initialize_border")
    );

    let mut payload = Vec::new();
    ClientboundInitializeBorderPacket {
        new_center_x: 12.5,
        new_center_z: -34.25,
        old_size: 100.0,
        new_size: 200.0,
        lerp_time: 300,
        new_absolute_max_size: 400,
        warning_blocks: 5,
        warning_time: 6,
    }
    .write(&mut payload)
    .unwrap();

    let mut expected = Vec::new();
    expected.extend_from_slice(&12.5_f64.to_be_bytes());
    expected.extend_from_slice(&(-34.25_f64).to_be_bytes());
    expected.extend_from_slice(&100.0_f64.to_be_bytes());
    expected.extend_from_slice(&200.0_f64.to_be_bytes());
    expected.extend_from_slice(&[0xac, 0x02]);
    expected.extend_from_slice(&[0x90, 0x03]);
    expected.extend_from_slice(&[0x05]);
    expected.extend_from_slice(&[0x06]);
    assert_eq!(payload, expected);
}
