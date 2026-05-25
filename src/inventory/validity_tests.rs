use super::*;

#[test]
fn still_valid_keeps_player_inventory_open_but_closes_invalid_block_menus() {
    let player_inventory = Menu::new(0);
    let far_context = MenuTickContext {
        player_eye_position: Vec3 {
            x: 100.0,
            y: 64.0,
            z: 100.0,
        },
        block_interaction_range: 4.5,
        block_at_menu_pos: None,
        block_entity_at_menu_pos: None,
    };
    assert_eq!(
        player_inventory.tick_validity(&far_context),
        MenuTickResult::StillValid
    );

    let pos = BlockPos { x: 1, y: 64, z: 1 };
    let crafting_menu = Menu::new(0).with_block_validity("minecraft:crafting_table", pos, 4.0);
    let nearby_valid = MenuTickContext {
        player_eye_position: Vec3 {
            x: 1.5,
            y: 64.5,
            z: 4.5,
        },
        block_interaction_range: 4.5,
        block_at_menu_pos: Some("minecraft:crafting_table"),
        block_entity_at_menu_pos: None,
    };
    assert_eq!(
        crafting_menu.tick_validity(&nearby_valid),
        MenuTickResult::StillValid
    );

    let wrong_block = MenuTickContext {
        block_at_menu_pos: Some("minecraft:air"),
        ..nearby_valid.clone()
    };
    assert_eq!(
        crafting_menu.tick_validity(&wrong_block),
        MenuTickResult::CloseMenu
    );

    let too_far = MenuTickContext {
        player_eye_position: Vec3 {
            x: 10.6,
            y: 64.5,
            z: 1.5,
        },
        ..nearby_valid
    };
    assert_eq!(
        crafting_menu.tick_validity(&too_far),
        MenuTickResult::CloseMenu
    );
}

#[test]
fn still_valid_block_entity_requires_same_entity_and_range() {
    let pos = BlockPos { x: 3, y: 65, z: 3 };
    let chest_menu = Menu::new(0).with_block_entity_validity(17, pos, 4.0);
    let valid = MenuTickContext {
        player_eye_position: Vec3 {
            x: 3.5,
            y: 65.5,
            z: 3.5,
        },
        block_interaction_range: 4.5,
        block_at_menu_pos: Some("minecraft:chest"),
        block_entity_at_menu_pos: Some(17),
    };
    assert!(chest_menu.still_valid(&valid));

    let replaced = MenuTickContext {
        block_entity_at_menu_pos: Some(18),
        ..valid
    };
    assert_eq!(
        chest_menu.tick_validity(&replaced),
        MenuTickResult::CloseMenu
    );
}

#[test]
fn block_interaction_range_uses_java_eye_to_aabb_distance() {
    let pos = BlockPos { x: 1, y: 64, z: 1 };

    assert!(is_within_block_interaction_range(
        Vec3 {
            x: -7.49,
            y: 64.5,
            z: 1.5,
        },
        pos,
        4.5,
        4.0,
    ));
    assert!(!is_within_block_interaction_range(
        Vec3 {
            x: -7.5,
            y: 64.5,
            z: 1.5,
        },
        pos,
        4.5,
        4.0,
    ));
    assert!(is_within_block_interaction_range(
        Vec3 {
            x: 1.5,
            y: 73.49,
            z: 1.5,
        },
        pos,
        4.5,
        4.0,
    ));
    assert!(!is_within_block_interaction_range(
        Vec3 {
            x: 1.5,
            y: 73.5,
            z: 1.5,
        },
        pos,
        4.5,
        4.0,
    ));
}
