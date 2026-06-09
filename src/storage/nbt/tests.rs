use super::accounter::{
    NbtAccounter, NbtAccounterError, DEFAULT_NBT_QUOTA, UNCOMPRESSED_NBT_QUOTA,
};
use super::{
    parse_snbt, read_gzip_named_tag, read_named_tag, read_named_tag_limited, write_gzip_named_tag,
    write_named_tag, NbtFieldSelector, Tag, DEFAULT_MAX_NBT_DEPTH,
};
use std::io::Cursor;

const NBT_ACCOUNTER_JAVA: &str =
    include_str!("../../../../decompiled-server-26.1.2/net/minecraft/nbt/NbtAccounter.java");
const NBT_EXCEPTION_JAVA: &str =
    include_str!("../../../../decompiled-server-26.1.2/net/minecraft/nbt/NbtException.java");
const NBT_ACCOUNTER_EXCEPTION_JAVA: &str = include_str!(
    "../../../../decompiled-server-26.1.2/net/minecraft/nbt/NbtAccounterException.java"
);
const NBT_FORMAT_EXCEPTION_JAVA: &str =
    include_str!("../../../../decompiled-server-26.1.2/net/minecraft/nbt/NbtFormatException.java");
const NBT_PACKAGE_INFO_JAVA: &str =
    include_str!("../../../../decompiled-server-26.1.2/net/minecraft/nbt/package-info.java");

#[test]
fn nbt_accounter_matches_java_quota_depth_and_error_contracts() {
    for sentinel in [
        "public static final int DEFAULT_NBT_QUOTA = 2097152;",
        "public static final int UNCOMPRESSED_NBT_QUOTA = 104857600;",
        "private static final int MAX_STACK_DEPTH = 512;",
        "return new NbtAccounter(quota, 512);",
        "return new NbtAccounter(2097152L, 512);",
        "return new NbtAccounter(104857600L, 512);",
        "return new NbtAccounter(Long.MAX_VALUE, 512);",
        "Tried to account NBT tag with negative size:",
        "Tried to read NBT tag that was too big; tried to allocate:",
        "Tried to read NBT tag with too high complexity, depth > ",
        "NBT-Accounter tried to pop stack-depth at top-level",
    ] {
        assert!(
            NBT_ACCOUNTER_JAVA.contains(sentinel),
            "missing NbtAccounter sentinel {sentinel}"
        );
    }

    assert_eq!(DEFAULT_NBT_QUOTA, 2_097_152);
    assert_eq!(UNCOMPRESSED_NBT_QUOTA, 104_857_600);
    assert_eq!(DEFAULT_MAX_NBT_DEPTH, 512);

    let mut accounter = NbtAccounter::create(10);
    accounter.account_bytes(4).unwrap();
    accounter.account_entries(3, 2).unwrap();
    assert_eq!(accounter.usage(), 10);
    assert_eq!(
        accounter.account_bytes(1).unwrap_err(),
        NbtAccounterError::QuotaExceeded {
            usage: 10,
            size: 1,
            quota: 10
        }
    );
    assert_eq!(
        NbtAccounter::default_quota()
            .account_bytes(-1)
            .unwrap_err()
            .to_string(),
        "Tried to account NBT tag with negative size: -1"
    );

    let mut depth_limited = NbtAccounter::new(100, 1);
    depth_limited.push_depth().unwrap();
    assert_eq!(depth_limited.depth(), 1);
    assert_eq!(
        depth_limited.push_depth().unwrap_err(),
        NbtAccounterError::DepthExceeded { max_depth: 1 }
    );
    depth_limited.pop_depth().unwrap();
    assert_eq!(
        depth_limited.pop_depth().unwrap_err(),
        NbtAccounterError::PopAtTopLevel
    );

    NbtAccounter::uncompressed_quota()
        .account_bytes(UNCOMPRESSED_NBT_QUOTA)
        .unwrap();
    NbtAccounter::unlimited_heap()
        .account_bytes(i64::MAX)
        .unwrap();
}

#[test]
fn nbt_exception_and_package_metadata_match_java_hierarchy() {
    assert!(NBT_EXCEPTION_JAVA.contains("public class NbtException extends RuntimeException"));
    assert!(NBT_ACCOUNTER_EXCEPTION_JAVA
        .contains("public class NbtAccounterException extends NbtException"));
    assert!(
        NBT_FORMAT_EXCEPTION_JAVA.contains("public class NbtFormatException extends NbtException")
    );
    assert!(NBT_PACKAGE_INFO_JAVA.contains("@NullMarked"));
    assert!(NBT_PACKAGE_INFO_JAVA.contains("package net.minecraft.nbt;"));
    assert!(NBT_PACKAGE_INFO_JAVA.contains("import org.jspecify.annotations.NullMarked;"));
}

#[test]
fn nbt_strings_use_java_modified_utf8_not_standard_utf8() {
    // Null char: Java modified UTF-8 encodes U+0000 as two bytes C0 80 (NOT the
    // single 0x00 standard UTF-8 uses), with an unsigned u16 length prefix.
    let mut buf = Vec::new();
    Tag::String("\u{0000}".to_string())
        .write_payload(&mut buf)
        .unwrap();
    assert_eq!(buf, vec![0x00, 0x02, 0xC0, 0x80]);

    // Supplementary char (emoji U+1F600): modified UTF-8 emits the UTF-16
    // surrogate pair D83D/DE00 as two 3-byte sequences = 6 bytes total (vs 4 in
    // standard UTF-8).
    let mut emoji = Vec::new();
    Tag::String("\u{1F600}".to_string())
        .write_payload(&mut emoji)
        .unwrap();
    assert_eq!(emoji, vec![0x00, 0x06, 0xED, 0xA0, 0xBD, 0xED, 0xB8, 0x80]);

    // BMP 3-byte char (snowman U+2603) matches standard UTF-8 (E2 98 83).
    let mut snowman = Vec::new();
    Tag::String("\u{2603}".to_string())
        .write_payload(&mut snowman)
        .unwrap();
    assert_eq!(snowman, vec![0x00, 0x03, 0xE2, 0x98, 0x83]);

    // Round-trip every case (including a mixed string) through read_payload.
    for s in ["", "abc", "\u{0000}", "\u{1F600}", "\u{2603}", "a\u{0000}b😀c"] {
        let mut bytes = Vec::new();
        Tag::String(s.to_string())
            .write_payload(&mut bytes)
            .unwrap();
        let decoded = Tag::read_payload(8, &mut bytes.as_slice()).unwrap();
        assert_eq!(decoded, Tag::String(s.to_string()));
    }
}

#[test]
fn exposes_all_vanilla_tag_ids() {
    assert_eq!(Tag::End.id(), 0);
    assert_eq!(Tag::Byte(0).id(), 1);
    assert_eq!(Tag::Short(0).id(), 2);
    assert_eq!(Tag::Int(0).id(), 3);
    assert_eq!(Tag::Long(0).id(), 4);
    assert_eq!(Tag::Float(0.0).id(), 5);
    assert_eq!(Tag::Double(0.0).id(), 6);
    assert_eq!(Tag::ByteArray(vec![]).id(), 7);
    assert_eq!(Tag::String(String::new()).id(), 8);
    assert_eq!(Tag::List(vec![]).id(), 9);
    assert_eq!(Tag::Compound(vec![]).id(), 10);
    assert_eq!(Tag::IntArray(vec![]).id(), 11);
    assert_eq!(Tag::LongArray(vec![]).id(), 12);
}

#[test]
fn round_trips_named_compound_with_nested_values() {
    let tag = Tag::Compound(vec![
        ("name".to_string(), Tag::String("RustCraft".to_string())),
        ("health".to_string(), Tag::Float(20.0)),
        (
            "pos".to_string(),
            Tag::List(vec![Tag::Double(1.0), Tag::Double(2.0)]),
        ),
        ("ints".to_string(), Tag::IntArray(vec![1, 2, 3])),
        ("longs".to_string(), Tag::LongArray(vec![4, 5, 6])),
    ]);

    let mut bytes = Vec::new();
    write_named_tag(&mut bytes, "root", &tag).unwrap();
    let (name, decoded) = read_named_tag(&mut Cursor::new(bytes)).unwrap();

    assert_eq!(name, "root");
    assert_eq!(decoded, tag);
}

#[test]
fn rejects_mixed_type_lists() {
    let mut bytes = Vec::new();
    let err = Tag::List(vec![Tag::Int(1), Tag::String("bad".to_string())])
        .write_payload(&mut bytes)
        .unwrap_err();
    assert_eq!(err.kind(), std::io::ErrorKind::InvalidInput);
}

#[test]
fn compressed_nbt_round_trips_named_tags() {
    let tag = Tag::Compound(vec![("DataVersion".to_string(), Tag::Int(4790))]);
    let mut bytes = Vec::new();
    write_gzip_named_tag(&mut bytes, "level", &tag).unwrap();

    assert_eq!(&bytes[..2], &[0x1f, 0x8b]);
    let (name, decoded) = read_gzip_named_tag(Cursor::new(bytes)).unwrap();
    assert_eq!(name, "level");
    assert_eq!(decoded, tag);
}

#[test]
fn snbt_printer_size_accounting_and_traversal_cover_nested_tags() {
    let tag = Tag::Compound(vec![
        (
            "name".to_string(),
            Tag::String("A \"quoted\" name".to_string()),
        ),
        ("bytes".to_string(), Tag::ByteArray(vec![1, 2])),
        (
            "nested".to_string(),
            Tag::List(vec![Tag::Int(1), Tag::Int(2)]),
        ),
    ]);

    // Byte-array elements use uppercase 'B', matching Java StringTagVisitor.
    assert_eq!(
        tag.to_snbt(),
        "{name:\"A \\\"quoted\\\" name\",bytes:[B;1B,2B],nested:[1,2]}"
    );
    assert_eq!(tag.payload_size(), 61);

    let mut ids = Vec::new();
    tag.visit_depth_first(&mut |visited| ids.push(visited.id()));
    assert_eq!(ids, vec![10, 8, 7, 9, 3, 3]);
}

#[test]
fn bounded_nbt_reads_reject_excessive_recursion() {
    let tag = Tag::Compound(vec![(
        "outer".to_string(),
        Tag::Compound(vec![("inner".to_string(), Tag::Int(1))]),
    )]);
    let mut bytes = Vec::new();
    write_named_tag(&mut bytes, "root", &tag).unwrap();

    let err = read_named_tag_limited(&mut Cursor::new(bytes), 1).unwrap_err();
    assert_eq!(err.kind(), std::io::ErrorKind::InvalidData);
    assert!(err.to_string().contains("depth limit"));
}

#[test]
fn nbt_size_tracker_and_field_selectors_cover_streaming_use_cases() {
    let tag = Tag::Compound(vec![
        (
            "Data".to_string(),
            Tag::Compound(vec![
                ("DataVersion".to_string(), Tag::Int(4790)),
                ("LevelName".to_string(), Tag::String("world".to_string())),
            ]),
        ),
        ("Other".to_string(), Tag::Byte(1)),
    ]);

    let tracked = tag.tracked_size();
    assert_eq!(tracked.nodes, 5);
    assert_eq!(tracked.max_depth, 2);
    assert!(tracked.payload_bytes >= tag.payload_size());

    let selectors = [
        NbtFieldSelector::dotted("Data.DataVersion"),
        NbtFieldSelector::dotted("Data.Missing"),
        NbtFieldSelector::dotted("Other"),
    ];
    let mut selected = Vec::new();
    tag.visit_selected_fields(&selectors, |selector, value| {
        selected.push((selector.path.join("."), value.clone()));
    });
    assert_eq!(
        selected,
        vec![
            ("Data.DataVersion".to_string(), Tag::Int(4790)),
            ("Other".to_string(), Tag::Byte(1)),
        ]
    );
}

#[test]
fn snbt_parser_round_trips_printer_shapes_and_reports_errors() {
    let tag = Tag::Compound(vec![
        (
            "name".to_string(),
            Tag::String("A \"quoted\" name".to_string()),
        ),
        ("bytes".to_string(), Tag::ByteArray(vec![1, 2])),
        ("ints".to_string(), Tag::IntArray(vec![3, 4])),
        ("longs".to_string(), Tag::LongArray(vec![5, 6])),
        (
            "nested".to_string(),
            Tag::List(vec![Tag::Int(1), Tag::Int(2)]),
        ),
        ("enabled".to_string(), Tag::Byte(1)),
    ]);

    let printed = tag.to_snbt();
    assert_eq!(parse_snbt(&printed).unwrap(), tag);
    assert_eq!(parse_snbt("{flag:true}").unwrap().to_snbt(), "{flag:1b}");
    assert_eq!(parse_snbt("12s").unwrap(), Tag::Short(12));
    assert_eq!(parse_snbt("3.5f").unwrap(), Tag::Float(3.5));
    assert!(parse_snbt("{broken")
        .unwrap_err()
        .to_string()
        .contains("expected"));
}

#[test]
fn text_component_visitor_matches_plain_java_layout_rules() {
    let tag = Tag::Compound(vec![
        ("z".to_string(), Tag::Int(3)),
        ("a key".to_string(), Tag::String("quoted".to_string())),
        (
            "list".to_string(),
            Tag::List(vec![
                Tag::Compound(vec![("inner".to_string(), Tag::Byte(1))]),
                Tag::Compound(vec![("inner".to_string(), Tag::Byte(2))]),
            ]),
        ),
    ]);

    assert_eq!(
        tag.to_text_component_plain("  ", true),
        "{\n  \"a key\": \"quoted\",\n  list: [\n    {\n      inner: 1b\n    },\n    {\n      inner: 2b\n    }\n  ],\n  z: 3\n}"
    );
    assert_eq!(
        tag.to_text_component("", true).to_json(),
        "{\"text\":\"{\\\"a key\\\": \\\"quoted\\\", list: [{inner: 1b}, {inner: 2b}], z: 3}\"}"
    );
}

#[test]
fn text_component_visitor_folds_deep_tags_and_long_arrays() {
    let long_array = Tag::ByteArray((0..130).map(|value| value as i8).collect());
    let rendered = long_array.to_text_component_plain("", true);
    assert!(rendered.starts_with("[B; 0b, 1b"));
    assert!(rendered.ends_with(",<...>]"));

    let mut deep = Tag::Int(1);
    for index in 0..65 {
        deep = Tag::Compound(vec![(format!("level{index}"), deep)]);
    }
    assert!(deep.to_text_component_plain("", true).contains("{<...>}"));
}
