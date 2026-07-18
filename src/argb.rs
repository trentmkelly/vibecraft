//! Packed ARGB colour helpers matching Minecraft's `ARGB` utility.
//!
//! Colours use Java's signed `int` representation.  Bitwise operations make
//! that representation transparent: callers can pass values such as
//! `CommonColors::WHITE` (`-1`) directly, while channel accessors always return
//! their unsigned eight-bit component as an `i32`.
#![allow(dead_code)]

use std::sync::OnceLock;

const LINEAR_CHANNEL_DEPTH: usize = 1024;

fn srgb_to_linear_lookup() -> &'static [i16; 256] {
    static LOOKUP: OnceLock<[i16; 256]> = OnceLock::new();
    LOOKUP.get_or_init(|| {
        std::array::from_fn(|index| {
            java_round(compute_srgb_to_linear(index as f32 / 255.0) * 1023.0) as i16
        })
    })
}

fn linear_to_srgb_lookup() -> &'static [u8; LINEAR_CHANNEL_DEPTH] {
    static LOOKUP: OnceLock<[u8; LINEAR_CHANNEL_DEPTH]> = OnceLock::new();
    LOOKUP.get_or_init(|| {
        std::array::from_fn(|index| {
            java_round(compute_linear_to_srgb(index as f32 / 1023.0) * 255.0) as u8
        })
    })
}

fn compute_srgb_to_linear(value: f32) -> f32 {
    if value >= 0.04045 {
        (((value + 0.055) / 1.055) as f64).powf(2.4) as f32
    } else {
        value / 12.92
    }
}

fn compute_linear_to_srgb(value: f32) -> f32 {
    if value >= 0.0031308 {
        (1.055 * (value as f64).powf(1.0 / 2.4) - 0.055) as f32
    } else {
        12.92 * value
    }
}

fn java_round(value: f32) -> i32 {
    (value + 0.5).floor() as i32
}

/// Converts an sRGB channel to its quantized linear representation.
pub fn srgb_to_linear_channel(srgb: i32) -> f32 {
    srgb_to_linear_lookup()[srgb as usize] as f32 / 1023.0
}

/// Converts a linear channel to an sRGB channel using Java's 1024-entry table.
pub fn linear_to_srgb_channel(linear: f32) -> i32 {
    linear_to_srgb_lookup()[(linear * 1023.0).floor() as usize] as i32
}

pub fn mean_linear(srgb1: i32, srgb2: i32, srgb3: i32, srgb4: i32) -> i32 {
    color(
        (alpha(srgb1) + alpha(srgb2) + alpha(srgb3) + alpha(srgb4)) / 4,
        linear_channel_mean(red(srgb1), red(srgb2), red(srgb3), red(srgb4)),
        linear_channel_mean(
            green(srgb1),
            green(srgb2),
            green(srgb3),
            green(srgb4),
        ),
        linear_channel_mean(blue(srgb1), blue(srgb2), blue(srgb3), blue(srgb4)),
    )
}

fn linear_channel_mean(c1: i32, c2: i32, c3: i32, c4: i32) -> i32 {
    let lookup = srgb_to_linear_lookup();
    let mean = (lookup[c1 as usize] as i32
        + lookup[c2 as usize] as i32
        + lookup[c3 as usize] as i32
        + lookup[c4 as usize] as i32)
        / 4;
    linear_to_srgb_lookup()[mean as usize] as i32
}

pub const fn alpha(color: i32) -> i32 {
    ((color as u32) >> 24) as i32
}

pub const fn red(color: i32) -> i32 {
    ((color as u32 >> 16) & 0xff) as i32
}

pub const fn green(color: i32) -> i32 {
    ((color as u32 >> 8) & 0xff) as i32
}

pub const fn blue(color: i32) -> i32 {
    (color as u32 & 0xff) as i32
}

pub const fn color(alpha: i32, red: i32, green: i32, blue: i32) -> i32 {
    ((alpha & 0xff) << 24) | ((red & 0xff) << 16) | ((green & 0xff) << 8) | (blue & 0xff)
}

pub const fn rgb(red: i32, green: i32, blue: i32) -> i32 {
    color(255, red, green, blue)
}

pub fn color_from_vec3(vec: [f64; 3]) -> i32 {
    rgb(
        as_8_bit_channel(vec[0] as f32),
        as_8_bit_channel(vec[1] as f32),
        as_8_bit_channel(vec[2] as f32),
    )
}

pub fn multiply(lhs: i32, rhs: i32) -> i32 {
    if lhs == -1 {
        rhs
    } else if rhs == -1 {
        lhs
    } else {
        color(
            alpha(lhs) * alpha(rhs) / 255,
            red(lhs) * red(rhs) / 255,
            green(lhs) * green(rhs) / 255,
            blue(lhs) * blue(rhs) / 255,
        )
    }
}

pub fn add_rgb(lhs: i32, rhs: i32) -> i32 {
    color(
        alpha(lhs),
        (red(lhs) + red(rhs)).min(255),
        (green(lhs) + green(rhs)).min(255),
        (blue(lhs) + blue(rhs)).min(255),
    )
}

pub fn subtract_rgb(lhs: i32, rhs: i32) -> i32 {
    color(
        alpha(lhs),
        (red(lhs) - red(rhs)).max(0),
        (green(lhs) - green(rhs)).max(0),
        (blue(lhs) - blue(rhs)).max(0),
    )
}

pub fn multiply_alpha(color: i32, alpha_multiplier: f32) -> i32 {
    if color == 0 || alpha_multiplier <= 0.0 {
        0
    } else if alpha_multiplier >= 1.0 {
        color
    } else {
        with_alpha_from_float(alpha_float(color) * alpha_multiplier, color)
    }
}

pub fn scale_rgb(color: i32, scale: f32) -> i32 {
    scale_rgb_components(color, scale, scale, scale)
}

pub fn scale_rgb_components(argb: i32, scale_red: f32, scale_green: f32, scale_blue: f32) -> i32 {
    color(
        alpha(argb),
        ((red(argb) as f32 * scale_red) as i32).clamp(0, 255),
        ((green(argb) as f32 * scale_green) as i32).clamp(0, 255),
        ((blue(argb) as f32 * scale_blue) as i32).clamp(0, 255),
    )
}

pub fn scale_rgb_int(argb: i32, scale: i32) -> i32 {
    color(
        alpha(argb),
        ((red(argb) as i64 * scale as i64 / 255).clamp(0, 255)) as i32,
        ((green(argb) as i64 * scale as i64 / 255).clamp(0, 255)) as i32,
        ((blue(argb) as i64 * scale as i64 / 255).clamp(0, 255)) as i32,
    )
}

pub fn greyscale(argb: i32) -> i32 {
    let channel =
        (red(argb) as f32 * 0.3 + green(argb) as f32 * 0.59 + blue(argb) as f32 * 0.11) as i32;
    color(alpha(argb), channel, channel, channel)
}

pub fn alpha_blend(destination: i32, source: i32) -> i32 {
    let destination_alpha = alpha(destination);
    let source_alpha = alpha(source);
    if source_alpha == 255 {
        return source;
    }
    if source_alpha == 0 {
        return destination;
    }

    let result_alpha = source_alpha + destination_alpha * (255 - source_alpha) / 255;
    color(
        result_alpha,
        alpha_blend_channel(result_alpha, source_alpha, red(destination), red(source)),
        alpha_blend_channel(result_alpha, source_alpha, green(destination), green(source)),
        alpha_blend_channel(result_alpha, source_alpha, blue(destination), blue(source)),
    )
}

fn alpha_blend_channel(result_alpha: i32, source_alpha: i32, destination: i32, source: i32) -> i32 {
    (source * source_alpha + destination * (result_alpha - source_alpha)) / result_alpha
}

pub fn srgb_lerp(blend: f32, first: i32, second: i32) -> i32 {
    color(
        lerp_int(blend, alpha(first), alpha(second)),
        lerp_int(blend, red(first), red(second)),
        lerp_int(blend, green(first), green(second)),
        lerp_int(blend, blue(first), blue(second)),
    )
}

pub fn linear_lerp(blend: f32, first: i32, second: i32) -> i32 {
    let linear = linear_to_srgb_lookup();
    let srgb = srgb_to_linear_lookup();
    color(
        lerp_int(blend, alpha(first), alpha(second)),
        linear[lerp_int(blend, srgb[red(first) as usize] as i32, srgb[red(second) as usize] as i32) as usize] as i32,
        linear[lerp_int(blend, srgb[green(first) as usize] as i32, srgb[green(second) as usize] as i32) as usize] as i32,
        linear[lerp_int(blend, srgb[blue(first) as usize] as i32, srgb[blue(second) as usize] as i32) as usize] as i32,
    )
}

fn lerp_int(blend: f32, start: i32, end: i32) -> i32 {
    // `Mth.lerpInt` is `p0 + floor(alpha * (p1 - p0))`, rather than a
    // truncation of the finished sum.  The distinction matters for negative
    // interpolation factors and descending channels.
    start + (blend * (end - start) as f32).floor() as i32
}

pub const fn opaque(color: i32) -> i32 { color | 0xff00_0000u32 as i32 }
pub const fn transparent(color: i32) -> i32 { color & 0x00ff_ffff }
pub const fn with_alpha(alpha: i32, rgb: i32) -> i32 { (alpha << 24) | (rgb & 0x00ff_ffff) }
pub const fn with_alpha_from_float(alpha: f32, rgb: i32) -> i32 { ((alpha * 255.0).floor() as i32) << 24 | (rgb & 0x00ff_ffff) }
pub const fn white(alpha: i32) -> i32 { (alpha << 24) | 0x00ff_ffff }
pub const fn white_from_float(alpha: f32) -> i32 { with_alpha_from_float(alpha, 0x00ff_ffff) }
pub const fn black(alpha: i32) -> i32 { alpha << 24 }
pub const fn black_from_float(alpha: f32) -> i32 { ((alpha * 255.0).floor() as i32) << 24 }

pub fn gray(brightness: f32) -> i32 {
    let channel = as_8_bit_channel(brightness);
    rgb(channel, channel, channel)
}

pub fn color_from_float(alpha: f32, red: f32, green: f32, blue: f32) -> i32 {
    color(as_8_bit_channel(alpha), as_8_bit_channel(red), as_8_bit_channel(green), as_8_bit_channel(blue))
}

pub fn vector3f_from_rgb24(color: i32) -> [f32; 3] { [red_float(color), green_float(color), blue_float(color)] }
pub fn vector4f_from_argb32(color: i32) -> [f32; 4] { [red_float(color), green_float(color), blue_float(color), alpha_float(color)] }

pub fn average(lhs: i32, rhs: i32) -> i32 {
    color((alpha(lhs) + alpha(rhs)) / 2, (red(lhs) + red(rhs)) / 2, (green(lhs) + green(rhs)) / 2, (blue(lhs) + blue(rhs)) / 2)
}

pub const fn as_8_bit_channel(value: f32) -> i32 { (value * 255.0).floor() as i32 }
pub const fn alpha_float(color: i32) -> f32 { alpha(color) as f32 / 255.0 }
pub const fn red_float(color: i32) -> f32 { red(color) as f32 / 255.0 }
pub const fn green_float(color: i32) -> f32 { green(color) as f32 / 255.0 }
pub const fn blue_float(color: i32) -> f32 { blue(color) as f32 / 255.0 }
pub const fn to_abgr(color: i32) -> i32 { (color & !0x00ff_00ff) | ((color & 0x00ff_0000) >> 16) | ((color & 0x0000_00ff) << 16) }
pub const fn from_abgr(color: i32) -> i32 { to_abgr(color) }

pub fn set_brightness(argb: i32, brightness: f32) -> i32 {
    let mut red = red(argb);
    let mut green = green(argb);
    let mut blue = blue(argb);
    let alpha = alpha(argb);
    let rgb_max = red.max(green).max(blue);
    let rgb_min = red.min(green).min(blue);
    let range = (rgb_max - rgb_min) as f32;
    let saturation = if rgb_max != 0 { range / rgb_max as f32 } else { 0.0 };
    let mut hue = if saturation == 0.0 { 0.0 } else if red == rgb_max {
        (rgb_max - blue) as f32 / range - (rgb_max - green) as f32 / range
    } else if green == rgb_max {
        2.0 + (rgb_max - red) as f32 / range - (rgb_max - blue) as f32 / range
    } else {
        4.0 + (rgb_max - green) as f32 / range - (rgb_max - red) as f32 / range
    } / 6.0;
    if hue < 0.0 { hue += 1.0; }

    if saturation == 0.0 {
        let channel = java_round(brightness * 255.0);
        return color(alpha, channel, channel, channel);
    }

    let segment = (hue - hue.floor()) * 6.0;
    let offset = segment - segment.floor();
    let primary = brightness * (1.0 - saturation);
    let secondary = brightness * (1.0 - saturation * offset);
    let tertiary = brightness * (1.0 - saturation * (1.0 - offset));
    match segment as i32 {
        0 => { red = java_round(brightness * 255.0); green = java_round(tertiary * 255.0); blue = java_round(primary * 255.0); }
        1 => { red = java_round(secondary * 255.0); green = java_round(brightness * 255.0); blue = java_round(primary * 255.0); }
        2 => { red = java_round(primary * 255.0); green = java_round(brightness * 255.0); blue = java_round(tertiary * 255.0); }
        3 => { red = java_round(primary * 255.0); green = java_round(secondary * 255.0); blue = java_round(brightness * 255.0); }
        4 => { red = java_round(tertiary * 255.0); green = java_round(primary * 255.0); blue = java_round(brightness * 255.0); }
        5 => { red = java_round(brightness * 255.0); green = java_round(primary * 255.0); blue = java_round(secondary * 255.0); }
        _ => {}
    }
    color(alpha, red, green, blue)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(vibecraft_has_decompiled_sources)]
    #[test]
    fn source_matches_java_surface() {
        const JAVA: &str = vibecraft_java_source!("/net/minecraft/util/ARGB.java");
        assert_eq!(JAVA.lines().count(), 335);
        for fragment in ["LINEAR_CHANNEL_DEPTH = 1024", "meanLinear", "alphaBlend", "linearLerp", "setBrightness"] {
            assert!(JAVA.contains(fragment));
        }
    }

    #[test]
    fn channel_packing_and_component_operations_match_java() {
        let packed = color(0x80, 0x12, 0x34, 0x56);
        assert_eq!(packed, 0x8012_3456u32 as i32);
        assert_eq!((alpha(packed), red(packed), green(packed), blue(packed)), (128, 18, 52, 86));
        assert_eq!(multiply(-1, packed), packed);
        assert_eq!(add_rgb(packed, rgb(250, 1, 1)), color(128, 255, 53, 87));
        assert_eq!(subtract_rgb(packed, rgb(20, 60, 100)), color(128, 0, 0, 0));
        assert_eq!(opaque(0x1234), -16_764_364);
        assert_eq!(transparent(packed), 0x0012_3456);
        assert_eq!(to_abgr(packed), 0x8056_3412u32 as i32);
    }

    #[test]
    fn interpolation_and_alpha_blending_follow_java_integer_math() {
        assert_eq!(srgb_lerp(0.5, rgb(0, 0, 0), rgb(255, 255, 255)), rgb(127, 127, 127));
        assert_eq!(alpha_blend(rgb(10, 20, 30), color(0, 99, 88, 77)), rgb(10, 20, 30));
        assert_eq!(alpha_blend(rgb(10, 20, 30), rgb(99, 88, 77)), rgb(99, 88, 77));
        assert_eq!(multiply_alpha(rgb(1, 2, 3), 0.5), color(127, 1, 2, 3));
        assert_eq!(srgb_lerp(-0.5, rgb(10, 10, 10), rgb(11, 11, 11)), rgb(9, 9, 9));
    }

    #[test]
    fn linear_conversion_and_brightness_are_quantized_like_java() {
        assert_eq!(srgb_to_linear_channel(0), 0.0);
        assert_eq!(linear_to_srgb_channel(1.0), 255);
        assert_eq!(mean_linear(rgb(0, 0, 0), rgb(255, 255, 255), rgb(0, 0, 0), rgb(255, 255, 255)), rgb(187, 187, 187));
        assert_eq!(set_brightness(rgb(255, 0, 0), 0.5), rgb(128, 0, 0));
        assert_eq!(set_brightness(rgb(20, 20, 20), 0.5), rgb(128, 128, 128));
    }
}
