use super::*;

#[test]
fn biome_generation_settings_plains_matches_registry_payload() {
    let plains = super::super::biome_generation_settings("plains").unwrap();
    assert_eq!(
        *plains,
        BiomeGenerationSettingsModel {
            biome: "minecraft:plains",
            carvers: &[
                "minecraft:cave",
                "minecraft:cave_extra_underground",
                "minecraft:canyon",
            ],
            feature_steps: super::super::PLAINS_FEATURE_STEPS,
            creature_spawn_probability: 0.1,
            spawn_costs: &[],
            spawners: super::super::PLAINS_SPAWNER_GROUPS,
        }
    );
    assert_eq!(plains.feature_steps.len(), 11);
    assert_eq!(plains.feature_steps[1].len(), 2);
    assert_eq!(plains.feature_steps[6].len(), 29);
    assert_eq!(
        plains.feature_steps[9],
        &[
            "minecraft:glow_lichen",
            "minecraft:patch_tall_grass_2",
            "minecraft:patch_bush",
            "minecraft:trees_plains",
            "minecraft:flower_plains",
            "minecraft:patch_grass_plain",
            "minecraft:brown_mushroom_normal",
            "minecraft:red_mushroom_normal",
            "minecraft:patch_pumpkin",
            "minecraft:patch_sugar_cane",
            "minecraft:patch_firefly_bush_near_water",
        ]
    );
    assert!(super::super::biome_has_placed_feature(
        plains,
        "trees_plains"
    ));
    assert!(super::super::biome_has_placed_feature(
        plains,
        "minecraft:ore_diamond_buried"
    ));
    assert!(!super::super::biome_has_placed_feature(
        plains,
        "trees_jungle"
    ));
    assert_eq!(
        super::super::biome_spawns_for_category(plains, "creature"),
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
                entity_type: "minecraft:horse",
                weight: 5,
                min_count: 2,
                max_count: 6,
            },
            MobSpawnerDataModel {
                entity_type: "minecraft:donkey",
                weight: 1,
                min_count: 1,
                max_count: 3,
            },
        ]
    );
    assert_eq!(
        super::super::biome_spawns_for_category(plains, "underground_water_creature"),
        &[MobSpawnerDataModel {
            entity_type: "minecraft:glow_squid",
            weight: 10,
            min_count: 4,
            max_count: 6,
        }]
    );
    assert!(super::super::biome_spawns_for_category(plains, "water_creature").is_empty());
    assert!(super::super::biome_generation_settings("minecraft:badlands").is_some());
}

#[test]
fn builtin_biome_generation_settings_cover_every_builtin_biome() {
    let mut builtins = crate::biome::BUILTIN_BIOMES
        .iter()
        .map(|biome| biome.id)
        .collect::<Vec<_>>();
    let mut generation_settings = super::super::BUILTIN_BIOME_GENERATION_SETTINGS
        .iter()
        .map(|entry| entry.biome)
        .collect::<Vec<_>>();
    builtins.sort_unstable();
    generation_settings.sort_unstable();
    assert_eq!(builtins, generation_settings);
}

#[test]
fn the_void_biome_generation_settings_match_json_contract() {
    let the_void = super::super::biome_generation_settings("minecraft:the_void").unwrap();
    assert_eq!(
        *the_void,
        BiomeGenerationSettingsModel {
            biome: "minecraft:the_void",
            carvers: &[],
            feature_steps: super::super::THE_VOID_FEATURE_STEPS,
            creature_spawn_probability: 0.1,
            spawn_costs: &[],
            spawners: super::super::THE_VOID_SPAWNER_GROUPS,
        }
    );
    assert_eq!(the_void.feature_steps.len(), 11);
    assert_eq!(
        the_void.feature_steps[10],
        &["minecraft:void_start_platform"]
    );
    assert_eq!(
        super::super::biome_spawns_for_category(the_void, "monster"),
        &[]
    );
}
