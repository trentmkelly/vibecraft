use super::*;

fn component_stack() -> RawItemStack {
    RawItemStack {
        count: 3,
        item_id: Some(42),
        components: RawDataComponentPatch {
            added: vec![(7, vec![0xaa, 0xbb])],
            removed: vec![9],
        },
    }
}

fn trusted_optional_stack_payload() -> Vec<u8> {
    vec![3, 42, 1, 1, 7, 0xaa, 0xbb, 9]
}

#[test]
fn clientbound_inventory_item_stacks_use_java_optional_stream_codec() {
    // Java 26.1.2 clientbound inventory stack packets use
    // ItemStack.OPTIONAL_STREAM_CODEC, whose DataComponentPatch payload is
    // trusted/raw. The length-delimited variant is only for untrusted
    // serverbound inputs such as SetCreativeModeSlot.
    let stack = component_stack();
    let trusted = trusted_optional_stack_payload();

    let mut cursor_item = Vec::new();
    ClientboundSetCursorItemPacket {
        item_stack: stack.clone(),
    }
    .write(&mut cursor_item)
    .unwrap();
    assert_eq!(cursor_item, trusted);

    let mut slot = Vec::new();
    ClientboundContainerSetSlotPacket {
        container_id: 0,
        state_id: 1,
        slot: 5,
        item_stack: stack.clone(),
    }
    .write(&mut slot)
    .unwrap();
    assert_eq!(slot, [vec![0, 1, 0, 5], trusted.clone()].concat());

    let mut content = Vec::new();
    ClientboundContainerPacket {
        container_id: 0,
        state_id: 1,
        slots: vec![stack.clone()],
        carried_item: RawItemStack::empty(),
    }
    .write(&mut content)
    .unwrap();
    assert_eq!(content, [vec![0, 1, 1], trusted.clone(), vec![0]].concat());

    let mut player_inventory = Vec::new();
    ClientboundSetPlayerInventoryPacket {
        slot: 40,
        contents: stack.clone(),
    }
    .write(&mut player_inventory)
    .unwrap();
    assert_eq!(player_inventory, [vec![40], trusted.clone()].concat());

    let mut equipment = Vec::new();
    ClientboundSetEquipmentPacket {
        entity: 300,
        slots: vec![EquipmentEntry {
            slot: EquipmentSlotKind::MainHand,
            item_stack: stack.clone(),
        }],
    }
    .write(&mut equipment)
    .unwrap();
    assert_eq!(
        equipment,
        [vec![0xac, 0x02, 0], trusted.clone()].concat()
    );

    let mut entity_data = Vec::new();
    ClientboundSetEntityDataPacket {
        id: 1,
        packed_items: vec![EntityDataValue::typed(
            7,
            EntityMetadataValue::ItemStack(stack),
        )
        .unwrap()],
    }
    .write(&mut entity_data)
    .unwrap();
    assert_eq!(entity_data, [vec![1, 7, 7], trusted, vec![0xff]].concat());
}
