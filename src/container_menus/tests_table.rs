use super::*;
use crate::item_properties::{ItemComponent, MapPostProcessing};

fn map_stack() -> ItemStack {
    ItemStack::new("minecraft:filled_map", 1)
}

#[test]
fn cartography_table_result_matches_java_setup_result_slot() {
    let mut player = PlayerInventory::new();

    // No additional item -> no result.
    let mut menu = CartographyTableMenu::new();
    menu.set_slot(0, map_stack(), &mut player);
    menu.update_result(0, false);
    assert!(menu.get_slot(2, &player).unwrap().is_empty());

    // Paper on an unlocked, sub-max-scale map -> zoom out (count 1, SCALE marker).
    menu.set_slot(1, ItemStack::new("minecraft:paper", 1), &mut player);
    menu.update_result(2, false);
    let r = menu.get_slot(2, &player).unwrap();
    assert_eq!(r.item_id(), "minecraft:filled_map");
    assert_eq!(r.count(), 1);
    assert_eq!(
        r.component("minecraft:map_post_processing"),
        Some(&ItemComponent::MapPostProcessing(MapPostProcessing::Scale))
    );

    // Paper but already at max scale (4) -> no result.
    menu.update_result(4, false);
    assert!(menu.get_slot(2, &player).unwrap().is_empty());

    // Paper but the map is locked -> no result.
    menu.update_result(2, true);
    assert!(menu.get_slot(2, &player).unwrap().is_empty());

    // Glass pane on an unlocked map -> lock (count 1, LOCK marker).
    let mut menu = CartographyTableMenu::new();
    menu.set_slot(0, map_stack(), &mut player);
    menu.set_slot(1, ItemStack::new("minecraft:glass_pane", 1), &mut player);
    menu.update_result(1, false);
    let r = menu.get_slot(2, &player).unwrap();
    assert_eq!(r.count(), 1);
    assert_eq!(
        r.component("minecraft:map_post_processing"),
        Some(&ItemComponent::MapPostProcessing(MapPostProcessing::Lock))
    );
    // Glass pane on a locked map -> no result.
    menu.update_result(1, true);
    assert!(menu.get_slot(2, &player).unwrap().is_empty());

    // Empty map -> clone (copyWithCount(2), no post-processing marker).
    let mut menu = CartographyTableMenu::new();
    menu.set_slot(0, map_stack(), &mut player);
    menu.set_slot(1, ItemStack::new("minecraft:map", 1), &mut player);
    menu.update_result(2, true); // locked still clones
    let r = menu.get_slot(2, &player).unwrap();
    assert_eq!(r.count(), 2);
    assert!(r.component("minecraft:map_post_processing").is_none());

    // An unrelated additional item -> no result.
    let mut menu = CartographyTableMenu::new();
    menu.set_slot(0, map_stack(), &mut player);
    menu.set_slot(1, ItemStack::new("minecraft:apple", 1), &mut player);
    menu.update_result(0, false);
    assert!(menu.get_slot(2, &player).unwrap().is_empty());
}

#[test]
fn map_post_processing_ids_match_java() {
    assert_eq!(MapPostProcessing::Lock.id(), 0);
    assert_eq!(MapPostProcessing::Scale.id(), 1);
}
