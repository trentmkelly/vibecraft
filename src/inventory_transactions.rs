#![allow(dead_code)]

use crate::inventory::{ClickAction, ContainerInput, InventoryAction, Menu};
use crate::item_stack::ItemStack;

const OUTSIDE_SLOT: i32 = -999;
const CARRIED_SLOT: i32 = -1;

#[derive(Debug, Clone, PartialEq)]
pub struct ScriptedContainerClickPacket {
    pub container_id: i32,
    pub state_id: i32,
    pub slot: i32,
    pub button: i32,
    pub mode: ContainerInput,
    pub changed_slots: Vec<(i32, ItemStack)>,
    pub carried: ItemStack,
}

#[derive(Debug, Clone, PartialEq)]
pub struct InventoryTransactionResult {
    pub action: InventoryAction,
    pub accepted: bool,
    pub expected_state_id: i32,
    pub next_state_id: i32,
    pub corrections: Vec<SlotCorrection>,
    pub carried: ItemStack,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SlotCorrection {
    pub slot: i32,
    pub expected: ItemStack,
    pub actual: ItemStack,
}

pub fn apply_scripted_packet(
    menu: &mut Menu,
    expected_state_id: i32,
    packet: &ScriptedContainerClickPacket,
) -> InventoryTransactionResult {
    if packet.state_id != expected_state_id {
        return InventoryTransactionResult {
            action: InventoryAction::Noop,
            accepted: false,
            expected_state_id,
            next_state_id: expected_state_id,
            corrections: collect_corrections(menu, packet),
            carried: menu.carried.clone(),
        };
    }

    let action = match packet.mode {
        ContainerInput::Pickup => {
            let click_action = if packet.button == 1 {
                ClickAction::Secondary
            } else {
                ClickAction::Primary
            };
            if packet.slot == OUTSIDE_SLOT {
                menu.click_pickup(None, click_action)
            } else if let Some(slot) = valid_slot(packet.slot, menu.slots.len()) {
                menu.click_pickup(Some(slot), click_action)
            } else {
                InventoryAction::Noop
            }
        }
        ContainerInput::QuickMove => packet
            .slot
            .try_into()
            .ok()
            .map_or(InventoryAction::Noop, |slot| menu.quick_move(slot)),
        ContainerInput::Swap => {
            match (usize::try_from(packet.slot), usize::try_from(packet.button)) {
                (Ok(slot), Ok(hotbar_slot)) => menu.swap_hotbar(slot, hotbar_slot),
                _ => InventoryAction::Noop,
            }
        }
        ContainerInput::Clone => packet
            .slot
            .try_into()
            .ok()
            .map_or(InventoryAction::Noop, |slot| menu.clone_slot(slot)),
        ContainerInput::Throw => packet
            .slot
            .try_into()
            .ok()
            .map_or(InventoryAction::Noop, |slot| {
                menu.throw_from_slot(slot, packet.button == 1)
            }),
        ContainerInput::QuickCraft => {
            let slots = packet
                .changed_slots
                .iter()
                .filter_map(|(slot, _)| usize::try_from(*slot).ok())
                .collect::<Vec<_>>();
            menu.quick_craft(&slots)
        }
        ContainerInput::PickupAll => packet
            .slot
            .try_into()
            .ok()
            .map_or(InventoryAction::Noop, |slot| menu.pickup_all(slot)),
    };

    let corrections = collect_corrections(menu, packet);
    InventoryTransactionResult {
        action,
        accepted: corrections.is_empty(),
        expected_state_id,
        next_state_id: expected_state_id + 1,
        corrections,
        carried: menu.carried.clone(),
    }
}

fn valid_slot(slot: i32, slot_count: usize) -> Option<usize> {
    usize::try_from(slot)
        .ok()
        .filter(|index| *index < slot_count)
}

fn collect_corrections(menu: &Menu, packet: &ScriptedContainerClickPacket) -> Vec<SlotCorrection> {
    let mut corrections = packet
        .changed_slots
        .iter()
        .filter_map(|(slot, expected)| {
            let actual = usize::try_from(*slot)
                .ok()
                .and_then(|index| menu.slots.get(index))
                .map(|slot| slot.stack.clone())
                .unwrap_or_else(ItemStack::empty);
            (!same_stack_for_transaction(&actual, expected)).then(|| SlotCorrection {
                slot: *slot,
                expected: expected.clone(),
                actual: normalize_empty(actual),
            })
        })
        .collect::<Vec<_>>();

    if !same_stack_for_transaction(&menu.carried, &packet.carried) {
        corrections.push(SlotCorrection {
            slot: CARRIED_SLOT,
            expected: packet.carried.clone(),
            actual: normalize_empty(menu.carried.clone()),
        });
    }

    corrections
}

fn same_stack_for_transaction(a: &ItemStack, b: &ItemStack) -> bool {
    (a.is_empty() && b.is_empty()) || a == b
}

fn normalize_empty(stack: ItemStack) -> ItemStack {
    if stack.is_empty() {
        ItemStack::empty()
    } else {
        stack
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inventory::Slot;

    fn packet(
        state_id: i32,
        slot: i32,
        button: i32,
        mode: ContainerInput,
        changed_slots: Vec<(i32, ItemStack)>,
        carried: ItemStack,
    ) -> ScriptedContainerClickPacket {
        ScriptedContainerClickPacket {
            container_id: 0,
            state_id,
            slot,
            button,
            mode,
            changed_slots,
            carried,
        }
    }

    #[test]
    fn scripted_pickup_and_place_sequence_tracks_state_and_carried_stack() {
        let mut menu = Menu::new(2);
        menu.slots[0] = Slot::with_stack(ItemStack::new("minecraft:stick", 6));

        let pickup = apply_scripted_packet(
            &mut menu,
            10,
            &packet(
                10,
                0,
                0,
                ContainerInput::Pickup,
                vec![(0, ItemStack::empty())],
                ItemStack::new("minecraft:stick", 6),
            ),
        );
        assert_eq!(
            pickup.action,
            InventoryAction::PickedUp { slot: 0, count: 6 }
        );
        assert!(pickup.accepted);
        assert_eq!(pickup.next_state_id, 11);

        let place = apply_scripted_packet(
            &mut menu,
            pickup.next_state_id,
            &packet(
                11,
                1,
                1,
                ContainerInput::Pickup,
                vec![(1, ItemStack::new("minecraft:stick", 1))],
                ItemStack::new("minecraft:stick", 5),
            ),
        );
        assert_eq!(place.action, InventoryAction::Placed { slot: 1, count: 1 });
        assert!(place.accepted);
        assert_eq!(menu.slots[1].stack.count(), 1);
        assert_eq!(menu.carried.count(), 5);
    }

    #[test]
    fn scripted_quick_move_and_hotbar_swap_use_packet_buttons() {
        let mut menu = Menu::new(3);
        menu.slots[0] = Slot::with_stack(ItemStack::new("minecraft:stick", 1));
        menu.hotbar[4] = ItemStack::new("minecraft:apple", 1);

        let quick_move = apply_scripted_packet(
            &mut menu,
            0,
            &packet(
                0,
                0,
                0,
                ContainerInput::QuickMove,
                vec![
                    (0, ItemStack::empty()),
                    (1, ItemStack::new("minecraft:stick", 1)),
                ],
                ItemStack::empty(),
            ),
        );
        assert_eq!(
            quick_move.action,
            InventoryAction::QuickMoved {
                from: 0,
                to: 1,
                count: 1
            }
        );
        assert!(quick_move.accepted);

        let swap = apply_scripted_packet(
            &mut menu,
            1,
            &packet(
                1,
                1,
                4,
                ContainerInput::Swap,
                vec![(1, ItemStack::new("minecraft:apple", 1))],
                ItemStack::empty(),
            ),
        );
        assert_eq!(
            swap.action,
            InventoryAction::Swapped {
                slot: 1,
                hotbar_slot: 4
            }
        );
        assert!(swap.accepted);
        assert_eq!(menu.hotbar[4].item_id(), "minecraft:stick");
    }

    #[test]
    fn stale_state_id_rejects_packet_and_reports_server_corrections() {
        let mut menu = Menu::new(1);
        menu.slots[0] = Slot::with_stack(ItemStack::new("minecraft:stone", 2));

        let rejected = apply_scripted_packet(
            &mut menu,
            8,
            &packet(
                7,
                0,
                0,
                ContainerInput::Pickup,
                vec![(0, ItemStack::empty())],
                ItemStack::new("minecraft:stone", 2),
            ),
        );
        assert!(!rejected.accepted);
        assert_eq!(rejected.action, InventoryAction::Noop);
        assert_eq!(rejected.next_state_id, 8);
        assert_eq!(rejected.corrections.len(), 2);
        assert_eq!(rejected.corrections[0].slot, 0);
        assert_eq!(rejected.corrections[0].actual.count(), 2);
        assert_eq!(rejected.corrections[1].slot, CARRIED_SLOT);
    }

    #[test]
    fn accepted_action_can_still_request_changed_slot_or_carried_correction() {
        let mut menu = Menu::new(1);
        menu.slots[0] = Slot::with_stack(ItemStack::new("minecraft:apple", 3));

        let result = apply_scripted_packet(
            &mut menu,
            0,
            &packet(
                0,
                0,
                0,
                ContainerInput::Pickup,
                vec![(0, ItemStack::new("minecraft:apple", 1))],
                ItemStack::empty(),
            ),
        );
        assert!(!result.accepted);
        assert_eq!(
            result.action,
            InventoryAction::PickedUp { slot: 0, count: 3 }
        );
        assert_eq!(result.corrections.len(), 2);
        assert_eq!(result.corrections[0].actual, ItemStack::empty());
        assert_eq!(result.corrections[1].actual.count(), 3);
    }

    #[test]
    fn scripted_throw_clone_quickcraft_and_pickup_all_modes_are_mapped() {
        let mut menu = Menu::new(4);
        menu.slots[0] = Slot::with_stack(ItemStack::new("minecraft:stick", 3));

        let throw = apply_scripted_packet(
            &mut menu,
            0,
            &packet(
                0,
                0,
                1,
                ContainerInput::Throw,
                vec![(0, ItemStack::empty())],
                ItemStack::empty(),
            ),
        );
        assert_eq!(
            throw.action,
            InventoryAction::Dropped {
                slot: Some(0),
                count: 3
            }
        );
        assert!(throw.accepted);

        menu.creative = true;
        menu.slots[1] = Slot::with_stack(ItemStack::new("minecraft:ender_pearl", 2));
        let clone = apply_scripted_packet(
            &mut menu,
            1,
            &packet(
                1,
                1,
                2,
                ContainerInput::Clone,
                vec![],
                ItemStack::new("minecraft:ender_pearl", 16),
            ),
        );
        assert_eq!(clone.action, InventoryAction::Cloned { slot: 1 });
        assert!(clone.accepted);

        let quickcraft = apply_scripted_packet(
            &mut menu,
            2,
            &packet(
                2,
                -1,
                0,
                ContainerInput::QuickCraft,
                vec![
                    (2, ItemStack::new("minecraft:ender_pearl", 8)),
                    (3, ItemStack::new("minecraft:ender_pearl", 8)),
                ],
                ItemStack::empty(),
            ),
        );
        assert_eq!(
            quickcraft.action,
            InventoryAction::QuickCrafted { slots: 2, each: 8 }
        );
        assert!(quickcraft.accepted);

        menu.carried = ItemStack::new("minecraft:stick", 1);
        menu.slots[0] = Slot::with_stack(ItemStack::new("minecraft:stick", 2));
        menu.slots[2] = Slot::with_stack(ItemStack::new("minecraft:stick", 1));
        let pickup_all = apply_scripted_packet(
            &mut menu,
            3,
            &packet(
                3,
                0,
                0,
                ContainerInput::PickupAll,
                vec![(0, ItemStack::empty()), (2, ItemStack::empty())],
                ItemStack::new("minecraft:stick", 4),
            ),
        );
        assert_eq!(pickup_all.action, InventoryAction::PickedUpAll { count: 3 });
        assert!(pickup_all.accepted);
    }

    #[test]
    fn scripted_outside_pickup_drops_from_carried_stack() {
        let mut menu = Menu::new(0);
        menu.carried = ItemStack::new("minecraft:stick", 4);

        let drop = apply_scripted_packet(
            &mut menu,
            0,
            &packet(
                0,
                OUTSIDE_SLOT,
                1,
                ContainerInput::Pickup,
                vec![],
                ItemStack::new("minecraft:stick", 3),
            ),
        );
        assert_eq!(
            drop.action,
            InventoryAction::Dropped {
                slot: None,
                count: 1
            }
        );
        assert!(drop.accepted);
        assert_eq!(menu.dropped[0].count(), 1);
    }

    #[test]
    fn scripted_invalid_pickup_slot_is_noop_without_treating_it_as_outside_drop() {
        let mut menu = Menu::new(1);
        menu.carried = ItemStack::new("minecraft:stick", 4);

        let negative = apply_scripted_packet(
            &mut menu,
            0,
            &packet(
                0,
                CARRIED_SLOT,
                1,
                ContainerInput::Pickup,
                vec![],
                ItemStack::new("minecraft:stick", 4),
            ),
        );
        assert_eq!(negative.action, InventoryAction::Noop);
        assert!(negative.accepted);
        assert_eq!(menu.carried.count(), 4);
        assert!(menu.dropped.is_empty());

        let out_of_range = apply_scripted_packet(
            &mut menu,
            1,
            &packet(
                1,
                99,
                0,
                ContainerInput::Pickup,
                vec![],
                ItemStack::new("minecraft:stick", 4),
            ),
        );
        assert_eq!(out_of_range.action, InventoryAction::Noop);
        assert!(out_of_range.accepted);
        assert_eq!(menu.carried.count(), 4);
        assert!(menu.dropped.is_empty());
    }
}
