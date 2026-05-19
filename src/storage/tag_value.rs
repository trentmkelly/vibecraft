use crate::storage::nbt::Tag;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TagValueProblem {
    pub path: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TagValueInput<'a> {
    path: String,
    input: &'a [(String, Tag)],
    problems: Vec<TagValueProblem>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TagValueOutput {
    path: String,
    output: Vec<(String, Tag)>,
    problems: Vec<TagValueProblem>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CompoundListInput<'a> {
    path: String,
    entries: Vec<&'a [(String, Tag)]>,
    problems: Vec<TagValueProblem>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CompoundListOutput {
    path: String,
    entries: Vec<Tag>,
    problems: Vec<TagValueProblem>,
}

impl<'a> TagValueInput<'a> {
    pub fn create(tag: &'a Tag) -> Self {
        match tag {
            Tag::Compound(values) => Self {
                path: String::new(),
                input: values,
                problems: Vec::new(),
            },
            other => Self {
                path: String::new(),
                input: &[],
                problems: vec![TagValueProblem {
                    path: String::new(),
                    message: format!("Expected CompoundTag, found {}", tag_type_name(other)),
                }],
            },
        }
    }

    fn with_path(path: String, input: &'a [(String, Tag)]) -> Self {
        Self {
            path,
            input,
            problems: Vec::new(),
        }
    }

    pub fn problems(&self) -> &[TagValueProblem] {
        &self.problems
    }

    pub fn child(&mut self, name: &str) -> Option<TagValueInput<'a>> {
        match self.field(name) {
            Some(Tag::Compound(values)) => Some(Self::with_path(self.child_path(name), values)),
            Some(tag) => {
                self.unexpected_type(name, "CompoundTag", tag);
                None
            }
            None => None,
        }
    }

    pub fn child_or_empty(&mut self, name: &str) -> TagValueInput<'a> {
        self.child(name)
            .unwrap_or_else(|| Self::with_path(self.child_path(name), &[]))
    }

    pub fn children_list(&mut self, name: &str) -> Option<CompoundListInput<'a>> {
        match self.field(name) {
            Some(Tag::List(values)) => {
                let mut entries = Vec::new();
                let mut problems = Vec::new();
                for (index, value) in values.iter().enumerate() {
                    match value {
                        Tag::Compound(fields) => entries.push(fields.as_slice()),
                        other => problems.push(TagValueProblem {
                            path: format!("{}[{}]", self.child_path(name), index),
                            message: format!(
                                "Expected CompoundTag, found {}",
                                tag_type_name(other)
                            ),
                        }),
                    }
                }
                Some(CompoundListInput {
                    path: self.child_path(name),
                    entries,
                    problems,
                })
            }
            Some(tag) => {
                self.unexpected_type(name, "ListTag", tag);
                None
            }
            None => None,
        }
    }

    pub fn children_list_or_empty(&mut self, name: &str) -> CompoundListInput<'a> {
        self.children_list(name)
            .unwrap_or_else(|| CompoundListInput {
                path: self.child_path(name),
                entries: Vec::new(),
                problems: Vec::new(),
            })
    }

    pub fn get_boolean_or(&mut self, name: &str, default_value: bool) -> bool {
        self.numeric_i64(name)
            .map(|value| value != 0)
            .unwrap_or(default_value)
    }

    pub fn get_byte_or(&mut self, name: &str, default_value: i8) -> i8 {
        self.numeric_i64(name)
            .map(|value| value as i8)
            .unwrap_or(default_value)
    }

    pub fn get_short_or(&mut self, name: &str, default_value: i16) -> i16 {
        self.numeric_i64(name)
            .map(|value| value as i16)
            .unwrap_or(default_value)
    }

    pub fn get_int(&mut self, name: &str) -> Option<i32> {
        self.numeric_i64(name).map(|value| value as i32)
    }

    pub fn get_int_or(&mut self, name: &str, default_value: i32) -> i32 {
        self.get_int(name).unwrap_or(default_value)
    }

    pub fn get_long(&mut self, name: &str) -> Option<i64> {
        self.numeric_i64(name)
    }

    pub fn get_long_or(&mut self, name: &str, default_value: i64) -> i64 {
        self.get_long(name).unwrap_or(default_value)
    }

    pub fn get_float_or(&mut self, name: &str, default_value: f32) -> f32 {
        self.numeric_f64(name)
            .map(|value| value as f32)
            .unwrap_or(default_value)
    }

    pub fn get_double_or(&mut self, name: &str, default_value: f64) -> f64 {
        self.numeric_f64(name).unwrap_or(default_value)
    }

    pub fn get_string(&mut self, name: &str) -> Option<&'a str> {
        match self.field(name) {
            Some(Tag::String(value)) => Some(value.as_str()),
            Some(tag) => {
                self.unexpected_type(name, "StringTag", tag);
                None
            }
            None => None,
        }
    }

    pub fn get_string_or<'b>(&'b mut self, name: &str, default_value: &'b str) -> &'b str
    where
        'a: 'b,
    {
        self.get_string(name).unwrap_or(default_value)
    }

    pub fn get_int_array(&mut self, name: &str) -> Option<&'a [i32]> {
        match self.field(name) {
            Some(Tag::IntArray(values)) => Some(values.as_slice()),
            Some(tag) => {
                self.unexpected_type(name, "IntArrayTag", tag);
                None
            }
            None => None,
        }
    }

    fn field(&self, name: &str) -> Option<&'a Tag> {
        self.input
            .iter()
            .find(|(field_name, _)| field_name == name)
            .map(|(_, tag)| tag)
    }

    fn numeric_i64(&mut self, name: &str) -> Option<i64> {
        match self.field(name) {
            Some(Tag::Byte(value)) => Some(*value as i64),
            Some(Tag::Short(value)) => Some(*value as i64),
            Some(Tag::Int(value)) => Some(*value as i64),
            Some(Tag::Long(value)) => Some(*value),
            Some(Tag::Float(value)) => Some(*value as i64),
            Some(Tag::Double(value)) => Some(*value as i64),
            Some(tag) => {
                self.problems.push(TagValueProblem {
                    path: self.child_path(name),
                    message: format!("Expected NumericTag, found {}", tag_type_name(tag)),
                });
                None
            }
            None => None,
        }
    }

    fn numeric_f64(&mut self, name: &str) -> Option<f64> {
        match self.field(name) {
            Some(Tag::Byte(value)) => Some(*value as f64),
            Some(Tag::Short(value)) => Some(*value as f64),
            Some(Tag::Int(value)) => Some(*value as f64),
            Some(Tag::Long(value)) => Some(*value as f64),
            Some(Tag::Float(value)) => Some(*value as f64),
            Some(Tag::Double(value)) => Some(*value),
            Some(tag) => {
                self.problems.push(TagValueProblem {
                    path: self.child_path(name),
                    message: format!("Expected NumericTag, found {}", tag_type_name(tag)),
                });
                None
            }
            None => None,
        }
    }

    fn unexpected_type(&mut self, name: &str, expected: &str, actual: &Tag) {
        self.problems.push(TagValueProblem {
            path: self.child_path(name),
            message: format!("Expected {expected}, found {}", tag_type_name(actual)),
        });
    }

    fn child_path(&self, name: &str) -> String {
        if self.path.is_empty() {
            name.to_string()
        } else {
            format!("{}.{}", self.path, name)
        }
    }
}

impl<'a> CompoundListInput<'a> {
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn problems(&self) -> &[TagValueProblem] {
        &self.problems
    }

    pub fn get(&self, index: usize) -> Option<TagValueInput<'a>> {
        self.entries
            .get(index)
            .map(|entry| TagValueInput::with_path(format!("{}[{}]", self.path, index), entry))
    }
}

impl Default for TagValueOutput {
    fn default() -> Self {
        Self::create_without_context()
    }
}

impl TagValueOutput {
    pub fn create_without_context() -> Self {
        Self {
            path: String::new(),
            output: Vec::new(),
            problems: Vec::new(),
        }
    }

    fn with_path(path: String) -> Self {
        Self {
            path,
            output: Vec::new(),
            problems: Vec::new(),
        }
    }

    pub fn problems(&self) -> &[TagValueProblem] {
        &self.problems
    }

    pub fn put_boolean(&mut self, name: &str, value: bool) {
        self.put(name, Tag::Byte(i8::from(value)));
    }

    pub fn put_byte(&mut self, name: &str, value: i8) {
        self.put(name, Tag::Byte(value));
    }

    pub fn put_short(&mut self, name: &str, value: i16) {
        self.put(name, Tag::Short(value));
    }

    pub fn put_int(&mut self, name: &str, value: i32) {
        self.put(name, Tag::Int(value));
    }

    pub fn put_long(&mut self, name: &str, value: i64) {
        self.put(name, Tag::Long(value));
    }

    pub fn put_float(&mut self, name: &str, value: f32) {
        self.put(name, Tag::Float(value));
    }

    pub fn put_double(&mut self, name: &str, value: f64) {
        self.put(name, Tag::Double(value));
    }

    pub fn put_string(&mut self, name: &str, value: impl Into<String>) {
        self.put(name, Tag::String(value.into()));
    }

    pub fn put_int_array(&mut self, name: &str, value: Vec<i32>) {
        self.put(name, Tag::IntArray(value));
    }

    pub fn child(&mut self, name: &str) -> TagValueOutput {
        let child = Self::with_path(self.child_path(name));
        self.put(name, Tag::Compound(Vec::new()));
        child
    }

    pub fn put_child(&mut self, name: &str, child: TagValueOutput) {
        self.problems.extend(child.problems);
        self.put(name, Tag::Compound(child.output));
    }

    pub fn children_list(&mut self, name: &str) -> CompoundListOutput {
        self.put(name, Tag::List(Vec::new()));
        CompoundListOutput {
            path: self.child_path(name),
            entries: Vec::new(),
            problems: Vec::new(),
        }
    }

    pub fn put_children_list(&mut self, name: &str, list: CompoundListOutput) {
        self.problems.extend(list.problems);
        self.put(name, Tag::List(list.entries));
    }

    pub fn discard(&mut self, name: &str) {
        self.output.retain(|(field_name, _)| field_name != name);
    }

    pub fn is_empty(&self) -> bool {
        self.output.is_empty()
    }

    pub fn build_result(self) -> Tag {
        Tag::Compound(self.output)
    }

    fn put(&mut self, name: &str, value: Tag) {
        if let Some((_, existing)) = self
            .output
            .iter_mut()
            .find(|(field_name, _)| field_name == name)
        {
            *existing = value;
        } else {
            self.output.push((name.to_string(), value));
        }
    }

    fn child_path(&self, name: &str) -> String {
        if self.path.is_empty() {
            name.to_string()
        } else {
            format!("{}.{}", self.path, name)
        }
    }
}

impl CompoundListOutput {
    pub fn add_child(&mut self, child: TagValueOutput) {
        self.problems.extend(child.problems);
        self.entries.push(Tag::Compound(child.output));
    }

    pub fn discard_last(&mut self) {
        self.entries.pop();
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

fn tag_type_name(tag: &Tag) -> &'static str {
    match tag {
        Tag::End => "EndTag",
        Tag::Byte(_) => "ByteTag",
        Tag::Short(_) => "ShortTag",
        Tag::Int(_) => "IntTag",
        Tag::Long(_) => "LongTag",
        Tag::Float(_) => "FloatTag",
        Tag::Double(_) => "DoubleTag",
        Tag::ByteArray(_) => "ByteArrayTag",
        Tag::String(_) => "StringTag",
        Tag::List(_) => "ListTag",
        Tag::Compound(_) => "CompoundTag",
        Tag::IntArray(_) => "IntArrayTag",
        Tag::LongArray(_) => "LongArrayTag",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tag_value_input_reads_vanilla_numeric_string_arrays_and_children() {
        let tag = Tag::Compound(vec![
            ("flag".to_string(), Tag::Byte(1)),
            ("byte".to_string(), Tag::Byte(-2)),
            ("short".to_string(), Tag::Short(300)),
            ("int".to_string(), Tag::Int(42)),
            ("long".to_string(), Tag::Long(9_000_000_000)),
            ("float".to_string(), Tag::Float(1.5)),
            ("double".to_string(), Tag::Double(2.25)),
            ("name".to_string(), Tag::String("Steve".to_string())),
            ("ints".to_string(), Tag::IntArray(vec![1, 2, 3])),
            (
                "child".to_string(),
                Tag::Compound(vec![("value".to_string(), Tag::Int(7))]),
            ),
            (
                "children".to_string(),
                Tag::List(vec![
                    Tag::Compound(vec![("index".to_string(), Tag::Int(0))]),
                    Tag::Compound(vec![("index".to_string(), Tag::Int(1))]),
                ]),
            ),
        ]);
        let mut input = TagValueInput::create(&tag);

        assert!(input.get_boolean_or("flag", false));
        assert_eq!(input.get_byte_or("byte", 0), -2);
        assert_eq!(input.get_short_or("short", 0), 300);
        assert_eq!(input.get_int("int"), Some(42));
        assert_eq!(input.get_long("long"), Some(9_000_000_000));
        assert_eq!(input.get_float_or("float", 0.0), 1.5);
        assert_eq!(input.get_double_or("double", 0.0), 2.25);
        assert_eq!(input.get_string("name"), Some("Steve"));
        assert_eq!(input.get_int_array("ints"), Some([1, 2, 3].as_slice()));
        assert_eq!(input.get_int_or("missing", 12), 12);

        let mut child = input.child("child").unwrap();
        assert_eq!(child.get_int("value"), Some(7));
        let children = input.children_list("children").unwrap();
        assert_eq!(children.len(), 2);
        assert_eq!(children.get(1).unwrap().get_int("index"), Some(1));
        assert!(input.problems().is_empty());
    }

    #[test]
    fn tag_value_input_reports_type_mismatches_and_returns_defaults() {
        let tag = Tag::Compound(vec![
            (
                "number".to_string(),
                Tag::String("not a number".to_string()),
            ),
            ("name".to_string(), Tag::Int(5)),
            ("children".to_string(), Tag::List(vec![Tag::Int(1)])),
        ]);
        let mut input = TagValueInput::create(&tag);

        assert_eq!(input.get_int_or("number", 99), 99);
        assert_eq!(input.get_string("name"), None);
        let children = input.children_list("children").unwrap();

        assert_eq!(input.problems().len(), 2);
        assert_eq!(children.problems().len(), 1);
        assert!(input.problems()[0].message.contains("Expected NumericTag"));
        assert!(input.problems()[1].message.contains("Expected StringTag"));
        assert!(children.problems()[0]
            .message
            .contains("Expected CompoundTag"));
    }

    #[test]
    fn tag_value_output_writes_discards_and_replaces_fields() {
        let mut output = TagValueOutput::create_without_context();
        output.put_boolean("flag", true);
        output.put_int("count", 1);
        output.put_int("count", 2);
        output.put_string("name", "Alex");
        output.discard("name");

        let mut child = output.child("child");
        child.put_long("ticks", 20);
        output.put_child("child", child);

        let mut list = output.children_list("children");
        let mut entry = TagValueOutput::create_without_context();
        entry.put_int("index", 0);
        list.add_child(entry);
        output.put_children_list("children", list);

        assert_eq!(
            output.build_result(),
            Tag::Compound(vec![
                ("flag".to_string(), Tag::Byte(1)),
                ("count".to_string(), Tag::Int(2)),
                (
                    "child".to_string(),
                    Tag::Compound(vec![("ticks".to_string(), Tag::Long(20))]),
                ),
                (
                    "children".to_string(),
                    Tag::List(vec![Tag::Compound(vec![(
                        "index".to_string(),
                        Tag::Int(0)
                    )])]),
                ),
            ])
        );
    }
}
