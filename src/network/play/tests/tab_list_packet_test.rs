use super::*;

fn text_component(text: &str) -> Tag {
    Tag::Compound(vec![("text".to_string(), Tag::String(text.to_string()))])
}

fn encoded_text_component(text: &[u8]) -> Vec<u8> {
    [
        vec![10, 8, 0, 4],
        b"text".to_vec(),
        vec![0, text.len() as u8],
        text.to_vec(),
        vec![0],
    ]
    .concat()
}

#[test]
fn clientbound_tab_list_packet_matches_java_codec() {
    assert_eq!(CLIENTBOUND_TAB_LIST_PACKET_ID, 122);
    let registry = PlayProtocolRegistry::new();
    assert_eq!(
        registry.clientbound_name(CLIENTBOUND_TAB_LIST_PACKET_ID),
        Some("tab_list")
    );

    let mut payload = Vec::new();
    ClientboundTabListPacket {
        header: text_component("Header"),
        footer: text_component("Footer"),
    }
    .write(&mut payload)
    .unwrap();

    assert_eq!(
        payload,
        [
            encoded_text_component(b"Header"),
            encoded_text_component(b"Footer"),
        ]
        .concat()
    );
}
