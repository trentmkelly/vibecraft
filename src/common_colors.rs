//! Shared packed ARGB color constants matching Minecraft's `CommonColors`.

#![allow(dead_code)]

pub const WHITE: i32 = -1;
pub const BLACK: i32 = -16_777_216;
pub const GRAY: i32 = -8_355_712;
pub const DARK_GRAY: i32 = -12_566_464;
pub const LIGHT_GRAY: i32 = -6_250_336;
pub const LIGHTER_GRAY: i32 = -4_539_718;
pub const RED: i32 = -65_536;
pub const SOFT_RED: i32 = -2_142_128;
pub const GREEN: i32 = -16_711_936;
pub const BLUE: i32 = -16_776_961;
pub const YELLOW: i32 = -256;
pub const SOFT_YELLOW: i32 = -171;
pub const DARK_PURPLE: i32 = -11_534_256;
pub const HIGH_CONTRAST_DIAMOND: i32 = -11_010_079;
pub const COSMOS_PINK: i32 = -13_108;
pub const TEXT_GRAY: i32 = -2_039_584;

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(vibecraft_has_decompiled_sources)]
    #[test]
    fn common_colors_matches_java_source() {
        const JAVA: &str = vibecraft_java_source!("/net/minecraft/util/CommonColors.java");
        assert_eq!(JAVA.lines().count(), 20);
        for fragment in [
            "public static final int WHITE = -1",
            "public static final int BLACK = -16777216",
            "public static final int SOFT_RED = -2142128",
            "public static final int HIGH_CONTRAST_DIAMOND = -11010079",
            "public static final int COSMOS_PINK = -13108",
            "public static final int TEXT_GRAY = -2039584",
        ] {
            assert!(JAVA.contains(fragment), "missing CommonColors source fragment: {fragment}");
        }
    }

    #[test]
    fn common_colors_preserve_java_signed_argb_values() {
        assert_eq!(WHITE, 0xffff_ffffu32 as i32);
        assert_eq!(BLACK, 0xff00_0000u32 as i32);
        assert_eq!(RED, 0xffff_0000u32 as i32);
        assert_eq!(GREEN, 0xff00_ff00u32 as i32);
        assert_eq!(BLUE, 0xff00_00ffu32 as i32);
        assert_eq!(YELLOW, 0xffff_ff00u32 as i32);
        assert_eq!(SOFT_YELLOW, 0xffff_ff55u32 as i32);
        assert_eq!(GRAY, -8_355_712);
        assert_eq!(TEXT_GRAY, -2_039_584);
    }
}
