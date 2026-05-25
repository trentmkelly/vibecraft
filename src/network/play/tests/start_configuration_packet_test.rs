use super::*;

#[test]
fn clientbound_start_configuration_packet_matches_java_unit_codec() {
    assert_eq!(CLIENTBOUND_START_CONFIGURATION_PACKET_ID, 118);
    let registry = PlayProtocolRegistry::new();
    assert_eq!(
        registry.clientbound_name(CLIENTBOUND_START_CONFIGURATION_PACKET_ID),
        Some("start_configuration")
    );

    let mut payload = Vec::new();
    ClientboundStartConfigurationPacket
        .write(&mut payload)
        .unwrap();
    assert!(payload.is_empty());
}
