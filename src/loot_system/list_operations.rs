//! Java `ListOperation` implementations used by loot functions that edit lists.

use serde_json::{Map, Value};

/// A vanilla list-edit mode (`ListOperation` in the loot function codecs).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LootListOperation {
    ReplaceAll,
    ReplaceSection { offset: usize, size: Option<usize> },
    Insert { offset: usize },
    Append,
}

/// Replacement values and the operation to apply to an existing component list.
#[derive(Debug, Clone, PartialEq)]
pub struct LootListEdit<T> {
    pub values: Vec<T>,
    pub operation: LootListOperation,
}

impl LootListOperation {
    /// Decodes `ListOperation.codec(max_size)`, including its `mode` dispatch field.
    pub fn decode(object: &Map<String, Value>, max_size: usize) -> Result<Self, String> {
        let mode = object
            .get("mode")
            .and_then(Value::as_str)
            .ok_or("list operation requires string field `mode`")?;
        let offset = || decode_non_negative(object, "offset", 0);
        match mode {
            "replace_all" => Ok(Self::ReplaceAll),
            "replace_section" => {
                let offset = offset()?;
                let size = object
                    .get("size")
                    .map(|value| decode_non_negative_value(value, "size"))
                    .transpose()?;
                if let Some(size) = size {
                    if size > max_size {
                        return Err(format!(
                            "Size value too large: {size}, max size is {max_size}"
                        ));
                    }
                }
                Ok(Self::ReplaceSection { offset, size })
            }
            "insert" => Ok(Self::Insert { offset: offset()? }),
            "append" => Ok(Self::Append),
            _ => Err(format!("Unknown list operation mode '{mode}'")),
        }
    }

    /// Applies the operation with the same overflow and invalid-offset behavior as
    /// Java `ListOperation.apply`: on failure the original list is returned intact.
    pub fn apply<T: Clone>(&self, original: &[T], replacement: &[T], max_size: usize) -> Vec<T> {
        match self {
            Self::ReplaceAll => replacement.to_vec(),
            Self::Append => {
                if original
                    .len()
                    .checked_add(replacement.len())
                    .is_none_or(|len| len > max_size)
                {
                    return original.to_vec();
                }
                original.iter().chain(replacement).cloned().collect()
            }
            Self::Insert { offset } => {
                let Some(new_len) = original.len().checked_add(replacement.len()) else {
                    return original.to_vec();
                };
                if *offset > original.len() || new_len > max_size {
                    return original.to_vec();
                }
                let mut result = Vec::with_capacity(new_len);
                result.extend_from_slice(&original[..*offset]);
                result.extend_from_slice(replacement);
                result.extend_from_slice(&original[*offset..]);
                result
            }
            Self::ReplaceSection { offset, size } => {
                if *offset > original.len() {
                    return original.to_vec();
                }
                let resume = offset.saturating_add(size.unwrap_or(replacement.len()));
                let mut result =
                    Vec::with_capacity(original.len().saturating_add(replacement.len()));
                result.extend_from_slice(&original[..*offset]);
                result.extend_from_slice(replacement);
                if resume < original.len() {
                    result.extend_from_slice(&original[resume..]);
                }
                if result.len() > max_size {
                    original.to_vec()
                } else {
                    result
                }
            }
        }
    }
}

fn decode_non_negative(
    object: &Map<String, Value>,
    key: &str,
    default: usize,
) -> Result<usize, String> {
    object
        .get(key)
        .map(|value| decode_non_negative_value(value, key))
        .transpose()
        .map(|value| value.unwrap_or(default))
}

fn decode_non_negative_value(value: &Value, key: &str) -> Result<usize, String> {
    let value = value
        .as_u64()
        .ok_or_else(|| format!("list operation `{key}` must be a non-negative integer"))?;
    if value > i32::MAX as u64 {
        return Err(format!("list operation `{key}` is too large"));
    }
    usize::try_from(value).map_err(|_| format!("list operation `{key}` is too large"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn applies_all_modes_and_preserves_original_on_java_failure_cases() {
        let original = [1, 2, 3, 4];
        let replacement = [8, 9];
        assert_eq!(
            LootListOperation::Append.apply(&original, &replacement, 6),
            [1, 2, 3, 4, 8, 9]
        );
        assert_eq!(
            LootListOperation::Append.apply(&original, &replacement, 5),
            original
        );
        assert_eq!(
            LootListOperation::Insert { offset: 2 }.apply(&original, &replacement, 6),
            [1, 2, 8, 9, 3, 4]
        );
        assert_eq!(
            LootListOperation::Insert { offset: 5 }.apply(&original, &replacement, 6),
            original
        );
        assert_eq!(
            LootListOperation::ReplaceAll.apply(&original, &replacement, 1),
            replacement
        );
        assert_eq!(
            LootListOperation::ReplaceSection {
                offset: 1,
                size: Some(2)
            }
            .apply(&original, &replacement, 5),
            [1, 8, 9, 4]
        );
        assert_eq!(
            LootListOperation::ReplaceSection {
                offset: 1,
                size: Some(2)
            }
            .apply(&original, &replacement, 3),
            original
        );
        assert_eq!(
            LootListOperation::ReplaceSection {
                offset: 5,
                size: None
            }
            .apply(&original, &replacement, 6),
            original
        );
    }

    #[test]
    fn codec_requires_known_mode_and_checks_only_section_size_against_limit() {
        let operation = serde_json::from_value::<Value>(serde_json::json!({
            "mode": "replace_section", "offset": 2, "size": 256
        }))
        .unwrap();
        assert!(LootListOperation::decode(operation.as_object().unwrap(), 256).is_ok());
        assert!(LootListOperation::decode(operation.as_object().unwrap(), 255).is_err());
        let append = serde_json::json!({"mode": "append", "offset": -1});
        assert!(LootListOperation::decode(append.as_object().unwrap(), 1).is_ok());
        let insert = serde_json::json!({"mode": "insert", "offset": -1});
        assert!(LootListOperation::decode(insert.as_object().unwrap(), 1).is_err());
        let oversized_offset = serde_json::json!({"mode": "insert", "offset": 2147483648_u64});
        assert!(LootListOperation::decode(oversized_offset.as_object().unwrap(), 1).is_err());
        let oversized_section =
            serde_json::json!({"mode": "replace_section", "size": 2147483648_u64});
        assert!(
            LootListOperation::decode(oversized_section.as_object().unwrap(), usize::MAX).is_err()
        );
    }
}
