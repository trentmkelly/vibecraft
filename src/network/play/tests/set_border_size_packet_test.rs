use super::*;

#[test]
fn clientbound_set_border_size_packet_matches_java_codec() {
    assert_eq!(CLIENTBOUND_SET_BORDER_SIZE_PACKET_ID, 90);
    let registry = PlayProtocolRegistry::new();
    assert_eq!(
        registry.clientbound_name(CLIENTBOUND_SET_BORDER_SIZE_PACKET_ID),
        Some("set_border_size")
    );

    let mut payload = Vec::new();
    ClientboundSetBorderSizePacket { size: 1234.5 }
        .write(&mut payload)
        .unwrap();

    assert_eq!(payload, 1234.5_f64.to_be_bytes());
}
