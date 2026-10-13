//! Source-derived cases for `DataFixers.addFixers` versions 1800 through 1905.

use crate::datafix::chain::{data_fixer, COMPLETE_THROUGH};
use crate::datafix::dynamic::{equal_unordered, get, get_str_or};
use crate::datafix::references as r;
use crate::datafix::schema::VersionKey;
use crate::storage::nbt::Tag;

use super::parse;

#[test]
fn village_chain_registers_every_java_fix_in_order() {
    let fixes: Vec<_> = data_fixer()
        .fix_names()
        .into_iter()
        .filter(|(key, _)| key.version > 1624)
        .collect();
    assert_eq!(
        fixes,
        vec![
            (VersionKey::new(1800), "Added 1.14 mobs fix"),
            (VersionKey::new(1800), "Rename dye items"),
            (VersionKey::new(1801), "Added Illager Beast"),
            (VersionKey::new(1802), "Rename sign blocks & stone slabs"),
            (VersionKey::new(1802), "Rename sign item & stone slabs"),
            (VersionKey::new(1803), "Item Lore componentize"),
            (VersionKey::new(1904), "Added Cats"),
            (VersionKey::new(1904), "EntityCatSplitFix"),
            (VersionKey::new(1905), "ChunkStatusFix"),
        ]
    );
    assert_eq!(COMPLETE_THROUGH, 1905);
    for version in [1800, 1801, 1802, 1803, 1904, 1905] {
        assert_eq!(
            data_fixer().schema_at(version).key,
            VersionKey::new(version)
        );
    }
}

#[test]
fn village_chain_renames_items_inside_new_pillager_inventory() {
    // The 1800 pillager template is necessary: generic mob equipment does not
    // expose this Inventory list to item fixes.
    let input = parse(
        r#"{id:"minecraft:pillager",Inventory:[{id:"cactus_green",Count:2b},{id:"minecraft:sign",Count:1b}],Unknown:7}"#,
    );
    let actual = data_fixer().update(r::ENTITY, &input, 1800, 1905).unwrap();
    // Starting at 1800 skips the dye fix, while the schema still exposes items.
    let Some(Tag::List(items)) = get(&actual, "Inventory") else {
        panic!("missing inventory")
    };
    assert_eq!(get_str_or(&items[0], "id", ""), "minecraft:cactus_green");
    assert_eq!(get_str_or(&items[1], "id", ""), "minecraft:oak_sign");
    // At 1799 the old schema cannot decode an entity that does not yet exist;
    // Java returns the original value instead of applying later fixes.
    assert_eq!(
        data_fixer().update(r::ENTITY, &input, 1799, 1905).unwrap(),
        input
    );
    // Existing entities can traverse both rename versions in one migration.
    let player_input = parse(
        r#"{Inventory:[{id:"cactus_green",Count:2b},{id:"minecraft:sign",Count:1b}],Unknown:7}"#,
    );
    let player_expected = parse(
        r#"{Inventory:[{id:"minecraft:green_dye",Count:2b},{id:"minecraft:oak_sign",Count:1b}],Unknown:7}"#,
    );
    let player_actual = data_fixer()
        .update(r::PLAYER, &player_input, 1624, 1905)
        .unwrap();
    assert!(
        equal_unordered(&player_actual, &player_expected),
        "{player_actual:?}"
    );
}

#[test]
fn village_chain_converts_nested_cats_and_chunk_status_together() {
    let input = parse(
        r#"{Level:{Status:"postprocessed",Entities:[{id:"minecraft:ocelot",CatType:2b,OwnerUUID:5,Owner:"Alex",Custom:9}],Sections:[],Keep:"value"}}"#,
    );
    let expected = parse(
        r#"{Level:{Status:"fullchunk",Entities:[{id:"minecraft:cat",CatType:2,OwnerUUID:"",Owner:"Alex",Custom:9}],Sections:[],Keep:"value"}}"#,
    );
    let actual = data_fixer().update(r::CHUNK, &input, 1624, 1905).unwrap();
    assert!(equal_unordered(&actual, &expected), "{actual:?}");
}

#[test]
fn village_chain_sign_and_slab_block_names_follow_java_map() {
    for (old, new) in [
        ("stone_slab", "smooth_stone_slab"),
        ("sign", "oak_sign"),
        ("wall_sign", "oak_wall_sign"),
    ] {
        let input = parse(&format!(
            r#"{{Name:"minecraft:{old}",Properties:{{custom:"unchanged"}}}}"#
        ));
        let expected = parse(&format!(
            r#"{{Name:"minecraft:{new}",Properties:{{custom:"unchanged"}}}}"#
        ));
        let actual = data_fixer()
            .update(r::BLOCK_STATE, &input, 1624, 1905)
            .unwrap();
        assert!(equal_unordered(&actual, &expected), "{actual:?}");
    }
}
