use super::*;

pub(in crate::network::status) fn apply_serverbound_player_abilities_packet(
    state: &mut PlaySessionState,
    packet: ServerboundPlayerAbilitiesPacket,
) {
    state.abilities.flying = packet.is_flying && state.abilities.mayfly;
}

pub(in crate::network::status) fn apply_set_creative_mode_slot_packet(
    state: &mut PlaySessionState,
    packet: ServerboundSetCreativeModeSlotPacket,
) -> Option<ClientboundContainerSetSlotPacket> {
    if !state.abilities.instabuild {
        return None;
    }

    let Some(stack) = raw_creative_item_stack_to_inventory_stack(&packet.item_stack) else {
        return None;
    };
    let valid_data = stack.is_empty() || packet.item_stack.count <= stack.max_stack_size() as i32;
    if !(valid_data && (1..=45).contains(&packet.slot_num)) {
        return None;
    }

    if state
        .inventory_menu
        .set_slot(packet.slot_num as usize, stack)
    {
        state.container_state_id = state.container_state_id.wrapping_add(1);
        Some(ClientboundContainerSetSlotPacket {
            container_id: 0,
            state_id: state.container_state_id,
            slot: packet.slot_num,
            item_stack: packet.item_stack,
        })
    } else {
        None
    }
}

fn raw_creative_item_stack_to_inventory_stack(raw: &RawItemStack) -> Option<ItemStack> {
    if raw.count <= 0 {
        return Some(ItemStack::empty());
    }
    let item_name = item_static_name_from_protocol_id(raw.item_id?)?;
    Some(ItemStack::new(item_name, raw.count))
}
