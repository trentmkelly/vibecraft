use super::*;

#[test]
fn clientbound_system_chat_packet_matches_java_codec() {
    assert_eq!(CLIENTBOUND_SYSTEM_CHAT_PACKET_ID, 121);
    let registry = PlayProtocolRegistry::new();
    assert_eq!(
        registry.clientbound_name(CLIENTBOUND_SYSTEM_CHAT_PACKET_ID),
        Some("system_chat")
    );

    let mut payload = Vec::new();
    ClientboundSystemChatPacket {
        content: Tag::Compound(vec![("text".to_string(), Tag::String("Hello".to_string()))]),
        overlay: true,
    }
    .write(&mut payload)
    .unwrap();

    assert_eq!(
        payload,
        [
            vec![10, 8, 0, 4],
            b"text".to_vec(),
            vec![0, 5],
            b"Hello".to_vec(),
            vec![0, 1],
        ]
        .concat()
    );
}
