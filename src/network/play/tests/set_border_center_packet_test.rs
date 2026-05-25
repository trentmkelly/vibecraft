use super::*;

#[test]
fn clientbound_set_border_center_packet_matches_java_codec() {
    assert_eq!(CLIENTBOUND_SET_BORDER_CENTER_PACKET_ID, 88);
    let registry = PlayProtocolRegistry::new();
    assert_eq!(
        registry.clientbound_name(CLIENTBOUND_SET_BORDER_CENTER_PACKET_ID),
        Some("set_border_center")
    );

    let mut payload = Vec::new();
    ClientboundSetBorderCenterPacket {
        new_center_x: 12.25,
        new_center_z: -34.5,
    }
    .write(&mut payload)
    .unwrap();

    assert_eq!(
        payload,
        [12.25_f64.to_be_bytes(), (-34.5_f64).to_be_bytes()].concat()
    );
}
