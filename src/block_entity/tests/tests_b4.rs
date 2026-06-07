use super::*;

#[test]
fn lectern_block_entity_container_contract_and_pre_remove_match_java() {
    let mut lectern = LecternBlockEntity::new();
    let book = stack("minecraft:written_book", 3);

    assert_eq!(LecternBlockEntity::DATA_PAGE, 0);
    assert_eq!(LecternBlockEntity::SLOT_BOOK, 0);
    assert_eq!(LecternBlockEntity::NUM_SLOTS, 1);
    assert_eq!(lectern.max_stack_size(), 1);
    assert!(!lectern.can_place_item(0, &book));
    assert!(!lectern.still_valid(true, 4.0));
    assert_eq!(lectern.remove_item(1, 1), None);
    assert_eq!(lectern.remove_item(0, 0), None);

    lectern.set_book(Some(book.clone()), 3);
    assert_eq!(lectern.get_book(), Some(&book));
    assert!(lectern.still_valid(true, 64.0));
    assert!(!lectern.still_valid(true, 64.01));
    assert!(!lectern.still_valid(false, 1.0));

    let drop = lectern
        .pre_remove_side_effects(BlockPos { x: 10, y: 70, z: -4 }, Direction::East, true)
        .expect("lectern with book drops its book before removal");
    assert_eq!(drop.item, book);
    assert_eq!((drop.x, drop.y, drop.z), (10.75, 71.0, -3.5));
    assert_eq!(
        lectern.pre_remove_side_effects(BlockPos { x: 10, y: 70, z: -4 }, Direction::East, false),
        None
    );

    assert_eq!(
        lectern.remove_item(0, 2),
        Some(stack("minecraft:written_book", 2))
    );
    assert_eq!(lectern.get_book(), Some(&stack("minecraft:written_book", 1)));
    assert!(lectern.has_book());

    assert_eq!(
        lectern.remove_item(0, 2),
        Some(stack("minecraft:written_book", 1))
    );
    assert!(!lectern.has_book());
    assert_eq!(lectern.page, 0);
    assert_eq!(lectern.page_count, 0);
}

#[test]
fn randomizable_container_components_and_loot_table_tags_match_java() {
    let mut chest = ContainerBlockEntityModel::new(ContainerBlockEntityKind::Chest);
    chest.custom_name = Some("Supply Cache".to_string());
    chest.lock_key = Some("brass_key".to_string());
    chest.items[2] = Some(stack("minecraft:apple", 5));
    chest.loot_table = Some("minecraft:chests/simple_dungeon".to_string());

    assert_eq!(
        chest.save_additional(),
        Tag::Compound(vec![
            ("CustomName".to_string(), Tag::String("Supply Cache".to_string())),
            ("lock".to_string(), Tag::String("brass_key".to_string())),
            (
                "LootTable".to_string(),
                Tag::String("minecraft:chests/simple_dungeon".to_string()),
            ),
        ])
    );

    chest.loot_table_seed = 42;
    let components = chest.collect_implicit_components();
    assert_eq!(
        components,
        Tag::Compound(vec![
            (
                "minecraft:custom_name".to_string(),
                Tag::String("Supply Cache".to_string()),
            ),
            (
                "minecraft:lock".to_string(),
                Tag::String("brass_key".to_string()),
            ),
            (
                "minecraft:container".to_string(),
                Tag::List(vec![Tag::Compound(vec![
                    ("Slot".to_string(), Tag::Byte(2)),
                    ("id".to_string(), Tag::String("minecraft:apple".to_string())),
                    ("count".to_string(), Tag::Int(5)),
                ])]),
            ),
            (
                "minecraft:container_loot".to_string(),
                Tag::Compound(vec![
                    (
                        "loot_table".to_string(),
                        Tag::String("minecraft:chests/simple_dungeon".to_string()),
                    ),
                    ("seed".to_string(), Tag::Long(42)),
                ]),
            ),
        ])
    );

    let mut loaded = ContainerBlockEntityModel::new(ContainerBlockEntityKind::Chest);
    loaded.apply_implicit_components(&components);
    assert_eq!(loaded.custom_name.as_deref(), Some("Supply Cache"));
    assert_eq!(loaded.lock_key.as_deref(), Some("brass_key"));
    assert_eq!(loaded.items[2], Some(stack("minecraft:apple", 5)));
    assert_eq!(
        loaded.loot_table.as_deref(),
        Some("minecraft:chests/simple_dungeon")
    );
    assert_eq!(loaded.loot_table_seed, 42);

    assert_eq!(
        ContainerBlockEntityModel::remove_components_from_tag(&Tag::Compound(vec![
            ("CustomName".to_string(), Tag::String("Supply Cache".to_string())),
            ("lock".to_string(), Tag::String("brass_key".to_string())),
            ("Items".to_string(), container_items_tag(&loaded.items)),
            (
                "LootTable".to_string(),
                Tag::String("minecraft:chests/simple_dungeon".to_string()),
            ),
            ("LootTableSeed".to_string(), Tag::Long(42)),
            ("TransferCooldown".to_string(), Tag::Int(8)),
        ])),
        Tag::Compound(vec![("TransferCooldown".to_string(), Tag::Int(8))])
    );
}

#[test]
fn shelf_block_entity_components_owner_and_change_effects_match_java() {
    let mut shelf = ShelfBlockEntity::new();
    shelf.set_item_no_update(1, Some(stack("minecraft:book", 1)));
    shelf.align_items_to_bottom = true;

    let components = shelf.collect_implicit_components();
    assert_eq!(
        components,
        Tag::Compound(vec![(
            "minecraft:container".to_string(),
            Tag::List(vec![Tag::Compound(vec![
                ("Slot".to_string(), Tag::Byte(1)),
                ("id".to_string(), Tag::String("minecraft:book".to_string())),
                ("count".to_string(), Tag::Int(1)),
            ])]),
        )])
    );

    let mut from_components = ShelfBlockEntity::new();
    from_components.apply_implicit_components(&components);
    assert_eq!(from_components.get_item(1), Some(&stack("minecraft:book", 1)));
    assert!(!from_components.get_align_items_to_bottom());

    let saved = shelf.save_additional();
    assert_eq!(
        ShelfBlockEntity::remove_components_from_tag(&saved),
        Tag::Compound(vec![(
            ShelfBlockEntity::ALIGN_ITEMS_TO_BOTTOM_TAG.to_string(),
            Tag::Byte(1),
        )])
    );
    assert!(shelf.still_valid(true, 64.0));
    assert!(!shelf.still_valid(true, 64.01));
    assert!(!shelf.still_valid(false, 1.0));
    assert_eq!(shelf.default_set_changed_side_effects(false), None);
    assert_eq!(
        shelf.default_set_changed_side_effects(true),
        Some((Some("minecraft:block_activate"), 3))
    );
    assert_eq!(shelf.set_changed_side_effects(true, None), Some((None, 3)));
    assert_eq!(
        ShelfBlockEntity::item_owner_position(BlockPos { x: 4, y: 70, z: -2 }),
        (4.5, 70.5, -1.5)
    );
    assert_eq!(
        ShelfBlockEntity::visual_rotation_y_degrees(Direction::North),
        0.0
    );
    assert_eq!(
        ShelfBlockEntity::visual_rotation_y_degrees(Direction::East),
        90.0
    );
}

#[test]
fn shulker_box_block_entity_constants_events_and_color_match_java() {
    assert_eq!(ContainerBlockEntityModel::SHULKER_COLUMNS, 9);
    assert_eq!(ContainerBlockEntityModel::SHULKER_ROWS, 3);
    assert_eq!(ContainerBlockEntityModel::SHULKER_CONTAINER_SIZE, 27);
    assert_eq!(ContainerBlockEntityModel::SHULKER_EVENT_SET_OPEN_COUNT, 1);
    assert_eq!(ContainerBlockEntityModel::SHULKER_OPENING_TICK_LENGTH, 10);
    assert_eq!(ContainerBlockEntityModel::SHULKER_MAX_LID_HEIGHT, 0.5);
    assert_eq!(ContainerBlockEntityModel::SHULKER_MAX_LID_ROTATION, 270.0);
    assert_eq!(
        ContainerBlockEntityModel::shulker_color_from_block_id("minecraft:purple_shulker_box"),
        Some(DyeColor::Purple)
    );
    assert_eq!(
        ContainerBlockEntityModel::shulker_color_from_block_id("minecraft:shulker_box"),
        None
    );

    let mut shulker = ContainerBlockEntityModel::new(ContainerBlockEntityKind::ShulkerBox);
    assert_eq!(shulker.shulker_start_open_effects(false, true), None);
    assert_eq!(shulker.shulker_start_open_effects(true, false), None);
    assert_eq!(
        shulker.shulker_start_open_effects(false, false),
        Some(shulker_box::ShulkerBoxBlockEvent {
            action: 1,
            open_count: 1,
            first_open_or_last_close: true,
        })
    );
    assert_eq!(shulker.shulker_status, ShulkerBoxAnimationStatus::Opening);
    assert_eq!(
        shulker.shulker_start_open_effects(false, false),
        Some(shulker_box::ShulkerBoxBlockEvent {
            action: 1,
            open_count: 2,
            first_open_or_last_close: false,
        })
    );
    assert_eq!(
        shulker.shulker_stop_open_effects(false, false),
        Some(shulker_box::ShulkerBoxBlockEvent {
            action: 1,
            open_count: 1,
            first_open_or_last_close: false,
        })
    );
    assert_eq!(
        shulker.shulker_stop_open_effects(false, false),
        Some(shulker_box::ShulkerBoxBlockEvent {
            action: 1,
            open_count: 0,
            first_open_or_last_close: true,
        })
    );
    assert_eq!(shulker.shulker_status, ShulkerBoxAnimationStatus::Closing);

    assert!(shulker.trigger_shulker_event(1, 1));
    assert_eq!(shulker.viewer_count, 1);
    assert_eq!(shulker.shulker_status, ShulkerBoxAnimationStatus::Opening);
    assert!(!shulker.trigger_shulker_event(99, 0));
}

#[test]
fn shulker_box_block_entity_progress_collision_and_sided_access_match_java() {
    let mut shulker = ContainerBlockEntityModel::new(ContainerBlockEntityKind::ShulkerBox);
    assert!(shulker.trigger_shulker_event(1, 1));
    shulker.tick_lid();

    assert_eq!(shulker.shulker_progress_old, 0.0);
    assert!((shulker.lid_progress - 0.1).abs() < f32::EPSILON);
    assert!((shulker.shulker_progress(0.5) - 0.05).abs() < f32::EPSILON);
    assert!((shulker.shulker_bounding_lid_height(1.0) - 0.05).abs() < f32::EPSILON);

    assert_eq!(
        shulker.shulker_collision_move_delta(Direction::East, 0.0, 0.1),
        Some(shulker_box::ShulkerBoxCollisionMove {
            dx: 0.06000000074505806,
            dy: 0.0,
            dz: 0.0,
        })
    );

    for _ in 0..9 {
        shulker.tick_lid();
    }
    assert_eq!(shulker.shulker_status, ShulkerBoxAnimationStatus::Opened);
    assert_eq!(shulker.lid_progress, 1.0);

    assert_eq!(shulker.shulker_slots_for_face(), (0..27).collect::<Vec<_>>());
    assert!(shulker.shulker_can_take_through_face());
    assert!(!shulker.can_place_through_face(0, "minecraft:white_shulker_box", Direction::Up));
    assert!(shulker.can_place_through_face(0, "minecraft:diamond", Direction::Up));
    assert!(!shulker.shulker_pre_remove_has_side_effects());

    assert!(shulker.trigger_shulker_event(1, 0));
    for _ in 0..10 {
        shulker.tick_lid();
    }
    assert!(shulker.shulker_forces_solid_collision());
}
