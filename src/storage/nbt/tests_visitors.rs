use super::tag_access::{
    NbtStreamTagVisitor, SkipAllVisitor, StreamEntryResult, StreamValueResult,
};
use super::tag_metadata::tag_type;

const STREAM_TAG_VISITOR_JAVA: &str =
    include_str!("../../../../decompiled-server-26.1.2/net/minecraft/nbt/StreamTagVisitor.java");
const SKIP_ALL_JAVA: &str =
    include_str!("../../../../decompiled-server-26.1.2/net/minecraft/nbt/visitors/SkipAll.java");
const NBT_VISITORS_PACKAGE_INFO_JAVA: &str = include_str!(
    "../../../../decompiled-server-26.1.2/net/minecraft/nbt/visitors/package-info.java"
);

#[test]
fn nbt_visitors_package_info_matches_java_metadata() {
    assert!(NBT_VISITORS_PACKAGE_INFO_JAVA.contains("@NullMarked"));
    assert!(NBT_VISITORS_PACKAGE_INFO_JAVA.contains("package net.minecraft.nbt.visitors;"));
    assert!(NBT_VISITORS_PACKAGE_INFO_JAVA.contains("import org.jspecify.annotations.NullMarked;"));
}

#[test]
fn stream_tag_visitor_interface_matches_java_contract() {
    for sentinel in [
        "StreamTagVisitor.ValueResult visitEnd();",
        "StreamTagVisitor.ValueResult visit(final String value);",
        "StreamTagVisitor.ValueResult visit(final byte[] value);",
        "StreamTagVisitor.ValueResult visitList(final TagType<?> elementType, final int size);",
        "StreamTagVisitor.EntryResult visitEntry(final TagType<?> type);",
        "StreamTagVisitor.EntryResult visitEntry(final TagType<?> type, final String id);",
        "StreamTagVisitor.EntryResult visitElement(final TagType<?> type, final int index);",
        "StreamTagVisitor.ValueResult visitContainerEnd();",
        "StreamTagVisitor.ValueResult visitRootEntry(final TagType<?> type);",
        "ENTER,",
        "SKIP,",
        "BREAK,",
        "HALT;",
        "CONTINUE,",
    ] {
        assert!(
            STREAM_TAG_VISITOR_JAVA.contains(sentinel),
            "missing StreamTagVisitor sentinel {sentinel}"
        );
    }

    let entry_results = [
        StreamEntryResult::Enter,
        StreamEntryResult::Skip,
        StreamEntryResult::Break,
        StreamEntryResult::Halt,
    ];
    let value_results = [
        StreamValueResult::Continue,
        StreamValueResult::Break,
        StreamValueResult::Halt,
    ];
    assert!(entry_results.contains(&StreamEntryResult::Enter));
    assert!(entry_results.contains(&StreamEntryResult::Skip));
    assert!(entry_results.contains(&StreamEntryResult::Break));
    assert!(entry_results.contains(&StreamEntryResult::Halt));
    assert!(value_results.contains(&StreamValueResult::Continue));
    assert!(value_results.contains(&StreamValueResult::Break));
    assert!(value_results.contains(&StreamValueResult::Halt));
}

#[test]
fn skip_all_visitor_matches_java_default_results() {
    for sentinel in [
        "SkipAll INSTANCE = new SkipAll() {};",
        "default StreamTagVisitor.ValueResult visitEnd()",
        "return StreamTagVisitor.ValueResult.CONTINUE;",
        "default StreamTagVisitor.EntryResult visitElement(final TagType<?> type, final int index)",
        "default StreamTagVisitor.EntryResult visitEntry(final TagType<?> type)",
        "default StreamTagVisitor.EntryResult visitEntry(final TagType<?> type, final String id)",
        "return StreamTagVisitor.EntryResult.SKIP;",
        "default StreamTagVisitor.ValueResult visitRootEntry(final TagType<?> type)",
    ] {
        assert!(
            SKIP_ALL_JAVA.contains(sentinel),
            "missing SkipAll sentinel {sentinel}"
        );
    }

    let mut visitor = SkipAllVisitor;
    assert_eq!(visitor.visit_end(), StreamValueResult::Continue);
    assert_eq!(visitor.visit_string("value"), StreamValueResult::Continue);
    assert_eq!(visitor.visit_byte(1), StreamValueResult::Continue);
    assert_eq!(visitor.visit_short(2), StreamValueResult::Continue);
    assert_eq!(visitor.visit_int(3), StreamValueResult::Continue);
    assert_eq!(visitor.visit_long(4), StreamValueResult::Continue);
    assert_eq!(visitor.visit_float(5.0), StreamValueResult::Continue);
    assert_eq!(visitor.visit_double(6.0), StreamValueResult::Continue);
    assert_eq!(
        visitor.visit_byte_array(&[1, 2]),
        StreamValueResult::Continue
    );
    assert_eq!(
        visitor.visit_int_array(&[3, 4]),
        StreamValueResult::Continue
    );
    assert_eq!(
        visitor.visit_long_array(&[5, 6]),
        StreamValueResult::Continue
    );
    assert_eq!(
        visitor.visit_list(tag_type(3), 2),
        StreamValueResult::Continue
    );
    assert_eq!(visitor.visit_entry(tag_type(10)), StreamEntryResult::Skip);
    assert_eq!(
        visitor.visit_named_entry(tag_type(3), "DataVersion"),
        StreamEntryResult::Skip
    );
    assert_eq!(
        visitor.visit_element(tag_type(8), 0),
        StreamEntryResult::Skip
    );
    assert_eq!(visitor.visit_container_end(), StreamValueResult::Continue);
    assert_eq!(
        visitor.visit_root_entry(tag_type(10)),
        StreamValueResult::Continue
    );
}
