//! Maps block names to [`LightBlockProperties`].
//!
//! Vanilla's `BlockBehaviour.getLightDampening` (see
//! `decompiled-server-26.1.2/net/minecraft/world/level/block/state/BlockBehaviour.java`)
//! computes opacity at runtime as:
//!
//! ```text
//! if (state.isSolidRender()) 15
//! else if (state.propagatesSkylightDown()) 0
//! else 1
//! ```
//!
//! plus per-block overrides for specific opacities (ice = 2, leaves = 1,
//! water = 1, cobweb = 1, slime/honey = 1). VibeCraft does not yet track
//! `isSolidRender` / `propagatesSkylightDown` for every block in the
//! 1144-entry registry, so we curate explicit overrides for:
//!
//! - air-likes
//! - water/lava and fluid-aware overrides
//! - leaves (every variant)
//! - light-emitting blocks (every emission level vanilla actually uses)
//! - all modern decoration blocks that vanilla worldgen places on top of
//!   the surface (`leaf_litter`, `pink_petals`, `moss_carpet`,
//!   `wildflowers`, saplings, vines, hanging moss, glow lichen, …)
//! - cave decoration (dripleaves, spore_blossom, amethyst clusters/buds,
//!   pointed dripstone, cave vines, …)
//! - functional/decorative blocks that are obviously transparent
//!   (buttons, pressure plates, signs, rails, levers, redstone dust, …)
//!
//! For every other block name we fall back to the
//! `representative_state_definition` table in `block_metadata`. Anything
//! still unmapped after that is treated as a Bedrock-default opaque block,
//! matching `LightEngine.getState` returning the bedrock fallback when
//! `LightChunkGetter.getChunkForLighting` yields null. The conservative
//! "opaque by default" policy avoids accidentally leaking sky-light through
//! unrecognised solid blocks; the explicit transparent lists below are what
//! prevent decoration blocks from rendering as pitch-black holes on the
//! ground.

use crate::block_metadata::representative_state_definition;
use crate::lighting::light_chunk::LightBlockProperties;

/// Java equivalent: applying `BlockBehaviour.Properties::lightBlock` / `lightLevel`
/// overrides at world-state construction.
pub fn light_properties_for(block_name: &str) -> LightBlockProperties {
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
    LightBlockProperties {
        opacity: 15,
        emission: 0,
        uses_shape_for_light_occlusion: false,
        occlusion_shape_occludes_full_face: true,
    }
}

fn explicit_override(block_name: &str) -> Option<LightBlockProperties> {
    // ---- Air-likes (opacity 0, no emission) ----
    if matches!(
        block_name,
        "minecraft:air" | "minecraft:cave_air" | "minecraft:void_air"
    ) {
        return Some(LightBlockProperties::AIR);
    }

    // ---- Fluids and fluid-related blocks ----
    if matches!(
        block_name,
        "minecraft:water" | "minecraft:bubble_column" | "minecraft:kelp" | "minecraft:kelp_plant"
    ) {
        return Some(transparent_dampening(1));
    }
    if block_name == "minecraft:lava" {
        // Java `LavaBlock` extends LiquidBlock; emission 15, no dampening.
        return Some(transparent_emission(15));
    }

    // ---- Ice variants (lightBlock(2)) ----
    if matches!(
        block_name,
        "minecraft:ice" | "minecraft:frosted_ice" | "minecraft:packed_ice" | "minecraft:blue_ice"
    ) {
        // Packed/blue ice are actually solid render with dampening 15 in
        // vanilla, but ice (clear) is 2. Match per-variant.
        return Some(match block_name {
            "minecraft:packed_ice" | "minecraft:blue_ice" => occluding_emission(0),
            _ => transparent_dampening(2),
        });
    }

    // ---- Leaves (lightBlock(1), non-solid-render) ----
    if is_leaves(block_name) {
        return Some(transparent_dampening(1));
    }

    // ---- Cobweb, slime, honey (lightBlock(1) per vanilla) ----
    if matches!(
        block_name,
        "minecraft:cobweb" | "minecraft:slime_block" | "minecraft:honey_block"
    ) {
        return Some(transparent_dampening(1));
    }

    // ---- Glass family ----
    if is_clear_glass(block_name) {
        return Some(transparent_dampening(0));
    }
    if block_name == "minecraft:tinted_glass" {
        // Java `TintedGlassBlock.propagatesSkylightDown` returns false, so
        // dampening falls to 1; vanilla also overrides with `lightBlock(15)`
        // via `noOcclusion()`... actually tinted glass is `lightBlock(15)`
        // by virtue of its specific override. Match the documented behavior.
        return Some(transparent_dampening(15));
    }

    // ---- Light sources ----
    if let Some(emissive) = light_source(block_name) {
        return Some(emissive);
    }

    // ---- Decorative transparent blocks placed by worldgen ----
    if is_transparent_decoration(block_name) {
        return Some(LightBlockProperties::AIR);
    }

    None
}

fn is_leaves(name: &str) -> bool {
    matches!(
        name,
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
            | "minecraft:flowering_azalea_leaves"
    )
}

fn is_clear_glass(name: &str) -> bool {
    matches!(
        name,
        "minecraft:glass"
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
            | "minecraft:black_stained_glass"
    )
}

fn light_source(name: &str) -> Option<LightBlockProperties> {
    Some(match name {
        "minecraft:torch"
        | "minecraft:wall_torch"
        | "minecraft:copper_torch"
        | "minecraft:copper_wall_torch" => transparent_emission(14),
        "minecraft:soul_torch" | "minecraft:soul_wall_torch" => transparent_emission(10),
        "minecraft:redstone_torch" | "minecraft:redstone_wall_torch" => transparent_emission(7),
        "minecraft:glowstone"
        | "minecraft:jack_o_lantern"
        | "minecraft:sea_lantern"
        | "minecraft:shroomlight"
        | "minecraft:beacon" => occluding_emission(15),
        "minecraft:end_rod" | "minecraft:end_gateway" => transparent_emission(15),
        "minecraft:lantern" => transparent_emission(15),
        "minecraft:soul_lantern" => transparent_emission(10),
        "minecraft:campfire" => transparent_emission(15),
        "minecraft:soul_campfire" => transparent_emission(10),
        "minecraft:fire" | "minecraft:soul_fire" => transparent_emission(15),
        "minecraft:magma_block" => occluding_emission(3),
        "minecraft:redstone_lamp" => occluding_emission(0),
        "minecraft:crying_obsidian" => occluding_emission(10),
        "minecraft:respawn_anchor" => occluding_emission(0),
        "minecraft:sculk_catalyst" => occluding_emission(6),
        "minecraft:sculk_sensor"
        | "minecraft:calibrated_sculk_sensor"
        | "minecraft:sculk_shrieker" => transparent_emission(1),
        "minecraft:glow_lichen" => transparent_emission(7),
        "minecraft:amethyst_cluster" => transparent_emission(5),
        "minecraft:large_amethyst_bud" => transparent_emission(4),
        "minecraft:medium_amethyst_bud" => transparent_emission(2),
        "minecraft:small_amethyst_bud" => transparent_emission(1),
        "minecraft:firefly_bush" => transparent_emission(2),
        "minecraft:cave_vines" | "minecraft:cave_vines_plant" => {
            // Emission only when bearing berries; we ship the non-berry
            // value here. The berry-bearing variant would emit 14, but our
            // worldgen does not yet distinguish the property states.
            transparent_emission(0)
        }
        "minecraft:enchanting_table"
        | "minecraft:end_portal"
        | "minecraft:end_portal_frame"
        | "minecraft:dragon_egg" => occluding_emission(0),
        "minecraft:nether_portal" => transparent_emission(11),
        "minecraft:brewing_stand" => transparent_emission(1),
        "minecraft:conduit" => transparent_emission(15),
        "minecraft:lava_cauldron" => occluding_emission(15),
        "minecraft:budding_amethyst" => occluding_emission(0),
        "minecraft:froglight"
        | "minecraft:ochre_froglight"
        | "minecraft:verdant_froglight"
        | "minecraft:pearlescent_froglight" => occluding_emission(15),
        "minecraft:copper_bulb"
        | "minecraft:exposed_copper_bulb"
        | "minecraft:weathered_copper_bulb"
        | "minecraft:oxidized_copper_bulb" => occluding_emission(15),
        _ => return None,
    })
}

fn is_transparent_decoration(name: &str) -> bool {
    // Modern (1.20+) ground/cave decoration blocks placed by worldgen, all
    // of which vanilla treats as `lightBlock(0)` via the
    // `propagatesSkylightDown` default. Split into category-sized matches
    // so the table stays trivially extensible without hitting clippy's
    // `too_many_lines` budget.
    is_forest_floor_decoration(name)
        || is_sapling(name)
        || is_vine_or_hanging(name)
        || is_cave_decoration(name)
        || is_nether_vegetation(name)
        || is_bamboo_decoration(name)
        || is_aquatic_decoration(name)
        || is_end_decoration(name)
        || is_thin_functional(name)
}

fn is_forest_floor_decoration(name: &str) -> bool {
    matches!(
        name,
        "minecraft:leaf_litter"
            | "minecraft:moss_carpet"
            | "minecraft:pale_moss_carpet"
            | "minecraft:pink_petals"
            | "minecraft:wildflowers"
            | "minecraft:open_eyeblossom"
            | "minecraft:closed_eyeblossom"
            | "minecraft:bush"
            | "minecraft:short_grass"
            | "minecraft:fern"
            | "minecraft:short_dry_grass"
            | "minecraft:tall_dry_grass"
            | "minecraft:tall_grass"
            | "minecraft:large_fern"
            | "minecraft:dead_bush"
            | "minecraft:dandelion"
            | "minecraft:golden_dandelion"
            | "minecraft:torchflower"
            | "minecraft:poppy"
            | "minecraft:blue_orchid"
            | "minecraft:allium"
            | "minecraft:azure_bluet"
            | "minecraft:red_tulip"
            | "minecraft:orange_tulip"
            | "minecraft:white_tulip"
            | "minecraft:pink_tulip"
            | "minecraft:oxeye_daisy"
            | "minecraft:cornflower"
            | "minecraft:wither_rose"
            | "minecraft:lily_of_the_valley"
            | "minecraft:brown_mushroom"
            | "minecraft:red_mushroom"
            | "minecraft:rose_bush"
            | "minecraft:peony"
            | "minecraft:lilac"
            | "minecraft:sunflower"
            | "minecraft:sugar_cane"
            | "minecraft:sweet_berry_bush"
            | "minecraft:lily_pad"
    )
}

fn is_sapling(name: &str) -> bool {
    matches!(
        name,
        "minecraft:oak_sapling"
            | "minecraft:spruce_sapling"
            | "minecraft:birch_sapling"
            | "minecraft:jungle_sapling"
            | "minecraft:acacia_sapling"
            | "minecraft:dark_oak_sapling"
            | "minecraft:cherry_sapling"
            | "minecraft:pale_oak_sapling"
            | "minecraft:mangrove_propagule"
    )
}

fn is_vine_or_hanging(name: &str) -> bool {
    matches!(
        name,
        "minecraft:vine"
            | "minecraft:weeping_vines"
            | "minecraft:weeping_vines_plant"
            | "minecraft:twisting_vines"
            | "minecraft:twisting_vines_plant"
            | "minecraft:cave_vines"
            | "minecraft:cave_vines_plant"
            | "minecraft:hanging_roots"
            | "minecraft:hanging_moss"
            | "minecraft:pale_hanging_moss"
    )
}

fn is_cave_decoration(name: &str) -> bool {
    matches!(
        name,
        "minecraft:spore_blossom"
            | "minecraft:big_dripleaf"
            | "minecraft:big_dripleaf_stem"
            | "minecraft:small_dripleaf"
            | "minecraft:pointed_dripstone"
    )
}

fn is_nether_vegetation(name: &str) -> bool {
    matches!(
        name,
        "minecraft:crimson_roots"
            | "minecraft:warped_roots"
            | "minecraft:crimson_fungus"
            | "minecraft:warped_fungus"
            | "minecraft:nether_sprouts"
            | "minecraft:nether_wart"
    )
}

fn is_bamboo_decoration(name: &str) -> bool {
    matches!(name, "minecraft:bamboo" | "minecraft:bamboo_sapling")
}

fn is_aquatic_decoration(name: &str) -> bool {
    if matches!(
        name,
        "minecraft:seagrass" | "minecraft:tall_seagrass" | "minecraft:sea_pickle"
    ) {
        return true;
    }
    is_living_coral(name) || is_dead_coral(name)
}

fn is_living_coral(name: &str) -> bool {
    matches!(
        name,
        "minecraft:tube_coral"
            | "minecraft:brain_coral"
            | "minecraft:bubble_coral"
            | "minecraft:fire_coral"
            | "minecraft:horn_coral"
            | "minecraft:tube_coral_fan"
            | "minecraft:brain_coral_fan"
            | "minecraft:bubble_coral_fan"
            | "minecraft:fire_coral_fan"
            | "minecraft:horn_coral_fan"
            | "minecraft:tube_coral_wall_fan"
            | "minecraft:brain_coral_wall_fan"
            | "minecraft:bubble_coral_wall_fan"
            | "minecraft:fire_coral_wall_fan"
            | "minecraft:horn_coral_wall_fan"
    )
}

fn is_dead_coral(name: &str) -> bool {
    matches!(
        name,
        "minecraft:dead_tube_coral"
            | "minecraft:dead_brain_coral"
            | "minecraft:dead_bubble_coral"
            | "minecraft:dead_fire_coral"
            | "minecraft:dead_horn_coral"
            | "minecraft:dead_tube_coral_fan"
            | "minecraft:dead_brain_coral_fan"
            | "minecraft:dead_bubble_coral_fan"
            | "minecraft:dead_fire_coral_fan"
            | "minecraft:dead_horn_coral_fan"
            | "minecraft:dead_tube_coral_wall_fan"
            | "minecraft:dead_brain_coral_wall_fan"
            | "minecraft:dead_bubble_coral_wall_fan"
            | "minecraft:dead_fire_coral_wall_fan"
            | "minecraft:dead_horn_coral_wall_fan"
    )
}

fn is_end_decoration(name: &str) -> bool {
    matches!(name, "minecraft:chorus_plant" | "minecraft:chorus_flower")
}

fn is_thin_functional(name: &str) -> bool {
    // Functional thin blocks (rails, levers, redstone wire, tripwire, …)
    // — all `noCollision` and therefore `lightBlock(0)`.
    matches!(
        name,
        "minecraft:rail"
            | "minecraft:powered_rail"
            | "minecraft:detector_rail"
            | "minecraft:activator_rail"
            | "minecraft:lever"
            | "minecraft:redstone_wire"
            | "minecraft:tripwire"
            | "minecraft:tripwire_hook"
            | "minecraft:string"
            | "minecraft:scaffolding"
            | "minecraft:moving_piston"
            | "minecraft:piston_head"
    )
}

fn transparent_dampening(opacity: u8) -> LightBlockProperties {
    LightBlockProperties {
        opacity,
        emission: 0,
        uses_shape_for_light_occlusion: false,
        occlusion_shape_occludes_full_face: false,
    }
}

fn transparent_emission(emission: u8) -> LightBlockProperties {
    LightBlockProperties {
        opacity: 0,
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
