use super::*;

#[test]
fn clientbound_mount_screen_open_packet_matches_java_codec() {
    assert_eq!(CLIENTBOUND_MOUNT_SCREEN_OPEN_PACKET_ID, 41);
    let registry = PlayProtocolRegistry::new();
    assert_eq!(
        registry.clientbound_name(CLIENTBOUND_MOUNT_SCREEN_OPEN_PACKET_ID),
        Some("mount_screen_open")
    );

    let mut payload = Vec::new();
    ClientboundMountScreenOpenPacket {
        container_id: 128,
        inventory_columns: 130,
        entity_id: -300,
    }
    .write(&mut payload)
    .unwrap();

    let mut expected = vec![0x80, 0x01, 0x82, 0x01];
    expected.extend_from_slice(&(-300_i32).to_be_bytes());
    assert_eq!(payload, expected);
}
