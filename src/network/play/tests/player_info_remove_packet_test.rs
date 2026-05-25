use super::*;

#[test]
fn clientbound_player_info_remove_packet_matches_java_codec() {
    assert_eq!(CLIENTBOUND_PLAYER_INFO_REMOVE_PACKET_ID, 69);
    let registry = PlayProtocolRegistry::new();
    assert_eq!(
        registry.clientbound_name(CLIENTBOUND_PLAYER_INFO_REMOVE_PACKET_ID),
        Some("player_info_remove")
    );

    let first = Uuid([
        0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd,
        0xee, 0xff,
    ]);
    let second = Uuid([
        0xff, 0xee, 0xdd, 0xcc, 0xbb, 0xaa, 0x99, 0x88, 0x77, 0x66, 0x55, 0x44, 0x33, 0x22,
        0x11, 0x00,
    ]);

    let mut payload = Vec::new();
    ClientboundPlayerInfoRemovePacket {
        profile_ids: vec![first, second],
    }
    .write(&mut payload)
    .unwrap();

    assert_eq!(payload[0], 2);
    assert_eq!(&payload[1..17], &first.0);
    assert_eq!(&payload[17..33], &second.0);
    assert_eq!(payload.len(), 33);
}
