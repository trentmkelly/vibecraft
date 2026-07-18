//! Packed block/sky light coordinates matching Minecraft's `LightCoordsUtil`.

#![allow(dead_code)]

pub const FULL_BRIGHT: i32 = 15_728_880;
pub const FULL_SKY: i32 = 15_728_640;
const MAX_SMOOTH_LIGHT_LEVEL: i32 = 240;

pub const fn pack(block: i32, sky: i32) -> i32 {
    block.wrapping_shl(4) | sky.wrapping_shl(20)
}

pub const fn block(packed: i32) -> i32 {
    (packed >> 4) & 15
}

pub const fn sky(packed: i32) -> i32 {
    (packed >> 20) & 15
}

pub const fn with_block(coords: i32, block: i32) -> i32 {
    (coords & 0x00FF_0000) | block.wrapping_shl(4)
}

pub const fn smooth_pack(block: i32, sky: i32) -> i32 {
    (block & 0xFF) | (sky & 0xFF).wrapping_shl(16)
}

pub const fn smooth_block(packed: i32) -> i32 {
    packed & 0xFF
}

pub const fn smooth_sky(packed: i32) -> i32 {
    (packed >> 16) & 0xFF
}

pub fn add_smooth_block_emission(light_coords: i32, block_light_emission: f32) -> i32 {
    let block_light_emission = clamp_unit(block_light_emission);
    let emitted_block = (clamp_unit(block_light_emission) * 240.0) as i32;
    let block = (smooth_block(light_coords) + emitted_block).min(MAX_SMOOTH_LIGHT_LEVEL);
    smooth_pack(block, smooth_sky(light_coords))
}

pub fn max(coords1: i32, coords2: i32) -> i32 {
    pack(block(coords1).max(block(coords2)), sky(coords1).max(sky(coords2)))
}

pub fn light_coords_with_emission(light_coords: i32, emission: i32) -> i32 {
    if emission == 0 {
        return light_coords;
    }
    pack(block(light_coords).max(emission), sky(light_coords).max(emission))
}

pub fn smooth_blend(mut neighbor1: i32, mut neighbor2: i32, mut neighbor3: i32, center: i32) -> i32 {
    if sky(center) > 2 || block(center) > 2 {
        for neighbor in [&mut neighbor1, &mut neighbor2, &mut neighbor3] {
            if sky(*neighbor) == 0 {
                *neighbor |= center & 0x00FF_0000;
            }
            if block(*neighbor) == 0 {
                *neighbor |= center & 0xFF;
            }
        }
    }
    neighbor1
        .wrapping_add(neighbor2)
        .wrapping_add(neighbor3)
        .wrapping_add(center)
        .wrapping_shr(2)
        & 16_711_935
}

pub fn smooth_weighted_blend(coords: [i32; 4], weights: [f32; 4]) -> i32 {
    let sky_level = coords
        .into_iter()
        .zip(weights)
        .map(|(coord, weight)| smooth_sky(coord) as f32 * weight)
        .sum::<f32>() as i32;
    let block_level = coords
        .into_iter()
        .zip(weights)
        .map(|(coord, weight)| smooth_block(coord) as f32 * weight)
        .sum::<f32>() as i32;
    smooth_pack(block_level, sky_level)
}

#[allow(clippy::manual_clamp)]
fn clamp_unit(value: f32) -> f32 {
    if value < 0.0 {
        0.0
    } else if value > 1.0 {
        1.0
    } else {
        value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(vibecraft_has_decompiled_sources)]
    #[test]
    fn light_coords_util_matches_java_source() {
        const JAVA: &str = vibecraft_java_source!("/net/minecraft/util/LightCoordsUtil.java");
        assert_eq!(JAVA.lines().count(), 105);
        for fragment in [
            "public static final int FULL_BRIGHT = 15728880",
            "public static int pack(final int block, final int sky)",
            "return block << 4 | sky << 20",
            "public static int smoothBlend(int neighbor1, int neighbor2, int neighbor3, final int center)",
            "public static int smoothWeightedBlend(",
            "Mth.clamp(blockLightEmission, 0.0F, 1.0F)",
        ] {
            assert!(JAVA.contains(fragment), "missing LightCoordsUtil source fragment: {fragment}");
        }
    }

    #[test]
    fn light_coords_pack_extract_and_emission_match_java() {
        let packed = pack(7, 13);
        assert_eq!(packed, 13 << 20 | 7 << 4);
        assert_eq!(block(packed), 7);
        assert_eq!(sky(packed), 13);
        assert_eq!(with_block(packed, 2), 13 << 20 | 2 << 4);
        assert_eq!(smooth_pack(200, 17), 17 << 16 | 200);
        assert_eq!(smooth_block(smooth_pack(200, 17)), 200);
        assert_eq!(smooth_sky(smooth_pack(200, 17)), 17);
        assert_eq!(add_smooth_block_emission(smooth_pack(40, 10), 0.5), smooth_pack(160, 10));
        assert_eq!(add_smooth_block_emission(smooth_pack(240, 10), 1.0), smooth_pack(240, 10));
        assert_eq!(max(pack(2, 9), pack(8, 3)), pack(8, 9));
        assert_eq!(light_coords_with_emission(pack(2, 4), 10), pack(10, 10));
    }

    #[test]
    fn smooth_blends_fill_missing_center_channels_and_weight_values() {
        let center = smooth_pack(80, 96);
        let zero = smooth_pack(0, 0);
        assert_eq!(smooth_blend(zero, zero, zero, center), center);
        assert_eq!(
            smooth_weighted_blend(
                [smooth_pack(10, 20), smooth_pack(30, 40), smooth_pack(50, 60), smooth_pack(70, 80)],
                [0.25, 0.25, 0.25, 0.25]
            ),
            smooth_pack(40, 50)
        );
    }
}
