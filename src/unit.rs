//! The singleton unit value and codecs matching Minecraft's `Unit` enum.

#![allow(dead_code)]

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Unit;

impl Unit {
    pub const INSTANCE: Self = Self;
    pub const CODEC: UnitCodec = UnitCodec;
    pub const STREAM_CODEC: UnitStreamCodec = UnitStreamCodec;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UnitCodec;

impl UnitCodec {
    pub fn encode_json(self, _value: Unit) -> serde_json::Value {
        serde_json::json!({})
    }

    pub fn decode_json(self, value: &serde_json::Value) -> Result<Unit, String> {
        if value.is_object() {
            Ok(Unit::INSTANCE)
        } else {
            Err("unit codec expects a JSON object".to_string())
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UnitStreamCodec;

impl UnitStreamCodec {
    pub const fn encode(self, _value: Unit) -> &'static [u8] {
        &[]
    }

    pub const fn decode(self) -> Unit {
        Unit::INSTANCE
    }
}

#[cfg(test)]
mod tests {
    use super::Unit;

    #[cfg(vibecraft_has_decompiled_sources)]
    #[test]
    fn unit_matches_java_source() {
        const JAVA: &str = vibecraft_java_source!("/net/minecraft/util/Unit.java");
        assert_eq!(JAVA.lines().count(), 13);
        for fragment in [
            "public enum Unit",
            "INSTANCE",
            "Codec<Unit> CODEC = MapCodec.unitCodec(INSTANCE)",
            "StreamCodec<ByteBuf, Unit> STREAM_CODEC = StreamCodec.unit(INSTANCE)",
        ] {
            assert!(JAVA.contains(fragment), "missing Unit source fragment: {fragment}");
        }
    }

    #[test]
    fn unit_is_singleton_and_zero_width_in_both_codecs() {
        assert_eq!(Unit::INSTANCE, Unit);
        assert_eq!(Unit::CODEC.encode_json(Unit::INSTANCE), serde_json::json!({}));
        assert_eq!(Unit::CODEC.decode_json(&serde_json::json!({"ignored": true})), Ok(Unit));
        assert!(Unit::CODEC.decode_json(&serde_json::json!(null)).is_err());
        assert!(Unit::STREAM_CODEC.encode(Unit::INSTANCE).is_empty());
        assert_eq!(Unit::STREAM_CODEC.decode(), Unit::INSTANCE);
    }
}
