use super::*;

fn text_component(text: &str) -> Tag {
    Tag::Compound(vec![("text".to_string(), Tag::String(text.to_string()))])
}

fn text_component_bytes(text: &str) -> Vec<u8> {
    assert!(text.len() <= u16::MAX as usize);

    [
        vec![10, 8, 0, 4],
        b"text".to_vec(),
        (text.len() as u16).to_be_bytes().to_vec(),
        text.as_bytes().to_vec(),
        vec![0],
    ]
    .concat()
}

#[test]
fn clientbound_disguised_chat_packet_matches_java_bound_chat_type_codec() {
    assert_eq!(CLIENTBOUND_DISGUISED_CHAT_PACKET_ID, 33);
    let registry = PlayProtocolRegistry::new();
    assert_eq!(
        registry.clientbound_name(CLIENTBOUND_DISGUISED_CHAT_PACKET_ID),
        Some("disguised_chat")
    );

    let mut payload = Vec::new();
    ClientboundDisguisedChatPacket {
        message: text_component("Hello"),
        chat_type: ChatTypeBound {
            chat_type_id: 0,
            name: text_component("Steve"),
            target_name: Some(text_component("Alex")),
        },
    }
    .write(&mut payload)
    .unwrap();

    assert_eq!(
        payload,
        [
            text_component_bytes("Hello"),
            vec![1],
            text_component_bytes("Steve"),
            vec![1],
            text_component_bytes("Alex"),
        ]
        .concat()
    );
}

#[test]
fn clientbound_disguised_chat_packet_encodes_absent_target_name() {
    let mut payload = Vec::new();
    ClientboundDisguisedChatPacket {
        message: text_component("* waves"),
        chat_type: ChatTypeBound {
            chat_type_id: 6,
            name: text_component("Steve"),
            target_name: None,
        },
    }
    .write(&mut payload)
    .unwrap();

    assert_eq!(
        payload,
        [
            text_component_bytes("* waves"),
            vec![7],
            text_component_bytes("Steve"),
            vec![0],
        ]
        .concat()
    );
}
