//! Cases whose Java behaviour crashes or is random inside DFU, so they cannot be
//! pinned by the oracle fixtures; the expectations follow the Java source.

use crate::datafix::chain::data_fixer;
use crate::datafix::dynamic::{equal_unordered, get, get_i32_or};
use crate::datafix::references as r;
use crate::storage::nbt::Tag;

use super::parse;

#[test]
fn shulker_box_items_take_their_colour_from_the_block_entity_tag() {
    // ItemShulkerBoxColorFix discards `blockEntityRest.remove("Color")`; the
    // sibling BlockEntityShulkerBoxColorFix of the same version then removes it
    // from the nested block entity.
    let input = parse(
        r#"{Inventory:[{id:"minecraft:shulker_box",Count:1b,Damage:0s,tag:{BlockEntityTag:{id:"minecraft:shulker_box",Color:4,Items:[]}}}]}"#,
    );
    let expected = parse(
        r#"{Inventory:[{id:"minecraft:yellow_shulker_box",Count:1b,Damage:0s,tag:{BlockEntityTag:{id:"minecraft:shulker_box",Items:[]}}}]}"#,
    );
    let actual = data_fixer().update(r::PLAYER, &input, 808, 813).unwrap();
    assert!(equal_unordered(&actual, &expected), "{actual:?}");
}

#[test]
fn zombies_marked_as_villagers_without_a_profession_get_a_random_one() {
    let input = parse(r#"{Level:{Entities:[{id:"Zombie",IsVillager:1b}]}}"#);
    for _ in 0..20 {
        let actual = data_fixer().update(r::CHUNK, &input, 99, 502).unwrap();
        let Some(Tag::List(entities)) = get(get(&actual, "Level").unwrap(), "Entities") else {
            panic!("entities missing");
        };
        let zombie_type = get_i32_or(&entities[0], "ZombieType", -1);
        assert!((0..6).contains(&zombie_type), "{zombie_type}");
        assert!(get(&entities[0], "IsVillager").is_none());
    }
}
