use super::numeric::NbtNumericValue;
use super::tag_metadata::{
    byte_array_size_in_bytes, int_array_size_in_bytes, list_size_in_bytes,
    long_array_size_in_bytes, string_size_in_bytes, tag_type, NbtTagTypeLookup,
};
use super::Tag;

#[allow(dead_code)]
impl Tag {
    pub fn tag_type(&self) -> NbtTagTypeLookup {
        tag_type(self.id() as i32)
    }

    pub fn copy_tag(&self) -> Self {
        self.clone()
    }

    pub fn size_in_bytes(&self) -> usize {
        match self {
            Tag::End => 8,
            Tag::Byte(_) => 9,
            Tag::Short(_) => 10,
            Tag::Int(_) | Tag::Float(_) => 12,
            Tag::Long(_) | Tag::Double(_) => 16,
            Tag::ByteArray(values) => byte_array_size_in_bytes(values.len()),
            Tag::String(value) => string_size_in_bytes(value),
            Tag::List(values) => {
                list_size_in_bytes(&values.iter().map(Tag::size_in_bytes).collect::<Vec<_>>())
            }
            Tag::Compound(values) => {
                48 + values
                    .iter()
                    .map(|(key, value)| {
                        28 + 2 * key.encode_utf16().count() + 36 + value.size_in_bytes()
                    })
                    .sum::<usize>()
            }
            Tag::IntArray(values) => int_array_size_in_bytes(values.len()),
            Tag::LongArray(values) => long_array_size_in_bytes(values.len()),
        }
    }

    pub fn as_string(&self) -> Option<&str> {
        match self {
            Tag::String(value) => Some(value),
            _ => None,
        }
    }

    pub fn as_number(&self) -> Option<NbtNumericValue> {
        self.numeric_value()
    }

    pub fn as_byte(&self) -> Option<i8> {
        self.numeric_value().map(|value| value.byte_value())
    }

    pub fn as_short(&self) -> Option<i16> {
        self.numeric_value().map(|value| value.short_value())
    }

    pub fn as_int(&self) -> Option<i32> {
        self.numeric_value().map(|value| value.int_value())
    }

    pub fn as_long(&self) -> Option<i64> {
        self.numeric_value().map(|value| value.long_value())
    }

    pub fn as_float(&self) -> Option<f32> {
        self.numeric_value().map(|value| value.float_value())
    }

    pub fn as_double(&self) -> Option<f64> {
        self.numeric_value().map(|value| value.double_value())
    }

    pub fn as_boolean(&self) -> Option<bool> {
        self.as_byte().map(|value| value != 0)
    }

    pub fn as_byte_array(&self) -> Option<&[i8]> {
        match self {
            Tag::ByteArray(values) => Some(values),
            _ => None,
        }
    }

    pub fn as_int_array(&self) -> Option<&[i32]> {
        match self {
            Tag::IntArray(values) => Some(values),
            _ => None,
        }
    }

    pub fn as_long_array(&self) -> Option<&[i64]> {
        match self {
            Tag::LongArray(values) => Some(values),
            _ => None,
        }
    }

    pub fn as_compound(&self) -> Option<&[(String, Tag)]> {
        match self {
            Tag::Compound(values) => Some(values),
            _ => None,
        }
    }

    pub fn as_list(&self) -> Option<&[Tag]> {
        match self {
            Tag::List(values) => Some(values),
            _ => None,
        }
    }
}
