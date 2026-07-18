//! Block and sky light pair matching Minecraft's `Brightness` record.

#![allow(dead_code)]

use crate::light_coords_util;

/// Java `ExtraCodecs.intRange(0, 15)` bounds used by `Brightness.CODEC`.
pub const MIN_LIGHT_VALUE: i32 = 0;
pub const MAX_LIGHT_VALUE: i32 = 15;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Brightness {
    block: i32,
    sky: i32,
}

impl Brightness {
    pub const FULL_BRIGHT: Self = Self::new(MAX_LIGHT_VALUE, MAX_LIGHT_VALUE);

    /// Mirrors the Java record constructor. Codec range validation is separate.
    pub const fn new(block: i32, sky: i32) -> Self {
        Self { block, sky }
    }

    pub const fn block(self) -> i32 {
        self.block
    }

    pub const fn sky(self) -> i32 {
        self.sky
    }

    pub const fn pack(self) -> i32 {
        light_coords_util::pack(self.block, self.sky)
    }

    pub const fn unpack(packed: i32) -> Self {
        Self::new(light_coords_util::block(packed), light_coords_util::sky(packed))
    }

    /// Decodes the two integer fields checked by Java's `Brightness.CODEC`.
    pub fn from_codec_values(block: i32, sky: i32) -> Result<Self, String> {
        validate_light_value("block", block)?;
        validate_light_value("sky", sky)?;
        Ok(Self::new(block, sky))
    }

    pub fn encode_json(self) -> serde_json::Value {
        serde_json::json!({ "block": self.block, "sky": self.sky })
    }

    pub fn decode_json(value: &serde_json::Value) -> Result<Self, String> {
        let object = value
            .as_object()
            .ok_or_else(|| "brightness value must be an object".to_string())?;
        let block = object
            .get("block")
            .and_then(serde_json::Value::as_i64)
            .ok_or_else(|| "brightness block must be an integer".to_string())?;
        let sky = object
            .get("sky")
            .and_then(serde_json::Value::as_i64)
            .ok_or_else(|| "brightness sky must be an integer".to_string())?;
        let block = i32::try_from(block).map_err(|_| "brightness block is out of i32 range".to_string())?;
        let sky = i32::try_from(sky).map_err(|_| "brightness sky is out of i32 range".to_string())?;
        Self::from_codec_values(block, sky)
    }
}

fn validate_light_value(field: &str, value: i32) -> Result<(), String> {
    if !(MIN_LIGHT_VALUE..=MAX_LIGHT_VALUE).contains(&value) {
        return Err(format!("brightness {field} must be in range [0, 15]: {value}"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::Brightness;

    #[cfg(vibecraft_has_decompiled_sources)]
    #[test]
    fn brightness_matches_java_record_and_codec_source() {
        const JAVA: &str = vibecraft_java_source!("/net/minecraft/util/Brightness.java");
        assert_eq!(JAVA.lines().count(), 21);
        for fragment in [
            "public record Brightness(int block, int sky)",
            "ExtraCodecs.intRange(0, 15)",
            "LIGHT_VALUE_CODEC.fieldOf(\"block\").forGetter(Brightness::block)",
            "public static final Brightness FULL_BRIGHT = new Brightness(15, 15)",
            "return LightCoordsUtil.pack(this.block, this.sky)",
            "return new Brightness(LightCoordsUtil.block(packed), LightCoordsUtil.sky(packed))",
        ] {
            assert!(JAVA.contains(fragment), "missing Brightness source fragment: {fragment}");
        }
    }

    #[test]
    fn brightness_packs_and_unpacks_with_full_bright_constant() {
        let brightness = Brightness::new(4, 12);
        assert_eq!(brightness.block(), 4);
        assert_eq!(brightness.sky(), 12);
        assert_eq!(brightness.pack(), 12 << 20 | 4 << 4);
        assert_eq!(Brightness::unpack(brightness.pack()), brightness);
        assert_eq!(Brightness::FULL_BRIGHT.pack(), crate::light_coords_util::FULL_BRIGHT);
    }

    #[test]
    fn brightness_codec_validates_both_light_fields() {
        assert_eq!(Brightness::from_codec_values(0, 15), Ok(Brightness::new(0, 15)));
        assert!(Brightness::from_codec_values(-1, 10).is_err());
        assert!(Brightness::from_codec_values(10, 16).is_err());
        let encoded = Brightness::new(3, 9).encode_json();
        assert_eq!(Brightness::decode_json(&encoded), Ok(Brightness::new(3, 9)));
        assert!(Brightness::decode_json(&serde_json::json!({ "block": 3, "sky": 99 })).is_err());
    }
}
