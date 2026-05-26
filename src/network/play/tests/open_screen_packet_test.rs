use super::*;

#[test]
fn clientbound_open_screen_packet_matches_java_codec() {
    assert_eq!(CLIENTBOUND_OPEN_SCREEN_PACKET_ID, 59);
    let registry = PlayProtocolRegistry::new();
    assert_eq!(
        registry.clientbound_name(CLIENTBOUND_OPEN_SCREEN_PACKET_ID),
        Some("open_screen")
    );

    let mut payload = Vec::new();
    ClientboundOpenScreenPacket {
        container_id: 300,
        // Java MenuType registers generic_9x3 third, so its registry VarInt is 2.
        menu_type_id: 2,
        title: Tag::Compound(vec![("text".to_string(), Tag::String("Chest".to_string()))]),
    }
    .write(&mut payload)
    .unwrap();
    assert_eq!(
        payload,
        vec![
            0xac, 0x02, 2, 10, 8, 0, 4, b't', b'e', b'x', b't', 0, 5, b'C', b'h', b'e', b's', b't',
            0
        ]
    );
}
