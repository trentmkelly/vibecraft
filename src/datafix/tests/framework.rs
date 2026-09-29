//! Tests of the framework itself (templates, decode, walk, update semantics).

use crate::datafix::chain::{data_fixer, COMPLETE_THROUGH};
use crate::datafix::dynamic::ensure_namespaced;
use crate::datafix::fixer::DataFixError;
use crate::datafix::references as r;
use crate::datafix::schema::VersionKey;

use super::parse;

#[test]
fn schema_lookup_matches_java_get_schema() {
    let fixer = data_fixer();
    assert_eq!(fixer.schema_at(0).key, VersionKey::new(99));
    assert_eq!(fixer.schema_at(100).key, VersionKey::new(100));
    // Between two schemas the earlier one applies.
    assert_eq!(fixer.schema_at(104).key, VersionKey::new(102));
    // Sub versions are not visible from a plain data version.
    assert_eq!(fixer.schema_at(808).key, VersionKey::new(808));
}

#[test]
fn fix_names_follow_registration_order() {
    let names: Vec<_> = data_fixer()
        .fix_names()
        .into_iter()
        .map(|(_, name)| name)
        .collect();
    assert_eq!(names[0], "EntityEquipmentToArmorAndHandFix");
    let cooked = names
        .iter()
        .position(|n| *n == "cooked_fished item renamer")
        .unwrap();
    let vbo = names
        .iter()
        .position(|n| *n == "OptionsForceVBOFix")
        .unwrap();
    assert!(cooked < vbo);
}

#[test]
fn update_to_an_unported_version_is_refused() {
    let tag = parse("{}");
    let error = data_fixer()
        .update(r::PLAYER, &tag, 1000, COMPLETE_THROUGH + 1)
        .unwrap_err();
    assert_eq!(
        error,
        DataFixError::IncompleteChain {
            complete_through: COMPLETE_THROUGH,
            requested: COMPLETE_THROUGH + 1,
        }
    );
}

#[test]
fn same_or_older_target_is_a_no_op() {
    let tag = parse("{Inventory:[{id:5s,Count:1b}]}");
    let unchanged = data_fixer().update(r::PLAYER, &tag, 500, 500).unwrap();
    assert_eq!(unchanged, tag);
}

#[test]
fn ensure_namespaced_matches_identifier_rules() {
    assert_eq!(ensure_namespaced("stone"), "minecraft:stone");
    assert_eq!(ensure_namespaced("minecraft:stone"), "minecraft:stone");
    assert_eq!(ensure_namespaced("mod:thing"), "mod:thing");
    // Not a valid identifier: left alone.
    assert_eq!(ensure_namespaced("Not Valid"), "Not Valid");
}
