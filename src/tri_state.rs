//! Three-valued state matching Minecraft's `TriState`.

#![allow(dead_code)]

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TriState {
    True,
    False,
    Default,
}

impl TriState {
    pub const fn from(value: bool) -> Self {
        if value { Self::True } else { Self::False }
    }

    pub const fn to_boolean(self, default_value: bool) -> bool {
        match self {
            Self::True => true,
            Self::False => false,
            Self::Default => default_value,
        }
    }

    pub const fn serialized_name(self) -> &'static str {
        match self {
            Self::True => "true",
            Self::False => "false",
            Self::Default => "default",
        }
    }

    pub fn encode_json(self) -> serde_json::Value {
        match self {
            Self::True => serde_json::Value::Bool(true),
            Self::False => serde_json::Value::Bool(false),
            Self::Default => serde_json::Value::String("default".to_string()),
        }
    }

    pub fn decode_json(value: &serde_json::Value) -> Result<Self, String> {
        match value {
            serde_json::Value::Bool(value) => Ok(Self::from(*value)),
            serde_json::Value::String(value) if value == "default" => Ok(Self::Default),
            serde_json::Value::String(value) => Err(format!("unknown tri-state value: {value}")),
            _ => Err("tri-state must be a boolean or string".to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::TriState;

    #[cfg(vibecraft_has_decompiled_sources)]
    #[test]
    fn tri_state_matches_java_source() {
        const JAVA: &str = vibecraft_java_source!("/net/minecraft/util/TriState.java");
        assert_eq!(JAVA.lines().count(), 42);
        for fragment in [
            "public enum TriState implements StringRepresentable",
            "TRUE(\"true\")",
            "FALSE(\"false\")",
            "DEFAULT(\"default\")",
            "public static TriState from(final boolean value)",
            "public boolean toBoolean(final boolean defaultValue)",
            "Codec.either(Codec.BOOL, StringRepresentable.fromEnum",
        ] {
            assert!(JAVA.contains(fragment), "missing TriState source fragment: {fragment}");
        }
    }

    #[test]
    fn tri_state_maps_booleans_and_codec_values_like_vanilla() {
        assert_eq!(TriState::from(true), TriState::True);
        assert_eq!(TriState::from(false), TriState::False);
        assert!(TriState::True.to_boolean(false));
        assert!(!TriState::False.to_boolean(true));
        assert!(TriState::Default.to_boolean(true));
        assert!(!TriState::Default.to_boolean(false));
        assert_eq!(TriState::True.serialized_name(), "true");
        assert_eq!(TriState::False.encode_json(), serde_json::json!(false));
        assert_eq!(TriState::decode_json(&serde_json::json!(true)), Ok(TriState::True));
        assert_eq!(TriState::decode_json(&serde_json::json!("default")), Ok(TriState::Default));
        assert!(TriState::decode_json(&serde_json::json!("true")).is_err());
    }
}
