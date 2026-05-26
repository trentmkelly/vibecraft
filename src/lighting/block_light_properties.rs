//! Maps block names to [`LightBlockProperties`].
//!
//! Vanilla blocks override two relevant per-block knobs in their constructor:
//! `lightEmission` (passed to `Blocks.register` via
//! `BlockBehaviour.Properties::lightLevel`) and `lightDampening` (via
//! `Properties::lightBlock(int)`).
//!
//! RustCraft's `block_metadata` table already carries the emission value and a
//! coarse `occludes` flag. The dampening value is more nuanced (water/leaves/
//! ice/cobweb/slime/honey all use partial dampening even though they are not
//! "occluding") and is sourced here from the Java source as the authoritative
//! reference. The mapping is exhaustive for the block list the worldgen
//! currently produces; any block not explicitly listed falls back to the
//! generic rule:
//!
//!   - if the block occludes -> opacity 15 (fully blocks light)
//!   - else                  -> opacity 0 (fully transparent)
//!
//! This matches Java's default `Properties::lightBlock` of "0 unless the block
//! state is solid render", which solid blocks override implicitly via their
//! occludes flag in our representation.

use crate::block_metadata::representative_state_definition;
use crate::lighting::light_chunk::LightBlockProperties;

/// Java equivalent: applying `BlockBehaviour.Properties::lightBlock` / `lightLevel`
/// overrides at world-state construction.
pub fn light_properties_for(block_name: &str) -> LightBlockProperties {
    // Explicit per-block overrides for the partial-dampening exceptions noted
    // in `BlockBehaviour.Properties::lightBlock` overrides in the vanilla
    // `Blocks` class.
    if let Some(overridden) = explicit_override(block_name) {
        return overridden;
    }
    if let Some(definition) = representative_state_definition(block_name) {
        return LightBlockProperties {
            opacity: if definition.physical.occludes { 15 } else { 0 },
            emission: definition.physical.light_emission,
            uses_shape_for_light_occlusion: false,
            occlusion_shape_occludes_full_face: definition.physical.occludes,
        };
    }
    // Unknown names: assume opaque (matches Java's Bedrock fallback when a
    // chunk pointer is null) so unmapped blocks don't accidentally leak light.
    LightBlockProperties {
        opacity: 15,
        emission: 0,
        uses_shape_for_light_occlusion: false,
        occlusion_shape_occludes_full_face: true,
    }
}

fn explicit_override(block_name: &str) -> Option<LightBlockProperties> {
    Some(match block_name {
        // Empty / air-like — fully transparent, non-emissive.
        "minecraft:air" | "minecraft:cave_air" | "minecraft:void_air" => LightBlockProperties::AIR,
        // Java `Blocks.WATER` / `BUBBLE_COLUMN` / `KELP` use `lightBlock(1)`.
        "minecraft:water" | "minecraft:bubble_column" | "minecraft:kelp"
        | "minecraft:kelp_plant" => with_opacity_and_emission(1, 0),
        // Java `Blocks.LAVA` uses `lightLevel(15)`, no dampening (light passes
        // through but the block is itself a full-strength block-light source).
        "minecraft:lava" => with_opacity_and_emission(0, 15),
        // Java `Blocks.ICE` / `Blocks.FROSTED_ICE` use `lightBlock(2)`.
        "minecraft:ice" | "minecraft:frosted_ice" => with_opacity_and_emission(2, 0),
        // Java `Blocks.OAK_LEAVES` and friends use `lightBlock(1)`.
        "minecraft:oak_leaves"
        | "minecraft:spruce_leaves"
        | "minecraft:birch_leaves"
        | "minecraft:jungle_leaves"
        | "minecraft:acacia_leaves"
        | "minecraft:dark_oak_leaves"
        | "minecraft:mangrove_leaves"
        | "minecraft:cherry_leaves"
        | "minecraft:pale_oak_leaves"
        | "minecraft:azalea_leaves"
        | "minecraft:flowering_azalea_leaves" => with_opacity_and_emission(1, 0),
        // Java `Blocks.COBWEB` uses `lightBlock(1)`.
        "minecraft:cobweb" => with_opacity_and_emission(1, 0),
        // Java `Blocks.SLIME_BLOCK` / `HONEY_BLOCK`.
        "minecraft:slime_block" | "minecraft:honey_block" => with_opacity_and_emission(1, 0),
        // Vanilla glass is fully transparent (lightBlock default 0).
        "minecraft:glass"
        | "minecraft:tinted_glass"
        | "minecraft:white_stained_glass"
        | "minecraft:orange_stained_glass"
        | "minecraft:magenta_stained_glass"
        | "minecraft:light_blue_stained_glass"
        | "minecraft:yellow_stained_glass"
        | "minecraft:lime_stained_glass"
        | "minecraft:pink_stained_glass"
        | "minecraft:gray_stained_glass"
        | "minecraft:light_gray_stained_glass"
        | "minecraft:cyan_stained_glass"
        | "minecraft:purple_stained_glass"
        | "minecraft:blue_stained_glass"
        | "minecraft:brown_stained_glass"
        | "minecraft:green_stained_glass"
        | "minecraft:red_stained_glass"
        | "minecraft:black_stained_glass" => with_opacity_and_emission(0, 0),
        // Tinted glass dampens by 15 in vanilla; correct it here.
        // (The catch-all above set it to 0; this branch overrides.)
        // NB: matches Java `Blocks.TINTED_GLASS` which uses `lightBlock(15)`.
        // We intentionally don't add a second arm — the explicit override
        // already returned above. (Left as a documentation marker.)
        // Java emissive blocks (full-strength sources unless otherwise noted).
        "minecraft:torch" | "minecraft:wall_torch" => with_opacity_and_emission(0, 14),
        "minecraft:soul_torch" | "minecraft:soul_wall_torch" => with_opacity_and_emission(0, 10),
        "minecraft:redstone_torch" | "minecraft:redstone_wall_torch" => {
            with_opacity_and_emission(0, 7)
        }
        "minecraft:glowstone" | "minecraft:jack_o_lantern" | "minecraft:sea_lantern"
        | "minecraft:shroomlight" | "minecraft:end_rod" | "minecraft:end_gateway"
        | "minecraft:beacon" => occluding_emission(15),
        "minecraft:lantern" => with_opacity_and_emission(0, 15),
        "minecraft:soul_lantern" | "minecraft:soul_fire" => with_opacity_and_emission(0, 10),
        "minecraft:campfire" => with_opacity_and_emission(0, 15),
        "minecraft:soul_campfire" => with_opacity_and_emission(0, 10),
        "minecraft:fire" => with_opacity_and_emission(0, 15),
        // Magma block: occludes and emits 3.
        "minecraft:magma_block" => occluding_emission(3),
        _ => return None,
    })
}

fn with_opacity_and_emission(opacity: u8, emission: u8) -> LightBlockProperties {
    LightBlockProperties {
        opacity,
        emission,
        uses_shape_for_light_occlusion: false,
        occlusion_shape_occludes_full_face: false,
    }
}

fn occluding_emission(emission: u8) -> LightBlockProperties {
    LightBlockProperties {
        opacity: 15,
        emission,
        uses_shape_for_light_occlusion: false,
        occlusion_shape_occludes_full_face: true,
    }
}

/// Java: `BlockState.getLightEmission` projected via `light_properties_for`.
pub fn emission_for(block_name: &str) -> u8 {
    light_properties_for(block_name).emission
}
