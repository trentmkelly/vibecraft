use super::*;

#[test]
fn clientbound_container_set_slot_packet_matches_java_codec() {
    assert_eq!(CLIENTBOUND_CONTAINER_SET_SLOT_PACKET_ID, 20);
    let registry = PlayProtocolRegistry::new();
    assert_eq!(
        registry.clientbound_name(CLIENTBOUND_CONTAINER_SET_SLOT_PACKET_ID),
        Some("container_set_slot")
    );

    let mut payload = Vec::new();
    ClientboundContainerSetSlotPacket {
        container_id: 128,
        state_id: 130,
        slot: -3,
        item_stack: RawItemStack {
            count: 2,
            item_id: Some(5),
            components: RawDataComponentPatch::empty(),
        },
    }
    .write(&mut payload)
    .unwrap();
    assert_eq!(
        payload,
        vec![0x80, 0x01, 0x82, 0x01, 0xff, 0xfd, 2, 5, 0, 0]
    );
}
