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
