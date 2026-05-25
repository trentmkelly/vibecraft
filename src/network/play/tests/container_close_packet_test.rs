use super::*;

#[test]
fn clientbound_container_close_packet_matches_java_codec() {
    assert_eq!(CLIENTBOUND_CONTAINER_CLOSE_PACKET_ID, 17);
    let registry = PlayProtocolRegistry::new();
    assert_eq!(
        registry.clientbound_name(CLIENTBOUND_CONTAINER_CLOSE_PACKET_ID),
        Some("container_close")
    );

    let mut payload = Vec::new();
    ClientboundContainerClosePacket { container_id: 128 }
        .write(&mut payload)
        .unwrap();
    assert_eq!(payload, vec![0x80, 0x01]);
}
