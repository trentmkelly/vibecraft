//! NbtOps's native arrays and DynamicOps's numeric-list fallback.
use super::{NbtNumericValue, NbtOpsModel, NbtOpsResult, Tag};

impl NbtOpsModel {
    pub fn get_byte_buffer(self, input: &Tag) -> NbtOpsResult<Vec<i8>> {
        numeric_stream(input, "bytes", NbtNumericValue::boxed_byte_value)
    }

    pub fn get_int_stream(self, input: &Tag) -> NbtOpsResult<Vec<i32>> {
        numeric_stream(input, "ints", NbtNumericValue::boxed_int_value)
    }

    pub fn get_long_stream(self, input: &Tag) -> NbtOpsResult<Vec<i64>> {
        numeric_stream(input, "longs", NbtNumericValue::boxed_long_value)
    }
}

/// Match getStream + Number narrowing without allocating intermediate Tag lists.
/// A nonnumeric element rejects the entire stream; Java supplies no partial data.
fn numeric_stream<T>(
    input: &Tag,
    kind: &str,
    convert: impl Fn(NbtNumericValue) -> T,
) -> NbtOpsResult<Vec<T>> {
    let values = match input {
        Tag::ByteArray(values) => values
            .iter()
            .map(|value| convert(NbtNumericValue::Byte(*value)))
            .collect(),
        Tag::IntArray(values) => values
            .iter()
            .map(|value| convert(NbtNumericValue::Int(*value)))
            .collect(),
        Tag::LongArray(values) => values
            .iter()
            .map(|value| convert(NbtNumericValue::Long(*value)))
            .collect(),
        Tag::List(values) => {
            let Some(values) = values
                .iter()
                .map(|value| value.numeric_value().map(&convert))
                .collect::<Option<Vec<_>>>()
            else {
                return NbtOpsResult::error(
                    format!("Some elements are not {kind}: {}", input.to_snbt()),
                    None,
                );
            };
            values
        }
        _ => return NbtOpsResult::error("Not a list", None),
    };
    NbtOpsResult::success(values)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_array_type_supports_every_numeric_stream() {
        let ops = NbtOpsModel::INSTANCE;
        for tag in [
            Tag::ByteArray(vec![-1, 2]),
            Tag::IntArray(vec![-1, 2]),
            Tag::LongArray(vec![-1, 2]),
        ] {
            assert_eq!(
                ops.get_byte_buffer(&tag),
                NbtOpsResult::Success(vec![-1, 2])
            );
            assert_eq!(ops.get_int_stream(&tag), NbtOpsResult::Success(vec![-1, 2]));
            assert_eq!(
                ops.get_long_stream(&tag),
                NbtOpsResult::Success(vec![-1, 2])
            );
        }
        assert_eq!(
            ops.get_byte_buffer(&Tag::IntArray(vec![256, 255, -129])),
            NbtOpsResult::Success(vec![0, -1, 127])
        );
        assert_eq!(
            ops.get_int_stream(&Tag::LongArray(vec![4_294_967_297])),
            NbtOpsResult::Success(vec![1])
        );
    }

    #[test]
    fn boxed_numbers_truncate_while_numeric_tag_accessors_keep_flooring() {
        let input = Tag::List(vec![
            Tag::Byte(3),
            Tag::Short(-4),
            Tag::Int(5),
            Tag::Long(6),
            Tag::Float(-1.9),
            Tag::Double(-2.9),
        ]);
        let ops = NbtOpsModel::INSTANCE;
        assert_eq!(
            ops.get_int_stream(&input),
            NbtOpsResult::Success(vec![3, -4, 5, 6, -1, -2])
        );
        assert_eq!(
            ops.get_long_stream(&input),
            NbtOpsResult::Success(vec![3, -4, 5, 6, -1, -2])
        );
        assert_eq!(
            ops.get_byte_buffer(&input),
            NbtOpsResult::Success(vec![3, -4, 5, 6, -1, -2])
        );
        assert_eq!(NbtNumericValue::Double(-2.9).int_value(), -3);
        assert_eq!(NbtNumericValue::Double(-2.9).long_value(), -3);
    }

    #[test]
    fn floating_special_values_use_java_saturating_narrowing() {
        let tag = Tag::List(vec![
            Tag::Double(f64::NAN),
            Tag::Double(f64::INFINITY),
            Tag::Double(f64::NEG_INFINITY),
        ]);
        let ops = NbtOpsModel::INSTANCE;
        assert_eq!(
            ops.get_int_stream(&tag),
            NbtOpsResult::Success(vec![0, i32::MAX, i32::MIN])
        );
        assert_eq!(
            ops.get_long_stream(&tag),
            NbtOpsResult::Success(vec![0, i64::MAX, i64::MIN])
        );
        assert_eq!(
            ops.get_byte_buffer(&tag),
            NbtOpsResult::Success(vec![0, -1, 0])
        );
    }

    #[test]
    fn empty_collections_succeed_and_bad_inputs_have_no_partial_result() {
        let ops = NbtOpsModel::INSTANCE;
        for tag in [
            Tag::List(vec![]),
            Tag::ByteArray(vec![]),
            Tag::IntArray(vec![]),
            Tag::LongArray(vec![]),
        ] {
            assert_eq!(ops.get_int_stream(&tag), NbtOpsResult::Success(vec![]));
        }
        assert_eq!(
            ops.get_int_stream(&Tag::Int(3)),
            NbtOpsResult::error("Not a list", None)
        );
        let bad = Tag::List(vec![Tag::Int(3), Tag::String("bad".to_owned())]);
        assert_eq!(
            ops.get_int_stream(&bad),
            NbtOpsResult::error(
                format!("Some elements are not ints: {}", bad.to_snbt()),
                None
            )
        );
        assert!(matches!(
            ops.get_byte_buffer(&bad),
            NbtOpsResult::Error { partial: None, .. }
        ));
        assert!(matches!(
            ops.get_long_stream(&bad),
            NbtOpsResult::Error { partial: None, .. }
        ));
    }
}
