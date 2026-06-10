use std::collections::HashMap;

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NbtFieldSelectorSpec {
    pub path: Vec<String>,
    pub tag_type: NbtTagTypeLookup,
    pub name: String,
}

impl NbtFieldSelectorSpec {
    pub fn new(path: Vec<String>, tag_type: NbtTagTypeLookup, name: impl Into<String>) -> Self {
        Self {
            path,
            tag_type,
            name: name.into(),
        }
    }

    pub fn root(tag_type: NbtTagTypeLookup, name: impl Into<String>) -> Self {
        Self::new(Vec::new(), tag_type, name)
    }

    pub fn child(
        parent: impl Into<String>,
        tag_type: NbtTagTypeLookup,
        name: impl Into<String>,
    ) -> Self {
        Self::new(vec![parent.into()], tag_type, name)
    }

    pub fn grandchild(
        grandparent: impl Into<String>,
        parent: impl Into<String>,
        tag_type: NbtTagTypeLookup,
        name: impl Into<String>,
    ) -> Self {
        Self::new(vec![grandparent.into(), parent.into()], tag_type, name)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NbtFieldTree {
    pub depth: usize,
    pub selected_fields: HashMap<String, NbtTagTypeLookup>,
    pub fields_to_recurse: HashMap<String, NbtFieldTree>,
}

impl NbtFieldTree {
    pub fn create_root() -> Self {
        Self::new(1)
    }

    fn new(depth: usize) -> Self {
        Self {
            depth,
            selected_fields: HashMap::new(),
            fields_to_recurse: HashMap::new(),
        }
    }

    pub fn add_entry(&mut self, field: &NbtFieldSelectorSpec) {
        if self.depth <= field.path.len() {
            self.fields_to_recurse
                .entry(field.path[self.depth - 1].clone())
                .or_insert_with(|| Self::new(self.depth + 1))
                .add_entry(field);
        } else {
            self.selected_fields
                .insert(field.name.clone(), field.tag_type.clone());
        }
    }

    pub fn is_selected(&self, tag_type: &NbtTagTypeLookup, id: &str) -> bool {
        self.selected_fields.get(id) == Some(tag_type)
    }

    fn remove_selected(&mut self, tag_type: &NbtTagTypeLookup, id: &str) -> bool {
        if self.is_selected(tag_type, id) {
            self.selected_fields.remove(id);
            true
        } else {
            false
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CollectToTagVisitor {
    container_stack: Vec<ContainerBuilder>,
}

impl CollectToTagVisitor {
    pub fn new() -> Self {
        Self {
            container_stack: vec![ContainerBuilder::Root { result: None }],
        }
    }

    pub fn get_result(&self) -> Option<&Tag> {
        self.container_stack
            .first()
            .and_then(ContainerBuilder::build_ref)
    }

    pub fn depth(&self) -> usize {
        self.container_stack.len().saturating_sub(1)
    }

    fn append_entry(&mut self, tag: Tag) {
        if let Some(container) = self.container_stack.last_mut() {
            container.accept_value(tag);
        }
    }

    fn enter_container_if_needed(&mut self, tag_type: &NbtTagTypeLookup) {
        if tag_type_id(tag_type) == Some(9) {
            self.container_stack
                .push(ContainerBuilder::List { values: Vec::new() });
        } else if tag_type_id(tag_type) == Some(10) {
            self.container_stack.push(ContainerBuilder::Compound {
                values: Vec::new(),
                last_id: String::new(),
            });
        }
    }
}

impl Default for CollectToTagVisitor {
    fn default() -> Self {
        Self::new()
    }
}

impl NbtStreamTagVisitor for CollectToTagVisitor {
    fn visit_end(&mut self) -> StreamValueResult {
        self.append_entry(Tag::End);
        StreamValueResult::Continue
    }

    fn visit_string(&mut self, value: &str) -> StreamValueResult {
        self.append_entry(Tag::String(value.to_string()));
        StreamValueResult::Continue
    }

    fn visit_byte(&mut self, value: i8) -> StreamValueResult {
        self.append_entry(Tag::Byte(value));
        StreamValueResult::Continue
    }

    fn visit_short(&mut self, value: i16) -> StreamValueResult {
        self.append_entry(Tag::Short(value));
        StreamValueResult::Continue
    }

    fn visit_int(&mut self, value: i32) -> StreamValueResult {
        self.append_entry(Tag::Int(value));
        StreamValueResult::Continue
    }

    fn visit_long(&mut self, value: i64) -> StreamValueResult {
        self.append_entry(Tag::Long(value));
        StreamValueResult::Continue
    }

    fn visit_float(&mut self, value: f32) -> StreamValueResult {
        self.append_entry(Tag::Float(value));
        StreamValueResult::Continue
    }

    fn visit_double(&mut self, value: f64) -> StreamValueResult {
        self.append_entry(Tag::Double(value));
        StreamValueResult::Continue
    }

    fn visit_byte_array(&mut self, value: &[i8]) -> StreamValueResult {
        self.append_entry(Tag::ByteArray(value.to_vec()));
        StreamValueResult::Continue
    }

    fn visit_int_array(&mut self, value: &[i32]) -> StreamValueResult {
        self.append_entry(Tag::IntArray(value.to_vec()));
        StreamValueResult::Continue
    }

    fn visit_long_array(&mut self, value: &[i64]) -> StreamValueResult {
        self.append_entry(Tag::LongArray(value.to_vec()));
        StreamValueResult::Continue
    }

    fn visit_list(&mut self, _element_type: NbtTagTypeLookup, _size: usize) -> StreamValueResult {
        StreamValueResult::Continue
    }

    fn visit_element(&mut self, tag_type: NbtTagTypeLookup, _index: usize) -> StreamEntryResult {
        self.enter_container_if_needed(&tag_type);
        StreamEntryResult::Enter
    }

    fn visit_entry(&mut self, tag_type: NbtTagTypeLookup) -> StreamEntryResult {
        self.enter_container_if_needed(&tag_type);
        StreamEntryResult::Enter
    }

    fn visit_named_entry(&mut self, tag_type: NbtTagTypeLookup, id: &str) -> StreamEntryResult {
        if let Some(container) = self.container_stack.last_mut() {
            container.accept_key(id);
        }
        self.enter_container_if_needed(&tag_type);
        StreamEntryResult::Enter
    }

    fn visit_container_end(&mut self) -> StreamValueResult {
        if self.container_stack.len() > 1 {
            if let Some(container) = self.container_stack.pop() {
                if let Some(tag) = container.build() {
                    self.append_entry(tag);
                }
            }
        }
        StreamValueResult::Continue
    }

    fn visit_root_entry(&mut self, tag_type: NbtTagTypeLookup) -> StreamValueResult {
        self.enter_container_if_needed(&tag_type);
        StreamValueResult::Continue
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SkipFieldsVisitor {
    collector: CollectToTagVisitor,
    stack: Vec<NbtFieldTree>,
}

impl SkipFieldsVisitor {
    pub fn new(wanted_fields: &[NbtFieldSelectorSpec]) -> Self {
        let mut root_frame = NbtFieldTree::create_root();
        for wanted_field in wanted_fields {
            root_frame.add_entry(wanted_field);
        }
        Self {
            collector: CollectToTagVisitor::new(),
            stack: vec![root_frame],
        }
    }

    pub fn get_result(&self) -> Option<&Tag> {
        self.collector.get_result()
    }

    pub fn depth(&self) -> usize {
        self.collector.depth()
    }
}

impl NbtStreamTagVisitor for SkipFieldsVisitor {
    fn visit_end(&mut self) -> StreamValueResult {
        self.collector.visit_end()
    }

    fn visit_string(&mut self, value: &str) -> StreamValueResult {
        self.collector.visit_string(value)
    }

    fn visit_byte(&mut self, value: i8) -> StreamValueResult {
        self.collector.visit_byte(value)
    }

    fn visit_short(&mut self, value: i16) -> StreamValueResult {
        self.collector.visit_short(value)
    }

    fn visit_int(&mut self, value: i32) -> StreamValueResult {
        self.collector.visit_int(value)
    }

    fn visit_long(&mut self, value: i64) -> StreamValueResult {
        self.collector.visit_long(value)
    }

    fn visit_float(&mut self, value: f32) -> StreamValueResult {
        self.collector.visit_float(value)
    }

    fn visit_double(&mut self, value: f64) -> StreamValueResult {
        self.collector.visit_double(value)
    }

    fn visit_byte_array(&mut self, value: &[i8]) -> StreamValueResult {
        self.collector.visit_byte_array(value)
    }

    fn visit_int_array(&mut self, value: &[i32]) -> StreamValueResult {
        self.collector.visit_int_array(value)
    }

    fn visit_long_array(&mut self, value: &[i64]) -> StreamValueResult {
        self.collector.visit_long_array(value)
    }

    fn visit_list(&mut self, element_type: NbtTagTypeLookup, size: usize) -> StreamValueResult {
        self.collector.visit_list(element_type, size)
    }

    fn visit_entry(&mut self, tag_type: NbtTagTypeLookup) -> StreamEntryResult {
        self.collector.visit_entry(tag_type)
    }

    fn visit_named_entry(&mut self, tag_type: NbtTagTypeLookup, id: &str) -> StreamEntryResult {
        if let Some(current_frame) = self.stack.last() {
            if current_frame.is_selected(&tag_type, id) {
                return StreamEntryResult::Skip;
            }

            if tag_type_id(&tag_type) == Some(10) {
                if let Some(new_frame) = current_frame.fields_to_recurse.get(id) {
                    self.stack.push(new_frame.clone());
                }
            }
        }

        self.collector.visit_named_entry(tag_type, id)
    }

    fn visit_element(&mut self, tag_type: NbtTagTypeLookup, index: usize) -> StreamEntryResult {
        self.collector.visit_element(tag_type, index)
    }

    fn visit_container_end(&mut self) -> StreamValueResult {
        if self
            .stack
            .last()
            .is_some_and(|frame| self.depth() == frame.depth)
        {
            self.stack.pop();
        }

        self.collector.visit_container_end()
    }

    fn visit_root_entry(&mut self, tag_type: NbtTagTypeLookup) -> StreamValueResult {
        self.collector.visit_root_entry(tag_type)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CollectFieldsVisitor {
    collector: CollectToTagVisitor,
    fields_to_get_count: usize,
    wanted_type_ids: Vec<u8>,
    stack: Vec<NbtFieldTree>,
}

impl CollectFieldsVisitor {
    pub fn new(wanted_fields: &[NbtFieldSelectorSpec]) -> Self {
        let mut root_frame = NbtFieldTree::create_root();
        let mut wanted_type_ids = Vec::new();
        for wanted_field in wanted_fields {
            root_frame.add_entry(wanted_field);
            push_unique_type_id(&mut wanted_type_ids, &wanted_field.tag_type);
        }
        push_unique_type_id(&mut wanted_type_ids, &tag_type(10));

        Self {
            collector: CollectToTagVisitor::new(),
            fields_to_get_count: wanted_fields.len(),
            wanted_type_ids,
            stack: vec![root_frame],
        }
    }

    pub fn get_result(&self) -> Option<&Tag> {
        self.collector.get_result()
    }

    pub fn depth(&self) -> usize {
        self.collector.depth()
    }

    pub fn missing_field_count(&self) -> usize {
        self.fields_to_get_count
    }

    fn wants_type(&self, tag_type: &NbtTagTypeLookup) -> bool {
        tag_type_id(tag_type).is_some_and(|id| self.wanted_type_ids.contains(&id))
    }
}

impl NbtStreamTagVisitor for CollectFieldsVisitor {
    fn visit_end(&mut self) -> StreamValueResult {
        self.collector.visit_end()
    }

    fn visit_string(&mut self, value: &str) -> StreamValueResult {
        self.collector.visit_string(value)
    }

    fn visit_byte(&mut self, value: i8) -> StreamValueResult {
        self.collector.visit_byte(value)
    }

    fn visit_short(&mut self, value: i16) -> StreamValueResult {
        self.collector.visit_short(value)
    }

    fn visit_int(&mut self, value: i32) -> StreamValueResult {
        self.collector.visit_int(value)
    }

    fn visit_long(&mut self, value: i64) -> StreamValueResult {
        self.collector.visit_long(value)
    }

    fn visit_float(&mut self, value: f32) -> StreamValueResult {
        self.collector.visit_float(value)
    }

    fn visit_double(&mut self, value: f64) -> StreamValueResult {
        self.collector.visit_double(value)
    }

    fn visit_byte_array(&mut self, value: &[i8]) -> StreamValueResult {
        self.collector.visit_byte_array(value)
    }

    fn visit_int_array(&mut self, value: &[i32]) -> StreamValueResult {
        self.collector.visit_int_array(value)
    }

    fn visit_long_array(&mut self, value: &[i64]) -> StreamValueResult {
        self.collector.visit_long_array(value)
    }

    fn visit_list(&mut self, element_type: NbtTagTypeLookup, size: usize) -> StreamValueResult {
        self.collector.visit_list(element_type, size)
    }

    fn visit_entry(&mut self, tag_type: NbtTagTypeLookup) -> StreamEntryResult {
        if self
            .stack
            .last()
            .is_some_and(|current_frame| self.depth() > current_frame.depth)
        {
            return StreamEntryResult::Enter;
        }

        if self.fields_to_get_count == 0 {
            StreamEntryResult::Break
        } else if !self.wants_type(&tag_type) {
            StreamEntryResult::Skip
        } else {
            StreamEntryResult::Enter
        }
    }

    fn visit_named_entry(&mut self, tag_type: NbtTagTypeLookup, id: &str) -> StreamEntryResult {
        if self
            .stack
            .last()
            .is_some_and(|current_frame| self.depth() > current_frame.depth)
        {
            return self.collector.visit_named_entry(tag_type, id);
        }

        if self
            .stack
            .last_mut()
            .is_some_and(|current_frame| current_frame.remove_selected(&tag_type, id))
        {
            self.fields_to_get_count = self.fields_to_get_count.saturating_sub(1);
            return self.collector.visit_named_entry(tag_type, id);
        }

        if tag_type_id(&tag_type) == Some(10) {
            let new_frame = self
                .stack
                .last()
                .and_then(|current_frame| current_frame.fields_to_recurse.get(id))
                .cloned();
            if let Some(new_frame) = new_frame {
                self.stack.push(new_frame);
                return self.collector.visit_named_entry(tag_type, id);
            }
        }

        StreamEntryResult::Skip
    }

    fn visit_element(&mut self, tag_type: NbtTagTypeLookup, index: usize) -> StreamEntryResult {
        self.collector.visit_element(tag_type, index)
    }

    fn visit_container_end(&mut self) -> StreamValueResult {
        if self
            .stack
            .last()
            .is_some_and(|frame| self.depth() == frame.depth)
        {
            self.stack.pop();
        }

        self.collector.visit_container_end()
    }

    fn visit_root_entry(&mut self, tag_type: NbtTagTypeLookup) -> StreamValueResult {
        if tag_type_id(&tag_type) != Some(10) {
            StreamValueResult::Halt
        } else {
            self.collector.visit_root_entry(tag_type)
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
enum ContainerBuilder {
    Root {
        result: Option<Tag>,
    },
    List {
        values: Vec<Tag>,
    },
    Compound {
        values: Vec<(String, Tag)>,
        last_id: String,
    },
}

impl ContainerBuilder {
    fn accept_key(&mut self, id: &str) {
        if let Self::Compound { last_id, .. } = self {
            *last_id = id.to_string();
        }
    }

    fn accept_value(&mut self, tag: Tag) {
        match self {
            Self::Root { result } => *result = Some(tag),
            Self::List { values } => values.push(try_unwrap_list_compound(tag)),
            Self::Compound { values, last_id } => values.push((last_id.clone(), tag)),
        }
    }

    fn build(self) -> Option<Tag> {
        match self {
            Self::Root { result } => result,
            Self::List { values } => Some(Tag::List(values)),
            Self::Compound { values, .. } => Some(Tag::Compound(values)),
        }
    }

    fn build_ref(&self) -> Option<&Tag> {
        match self {
            Self::Root { result } => result.as_ref(),
            Self::List { .. } | Self::Compound { .. } => None,
        }
    }
}

fn try_unwrap_list_compound(tag: Tag) -> Tag {
    match tag {
        Tag::Compound(mut entries) if entries.len() == 1 && entries[0].0.is_empty() => {
            entries.remove(0).1
        }
        tag => tag,
    }
}

fn tag_type_id(tag_type: &NbtTagTypeLookup) -> Option<u8> {
    match tag_type {
        NbtTagTypeLookup::Known(info) => Some(info.id),
        NbtTagTypeLookup::Invalid { .. } => None,
    }
}

fn push_unique_type_id(ids: &mut Vec<u8>, tag_type: &NbtTagTypeLookup) {
    if let Some(id) = tag_type_id(tag_type) {
        if !ids.contains(&id) {
            ids.push(id);
        }
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
