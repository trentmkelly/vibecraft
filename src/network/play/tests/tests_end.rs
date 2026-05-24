use super::super::*;
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
    let mut carried = ItemStack::empty();
    let mut state_id: i32 = 0;

    // Step 1: place oak log into crafting grid slot 1.
    let place_log = ServerboundContainerClickPacket {
        container_id: 0,
        state_id: 0,
        slot_num: 1,
        button_num: 0,
        container_input: ContainerInput::Pickup,
        changed_slots: BTreeMap::new(),
        carried_item: HashedStack::empty(),
    };
    carried = ItemStack::new("minecraft:oak_log", 1);
    let instructions =
        handle_container_click(&place_log, &mut state_id, &mut inventory_menu, &mut carried);

    // Slot 1 now holds the log; slot 0 shows the result (4 planks).
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
    // Server must send ContainerSetSlot for slot 0 (result) and slot 1 (log placed).
    assert!(
        instructions.iter().any(|i| matches!(
            i,
            PlayInstruction::ContainerSetSlot(p)
                if p.slot == 0 && p.item_stack.count == 4
        )),
        "expected ContainerSetSlot slot=0 count=4 planks"
    );
    assert!(
        instructions.iter().any(|i| matches!(
            i,
            PlayInstruction::ContainerSetSlot(p)
                if p.slot == 1 && p.item_stack.count == 1
        )),
        "expected ContainerSetSlot slot=1 count=1 log"
    );

    // Step 2: take result from slot 0.
    let take_result = ServerboundContainerClickPacket {
        container_id: 0,
        state_id: 1,
        slot_num: 0,
        button_num: 0,
        container_input: ContainerInput::Pickup,
        changed_slots: BTreeMap::new(),
        carried_item: HashedStack::empty(),
    };
    let instructions = handle_container_click(
        &take_result,
        &mut state_id,
        &mut inventory_menu,
        &mut carried,
    );

    assert_eq!(state_id, 2);
    assert_eq!(
        carried,
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
    assert!(
        instructions.iter().any(|i| matches!(
            i,
            PlayInstruction::ContainerSetSlot(p)
                if p.slot == 0 && p.item_stack.count == 0
        )),
        "expected ContainerSetSlot slot=0 count=0 (empty result)"
    );
    assert!(
        instructions.iter().any(|i| matches!(
            i,
            PlayInstruction::ContainerSetSlot(p)
                if p.slot == 1 && p.item_stack.count == 0
        )),
        "expected ContainerSetSlot slot=1 count=0 (log consumed)"
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

/// Parity test: result slot updates after each grid change; stale state ID is rejected with
/// full slot corrections; second craft of the same recipe does NOT emit another unlock.
#[test]
fn crafting_grid_result_updates_per_slot_change_stale_id_corrected_no_double_unlock() {
    let recipes = network_crafting_test_recipes();
    let mut inventory_menu = InventoryMenu::new(
        crate::player_inventory::PlayerInventory::new(),
        recipes.clone(),
    );
    let mut carried = ItemStack::new("minecraft:oak_log", 1);
    let mut state_id: i32 = 0;

    // Place one log into crafting slot 1 — result updates to 4 planks.
    let place1 = ServerboundContainerClickPacket {
        container_id: 0,
        state_id: 0,
        slot_num: 1,
        button_num: 0,
        container_input: ContainerInput::Pickup,
        changed_slots: BTreeMap::new(),
        carried_item: HashedStack::empty(),
    };
    handle_container_click(&place1, &mut state_id, &mut inventory_menu, &mut carried);
    assert_eq!(state_id, 1);
    assert_eq!(
        inventory_menu.get_slot(0),
        Some(ItemStack::new("minecraft:oak_planks", 4)),
        "result must update immediately after placing log in grid"
    );

    // Stale state-ID click is rejected; server returns full slot corrections.
    let stale_click = ServerboundContainerClickPacket {
        container_id: 0,
        state_id: 0, // outdated — correct value is 1
        slot_num: 2,
        button_num: 0,
        container_input: ContainerInput::Pickup,
        changed_slots: BTreeMap::new(),
        carried_item: HashedStack::empty(),
    };
    let corrections = handle_container_click(
        &stale_click,
        &mut state_id,
        &mut inventory_menu,
        &mut carried,
    );
    assert_eq!(state_id, 1, "stale click must not advance state ID");
    let set_slot_count = corrections
        .iter()
        .filter(|i| matches!(i, PlayInstruction::ContainerSetSlot(_)))
        .count();
    assert_eq!(
        set_slot_count,
        InventoryMenu::SLOT_COUNT,
        "stale click must send corrections for all 46 slots"
    );

    // Take result — log consumed (1→0), result cleared, recipe unlocked.
    let take1 = ServerboundContainerClickPacket {
        container_id: 0,
        state_id: 1,
        slot_num: 0,
        button_num: 0,
        container_input: ContainerInput::Pickup,
        changed_slots: BTreeMap::new(),
        carried_item: HashedStack::empty(),
    };
    let take1_instrs =
        handle_container_click(&take1, &mut state_id, &mut inventory_menu, &mut carried);
    assert_eq!(state_id, 2);
    assert_eq!(carried, ItemStack::new("minecraft:oak_planks", 4));
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

    // Place a second log and craft again — no unlock event this time.
    let mut carried2 = ItemStack::new("minecraft:oak_log", 1);
    let place2 = ServerboundContainerClickPacket {
        container_id: 0,
        state_id: 2,
        slot_num: 1,
        button_num: 0,
        container_input: ContainerInput::Pickup,
        changed_slots: BTreeMap::new(),
        carried_item: HashedStack::empty(),
    };
    handle_container_click(&place2, &mut state_id, &mut inventory_menu, &mut carried2);
    let take2 = ServerboundContainerClickPacket {
        container_id: 0,
        state_id: 3,
        slot_num: 0,
        button_num: 0,
        container_input: ContainerInput::Pickup,
        changed_slots: BTreeMap::new(),
        carried_item: HashedStack::empty(),
    };
    let take2_instrs =
        handle_container_click(&take2, &mut state_id, &mut inventory_menu, &mut carried2);
    assert!(
        !take2_instrs
            .iter()
            .any(|i| matches!(i, PlayInstruction::RecipesUnlocked(_))),
        "second craft of the same recipe must NOT emit another RecipesUnlocked"
    );
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
    assert!(ServerboundClientCommandPacket::read(&mut cursor(Vec::<u8>::new())).is_err());
    assert!(
        ServerboundChunkBatchReceivedPacket::read(&mut cursor(vec![0x7f, 0x7f, 0x7f])).is_err()
    );
    assert!(ServerboundLockDifficultyPacket::read(&mut cursor(Vec::<u8>::new())).is_err());
    assert!(ServerboundPaddleBoatPacket::read(&mut cursor(vec![1])).is_err());
    assert!(ServerboundPlayerInputPacket::read(&mut cursor(Vec::<u8>::new())).is_err());
    assert!(ServerboundClientTickEndPacket::read(&mut cursor(vec![1])).is_err());
    assert!(ServerboundPlayerLoadedPacket::read(&mut cursor(vec![2])).is_err());
    assert!(ServerboundChangeDifficultyPacket::read(&mut cursor(vec![0x10])).is_err());
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
