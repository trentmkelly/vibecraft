use super::*;

#[test]
fn clientbound_set_equipment_packet_matches_java_codec() {
    assert_eq!(CLIENTBOUND_SET_EQUIPMENT_PACKET_ID, 102);
    let registry = PlayProtocolRegistry::new();
    assert_eq!(
        registry.clientbound_name(CLIENTBOUND_SET_EQUIPMENT_PACKET_ID),
        Some("set_equipment")
    );

    // Java writes EquipmentSlot.ordinal(), not EquipmentSlot.getId().
    assert_eq!(EquipmentSlotKind::MainHand as u8, 0);
    assert_eq!(EquipmentSlotKind::OffHand as u8, 1);
    assert_eq!(EquipmentSlotKind::Feet as u8, 2);
    assert_eq!(EquipmentSlotKind::Legs as u8, 3);
    assert_eq!(EquipmentSlotKind::Chest as u8, 4);
    assert_eq!(EquipmentSlotKind::Head as u8, 5);
    assert_eq!(EquipmentSlotKind::Body as u8, 6);
    assert_eq!(EquipmentSlotKind::Saddle as u8, 7);

    let packet = ClientboundSetEquipmentPacket {
        entity: 300,
        slots: vec![
            EquipmentEntry {
                slot: EquipmentSlotKind::MainHand,
                item_stack: RawItemStack::empty(),
            },
            EquipmentEntry {
                slot: EquipmentSlotKind::Body,
                item_stack: RawItemStack::empty(),
            },
            EquipmentEntry {
                slot: EquipmentSlotKind::Saddle,
                item_stack: RawItemStack::empty(),
            },
        ],
    };

    assert_eq!(packet.encoded_slot_bytes(), vec![0x80, 0x86, 0x07]);

    let mut payload = Vec::new();
    packet.write(&mut payload).unwrap();

    assert_eq!(
        payload,
        vec![
            0xac, 0x02, // entity id 300
            0x80, 0x00, // continued MAINHAND + empty optional stack
            0x86, 0x00, // continued BODY + empty optional stack
            0x07, 0x00, // final SADDLE + empty optional stack
        ]
    );
}
