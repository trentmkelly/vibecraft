#![allow(dead_code)]

use crate::map_state::DyeColor;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DyedItemColor {
    pub rgb: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DyedItemColorTooltip {
    pub translation_key: &'static str,
    pub argument: Option<String>,
    pub gray: bool,
    pub italic: bool,
}

impl DyedItemColor {
    pub const LEATHER_COLOR_ARGB: u32 = 0xFFA0_6540;

    pub fn new(rgb: u32) -> Self {
        Self {
            rgb: rgb & 0x00FF_FFFF,
        }
    }

    pub fn get_or_default(color: Option<Self>, default_color: u32) -> u32 {
        color
            .map(|color| 0xFF00_0000 | color.rgb)
            .unwrap_or(default_color)
    }

    pub fn apply_dyes(current: Option<Self>, dyes: &[DyeColor]) -> Self {
        let dye_textures = dyes
            .iter()
            .map(|dye| dye.texture_diffuse_rgb())
            .collect::<Vec<_>>();
        Self::new(apply_dye_textures(
            current.map(|color| color.rgb),
            &dye_textures,
        ))
    }

    pub fn tooltip(self, advanced: bool) -> DyedItemColorTooltip {
        if advanced {
            DyedItemColorTooltip {
                translation_key: "item.color",
                argument: Some(format!("#{:06X}", self.rgb)),
                gray: true,
                italic: false,
            }
        } else {
            DyedItemColorTooltip {
                translation_key: "item.dyed",
                argument: None,
                gray: true,
                italic: true,
            }
        }
    }
}

/// `DyedItemColor.applyDyes` — intensity-scaled average over the optional current
/// RGB value and each dye's `DyeColor.getTextureDiffuseColor()` RGB.
pub fn apply_dye_textures(current: Option<u32>, dye_textures: &[u32]) -> u32 {
    let (mut red_total, mut green_total, mut blue_total, mut intensity_total, mut count) =
        (0u32, 0u32, 0u32, 0u32, 0u32);
    let mut accumulate = |rgb: u32| {
        let rgb = rgb & 0x00FF_FFFF;
        let (red, green, blue) = ((rgb >> 16) & 0xFF, (rgb >> 8) & 0xFF, rgb & 0xFF);
        intensity_total += red.max(green).max(blue);
        red_total += red;
        green_total += green;
        blue_total += blue;
        count += 1;
    };
    if let Some(rgb) = current {
        accumulate(rgb);
    }
    for rgb in dye_textures {
        accumulate(*rgb);
    }
    if count == 0 {
        return 0;
    }
    let (mut red, mut green, mut blue) =
        (red_total / count, green_total / count, blue_total / count);
    let average_intensity = intensity_total as f32 / count as f32;
    let result_intensity = red.max(green).max(blue) as f32;
    if result_intensity > 0.0 {
        red = (red as f32 * average_intensity / result_intensity) as u32;
        green = (green as f32 * average_intensity / result_intensity) as u32;
        blue = (blue as f32 * average_intensity / result_intensity) as u32;
    }
    (red << 16) | (green << 8) | blue
}

#[cfg(all(test, vibecraft_has_decompiled_sources))]
mod tests {
    use super::*;

    const DYED_ITEM_COLOR_JAVA: &str = vibecraft_java_source!("/net/minecraft/world/item/component/DyedItemColor.java");

    #[test]
    fn dyed_item_color_defaults_blending_and_tooltips_match_java() {
        for sentinel in [
            "public record DyedItemColor(int rgb) implements TooltipProvider",
            "ExtraCodecs.RGB_COLOR_CODEC.xmap(DyedItemColor::new, DyedItemColor::rgb)",
            "public static final int LEATHER_COLOR = -6265536;",
            "return color != null ? ARGB.opaque(color.rgb()) : defaultColor;",
            "public static DyedItemColor applyDyes(final @Nullable DyedItemColor currentDye, final List<DyeColor> dyes)",
            "Component.translatable(\"item.color\", String.format(Locale.ROOT, \"#%06X\", this.rgb)).withStyle(ChatFormatting.GRAY)",
            "Component.translatable(\"item.dyed\").withStyle(ChatFormatting.GRAY, ChatFormatting.ITALIC)",
        ] {
            assert!(
                DYED_ITEM_COLOR_JAVA.contains(sentinel),
                "missing DyedItemColor sentinel {sentinel}"
            );
        }

        assert_eq!(DyedItemColor::LEATHER_COLOR_ARGB, 0xFFA0_6540);
        assert_eq!(DyedItemColor::new(0x12_3456).rgb, 0x12_3456);
        assert_eq!(DyedItemColor::new(0xFF12_3456).rgb, 0x12_3456);
        assert_eq!(
            DyedItemColor::get_or_default(Some(DyedItemColor::new(0x12_3456)), 0xAABB_CCDD),
            0xFF12_3456
        );
        assert_eq!(
            DyedItemColor::get_or_default(None, DyedItemColor::LEATHER_COLOR_ARGB),
            0xFFA0_6540
        );

        assert_eq!(
            DyedItemColor::apply_dyes(None, &[DyeColor::Red]),
            DyedItemColor::new(11_546_150)
        );
        assert_eq!(
            DyedItemColor::apply_dyes(Some(DyedItemColor::new(0x00_00FF)), &[DyeColor::Red]),
            DyedItemColor::new(0x81_21D7)
        );

        assert_eq!(
            DyedItemColor::new(0x12_3456).tooltip(false),
            DyedItemColorTooltip {
                translation_key: "item.dyed",
                argument: None,
                gray: true,
                italic: true,
            }
        );
        assert_eq!(
            DyedItemColor::new(0x12_3456).tooltip(true),
            DyedItemColorTooltip {
                translation_key: "item.color",
                argument: Some("#123456".to_string()),
                gray: true,
                italic: false,
            }
        );
    }
}
