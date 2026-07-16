//! String-name lookup and codecs matching Minecraft's `StringRepresentable`.

#![allow(dead_code)]

pub const PRE_BUILT_MAP_THRESHOLD: usize = 16;

pub trait StringRepresentable {
    fn get_serialized_name(&self) -> &str;
}

#[derive(Debug, Clone)]
pub struct NameLookup<T> {
    values: Vec<T>,
    names: Vec<String>,
}

impl<T> NameLookup<T> {
    pub fn new(values: Vec<T>, converter: impl Fn(&T) -> String) -> Result<Self, String> {
        let names = values.iter().map(converter).collect::<Vec<_>>();
        if values.len() > PRE_BUILT_MAP_THRESHOLD {
            for (index, name) in names.iter().enumerate() {
                if names[..index].iter().any(|existing| existing == name) {
                    return Err(format!("duplicate key: {name}"));
                }
            }
        }
        Ok(Self { values, names })
    }

    pub fn get(&self, name: &str) -> Option<&T> {
        self.names
            .iter()
            .position(|candidate| candidate == name)
            .map(|index| &self.values[index])
    }

    pub fn values(&self) -> &[T] {
        &self.values
    }
}

pub fn create_name_lookup<T: StringRepresentable>(values: Vec<T>) -> Result<NameLookup<T>, String> {
    NameLookup::new(values, |value| value.get_serialized_name().to_string())
}

pub fn keys(values: &[impl StringRepresentable]) -> Vec<String> {
    values
        .iter()
        .map(|value| value.get_serialized_name().to_string())
        .collect()
}

#[derive(Debug, Clone)]
pub struct StringRepresentableCodec<T> {
    values: Vec<T>,
    lookup: NameLookup<T>,
}

impl<T> StringRepresentableCodec<T>
where
    T: Clone + StringRepresentable + PartialEq,
{
    pub fn from_values(values: Vec<T>) -> Result<Self, String> {
        let lookup = create_name_lookup(values.clone())?;
        Ok(Self { values, lookup })
    }

    pub fn encode_json(&self, value: &T) -> serde_json::Value {
        serde_json::Value::String(value.get_serialized_name().to_string())
    }

    pub fn decode_json(&self, value: &serde_json::Value) -> Result<T, String> {
        if let Some(name) = value.as_str() {
            return self
                .lookup
                .get(name)
                .cloned()
                .ok_or_else(|| format!("unknown serialized name: {name}"));
        }
        if let Some(id) = value.as_i64() {
            return self
                .values
                .get(id as usize)
                .cloned()
                .ok_or_else(|| format!("unknown id: {id}"));
        }
        Err("string representable value must be a string or integer".to_string())
    }
}

#[derive(Debug, Clone)]
pub struct EnumCodec<E> {
    codec: StringRepresentableCodec<E>,
}

impl<E> EnumCodec<E>
where
    E: Clone + StringRepresentable + PartialEq,
{
    pub fn from_values(values: Vec<E>) -> Result<Self, String> {
        Ok(Self {
            codec: StringRepresentableCodec::from_values(values)?,
        })
    }

    pub fn by_name(&self, name: &str) -> Option<E> {
        self.codec.lookup.get(name).cloned()
    }

    pub fn by_name_or(&self, name: &str, default: E) -> E {
        self.by_name(name).unwrap_or(default)
    }

    pub fn encode_json(&self, value: &E) -> serde_json::Value {
        self.codec.encode_json(value)
    }

    pub fn decode_json(&self, value: &serde_json::Value) -> Result<E, String> {
        self.codec.decode_json(value)
    }
}

#[cfg(test)]
mod tests {
    use super::{create_name_lookup, keys, EnumCodec, StringRepresentable, PRE_BUILT_MAP_THRESHOLD};

    #[derive(Debug, Clone, PartialEq, Eq)]
    enum Example {
        First,
        Second,
        Third,
    }

    impl StringRepresentable for Example {
        fn get_serialized_name(&self) -> &str {
            match self {
                Self::First => "first",
                Self::Second => "second",
                Self::Third => "third",
            }
        }
    }

    #[cfg(vibecraft_has_decompiled_sources)]
    #[test]
    fn string_representable_matches_java_source() {
        const JAVA: &str = vibecraft_java_source!("/net/minecraft/util/StringRepresentable.java");
        assert_eq!(JAVA.lines().count(), 110);
        for fragment in [
            "int PRE_BUILT_MAP_THRESHOLD = 16",
            "String getSerializedName()",
            "fromEnumWithMapping",
            "static <T extends StringRepresentable> Codec<T> fromValues",
            "createNameLookup",
            "if (valueArray.length > 16)",
            "static Keyable keys",
            "public @Nullable E byName(final String name)",
            "Objects.requireNonNullElse",
        ] {
            assert!(JAVA.contains(fragment), "missing StringRepresentable source fragment: {fragment}");
        }
    }

    #[test]
    fn string_representable_lookup_keys_and_codec_match_vanilla() {
        let values = vec![Example::First, Example::Second, Example::Third];
        let lookup = create_name_lookup(values.clone()).expect("unique names");
        assert_eq!(lookup.get("second"), Some(&Example::Second));
        assert_eq!(lookup.get("missing"), None);
        assert_eq!(keys(&values), ["first", "second", "third"]);
        assert_eq!(PRE_BUILT_MAP_THRESHOLD, 16);

        let codec = EnumCodec::from_values(values).expect("unique names");
        assert_eq!(codec.by_name("third"), Some(Example::Third));
        assert_eq!(codec.by_name_or("missing", Example::First), Example::First);
        assert_eq!(codec.encode_json(&Example::Second), serde_json::json!("second"));
        assert_eq!(codec.decode_json(&serde_json::json!("first")), Ok(Example::First));
        assert_eq!(codec.decode_json(&serde_json::json!(2)), Ok(Example::Third));
        assert!(codec.decode_json(&serde_json::json!(9)).is_err());
    }
}
