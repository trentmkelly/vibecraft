use super::*;
use crate::recipe_system::RecipeMap;

fn empty_recipes() -> RecipeMap {
    RecipeMap::create(Vec::new())
}

// -------- Stale state-ID style sweep: all_slots length checks --------

#[test]
fn every_menu_reports_correct_slot_count_for_full_resync() {
    let player = PlayerInventory::new();
    assert_eq!(
        CraftingMenu::new(empty_recipes()).all_slots(&player).len(),
        46
    );
    assert_eq!(
        AbstractFurnaceMenu::new(FurnaceKind::Furnace, FuelValues::vanilla())
            .all_slots(&player)
            .len(),
        39
    );
    assert_eq!(
        ChestMenu::new(3).all_slots(&player).len(),
        27 + PLAYER_SLOTS
    );
    assert_eq!(
        ChestMenu::new(6).all_slots(&player).len(),
        54 + PLAYER_SLOTS
    );
    assert_eq!(HopperMenu::new().all_slots(&player).len(), 41);
    assert_eq!(DispenserMenu::new().all_slots(&player).len(), 45);
    assert_eq!(ShulkerBoxMenu::new().all_slots(&player).len(), 63);
    assert_eq!(AnvilMenu::new().all_slots(&player).len(), 39);
    assert_eq!(SmithingMenu::new(RecipeMap::create(Vec::new())).all_slots(&player).len(), 40);
    assert_eq!(StonecutterMenu::new().all_slots(&player).len(), 38);
    assert_eq!(GrindstoneMenu::new().all_slots(&player).len(), 39);
    assert_eq!(EnchantmentMenu::new().all_slots(&player).len(), 38);
    assert_eq!(BrewingStandMenu::new().all_slots(&player).len(), 41);
    assert_eq!(CartographyTableMenu::new().all_slots(&player).len(), 39);
    assert_eq!(LoomMenu::new().all_slots(&player).len(), 40);
    assert_eq!(LecternMenu::new().all_slots().len(), 1);
    assert_eq!(BeaconMenu::new().all_slots(&player).len(), 37);
    assert_eq!(CrafterMenu::new().all_slots(&player).len(), 46);
    assert_eq!(NautilusInventoryMenu::new().all_slots(&player).len(), 38);
    assert_eq!(MerchantMenu::new().all_slots(&player).len(), 39);
}

/// `quickMoveStack` is the one menu method that is *not* shared in the
/// `AbstractContainerMenu`/inventory layer — every menu overrides it with its
/// own destination-priority routing. This test exercises that override for
/// every menu type by the universal invariant shared by all of them: an item
/// placed in a container/input slot is shift-clicked into the player inventory
/// (the first branch of every `quickMoveStack`), emptying the source slot.
/// (`MerchantMenu`, `HorseInventoryMenu` and `NautilusInventoryMenu` have their
/// own dedicated routing tests; `LecternMenu` has no player inventory and is
/// asserted as a no-op separately below.)
#[test]
#[allow(clippy::cognitive_complexity)] // one assertion per menu type — flat by design
fn every_menu_quick_moves_a_container_slot_into_the_player_inventory() {
    macro_rules! assert_qm {
        ($menu:expr, $slot:expr, $item:expr) => {{
            let mut player = PlayerInventory::new();
            let mut menu = $menu;
            menu.set_slot($slot, ItemStack::new($item, 1), &mut player);
            assert_eq!(
                menu.get_slot($slot, &player).map(|s| s.item_id().to_string()),
                Some($item.to_string()),
                "set_slot rejected {} into slot {}",
                $item,
                $slot
            );
            let moved = menu.quick_move($slot, &mut player);
            assert!(
                !moved.is_empty(),
                "quick_move failed to move {} out of slot {}",
                $item,
                $slot
            );
            assert!(
                menu.get_slot($slot, &player).is_none_or(|s| s.is_empty()),
                "slot {} was not emptied after quick_move of {}",
                $slot,
                $item
            );
        }};
    }

    assert_qm!(CraftingMenu::new(empty_recipes()), 1, "minecraft:stone");
    assert_qm!(CrafterMenu::new(), 0, "minecraft:stone");
    // AbstractFurnaceMenu.quickMoveStack is recipe-aware (smeltable -> input,
    // fuel -> fuel slot), so it is exercised directly with the recipe map.
    {
        let recipes = empty_recipes();
        let mut player = PlayerInventory::new();
        let mut menu = AbstractFurnaceMenu::new(FurnaceKind::Furnace, FuelValues::vanilla());
        menu.set_slot(0, ItemStack::new("minecraft:raw_iron", 1), &mut player);
        let moved = menu.quick_move(0, &recipes, &mut player);
        assert!(!moved.is_empty(), "AbstractFurnaceMenu quick_move failed");
        assert!(menu.get_slot(0, &player).is_none_or(|s| s.is_empty()));
    }
    assert_qm!(ChestMenu::new(3), 0, "minecraft:stone");
    assert_qm!(HopperMenu::new(), 0, "minecraft:stone");
    assert_qm!(DispenserMenu::new(), 0, "minecraft:stone");
    assert_qm!(ShulkerBoxMenu::new(), 0, "minecraft:stone");
    assert_qm!(AnvilMenu::new(), 0, "minecraft:diamond_sword");
    assert_qm!(SmithingMenu::new(RecipeMap::create(Vec::new())), 1, "minecraft:diamond_chestplate");
    assert_qm!(StonecutterMenu::new(), 0, "minecraft:stone");
    assert_qm!(GrindstoneMenu::new(), 0, "minecraft:diamond_sword");
    assert_qm!(EnchantmentMenu::new(), 0, "minecraft:diamond_sword");
    assert_qm!(BrewingStandMenu::new(), 3, "minecraft:nether_wart");
    assert_qm!(CartographyTableMenu::new(), 0, "minecraft:filled_map");
    assert_qm!(LoomMenu::new(), 0, "minecraft:white_banner");
    assert_qm!(BeaconMenu::new(), 0, "minecraft:diamond");

    // LecternMenu holds only the book and exposes no player inventory, so its
    // quickMoveStack is a no-op (returns EMPTY) — matching Java's LecternMenu.
    let mut lectern = LecternMenu::new();
    lectern.set_slot(0, ItemStack::new("minecraft:written_book", 1));
    assert!(lectern.quick_move(0).is_empty());
}
