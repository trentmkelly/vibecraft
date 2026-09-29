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

/// Every built-in biome generation-settings row (carvers, per-step placed features, creature
/// spawn probability, spawn costs and per-category spawners) must equal the vanilla biome JSON
/// bundled in `vanilla-data` (mirrors `Biome.java`'s `BiomeGenerationSettings` + `MobSpawnSettings`).
#[test]
fn builtin_biome_generation_settings_match_vanilla_biome_json() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("vanilla-data/data/minecraft/worldgen/biome");
    let qualify = |id: &str| {
        if id.contains(':') {
            id.to_owned()
        } else {
            format!("minecraft:{id}")
        }
    };
    let mut checked = 0;
    for entry in std::fs::read_dir(&dir).unwrap() {
        let path = entry.unwrap().path();
        let name = path.file_stem().unwrap().to_str().unwrap().to_owned();
        let json: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        let model = super::super::biome_generation_settings(&name)
            .unwrap_or_else(|| panic!("missing generation settings for {name}"));

        // Carvers may be serialized as a single string or a list.
        let carvers: Vec<String> = match &json["carvers"] {
            serde_json::Value::String(s) => vec![qualify(s)],
            serde_json::Value::Array(a) => a.iter().map(|v| qualify(v.as_str().unwrap())).collect(),
            _ => Vec::new(),
        };
        assert_eq!(model.carvers, carvers, "{name} carvers");

        let mut steps: Vec<Vec<String>> = json["features"]
            .as_array()
            .unwrap()
            .iter()
            .map(|step| {
                step.as_array()
                    .unwrap()
                    .iter()
                    .map(|f| qualify(f.as_str().unwrap()))
                    .collect()
            })
            .collect();
        let mut model_steps: Vec<Vec<String>> = model
            .feature_steps
            .iter()
            .map(|step| step.iter().map(|f| (*f).to_owned()).collect())
            .collect();
        // Trailing empty decoration steps are irrelevant to feature ordering.
        while steps.last().is_some_and(Vec::is_empty) {
            steps.pop();
        }
        while model_steps.last().is_some_and(Vec::is_empty) {
            model_steps.pop();
        }
        assert_eq!(model_steps, steps, "{name} features");

        let probability = json["creature_spawn_probability"].as_f64().unwrap_or(0.1);
        assert!(
            (f64::from(model.creature_spawn_probability) - probability).abs() < 1e-6,
            "{name} creature_spawn_probability"
        );

        let costs = json["spawn_costs"].as_object().cloned().unwrap_or_default();
        assert_eq!(model.spawn_costs.len(), costs.len(), "{name} spawn_costs");
        for cost in model.spawn_costs {
            let expected = &costs[cost.entity_type];
            assert_eq!(
                cost.energy_budget,
                expected["energy_budget"].as_f64().unwrap()
            );
            assert_eq!(cost.charge, expected["charge"].as_f64().unwrap());
        }

        let spawners = json["spawners"].as_object().unwrap();
        for group in model.spawners {
            let expected = spawners[group.category].as_array().unwrap();
            let actual: Vec<(String, i64, i64, i64)> = group
                .entries
                .iter()
                .map(|e| {
                    (
                        e.entity_type.to_owned(),
                        i64::from(e.weight),
                        i64::from(e.min_count),
                        i64::from(e.max_count),
                    )
                })
                .collect();
            let expected: Vec<(String, i64, i64, i64)> = expected
                .iter()
                .map(|e| {
                    (
                        e["type"].as_str().unwrap().to_owned(),
                        e["weight"].as_i64().unwrap(),
                        e["minCount"].as_i64().unwrap(),
                        e["maxCount"].as_i64().unwrap(),
                    )
                })
                .collect();
            assert_eq!(actual, expected, "{name} spawner group {}", group.category);
        }
        checked += 1;
    }
    assert_eq!(checked, 65);
}
