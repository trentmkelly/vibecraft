use super::*;

#[test]
fn clientbound_set_subtitle_text_packet_matches_java_codec() {
    assert_eq!(CLIENTBOUND_SET_SUBTITLE_TEXT_PACKET_ID, 112);
    let registry = PlayProtocolRegistry::new();
    assert_eq!(
        registry.clientbound_name(CLIENTBOUND_SET_SUBTITLE_TEXT_PACKET_ID),
        Some("set_subtitle_text")
    );

    let mut payload = Vec::new();
    ClientboundSetSubtitleTextPacket {
        text: Tag::Compound(vec![(
            "text".to_string(),
            Tag::String("Subtitle".to_string()),
        )]),
    }
    .write(&mut payload)
    .unwrap();

    assert_eq!(
        payload,
        [
            vec![10, 8, 0, 4],
            b"text".to_vec(),
            vec![0, 8],
            b"Subtitle".to_vec(),
            vec![0],
        ]
        .concat()
    );
}
