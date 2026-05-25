use super::*;

#[test]
fn clientbound_recipe_book_remove_packet_matches_java_codec() {
    assert_eq!(CLIENTBOUND_RECIPE_BOOK_REMOVE_PACKET_ID, 75);
    let registry = PlayProtocolRegistry::new();
    assert_eq!(
        registry.clientbound_name(CLIENTBOUND_RECIPE_BOOK_REMOVE_PACKET_ID),
        Some("recipe_book_remove")
    );

    let mut payload = Vec::new();
    ClientboundRecipeBookRemovePacket {
        recipe_display_ids: vec![1, 128, 16_384],
    }
    .write(&mut payload)
    .unwrap();
    assert_eq!(payload, vec![3, 1, 0x80, 0x01, 0x80, 0x80, 0x01]);
}
