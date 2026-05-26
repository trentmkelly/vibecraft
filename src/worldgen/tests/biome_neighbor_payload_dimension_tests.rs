use super::super::*;

pub(super) fn assert_dimension_and_remaining_overworld_payloads() {
    assert_nether_biome_payloads();
    assert_end_biome_payloads();
    assert_mountain_biome_payloads();
    assert_windswept_biome_payloads();
    assert_desert_and_savanna_payloads();
    assert_taiga_payloads();
    assert_snowy_biome_payloads();
    assert_jungle_payloads();
    assert_swamp_payloads();
}

fn assert_nether_biome_payloads() {
    assert_nether_wastes_payload();
    assert_nether_forest_payloads();
    assert_soul_sand_and_basalt_payloads();
}

fn assert_nether_wastes_payload() {
    let nether_wastes = super::super::biome_generation_settings("nether_wastes").unwrap();
    assert_eq!(nether_wastes.biome, "minecraft:nether_wastes");
    assert_eq!(nether_wastes.carvers, super::super::NETHER_COMMON_CARVERS);
    assert_eq!(nether_wastes.feature_steps.len(), 10);
    assert!(super::super::biome_has_placed_feature(
        nether_wastes,
        "minecraft:patch_soul_fire"
    ));
    assert!(super::super::biome_has_placed_feature(
        nether_wastes,
        "minecraft:brown_mushroom_nether"
    ));
    assert_eq!(
        super::super::biome_spawns_for_category(nether_wastes, "creature"),
        super::super::NETHER_CREATURE_SPAWNS
    );
    assert_eq!(
        super::super::biome_spawns_for_category(nether_wastes, "monster"),
        super::super::NETHER_WASTES_MONSTER_SPAWNS
    );
}

fn assert_nether_forest_payloads() {
    let crimson_forest = super::super::biome_generation_settings("crimson_forest").unwrap();
    assert_eq!(crimson_forest.biome, "minecraft:crimson_forest");
    assert!(super::super::biome_has_placed_feature(
        crimson_forest,
        "minecraft:weeping_vines"
    ));
    assert!(super::super::biome_has_placed_feature(
        crimson_forest,
        "minecraft:crimson_fungi"
    ));
    assert!(!super::super::biome_has_placed_feature(
        crimson_forest,
        "minecraft:patch_soul_fire"
    ));
    assert_eq!(
        super::super::biome_spawns_for_category(crimson_forest, "monster"),
        super::super::CRIMSON_FOREST_MONSTER_SPAWNS
    );

    let warped_forest = super::super::biome_generation_settings("warped_forest").unwrap();
    assert_eq!(warped_forest.biome, "minecraft:warped_forest");
    assert!(super::super::biome_has_placed_feature(
        warped_forest,
        "minecraft:warped_forest_vegetation"
    ));
    assert!(super::super::biome_has_placed_feature(
        warped_forest,
        "minecraft:twisting_vines"
    ));
    assert_eq!(
        warped_forest.spawn_costs,
        super::super::WARPED_FOREST_SPAWN_COSTS
    );
    assert_eq!(
        super::super::biome_spawns_for_category(warped_forest, "monster"),
        super::super::WARPED_FOREST_MONSTER_SPAWNS
    );
}

fn assert_soul_sand_and_basalt_payloads() {
    let soul_sand_valley = super::super::biome_generation_settings("soul_sand_valley").unwrap();
    assert_eq!(soul_sand_valley.biome, "minecraft:soul_sand_valley");
    assert!(super::super::biome_has_placed_feature(
        soul_sand_valley,
        "minecraft:basalt_pillar"
    ));
    assert!(super::super::biome_has_placed_feature(
        soul_sand_valley,
        "minecraft:ore_soul_sand"
    ));
    assert_eq!(
        soul_sand_valley.spawn_costs,
        super::super::SOUL_SAND_VALLEY_SPAWN_COSTS
    );
    assert_eq!(
        super::super::biome_spawns_for_category(soul_sand_valley, "monster")[0],
        MobSpawnerDataModel {
            entity_type: "minecraft:skeleton",
            weight: 20,
            min_count: 5,
            max_count: 5,
        }
    );

    let basalt_deltas = super::super::biome_generation_settings("basalt_deltas").unwrap();
    assert_eq!(basalt_deltas.biome, "minecraft:basalt_deltas");
    assert_eq!(basalt_deltas.feature_steps.len(), 8);
    assert!(super::super::biome_has_placed_feature(
        basalt_deltas,
        "minecraft:delta"
    ));
    assert!(super::super::biome_has_placed_feature(
        basalt_deltas,
        "minecraft:spring_closed_double"
    ));
    assert!(!super::super::biome_has_placed_feature(
        basalt_deltas,
        "minecraft:spring_lava"
    ));
    assert_eq!(
        super::super::biome_spawns_for_category(basalt_deltas, "monster"),
        super::super::BASALT_DELTAS_MONSTER_SPAWNS
    );
}

fn assert_end_biome_payloads() {
    let the_end = super::super::biome_generation_settings("the_end").unwrap();
    assert_eq!(the_end.biome, "minecraft:the_end");
    assert!(the_end.carvers.is_empty());
    assert_eq!(the_end.feature_steps.len(), 11);
    assert!(super::super::biome_has_placed_feature(
        the_end,
        "minecraft:end_spike"
    ));
    assert!(super::super::biome_has_placed_feature(
        the_end,
        "minecraft:end_platform"
    ));
    assert_eq!(
        super::super::biome_spawns_for_category(the_end, "monster"),
        super::super::END_MONSTER_SPAWNS
    );

    let end_highlands = super::super::biome_generation_settings("end_highlands").unwrap();
    assert_eq!(end_highlands.biome, "minecraft:end_highlands");
    assert_eq!(end_highlands.feature_steps.len(), 10);
    assert!(super::super::biome_has_placed_feature(
        end_highlands,
        "minecraft:end_gateway_return"
    ));
    assert!(super::super::biome_has_placed_feature(
        end_highlands,
        "minecraft:chorus_plant"
    ));
    assert!(!super::super::biome_has_placed_feature(
        end_highlands,
        "minecraft:end_platform"
    ));

    let end_midlands = super::super::biome_generation_settings("end_midlands").unwrap();
    assert_eq!(end_midlands.biome, "minecraft:end_midlands");
    assert!(end_midlands.carvers.is_empty());
    assert!(end_midlands.feature_steps.is_empty());
    assert_eq!(
        super::super::biome_spawns_for_category(end_midlands, "monster"),
        super::super::END_MONSTER_SPAWNS
    );

    let small_end_islands = super::super::biome_generation_settings("small_end_islands").unwrap();
    assert_eq!(small_end_islands.biome, "minecraft:small_end_islands");
    assert_eq!(small_end_islands.feature_steps.len(), 1);
    assert!(super::super::biome_has_placed_feature(
        small_end_islands,
        "minecraft:end_island_decorated"
    ));

    let end_barrens = super::super::biome_generation_settings("end_barrens").unwrap();
    assert_eq!(end_barrens.biome, "minecraft:end_barrens");
    assert!(end_barrens.feature_steps.is_empty());
    assert_eq!(
        super::super::biome_spawns_for_category(end_barrens, "monster"),
        super::super::END_MONSTER_SPAWNS
    );
}

fn assert_mountain_biome_payloads() {
    let grove = super::super::biome_generation_settings("grove").unwrap();
    assert_eq!(grove.biome, "minecraft:grove");
    assert!(super::super::biome_has_placed_feature(
        grove,
        "minecraft:spring_lava_frozen"
    ));
    assert!(super::super::biome_has_placed_feature(
        grove,
        "minecraft:trees_grove"
    ));
    assert!(!super::super::biome_has_placed_feature(
        grove,
        "minecraft:patch_grass_meadow"
    ));
    assert_eq!(
        super::super::biome_spawns_for_category(grove, "creature"),
        super::super::GROVE_CREATURE_SPAWNS
    );

    let snowy_slopes = super::super::biome_generation_settings("snowy_slopes").unwrap();
    assert_eq!(snowy_slopes.biome, "minecraft:snowy_slopes");
    assert!(super::super::biome_has_placed_feature(
        snowy_slopes,
        "minecraft:spring_lava_frozen"
    ));
    assert!(super::super::biome_has_placed_feature(
        snowy_slopes,
        "minecraft:patch_pumpkin"
    ));
    assert!(!super::super::biome_has_placed_feature(
        snowy_slopes,
        "minecraft:trees_grove"
    ));
    assert_eq!(
        super::super::biome_spawns_for_category(snowy_slopes, "creature"),
        super::super::SNOWY_SLOPES_CREATURE_SPAWNS
    );

    let frozen_peaks = super::super::biome_generation_settings("frozen_peaks").unwrap();
    assert_eq!(frozen_peaks.biome, "minecraft:frozen_peaks");
    assert!(super::super::biome_has_placed_feature(
        frozen_peaks,
        "minecraft:ore_emerald"
    ));
    assert!(super::super::biome_has_placed_feature(
        frozen_peaks,
        "minecraft:spring_lava_frozen"
    ));
    assert!(!super::super::biome_has_placed_feature(
        frozen_peaks,
        "minecraft:patch_pumpkin"
    ));
    assert_eq!(
        super::super::biome_spawns_for_category(frozen_peaks, "creature"),
        super::super::GOAT_CREATURE_SPAWNS
    );

    let jagged_peaks = super::super::biome_generation_settings("jagged_peaks").unwrap();
    assert_eq!(jagged_peaks.biome, "minecraft:jagged_peaks");
    assert_eq!(
        jagged_peaks.feature_steps,
        super::super::FROZEN_PEAKS_FEATURE_STEPS
    );
    assert_eq!(
        super::super::biome_spawns_for_category(jagged_peaks, "creature"),
        super::super::GOAT_CREATURE_SPAWNS
    );

    let stony_peaks = super::super::biome_generation_settings("stony_peaks").unwrap();
    assert_eq!(stony_peaks.biome, "minecraft:stony_peaks");
    assert!(super::super::biome_has_placed_feature(
        stony_peaks,
        "minecraft:ore_infested"
    ));
    assert!(!super::super::biome_has_placed_feature(
        stony_peaks,
        "minecraft:spring_lava_frozen"
    ));
    assert!(super::super::biome_spawns_for_category(stony_peaks, "creature").is_empty());
}

fn assert_windswept_biome_payloads() {
    let windswept_hills = super::super::biome_generation_settings("windswept_hills").unwrap();
    assert_eq!(windswept_hills.biome, "minecraft:windswept_hills");
    assert!(super::super::biome_has_placed_feature(
        windswept_hills,
        "minecraft:trees_windswept_hills"
    ));
    assert!(super::super::biome_has_placed_feature(
        windswept_hills,
        "minecraft:ore_emerald"
    ));
    assert!(!super::super::biome_has_placed_feature(
        windswept_hills,
        "minecraft:trees_windswept_forest"
    ));
    assert_eq!(
        super::super::biome_spawns_for_category(windswept_hills, "creature").last(),
        Some(&MobSpawnerDataModel {
            entity_type: "minecraft:llama",
            weight: 5,
            min_count: 4,
            max_count: 6,
        })
    );

    let windswept_gravelly =
        super::super::biome_generation_settings("windswept_gravelly_hills").unwrap();
    assert_eq!(
        windswept_gravelly.biome,
        "minecraft:windswept_gravelly_hills"
    );
    assert_eq!(
        windswept_gravelly.feature_steps,
        super::super::WINDSWEPT_HILLS_FEATURE_STEPS
    );

    let windswept_forest = super::super::biome_generation_settings("windswept_forest").unwrap();
    assert_eq!(windswept_forest.biome, "minecraft:windswept_forest");
    assert!(super::super::biome_has_placed_feature(
        windswept_forest,
        "minecraft:trees_windswept_forest"
    ));
    assert!(!super::super::biome_has_placed_feature(
        windswept_forest,
        "minecraft:trees_windswept_hills"
    ));
    assert_eq!(
        super::super::biome_spawns_for_category(windswept_forest, "creature"),
        super::super::WINDSWEPT_CREATURE_SPAWNS
    );
}

fn assert_desert_and_savanna_payloads() {
    assert_desert_payload();
    assert_savanna_payloads();
}

fn assert_desert_payload() {
    let desert = super::super::biome_generation_settings("desert").unwrap();
    assert_eq!(desert.biome, "minecraft:desert");
    assert!(super::super::biome_has_placed_feature(
        desert,
        "minecraft:desert_well"
    ));
    assert!(super::super::biome_has_placed_feature(
        desert,
        "minecraft:fossil_upper"
    ));
    assert!(super::super::biome_has_placed_feature(
        desert,
        "minecraft:patch_cactus_desert"
    ));
    assert!(!super::super::biome_has_placed_feature(
        desert,
        "minecraft:patch_sugar_cane"
    ));
    assert_eq!(
        super::super::biome_spawns_for_category(desert, "creature"),
        super::super::DESERT_CREATURE_SPAWNS
    );
    assert_eq!(
        super::super::biome_spawns_for_category(desert, "monster")[1],
        MobSpawnerDataModel {
            entity_type: "minecraft:zombie",
            weight: 19,
            min_count: 4,
            max_count: 4,
        }
    );
    assert_eq!(
        super::super::biome_spawns_for_category(desert, "monster").last(),
        Some(&MobSpawnerDataModel {
            entity_type: "minecraft:parched",
            weight: 50,
            min_count: 4,
            max_count: 4,
        })
    );
}

fn assert_savanna_payloads() {
    let savanna = super::super::biome_generation_settings("savanna").unwrap();
    assert_eq!(savanna.biome, "minecraft:savanna");
    assert!(super::super::biome_has_placed_feature(
        savanna,
        "minecraft:trees_savanna"
    ));
    assert!(super::super::biome_has_placed_feature(
        savanna,
        "minecraft:flower_warm"
    ));
    assert!(super::super::biome_has_placed_feature(
        savanna,
        "minecraft:patch_grass_savanna"
    ));
    assert!(!super::super::biome_has_placed_feature(
        savanna,
        "minecraft:trees_plains"
    ));
    assert_eq!(
        super::super::biome_spawns_for_category(savanna, "creature").last(),
        Some(&MobSpawnerDataModel {
            entity_type: "minecraft:armadillo",
            weight: 10,
            min_count: 2,
            max_count: 3,
        })
    );
    assert_eq!(
        super::super::biome_spawns_for_category(savanna, "monster"),
        super::super::PLAINS_MONSTER_SPAWNS
    );

    let savanna_plateau = super::super::biome_generation_settings("savanna_plateau").unwrap();
    assert_eq!(savanna_plateau.biome, "minecraft:savanna_plateau");
    assert_eq!(
        savanna_plateau.feature_steps,
        super::super::SAVANNA_FEATURE_STEPS
    );
    assert_eq!(
        super::super::biome_spawns_for_category(savanna_plateau, "creature").last(),
        Some(&MobSpawnerDataModel {
            entity_type: "minecraft:wolf",
            weight: 8,
            min_count: 4,
            max_count: 8,
        })
    );

    let windswept_savanna = super::super::biome_generation_settings("windswept_savanna").unwrap();
    assert_eq!(windswept_savanna.biome, "minecraft:windswept_savanna");
    assert!(super::super::biome_has_placed_feature(
        windswept_savanna,
        "minecraft:trees_windswept_savanna"
    ));
    assert!(super::super::biome_has_placed_feature(
        windswept_savanna,
        "minecraft:patch_grass_normal"
    ));
    assert!(!super::super::biome_has_placed_feature(
        windswept_savanna,
        "minecraft:flower_warm"
    ));
    assert_eq!(
        super::super::biome_spawns_for_category(windswept_savanna, "creature"),
        super::super::SAVANNA_CREATURE_SPAWNS
    );
}

fn assert_taiga_payloads() {
    let taiga = super::super::biome_generation_settings("taiga").unwrap();
    assert_eq!(taiga.biome, "minecraft:taiga");
    assert!(super::super::biome_has_placed_feature(
        taiga,
        "minecraft:patch_large_fern"
    ));
    assert!(super::super::biome_has_placed_feature(
        taiga,
        "minecraft:trees_taiga"
    ));
    assert!(super::super::biome_has_placed_feature(
        taiga,
        "minecraft:patch_berry_common"
    ));
    assert!(super::super::biome_has_placed_feature(
        taiga,
        "minecraft:brown_mushroom_taiga"
    ));
    assert!(!super::super::biome_has_placed_feature(
        taiga,
        "minecraft:trees_savanna"
    ));
    assert_eq!(
        super::super::biome_spawns_for_category(taiga, "creature").last(),
        Some(&MobSpawnerDataModel {
            entity_type: "minecraft:fox",
            weight: 8,
            min_count: 2,
            max_count: 4,
        })
    );
    assert_eq!(
        super::super::biome_spawns_for_category(taiga, "monster"),
        super::super::FOREST_MONSTER_SPAWNS
    );

    let snowy_taiga = super::super::biome_generation_settings("snowy_taiga").unwrap();
    assert_eq!(snowy_taiga.biome, "minecraft:snowy_taiga");
    assert!(super::super::biome_has_placed_feature(
        snowy_taiga,
        "minecraft:patch_berry_rare"
    ));
    assert!(!super::super::biome_has_placed_feature(
        snowy_taiga,
        "minecraft:patch_berry_common"
    ));
    assert_eq!(
        super::super::biome_spawns_for_category(snowy_taiga, "creature"),
        super::super::TAIGA_CREATURE_SPAWNS
    );

    let old_growth_pine = super::super::biome_generation_settings("old_growth_pine_taiga").unwrap();
    assert_eq!(old_growth_pine.biome, "minecraft:old_growth_pine_taiga");
    assert!(super::super::biome_has_placed_feature(
        old_growth_pine,
        "minecraft:forest_rock"
    ));
    assert!(super::super::biome_has_placed_feature(
        old_growth_pine,
        "minecraft:trees_old_growth_pine_taiga"
    ));
    assert!(super::super::biome_has_placed_feature(
        old_growth_pine,
        "minecraft:brown_mushroom_old_growth"
    ));
    assert!(!super::super::biome_has_placed_feature(
        old_growth_pine,
        "minecraft:trees_taiga"
    ));
    assert_eq!(
        super::super::biome_spawns_for_category(old_growth_pine, "monster")[1],
        MobSpawnerDataModel {
            entity_type: "minecraft:zombie",
            weight: 100,
            min_count: 4,
            max_count: 4,
        }
    );
    assert_eq!(
        super::super::biome_spawns_for_category(old_growth_pine, "monster")[2],
        MobSpawnerDataModel {
            entity_type: "minecraft:zombie_villager",
            weight: 25,
            min_count: 1,
            max_count: 1,
        }
    );

    let old_growth_spruce =
        super::super::biome_generation_settings("old_growth_spruce_taiga").unwrap();
    assert_eq!(old_growth_spruce.biome, "minecraft:old_growth_spruce_taiga");
    assert!(super::super::biome_has_placed_feature(
        old_growth_spruce,
        "minecraft:trees_old_growth_spruce_taiga"
    ));
    assert!(!super::super::biome_has_placed_feature(
        old_growth_spruce,
        "minecraft:trees_old_growth_pine_taiga"
    ));
    assert_eq!(
        super::super::biome_spawns_for_category(old_growth_spruce, "monster"),
        super::super::FOREST_MONSTER_SPAWNS
    );
}

fn assert_snowy_biome_payloads() {
    let snowy = super::super::biome_generation_settings("snowy_plains").unwrap();
    assert_eq!(snowy.biome, "minecraft:snowy_plains");
    assert_eq!(snowy.creature_spawn_probability, 0.07);
    assert!(super::super::biome_has_placed_feature(
        snowy,
        "minecraft:trees_snowy"
    ));
    assert!(super::super::biome_has_placed_feature(
        snowy,
        "minecraft:patch_grass_badlands"
    ));
    assert!(!super::super::biome_has_placed_feature(
        snowy,
        "minecraft:trees_taiga"
    ));
    assert_eq!(
        super::super::biome_spawns_for_category(snowy, "creature"),
        super::super::SNOWY_PLAINS_CREATURE_SPAWNS
    );
    assert_eq!(
        super::super::biome_spawns_for_category(snowy, "monster")[4],
        MobSpawnerDataModel {
            entity_type: "minecraft:skeleton",
            weight: 20,
            min_count: 4,
            max_count: 4,
        }
    );
    assert_eq!(
        super::super::biome_spawns_for_category(snowy, "monster").last(),
        Some(&MobSpawnerDataModel {
            entity_type: "minecraft:stray",
            weight: 80,
            min_count: 4,
            max_count: 4,
        })
    );

    let ice_spikes = super::super::biome_generation_settings("ice_spikes").unwrap();
    assert_eq!(ice_spikes.biome, "minecraft:ice_spikes");
    assert_eq!(ice_spikes.creature_spawn_probability, 0.07);
    assert!(super::super::biome_has_placed_feature(
        ice_spikes,
        "minecraft:ice_spike"
    ));
    assert!(super::super::biome_has_placed_feature(
        ice_spikes,
        "minecraft:ice_patch"
    ));
    assert_eq!(
        super::super::biome_spawns_for_category(ice_spikes, "monster"),
        super::super::SNOWY_PLAINS_MONSTER_SPAWNS
    );
}

fn assert_jungle_payloads() {
    assert_dense_jungle_payload();
    assert_sparse_jungle_payload();
    assert_bamboo_jungle_payload();
}

fn assert_dense_jungle_payload() {
    let jungle = super::super::biome_generation_settings("jungle").unwrap();
    assert_eq!(jungle.biome, "minecraft:jungle");
    assert!(super::super::biome_has_placed_feature(
        jungle,
        "minecraft:bamboo_light"
    ));
    assert!(super::super::biome_has_placed_feature(
        jungle,
        "minecraft:trees_jungle"
    ));
    assert!(super::super::biome_has_placed_feature(
        jungle,
        "minecraft:vines"
    ));
    assert!(super::super::biome_has_placed_feature(
        jungle,
        "minecraft:patch_melon"
    ));
    assert!(!super::super::biome_has_placed_feature(
        jungle,
        "minecraft:trees_taiga"
    ));
    assert_eq!(
        super::super::biome_spawns_for_category(jungle, "creature")[4],
        MobSpawnerDataModel {
            entity_type: "minecraft:chicken",
            weight: 10,
            min_count: 4,
            max_count: 4,
        }
    );
    assert_eq!(
        super::super::biome_spawns_for_category(jungle, "creature").last(),
        Some(&MobSpawnerDataModel {
            entity_type: "minecraft:panda",
            weight: 1,
            min_count: 1,
            max_count: 2,
        })
    );
    assert_eq!(
        super::super::biome_spawns_for_category(jungle, "monster").last(),
        Some(&MobSpawnerDataModel {
            entity_type: "minecraft:ocelot",
            weight: 2,
            min_count: 1,
            max_count: 3,
        })
    );
}

fn assert_sparse_jungle_payload() {
    let sparse_jungle = super::super::biome_generation_settings("sparse_jungle").unwrap();
    assert_eq!(sparse_jungle.biome, "minecraft:sparse_jungle");
    assert!(super::super::biome_has_placed_feature(
        sparse_jungle,
        "minecraft:trees_sparse_jungle"
    ));
    assert!(super::super::biome_has_placed_feature(
        sparse_jungle,
        "minecraft:patch_melon_sparse"
    ));
    assert!(!super::super::biome_has_placed_feature(
        sparse_jungle,
        "minecraft:trees_jungle"
    ));
    assert_eq!(
        super::super::biome_spawns_for_category(sparse_jungle, "creature").last(),
        Some(&MobSpawnerDataModel {
            entity_type: "minecraft:wolf",
            weight: 8,
            min_count: 2,
            max_count: 4,
        })
    );
    assert_eq!(
        super::super::biome_spawns_for_category(sparse_jungle, "monster"),
        super::super::FOREST_MONSTER_SPAWNS
    );
}

fn assert_bamboo_jungle_payload() {
    let bamboo_jungle = super::super::biome_generation_settings("bamboo_jungle").unwrap();
    assert_eq!(bamboo_jungle.biome, "minecraft:bamboo_jungle");
    assert!(super::super::biome_has_placed_feature(
        bamboo_jungle,
        "minecraft:bamboo"
    ));
    assert!(super::super::biome_has_placed_feature(
        bamboo_jungle,
        "minecraft:bamboo_vegetation"
    ));
    assert!(!super::super::biome_has_placed_feature(
        bamboo_jungle,
        "minecraft:bamboo_light"
    ));
    assert_eq!(
        super::super::biome_spawns_for_category(bamboo_jungle, "creature").last(),
        Some(&MobSpawnerDataModel {
            entity_type: "minecraft:panda",
            weight: 80,
            min_count: 1,
            max_count: 2,
        })
    );
    assert_eq!(
        super::super::biome_spawns_for_category(bamboo_jungle, "monster"),
        super::super::JUNGLE_MONSTER_SPAWNS
    );
}

fn assert_swamp_payloads() {
    let swamp = super::super::biome_generation_settings("swamp").unwrap();
    assert_eq!(swamp.biome, "minecraft:swamp");
    assert!(super::super::biome_has_placed_feature(
        swamp,
        "minecraft:fossil_upper"
    ));
    assert!(super::super::biome_has_placed_feature(
        swamp,
        "minecraft:trees_swamp"
    ));
    assert!(super::super::biome_has_placed_feature(
        swamp,
        "minecraft:patch_waterlily"
    ));
    assert!(super::super::biome_has_placed_feature(
        swamp,
        "minecraft:seagrass_swamp"
    ));
    assert!(!super::super::biome_has_placed_feature(
        swamp,
        "minecraft:disk_sand"
    ));
    assert_eq!(
        super::super::biome_spawns_for_category(swamp, "creature").last(),
        Some(&MobSpawnerDataModel {
            entity_type: "minecraft:frog",
            weight: 10,
            min_count: 2,
            max_count: 5,
        })
    );
    assert_eq!(
        super::super::biome_spawns_for_category(swamp, "monster")[3],
        MobSpawnerDataModel {
            entity_type: "minecraft:skeleton",
            weight: 70,
            min_count: 4,
            max_count: 4,
        }
    );
    assert_eq!(
        super::super::biome_spawns_for_category(swamp, "monster")[8],
        MobSpawnerDataModel {
            entity_type: "minecraft:slime",
            weight: 1,
            min_count: 1,
            max_count: 1,
        }
    );
    assert_eq!(
        super::super::biome_spawns_for_category(swamp, "monster").last(),
        Some(&MobSpawnerDataModel {
            entity_type: "minecraft:bogged",
            weight: 30,
            min_count: 4,
            max_count: 4,
        })
    );

    let mangrove = super::super::biome_generation_settings("mangrove_swamp").unwrap();
    assert_eq!(mangrove.biome, "minecraft:mangrove_swamp");
    assert!(super::super::biome_has_placed_feature(
        mangrove,
        "minecraft:fossil_lower"
    ));
    assert!(super::super::biome_has_placed_feature(
        mangrove,
        "minecraft:trees_mangrove"
    ));
    assert!(super::super::biome_has_placed_feature(
        mangrove,
        "minecraft:disk_grass"
    ));
    assert!(super::super::biome_has_placed_feature(
        mangrove,
        "minecraft:seagrass_swamp"
    ));
    assert!(!super::super::biome_has_placed_feature(
        mangrove,
        "minecraft:trees_swamp"
    ));
    assert_eq!(
        super::super::biome_spawns_for_category(mangrove, "creature"),
        super::super::MANGROVE_SWAMP_CREATURE_SPAWNS
    );
    assert_eq!(
        super::super::biome_spawns_for_category(mangrove, "monster"),
        super::super::SWAMP_MONSTER_SPAWNS
    );
    assert_eq!(
        super::super::biome_spawns_for_category(mangrove, "water_ambient"),
        super::super::MANGROVE_SWAMP_WATER_AMBIENT_SPAWNS
    );
}
