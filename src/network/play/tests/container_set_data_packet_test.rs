use super::*;

#[test]
fn clientbound_container_set_data_packet_matches_java_codec() {
    assert_eq!(CLIENTBOUND_CONTAINER_SET_DATA_PACKET_ID, 19);
    let registry = PlayProtocolRegistry::new();
    assert_eq!(
        registry.clientbound_name(CLIENTBOUND_CONTAINER_SET_DATA_PACKET_ID),
        Some("container_set_data")
    );

    let mut payload = Vec::new();
    ClientboundContainerSetDataPacket {
        container_id: 128,
        id: -3,
        value: 400,
    }
    .write(&mut payload)
    .unwrap();
    assert_eq!(payload, vec![0x80, 0x01, 0xff, 0xfd, 0x01, 0x90]);
}
