use super::numeric::NbtNumericValue;
use super::tag_metadata::{
    byte_array_size_in_bytes, int_array_size_in_bytes, list_size_in_bytes,
    long_array_size_in_bytes, string_size_in_bytes, tag_type, NbtTagTypeLookup,
};
use super::Tag;

#[allow(dead_code)]
pub trait NbtTagVisitor {
    fn visit_string(&mut self, value: &str);
    fn visit_byte(&mut self, value: i8);
    fn visit_short(&mut self, value: i16);
    fn visit_int(&mut self, value: i32);
    fn visit_long(&mut self, value: i64);
    fn visit_float(&mut self, value: f32);
    fn visit_double(&mut self, value: f64);
    fn visit_byte_array(&mut self, value: &[i8]);
    fn visit_int_array(&mut self, value: &[i32]);
    fn visit_long_array(&mut self, value: &[i64]);
    fn visit_list(&mut self, value: &[Tag]);
    fn visit_compound(&mut self, value: &[(String, Tag)]);
    fn visit_end(&mut self);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StreamEntryResult {
    Enter,
    Skip,
    Break,
    Halt,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StreamValueResult {
    Continue,
    Break,
    Halt,
}

#[allow(dead_code)]
pub trait NbtStreamTagVisitor {
    fn visit_end(&mut self) -> StreamValueResult;
    fn visit_string(&mut self, value: &str) -> StreamValueResult;
    fn visit_byte(&mut self, value: i8) -> StreamValueResult;
    fn visit_short(&mut self, value: i16) -> StreamValueResult;
    fn visit_int(&mut self, value: i32) -> StreamValueResult;
    fn visit_long(&mut self, value: i64) -> StreamValueResult;
    fn visit_float(&mut self, value: f32) -> StreamValueResult;
    fn visit_double(&mut self, value: f64) -> StreamValueResult;
    fn visit_byte_array(&mut self, value: &[i8]) -> StreamValueResult;
    fn visit_int_array(&mut self, value: &[i32]) -> StreamValueResult;
    fn visit_long_array(&mut self, value: &[i64]) -> StreamValueResult;
    fn visit_list(&mut self, element_type: NbtTagTypeLookup, size: usize) -> StreamValueResult;
    fn visit_entry(&mut self, tag_type: NbtTagTypeLookup) -> StreamEntryResult;
    fn visit_named_entry(&mut self, tag_type: NbtTagTypeLookup, id: &str) -> StreamEntryResult;
    fn visit_element(&mut self, tag_type: NbtTagTypeLookup, index: usize) -> StreamEntryResult;
    fn visit_container_end(&mut self) -> StreamValueResult;
    fn visit_root_entry(&mut self, tag_type: NbtTagTypeLookup) -> StreamValueResult;
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct SkipAllVisitor;

impl NbtStreamTagVisitor for SkipAllVisitor {
    fn visit_end(&mut self) -> StreamValueResult {
        StreamValueResult::Continue
    }

    fn visit_string(&mut self, _value: &str) -> StreamValueResult {
        StreamValueResult::Continue
    }

    fn visit_byte(&mut self, _value: i8) -> StreamValueResult {
        StreamValueResult::Continue
    }

    fn visit_short(&mut self, _value: i16) -> StreamValueResult {
        StreamValueResult::Continue
    }

    fn visit_int(&mut self, _value: i32) -> StreamValueResult {
        StreamValueResult::Continue
    }

    fn visit_long(&mut self, _value: i64) -> StreamValueResult {
        StreamValueResult::Continue
    }

    fn visit_float(&mut self, _value: f32) -> StreamValueResult {
        StreamValueResult::Continue
    }

    fn visit_double(&mut self, _value: f64) -> StreamValueResult {
        StreamValueResult::Continue
    }

    fn visit_byte_array(&mut self, _value: &[i8]) -> StreamValueResult {
        StreamValueResult::Continue
    }

    fn visit_int_array(&mut self, _value: &[i32]) -> StreamValueResult {
        StreamValueResult::Continue
    }

    fn visit_long_array(&mut self, _value: &[i64]) -> StreamValueResult {
        StreamValueResult::Continue
    }

    fn visit_list(&mut self, _element_type: NbtTagTypeLookup, _size: usize) -> StreamValueResult {
        StreamValueResult::Continue
    }

    fn visit_entry(&mut self, _tag_type: NbtTagTypeLookup) -> StreamEntryResult {
        StreamEntryResult::Skip
    }

    fn visit_named_entry(&mut self, _tag_type: NbtTagTypeLookup, _id: &str) -> StreamEntryResult {
        StreamEntryResult::Skip
    }

    fn visit_element(&mut self, _tag_type: NbtTagTypeLookup, _index: usize) -> StreamEntryResult {
        StreamEntryResult::Skip
    }

    fn visit_container_end(&mut self) -> StreamValueResult {
        StreamValueResult::Continue
    }

    fn visit_root_entry(&mut self, _tag_type: NbtTagTypeLookup) -> StreamValueResult {
        StreamValueResult::Continue
    }
}

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

    pub fn accept_tag_visitor<V: NbtTagVisitor>(&self, visitor: &mut V) {
        match self {
            Tag::End => visitor.visit_end(),
            Tag::Byte(value) => visitor.visit_byte(*value),
            Tag::Short(value) => visitor.visit_short(*value),
            Tag::Int(value) => visitor.visit_int(*value),
            Tag::Long(value) => visitor.visit_long(*value),
            Tag::Float(value) => visitor.visit_float(*value),
            Tag::Double(value) => visitor.visit_double(*value),
            Tag::ByteArray(values) => visitor.visit_byte_array(values),
            Tag::String(value) => visitor.visit_string(value),
            Tag::List(values) => visitor.visit_list(values),
            Tag::Compound(values) => visitor.visit_compound(values),
            Tag::IntArray(values) => visitor.visit_int_array(values),
            Tag::LongArray(values) => visitor.visit_long_array(values),
        }
    }
}
