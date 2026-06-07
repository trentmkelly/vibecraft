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

    let stack = raw_creative_item_stack_to_inventory_stack(&packet.item_stack)?;
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::network::status) enum PickItemOutcome {
    NoItem,
    Picked { inventory_changed: bool },
}

/// Applies Java's middle-click pick-block path for the live play loop.
///
/// Java parity notes:
/// - `ServerGamePacketListenerImpl.handlePickItemFromBlock` first checks
///   `isWithinBlockInteractionRange(pos, 1.0)` and `level.isLoaded(pos)`.
/// - `BlockState.getCloneItemStack` returns the block's item stack; block-entity
///   component copying is intentionally deferred until live block entities expose
///   their saved custom data to this path.
pub(in crate::network::status) fn apply_pick_item_from_block_packet(
    state: &mut PlaySessionState,
    packet: ServerboundPickItemFromBlockPacket,
    world_layout: &WorldLayout,
    chunk_cache: &GeneratedChunkCache,
) -> PickItemOutcome {
    let pos = crate::block_update::BlockPos {
        x: packet.x,
        y: packet.y,
        z: packet.z,
    };
    if !is_within_block_interaction_range(state, pos) {
        return PickItemOutcome::NoItem;
    }
    let Some(block_state) = try_read_block_model_at(chunk_cache, world_layout, pos) else {
        return PickItemOutcome::NoItem;
    };
    let block_name = block_state_model_name(&block_state);
    if block_name == "minecraft:air" {
        return PickItemOutcome::NoItem;
    }
    let Some(item_name) = item_static_name(&block_name) else {
        return PickItemOutcome::NoItem;
    };
    apply_pick_item_stack(state, ItemStack::new(item_name, 1))
}

pub(in crate::network::status) fn apply_pick_item_from_entity_packet(
    _state: &mut PlaySessionState,
    _packet: ServerboundPickItemFromEntityPacket,
) -> PickItemOutcome {
    // Java's base Entity.getPickResult returns null. RustCraft does not yet keep
    // a live pickable mob/entity registry in the play loop, so decoded entity
    // pick packets currently match the Java no-entity/null-pick-result path.
    PickItemOutcome::NoItem
}

fn apply_pick_item_stack(state: &mut PlaySessionState, stack: ItemStack) -> PickItemOutcome {
    if stack.is_empty() {
        return PickItemOutcome::NoItem;
    }

    let selected_slot = state.selected_slot;
    let inventory = state.inventory_menu.player_inventory_mut();
    if (0..HOTBAR_SIZE as i32).contains(&selected_slot) {
        let _ = inventory.set_selected_slot(selected_slot as usize);
    }

    if let Some(slot) =
        (0..INVENTORY_SIZE).find(|&slot| same_item_same_components(inventory.get(slot), &stack))
    {
        if slot < HOTBAR_SIZE {
            let _ = inventory.set_selected_slot(slot);
            state.selected_slot = slot as i32;
            PickItemOutcome::Picked {
                inventory_changed: false,
            }
        } else {
            inventory.pick_slot(slot);
            state.selected_slot = inventory.selected_slot() as i32;
            PickItemOutcome::Picked {
                inventory_changed: true,
            }
        }
    } else {
        if state.abilities.instabuild {
            inventory.add_and_pick_item(stack);
            state.selected_slot = inventory.selected_slot() as i32;
            return PickItemOutcome::Picked {
                inventory_changed: true,
            };
        }
        PickItemOutcome::Picked {
            inventory_changed: false,
        }
    }
}

/// Server-authoritative block interaction reach, 1:1 with Java
/// `Player.isWithinBlockInteractionRange(pos, 1.0)`: the squared AABB distance
/// from the player's eye to the target block must be below
/// `(blockInteractionRange + 1.0)²`, where the range is 4.5 survival / 5.0
/// creative (`Player.DEFAULT_BLOCK_INTERACTION_RANGE` 4.5 plus the creative
/// `+0.5` modifier). Used for both block placement and block breaking.
pub(crate) fn is_within_block_interaction_range(
    state: &PlaySessionState,
    pos: crate::block_update::BlockPos,
) -> bool {
    let block_interaction_range = if state.game_mode == GameMode::Creative {
        5.0
    } else {
        4.5
    };
    let max_range = block_interaction_range + 1.0;
    let eye_x = state.x;
    let eye_y = state.y + 1.62;
    let eye_z = state.z;
    let dx = axis_distance_to_unit_interval(eye_x, pos.x as f64, pos.x as f64 + 1.0);
    let dy = axis_distance_to_unit_interval(eye_y, pos.y as f64, pos.y as f64 + 1.0);
    let dz = axis_distance_to_unit_interval(eye_z, pos.z as f64, pos.z as f64 + 1.0);
    dx * dx + dy * dy + dz * dz < max_range * max_range
}

fn axis_distance_to_unit_interval(point: f64, min: f64, max: f64) -> f64 {
    if point < min {
        min - point
    } else if point > max {
        point - max
    } else {
        0.0
    }
}

#[cfg(test)]
mod reach_tests {
    use super::*;
    use crate::block_update::BlockPos;

    /// `is_within_block_interaction_range` must match Java
    /// `Player.isWithinBlockInteractionRange(pos, 1.0)` boundaries: survival
    /// reaches `(4.5+1.0)²` and creative `(5.0+1.0)²` squared eye-to-AABB
    /// distance. With the eye at (0.5, 1.62, 0.5), a block at x=6 sits at
    /// squared distance 5.5²+0.62² ≈ 30.63 — outside survival's 30.25 but inside
    /// creative's 36.
    #[test]
    fn block_interaction_range_matches_java_survival_and_creative_bounds() {
        let mut state = PlaySessionState {
            x: 0.5,
            y: 0.0,
            z: 0.5,
            game_mode: GameMode::Survival,
            ..PlaySessionState::default()
        };

        // Nearby block (nearest face 4.5 out): within reach in both modes.
        assert!(is_within_block_interaction_range(
            &state,
            BlockPos { x: 5, y: 0, z: 0 }
        ));
        // Far block (6.5 out): out of reach in both modes.
        assert!(!is_within_block_interaction_range(
            &state,
            BlockPos { x: 7, y: 0, z: 0 }
        ));
        // Boundary block (5.5 out): just out of survival reach...
        assert!(!is_within_block_interaction_range(
            &state,
            BlockPos { x: 6, y: 0, z: 0 }
        ));

        // ...but within creative reach.
        state.game_mode = GameMode::Creative;
        assert!(is_within_block_interaction_range(
            &state,
            BlockPos { x: 6, y: 0, z: 0 }
        ));
    }
}
