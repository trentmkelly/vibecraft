use super::*;

#[test]
fn serverbound_container_click_packet_matches_java_codec_order() {
    const SERVERBOUND_CONTAINER_CLICK_PACKET_JAVA: &str = vibecraft_java_source!("/net/minecraft/network/protocol/game/ServerboundContainerClickPacket.java");
    for sentinel in [
        "int containerId, int stateId, short slotNum, byte buttonNum, ContainerInput containerInput, Int2ObjectMap<HashedStack> changedSlots, HashedStack carriedItem",
        "private static final int MAX_SLOT_COUNT = 128;",
        "Int2ObjectOpenHashMap::new, ByteBufCodecs.SHORT.map(Short::intValue, Integer::shortValue), HashedStack.STREAM_CODEC, 128",
        "ByteBufCodecs.CONTAINER_ID",
        "ServerboundContainerClickPacket::containerId",
        "ByteBufCodecs.VAR_INT",
        "ServerboundContainerClickPacket::stateId",
        "ByteBufCodecs.SHORT",
        "ServerboundContainerClickPacket::slotNum",
        "ByteBufCodecs.BYTE",
        "ServerboundContainerClickPacket::buttonNum",
        "ContainerInput.STREAM_CODEC",
        "ServerboundContainerClickPacket::containerInput",
        "SLOTS_STREAM_CODEC",
        "ServerboundContainerClickPacket::changedSlots",
        "HashedStack.STREAM_CODEC",
        "ServerboundContainerClickPacket::carriedItem",
        "changedSlots = Int2ObjectMaps.unmodifiable(changedSlots);",
        "return GamePacketTypes.SERVERBOUND_CONTAINER_CLICK;",
        "listener.handleContainerClick(this);",
    ] {
        assert!(
            SERVERBOUND_CONTAINER_CLICK_PACKET_JAVA.contains(sentinel),
            "missing ServerboundContainerClickPacket sentinel {sentinel}"
        );
    }

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

#[test]
fn hashed_patch_map_and_stack_match_java_create_and_matches_contracts() {
    const HASHED_PATCH_MAP_JAVA: &str = vibecraft_java_source!("/net/minecraft/network/HashedPatchMap.java");
    const HASHED_STACK_JAVA: &str = vibecraft_java_source!("/net/minecraft/network/HashedStack.java");

    for sentinel in [
        "new IdentityHashMap<>(split.added().size())",
        "setComponentHashes.put(e.type(), hasher.apply((TypedDataComponent<?>)e))",
        "if (!split.removed().equals(this.removedComponents))",
        "if (this.addedComponents.size() != split.added().size())",
        "Integer expectedHash = this.addedComponents.get(typedDataComponent.type())",
        "if (!actualHash.equals(expectedHash))",
    ] {
        assert!(
            HASHED_PATCH_MAP_JAVA.contains(sentinel),
            "missing HashedPatchMap sentinel {sentinel}"
        );
    }
    for sentinel in [
        "HashedStack EMPTY = new HashedStack()",
        "return stack.isEmpty();",
        "return itemStack.isEmpty()",
        "itemStack.typeHolder()",
        "itemStack.getCount()",
        "if (this.count != itemStack.getCount())",
        "this.components.matches(itemStack.getComponentsPatch(), hasher)",
    ] {
        assert!(
            HASHED_STACK_JAVA.contains(sentinel),
            "missing HashedStack sentinel {sentinel}"
        );
    }

    fn hash(component_type_id: i32, payload: &[u8]) -> i32 {
        component_type_id * 31 + payload.iter().map(|byte| i32::from(*byte)).sum::<i32>()
    }

    let patch = RawDataComponentPatch {
        added: vec![(9, vec![1, 2, 3]), (7, vec![4])],
        removed: vec![12, 10],
    };
    let hashed = HashedPatchMap::create(&patch, hash);
    assert_eq!(
        hashed.added_component_hashes,
        vec![(7, hash(7, &[4])), (9, hash(9, &[1, 2, 3]))]
    );
    assert_eq!(hashed.removed_components, vec![10, 12]);

    let reordered_removed = RawDataComponentPatch {
        added: vec![(7, vec![4]), (9, vec![1, 2, 3])],
        removed: vec![10, 12],
    };
    assert!(hashed.matches(&reordered_removed, hash));

    let missing_component = RawDataComponentPatch {
        added: vec![(7, vec![4])],
        removed: vec![10, 12],
    };
    assert!(!hashed.matches(&missing_component, hash));

    let wrong_hash = RawDataComponentPatch {
        added: vec![(7, vec![5]), (9, vec![1, 2, 3])],
        removed: vec![10, 12],
    };
    assert!(!hashed.matches(&wrong_hash, hash));

    let stack = RawItemStack {
        count: 3,
        item_id: Some(42),
        components: patch,
    };
    let hashed_stack = HashedStack::create_from_raw(&stack, hash);
    assert_eq!(hashed_stack.item_id, Some(42));
    assert_eq!(hashed_stack.count, 3);
    assert!(hashed_stack.matches_raw(&stack, hash));

    let mut wrong_count = stack.clone();
    wrong_count.count = 4;
    assert!(!hashed_stack.matches_raw(&wrong_count, hash));

    let mut wrong_item = stack.clone();
    wrong_item.item_id = Some(43);
    assert!(!hashed_stack.matches_raw(&wrong_item, hash));

    assert_eq!(
        HashedStack::create_from_raw(&RawItemStack::empty(), hash),
        HashedStack::empty()
    );
    assert!(HashedStack::empty().matches_raw(&RawItemStack::empty(), hash));
    assert!(!HashedStack::empty().matches_raw(&stack, hash));
}
