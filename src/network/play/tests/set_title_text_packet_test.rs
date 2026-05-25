use super::*;

#[test]
fn clientbound_set_title_text_packet_matches_java_codec() {
    assert_eq!(CLIENTBOUND_SET_TITLE_TEXT_PACKET_ID, 114);
    let registry = PlayProtocolRegistry::new();
    assert_eq!(
        registry.clientbound_name(CLIENTBOUND_SET_TITLE_TEXT_PACKET_ID),
        Some("set_title_text")
    );

    let mut payload = Vec::new();
    ClientboundSetTitleTextPacket {
        text: Tag::Compound(vec![("text".to_string(), Tag::String("Title".to_string()))]),
    }
    .write(&mut payload)
    .unwrap();

    assert_eq!(
        payload,
        [
            vec![10, 8, 0, 4],
            b"text".to_vec(),
            vec![0, 5],
            b"Title".to_vec(),
            vec![0],
        ]
        .concat()
    );
}
