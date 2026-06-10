use super::tag_access::{
    CollectToTagVisitor, NbtFieldSelectorSpec, NbtFieldTree, NbtStreamTagVisitor, SkipAllVisitor,
    SkipFieldsVisitor, StreamEntryResult, StreamValueResult,
};
use super::tag_metadata::tag_type;
use super::Tag;

const STREAM_TAG_VISITOR_JAVA: &str =
    include_str!("../../../../decompiled-server-26.1.2/net/minecraft/nbt/StreamTagVisitor.java");
const SKIP_ALL_JAVA: &str =
    include_str!("../../../../decompiled-server-26.1.2/net/minecraft/nbt/visitors/SkipAll.java");
const SKIP_FIELDS_JAVA: &str =
    include_str!("../../../../decompiled-server-26.1.2/net/minecraft/nbt/visitors/SkipFields.java");
const COLLECT_TO_TAG_JAVA: &str = include_str!(
    "../../../../decompiled-server-26.1.2/net/minecraft/nbt/visitors/CollectToTag.java"
);
const FIELD_SELECTOR_JAVA: &str = include_str!(
    "../../../../decompiled-server-26.1.2/net/minecraft/nbt/visitors/FieldSelector.java"
);
const FIELD_TREE_JAVA: &str =
    include_str!("../../../../decompiled-server-26.1.2/net/minecraft/nbt/visitors/FieldTree.java");
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

#[test]
fn field_selector_and_tree_match_java_path_type_name_contract() {
    for sentinel in [
        "public record FieldSelector(List<String> path, TagType<?> type, String name)",
        "this(List.of(), type, name);",
        "this(List.of(parent), type, name);",
        "this(List.of(grandparent, parent), type, name);",
    ] {
        assert!(
            FIELD_SELECTOR_JAVA.contains(sentinel),
            "missing FieldSelector sentinel {sentinel}"
        );
    }
    for sentinel in [
        "public record FieldTree(int depth, Map<String, TagType<?>> selectedFields, Map<String, FieldTree> fieldsToRecurse)",
        "return new FieldTree(1);",
        "if (this.depth <= field.path().size())",
        "this.fieldsToRecurse.computeIfAbsent(field.path().get(this.depth - 1), s -> new FieldTree(this.depth + 1)).addEntry(field);",
        "this.selectedFields.put(field.name(), field.type());",
        "return type.equals(this.selectedFields().get(id));",
    ] {
        assert!(
            FIELD_TREE_JAVA.contains(sentinel),
            "missing FieldTree sentinel {sentinel}"
        );
    }

    let root_selector = NbtFieldSelectorSpec::root(tag_type(3), "DataVersion");
    assert!(root_selector.path.is_empty());
    assert_eq!(root_selector.name, "DataVersion");
    assert_eq!(root_selector.tag_type, tag_type(3));

    let child_selector = NbtFieldSelectorSpec::child("Data", tag_type(8), "LevelName");
    assert_eq!(child_selector.path, vec!["Data"]);
    let grandchild_selector =
        NbtFieldSelectorSpec::grandchild("Data", "Player", tag_type(1), "OnGround");
    assert_eq!(grandchild_selector.path, vec!["Data", "Player"]);

    let mut root = NbtFieldTree::create_root();
    assert_eq!(root.depth, 1);
    root.add_entry(&root_selector);
    root.add_entry(&child_selector);
    root.add_entry(&grandchild_selector);

    assert!(root.is_selected(&tag_type(3), "DataVersion"));
    assert!(!root.is_selected(&tag_type(8), "DataVersion"));
    let data_frame = root.fields_to_recurse.get("Data").unwrap();
    assert_eq!(data_frame.depth, 2);
    assert!(data_frame.is_selected(&tag_type(8), "LevelName"));
    let player_frame = data_frame.fields_to_recurse.get("Player").unwrap();
    assert_eq!(player_frame.depth, 3);
    assert!(player_frame.is_selected(&tag_type(1), "OnGround"));
}

#[test]
fn collect_to_tag_visitor_matches_java_builder_stack_contract() {
    for sentinel in [
        "private final Deque<CollectToTag.ContainerBuilder> containerStack = new ArrayDeque<>();",
        "this.containerStack.addLast(new CollectToTag.RootBuilder());",
        "public @Nullable Tag getResult()",
        "return this.containerStack.size() - 1;",
        "this.containerStack.getLast().acceptValue(instance);",
        "this.containerStack.getLast().acceptKey(id);",
        "this.enterContainerIfNeeded(type);",
        "if (type == ListTag.TYPE)",
        "this.containerStack.addLast(new CollectToTag.ListBuilder());",
        "else if (type == CompoundTag.TYPE)",
        "this.containerStack.addLast(new CollectToTag.CompoundBuilder());",
        "CollectToTag.ContainerBuilder container = this.containerStack.removeLast();",
        "this.containerStack.getLast().acceptValue(tag);",
        "this.list.addAndUnwrap(tag);",
    ] {
        assert!(
            COLLECT_TO_TAG_JAVA.contains(sentinel),
            "missing CollectToTag sentinel {sentinel}"
        );
    }

    let mut scalar = CollectToTagVisitor::new();
    assert_eq!(scalar.depth(), 0);
    assert_eq!(
        scalar.visit_root_entry(tag_type(3)),
        StreamValueResult::Continue
    );
    assert_eq!(scalar.visit_int(4790), StreamValueResult::Continue);
    assert_eq!(scalar.get_result(), Some(&Tag::Int(4790)));

    let mut collector = CollectToTagVisitor::new();
    assert_eq!(
        collector.visit_root_entry(tag_type(10)),
        StreamValueResult::Continue
    );
    assert_eq!(collector.depth(), 1);
    assert_eq!(
        collector.visit_named_entry(tag_type(3), "DataVersion"),
        StreamEntryResult::Enter
    );
    assert_eq!(collector.visit_int(4790), StreamValueResult::Continue);
    assert_eq!(
        collector.visit_named_entry(tag_type(9), "Pos"),
        StreamEntryResult::Enter
    );
    assert_eq!(collector.depth(), 2);
    assert_eq!(
        collector.visit_list(tag_type(6), 2),
        StreamValueResult::Continue
    );
    assert_eq!(
        collector.visit_element(tag_type(6), 0),
        StreamEntryResult::Enter
    );
    assert_eq!(collector.visit_double(1.25), StreamValueResult::Continue);
    assert_eq!(
        collector.visit_element(tag_type(6), 1),
        StreamEntryResult::Enter
    );
    assert_eq!(collector.visit_double(-2.5), StreamValueResult::Continue);
    assert_eq!(collector.visit_container_end(), StreamValueResult::Continue);
    assert_eq!(collector.depth(), 1);
    assert_eq!(
        collector.visit_named_entry(tag_type(10), "Inventory"),
        StreamEntryResult::Enter
    );
    assert_eq!(
        collector.visit_named_entry(tag_type(8), "Slot"),
        StreamEntryResult::Enter
    );
    assert_eq!(
        collector.visit_string("mainhand"),
        StreamValueResult::Continue
    );
    assert_eq!(collector.visit_container_end(), StreamValueResult::Continue);
    assert_eq!(collector.visit_container_end(), StreamValueResult::Continue);

    assert_eq!(
        collector.get_result(),
        Some(&Tag::Compound(vec![
            ("DataVersion".to_string(), Tag::Int(4790)),
            (
                "Pos".to_string(),
                Tag::List(vec![Tag::Double(1.25), Tag::Double(-2.5)])
            ),
            (
                "Inventory".to_string(),
                Tag::Compound(vec![(
                    "Slot".to_string(),
                    Tag::String("mainhand".to_string())
                )])
            ),
        ]))
    );

    let mut list_collector = CollectToTagVisitor::new();
    list_collector.visit_root_entry(tag_type(9));
    list_collector.visit_list(tag_type(10), 1);
    list_collector.visit_element(tag_type(10), 0);
    list_collector.visit_named_entry(tag_type(1), "");
    list_collector.visit_byte(7);
    list_collector.visit_container_end();
    list_collector.visit_container_end();
    assert_eq!(
        list_collector.get_result(),
        Some(&Tag::List(vec![Tag::Byte(7)]))
    );
}

#[test]
fn skip_fields_visitor_matches_java_selected_field_filtering() {
    for sentinel in [
        "public class SkipFields extends CollectToTag",
        "private final Deque<FieldTree> stack = new ArrayDeque<>();",
        "FieldTree rootFrame = FieldTree.createRoot();",
        "rootFrame.addEntry(wantedField);",
        "this.stack.push(rootFrame);",
        "if (currentFrame.isSelected(type, id))",
        "return StreamTagVisitor.EntryResult.SKIP;",
        "if (type == CompoundTag.TYPE)",
        "FieldTree newFrame = currentFrame.fieldsToRecurse().get(id);",
        "this.stack.push(newFrame);",
        "if (this.depth() == this.stack.element().depth())",
        "this.stack.pop();",
        "return super.visitContainerEnd();",
    ] {
        assert!(
            SKIP_FIELDS_JAVA.contains(sentinel),
            "missing SkipFields sentinel {sentinel}"
        );
    }

    let selectors = [
        NbtFieldSelectorSpec::root(tag_type(3), "DataVersion"),
        NbtFieldSelectorSpec::child("Data", tag_type(8), "LevelName"),
        NbtFieldSelectorSpec::child("Data", tag_type(3), "TypeSensitive"),
    ];
    let mut visitor = SkipFieldsVisitor::new(&selectors);
    assert_eq!(
        visitor.visit_root_entry(tag_type(10)),
        StreamValueResult::Continue
    );
    assert_eq!(
        visitor.visit_named_entry(tag_type(3), "DataVersion"),
        StreamEntryResult::Skip
    );
    assert_eq!(
        visitor.visit_named_entry(tag_type(8), "Name"),
        StreamEntryResult::Enter
    );
    assert_eq!(visitor.visit_string("kept"), StreamValueResult::Continue);
    assert_eq!(
        visitor.visit_named_entry(tag_type(10), "Data"),
        StreamEntryResult::Enter
    );
    assert_eq!(visitor.depth(), 2);
    assert_eq!(
        visitor.visit_named_entry(tag_type(8), "LevelName"),
        StreamEntryResult::Skip
    );
    assert_eq!(
        visitor.visit_named_entry(tag_type(8), "TypeSensitive"),
        StreamEntryResult::Enter
    );
    assert_eq!(
        visitor.visit_string("wrong type is kept"),
        StreamValueResult::Continue
    );
    assert_eq!(
        visitor.visit_named_entry(tag_type(1), "Flag"),
        StreamEntryResult::Enter
    );
    assert_eq!(visitor.visit_byte(1), StreamValueResult::Continue);
    assert_eq!(visitor.visit_container_end(), StreamValueResult::Continue);
    assert_eq!(visitor.visit_container_end(), StreamValueResult::Continue);

    assert_eq!(
        visitor.get_result(),
        Some(&Tag::Compound(vec![
            ("Name".to_string(), Tag::String("kept".to_string())),
            (
                "Data".to_string(),
                Tag::Compound(vec![
                    (
                        "TypeSensitive".to_string(),
                        Tag::String("wrong type is kept".to_string())
                    ),
                    ("Flag".to_string(), Tag::Byte(1)),
                ])
            )
        ]))
    );
}
