//! RGBA color value and its string codec, matching Minecraft's `ColorRGBA`.

#![allow(dead_code)]

use std::fmt;

/// A packed RGBA color represented using Java's signed `int` bit pattern.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ColorRgba {
    rgba: i32,
}

impl ColorRgba {
    pub const fn new(rgba: i32) -> Self {
        Self { rgba }
    }

    pub const fn rgba(self) -> i32 {
        self.rgba
    }

    /// Decodes the `ExtraCodecs.STRING_ARGB_COLOR` hexadecimal form.
    pub fn from_hex(value: &str) -> Result<Self, String> {
        let digits = value
            .strip_prefix('#')
            .ok_or_else(|| "Hex color must begin with #".to_string())?;
        if digits.len() != 8 {
            return Err(format!(
                "Hex color is wrong size, expected 8 digits but got {}",
                digits.len()
            ));
        }
        let parsed = u32::from_str_radix(digits, 16)
            .map_err(|_| format!("Invalid color value: {value}"))?;
        Ok(Self::new(parsed as i32))
    }

    /// Encodes the `ExtraCodecs.STRING_ARGB_COLOR` hexadecimal form.
    pub fn to_hex(self) -> String {
        format!("#{:08x}", self.rgba as u32)
    }
}

impl fmt::Display for ColorRgba {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{:08x}", self.rgba as u32)
    }
}

#[cfg(test)]
mod tests {
    use super::ColorRgba;

    #[cfg(vibecraft_has_decompiled_sources)]
    #[test]
    fn color_rgba_matches_java_record_and_codec_source() {
        const JAVA: &str = vibecraft_java_source!("/net/minecraft/util/ColorRGBA.java");
        assert_eq!(JAVA.lines().count(), 13);
        for fragment in [
            "public record ColorRGBA(int rgba)",
            "ExtraCodecs.STRING_ARGB_COLOR.xmap(ColorRGBA::new, ColorRGBA::rgba)",
            "HexFormat.of().toHexDigits(this.rgba, 8)",
        ] {
            assert!(JAVA.contains(fragment), "missing ColorRGBA source fragment: {fragment}");
        }
    }

    #[test]
    fn color_rgba_preserves_signed_bits_and_hex_round_trip() {
        let color = ColorRgba::new(-1);
        assert_eq!(color.rgba(), -1);
        assert_eq!(color.to_string(), "ffffffff");
        assert_eq!(color.to_hex(), "#ffffffff");
        assert_eq!(ColorRgba::from_hex("#ffffffff"), Ok(color));
        assert_eq!(
            ColorRgba::from_hex("#1234"),
            Err("Hex color is wrong size, expected 8 digits but got 4".to_string())
        );
        assert_eq!(
            ColorRgba::from_hex("12345678"),
            Err("Hex color must begin with #".to_string())
        );
    }
}
