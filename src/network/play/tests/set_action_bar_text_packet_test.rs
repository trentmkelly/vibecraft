use super::*;

#[test]
fn clientbound_set_action_bar_text_packet_matches_java_codec() {
    assert_eq!(CLIENTBOUND_SET_ACTION_BAR_TEXT_PACKET_ID, 87);
    let registry = PlayProtocolRegistry::new();
    assert_eq!(
        registry.clientbound_name(CLIENTBOUND_SET_ACTION_BAR_TEXT_PACKET_ID),
        Some("set_action_bar_text")
    );

    let mut payload = Vec::new();
    ClientboundSetActionBarTextPacket {
        text: Tag::Compound(vec![(
            "text".to_string(),
            Tag::String("Action".to_string()),
        )]),
    }
    .write(&mut payload)
    .unwrap();

    assert_eq!(
        payload,
        [
            vec![10, 8, 0, 4],
            b"text".to_vec(),
            vec![0, 6],
            b"Action".to_vec(),
            vec![0],
        ]
        .concat()
    );
}
