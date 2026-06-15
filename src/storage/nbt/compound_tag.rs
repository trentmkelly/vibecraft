#![allow(dead_code)]

use super::Tag;

pub fn put_compound_entry(
    entries: &mut Vec<(String, Tag)>,
    name: impl Into<String>,
    tag: Tag,
) -> Option<Tag> {
    let name = name.into();
    if let Some((_, value)) = entries.iter_mut().find(|(key, _)| key == &name) {
        return Some(std::mem::replace(value, tag));
    }
    entries.push((name, tag));
    None
}

pub fn remove_compound_entry(entries: &mut Vec<(String, Tag)>, name: &str) -> Option<Tag> {
    entries
        .iter()
        .position(|(key, _)| key == name)
        .map(|index| entries.remove(index).1)
}

pub fn compound_get<'a>(entries: &'a [(String, Tag)], name: &str) -> Option<&'a Tag> {
    entries
        .iter()
        .find(|(key, _)| key == name)
        .map(|(_, tag)| tag)
}

pub fn compound_get_mut<'a>(entries: &'a mut [(String, Tag)], name: &str) -> Option<&'a mut Tag> {
    entries
        .iter_mut()
        .find(|(key, _)| key == name)
        .map(|(_, tag)| tag)
}

pub fn compound_shallow_copy(entries: &[(String, Tag)]) -> Vec<(String, Tag)> {
    entries.to_vec()
}

pub fn compound_deep_copy(entries: &[(String, Tag)]) -> Vec<(String, Tag)> {
    entries
        .iter()
        .map(|(key, value)| (key.clone(), value.copy_tag()))
        .collect()
}

pub fn merge_compound_entries(target: &mut Vec<(String, Tag)>, other: &[(String, Tag)]) {
    for (name, other_tag) in other {
        match (compound_get_mut(target, name), other_tag) {
            (Some(Tag::Compound(self_entries)), Tag::Compound(other_entries)) => {
                merge_compound_entries(self_entries, other_entries);
            }
            _ => {
                put_compound_entry(target, name.clone(), other_tag.copy_tag());
            }
        }
    }
}

#[allow(dead_code)]
impl Tag {
    pub fn compound_len(&self) -> Option<usize> {
        self.as_compound().map(<[_]>::len)
    }

    pub fn compound_is_empty(&self) -> Option<bool> {
        self.as_compound().map(<[_]>::is_empty)
    }

    pub fn compound_keys(&self) -> Option<Vec<&str>> {
        self.as_compound()
            .map(|entries| entries.iter().map(|(key, _)| key.as_str()).collect())
    }

    pub fn compound_values(&self) -> Option<Vec<&Tag>> {
        self.as_compound()
            .map(|entries| entries.iter().map(|(_, value)| value).collect())
    }

    pub fn compound_get(&self, name: &str) -> Option<&Tag> {
        self.as_compound()
            .and_then(|entries| compound_get(entries, name))
    }

    pub fn compound_contains(&self, name: &str) -> bool {
        self.compound_get(name).is_some()
    }

    pub fn compound_put(&mut self, name: impl Into<String>, tag: Tag) -> Option<Tag> {
        match self {
            Tag::Compound(entries) => put_compound_entry(entries, name, tag),
            _ => None,
        }
    }

    pub fn compound_remove(&mut self, name: &str) -> Option<Tag> {
        match self {
            Tag::Compound(entries) => remove_compound_entry(entries, name),
            _ => None,
        }
    }

    pub fn compound_put_byte(&mut self, name: impl Into<String>, value: i8) {
        self.compound_put(name, Tag::Byte(value));
    }

    pub fn compound_put_short(&mut self, name: impl Into<String>, value: i16) {
        self.compound_put(name, Tag::Short(value));
    }

    pub fn compound_put_int(&mut self, name: impl Into<String>, value: i32) {
        self.compound_put(name, Tag::Int(value));
    }

    pub fn compound_put_long(&mut self, name: impl Into<String>, value: i64) {
        self.compound_put(name, Tag::Long(value));
    }

    pub fn compound_put_float(&mut self, name: impl Into<String>, value: f32) {
        self.compound_put(name, Tag::Float(value));
    }

    pub fn compound_put_double(&mut self, name: impl Into<String>, value: f64) {
        self.compound_put(name, Tag::Double(value));
    }

    pub fn compound_put_string(&mut self, name: impl Into<String>, value: impl Into<String>) {
        self.compound_put(name, Tag::String(value.into()));
    }

    pub fn compound_put_byte_array(&mut self, name: impl Into<String>, value: Vec<i8>) {
        self.compound_put(name, Tag::ByteArray(value));
    }

    pub fn compound_put_int_array(&mut self, name: impl Into<String>, value: Vec<i32>) {
        self.compound_put(name, Tag::IntArray(value));
    }

    pub fn compound_put_long_array(&mut self, name: impl Into<String>, value: Vec<i64>) {
        self.compound_put(name, Tag::LongArray(value));
    }

    pub fn compound_put_boolean(&mut self, name: impl Into<String>, value: bool) {
        self.compound_put(name, Tag::Byte(i8::from(value)));
    }

    pub fn compound_get_byte(&self, name: &str) -> Option<i8> {
        self.compound_get(name).and_then(Tag::as_byte)
    }

    pub fn compound_get_byte_or(&self, name: &str, default_value: i8) -> i8 {
        self.compound_get_byte(name).unwrap_or(default_value)
    }

    pub fn compound_get_short(&self, name: &str) -> Option<i16> {
        self.compound_get(name).and_then(Tag::as_short)
    }

    pub fn compound_get_short_or(&self, name: &str, default_value: i16) -> i16 {
        self.compound_get_short(name).unwrap_or(default_value)
    }

    pub fn compound_get_int(&self, name: &str) -> Option<i32> {
        self.compound_get(name).and_then(Tag::as_int)
    }

    pub fn compound_get_int_or(&self, name: &str, default_value: i32) -> i32 {
        self.compound_get_int(name).unwrap_or(default_value)
    }

    pub fn compound_get_long(&self, name: &str) -> Option<i64> {
        self.compound_get(name).and_then(Tag::as_long)
    }

    pub fn compound_get_long_or(&self, name: &str, default_value: i64) -> i64 {
        self.compound_get_long(name).unwrap_or(default_value)
    }

    pub fn compound_get_float(&self, name: &str) -> Option<f32> {
        self.compound_get(name).and_then(Tag::as_float)
    }

    pub fn compound_get_float_or(&self, name: &str, default_value: f32) -> f32 {
        self.compound_get_float(name).unwrap_or(default_value)
    }

    pub fn compound_get_double(&self, name: &str) -> Option<f64> {
        self.compound_get(name).and_then(Tag::as_double)
    }

    pub fn compound_get_double_or(&self, name: &str, default_value: f64) -> f64 {
        self.compound_get_double(name).unwrap_or(default_value)
    }

    pub fn compound_get_string(&self, name: &str) -> Option<&str> {
        self.compound_get(name).and_then(Tag::as_string)
    }

    pub fn compound_get_string_or<'a>(&'a self, name: &str, default_value: &'a str) -> &'a str {
        self.compound_get_string(name).unwrap_or(default_value)
    }

    pub fn compound_get_byte_array(&self, name: &str) -> Option<&[i8]> {
        self.compound_get(name).and_then(Tag::as_byte_array)
    }

    pub fn compound_get_int_array(&self, name: &str) -> Option<&[i32]> {
        self.compound_get(name).and_then(Tag::as_int_array)
    }

    pub fn compound_get_long_array(&self, name: &str) -> Option<&[i64]> {
        self.compound_get(name).and_then(Tag::as_long_array)
    }

    pub fn compound_get_compound(&self, name: &str) -> Option<&[(String, Tag)]> {
        self.compound_get(name).and_then(Tag::as_compound)
    }

    pub fn compound_get_compound_or_empty(&self, name: &str) -> Vec<(String, Tag)> {
        self.compound_get_compound(name)
            .map(compound_shallow_copy)
            .unwrap_or_default()
    }

    pub fn compound_get_list(&self, name: &str) -> Option<&[Tag]> {
        self.compound_get(name).and_then(Tag::as_list)
    }

    pub fn compound_get_list_or_empty(&self, name: &str) -> Vec<Tag> {
        self.compound_get_list(name)
            .map(<[_]>::to_vec)
            .unwrap_or_default()
    }

    pub fn compound_get_boolean(&self, name: &str) -> Option<bool> {
        self.compound_get(name).and_then(Tag::as_boolean)
    }

    pub fn compound_get_boolean_or(&self, name: &str, default_value: bool) -> bool {
        self.compound_get_byte_or(name, i8::from(default_value)) != 0
    }

    pub fn compound_shallow_copy(&self) -> Option<Tag> {
        self.as_compound()
            .map(|entries| Tag::Compound(compound_shallow_copy(entries)))
    }

    pub fn compound_copy(&self) -> Option<Tag> {
        self.as_compound()
            .map(|entries| Tag::Compound(compound_deep_copy(entries)))
    }

    pub fn compound_merge(&mut self, other: &Tag) -> bool {
        match (self, other) {
            (Tag::Compound(target), Tag::Compound(source)) => {
                merge_compound_entries(target, source);
                true
            }
            _ => false,
        }
    }
}

#[cfg(all(test, vibecraft_has_decompiled_sources))]
mod tests {
    use super::*;

    const COMPOUND_TAG_JAVA: &str =
        vibecraft_java_source!("/net/minecraft/nbt/CompoundTag.java");

    #[test]
    fn compound_tag_matches_java_map_put_get_and_default_accessors() {
        for sentinel in [
            "private static final int SELF_SIZE_IN_BYTES = 48;",
            "private static final int MAP_ENTRY_SIZE_IN_BYTES = 32;",
            "public @Nullable Tag put(final String name, final Tag tag)",
            "public boolean contains(final String name)",
            "return this.tags.get(name) instanceof NumericTag tag ? tag.intValue() : defaultValue;",
            "return this.tags.get(name) instanceof StringTag(String var8) ? var8 : defaultValue;",
            "return this.getByteOr(string, (byte)(defaultValue ? 1 : 0)) != 0;",
        ] {
            assert!(
                COMPOUND_TAG_JAVA.contains(sentinel),
                "missing CompoundTag sentinel {sentinel}"
            );
        }

        let mut tag = Tag::Compound(Vec::new());
        assert_eq!(tag.compound_len(), Some(0));
        assert_eq!(tag.compound_is_empty(), Some(true));

        tag.compound_put_int("answer", 42);
        assert_eq!(
            tag.compound_put("answer", Tag::Long(300)),
            Some(Tag::Int(42))
        );
        tag.compound_put_string("name", "Steve");
        tag.compound_put_boolean("flag", true);
        tag.compound_put_byte_array("bytes", vec![1, -2]);
        tag.compound_put_int_array("ints", vec![3, 4]);
        tag.compound_put_long_array("longs", vec![5, 6]);

        assert!(tag.compound_contains("answer"));
        assert_eq!(tag.compound_len(), Some(6));
        assert_eq!(tag.compound_get_int("answer"), Some(300));
        assert_eq!(tag.compound_get_int_or("missing", 7), 7);
        assert_eq!(tag.compound_get_byte_or("answer", 0), 44);
        assert_eq!(tag.compound_get_string("name"), Some("Steve"));
        assert_eq!(tag.compound_get_string_or("answer", "fallback"), "fallback");
        assert_eq!(tag.compound_get_boolean("flag"), Some(true));
        assert!(tag.compound_get_boolean_or("missing", true));
        assert_eq!(tag.compound_get_byte_array("bytes"), Some(&[1, -2][..]));
        assert_eq!(tag.compound_get_int_array("ints"), Some(&[3, 4][..]));
        assert_eq!(tag.compound_get_long_array("longs"), Some(&[5, 6][..]));
        assert_eq!(tag.compound_remove("answer"), Some(Tag::Long(300)));
        assert!(!tag.compound_contains("answer"));
    }

    #[test]
    fn compound_tag_copy_merge_and_duplicate_read_match_java_map_semantics() {
        for sentinel in [
            "protected CompoundTag shallowCopy()",
            "public CompoundTag copy()",
            "newTags.put(key, tag.copy())",
            "public CompoundTag merge(final CompoundTag other)",
            "selfCompound.merge(otherCompound);",
            "values.put(key, tag)",
        ] {
            assert!(
                COMPOUND_TAG_JAVA.contains(sentinel),
                "missing CompoundTag sentinel {sentinel}"
            );
        }

        let mut base = Tag::Compound(vec![
            (
                "nested".to_string(),
                Tag::Compound(vec![("keep".to_string(), Tag::Int(1))]),
            ),
            ("replace".to_string(), Tag::String("old".to_string())),
        ]);
        let other = Tag::Compound(vec![
            (
                "nested".to_string(),
                Tag::Compound(vec![("add".to_string(), Tag::Int(2))]),
            ),
            ("replace".to_string(), Tag::String("new".to_string())),
        ]);
        assert!(base.compound_merge(&other));
        assert_eq!(
            base,
            Tag::Compound(vec![
                (
                    "nested".to_string(),
                    Tag::Compound(vec![
                        ("keep".to_string(), Tag::Int(1)),
                        ("add".to_string(), Tag::Int(2))
                    ])
                ),
                ("replace".to_string(), Tag::String("new".to_string()))
            ])
        );
        assert_eq!(base.compound_shallow_copy(), Some(base.clone()));
        assert_eq!(base.compound_copy(), Some(base.clone()));

        let mut payload = Vec::new();
        payload.push(3);
        super::super::write_string(&mut payload, "dup").unwrap();
        payload.extend_from_slice(&1_i32.to_be_bytes());
        payload.push(3);
        super::super::write_string(&mut payload, "dup").unwrap();
        payload.extend_from_slice(&2_i32.to_be_bytes());
        payload.push(0);
        assert_eq!(
            Tag::read_payload(10, &mut payload.as_slice()).unwrap(),
            Tag::Compound(vec![("dup".to_string(), Tag::Int(2))])
        );
    }
}
