#![allow(dead_code)]

use std::collections::BTreeMap;

use super::compound_tag::{compound_shallow_copy, put_compound_entry, remove_compound_entry};
use super::numeric::NbtNumericValue;
use super::Tag;

// TODO(nbt-ops-dynamicops): Port `DynamicOps<Tag>` conversion to arbitrary
// output ops, `MapLike<Tag>`, and the DFU `RecordBuilder` inheritance once the
// repo has a local DataFixerUpper/DynamicOps abstraction. This module owns the
// concrete NBT-side semantics from NbtOps.java.

#[derive(Debug, Clone, PartialEq)]
pub enum NbtOpsResult<T> {
    Success(T),
    Error {
        message: String,
        partial: Option<Tag>,
    },
}

impl<T> NbtOpsResult<T> {
    pub fn success(value: T) -> Self {
        Self::Success(value)
    }

    pub fn error(message: impl Into<String>, partial: Option<Tag>) -> Self {
        Self::Error {
            message: message.into(),
            partial,
        }
    }

    pub fn unwrap_success(self) -> T {
        match self {
            Self::Success(value) => value,
            Self::Error { message, .. } => panic!("{message}"),
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct NbtOpsModel;

impl NbtOpsModel {
    pub const INSTANCE: Self = Self;

    pub fn empty(self) -> Tag {
        Tag::End
    }

    pub fn empty_list(self) -> Tag {
        Tag::List(Vec::new())
    }

    pub fn empty_map(self) -> Tag {
        Tag::Compound(Vec::new())
    }

    pub fn get_number_value(self, input: &Tag) -> NbtOpsResult<NbtNumericValue> {
        input
            .numeric_value()
            .map(NbtOpsResult::success)
            .unwrap_or_else(|| NbtOpsResult::error("Not a number", None))
    }

    pub fn create_numeric(self, value: f64) -> Tag {
        Tag::Double(value)
    }

    pub fn create_byte(self, value: i8) -> Tag {
        Tag::Byte(value)
    }

    pub fn create_short(self, value: i16) -> Tag {
        Tag::Short(value)
    }

    pub fn create_int(self, value: i32) -> Tag {
        Tag::Int(value)
    }

    pub fn create_long(self, value: i64) -> Tag {
        Tag::Long(value)
    }

    pub fn create_float(self, value: f32) -> Tag {
        Tag::Float(value)
    }

    pub fn create_double(self, value: f64) -> Tag {
        Tag::Double(value)
    }

    pub fn create_boolean(self, value: bool) -> Tag {
        Tag::Byte(i8::from(value))
    }

    pub fn get_string_value(self, input: &Tag) -> NbtOpsResult<&str> {
        input
            .as_string()
            .map(NbtOpsResult::success)
            .unwrap_or_else(|| NbtOpsResult::error("Not a string", None))
    }

    pub fn create_string(self, value: impl Into<String>) -> Tag {
        Tag::String(value.into())
    }

    pub fn merge_to_list(self, list: &Tag, value: Tag) -> NbtOpsResult<Tag> {
        create_collector(list)
            .map(|collector| NbtOpsResult::success(collector.accept(value).result()))
            .unwrap_or_else(|| {
                NbtOpsResult::error(
                    format!("mergeToList called with not a list: {}", list.to_snbt()),
                    Some(list.clone()),
                )
            })
    }

    pub fn merge_all_to_list(self, list: &Tag, values: &[Tag]) -> NbtOpsResult<Tag> {
        create_collector(list)
            .map(|collector| NbtOpsResult::success(collector.accept_all(values).result()))
            .unwrap_or_else(|| {
                NbtOpsResult::error(
                    format!("mergeToList called with not a list: {}", list.to_snbt()),
                    Some(list.clone()),
                )
            })
    }

    pub fn merge_to_map(self, map: &Tag, key: &Tag, value: Tag) -> NbtOpsResult<Tag> {
        if !matches!(map, Tag::Compound(_) | Tag::End) {
            return NbtOpsResult::error(
                format!("mergeToMap called with not a map: {}", map.to_snbt()),
                Some(map.clone()),
            );
        }
        let Some(key) = key.as_string() else {
            return NbtOpsResult::error(
                format!("key is not a string: {}", key.to_snbt()),
                Some(map.clone()),
            );
        };
        let mut output = shallow_compound_or_empty(map);
        put_compound_entry(&mut output, key, value);
        NbtOpsResult::success(Tag::Compound(output))
    }

    pub fn merge_map_entries(self, map: &Tag, values: &[(Tag, Tag)]) -> NbtOpsResult<Tag> {
        if !matches!(map, Tag::Compound(_) | Tag::End) {
            return NbtOpsResult::error(
                format!("mergeToMap called with not a map: {}", map.to_snbt()),
                Some(map.clone()),
            );
        }
        if values.is_empty() {
            return if matches!(map, Tag::End) {
                NbtOpsResult::success(self.empty_map())
            } else {
                NbtOpsResult::success(map.clone())
            };
        }

        let mut output = shallow_compound_or_empty(map);
        let mut missed = Vec::new();
        for (key, value) in values {
            if let Some(key) = key.as_string() {
                put_compound_entry(&mut output, key, value.clone());
            } else {
                missed.push(key.clone());
            }
        }
        if missed.is_empty() {
            NbtOpsResult::success(Tag::Compound(output))
        } else {
            NbtOpsResult::error(
                format!("some keys are not strings: {missed:?}"),
                Some(Tag::Compound(output)),
            )
        }
    }

    pub fn get_map_values(self, input: &Tag) -> NbtOpsResult<Vec<(Tag, Tag)>> {
        match input {
            Tag::Compound(entries) => NbtOpsResult::success(
                entries
                    .iter()
                    .map(|(key, value)| (Tag::String(key.clone()), value.clone()))
                    .collect(),
            ),
            _ => NbtOpsResult::error(format!("Not a map: {}", input.to_snbt()), None),
        }
    }

    pub fn get_map_entry<'a>(&self, input: &'a Tag, key: &Tag) -> NbtOpsResult<Option<&'a Tag>> {
        match (input, key.as_string()) {
            (Tag::Compound(entries), Some(key)) => NbtOpsResult::success(
                entries
                    .iter()
                    .find(|(name, _)| name == key)
                    .map(|(_, tag)| tag),
            ),
            (Tag::Compound(_), None) => NbtOpsResult::error(
                format!(
                    "Cannot get map entry with non-string key: {}",
                    key.to_snbt()
                ),
                None,
            ),
            _ => NbtOpsResult::error(format!("Not a map: {}", input.to_snbt()), None),
        }
    }

    pub fn create_map(self, values: &[(Tag, Tag)]) -> NbtOpsResult<Tag> {
        let mut tag = Vec::new();
        for (key, value) in values {
            let Some(key) = key.as_string() else {
                return NbtOpsResult::error(
                    format!("Cannot create map with non-string key: {}", key.to_snbt()),
                    None,
                );
            };
            put_compound_entry(&mut tag, key, value.clone());
        }
        NbtOpsResult::success(Tag::Compound(tag))
    }

    pub fn get_stream(self, input: &Tag) -> NbtOpsResult<Vec<Tag>> {
        match input {
            Tag::List(values) => NbtOpsResult::success(values.clone()),
            Tag::ByteArray(values) => {
                NbtOpsResult::success(values.iter().copied().map(Tag::Byte).collect())
            }
            Tag::IntArray(values) => {
                NbtOpsResult::success(values.iter().copied().map(Tag::Int).collect())
            }
            Tag::LongArray(values) => {
                NbtOpsResult::success(values.iter().copied().map(Tag::Long).collect())
            }
            _ => NbtOpsResult::error("Not a list", None),
        }
    }

    pub fn get_list_values(self, input: &Tag) -> NbtOpsResult<Vec<Tag>> {
        match input {
            Tag::List(values) => NbtOpsResult::success(values.clone()),
            Tag::ByteArray(values) => {
                NbtOpsResult::success(values.iter().copied().map(Tag::Byte).collect())
            }
            Tag::IntArray(values) => {
                NbtOpsResult::success(values.iter().copied().map(Tag::Int).collect())
            }
            Tag::LongArray(values) => {
                NbtOpsResult::success(values.iter().copied().map(Tag::Long).collect())
            }
            _ => NbtOpsResult::error(format!("Not a list: {}", input.to_snbt()), None),
        }
    }

    pub fn get_byte_buffer(self, input: &Tag) -> NbtOpsResult<Vec<i8>> {
        match input {
            Tag::ByteArray(values) => NbtOpsResult::success(values.clone()),
            _ => NbtOpsResult::error("Not a byte list", None),
        }
    }

    pub fn create_byte_list(self, input: &[i8]) -> Tag {
        Tag::ByteArray(input.to_vec())
    }

    pub fn get_int_stream(self, input: &Tag) -> NbtOpsResult<Vec<i32>> {
        match input {
            Tag::IntArray(values) => NbtOpsResult::success(values.clone()),
            _ => NbtOpsResult::error("Not an int list", None),
        }
    }

    pub fn create_int_list(self, input: &[i32]) -> Tag {
        Tag::IntArray(input.to_vec())
    }

    pub fn get_long_stream(self, input: &Tag) -> NbtOpsResult<Vec<i64>> {
        match input {
            Tag::LongArray(values) => NbtOpsResult::success(values.clone()),
            _ => NbtOpsResult::error("Not a long list", None),
        }
    }

    pub fn create_long_list(self, input: &[i64]) -> Tag {
        Tag::LongArray(input.to_vec())
    }

    pub fn create_list(self, input: &[Tag]) -> Tag {
        Tag::List(input.to_vec())
    }

    pub fn remove(self, input: &Tag, key: &str) -> Tag {
        match input {
            Tag::Compound(entries) => {
                let mut result = compound_shallow_copy(entries);
                remove_compound_entry(&mut result, key);
                Tag::Compound(result)
            }
            _ => input.clone(),
        }
    }

    pub fn map_builder(self) -> NbtRecordBuilderModel {
        NbtRecordBuilderModel::default()
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct NbtRecordBuilderModel {
    entries: Vec<(String, Tag)>,
}

impl NbtRecordBuilderModel {
    pub fn append(mut self, key: impl Into<String>, value: Tag) -> Self {
        put_compound_entry(&mut self.entries, key, value);
        self
    }

    pub fn build(self, prefix: Option<&Tag>) -> NbtOpsResult<Tag> {
        match prefix {
            None | Some(Tag::End) => NbtOpsResult::success(Tag::Compound(self.entries)),
            Some(Tag::Compound(prefix_entries)) => {
                let mut result = compound_shallow_copy(prefix_entries);
                for (key, value) in self.entries {
                    put_compound_entry(&mut result, key, value);
                }
                NbtOpsResult::success(Tag::Compound(result))
            }
            Some(prefix) => NbtOpsResult::error(
                format!("mergeToMap called with not a map: {}", prefix.to_snbt()),
                Some(prefix.clone()),
            ),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
enum ListCollectorModel {
    Generic(Vec<Tag>),
    Byte(Vec<i8>),
    Int(Vec<i32>),
    Long(Vec<i64>),
}

impl ListCollectorModel {
    fn accept(self, tag: Tag) -> Self {
        match (self, tag) {
            (Self::Byte(mut values), Tag::Byte(value)) => {
                values.push(value);
                Self::Byte(values)
            }
            (Self::Byte(values), tag) => {
                let mut generic = values.into_iter().map(Tag::Byte).collect::<Vec<_>>();
                generic.push(tag);
                Self::Generic(generic)
            }
            (Self::Int(mut values), Tag::Int(value)) => {
                values.push(value);
                Self::Int(values)
            }
            (Self::Int(values), tag) => {
                let mut generic = values.into_iter().map(Tag::Int).collect::<Vec<_>>();
                generic.push(tag);
                Self::Generic(generic)
            }
            (Self::Long(mut values), Tag::Long(value)) => {
                values.push(value);
                Self::Long(values)
            }
            (Self::Long(values), tag) => {
                let mut generic = values.into_iter().map(Tag::Long).collect::<Vec<_>>();
                generic.push(tag);
                Self::Generic(generic)
            }
            (Self::Generic(mut values), tag) => {
                values.push(tag);
                Self::Generic(values)
            }
        }
    }

    fn accept_all(mut self, tags: &[Tag]) -> Self {
        for tag in tags {
            self = self.accept(tag.clone());
        }
        self
    }

    fn result(self) -> Tag {
        match self {
            Self::Generic(values) => Tag::List(values),
            Self::Byte(values) => Tag::ByteArray(values),
            Self::Int(values) => Tag::IntArray(values),
            Self::Long(values) => Tag::LongArray(values),
        }
    }
}

fn create_collector(tag: &Tag) -> Option<ListCollectorModel> {
    match tag {
        Tag::End => Some(ListCollectorModel::Generic(Vec::new())),
        Tag::List(values) => Some(ListCollectorModel::Generic(values.clone())),
        Tag::ByteArray(values) => Some(ListCollectorModel::Byte(values.clone())),
        Tag::IntArray(values) => Some(ListCollectorModel::Int(values.clone())),
        Tag::LongArray(values) => Some(ListCollectorModel::Long(values.clone())),
        _ => None,
    }
}

fn shallow_compound_or_empty(tag: &Tag) -> Vec<(String, Tag)> {
    match tag {
        Tag::Compound(entries) => compound_shallow_copy(entries),
        _ => Vec::new(),
    }
}

pub fn to_tree_map(entries: &[(Tag, Tag)]) -> BTreeMap<String, Tag> {
    let mut map = BTreeMap::new();
    for (key, value) in entries {
        if let Some(key) = key.as_string() {
            map.insert(key.to_string(), value.clone());
        }
    }
    map
}

#[cfg(all(test, vibecraft_has_decompiled_sources))]
mod tests {
    use super::*;

    const NBT_OPS_JAVA: &str =
        include_str!("../../../../decompiled-server-26.1.2/net/minecraft/nbt/NbtOps.java");

    #[test]
    fn nbt_ops_scalars_maps_and_remove_match_java_contracts() {
        for sentinel in [
            "public class NbtOps implements DynamicOps<Tag>",
            "public static final NbtOps INSTANCE = new NbtOps();",
            "return EndTag.INSTANCE;",
            "return new ListTag();",
            "return new CompoundTag();",
            "return DoubleTag.valueOf(i.doubleValue());",
            "return ByteTag.valueOf(value);",
            "return StringTag.valueOf(value);",
            "mergeToMap called with not a map",
            "key is not a string:",
            "Cannot create map with non-string key:",
            "public Tag remove(final Tag input, final String key)",
        ] {
            assert!(
                NBT_OPS_JAVA.contains(sentinel),
                "missing NbtOps sentinel {sentinel}"
            );
        }

        let ops = NbtOpsModel::INSTANCE;
        assert_eq!(ops.empty(), Tag::End);
        assert_eq!(ops.empty_list(), Tag::List(Vec::new()));
        assert_eq!(ops.empty_map(), Tag::Compound(Vec::new()));
        assert_eq!(ops.create_numeric(1.25), Tag::Double(1.25));
        assert_eq!(ops.create_boolean(true), Tag::Byte(1));
        assert_eq!(
            ops.get_number_value(&Tag::Int(7)),
            NbtOpsResult::Success(NbtNumericValue::Int(7))
        );
        assert!(matches!(
            ops.get_number_value(&Tag::String("x".to_string())),
            NbtOpsResult::Error { .. }
        ));
        assert_eq!(
            ops.get_string_value(&Tag::String("x".to_string())),
            NbtOpsResult::Success("x")
        );

        let merged = ops
            .merge_to_map(&Tag::End, &Tag::String("name".to_string()), Tag::Int(3))
            .unwrap_success();
        assert_eq!(
            merged,
            Tag::Compound(vec![("name".to_string(), Tag::Int(3))])
        );
        assert!(matches!(
            ops.merge_to_map(&Tag::Int(1), &Tag::String("x".to_string()), Tag::Int(2)),
            NbtOpsResult::Error { .. }
        ));
        assert!(matches!(
            ops.merge_to_map(&Tag::End, &Tag::Int(1), Tag::Int(2)),
            NbtOpsResult::Error { .. }
        ));

        let created = ops
            .create_map(&[(Tag::String("a".to_string()), Tag::Byte(1))])
            .unwrap_success();
        assert_eq!(
            created,
            Tag::Compound(vec![("a".to_string(), Tag::Byte(1))])
        );
        assert_eq!(
            ops.remove(
                &Tag::Compound(vec![
                    ("a".to_string(), Tag::Int(1)),
                    ("b".to_string(), Tag::Int(2))
                ]),
                "a"
            ),
            Tag::Compound(vec![("b".to_string(), Tag::Int(2))])
        );
    }

    #[test]
    fn nbt_ops_list_collectors_preserve_primitive_arrays_until_mixed() {
        for sentinel in [
            "private static Optional<NbtOps.ListCollector> createCollector",
            "case ByteArrayTag array -> Optional.of(new NbtOps.ByteListCollector",
            "return new NbtOps.GenericListCollector(this.values).accept(tag);",
            "return new ByteArrayTag(this.values.toByteArray());",
            "return new IntArrayTag(this.values.toIntArray());",
            "return new LongArrayTag(this.values.toLongArray());",
        ] {
            assert!(
                NBT_OPS_JAVA.contains(sentinel),
                "missing NbtOps sentinel {sentinel}"
            );
        }

        let ops = NbtOpsModel::INSTANCE;
        assert_eq!(
            ops.merge_to_list(&Tag::ByteArray(vec![1]), Tag::Byte(2))
                .unwrap_success(),
            Tag::ByteArray(vec![1, 2])
        );
        assert_eq!(
            ops.merge_to_list(&Tag::ByteArray(vec![1]), Tag::String("x".to_string()))
                .unwrap_success(),
            Tag::List(vec![Tag::Byte(1), Tag::String("x".to_string())])
        );
        assert_eq!(
            ops.merge_all_to_list(&Tag::End, &[Tag::Int(1), Tag::String("x".to_string())])
                .unwrap_success(),
            Tag::List(vec![Tag::Int(1), Tag::String("x".to_string())])
        );
        assert!(matches!(
            ops.merge_to_list(&Tag::String("no".to_string()), Tag::Int(1)),
            NbtOpsResult::Error { .. }
        ));
    }

    #[test]
    fn nbt_ops_streams_buffers_and_record_builder_match_java_contracts() {
        for sentinel in [
            "public DataResult<Stream<Tag>> getStream",
            "public DataResult<ByteBuffer> getByteBuffer",
            "public Tag createByteList(final ByteBuffer input)",
            "public Tag createIntList(final IntStream input)",
            "public Tag createLongList(final LongStream input)",
            "public RecordBuilder<Tag> mapBuilder()",
            "private class NbtRecordBuilder extends AbstractStringBuilder<Tag, CompoundTag>",
            "protected DataResult<Tag> build(final CompoundTag builder, final Tag prefix)",
        ] {
            assert!(
                NBT_OPS_JAVA.contains(sentinel),
                "missing NbtOps sentinel {sentinel}"
            );
        }

        let ops = NbtOpsModel::INSTANCE;
        assert_eq!(
            ops.get_byte_buffer(&Tag::ByteArray(vec![1, -2])),
            NbtOpsResult::Success(vec![1, -2])
        );
        assert_eq!(ops.create_byte_list(&[1, -2]), Tag::ByteArray(vec![1, -2]));
        assert_eq!(
            ops.get_int_stream(&Tag::IntArray(vec![3, 4])),
            NbtOpsResult::Success(vec![3, 4])
        );
        assert_eq!(ops.create_long_list(&[5, 6]), Tag::LongArray(vec![5, 6]));
        assert_eq!(
            ops.get_list_values(&Tag::IntArray(vec![1, 2])),
            NbtOpsResult::Success(vec![Tag::Int(1), Tag::Int(2)])
        );

        let built = ops
            .map_builder()
            .append("b", Tag::Int(2))
            .build(Some(&Tag::Compound(vec![("a".to_string(), Tag::Int(1))])))
            .unwrap_success();
        assert_eq!(
            built,
            Tag::Compound(vec![
                ("a".to_string(), Tag::Int(1)),
                ("b".to_string(), Tag::Int(2))
            ])
        );
        assert!(matches!(
            ops.map_builder()
                .append("x", Tag::Int(1))
                .build(Some(&Tag::Int(0))),
            NbtOpsResult::Error { .. }
        ));
    }
}
