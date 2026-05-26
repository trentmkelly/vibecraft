use super::super::*;

pub(super) fn assert_surface_and_cave_payloads() {
    assert_plains_and_forest_payloads();
    assert_forest_variant_payloads();
    assert_river_payloads();
    assert_standard_ocean_payloads();
    assert_temperature_ocean_payloads();
    assert_frozen_ocean_payloads();
    assert_shore_and_mushroom_payloads();
    assert_badlands_payloads();
    assert_meadow_grove_and_pale_garden_payloads();
    assert_cave_biome_payloads();
}

fn assert_plains_and_forest_payloads() {
    let sunflower = super::super::biome_generation_settings("sunflower_plains").unwrap();
    assert_eq!(sunflower.biome, "minecraft:sunflower_plains");
    assert_eq!(sunflower.carvers, super::super::OVERWORLD_COMMON_CARVERS);
    assert!(super::super::biome_has_placed_feature(
        sunflower,
        "minecraft:patch_sunflower"
    ));
    assert!(super::super::biome_has_placed_feature(
        sunflower,
        "minecraft:trees_plains"
    ));
    assert!(!super::super::biome_has_placed_feature(
        sunflower,
        "minecraft:trees_birch_and_oak_leaf_litter"
    ));
    assert_eq!(
        super::super::biome_spawns_for_category(sunflower, "creature"),
        super::super::PLAINS_CREATURE_SPAWNS
    );

    let forest = super::super::biome_generation_settings("minecraft:forest").unwrap();
    assert_eq!(forest.biome, "minecraft:forest");
    assert_eq!(forest.feature_steps.len(), 11);
    assert!(super::super::biome_has_placed_feature(
        forest,
        "minecraft:forest_flowers"
    ));
    assert!(super::super::biome_has_placed_feature(
        forest,
        "minecraft:trees_birch_and_oak_leaf_litter"
    ));
    assert!(!super::super::biome_has_placed_feature(
        forest,
        "minecraft:flower_plains"
    ));
    assert_eq!(
        super::super::biome_spawns_for_category(forest, "creature"),
        &[
            MobSpawnerDataModel {
                entity_type: "minecraft:sheep",
                weight: 12,
                min_count: 4,
                max_count: 4,
            },
            MobSpawnerDataModel {
                entity_type: "minecraft:pig",
                weight: 10,
                min_count: 4,
                max_count: 4,
            },
            MobSpawnerDataModel {
                entity_type: "minecraft:chicken",
                weight: 10,
                min_count: 4,
                max_count: 4,
            },
            MobSpawnerDataModel {
                entity_type: "minecraft:cow",
                weight: 8,
                min_count: 4,
                max_count: 4,
            },
            MobSpawnerDataModel {
                entity_type: "minecraft:wolf",
                weight: 5,
                min_count: 4,
                max_count: 4,
            },
        ]
    );
    assert_eq!(
        super::super::biome_spawns_for_category(forest, "monster")[1],
        MobSpawnerDataModel {
            entity_type: "minecraft:zombie",
            weight: 95,
            min_count: 4,
            max_count: 4,
        }
    );
}

fn assert_forest_variant_payloads() {
    let birch = super::super::biome_generation_settings("birch_forest").unwrap();
    assert_eq!(birch.biome, "minecraft:birch_forest");
    assert!(super::super::biome_has_placed_feature(
        birch,
        "minecraft:trees_birch"
    ));
    assert!(super::super::biome_has_placed_feature(
        birch,
        "minecraft:wildflowers_birch_forest"
    ));
    assert!(!super::super::biome_has_placed_feature(
        birch,
        "minecraft:birch_tall"
    ));
    assert_eq!(
        super::super::biome_spawns_for_category(birch, "creature"),
        super::super::BIRCH_FOREST_CREATURE_SPAWNS
    );
    assert!(!super::super::biome_spawns_for_category(birch, "creature")
        .iter()
        .any(|spawn| spawn.entity_type == "minecraft:wolf"));
    assert_eq!(
        super::super::biome_spawns_for_category(birch, "monster"),
        super::super::FOREST_MONSTER_SPAWNS
    );

    let old_growth_birch =
        super::super::biome_generation_settings("old_growth_birch_forest").unwrap();
    assert_eq!(old_growth_birch.biome, "minecraft:old_growth_birch_forest");
    assert!(super::super::biome_has_placed_feature(
        old_growth_birch,
        "minecraft:birch_tall"
    ));
    assert!(!super::super::biome_has_placed_feature(
        old_growth_birch,
        "minecraft:trees_birch"
    ));
    assert_eq!(
        super::super::biome_spawns_for_category(old_growth_birch, "creature"),
        super::super::BIRCH_FOREST_CREATURE_SPAWNS
    );

    let dark_forest = super::super::biome_generation_settings("dark_forest").unwrap();
    assert_eq!(dark_forest.biome, "minecraft:dark_forest");
    assert!(super::super::biome_has_placed_feature(
        dark_forest,
        "minecraft:dark_forest_vegetation"
    ));
    assert!(super::super::biome_has_placed_feature(
        dark_forest,
        "minecraft:patch_leaf_litter"
    ));
    assert!(!super::super::biome_has_placed_feature(
        dark_forest,
        "minecraft:trees_birch"
    ));
    assert_eq!(
        super::super::biome_spawns_for_category(dark_forest, "creature"),
        super::super::BIRCH_FOREST_CREATURE_SPAWNS
    );

    let flower_forest = super::super::biome_generation_settings("flower_forest").unwrap();
    assert_eq!(flower_forest.biome, "minecraft:flower_forest");
    assert!(super::super::biome_has_placed_feature(
        flower_forest,
        "minecraft:flower_forest_flowers"
    ));
    assert!(super::super::biome_has_placed_feature(
        flower_forest,
        "minecraft:trees_flower_forest"
    ));
    assert!(super::super::biome_has_placed_feature(
        flower_forest,
        "minecraft:patch_grass_badlands"
    ));
    assert!(!super::super::biome_has_placed_feature(
        flower_forest,
        "minecraft:patch_grass_forest"
    ));
    assert_eq!(
        super::super::biome_spawns_for_category(flower_forest, "creature").last(),
        Some(&MobSpawnerDataModel {
            entity_type: "minecraft:rabbit",
            weight: 4,
            min_count: 2,
            max_count: 3,
        })
    );
}

fn assert_river_payloads() {
    let river = super::super::biome_generation_settings("river").unwrap();
    assert_eq!(river.biome, "minecraft:river");
    assert_eq!(river.carvers, super::super::OVERWORLD_COMMON_CARVERS);
    assert_eq!(river.feature_steps.len(), 11);
    assert!(super::super::biome_has_placed_feature(
        river,
        "minecraft:trees_water"
    ));
    assert!(super::super::biome_has_placed_feature(
        river,
        "minecraft:seagrass_river"
    ));
    assert!(!super::super::biome_has_placed_feature(
        river,
        "minecraft:trees_plains"
    ));
    assert!(super::super::biome_spawns_for_category(river, "creature").is_empty());
    assert_eq!(
        super::super::biome_spawns_for_category(river, "monster").last(),
        Some(&MobSpawnerDataModel {
            entity_type: "minecraft:drowned",
            weight: 100,
            min_count: 1,
            max_count: 1,
        })
    );
    assert_eq!(
        super::super::biome_spawns_for_category(river, "water_ambient"),
        super::super::RIVER_WATER_AMBIENT_SPAWNS
    );
    assert_eq!(
        super::super::biome_spawns_for_category(river, "water_creature"),
        super::super::RIVER_WATER_CREATURE_SPAWNS
    );

    let frozen_river = super::super::biome_generation_settings("frozen_river").unwrap();
    assert_eq!(frozen_river.biome, "minecraft:frozen_river");
    assert!(super::super::biome_has_placed_feature(
        frozen_river,
        "minecraft:patch_bush"
    ));
    assert!(!super::super::biome_has_placed_feature(
        frozen_river,
        "minecraft:seagrass_river"
    ));
    assert_eq!(
        super::super::biome_spawns_for_category(frozen_river, "monster").last(),
        Some(&MobSpawnerDataModel {
            entity_type: "minecraft:drowned",
            weight: 1,
            min_count: 1,
            max_count: 1,
        })
    );
    assert_eq!(
        super::super::biome_spawns_for_category(frozen_river, "water_ambient"),
        super::super::FROZEN_RIVER_WATER_AMBIENT_SPAWNS
    );
    assert_eq!(
        super::super::biome_spawns_for_category(frozen_river, "water_creature"),
        super::super::FROZEN_RIVER_WATER_CREATURE_SPAWNS
    );
}

fn assert_standard_ocean_payloads() {
    let ocean = super::super::biome_generation_settings("ocean").unwrap();
    assert_eq!(ocean.biome, "minecraft:ocean");
    assert!(super::super::biome_has_placed_feature(
        ocean,
        "minecraft:seagrass_normal"
    ));
    assert!(super::super::biome_has_placed_feature(
        ocean,
        "minecraft:kelp_cold"
    ));
    assert!(!super::super::biome_has_placed_feature(
        ocean,
        "minecraft:warm_ocean_vegetation"
    ));
    assert_eq!(
        super::super::biome_spawns_for_category(ocean, "monster").last(),
        Some(&MobSpawnerDataModel {
            entity_type: "minecraft:drowned",
            weight: 5,
            min_count: 1,
            max_count: 1,
        })
    );
    assert_eq!(
        super::super::biome_spawns_for_category(ocean, "water_ambient"),
        super::super::OCEAN_WATER_AMBIENT_SPAWNS
    );
    assert_eq!(
        super::super::biome_spawns_for_category(ocean, "water_creature"),
        super::super::OCEAN_WATER_CREATURE_SPAWNS
    );

    let deep_ocean = super::super::biome_generation_settings("deep_ocean").unwrap();
    assert_eq!(deep_ocean.biome, "minecraft:deep_ocean");
    assert!(super::super::biome_has_placed_feature(
        deep_ocean,
        "minecraft:seagrass_deep"
    ));
    assert!(!super::super::biome_has_placed_feature(
        deep_ocean,
        "minecraft:seagrass_normal"
    ));
    assert_eq!(
        super::super::biome_spawns_for_category(deep_ocean, "water_ambient"),
        super::super::OCEAN_WATER_AMBIENT_SPAWNS
    );
    assert_eq!(
        super::super::biome_spawns_for_category(deep_ocean, "water_creature"),
        super::super::OCEAN_WATER_CREATURE_SPAWNS
    );

    let cold_ocean = super::super::biome_generation_settings("cold_ocean").unwrap();
    assert_eq!(cold_ocean.biome, "minecraft:cold_ocean");
    assert!(super::super::biome_has_placed_feature(
        cold_ocean,
        "minecraft:seagrass_cold"
    ));
    assert!(super::super::biome_has_placed_feature(
        cold_ocean,
        "minecraft:kelp_cold"
    ));
    assert!(!super::super::biome_has_placed_feature(
        cold_ocean,
        "minecraft:seagrass_normal"
    ));
    assert_eq!(
        super::super::biome_spawns_for_category(cold_ocean, "water_ambient"),
        super::super::COLD_OCEAN_WATER_AMBIENT_SPAWNS
    );
    assert_eq!(
        super::super::biome_spawns_for_category(cold_ocean, "water_creature"),
        super::super::COLD_OCEAN_WATER_CREATURE_SPAWNS
    );
}

fn assert_temperature_ocean_payloads() {
    let deep_cold_ocean = super::super::biome_generation_settings("deep_cold_ocean").unwrap();
    assert_eq!(deep_cold_ocean.biome, "minecraft:deep_cold_ocean");
    assert!(super::super::biome_has_placed_feature(
        deep_cold_ocean,
        "minecraft:seagrass_deep_cold"
    ));
    assert!(!super::super::biome_has_placed_feature(
        deep_cold_ocean,
        "minecraft:seagrass_cold"
    ));
    assert_eq!(
        super::super::biome_spawns_for_category(deep_cold_ocean, "water_ambient"),
        super::super::COLD_OCEAN_WATER_AMBIENT_SPAWNS
    );

    let lukewarm_ocean = super::super::biome_generation_settings("lukewarm_ocean").unwrap();
    assert_eq!(lukewarm_ocean.biome, "minecraft:lukewarm_ocean");
    assert!(super::super::biome_has_placed_feature(
        lukewarm_ocean,
        "minecraft:seagrass_warm"
    ));
    assert!(super::super::biome_has_placed_feature(
        lukewarm_ocean,
        "minecraft:kelp_warm"
    ));
    assert!(!super::super::biome_has_placed_feature(
        lukewarm_ocean,
        "minecraft:warm_ocean_vegetation"
    ));
    assert_eq!(
        super::super::biome_spawns_for_category(lukewarm_ocean, "water_ambient"),
        super::super::LUKEWARM_OCEAN_WATER_AMBIENT_SPAWNS
    );
    assert_eq!(
        super::super::biome_spawns_for_category(lukewarm_ocean, "water_creature"),
        super::super::LUKEWARM_OCEAN_WATER_CREATURE_SPAWNS
    );

    let deep_lukewarm_ocean =
        super::super::biome_generation_settings("deep_lukewarm_ocean").unwrap();
    assert_eq!(deep_lukewarm_ocean.biome, "minecraft:deep_lukewarm_ocean");
    assert!(super::super::biome_has_placed_feature(
        deep_lukewarm_ocean,
        "minecraft:seagrass_deep_warm"
    ));
    assert!(super::super::biome_has_placed_feature(
        deep_lukewarm_ocean,
        "minecraft:kelp_warm"
    ));
    assert_eq!(
        super::super::biome_spawns_for_category(deep_lukewarm_ocean, "water_ambient"),
        super::super::DEEP_LUKEWARM_OCEAN_WATER_AMBIENT_SPAWNS
    );
    assert_eq!(
        super::super::biome_spawns_for_category(deep_lukewarm_ocean, "water_creature"),
        super::super::DEEP_LUKEWARM_OCEAN_WATER_CREATURE_SPAWNS
    );

    let warm_ocean = super::super::biome_generation_settings("warm_ocean").unwrap();
    assert_eq!(warm_ocean.biome, "minecraft:warm_ocean");
    assert!(super::super::biome_has_placed_feature(
        warm_ocean,
        "minecraft:warm_ocean_vegetation"
    ));
    assert!(super::super::biome_has_placed_feature(
        warm_ocean,
        "minecraft:seagrass_warm"
    ));
    assert!(super::super::biome_has_placed_feature(
        warm_ocean,
        "minecraft:sea_pickle"
    ));
    assert!(!super::super::biome_has_placed_feature(
        warm_ocean,
        "minecraft:kelp_cold"
    ));
    assert_eq!(
        super::super::biome_spawns_for_category(warm_ocean, "monster").first(),
        Some(&MobSpawnerDataModel {
            entity_type: "minecraft:drowned",
            weight: 5,
            min_count: 1,
            max_count: 1,
        })
    );
    assert_eq!(
        super::super::biome_spawns_for_category(warm_ocean, "water_ambient"),
        super::super::WARM_OCEAN_WATER_AMBIENT_SPAWNS
    );
    assert_eq!(
        super::super::biome_spawns_for_category(warm_ocean, "water_creature"),
        super::super::WARM_OCEAN_WATER_CREATURE_SPAWNS
    );
}

fn assert_frozen_ocean_payloads() {
    let deep_frozen_ocean = super::super::biome_generation_settings("deep_frozen_ocean").unwrap();
    assert_eq!(deep_frozen_ocean.biome, "minecraft:deep_frozen_ocean");
    assert!(super::super::biome_has_placed_feature(
        deep_frozen_ocean,
        "minecraft:iceberg_packed"
    ));
    assert!(super::super::biome_has_placed_feature(
        deep_frozen_ocean,
        "minecraft:blue_ice"
    ));
    assert!(!super::super::biome_has_placed_feature(
        deep_frozen_ocean,
        "minecraft:kelp_cold"
    ));
    assert_eq!(
        super::super::biome_spawns_for_category(deep_frozen_ocean, "creature"),
        super::super::DEEP_FROZEN_OCEAN_CREATURE_SPAWNS
    );
    assert_eq!(
        super::super::biome_spawns_for_category(deep_frozen_ocean, "water_ambient"),
        super::super::DEEP_FROZEN_OCEAN_WATER_AMBIENT_SPAWNS
    );

    let frozen_ocean = super::super::biome_generation_settings("frozen_ocean").unwrap();
    assert_eq!(frozen_ocean.biome, "minecraft:frozen_ocean");
    assert_eq!(
        frozen_ocean.feature_steps,
        super::super::DEEP_FROZEN_OCEAN_FEATURE_STEPS
    );
    assert_eq!(
        super::super::biome_spawns_for_category(frozen_ocean, "creature"),
        super::super::DEEP_FROZEN_OCEAN_CREATURE_SPAWNS
    );
}

fn assert_shore_and_mushroom_payloads() {
    assert_beach_payloads();
    assert_stony_shore_payload();
    assert_mushroom_fields_payload();
}

fn assert_beach_payloads() {
    let beach = super::super::biome_generation_settings("minecraft:beach").unwrap();
    assert_eq!(beach.biome, "minecraft:beach");
    assert_eq!(beach.carvers, super::super::OVERWORLD_COMMON_CARVERS);
    assert!(super::super::biome_has_placed_feature(
        beach,
        "minecraft:flower_default"
    ));
    assert!(!super::super::biome_has_placed_feature(
        beach,
        "minecraft:trees_water"
    ));
    assert!(!super::super::biome_has_placed_feature(
        beach,
        "minecraft:seagrass_river"
    ));
    assert_eq!(
        super::super::biome_spawns_for_category(beach, "creature"),
        super::super::BEACH_CREATURE_SPAWNS
    );
    assert_eq!(
        super::super::biome_spawns_for_category(beach, "monster"),
        super::super::FOREST_MONSTER_SPAWNS
    );
    assert!(super::super::biome_spawns_for_category(beach, "water_ambient").is_empty());
    assert!(super::super::biome_spawns_for_category(beach, "water_creature").is_empty());

    let snowy_beach = super::super::biome_generation_settings("snowy_beach").unwrap();
    assert_eq!(snowy_beach.biome, "minecraft:snowy_beach");
    assert!(super::super::biome_has_placed_feature(
        snowy_beach,
        "minecraft:flower_default"
    ));
    assert!(!super::super::biome_has_placed_feature(
        snowy_beach,
        "minecraft:trees_water"
    ));
    assert!(super::super::biome_spawns_for_category(snowy_beach, "creature").is_empty());
    assert_eq!(
        super::super::biome_spawns_for_category(snowy_beach, "monster"),
        super::super::FOREST_MONSTER_SPAWNS
    );
}

fn assert_stony_shore_payload() {
    let stony_shore = super::super::biome_generation_settings("stony_shore").unwrap();
    assert_eq!(stony_shore.biome, "minecraft:stony_shore");
    assert!(super::super::biome_has_placed_feature(
        stony_shore,
        "minecraft:flower_default"
    ));
    assert!(super::super::biome_has_placed_feature(
        stony_shore,
        "minecraft:patch_grass_badlands"
    ));
    assert!(!super::super::biome_has_placed_feature(
        stony_shore,
        "minecraft:trees_water"
    ));
    assert!(super::super::biome_spawns_for_category(stony_shore, "creature").is_empty());
    assert_eq!(
        super::super::biome_spawns_for_category(stony_shore, "monster"),
        super::super::FOREST_MONSTER_SPAWNS
    );
}

fn assert_mushroom_fields_payload() {
    let mushroom_fields = super::super::biome_generation_settings("mushroom_fields").unwrap();
    assert_eq!(mushroom_fields.biome, "minecraft:mushroom_fields");
    assert!(super::super::biome_has_placed_feature(
        mushroom_fields,
        "minecraft:mushroom_island_vegetation"
    ));
    assert!(super::super::biome_has_placed_feature(
        mushroom_fields,
        "minecraft:brown_mushroom_taiga"
    ));
    assert!(!super::super::biome_has_placed_feature(
        mushroom_fields,
        "minecraft:flower_default"
    ));
    assert_eq!(
        super::super::biome_spawns_for_category(mushroom_fields, "creature"),
        super::super::MUSHROOM_FIELDS_CREATURE_SPAWNS
    );
    assert!(super::super::biome_spawns_for_category(mushroom_fields, "monster").is_empty());
}

fn assert_badlands_payloads() {
    let badlands = super::super::biome_generation_settings("badlands").unwrap();
    assert_eq!(badlands.biome, "minecraft:badlands");
    assert_eq!(badlands.creature_spawn_probability, 0.03);
    assert!(super::super::biome_has_placed_feature(
        badlands,
        "minecraft:ore_gold_extra"
    ));
    assert!(super::super::biome_has_placed_feature(
        badlands,
        "minecraft:patch_cactus_decorated"
    ));
    assert!(super::super::biome_has_placed_feature(
        badlands,
        "minecraft:patch_sugar_cane_badlands"
    ));
    assert!(!super::super::biome_has_placed_feature(
        badlands,
        "minecraft:patch_cactus_desert"
    ));
    assert_eq!(
        super::super::biome_spawns_for_category(badlands, "creature"),
        super::super::BADLANDS_CREATURE_SPAWNS
    );
    assert_eq!(
        super::super::biome_spawns_for_category(badlands, "monster"),
        super::super::FOREST_MONSTER_SPAWNS
    );

    let eroded_badlands = super::super::biome_generation_settings("eroded_badlands").unwrap();
    assert_eq!(eroded_badlands.biome, "minecraft:eroded_badlands");
    assert_eq!(
        eroded_badlands.feature_steps,
        super::super::BADLANDS_FEATURE_STEPS
    );
    assert_eq!(eroded_badlands.creature_spawn_probability, 0.03);
    assert_eq!(
        super::super::biome_spawns_for_category(eroded_badlands, "creature"),
        super::super::BADLANDS_CREATURE_SPAWNS
    );

    let wooded_badlands = super::super::biome_generation_settings("wooded_badlands").unwrap();
    assert_eq!(wooded_badlands.biome, "minecraft:wooded_badlands");
    assert_eq!(wooded_badlands.creature_spawn_probability, 0.04);
    assert!(super::super::biome_has_placed_feature(
        wooded_badlands,
        "minecraft:trees_badlands"
    ));
    assert_eq!(
        super::super::biome_spawns_for_category(wooded_badlands, "creature").last(),
        Some(&MobSpawnerDataModel {
            entity_type: "minecraft:wolf",
            weight: 2,
            min_count: 4,
            max_count: 8,
        })
    );
}

fn assert_meadow_grove_and_pale_garden_payloads() {
    let meadow = super::super::biome_generation_settings("meadow").unwrap();
    assert_eq!(meadow.biome, "minecraft:meadow");
    assert!(super::super::biome_has_placed_feature(
        meadow,
        "minecraft:ore_emerald"
    ));
    assert!(super::super::biome_has_placed_feature(
        meadow,
        "minecraft:ore_infested"
    ));
    assert!(super::super::biome_has_placed_feature(
        meadow,
        "minecraft:wildflowers_meadow"
    ));
    assert!(!super::super::biome_has_placed_feature(
        meadow,
        "minecraft:flower_default"
    ));
    assert_eq!(
        super::super::biome_spawns_for_category(meadow, "creature"),
        super::super::MEADOW_CREATURE_SPAWNS
    );

    let cherry_grove = super::super::biome_generation_settings("cherry_grove").unwrap();
    assert_eq!(cherry_grove.biome, "minecraft:cherry_grove");
    assert!(super::super::biome_has_placed_feature(
        cherry_grove,
        "minecraft:trees_cherry"
    ));
    assert!(super::super::biome_has_placed_feature(
        cherry_grove,
        "minecraft:flower_cherry"
    ));
    assert!(super::super::biome_has_placed_feature(
        cherry_grove,
        "minecraft:ore_emerald"
    ));
    assert_eq!(
        super::super::biome_spawns_for_category(cherry_grove, "creature"),
        super::super::CHERRY_GROVE_CREATURE_SPAWNS
    );

    let pale_garden = super::super::biome_generation_settings("pale_garden").unwrap();
    assert_eq!(pale_garden.biome, "minecraft:pale_garden");
    assert!(super::super::biome_has_placed_feature(
        pale_garden,
        "minecraft:pale_garden_vegetation"
    ));
    assert!(super::super::biome_has_placed_feature(
        pale_garden,
        "minecraft:pale_moss_patch"
    ));
    assert!(super::super::biome_has_placed_feature(
        pale_garden,
        "minecraft:flower_pale_garden"
    ));
    assert!(super::super::biome_spawns_for_category(pale_garden, "creature").is_empty());
}

fn assert_cave_biome_payloads() {
    let lush_caves = super::super::biome_generation_settings("lush_caves").unwrap();
    assert_eq!(lush_caves.biome, "minecraft:lush_caves");
    assert!(super::super::biome_has_placed_feature(
        lush_caves,
        "minecraft:ore_clay"
    ));
    assert!(super::super::biome_has_placed_feature(
        lush_caves,
        "minecraft:lush_caves_ceiling_vegetation"
    ));
    assert!(super::super::biome_has_placed_feature(
        lush_caves,
        "minecraft:cave_vines"
    ));
    assert!(!super::super::biome_has_placed_feature(
        lush_caves,
        "minecraft:trees_plains"
    ));
    assert_eq!(
        super::super::biome_spawns_for_category(lush_caves, "axolotls"),
        super::super::LUSH_CAVES_AXOLOTL_SPAWNS
    );
    assert_eq!(
        super::super::biome_spawns_for_category(lush_caves, "water_ambient"),
        super::super::MANGROVE_SWAMP_WATER_AMBIENT_SPAWNS
    );

    let dripstone_caves = super::super::biome_generation_settings("dripstone_caves").unwrap();
    assert_eq!(dripstone_caves.biome, "minecraft:dripstone_caves");
    assert!(super::super::biome_has_placed_feature(
        dripstone_caves,
        "minecraft:large_dripstone"
    ));
    assert!(super::super::biome_has_placed_feature(
        dripstone_caves,
        "minecraft:ore_copper_large"
    ));
    assert!(super::super::biome_has_placed_feature(
        dripstone_caves,
        "minecraft:dripstone_cluster"
    ));
    assert!(!super::super::biome_has_placed_feature(
        dripstone_caves,
        "minecraft:ore_copper"
    ));
    assert_eq!(
        super::super::biome_spawns_for_category(dripstone_caves, "monster").last(),
        Some(&MobSpawnerDataModel {
            entity_type: "minecraft:drowned",
            weight: 95,
            min_count: 4,
            max_count: 4,
        })
    );

    let deep_dark = super::super::biome_generation_settings("deep_dark").unwrap();
    assert_eq!(deep_dark.biome, "minecraft:deep_dark");
    assert!(!super::super::biome_has_placed_feature(
        deep_dark,
        "minecraft:lake_lava_surface"
    ));
    assert!(super::super::biome_has_placed_feature(
        deep_dark,
        "minecraft:sculk_vein"
    ));
    assert!(super::super::biome_has_placed_feature(
        deep_dark,
        "minecraft:sculk_patch_deep_dark"
    ));
    assert!(!super::super::biome_has_placed_feature(
        deep_dark,
        "minecraft:spring_lava"
    ));
    assert!(deep_dark
        .spawners
        .iter()
        .all(|group| group.entries.is_empty()));
}
