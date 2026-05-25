use super::*;

pub(in crate::network::status) fn apply_edit_book_packet(
    state: &mut PlaySessionState,
    packet: ServerboundEditBookPacket,
    author: &str,
) -> bool {
    let slot = packet.slot;
    if !((0..HOTBAR_SIZE as i32).contains(&slot) || slot == SLOT_OFFHAND as i32) {
        return false;
    }

    let slot = slot as usize;
    let current = state.inventory_menu.player_inventory().get(slot).clone();
    if current
        .component("minecraft:writable_book_content")
        .is_none()
    {
        return false;
    }

    let mut updated = current.clone();
    if let Some(title) = packet.title {
        updated = current.transmute_copy("minecraft:written_book", current.count());
        updated.remove_component("minecraft:writable_book_content");
        updated.set_component(ItemComponent::WrittenBookContent {
            title,
            author: author.to_string(),
            generation: 0,
            pages: packet.pages,
            resolved: true,
        });
    } else {
        updated.set_component(ItemComponent::WritableBookContent(packet.pages));
    }
    state
        .inventory_menu
        .player_inventory_mut()
        .set(slot, updated);
    true
}
