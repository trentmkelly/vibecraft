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
