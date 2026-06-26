use crate::biome::OVERWORLD_MULTI_NOISE_USED_BIOMES;

const DEEP_OCEANS: &[&str] = &[
    "minecraft:deep_frozen_ocean",
    "minecraft:deep_cold_ocean",
    "minecraft:deep_ocean",
    "minecraft:deep_lukewarm_ocean",
];
const OCEANS: &[&str] = &[
    "minecraft:frozen_ocean",
    "minecraft:ocean",
    "minecraft:cold_ocean",
    "minecraft:lukewarm_ocean",
    "minecraft:warm_ocean",
];
const BEACHES: &[&str] = &["minecraft:beach", "minecraft:snowy_beach"];
const RIVERS: &[&str] = &["minecraft:river", "minecraft:frozen_river"];
const MOUNTAINS: &[&str] = &[
    "minecraft:meadow",
    "minecraft:frozen_peaks",
    "minecraft:jagged_peaks",
    "minecraft:stony_peaks",
    "minecraft:snowy_slopes",
    "minecraft:cherry_grove",
];
const BADLANDS: &[&str] = &[
    "minecraft:badlands",
    "minecraft:eroded_badlands",
    "minecraft:wooded_badlands",
];
const HILLS: &[&str] = &[
    "minecraft:windswept_hills",
    "minecraft:windswept_forest",
    "minecraft:windswept_gravelly_hills",
];
const TAIGAS: &[&str] = &[
    "minecraft:taiga",
    "minecraft:snowy_taiga",
    "minecraft:old_growth_pine_taiga",
    "minecraft:old_growth_spruce_taiga",
];
const JUNGLES: &[&str] = &[
    "minecraft:bamboo_jungle",
    "minecraft:jungle",
    "minecraft:sparse_jungle",
];
const FORESTS: &[&str] = &[
    "minecraft:forest",
    "minecraft:flower_forest",
    "minecraft:birch_forest",
    "minecraft:old_growth_birch_forest",
    "minecraft:dark_forest",
    "minecraft:pale_garden",
    "minecraft:grove",
];
const SAVANNAS: &[&str] = &[
    "minecraft:savanna",
    "minecraft:savanna_plateau",
    "minecraft:windswept_savanna",
];
const NETHER_BIOMES: &[&str] = &[
    "minecraft:nether_wastes",
    "minecraft:soul_sand_valley",
    "minecraft:crimson_forest",
    "minecraft:warped_forest",
    "minecraft:basalt_deltas",
];
const END_BIOMES: &[&str] = &[
    "minecraft:the_end",
    "minecraft:end_highlands",
    "minecraft:end_midlands",
    "minecraft:small_end_islands",
    "minecraft:end_barrens",
];

pub fn is_known_biome_tag(tag_id: &str) -> bool {
    matches!(
        tag_id,
        "minecraft:is_deep_ocean"
            | "minecraft:is_ocean"
            | "minecraft:is_beach"
            | "minecraft:is_river"
            | "minecraft:is_mountain"
            | "minecraft:is_badlands"
            | "minecraft:is_hill"
            | "minecraft:is_taiga"
            | "minecraft:is_jungle"
            | "minecraft:is_forest"
            | "minecraft:is_savanna"
            | "minecraft:is_nether"
            | "minecraft:is_overworld"
            | "minecraft:is_end"
            | "minecraft:has_buried_treasure"
            | "minecraft:has_desert_pyramid"
            | "minecraft:has_igloo"
            | "minecraft:has_jungle_temple"
            | "minecraft:has_mineshaft"
            | "minecraft:has_mineshaft_mesa"
            | "minecraft:mineshaft_blocking"
            | "minecraft:has_ocean_monument"
            | "minecraft:required_ocean_monument_surrounding"
            | "minecraft:has_ocean_ruin_cold"
            | "minecraft:has_ocean_ruin_warm"
            | "minecraft:has_pillager_outpost"
            | "minecraft:has_ruined_portal_desert"
            | "minecraft:has_ruined_portal_jungle"
            | "minecraft:has_ruined_portal_ocean"
            | "minecraft:has_ruined_portal_swamp"
            | "minecraft:has_ruined_portal_mountain"
            | "minecraft:has_ruined_portal_standard"
            | "minecraft:has_shipwreck_beached"
            | "minecraft:has_shipwreck"
            | "minecraft:has_swamp_hut"
            | "minecraft:has_village_desert"
            | "minecraft:has_village_plains"
            | "minecraft:has_village_savanna"
            | "minecraft:has_village_snowy"
            | "minecraft:has_village_taiga"
            | "minecraft:has_trail_ruins"
            | "minecraft:has_woodland_mansion"
            | "minecraft:stronghold_biased_to"
            | "minecraft:has_stronghold"
            | "minecraft:has_trial_chambers"
            | "minecraft:has_nether_fortress"
            | "minecraft:has_nether_fossil"
            | "minecraft:has_bastion_remnant"
            | "minecraft:has_ancient_city"
            | "minecraft:has_ruined_portal_nether"
            | "minecraft:has_end_city"
            | "minecraft:produces_corals_from_bonemeal"
            | "minecraft:water_on_map_outlines"
            | "minecraft:without_zombie_sieges"
            | "minecraft:without_wandering_trader_spawns"
            | "minecraft:spawns_cold_variant_frogs"
            | "minecraft:spawns_warm_variant_frogs"
            | "minecraft:spawns_cold_variant_farm_animals"
            | "minecraft:spawns_warm_variant_farm_animals"
            | "minecraft:spawns_gold_rabbits"
            | "minecraft:spawns_white_rabbits"
            | "minecraft:reduced_water_ambient_spawns"
            | "minecraft:allows_tropical_fish_spawns_at_any_height"
            | "minecraft:polar_bears_spawn_on_alternate_blocks"
            | "minecraft:more_frequent_drowned_spawns"
            | "minecraft:allows_surface_slime_spawns"
            | "minecraft:spawns_snow_foxes"
            | "minecraft:spawns_coral_variant_zombie_nautilus"
    )
}

// Keep this table-shaped so each arm can be audited directly against Java's
// BiomeTagsProvider.addTags() declarations.
#[allow(clippy::too_many_lines)]
pub fn biome_in_tag(biome_id: &str, tag_id: &str) -> bool {
    let tag_id = tag_id.strip_prefix('#').unwrap_or(tag_id);
    match tag_id {
        "minecraft:is_deep_ocean" => contains(DEEP_OCEANS, biome_id),
        "minecraft:is_ocean" => contains(DEEP_OCEANS, biome_id) || contains(OCEANS, biome_id),
        "minecraft:is_beach" => contains(BEACHES, biome_id),
        "minecraft:is_river" => contains(RIVERS, biome_id),
        "minecraft:is_mountain" => contains(MOUNTAINS, biome_id),
        "minecraft:is_badlands" => contains(BADLANDS, biome_id),
        "minecraft:is_hill" => contains(HILLS, biome_id),
        "minecraft:is_taiga" => contains(TAIGAS, biome_id),
        "minecraft:is_jungle" => contains(JUNGLES, biome_id),
        "minecraft:is_forest" => contains(FORESTS, biome_id),
        "minecraft:is_savanna" => contains(SAVANNAS, biome_id),
        "minecraft:is_nether" => contains(NETHER_BIOMES, biome_id),
        "minecraft:is_overworld" => contains(OVERWORLD_MULTI_NOISE_USED_BIOMES, biome_id),
        "minecraft:is_end" => contains(END_BIOMES, biome_id),
        "minecraft:has_buried_treasure" | "minecraft:has_shipwreck_beached" => {
            biome_in_tag(biome_id, "minecraft:is_beach")
        }
        "minecraft:has_desert_pyramid"
        | "minecraft:has_ruined_portal_desert"
        | "minecraft:has_village_desert"
        | "minecraft:spawns_gold_rabbits" => biome_id == "minecraft:desert",
        "minecraft:has_igloo" => contains(
            &[
                "minecraft:snowy_taiga",
                "minecraft:snowy_plains",
                "minecraft:snowy_slopes",
            ],
            biome_id,
        ),
        "minecraft:has_jungle_temple" => {
            contains(&["minecraft:bamboo_jungle", "minecraft:jungle"], biome_id)
        }
        "minecraft:has_mineshaft" => {
            any_tag(
                &[
                    "minecraft:is_ocean",
                    "minecraft:is_river",
                    "minecraft:is_beach",
                    "minecraft:is_mountain",
                    "minecraft:is_hill",
                    "minecraft:is_taiga",
                    "minecraft:is_jungle",
                    "minecraft:is_forest",
                ],
                biome_id,
            ) || contains(
                &[
                    "minecraft:stony_shore",
                    "minecraft:mushroom_fields",
                    "minecraft:ice_spikes",
                    "minecraft:windswept_savanna",
                    "minecraft:desert",
                    "minecraft:savanna",
                    "minecraft:snowy_plains",
                    "minecraft:plains",
                    "minecraft:sunflower_plains",
                    "minecraft:swamp",
                    "minecraft:mangrove_swamp",
                    "minecraft:savanna_plateau",
                    "minecraft:dripstone_caves",
                    "minecraft:lush_caves",
                ],
                biome_id,
            )
        }
        "minecraft:has_mineshaft_mesa" => biome_in_tag(biome_id, "minecraft:is_badlands"),
        "minecraft:mineshaft_blocking" | "minecraft:has_ancient_city" => {
            biome_id == "minecraft:deep_dark"
        }
        "minecraft:has_ocean_monument" => biome_in_tag(biome_id, "minecraft:is_deep_ocean"),
        "minecraft:required_ocean_monument_surrounding" => {
            biome_in_tag(biome_id, "minecraft:is_ocean")
                || biome_in_tag(biome_id, "minecraft:is_river")
        }
        "minecraft:has_ocean_ruin_cold" => contains(
            &[
                "minecraft:frozen_ocean",
                "minecraft:cold_ocean",
                "minecraft:ocean",
                "minecraft:deep_frozen_ocean",
                "minecraft:deep_cold_ocean",
                "minecraft:deep_ocean",
            ],
            biome_id,
        ),
        "minecraft:has_ocean_ruin_warm" => contains(
            &[
                "minecraft:lukewarm_ocean",
                "minecraft:warm_ocean",
                "minecraft:deep_lukewarm_ocean",
            ],
            biome_id,
        ),
        "minecraft:has_pillager_outpost" => {
            contains(
                &[
                    "minecraft:desert",
                    "minecraft:plains",
                    "minecraft:savanna",
                    "minecraft:snowy_plains",
                    "minecraft:taiga",
                    "minecraft:grove",
                ],
                biome_id,
            ) || biome_in_tag(biome_id, "minecraft:is_mountain")
        }
        "minecraft:has_ruined_portal_jungle" => biome_in_tag(biome_id, "minecraft:is_jungle"),
        "minecraft:has_ruined_portal_ocean" | "minecraft:has_shipwreck" => {
            biome_in_tag(biome_id, "minecraft:is_ocean")
        }
        "minecraft:has_ruined_portal_swamp" | "minecraft:allows_surface_slime_spawns" => {
            contains(&["minecraft:swamp", "minecraft:mangrove_swamp"], biome_id)
        }
        "minecraft:has_ruined_portal_mountain" => {
            any_tag(
                &[
                    "minecraft:is_badlands",
                    "minecraft:is_hill",
                    "minecraft:is_mountain",
                ],
                biome_id,
            ) || contains(
                &[
                    "minecraft:savanna_plateau",
                    "minecraft:windswept_savanna",
                    "minecraft:stony_shore",
                ],
                biome_id,
            )
        }
        "minecraft:has_ruined_portal_standard" => {
            any_tag(
                &[
                    "minecraft:is_beach",
                    "minecraft:is_river",
                    "minecraft:is_taiga",
                    "minecraft:is_forest",
                ],
                biome_id,
            ) || contains(
                &[
                    "minecraft:mushroom_fields",
                    "minecraft:ice_spikes",
                    "minecraft:dripstone_caves",
                    "minecraft:lush_caves",
                    "minecraft:savanna",
                    "minecraft:snowy_plains",
                    "minecraft:plains",
                    "minecraft:sunflower_plains",
                ],
                biome_id,
            )
        }
        "minecraft:has_swamp_hut" => biome_id == "minecraft:swamp",
        "minecraft:has_village_plains" => {
            contains(&["minecraft:plains", "minecraft:meadow"], biome_id)
        }
        "minecraft:has_village_savanna" => biome_id == "minecraft:savanna",
        "minecraft:has_village_snowy" => biome_id == "minecraft:snowy_plains",
        "minecraft:has_village_taiga" => biome_id == "minecraft:taiga",
        "minecraft:has_trail_ruins" => contains(
            &[
                "minecraft:taiga",
                "minecraft:snowy_taiga",
                "minecraft:old_growth_pine_taiga",
                "minecraft:old_growth_spruce_taiga",
                "minecraft:old_growth_birch_forest",
                "minecraft:jungle",
            ],
            biome_id,
        ),
        "minecraft:has_woodland_mansion" => {
            contains(&["minecraft:dark_forest", "minecraft:pale_garden"], biome_id)
        }
        "minecraft:stronghold_biased_to" => {
            contains(OVERWORLD_MULTI_NOISE_USED_BIOMES, biome_id)
                && !contains(
                    &[
                        "minecraft:deep_frozen_ocean",
                        "minecraft:deep_cold_ocean",
                        "minecraft:deep_ocean",
                        "minecraft:deep_lukewarm_ocean",
                        "minecraft:warm_ocean",
                        "minecraft:frozen_ocean",
                        "minecraft:cold_ocean",
                        "minecraft:ocean",
                        "minecraft:lukewarm_ocean",
                        "minecraft:stony_shore",
                        "minecraft:snowy_beach",
                        "minecraft:beach",
                        "minecraft:swamp",
                        "minecraft:mangrove_swamp",
                        "minecraft:river",
                        "minecraft:frozen_river",
                        "minecraft:deep_dark",
                    ],
                    biome_id,
                )
        }
        "minecraft:has_stronghold" => biome_in_tag(biome_id, "minecraft:is_overworld"),
        "minecraft:has_trial_chambers" => {
            biome_in_tag(biome_id, "minecraft:is_overworld") && biome_id != "minecraft:deep_dark"
        }
        "minecraft:has_nether_fortress" | "minecraft:has_ruined_portal_nether" => {
            biome_in_tag(biome_id, "minecraft:is_nether")
        }
        "minecraft:has_nether_fossil" => biome_id == "minecraft:soul_sand_valley",
        "minecraft:has_bastion_remnant" => contains(
            &[
                "minecraft:crimson_forest",
                "minecraft:nether_wastes",
                "minecraft:soul_sand_valley",
                "minecraft:warped_forest",
            ],
            biome_id,
        ),
        "minecraft:has_end_city" => {
            contains(&["minecraft:end_highlands", "minecraft:end_midlands"], biome_id)
        }
        "minecraft:produces_corals_from_bonemeal"
        | "minecraft:spawns_coral_variant_zombie_nautilus" => biome_id == "minecraft:warm_ocean",
        "minecraft:water_on_map_outlines" => {
            biome_in_tag(biome_id, "minecraft:is_ocean")
                || biome_in_tag(biome_id, "minecraft:is_river")
                || contains(&["minecraft:swamp", "minecraft:mangrove_swamp"], biome_id)
        }
        "minecraft:without_zombie_sieges" => biome_id == "minecraft:mushroom_fields",
        "minecraft:without_wandering_trader_spawns" => biome_id == "minecraft:the_void",
        "minecraft:spawns_cold_variant_frogs" => {
            contains(COLD_VARIANT_BASE, biome_id) || biome_in_tag(biome_id, "minecraft:is_end")
        }
        "minecraft:spawns_warm_variant_frogs" => warm_variant_base_or_nested(biome_id),
        "minecraft:spawns_cold_variant_farm_animals" => {
            contains(COLD_VARIANT_BASE, biome_id)
                || contains(
                    &[
                        "minecraft:cold_ocean",
                        "minecraft:deep_cold_ocean",
                        "minecraft:old_growth_pine_taiga",
                        "minecraft:old_growth_spruce_taiga",
                        "minecraft:taiga",
                        "minecraft:windswept_forest",
                        "minecraft:windswept_gravelly_hills",
                        "minecraft:windswept_hills",
                        "minecraft:stony_peaks",
                    ],
                    biome_id,
                )
                || biome_in_tag(biome_id, "minecraft:is_end")
        }
        "minecraft:spawns_warm_variant_farm_animals" => {
            warm_variant_base_or_nested(biome_id)
                || contains(
                    &["minecraft:deep_lukewarm_ocean", "minecraft:lukewarm_ocean"],
                    biome_id,
                )
        }
        "minecraft:spawns_white_rabbits" | "minecraft:spawns_snow_foxes" => contains(
            &[
                "minecraft:snowy_plains",
                "minecraft:ice_spikes",
                "minecraft:frozen_ocean",
                "minecraft:snowy_taiga",
                "minecraft:frozen_river",
                "minecraft:snowy_beach",
                "minecraft:frozen_peaks",
                "minecraft:jagged_peaks",
                "minecraft:snowy_slopes",
                "minecraft:grove",
            ],
            biome_id,
        ),
        "minecraft:reduced_water_ambient_spawns" | "minecraft:more_frequent_drowned_spawns" => {
            biome_in_tag(biome_id, "minecraft:is_river")
        }
        "minecraft:allows_tropical_fish_spawns_at_any_height" => biome_id == "minecraft:lush_caves",
        "minecraft:polar_bears_spawn_on_alternate_blocks" => {
            contains(&["minecraft:frozen_ocean", "minecraft:deep_frozen_ocean"], biome_id)
        }
        _ => false,
    }
}

const COLD_VARIANT_BASE: &[&str] = &[
    "minecraft:snowy_plains",
    "minecraft:ice_spikes",
    "minecraft:frozen_peaks",
    "minecraft:jagged_peaks",
    "minecraft:snowy_slopes",
    "minecraft:frozen_ocean",
    "minecraft:deep_frozen_ocean",
    "minecraft:grove",
    "minecraft:deep_dark",
    "minecraft:frozen_river",
    "minecraft:snowy_taiga",
    "minecraft:snowy_beach",
];

fn warm_variant_base_or_nested(biome_id: &str) -> bool {
    contains(
        &[
            "minecraft:desert",
            "minecraft:warm_ocean",
            "minecraft:mangrove_swamp",
        ],
        biome_id,
    ) || any_tag(
        &[
            "minecraft:is_jungle",
            "minecraft:is_savanna",
            "minecraft:is_nether",
            "minecraft:is_badlands",
        ],
        biome_id,
    )
}

fn any_tag(tags: &[&str], biome_id: &str) -> bool {
    tags.iter().any(|tag| biome_in_tag(biome_id, tag))
}

fn contains(values: &[&str], biome_id: &str) -> bool {
    values.contains(&biome_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn biome_tags_match_java_provider_sets() {
        assert!(is_known_biome_tag("minecraft:is_forest"));
        assert!(biome_in_tag("minecraft:forest", "minecraft:is_forest"));
        assert!(biome_in_tag("minecraft:plains", "minecraft:is_overworld"));
        assert!(biome_in_tag(
            "minecraft:nether_wastes",
            "minecraft:is_nether"
        ));
        assert!(biome_in_tag(
            "minecraft:end_highlands",
            "minecraft:spawns_cold_variant_frogs"
        ));
        assert!(biome_in_tag(
            "minecraft:warm_ocean",
            "minecraft:spawns_coral_variant_zombie_nautilus"
        ));
        assert!(!biome_in_tag("minecraft:deep_dark", "minecraft:has_trial_chambers"));
        assert!(!is_known_biome_tag("minecraft:not_a_vanilla_biome_tag"));
    }
}
