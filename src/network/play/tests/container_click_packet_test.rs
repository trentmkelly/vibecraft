use super::*;

#[test]
fn serverbound_container_click_packet_matches_java_codec_order() {
    assert_eq!(SERVERBOUND_CONTAINER_CLICK_PACKET_ID, 18);
    let registry = PlayProtocolRegistry::new();
    assert_eq!(
        registry.serverbound_name(SERVERBOUND_CONTAINER_CLICK_PACKET_ID),
        Some("container_click")
    );

    let mut changed_slots = BTreeMap::new();
    changed_slots.insert(-1, HashedStack::empty());
    changed_slots.insert(
        7,
        HashedStack {
            item_id: Some(300),
            count: 64,
            components: HashedPatchMap::empty(),
        },
    );

    let packet = ServerboundContainerClickPacket {
        container_id: 300,
        state_id: 127,
        slot_num: -32768,
        button_num: -1,
        container_input: ContainerInput::QuickCraft,
        changed_slots,
        carried_item: HashedStack {
            item_id: Some(5),
            count: 1,
            components: HashedPatchMap {
                added_component_hashes: vec![(9, 0x01020304)],
                removed_components: vec![10],
            },
        },
    };

    let mut payload = Vec::new();
    packet.write(&mut payload).unwrap();

    assert_eq!(
        payload,
        vec![
            0xac, 0x02, // container id
            0x7f, // state id
            0x80, 0x00, // slot
            0xff, // button
            5,    // ContainerInput.QUICK_CRAFT
            2,    // changed slot count
            0xff, 0xff, 0, // slot -1 empty HashedStack
            0, 7, 1, 0xac, 0x02, 64, 0, 0, // slot 7 actual HashedStack
            1, 5, 1, 1, 9, 1, 2, 3, 4, 1, 10, // carried actual HashedStack
        ]
    );
    assert_eq!(
        ServerboundContainerClickPacket::read(&mut cursor(payload)).unwrap(),
        packet
    );
}

#[test]
fn serverbound_container_click_packet_limits_changed_slots_to_java_cap() {
    let mut payload = vec![0, 0, 0, 0, 0, 0];
    write_var_i32(&mut payload, 129).unwrap();
    assert!(ServerboundContainerClickPacket::read(&mut cursor(payload)).is_err());

    let err = ServerboundContainerClickPacket {
        container_id: 0,
        state_id: 0,
        slot_num: 0,
        button_num: 0,
        container_input: ContainerInput::Pickup,
        changed_slots: (0..129).map(|slot| (slot, HashedStack::empty())).collect(),
        carried_item: HashedStack::empty(),
    }
    .write(&mut Vec::new())
    .unwrap_err();
    assert_eq!(err.kind(), io::ErrorKind::InvalidInput);
}

#[test]
fn serverbound_container_click_packet_maps_unknown_input_to_pickup() {
    let payload = vec![0, 0, 0, 0, 0, 99, 0, 0];

    let packet = ServerboundContainerClickPacket::read(&mut cursor(payload)).unwrap();

    assert_eq!(packet.container_input, ContainerInput::Pickup);
}
