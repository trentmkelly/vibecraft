//! Tests of the supporting utilities ported alongside the fixes.

use crate::datafix::dynamic::{as_byte_buffer, compound, ensure_namespaced, parse_java_int};
use crate::datafix::fixes::block_state_data::{
    get_tag, group_state, map_group_count, upgrade_block_id, upgrade_block_name,
    upgrade_block_state_tag,
};
use crate::datafix::fixes::level_flat_generator_info_fix::fix_string;
use crate::datafix::legacy_component_data_fix_utils::{
    create_text_component_json, extract_translation_string, rewrite_from_lenient,
};
use crate::datafix::packed_bit_storage::PackedBitStorage;
use crate::datafix::uuid::JavaUuid;
use crate::storage::nbt::Tag;

#[test]
fn packed_bit_storage_round_trips_values_that_span_two_longs() {
    for bits in [4_u32, 5, 6, 9, 13] {
        let mut storage = PackedBitStorage::new(bits, 4096);
        let mask = (1_i64 << bits) - 1;
        for index in 0..4096 {
            storage.set(index, (index as i64 * 7 + 3) & mask);
        }
        for index in 0..4096 {
            assert_eq!(
                storage.get(index),
                (index as i64 * 7 + 3) & mask,
                "{bits} bits"
            );
        }
        assert_eq!(storage.raw().len(), (4096 * bits as usize).div_ceil(64));
    }
}

#[test]
fn packed_bit_storage_lays_values_out_least_significant_bit_first() {
    let mut storage = PackedBitStorage::new(5, 64);
    storage.set(0, 0b10101);
    storage.set(1, 0b00011);
    // Entry 12 starts at bit 60 and continues in the next long.
    storage.set(12, 0b11111);
    assert_eq!(storage.raw()[0] & 0x3FF, 0b00011_10101);
    assert_eq!((storage.raw()[0] as u64) >> 60, 0b1111);
    assert_eq!(storage.raw()[1] & 1, 1);
    assert_eq!(storage.get(12), 0b11111);
}

#[test]
fn java_uuid_parses_the_lenient_forms_java_accepts() {
    let uuid = JavaUuid::from_string("12345678-1234-5678-9abc-def012345678").unwrap();
    assert_eq!(uuid.most, 0x1234_5678_1234_5678);
    assert_eq!(uuid.least, 0x9abc_def0_1234_5678_u64 as i64);
    let short = JavaUuid::from_string("1-2-3-4-5").unwrap();
    assert_eq!(short.most, (1_i64 << 32) | (2 << 16) | 3);
    assert_eq!(short.least, (4_i64 << 48) | 5);
    assert!(JavaUuid::from_string("not-a-uuid").is_none());
    assert!(JavaUuid::from_string("1-2-3-4").is_none());
    assert!(JavaUuid::from_string("1-2-3-4-5-6").is_none());
}

#[test]
fn java_number_parsing_matches_parse_int() {
    assert_eq!(parse_java_int("57"), Some(57));
    assert_eq!(parse_java_int("-100"), Some(-100));
    assert_eq!(parse_java_int("+7"), Some(7));
    assert_eq!(parse_java_int("abc"), None);
    assert_eq!(parse_java_int(""), None);
    assert_eq!(parse_java_int("2147483648"), None);
}

#[test]
fn byte_buffers_accept_arrays_and_numeric_lists() {
    assert_eq!(
        as_byte_buffer(&Tag::ByteArray(vec![1, -2])),
        Some(vec![1, -2])
    );
    assert_eq!(
        as_byte_buffer(&Tag::List(vec![Tag::Int(1), Tag::Int(257)])),
        Some(vec![1, 1])
    );
    assert_eq!(as_byte_buffer(&Tag::IntArray(vec![3])), Some(vec![3]));
    assert_eq!(as_byte_buffer(&Tag::String("x".into())), None);
    assert_eq!(
        as_byte_buffer(&Tag::List(vec![Tag::String("x".into())])),
        None
    );
}

#[test]
fn identifiers_are_normalised_like_ensure_namespaced() {
    assert_eq!(ensure_namespaced("stone"), "minecraft:stone");
    assert_eq!(ensure_namespaced("a:b/c"), "a:b/c");
    assert_eq!(ensure_namespaced("bad id"), "bad id");
}

#[test]
fn block_state_data_maps_legacy_ids_and_states() {
    assert_eq!(upgrade_block_id(0), "minecraft:air");
    assert_eq!(upgrade_block_id(-1), "minecraft:air");
    assert_eq!(upgrade_block_id(1 << 30), "minecraft:air");
    // 16 = stone:0, 17 = stone:1 (granite).
    assert_eq!(upgrade_block_id(17), "minecraft:granite");
    assert_eq!(
        get_tag(17),
        compound(vec![("Name", Tag::String("minecraft:granite".into()))])
    );
    assert_eq!(
        upgrade_block_name("minecraft:not_a_block"),
        "minecraft:not_a_block"
    );
    let old = compound(vec![
        ("Name", Tag::String("minecraft:stone".into())),
        (
            "Properties",
            compound(vec![("variant", Tag::String("granite".into()))]),
        ),
    ]);
    assert_eq!(
        upgrade_block_state_tag(&old),
        compound(vec![("Name", Tag::String("minecraft:granite".into()))])
    );
    // Extra keys make the compound a different `Dynamic`, so nothing matches.
    let with_extra = compound(vec![
        ("Name", Tag::String("minecraft:stone".into())),
        ("Extra", Tag::Byte(1)),
    ]);
    assert_eq!(upgrade_block_state_tag(&with_extra), with_extra);
}

#[test]
fn block_state_data_identity_groups_are_shared_by_default_states() {
    assert!(map_group_count() > 1000);
    // Data values 1..15 of an unregistered block share the block default instance.
    let (group_zero, _) = crate::datafix::fixes::block_state_data::get_state(0);
    let (group_one, _) = crate::datafix::fixes::block_state_data::get_state(1);
    assert_eq!(group_zero, group_one);
    assert_eq!(group_state(group_zero).0, "minecraft:air");
}

#[test]
fn legacy_text_components_are_rewritten_like_java() {
    assert_eq!(create_text_component_json("a\"b"), r#"{"text":"a\"b"}"#);
    assert_eq!(rewrite_from_lenient(""), r#"{"text":""}"#);
    assert_eq!(rewrite_from_lenient("null"), r#"{"text":""}"#);
    assert_eq!(rewrite_from_lenient("Hello"), r#"{"text":"Hello"}"#);
    assert_eq!(rewrite_from_lenient(r#""quoted""#), r#"{"text":"quoted"}"#);
    assert_eq!(rewrite_from_lenient("{b:1,a:2}"), r#"{"a":2,"b":1}"#);
    assert_eq!(
        extract_translation_string(r#"{"translate":"key.a"}"#),
        Some("key.a".to_string())
    );
    assert_eq!(extract_translation_string(r#"{"text":"x"}"#), None);
    assert_eq!(extract_translation_string("not json {"), None);
}

#[test]
fn flat_generator_options_are_upgraded_like_java() {
    assert_eq!(
        fix_string("").as_deref(),
        Some("minecraft:bedrock,2*minecraft:dirt,minecraft:grass_block;1;village")
    );
    // Version 3 resolves `minecraft:<name>` through the legacy id table, where
    // `grass_block` is unknown (air). Version 0-2 layers use `<n>x<id>:<data>`, version 3 uses `<n>*<name>`.
    assert_eq!(
        fix_string("2;7,2x3,2;1;village").as_deref(),
        Some("minecraft:bedrock,2*minecraft:dirt,minecraft:grass_block;1;village")
    );
    assert_eq!(
        fix_string("3;minecraft:bedrock,2*minecraft:dirt,minecraft:grass_block;1").as_deref(),
        Some("minecraft:bedrock,2*minecraft:dirt,minecraft:air;1")
    );
    // Out-of-range versions fall back to the default preset.
    assert_eq!(
        fix_string("9;7,2x3,2;1").as_deref(),
        Some("minecraft:bedrock,2*minecraft:dirt,minecraft:grass_block;1;village")
    );
}
