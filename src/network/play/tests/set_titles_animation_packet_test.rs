use super::*;

#[test]
fn clientbound_set_titles_animation_packet_matches_java_codec() {
    assert_eq!(CLIENTBOUND_SET_TITLES_ANIMATION_PACKET_ID, 115);
    let registry = PlayProtocolRegistry::new();
    assert_eq!(
        registry.clientbound_name(CLIENTBOUND_SET_TITLES_ANIMATION_PACKET_ID),
        Some("set_titles_animation")
    );

    let mut payload = Vec::new();
    ClientboundSetTitlesAnimationPacket {
        fade_in: 20,
        stay: 60,
        fade_out: -1,
    }
    .write(&mut payload)
    .unwrap();

    assert_eq!(
        payload,
        [
            0x00, 0x00, 0x00, 0x14, // fadeIn
            0x00, 0x00, 0x00, 0x3c, // stay
            0xff, 0xff, 0xff, 0xff, // fadeOut
        ]
    );
}
