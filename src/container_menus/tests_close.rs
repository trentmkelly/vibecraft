use super::*;

fn empty_recipes() -> crate::recipe_system::RecipeMap {
    crate::recipe_system::RecipeMap::create(Vec::new())
}

#[test]
#[allow(clippy::too_many_lines)]
fn close_while_carrying_returns_work_slots_and_cursor_to_inventory() {
    // AbstractContainerMenu.removed routes the carried cursor item back into the
    // inventory on a normal close, then each work-area menu `clearContainer`s its
    // work slots back. Exercise every work-area menu type via the unified `removed`.
    // Each closure loads the menu's input slots, then `removed(close)` must empty
    // them and return the items + the carried cursor to the player inventory.
    fn assert_close_returns(
        load: impl Fn(&mut PlayerInventory) -> Box<dyn FnOnce(&mut PlayerInventory, &mut ItemStack)>,
        loaded_items: &[&str],
    ) {
        let mut player = PlayerInventory::new();
        let close = load(&mut player);
        let mut carried = ItemStack::new("minecraft:diamond", 3);
        close(&mut player, &mut carried);
        // Carried cursor returned to the inventory (placed, not dropped).
        assert!(carried.is_empty());
        assert!(player.dropped().is_empty(), "close must not drop in-world");
        // Every loaded work item is now somewhere in the player inventory.
        for id in loaded_items.iter().chain(std::iter::once(&"minecraft:diamond")) {
            assert!(
                (0..36).any(|i| player.get(i).item_id() == *id),
                "expected {id} returned to inventory on close"
            );
        }
    }

    assert_close_returns(
        |p| {
            let mut m = CraftingMenu::new(empty_recipes());
            // Slot 0 is the (virtual) result; the 3x3 craft grid is slots 1..=9.
            m.set_slot(1, ItemStack::new("minecraft:oak_planks", 1), p);
            m.set_slot(5, ItemStack::new("minecraft:stick", 1), p);
            Box::new(move |p, c| m.removed(p, c, false))
        },
        &["minecraft:oak_planks", "minecraft:stick"],
    );
    assert_close_returns(
        |p| {
            let mut m = AnvilMenu::new();
            m.set_slot(0, ItemStack::new("minecraft:iron_sword", 1), p);
            m.set_slot(1, ItemStack::new("minecraft:iron_ingot", 2), p);
            Box::new(move |p, c| m.removed(p, c, false))
        },
        &["minecraft:iron_sword", "minecraft:iron_ingot"],
    );
    assert_close_returns(
        |p| {
            let mut m = SmithingMenu::new();
            m.set_slot(0, ItemStack::new("minecraft:netherite_upgrade_smithing_template", 1), p);
            m.set_slot(1, ItemStack::new("minecraft:diamond_chestplate", 1), p);
            m.set_slot(2, ItemStack::new("minecraft:netherite_ingot", 1), p);
            Box::new(move |p, c| m.removed(p, c, false))
        },
        &[
            "minecraft:netherite_upgrade_smithing_template",
            "minecraft:diamond_chestplate",
            "minecraft:netherite_ingot",
        ],
    );
    assert_close_returns(
        |p| {
            let mut m = GrindstoneMenu::new();
            m.set_slot(0, ItemStack::new("minecraft:diamond_pickaxe", 1), p);
            m.set_slot(1, ItemStack::new("minecraft:diamond_pickaxe", 1), p);
            Box::new(move |p, c| m.removed(p, c, false))
        },
        &["minecraft:diamond_pickaxe"],
    );
    assert_close_returns(
        |p| {
            let mut m = EnchantmentMenu::new();
            m.set_slot(0, ItemStack::new("minecraft:diamond_sword", 1), p);
            m.set_slot(1, ItemStack::new("minecraft:lapis_lazuli", 3), p);
            Box::new(move |p, c| m.removed(p, c, false))
        },
        &["minecraft:diamond_sword", "minecraft:lapis_lazuli"],
    );
    assert_close_returns(
        |p| {
            let mut m = LoomMenu::new();
            m.set_slot(0, ItemStack::new("minecraft:white_banner", 1), p);
            m.set_slot(1, ItemStack::new("minecraft:red_dye", 1), p);
            Box::new(move |p, c| m.removed(p, c, false))
        },
        &["minecraft:white_banner", "minecraft:red_dye"],
    );
    assert_close_returns(
        |p| {
            let mut m = CartographyTableMenu::new();
            m.set_slot(0, ItemStack::new("minecraft:filled_map", 1), p);
            m.set_slot(1, ItemStack::new("minecraft:paper", 1), p);
            Box::new(move |p, c| m.removed(p, c, false))
        },
        &["minecraft:filled_map", "minecraft:paper"],
    );
    assert_close_returns(
        |p| {
            let mut m = StonecutterMenu::new();
            m.set_slot(0, ItemStack::new("minecraft:stone", 4), p);
            Box::new(move |p, c| m.removed(p, c, false))
        },
        &["minecraft:stone"],
    );
    assert_close_returns(
        |p| {
            let mut m = MerchantMenu::new();
            m.set_slot(0, ItemStack::new("minecraft:emerald", 5), p);
            m.set_slot(1, ItemStack::new("minecraft:book", 1), p);
            Box::new(move |p, c| m.removed(p, c, false))
        },
        &["minecraft:emerald", "minecraft:book"],
    );
}

#[test]
fn disconnect_while_open_drops_work_slots_and_cursor() {
    // On disconnect, `dropOrPlaceInInventory` drops in-world rather than placing back.
    // Verify every work-area menu drops its work slots + the carried cursor.
    fn assert_disconnect_drops(
        build: impl FnOnce(&mut PlayerInventory) -> Box<dyn FnOnce(&mut PlayerInventory, &mut ItemStack)>,
        dropped_ids: &[&str],
    ) {
        let mut player = PlayerInventory::new();
        let close = build(&mut player);
        let mut carried = ItemStack::new("minecraft:diamond", 1);
        close(&mut player, &mut carried);
        assert!(carried.is_empty());
        for id in dropped_ids.iter().chain(std::iter::once(&"minecraft:diamond")) {
            assert!(
                player.dropped().iter().any(|s| s.item_id() == *id),
                "expected {id} dropped in-world on disconnect"
            );
        }
    }

    assert_disconnect_drops(
        |p| {
            let mut m = AnvilMenu::new();
            m.set_slot(0, ItemStack::new("minecraft:iron_sword", 1), p);
            m.set_slot(1, ItemStack::new("minecraft:iron_ingot", 2), p);
            Box::new(move |p, c| m.removed(p, c, true))
        },
        &["minecraft:iron_sword", "minecraft:iron_ingot"],
    );
    assert_disconnect_drops(
        |p| {
            let mut m = MerchantMenu::new();
            m.set_slot(0, ItemStack::new("minecraft:emerald", 5), p);
            m.set_slot(1, ItemStack::new("minecraft:book", 1), p);
            Box::new(move |p, c| m.removed(p, c, true))
        },
        &["minecraft:emerald", "minecraft:book"],
    );

    // BeaconMenu always drops its payment in-world even on a NORMAL close, while the
    // carried cursor follows the normal place-back-on-close path.
    let mut player = PlayerInventory::new();
    let mut menu = BeaconMenu::new();
    menu.set_slot(0, ItemStack::new("minecraft:netherite_ingot", 1), &mut player);
    let mut carried = ItemStack::new("minecraft:diamond", 1);
    menu.removed(&mut player, &mut carried, false);
    assert!(carried.is_empty()); // carried placed back on close
    assert!(player
        .dropped()
        .iter()
        .any(|s| s.item_id() == "minecraft:netherite_ingot"));
    assert!(
        !player.dropped().iter().any(|s| s.item_id() == "minecraft:diamond"),
        "carried must be placed back, not dropped, on a normal close"
    );
    assert!((0..36).any(|i| player.get(i).item_id() == "minecraft:diamond"));
}
