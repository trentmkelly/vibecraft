use super::*;

#[test]
fn clientbound_container_set_content_packet_matches_java_codec() {
    assert_eq!(CLIENTBOUND_CONTAINER_SET_CONTENT_PACKET_ID, 18);
    let registry = PlayProtocolRegistry::new();
    assert_eq!(
        registry.clientbound_name(CLIENTBOUND_CONTAINER_SET_CONTENT_PACKET_ID),
        Some("container_set_content")
    );

    let mut payload = Vec::new();
    ClientboundContainerPacket {
        container_id: 128,
        state_id: 130,
        slots: vec![
            RawItemStack {
                count: 2,
                item_id: Some(5),
                components: RawDataComponentPatch::empty(),
            },
            RawItemStack::empty(),
        ],
        carried_item: RawItemStack {
            count: 3,
            item_id: Some(42),
            components: RawDataComponentPatch::empty(),
        },
    }
    .write(&mut payload)
    .unwrap();
    assert_eq!(
        payload,
        vec![
            0x80, 0x01, 0x82, 0x01, 2, 2, 5, 0, 0, 0, 3, 42, 0, 0
        ]
    );
}
