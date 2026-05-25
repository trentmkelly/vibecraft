use super::super::*;
use crate::item_properties::ItemComponent;

#[test]
pub fn edit_book_packet_live_handler_matches_java_slot_and_component_gates() {
    assert_eq!(SERVERBOUND_EDIT_BOOK_PACKET_ID, 24);
    assert!(play_packet_has_live_status_handler(
        SERVERBOUND_EDIT_BOOK_PACKET_ID
    ));

    let mut state = PlaySessionState::default();
    let packet = ServerboundEditBookPacket {
        slot: 0,
        pages: vec!["first".to_string(), "second".to_string()],
        title: None,
    };
    assert!(!player_book_packets::apply_edit_book_packet(
        &mut state,
        packet.clone(),
        "Alex"
    ));

    state
        .inventory_menu
        .player_inventory_mut()
        .set(0, ItemStack::new("minecraft:writable_book", 1));
    assert!(player_book_packets::apply_edit_book_packet(
        &mut state, packet, "Alex"
    ));
    let edited = state.inventory_menu.player_inventory().get(0);
    assert_eq!(edited.item_id(), "minecraft:writable_book");
    assert_eq!(
        edited.component("minecraft:writable_book_content"),
        Some(&ItemComponent::WritableBookContent(vec![
            "first".to_string(),
            "second".to_string()
        ]))
    );

    let invalid_slot = ServerboundEditBookPacket {
        slot: 9,
        pages: vec!["ignored".to_string()],
        title: None,
    };
    assert!(!player_book_packets::apply_edit_book_packet(
        &mut state,
        invalid_slot,
        "Alex"
    ));

    state
        .inventory_menu
        .player_inventory_mut()
        .set(40, ItemStack::new("minecraft:writable_book", 1));
    assert!(player_book_packets::apply_edit_book_packet(
        &mut state,
        ServerboundEditBookPacket {
            slot: 40,
            pages: vec!["signed page".to_string()],
            title: Some("Guide".to_string()),
        },
        "Alex"
    ));
    let signed = state.inventory_menu.player_inventory().get(40);
    assert_eq!(signed.item_id(), "minecraft:written_book");
    assert!(signed
        .component("minecraft:writable_book_content")
        .is_none());
    assert_eq!(
        signed.component("minecraft:written_book_content"),
        Some(&ItemComponent::WrittenBookContent {
            title: "Guide".to_string(),
            author: "Alex".to_string(),
            generation: 0,
            pages: vec!["signed page".to_string()],
            resolved: true,
        })
    );
}

#[test]
pub fn edit_book_packet_rejects_java_bounded_page_and_title_payloads() {
    let mut too_many_pages = Vec::new();
    write_var_i32(&mut too_many_pages, 0).unwrap();
    write_var_i32(&mut too_many_pages, 101).unwrap();
    assert!(ServerboundEditBookPacket::read(&mut Cursor::new(too_many_pages)).is_err());

    let mut overlong_page = Vec::new();
    write_var_i32(&mut overlong_page, 0).unwrap();
    write_var_i32(&mut overlong_page, 1).unwrap();
    write_var_i32(&mut overlong_page, 1025).unwrap();
    overlong_page.extend(std::iter::repeat_n(b'x', 1025));
    assert!(ServerboundEditBookPacket::read(&mut Cursor::new(overlong_page)).is_err());

    let mut overlong_title = Vec::new();
    write_var_i32(&mut overlong_title, 0).unwrap();
    write_var_i32(&mut overlong_title, 0).unwrap();
    overlong_title.push(1);
    write_var_i32(&mut overlong_title, 33).unwrap();
    overlong_title.extend(std::iter::repeat_n(b't', 33));
    assert!(ServerboundEditBookPacket::read(&mut Cursor::new(overlong_title)).is_err());

    assert!(ServerboundEditBookPacket {
        slot: 0,
        pages: vec!["x".to_string(); 101],
        title: None,
    }
    .write(&mut Vec::new())
    .is_err());
    assert!(ServerboundEditBookPacket {
        slot: 0,
        pages: Vec::new(),
        title: Some("t".repeat(33)),
    }
    .write(&mut Vec::new())
    .is_err());
}

#[test]
pub fn book_item_components_encode_java_network_payloads() {
    let mut writable = ItemStack::new("minecraft:writable_book", 1);
    writable.set_component(ItemComponent::WritableBookContent(
        vec!["Draft".to_string()],
    ));
    let raw = crate::network::play::raw_item_stack_from_item_stack(&writable).unwrap();
    assert_eq!(raw.components.added.len(), 1);
    assert_eq!(raw.components.added[0].0, 54);
    assert_eq!(
        raw.components.added[0].1,
        vec![1, 5, b'D', b'r', b'a', b'f', b't', 0]
    );

    let mut signed = ItemStack::new("minecraft:written_book", 1);
    signed.set_component(ItemComponent::WrittenBookContent {
        title: "Guide".to_string(),
        author: "Alex".to_string(),
        generation: 0,
        pages: vec!["Page".to_string()],
        resolved: true,
    });
    let raw = crate::network::play::raw_item_stack_from_item_stack(&signed).unwrap();
    assert_eq!(raw.components.added.len(), 1);
    assert_eq!(raw.components.added[0].0, 55);
    assert!(raw.components.added[0]
        .1
        .starts_with(&[5, b'G', b'u', b'i', b'd', b'e', 0, 4, b'A', b'l', b'e', b'x', 0, 1]));
    assert_eq!(raw.components.added[0].1.last().copied(), Some(1));
}
