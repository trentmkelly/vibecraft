use super::*;
use crate::storage::nbt::tag_access::{
    CollectToTagVisitor, NbtFieldSelectorSpec, SkipFieldsVisitor,
};
use crate::storage::nbt::tag_metadata::NbtTagTypeLookup;
use std::io::Cursor;

struct Probe {
    root: StreamValueResult,
    entry: StreamEntryResult,
    named: StreamEntryResult,
    list: StreamValueResult,
    element: StreamEntryResult,
    value: StreamValueResult,
    ends: usize,
    ints: Vec<i32>,
}
impl Default for Probe {
    fn default() -> Self {
        Self {
            root: StreamValueResult::Continue,
            entry: StreamEntryResult::Enter,
            named: StreamEntryResult::Enter,
            list: StreamValueResult::Continue,
            element: StreamEntryResult::Enter,
            value: StreamValueResult::Continue,
            ends: 0,
            ints: vec![],
        }
    }
}
macro_rules! values { ($($name:ident: $ty:ty),*) => { $(fn $name(&mut self, _: $ty) -> StreamValueResult { self.value })* }; }
impl NbtStreamTagVisitor for Probe {
    values!(visit_string: &str, visit_byte: i8, visit_short: i16, visit_long: i64, visit_float: f32,
        visit_double: f64, visit_byte_array: &[i8], visit_int_array: &[i32], visit_long_array: &[i64]);
    fn visit_int(&mut self, value: i32) -> StreamValueResult {
        self.ints.push(value);
        self.value
    }
    fn visit_end(&mut self) -> StreamValueResult {
        self.value
    }
    fn visit_root_entry(&mut self, _: NbtTagTypeLookup) -> StreamValueResult {
        self.root
    }
    fn visit_entry(&mut self, _: NbtTagTypeLookup) -> StreamEntryResult {
        self.entry
    }
    fn visit_named_entry(&mut self, _: NbtTagTypeLookup, _: &str) -> StreamEntryResult {
        self.named
    }
    fn visit_list(&mut self, _: NbtTagTypeLookup, _: usize) -> StreamValueResult {
        self.list
    }
    fn visit_element(&mut self, _: NbtTagTypeLookup, _: usize) -> StreamEntryResult {
        self.element
    }
    fn visit_container_end(&mut self) -> StreamValueResult {
        self.ends += 1;
        StreamValueResult::Continue
    }
}

fn bytes(tag: &Tag) -> Vec<u8> {
    let mut bytes = vec![];
    write_unnamed_tag(tag, &mut bytes).unwrap();
    bytes
}

#[test]
fn halt_leaves_cursor_at_java_root_type_name_and_list_boundaries() {
    let compound = bytes(&Tag::Compound(vec![("x".to_owned(), Tag::Int(4))]));
    for (mut probe, position) in [
        (
            Probe {
                root: StreamValueResult::Halt,
                ..Probe::default()
            },
            1,
        ),
        (
            Probe {
                entry: StreamEntryResult::Halt,
                ..Probe::default()
            },
            4,
        ),
        (
            Probe {
                named: StreamEntryResult::Halt,
                ..Probe::default()
            },
            7,
        ),
    ] {
        let mut reader = Cursor::new(&compound);
        parse(&mut reader, &mut probe).unwrap();
        assert_eq!(reader.position(), position);
        assert_eq!(probe.ends, 0);
    }
    let list = bytes(&Tag::List(vec![Tag::Int(4)]));
    for mut probe in [
        Probe {
            list: StreamValueResult::Halt,
            ..Probe::default()
        },
        Probe {
            element: StreamEntryResult::Halt,
            ..Probe::default()
        },
    ] {
        let mut reader = Cursor::new(&list);
        parse(&mut reader, &mut probe).unwrap();
        assert_eq!(reader.position(), 8);
    }
    let mut reader = Cursor::new([10, 0, 0, 3]); // no name or payload exists
    parse(
        &mut reader,
        &mut Probe {
            entry: StreamEntryResult::Halt,
            ..Probe::default()
        },
    )
    .unwrap();
}

#[test]
fn break_consumes_remaining_container_and_value_break_obeys_container_kind() {
    let compound = bytes(&Tag::Compound(vec![
        ("x".to_owned(), Tag::Int(4)),
        ("y".to_owned(), Tag::Int(5)),
    ]));
    for mut probe in [
        Probe {
            entry: StreamEntryResult::Break,
            ..Probe::default()
        },
        Probe {
            named: StreamEntryResult::Break,
            ..Probe::default()
        },
    ] {
        let mut reader = Cursor::new(&compound);
        parse(&mut reader, &mut probe).unwrap();
        assert_eq!(reader.position(), compound.len() as u64);
        assert_eq!(probe.ends, 1);
        assert!(probe.ints.is_empty());
    }
    let mut probe = Probe {
        value: StreamValueResult::Break,
        ..Probe::default()
    };
    parse(&mut compound.as_slice(), &mut probe).unwrap();
    assert_eq!(probe.ints, [4, 5]);
    let list = bytes(&Tag::List(vec![Tag::Int(4), Tag::Int(5)]));
    let mut probe = Probe {
        value: StreamValueResult::Break,
        ..Probe::default()
    };
    let mut reader = Cursor::new(&list);
    parse(&mut reader, &mut probe).unwrap();
    assert_eq!(probe.ints, [4]);
    assert_eq!(reader.position(), list.len() as u64);
}

#[test]
fn collector_round_trip_enters_nested_containers_once_and_unwraps_mixed_lists() {
    let tag = Tag::Compound(vec![(
        "nested".to_owned(),
        Tag::Compound(vec![
            (
                "mixed".to_owned(),
                Tag::List(vec![Tag::Int(2), Tag::String("three".to_owned())]),
            ),
            ("bytes".to_owned(), Tag::ByteArray(vec![1, -2])),
            ("ints".to_owned(), Tag::IntArray(vec![3])),
            ("longs".to_owned(), Tag::LongArray(vec![i64::MAX])),
        ]),
    )]);
    let mut collector = CollectToTagVisitor::new();
    parse(&mut bytes(&tag).as_slice(), &mut collector).unwrap();
    assert_eq!(collector.get_result(), Some(&tag));
    assert_eq!(collector.depth(), 0);
}

#[test]
fn streamed_duplicate_compound_keys_replace_the_previous_value_like_java_put() {
    let encoded = bytes(&Tag::Compound(vec![
        ("same".to_owned(), Tag::Int(1)),
        ("other".to_owned(), Tag::Long(2)),
        ("same".to_owned(), Tag::String("last".to_owned())),
    ]));
    let mut collector = CollectToTagVisitor::new();
    parse(&mut encoded.as_slice(), &mut collector).unwrap();
    assert_eq!(
        collector.get_result(),
        Some(&Tag::Compound(vec![
            ("same".to_owned(), Tag::String("last".to_owned())),
            ("other".to_owned(), Tag::Long(2)),
        ]))
    );
}

#[test]
fn skipped_strings_are_not_decoded_and_skipped_nesting_still_has_a_depth_limit() {
    let mut data = bytes(&Tag::Compound(vec![
        ("skip".to_owned(), Tag::String("BAD".to_owned())),
        ("keep".to_owned(), Tag::Int(7)),
    ]));
    let offset = data.windows(3).position(|part| part == b"BAD").unwrap();
    data[offset..offset + 3].fill(0xff);
    let mut visitor = SkipFieldsVisitor::new(&[NbtFieldSelectorSpec::root(tag_type(8), "skip")]);
    parse(&mut data.as_slice(), &mut visitor).unwrap();
    assert_eq!(
        visitor.get_result(),
        Some(&Tag::Compound(vec![("keep".to_owned(), Tag::Int(7))]))
    );
    let mut deep = vec![10, 0, 0];
    for _ in 0..DEFAULT_MAX_NBT_DEPTH {
        deep.extend([10, 0, 0]);
    }
    deep.resize(deep.len() + DEFAULT_MAX_NBT_DEPTH + 1, 0);
    let error = parse(
        &mut deep.as_slice(),
        &mut Probe {
            root: StreamValueResult::Break,
            ..Probe::default()
        },
    )
    .unwrap_err();
    assert!(error.to_string().contains("depth limit"));
}
