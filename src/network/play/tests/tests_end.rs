use super::*;

// ── 2×2 crafting parity tests ──────────────────────────────────────────────────────────────

/// Full network round-trip for log → planks:
///   1. Client places one oak log in grid slot 1.
///   2. Server responds with ContainerSetSlot slot=0 (4 planks) and slot=1 (log).
///   3. Client clicks result slot 0 to take.
///   4. Server responds: slot=0 empty, slot=1 empty, cursor = 4 planks, recipe unlock emitted.
#[test]
fn crafting_grid_log_to_planks_full_round_trip() {
    let recipes = network_crafting_test_recipes();
    let mut inventory_menu = InventoryMenu::new(
        crate::player_inventory::PlayerInventory::new(),
        recipes.clone(),
    );
    let mut state_id: i32 = 0;
    let mut carried = ItemStack::new("minecraft:oak_log", 1);

    let instructions = place_log_in_crafting_grid(&mut inventory_menu, &mut state_id, &mut carried);
    assert_log_placed_in_crafting_grid(&inventory_menu, &instructions, state_id, &carried);

    let instructions = take_crafting_result(&mut inventory_menu, &mut state_id, &mut carried);
    assert_log_to_planks_result_taken(&inventory_menu, &instructions, state_id, &carried);
}

fn container_pickup_click(state_id: i32, slot_num: i16) -> ServerboundContainerClickPacket {
    container_pickup_click_with_button(state_id, slot_num, 0)
}

fn container_pickup_click_with_button(
    state_id: i32,
    slot_num: i16,
    button_num: i8,
) -> ServerboundContainerClickPacket {
    ServerboundContainerClickPacket {
        container_id: 0,
        state_id,
        slot_num,
        button_num,
        container_input: ContainerInput::Pickup,
        changed_slots: BTreeMap::new(),
        carried_item: HashedStack::empty(),
    }
}

fn place_log_in_crafting_grid(
    inventory_menu: &mut InventoryMenu,
    state_id: &mut i32,
    carried: &mut ItemStack,
) -> Vec<PlayInstruction> {
    let place_log = container_pickup_click(0, 1);
    handle_container_click(&place_log, state_id, inventory_menu, carried)
}

fn assert_log_placed_in_crafting_grid(
    inventory_menu: &InventoryMenu,
    instructions: &[PlayInstruction],
    state_id: i32,
    carried: &ItemStack,
) {
    assert_eq!(state_id, 1);
    assert!(
        carried.is_empty(),
        "log should have been placed into slot 1, cursor empty"
    );
    assert_eq!(
        inventory_menu.get_slot(1),
        Some(ItemStack::new("minecraft:oak_log", 1))
    );
    assert_eq!(
        inventory_menu.get_slot(0),
        Some(ItemStack::new("minecraft:oak_planks", 4))
    );
    assert_has_container_set_slot(
        instructions,
        0,
        4,
        "expected ContainerSetSlot slot=0 count=4 planks",
    );
    assert_has_container_set_slot(
        instructions,
        1,
        1,
        "expected ContainerSetSlot slot=1 count=1 log",
    );
}

fn take_crafting_result(
    inventory_menu: &mut InventoryMenu,
    state_id: &mut i32,
    carried: &mut ItemStack,
) -> Vec<PlayInstruction> {
    let take_result = container_pickup_click(1, 0);
    handle_container_click(&take_result, state_id, inventory_menu, carried)
}

fn assert_log_to_planks_result_taken(
    inventory_menu: &InventoryMenu,
    instructions: &[PlayInstruction],
    state_id: i32,
    carried: &ItemStack,
) {
    assert_eq!(state_id, 2);
    assert_eq!(
        *carried,
        ItemStack::new("minecraft:oak_planks", 4),
        "cursor should hold 4 planks"
    );
    assert_eq!(
        inventory_menu.get_slot(1),
        Some(ItemStack::empty()),
        "log should be consumed"
    );
    assert_eq!(
        inventory_menu.get_slot(0),
        Some(ItemStack::empty()),
        "result slot should be empty"
    );
    assert_has_container_set_slot(
        instructions,
        0,
        0,
        "expected ContainerSetSlot slot=0 count=0 (empty result)",
    );
    assert_has_container_set_slot(
        instructions,
        1,
        0,
        "expected ContainerSetSlot slot=1 count=0 (log consumed)",
    );
    assert!(
        instructions.iter().any(|i| matches!(
            i,
            PlayInstruction::SetCursorItem(p)
                if p.item_stack.count == 4
                    && p.item_stack.item_id == item_protocol_id("minecraft:oak_planks")
        )),
        "expected SetCursorItem with 4 oak planks"
    );
    assert!(
        instructions.iter().any(|i| matches!(
            i,
            PlayInstruction::RecipesUnlocked(ids) if ids.contains(&"minecraft:oak_planks")
        )),
        "expected RecipesUnlocked with oak_planks on first craft"
    );
}

fn assert_has_container_set_slot(
    instructions: &[PlayInstruction],
    slot: i16,
    count: i32,
    message: &str,
) {
    assert!(
        instructions.iter().any(|i| matches!(
            i,
            PlayInstruction::ContainerSetSlot(p)
                if p.slot == slot && p.item_stack.count == count
        )),
        "{message}"
    );
}

#[test]
fn first_craft_recipe_unlock_builds_highlighted_notification_packet() {
    let recipes = network_crafting_test_recipes();
    let packet = build_recipe_book_add(&["minecraft:oak_planks"], &recipes)
        .expect("known recipe unlock should build recipe-book add packet");

    assert!(!packet.replace);
    assert_eq!(packet.entries.len(), 1);
    assert_eq!(
        packet.entries[0].flags,
        RecipeBookAddEntry::FLAG_NOTIFICATION | RecipeBookAddEntry::FLAG_HIGHLIGHT
    );
}

#[test]
fn vanilla_oak_log_pickup_to_inventory_grid_populates_planks_result() {
    let recipes = vanilla_recipe_map();
    let mut inventory_menu = InventoryMenu::new(
        crate::player_inventory::PlayerInventory::new(),
        recipes.clone(),
    );
    let mut carried = ItemStack::empty();
    let mut state_id: i32 = 1;

    inventory_menu
        .player_inventory_mut()
        .add(ItemStack::new("minecraft:oak_log", 1));

    let pickup_from_hotbar = ServerboundContainerClickPacket {
        container_id: 0,
        state_id,
        slot_num: 36,
        button_num: 0,
        container_input: ContainerInput::Pickup,
        changed_slots: BTreeMap::new(),
        carried_item: HashedStack::empty(),
    };
    handle_container_click(
        &pickup_from_hotbar,
        &mut state_id,
        &mut inventory_menu,
        &mut carried,
    );
    assert_eq!(state_id, 2);
    assert_eq!(carried.item_id(), "minecraft:oak_log");
    assert_eq!(carried.count(), 1);

    let place_in_grid = ServerboundContainerClickPacket {
        container_id: 0,
        state_id,
        slot_num: 1,
        button_num: 0,
        container_input: ContainerInput::Pickup,
        changed_slots: BTreeMap::new(),
        carried_item: HashedStack::empty(),
    };
    let instructions = handle_container_click(
        &place_in_grid,
        &mut state_id,
        &mut inventory_menu,
        &mut carried,
    );

    assert_eq!(state_id, 3);
    assert!(carried.is_empty());
    assert_eq!(
        inventory_menu.get_slot(0),
        Some(ItemStack::new("minecraft:oak_planks", 4)),
        "vanilla #minecraft:oak_logs recipe must populate the 2x2 result slot"
    );
    assert!(
        instructions.iter().any(|i| matches!(
            i,
            PlayInstruction::ContainerSetSlot(p)
                if p.slot == 0
                    && p.item_stack.count == 4
                    && p.item_stack.item_id == item_protocol_id("minecraft:oak_planks")
        )),
        "server must send the client a result-slot update for oak planks"
    );
}

#[test]
fn vanilla_two_by_two_planks_send_crafting_table_result_slot() {
    let recipes = vanilla_recipe_map();
    let mut inventory_menu =
        InventoryMenu::new(crate::player_inventory::PlayerInventory::new(), recipes);
    let mut carried = ItemStack::new("minecraft:oak_planks", 4);
    let mut state_id: i32 = 0;
    let mut last_instructions = Vec::new();

    for slot_num in 1..=4 {
        let place_one_plank = container_pickup_click_with_button(state_id, slot_num, 1);
        last_instructions = handle_container_click(
            &place_one_plank,
            &mut state_id,
            &mut inventory_menu,
            &mut carried,
        );
    }

    assert_eq!(state_id, 4);
    assert!(carried.is_empty());
    assert_eq!(
        inventory_menu.get_slot(0),
        Some(ItemStack::new("minecraft:crafting_table", 1)),
        "vanilla crafting_table recipe must populate the 2x2 result slot"
    );
    assert!(
        last_instructions.iter().any(|i| matches!(
            i,
            PlayInstruction::ContainerSetSlot(p)
                if p.slot == 0
                    && p.item_stack.count == 1
                    && p.item_stack.item_id == item_protocol_id("minecraft:crafting_table")
        )),
        "server must send the client a result-slot update for the crafting table"
    );

    let take_result = container_pickup_click(state_id, 0);
    let take_instructions = handle_container_click(
        &take_result,
        &mut state_id,
        &mut inventory_menu,
        &mut carried,
    );

    assert_eq!(state_id, 5);
    assert_eq!(carried, ItemStack::new("minecraft:crafting_table", 1));
    assert_eq!(inventory_menu.get_slot(0), Some(ItemStack::empty()));
    for slot_num in 1..=4 {
        assert_eq!(inventory_menu.get_slot(slot_num), Some(ItemStack::empty()));
    }
    assert!(
        take_instructions.iter().any(|i| matches!(
            i,
            PlayInstruction::SetCursorItem(p)
                if p.item_stack.count == 1
                    && p.item_stack.item_id == item_protocol_id("minecraft:crafting_table")
        )),
        "taking the result must move the crafting table onto the cursor"
    );
}

/// Parity test: result slot updates after each grid change; stale state ID still applies the
/// click and returns a full resync; second craft of the same recipe does NOT emit another unlock.
#[test]
fn crafting_grid_result_updates_per_slot_change_stale_id_corrected_no_double_unlock() {
    let recipes = network_crafting_test_recipes();
    let mut inventory_menu = InventoryMenu::new(
        crate::player_inventory::PlayerInventory::new(),
        recipes.clone(),
    );
    let mut carried = ItemStack::new("minecraft:oak_log", 1);
    let mut state_id: i32 = 0;

    place_first_log_and_assert_result(&mut inventory_menu, &mut state_id, &mut carried);
    apply_stale_grid_click_and_assert_resync(&mut inventory_menu, &mut state_id, &mut carried);
    take_first_result_and_assert_unlock(&mut inventory_menu, &mut state_id, &mut carried);
    craft_second_log_and_assert_no_unlock(&mut inventory_menu, &mut state_id);
}

fn place_first_log_and_assert_result(
    inventory_menu: &mut InventoryMenu,
    state_id: &mut i32,
    carried: &mut ItemStack,
) {
    let place1 = container_pickup_click(0, 1);
    handle_container_click(&place1, state_id, inventory_menu, carried);
    assert_eq!(*state_id, 1);
    assert_eq!(
        inventory_menu.get_slot(0),
        Some(ItemStack::new("minecraft:oak_planks", 4)),
        "result must update immediately after placing log in grid"
    );
}

fn apply_stale_grid_click_and_assert_resync(
    inventory_menu: &mut InventoryMenu,
    state_id: &mut i32,
    carried: &mut ItemStack,
) {
    let stale_click = container_pickup_click(0, 2);
    let corrections = handle_container_click(&stale_click, state_id, inventory_menu, carried);
    assert_eq!(
        *state_id, 2,
        "stale click still advances state ID after applying"
    );
    let set_slot_count = corrections
        .iter()
        .filter(|i| matches!(i, PlayInstruction::ContainerSetSlot(_)))
        .count();
    assert_eq!(
        set_slot_count,
        InventoryMenu::SLOT_COUNT,
        "stale click must send a full resync for all 46 slots"
    );
}

fn take_first_result_and_assert_unlock(
    inventory_menu: &mut InventoryMenu,
    state_id: &mut i32,
    carried: &mut ItemStack,
) {
    let take1 = container_pickup_click(2, 0);
    let take1_instrs = handle_container_click(&take1, state_id, inventory_menu, carried);
    assert_eq!(*state_id, 3);
    assert_eq!(*carried, ItemStack::new("minecraft:oak_planks", 4));
    assert_eq!(
        inventory_menu.get_slot(1),
        Some(ItemStack::empty()),
        "log must be consumed"
    );
    assert_eq!(
        inventory_menu.get_slot(0),
        Some(ItemStack::empty()),
        "result must clear"
    );
    assert!(
        take1_instrs.iter().any(|i| matches!(
            i,
            PlayInstruction::RecipesUnlocked(ids) if ids.contains(&"minecraft:oak_planks")
        )),
        "first craft must emit RecipesUnlocked"
    );
}

fn craft_second_log_and_assert_no_unlock(inventory_menu: &mut InventoryMenu, state_id: &mut i32) {
    let mut carried2 = ItemStack::new("minecraft:oak_log", 1);
    let place2 = container_pickup_click(3, 1);
    handle_container_click(&place2, state_id, inventory_menu, &mut carried2);
    let take2 = container_pickup_click(4, 0);
    let take2_instrs = handle_container_click(&take2, state_id, inventory_menu, &mut carried2);
    assert!(
        !take2_instrs
            .iter()
            .any(|i| matches!(i, PlayInstruction::RecipesUnlocked(_))),
        "second craft of the same recipe must NOT emit another RecipesUnlocked"
    );
}

#[test]
fn stale_inventory_menu_click_still_places_carried_item_before_full_resync() {
    let recipes = network_crafting_test_recipes();
    let mut inventory_menu = InventoryMenu::new(
        crate::player_inventory::PlayerInventory::new(),
        recipes.clone(),
    );
    let mut carried = ItemStack::new("minecraft:oak_log", 1);
    let mut state_id: i32 = 4;

    let stale_place = ServerboundContainerClickPacket {
        container_id: 0,
        state_id: 3,
        slot_num: 1,
        button_num: 0,
        container_input: ContainerInput::Pickup,
        changed_slots: BTreeMap::new(),
        carried_item: HashedStack::empty(),
    };
    let instructions = handle_container_click(
        &stale_place,
        &mut state_id,
        &mut inventory_menu,
        &mut carried,
    );

    assert_eq!(state_id, 5);
    assert!(
        carried.is_empty(),
        "cursor item must move into the clicked slot"
    );
    assert_eq!(
        inventory_menu.get_slot(1),
        Some(ItemStack::new("minecraft:oak_log", 1))
    );
    assert_eq!(
        inventory_menu.get_slot(0),
        Some(ItemStack::new("minecraft:oak_planks", 4))
    );
    assert_eq!(
        instructions
            .iter()
            .filter(|i| matches!(i, PlayInstruction::ContainerSetSlot(_)))
            .count(),
        InventoryMenu::SLOT_COUNT
    );
    assert!(instructions.iter().any(|i| matches!(
        i,
        PlayInstruction::SetCursorItem(p) if p.item_stack.count == 0
    )));
}

/// Regression test for the pickup→crafting state-ID desync bug.
///
/// When a player picks up a ground item, the server advances `container_state_id` and
/// sends a `ContainerSetContent` carrying the new value.  If instead a `SetPlayerInventory`
/// packet were sent (which carries no state_id), the client would still hold the old
/// state_id, causing the very next `ContainerClick` to be treated as stale and rejected,
/// leaving the crafting result slot empty even though the ingredients are in the grid.
///
/// This test simulates that scenario at the `handle_container_click` level by manually
/// advancing `state_id` (mimicking what `process_item_pickups` does when it sends
/// `ContainerSetContent`) before the player places an ingredient.  The click must be
/// accepted and the result slot must populate with planks.
#[test]
fn crafting_after_pickup_state_id_advanced_externally() {
    let recipes = network_crafting_test_recipes();
    let mut inventory_menu = InventoryMenu::new(
        crate::player_inventory::PlayerInventory::new(),
        recipes.clone(),
    );
    let mut carried = ItemStack::new("minecraft:oak_log", 1);

    // Simulate the state_id that the server advances when it sends ContainerSetContent
    // after a ground-item pickup.  The client receives this packet and knows state_id=1.
    let mut state_id: i32 = 1;

    // Player now places the log into crafting slot 1 using the updated state_id.
    let place_log = ServerboundContainerClickPacket {
        container_id: 0,
        state_id: 1, // client echoes back the state_id it learned from ContainerSetContent
        slot_num: 1,
        button_num: 0,
        container_input: ContainerInput::Pickup,
        changed_slots: BTreeMap::new(),
        carried_item: HashedStack::empty(),
    };
    let instructions =
        handle_container_click(&place_log, &mut state_id, &mut inventory_menu, &mut carried);

    assert!(
        carried.is_empty(),
        "log must have moved from cursor to grid"
    );
    assert_eq!(
        inventory_menu.get_slot(0),
        Some(ItemStack::new("minecraft:oak_planks", 4)),
        "result slot must show 4 planks immediately after placing the log"
    );
    assert!(
        instructions.iter().any(|i| matches!(
            i,
            PlayInstruction::ContainerSetSlot(p) if p.slot == 0 && p.item_stack.count == 4
        )),
        "server must send ContainerSetSlot for result slot with 4 planks"
    );
}

#[test]
fn serverbound_scalar_packet_payload_validation_rejects_malformed_inputs() {
    assert_scalar_state_payloads_reject_malformed_inputs();
    assert_action_payloads_reject_malformed_inputs();
    assert_use_item_payloads_reject_malformed_inputs();
}

fn assert_scalar_state_payloads_reject_malformed_inputs() {
    assert!(ServerboundAcceptTeleportationPacket::read(&mut cursor(Vec::<u8>::new())).is_err());
    assert!(ServerboundAcceptTeleportationPacket::read(&mut cursor(vec![0x80])).is_err());
    assert!(ServerboundAcceptTeleportationPacket::read(&mut cursor(vec![0xac, 0x02, 0])).is_err());
    assert!(ServerboundClientCommandPacket::read(&mut cursor(Vec::<u8>::new())).is_err());
    assert!(ServerboundClientCommandPacket::read(&mut cursor(vec![3])).is_err());
    assert!(ServerboundClientCommandPacket::read(&mut cursor(vec![1, 0])).is_err());
    assert!(
        ServerboundChunkBatchReceivedPacket::read(&mut cursor(vec![0x7f, 0x7f, 0x7f])).is_err()
    );
    assert!(ServerboundLockDifficultyPacket::read(&mut cursor(Vec::<u8>::new())).is_err());
    assert!(ServerboundLockDifficultyPacket::read(&mut cursor(vec![1, 0])).is_err());
    assert!(ServerboundPaddleBoatPacket::read(&mut cursor(vec![1])).is_err());
    assert!(ServerboundPaddleBoatPacket::read(&mut cursor(vec![1, 0, 1])).is_err());
    assert!(ServerboundPlayerInputPacket::read(&mut cursor(Vec::<u8>::new())).is_err());
    assert!(ServerboundPlayerInputPacket::read(&mut cursor(vec![0x55, 0])).is_err());
    assert!(ServerboundClientTickEndPacket::read(&mut cursor(vec![1])).is_err());
    assert!(ServerboundPlayerLoadedPacket::read(&mut cursor(vec![2])).is_err());
    assert!(ServerboundChangeDifficultyPacket::read(&mut cursor(vec![0x80])).is_err());
    assert!(ServerboundChangeDifficultyPacket::read(&mut cursor(vec![1, 0])).is_err());
    assert!(ServerboundSwingPacket::read(&mut cursor(Vec::<u8>::new())).is_err());
    assert!(ServerboundSwingPacket::read(&mut cursor(vec![2])).is_err());
    assert!(ServerboundSwingPacket::read(&mut cursor(vec![1, 0])).is_err());
}

fn assert_action_payloads_reject_malformed_inputs() {
    assert!(ServerboundPlayerCommandPacket::read(&mut cursor(Vec::<u8>::new())).is_err());
    assert!(ServerboundPlayerCommandPacket::read(&mut cursor(vec![37, 7])).is_err());
    assert!(ServerboundPlayerCommandPacket::read(&mut cursor(vec![37, 3])).is_err());
    assert!(ServerboundPlayerCommandPacket::read(&mut cursor(vec![37, 3, 0x80, 0x01, 0])).is_err());
    assert!(ServerboundInteractPacket::read(&mut cursor(Vec::<u8>::new())).is_err());
    assert!(ServerboundInteractPacket::read(&mut cursor(vec![128, 1, 1])).is_err());
    assert!(ServerboundInteractPacket::read(&mut cursor(vec![128, 1, 1, 0, 1, 0])).is_err());
    assert!(ServerboundPlayerActionPacket::read(&mut cursor(Vec::<u8>::new())).is_err());
    assert!(ServerboundPlayerActionPacket::read(&mut cursor(vec![8])).is_err());
    assert!(ServerboundPlayerActionPacket::read(&mut cursor(vec![2])).is_err());
    assert!(ServerboundPlayerActionPacket::read(&mut cursor(vec![
        2, 0xff, 0xff, 0xfd, 0x00, 0x00, 0x02, 0x20, 0x40, 4, 0xac, 0x02, 0
    ]))
    .is_err());
}

fn assert_use_item_payloads_reject_malformed_inputs() {
    assert!(ServerboundUseItemPacket::read(&mut cursor(Vec::<u8>::new())).is_err());
    assert!(ServerboundUseItemPacket::read(&mut cursor(vec![2])).is_err());
    assert!(ServerboundUseItemPacket::read(&mut cursor(vec![0])).is_err());
    assert!(ServerboundUseItemPacket::read(&mut cursor(vec![
        1, 0xac, 0x02, 0x42, 0x34, 0x00, 0x00, 0xc1, 0x28, 0x00, 0x00, 0
    ]))
    .is_err());
    assert!(ServerboundUseItemOnPacket::read(&mut cursor(Vec::<u8>::new())).is_err());
    assert!(ServerboundUseItemOnPacket::read(&mut cursor(vec![2])).is_err());
    assert!(
        ServerboundUseItemOnPacket::read(&mut cursor(vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 6])).is_err()
    );
    assert!(ServerboundUseItemOnPacket::read(&mut cursor(vec![
        0, 0xff, 0xff, 0xfd, 0x00, 0x00, 0x02, 0x20, 0x40, 1, 0x3e, 0x80, 0x00, 0x00, 0x3f, 0x00,
        0x00, 0x00, 0x3f, 0x40, 0x00, 0x00, 1, 0, 0xad, 0x02, 0
    ]))
    .is_err());
}

#[test]
fn serverbound_accept_teleportation_packet_uses_vanilla_varint_id() {
    let packet = ServerboundAcceptTeleportationPacket { teleport_id: 300 };
    let mut payload = Vec::new();
    packet.write(&mut payload).unwrap();
    assert_eq!(payload, vec![0xac, 0x02]);
    assert_eq!(
        ServerboundAcceptTeleportationPacket::read(&mut cursor(payload)).unwrap(),
        packet
    );
}

#[test]
fn serverbound_player_input_packet_uses_vanilla_input_bitset() {
    let packet = ServerboundPlayerInputPacket {
        input: ServerboundPlayerInput {
            forward: true,
            backward: false,
            left: true,
            right: false,
            jump: true,
            shift: false,
            sprint: true,
        },
    };
    let mut payload = Vec::new();
    packet.write(&mut payload).unwrap();
    assert_eq!(payload, vec![0x55]);
    assert_eq!(
        ServerboundPlayerInputPacket::read(&mut cursor(vec![0xff])).unwrap(),
        ServerboundPlayerInputPacket {
            input: ServerboundPlayerInput {
                forward: true,
                backward: true,
                left: true,
                right: true,
                jump: true,
                shift: true,
                sprint: true,
            },
        }
    );
}

#[test]
fn serverbound_change_difficulty_packet_uses_vanilla_varint_wrapping() {
    let mut payload = Vec::new();
    ServerboundChangeDifficultyPacket {
        difficulty: GameDifficulty::Hard,
    }
    .write(&mut payload)
    .unwrap();
    assert_eq!(payload, vec![3]);
    assert_eq!(
        ServerboundChangeDifficultyPacket::read(&mut cursor(vec![0x10])).unwrap(),
        ServerboundChangeDifficultyPacket {
            difficulty: GameDifficulty::Peaceful,
        }
    );
    assert_eq!(
        ServerboundChangeDifficultyPacket::read(&mut cursor(vec![0x7f])).unwrap(),
        ServerboundChangeDifficultyPacket {
            difficulty: GameDifficulty::Hard,
        }
    );
}

#[test]
fn serverbound_client_command_packet_uses_vanilla_action_ordinals() {
    let cases = [
        (
            ServerboundClientCommandAction::PerformRespawn,
            0,
            "perform respawn",
        ),
        (
            ServerboundClientCommandAction::RequestStats,
            1,
            "request stats",
        ),
        (
            ServerboundClientCommandAction::RequestGameruleValues,
            2,
            "request gamerule values",
        ),
    ];

    for (action, ordinal, label) in cases {
        let mut payload = Vec::new();
        ServerboundClientCommandPacket { action }
            .write(&mut payload)
            .unwrap();
        assert_eq!(payload, vec![ordinal], "{label} ordinal");
        assert_eq!(
            ServerboundClientCommandPacket::read(&mut cursor(vec![ordinal]))
                .unwrap()
                .action,
            action,
            "{label} decode"
        );
    }
}

#[test]
fn serverbound_swing_packet_uses_vanilla_interaction_hand_ordinals() {
    let cases = [
        (ServerboundSwingHand::MainHand, 0, "main hand"),
        (ServerboundSwingHand::OffHand, 1, "off hand"),
    ];

    for (hand, ordinal, label) in cases {
        let mut payload = Vec::new();
        ServerboundSwingPacket { hand }.write(&mut payload).unwrap();
        assert_eq!(payload, vec![ordinal], "{label} ordinal");
        assert_eq!(
            ServerboundSwingPacket::read(&mut cursor(vec![ordinal]))
                .unwrap()
                .hand,
            hand,
            "{label} decode"
        );
    }
}

#[test]
fn serverbound_player_action_packet_uses_vanilla_field_order_and_direction_wrapping() {
    let cases = [
        (
            ServerboundPlayerAction::StartDestroyBlock,
            0,
            "start destroy",
        ),
        (
            ServerboundPlayerAction::AbortDestroyBlock,
            1,
            "abort destroy",
        ),
        (ServerboundPlayerAction::StopDestroyBlock, 2, "stop destroy"),
        (ServerboundPlayerAction::DropAllItems, 3, "drop all"),
        (ServerboundPlayerAction::DropItem, 4, "drop one"),
        (ServerboundPlayerAction::ReleaseUseItem, 5, "release use"),
        (
            ServerboundPlayerAction::SwapItemWithOffhand,
            6,
            "swap offhand",
        ),
        (ServerboundPlayerAction::Stab, 7, "stab"),
    ];

    for (action, ordinal, label) in cases {
        let mut payload = Vec::new();
        ServerboundPlayerActionPacket {
            action,
            x: 0,
            y: 0,
            z: 0,
            direction: Direction3d::Down,
            sequence: 0,
        }
        .write(&mut payload)
        .unwrap();
        assert_eq!(payload[0], ordinal, "{label} ordinal");
        assert_eq!(
            ServerboundPlayerActionPacket::read(&mut cursor(payload))
                .unwrap()
                .action,
            action,
            "{label} decode"
        );
    }

    let packet = ServerboundPlayerActionPacket {
        action: ServerboundPlayerAction::StopDestroyBlock,
        x: -12,
        y: 64,
        z: 34,
        direction: Direction3d::West,
        sequence: 300,
    };
    let mut payload = Vec::new();
    packet.write(&mut payload).unwrap();
    assert_eq!(
        payload,
        vec![2, 0xff, 0xff, 0xfd, 0x00, 0x00, 0x02, 0x20, 0x40, 4, 0xac, 0x02]
    );
    assert_eq!(
        ServerboundPlayerActionPacket::read(&mut cursor(payload)).unwrap(),
        packet
    );

    let wrapped_direction =
        ServerboundPlayerActionPacket::read(&mut cursor(vec![2, 0, 0, 0, 0, 0, 0, 0, 0, 255, 0]))
            .unwrap();
    assert_eq!(wrapped_direction.direction, Direction3d::South);
}

#[test]
fn serverbound_player_command_packet_uses_vanilla_field_order() {
    let cases = [
        (
            ServerboundPlayerCommandAction::StopSleeping,
            0,
            "stop sleeping",
        ),
        (
            ServerboundPlayerCommandAction::StartSprinting,
            1,
            "start sprinting",
        ),
        (
            ServerboundPlayerCommandAction::StopSprinting,
            2,
            "stop sprinting",
        ),
        (
            ServerboundPlayerCommandAction::StartRidingJump,
            3,
            "start riding jump",
        ),
        (
            ServerboundPlayerCommandAction::StopRidingJump,
            4,
            "stop riding jump",
        ),
        (
            ServerboundPlayerCommandAction::OpenInventory,
            5,
            "open inventory",
        ),
        (
            ServerboundPlayerCommandAction::StartFallFlying,
            6,
            "start fall flying",
        ),
    ];

    for (action, ordinal, label) in cases {
        let mut payload = Vec::new();
        ServerboundPlayerCommandPacket {
            entity_id: 37,
            action,
            data: 128,
        }
        .write(&mut payload)
        .unwrap();
        assert_eq!(payload[0], 37, "{label} entity id");
        assert_eq!(payload[1], ordinal, "{label} ordinal");
        assert_eq!(&payload[2..], &[0x80, 0x01], "{label} data");
        assert_eq!(
            ServerboundPlayerCommandPacket::read(&mut cursor(payload)).unwrap(),
            ServerboundPlayerCommandPacket {
                entity_id: 37,
                action,
                data: 128,
            },
            "{label} decode"
        );
    }
}

#[test]
fn serverbound_attack_packet_uses_java_entity_id_varint_only() {
    let packet = ServerboundAttackPacket { entity_id: 128 };

    let mut payload = Vec::new();
    packet.write(&mut payload).unwrap();
    assert_eq!(payload, vec![0x80, 0x01]);
    assert_eq!(
        ServerboundAttackPacket::read(&mut cursor(payload)).unwrap(),
        packet
    );
    assert!(ServerboundAttackPacket::read(&mut cursor(Vec::<u8>::new())).is_err());
    assert!(ServerboundAttackPacket::read(&mut cursor(vec![1, 0])).is_err());
}

#[test]
fn serverbound_interact_packet_uses_vanilla_flat_stream_codec_order() {
    const SERVERBOUND_INTERACT_PACKET_JAVA: &str = vibecraft_java_source!("/net/minecraft/network/protocol/game/ServerboundInteractPacket.java");
    const INTERACTION_HAND_JAVA: &str = vibecraft_java_source!("/net/minecraft/world/InteractionHand.java");
    for sentinel in [
        "public record ServerboundInteractPacket(int entityId, InteractionHand hand, Vec3 location, boolean usingSecondaryAction)",
        "ByteBufCodecs.VAR_INT",
        "ServerboundInteractPacket::entityId",
        "InteractionHand.STREAM_CODEC",
        "ServerboundInteractPacket::hand",
        "Vec3.LP_STREAM_CODEC",
        "ServerboundInteractPacket::location",
        "ByteBufCodecs.BOOL",
        "ServerboundInteractPacket::usingSecondaryAction",
        "return GamePacketTypes.SERVERBOUND_INTERACT;",
        "listener.handleInteract(this);",
    ] {
        assert!(
            SERVERBOUND_INTERACT_PACKET_JAVA.contains(sentinel),
            "missing ServerboundInteractPacket sentinel {sentinel}"
        );
    }
    for sentinel in [
        "MAIN_HAND(0)",
        "OFF_HAND(1)",
        "ByIdMap.OutOfBoundsStrategy.ZERO",
        "ByteBufCodecs.idMapper(BY_ID, h -> h.id)",
    ] {
        assert!(
            INTERACTION_HAND_JAVA.contains(sentinel),
            "missing InteractionHand sentinel {sentinel}"
        );
    }

    let packet = ServerboundInteractPacket {
        entity_id: 128,
        hand: ServerboundInteractionHand::OffHand,
        location: Vec3 {
            x: 0.5,
            y: -0.25,
            z: 0.75,
        },
        using_secondary_action: true,
    };

    let mut payload = Vec::new();
    packet.write(&mut payload).unwrap();
    assert_eq!(
        payload,
        vec![0x80, 0x01, 1, 0xf9, 0xff, 0xdf, 0xfc, 0xbf, 0xfe, 1]
    );

    let decoded = ServerboundInteractPacket::read(&mut cursor(payload)).unwrap();
    assert_eq!(decoded.entity_id, packet.entity_id);
    assert_eq!(decoded.hand, packet.hand);
    assert!(decoded.using_secondary_action);
    assert!((decoded.location.x - packet.location.x).abs() < 0.0001);
    assert!((decoded.location.y - packet.location.y).abs() < 0.0001);
    assert!((decoded.location.z - packet.location.z).abs() < 0.0001);

    let invalid_hand = ServerboundInteractPacket::read(&mut cursor(vec![1, 7, 0, 0])).unwrap();
    assert_eq!(invalid_hand.hand, ServerboundInteractionHand::MainHand);
    assert_eq!(
        invalid_hand.location,
        Vec3 {
            x: 0.0,
            y: 0.0,
            z: 0.0
        }
    );
    assert!(!invalid_hand.using_secondary_action);
}

#[test]
fn serverbound_use_item_packet_uses_vanilla_field_order() {
    let packet = ServerboundUseItemPacket {
        hand: ServerboundSwingHand::OffHand,
        sequence: 300,
        y_rot: 45.0,
        x_rot: -10.5,
    };

    let mut payload = Vec::new();
    packet.write(&mut payload).unwrap();
    assert_eq!(
        payload,
        vec![1, 0xac, 0x02, 0x42, 0x34, 0x00, 0x00, 0xc1, 0x28, 0x00, 0x00]
    );
    assert_eq!(
        ServerboundUseItemPacket::read(&mut cursor(payload)).unwrap(),
        packet
    );
}

#[test]
fn serverbound_use_item_on_packet_uses_vanilla_block_hit_result_order() {
    let packet = ServerboundUseItemOnPacket {
        hand: ServerboundSwingHand::MainHand,
        block_hit: BlockHitResultPacketData {
            x: -12,
            y: 64,
            z: 34,
            direction: Direction3d::Up,
            click_x: 0.25,
            click_y: 0.5,
            click_z: 0.75,
            inside: true,
            world_border_hit: false,
        },
        sequence: 301,
    };

    let mut payload = Vec::new();
    packet.write(&mut payload).unwrap();
    assert_eq!(
        payload,
        vec![
            0, 0xff, 0xff, 0xfd, 0x00, 0x00, 0x02, 0x20, 0x40, 1, 0x3e, 0x80, 0x00, 0x00, 0x3f,
            0x00, 0x00, 0x00, 0x3f, 0x40, 0x00, 0x00, 1, 0, 0xad, 0x02
        ]
    );
    assert_eq!(
        ServerboundUseItemOnPacket::read(&mut cursor(payload)).unwrap(),
        packet
    );
}

#[test]
fn chunk_batch_finished_packet_round_trips_vanilla_varint_batch_size() {
    let mut bytes = Vec::new();
    ClientboundChunkBatchFinishedPacket { batch_size: 300 }
        .write(&mut bytes)
        .unwrap();

    assert_eq!(bytes, vec![0xac, 0x02]);
    assert_eq!(
        ClientboundChunkBatchFinishedPacket::read(&mut cursor(bytes)).unwrap(),
        ClientboundChunkBatchFinishedPacket { batch_size: 300 }
    );
}
