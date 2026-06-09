use super::{
    biome_source_codec, biome_source_from_stem_id, builtin_biome, checkerboard_biome_source,
    climate_node_distance, climate_point, climate_sampler_sample, climate_target,
    multi_noise_parameter_list_preset, overworld_biome_parameters, parse_biome_json,
    parse_multi_noise_preset_json, quantize_coord, quart_to_block, select_biome_from_source,
    select_climate_biome, select_end_biome, unquantize_coord, BiomeSourceModel, ClimateBiomeEntry,
    ClimateParameter, ClimateParameterList, ClimateSamplerInput, ClimateTarget, MobCategory,
    BUILTIN_BIOMES, NETHER_BIOME_PARAMETERS,
};

const BIOME_DATA_JAVA: &str = include_str!(
    "../../../decompiled-server-26.1.2/net/minecraft/data/worldgen/biome/BiomeData.java"
);
const END_BIOMES_JAVA: &str = include_str!(
    "../../../decompiled-server-26.1.2/net/minecraft/data/worldgen/biome/EndBiomes.java"
);

fn count_occurrences(source: &str, needle: &str) -> usize {
    source.match_indices(needle).count()
}

fn end_biome_json(id: &str) -> super::BiomeData {
    let path = format!(
        "../decompiled-server-26.1.2/data/minecraft/worldgen/biome/{}.json",
        id.trim_start_matches("minecraft:")
    );
    let json =
        std::fs::read_to_string(&path).unwrap_or_else(|err| panic!("failed to read {path}: {err}"));
    parse_biome_json(id, &json).unwrap_or_else(|err| panic!("failed to parse {path}: {err}"))
}

fn padded_feature_steps(steps: &[&[&str]]) -> [Vec<String>; 11] {
    let mut padded: [Vec<String>; 11] = Default::default();
    for (index, step) in steps.iter().take(11).enumerate() {
        padded[index] = step.iter().map(|feature| (*feature).to_string()).collect();
    }
    padded
}

#[test]
fn builtin_biome_registry_keys_match_26_1_2_biomes_order() {
    assert_eq!(BUILTIN_BIOMES.len(), 65);
    assert_eq!(BUILTIN_BIOMES.first().unwrap().id, "minecraft:the_void");
    assert_eq!(BUILTIN_BIOMES[1].id, "minecraft:plains");
    assert_eq!(BUILTIN_BIOMES[12].id, "minecraft:pale_garden");
    assert_eq!(BUILTIN_BIOMES[55].id, "minecraft:nether_wastes");
    assert_eq!(BUILTIN_BIOMES.last().unwrap().id, "minecraft:end_barrens");
    assert_eq!(builtin_biome("plains").unwrap().id, "minecraft:plains");
    assert!(builtin_biome("missing").is_none());
}

#[test]
fn biome_data_java_bootstrap_shape_matches_builtin_biome_registry() {
    assert_eq!(BIOME_DATA_JAVA.lines().count(), 81);
    assert!(BIOME_DATA_JAVA.contains(
        "HolderGetter<PlacedFeature> placedFeatures = context.lookup(Registries.PLACED_FEATURE);"
    ));
    assert!(BIOME_DATA_JAVA.contains("HolderGetter<ConfiguredWorldCarver<?>> carvers = context.lookup(Registries.CONFIGURED_CARVER);"));
    assert_eq!(count_occurrences(BIOME_DATA_JAVA, "context.register("), 65);
    assert_eq!(count_occurrences(BIOME_DATA_JAVA, "OverworldBiomes."), 55);
    assert_eq!(count_occurrences(BIOME_DATA_JAVA, "NetherBiomes."), 5);
    assert_eq!(count_occurrences(BIOME_DATA_JAVA, "EndBiomes."), 5);
    assert_eq!(
        count_occurrences(BIOME_DATA_JAVA, "context.register(Biomes."),
        BUILTIN_BIOMES.len()
    );
    for sentinel in [
        "context.register(Biomes.THE_VOID, OverworldBiomes.theVoid(placedFeatures, carvers));",
        "context.register(Biomes.PALE_GARDEN, OverworldBiomes.darkForest(placedFeatures, carvers, true));",
        "context.register(Biomes.DEEP_LUKEWARM_OCEAN, OverworldBiomes.lukeWarmOcean(placedFeatures, carvers, true));",
        "context.register(Biomes.NETHER_WASTES, NetherBiomes.netherWastes(placedFeatures, carvers));",
        "context.register(Biomes.THE_END, EndBiomes.theEnd(placedFeatures, carvers));",
        "context.register(Biomes.END_BARRENS, EndBiomes.endBarrens(placedFeatures, carvers));",
    ] {
        assert!(BIOME_DATA_JAVA.contains(sentinel), "missing BiomeData sentinel {sentinel}");
    }
}

#[test]
fn biome_data_registered_biomes_all_have_parseable_vanilla_json() {
    let mut parsed_ids = Vec::new();
    for biome in BUILTIN_BIOMES {
        let path = format!(
            "../decompiled-server-26.1.2/data/minecraft/worldgen/biome/{}.json",
            biome.id.trim_start_matches("minecraft:")
        );
        let json = std::fs::read_to_string(&path)
            .unwrap_or_else(|err| panic!("failed to read {path}: {err}"));
        let parsed = parse_biome_json(biome.id, &json)
            .unwrap_or_else(|err| panic!("failed to parse {path}: {err}"));
        parsed_ids.push(parsed.id);
    }

    assert_eq!(parsed_ids.len(), 65);
    assert_eq!(
        parsed_ids.first().map(String::as_str),
        Some("minecraft:the_void")
    );
    assert_eq!(
        parsed_ids.last().map(String::as_str),
        Some("minecraft:end_barrens")
    );
    assert!(parsed_ids.contains(&"minecraft:pale_garden".to_string()));
    assert!(parsed_ids.contains(&"minecraft:nether_wastes".to_string()));
    assert!(parsed_ids.contains(&"minecraft:the_end".to_string()));
}

#[test]
fn end_biomes_java_source_shape_matches_rust_end_models() {
    assert_eq!(END_BIOMES_JAVA.lines().count(), 57);
    assert_eq!(
        count_occurrences(END_BIOMES_JAVA, "public static Biome "),
        5
    );
    assert_eq!(
        count_occurrences(END_BIOMES_JAVA, "private static Biome baseEndBiome"),
        1
    );
    assert_eq!(count_occurrences(END_BIOMES_JAVA, ".addFeature("), 5);
    assert_eq!(
        count_occurrences(END_BIOMES_JAVA, "BiomeDefaultFeatures.endSpawns(mobs);"),
        1
    );
    for sentinel in [
        ".hasPrecipitation(false)",
        ".temperature(0.5F)",
        ".downfall(0.5F)",
        ".waterColor(4159204)",
        "GenerationStep.Decoration.SURFACE_STRUCTURES, EndPlacements.END_SPIKE",
        "GenerationStep.Decoration.TOP_LAYER_MODIFICATION, EndPlacements.END_PLATFORM",
        "GenerationStep.Decoration.SURFACE_STRUCTURES, EndPlacements.END_GATEWAY_RETURN",
        "GenerationStep.Decoration.VEGETAL_DECORATION, EndPlacements.CHORUS_PLANT",
        "GenerationStep.Decoration.RAW_GENERATION, EndPlacements.END_ISLAND_DECORATED",
    ] {
        assert!(
            END_BIOMES_JAVA.contains(sentinel),
            "missing EndBiomes sentinel {sentinel}"
        );
    }
}

#[test]
fn end_biomes_builtin_generation_models_match_vanilla_json() {
    for id in [
        "minecraft:the_end",
        "minecraft:end_highlands",
        "minecraft:end_midlands",
        "minecraft:small_end_islands",
        "minecraft:end_barrens",
    ] {
        let parsed = end_biome_json(id);
        let model = crate::worldgen::biome_generation_settings(id)
            .unwrap_or_else(|| panic!("missing Rust biome generation settings for {id}"));
        assert!(!parsed.has_precipitation);
        assert_eq!(parsed.temperature, 0.5);
        assert_eq!(parsed.downfall, 0.5);
        assert!(parsed.generation_settings.carvers.is_empty());
        assert_eq!(
            parsed.generation_settings.features,
            padded_feature_steps(model.feature_steps),
            "feature steps mismatch for {id}"
        );
        let parsed_monsters = parsed
            .mob_spawn_settings
            .spawners
            .get(&MobCategory::Monster)
            .expect("End biome should have monster spawns");
        let model_monsters = crate::worldgen::biome_spawns_for_category(model, "monster");
        assert_eq!(parsed_monsters.len(), model_monsters.len());
        assert_eq!(parsed_monsters[0].entity_type, "minecraft:enderman");
        assert_eq!(parsed_monsters[0].weight, model_monsters[0].weight);
        assert_eq!(parsed_monsters[0].min_count, model_monsters[0].min_count);
        assert_eq!(parsed_monsters[0].max_count, model_monsters[0].max_count);
    }
}

#[test]
fn climate_quantization_and_parameter_distance_match_vanilla_rules() {
    assert_eq!(quantize_coord(0.375), 3750);
    assert_eq!(quantize_coord(-0.5), -5000);
    assert_eq!(unquantize_coord(3750), 0.375);

    let point = climate_point(0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.175);
    assert_eq!(point.offset, 1750);
    assert_eq!(point.temperature.distance(quantize_coord(0.4)), 4000);
    assert_eq!(point.temperature.distance(quantize_coord(0.0)), 0);
}

#[test]
fn climate_sampler_sample_uses_quart_to_block_context_before_quantization() {
    assert_eq!(quart_to_block(7), 28);
    assert_eq!(quart_to_block(-3), -12);

    let sample = climate_sampler_sample(
        7,
        -3,
        11,
        ClimateSamplerInput {
            temperature: 0.25,
            humidity: -0.125,
            continentalness: -0.5,
            erosion: 0.75,
            depth: 0.0,
            weirdness: 0.375,
        },
    );
    assert_eq!(sample.block_x, 28);
    assert_eq!(sample.block_y, -12);
    assert_eq!(sample.block_z, 44);
    assert_eq!(sample.target.temperature, 2500);
    assert_eq!(sample.target.humidity, -1250);
    assert_eq!(sample.target.continentalness, -5000);
    assert_eq!(sample.target.erosion, 7500);
    assert_eq!(sample.target.depth, 0);
    assert_eq!(sample.target.weirdness, 3750);
}

#[test]
fn nether_multi_noise_parameters_select_nearest_biome() {
    assert_eq!(NETHER_BIOME_PARAMETERS.len(), 5);
    assert_eq!(
        select_climate_biome(
            NETHER_BIOME_PARAMETERS,
            climate_target(0.4, 0.0, 0.0, 0.0, 0.0, 0.0)
        ),
        Some("minecraft:crimson_forest")
    );
    assert_eq!(
        select_climate_biome(
            NETHER_BIOME_PARAMETERS,
            climate_target(0.0, 0.5, 0.0, 0.0, 0.0, 0.0)
        ),
        Some("minecraft:warped_forest")
    );
    assert_eq!(
        select_climate_biome(
            NETHER_BIOME_PARAMETERS,
            climate_target(-0.5, 0.0, 0.0, 0.0, 0.0, 0.0)
        ),
        Some("minecraft:basalt_deltas")
    );
}

#[test]
fn multi_noise_parameter_list_presets_match_vanilla_known_presets() {
    let nether = multi_noise_parameter_list_preset("nether").unwrap();
    assert_eq!(nether.id, "minecraft:nether");
    assert_eq!(nether.parameters, NETHER_BIOME_PARAMETERS);
    assert_eq!(
        nether.used_biomes,
        &[
            "minecraft:nether_wastes",
            "minecraft:soul_sand_valley",
            "minecraft:crimson_forest",
            "minecraft:warped_forest",
            "minecraft:basalt_deltas"
        ]
    );

    let overworld = multi_noise_parameter_list_preset("minecraft:overworld").unwrap();
    assert_eq!(overworld.id, "minecraft:overworld");
    // Parameter list is generated at runtime by OverworldBiomeBuilder; verify its extent.
    assert!(overworld.parameters.len() > 100);
    assert!(overworld.used_biomes.contains(&"minecraft:mushroom_fields"));
    assert!(overworld.used_biomes.contains(&"minecraft:plains"));
    assert!(overworld.used_biomes.contains(&"minecraft:deep_dark"));
    assert!(multi_noise_parameter_list_preset("missing").is_none());
}

#[test]
fn overworld_biome_parameters_builder_generates_complete_parameter_list() {
    let params = overworld_biome_parameters();
    // The builder generates hundreds of entries covering all slices, temperatures,
    // humidities, continentalnesses, erosions, and depths.
    assert!(
        params.len() > 100,
        "expected hundreds of entries, got {}",
        params.len()
    );

    // Lazy init is idempotent — same slice is returned every call.
    assert!(std::ptr::eq(params, overworld_biome_parameters()));

    // Mushroom fields occupies mushroomFieldsContinentalness (-1.2 to -1.05), which is
    // disjoint from all other continentalness ranges, so it unambiguously wins when
    // continentalness is anywhere in that interval.
    assert_eq!(
        select_climate_biome(params, climate_target(0.0, 0.0, -1.1, 0.0, 0.0, 0.0)),
        Some("minecraft:mushroom_fields")
    );

    // Underground biomes have depth=span(0.2, 0.9) or depth=point(1.1), making them
    // unambiguous winners when the depth target is in those ranges.
    //
    // Dripstone caves: cont=span(0.8,1.0), hum=full_range. Low humidity ensures
    // lush_caves (hum=span(0.7,1.0)) loses heavily on the humidity axis.
    assert_eq!(
        select_climate_biome(params, climate_target(0.0, -0.5, 0.9, 0.0, 0.5, 0.0)),
        Some("minecraft:dripstone_caves")
    );

    // Lush caves: hum=span(0.7,1.0), cont=full_range. High humidity and mid-depth.
    assert_eq!(
        select_climate_biome(params, climate_target(0.0, 0.9, 0.0, 0.0, 0.5, 0.0)),
        Some("minecraft:lush_caves")
    );

    // Deep dark: depth=point(1.1), eros=span(eros[0],eros[1]). The point(1.1) depth
    // is far from surface entries (distance≥1000) and underground span (distance=2000),
    // so deep_dark wins decisively when depth=1.1.
    assert_eq!(
        select_climate_biome(params, climate_target(0.0, 0.0, 0.0, -0.9, 1.1, 0.0)),
        Some("minecraft:deep_dark")
    );

    // All key biomes are reachable from the builder output.
    let biome_ids: std::collections::HashSet<_> = params.iter().map(|e| e.biome).collect();
    for expected in &[
        "minecraft:mushroom_fields",
        "minecraft:jagged_peaks",
        "minecraft:frozen_peaks",
        "minecraft:stony_peaks",
        "minecraft:snowy_slopes",
        "minecraft:grove",
        "minecraft:river",
        "minecraft:frozen_river",
        "minecraft:swamp",
        "minecraft:mangrove_swamp",
        "minecraft:deep_dark",
        "minecraft:dripstone_caves",
        "minecraft:lush_caves",
        "minecraft:windswept_savanna",
        "minecraft:cherry_grove",
        "minecraft:pale_garden",
    ] {
        assert!(
            biome_ids.contains(expected),
            "{expected} not found in overworld parameter list"
        );
    }
}

#[test]
fn climate_selection_uses_first_entry_when_fitness_ties() {
    let entries = [
        ClimateBiomeEntry {
            parameters: climate_point(0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0),
            biome: "minecraft:plains",
        },
        ClimateBiomeEntry {
            parameters: climate_point(0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0),
            biome: "minecraft:forest",
        },
    ];
    assert_eq!(
        select_climate_biome(&entries, climate_target(0.0, 0.0, 0.0, 0.0, 0.0, 0.0)),
        Some("minecraft:plains")
    );
}

#[test]
fn climate_parameter_list_index_matches_bruteforce_search_contract() {
    let list = ClimateParameterList::new(vec![
        ClimateBiomeEntry {
            parameters: climate_point(-0.5, 0.0, 0.0, 0.0, 0.0, 0.0, 0.175),
            biome: "minecraft:basalt_deltas",
        },
        ClimateBiomeEntry {
            parameters: climate_point(0.4, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0),
            biome: "minecraft:crimson_forest",
        },
        ClimateBiomeEntry {
            parameters: climate_point(0.0, 0.5, 0.0, 0.0, 0.0, 0.0, 0.375),
            biome: "minecraft:warped_forest",
        },
    ])
    .unwrap();
    let target = climate_target(0.35, 0.02, 0.0, 0.0, 0.0, 0.0);
    assert_eq!(
        list.find_value_bruteforce(target),
        "minecraft:crimson_forest"
    );
    assert_eq!(list.find_value_index(target), "minecraft:crimson_forest");

    // Verify climate_node_distance computes sum-of-squared-parameter-distances.
    let explicit_ps = climate_point(-0.5, 0.0, 0.0, 0.0, 0.0, 0.0, 0.175).parameter_space();
    let expected_dist: i64 = explicit_ps
        .iter()
        .zip(target.parameter_array())
        .map(|(p, t)| {
            let d = p.distance(t);
            d * d
        })
        .sum();
    assert_eq!(
        climate_node_distance(explicit_ps, target.parameter_array()),
        expected_dist
    );
    assert_eq!(
        ClimateParameter { min: -10, max: -2 }
            .distance_parameter(ClimateParameter { min: 3, max: 7 }),
        5
    );
    assert_eq!(
        ClimateParameter { min: -10, max: -2 }
            .span_parameter(Some(ClimateParameter { min: 3, max: 7 })),
        ClimateParameter { min: -10, max: 7 }
    );
    assert_eq!(
        ClimateParameterList::new(Vec::new()).unwrap_err(),
        "Need at least one value to build the search tree.".to_string()
    );
}

#[test]
fn biome_source_codecs_match_vanilla_registry_order() {
    assert_eq!(biome_source_codec("fixed"), Some("minecraft:fixed"));
    assert_eq!(
        biome_source_codec("minecraft:multi_noise"),
        Some("minecraft:multi_noise")
    );
    assert_eq!(
        biome_source_codec("checkerboard"),
        Some("minecraft:checkerboard")
    );
    assert_eq!(biome_source_codec("the_end"), Some("minecraft:the_end"));
    assert_eq!(biome_source_codec("custom"), None);

    assert_eq!(
        biome_source_from_stem_id("minecraft:plains"),
        Some(BiomeSourceModel::Fixed {
            biome: "minecraft:plains"
        })
    );
    assert_eq!(
        biome_source_from_stem_id("multi_noise/nether"),
        Some(BiomeSourceModel::MultiNoisePreset {
            preset: "minecraft:nether"
        })
    );
    assert_eq!(
        biome_source_from_stem_id("the_end"),
        Some(BiomeSourceModel::TheEnd)
    );
}

#[test]
fn fixed_checkerboard_nether_and_end_sources_select_like_vanilla() {
    let fixed = BiomeSourceModel::Fixed {
        biome: "minecraft:plains",
    };
    assert_eq!(
        select_biome_from_source(
            &fixed,
            120,
            -10,
            -44,
            climate_target(0.4, 0.0, 0.0, 0.0, 0.0, 0.0),
            0.0
        ),
        Some("minecraft:plains")
    );

    let checkerboard = checkerboard_biome_source(
        vec!["minecraft:plains", "minecraft:desert", "minecraft:forest"],
        2,
    )
    .unwrap();
    assert_eq!(
        select_biome_from_source(
            &checkerboard,
            0,
            0,
            0,
            climate_target(0.0, 0.0, 0.0, 0.0, 0.0, 0.0),
            0.0
        ),
        Some("minecraft:plains")
    );
    assert_eq!(
        select_biome_from_source(
            &checkerboard,
            16,
            0,
            0,
            climate_target(0.0, 0.0, 0.0, 0.0, 0.0, 0.0),
            0.0
        ),
        Some("minecraft:desert")
    );
    assert_eq!(
        checkerboard_biome_source(vec!["minecraft:plains"], 63).unwrap_err(),
        "Checkerboard biome source scale must be in 0..=62".to_string()
    );

    let nether = BiomeSourceModel::MultiNoisePreset {
        preset: "minecraft:nether",
    };
    assert_eq!(
        select_biome_from_source(
            &nether,
            0,
            0,
            0,
            climate_target(0.0, 0.5, 0.0, 0.0, 0.0, 0.0),
            0.0
        ),
        Some("minecraft:warped_forest")
    );

    assert_eq!(select_end_biome(0, 64, 0, -1.0), "minecraft:the_end");
    assert_eq!(select_end_biome(300, 64, 0, 0.3), "minecraft:end_highlands");
    assert_eq!(select_end_biome(300, 64, 0, 0.0), "minecraft:end_midlands");
    assert_eq!(
        select_end_biome(300, 64, 0, -0.3),
        "minecraft:small_end_islands"
    );
    assert_eq!(select_end_biome(300, 64, 0, -0.1), "minecraft:end_barrens");
}

#[test]
fn parse_multi_noise_preset_json_resolves_vanilla_data_files() {
    // Both vanilla JSON files just contain {"preset":"minecraft:X"}.
    let overworld = parse_multi_noise_preset_json(r#"{"preset":"minecraft:overworld"}"#);
    assert!(overworld.is_some());
    assert_eq!(overworld.unwrap().id, "minecraft:overworld");

    let nether = parse_multi_noise_preset_json(r#"{"preset":"minecraft:nether"}"#);
    assert!(nether.is_some());
    assert_eq!(nether.unwrap().id, "minecraft:nether");

    // Unknown preset returns None.
    assert!(parse_multi_noise_preset_json(r#"{"preset":"minecraft:unknown"}"#).is_none());
    // Malformed JSON returns None.
    assert!(parse_multi_noise_preset_json("{}").is_none());
    assert!(parse_multi_noise_preset_json("not json").is_none());
}

#[test]
fn climate_rtree_find_nearest_agrees_with_bruteforce_on_overworld_params() {
    use super::ClimateParameterList;
    let params = overworld_biome_parameters();
    let list = ClimateParameterList::new(params.to_vec()).unwrap();

    // Test a variety of climate targets across the parameter space.
    let test_targets: &[ClimateTarget] = &[
        climate_target(0.0, 0.0, 0.0, 0.0, 0.0, 0.0),
        climate_target(-0.8, -0.5, -1.1, 0.0, 0.0, 0.0), // mushroom fields
        climate_target(0.0, 0.0, 0.9, 0.0, 0.5, 0.0),    // dripstone caves
        climate_target(0.0, 0.9, 0.0, 0.0, 0.5, 0.0),    // lush caves
        climate_target(0.0, 0.0, 0.0, -0.9, 1.1, 0.0),   // deep dark
        climate_target(0.8, 0.8, 0.5, -0.5, 0.0, -0.4),  // warm inland
        climate_target(-0.8, -0.8, -0.5, 0.3, 0.0, 0.4), // cold ocean area
    ];

    for &target in test_targets {
        let bruteforce = list.find_value_bruteforce(target);
        let rtree = list.find_value_index(target);
        assert_eq!(
            bruteforce, rtree,
            "R-tree and brute-force disagree at {:?}: brute={bruteforce} rtree={rtree}",
            target
        );
    }
}

#[test]
fn plains_biome_generation_and_mob_spawn_settings_match_vanilla_json() {
    // Load from the vanilla 26.1.2 data file.
    let json = std::fs::read_to_string(
        "../decompiled-server-26.1.2/data/minecraft/worldgen/biome/plains.json",
    )
    .expect("plains.json must be present in decompiled server data");

    let biome =
        parse_biome_json("minecraft:plains", &json).expect("plains.json must parse without error");

    assert_eq!(biome.id, "minecraft:plains");
    assert!(biome.has_precipitation);
    assert!(
        (biome.temperature - 0.8).abs() < 1e-4,
        "temperature should be 0.8"
    );
    assert!(
        (biome.downfall - 0.4).abs() < 1e-4,
        "downfall should be 0.4"
    );

    // --- BiomeGenerationSettings ---

    let gen = &biome.generation_settings;

    // Exactly 11 GenerationStep.Decoration steps.
    assert_eq!(
        gen.features.len(),
        11,
        "features must have exactly 11 steps"
    );

    // Step 0 (RAW_GENERATION): empty for plains.
    assert!(
        gen.features[0].is_empty(),
        "RAW_GENERATION step should be empty for plains"
    );

    // Step 6 (UNDERGROUND_ORES): 29 entries including "minecraft:ore_dirt".
    assert_eq!(
        gen.features[6].len(),
        29,
        "UNDERGROUND_ORES step should have 29 entries"
    );
    assert!(
        gen.features[6].contains(&"minecraft:ore_dirt".to_string()),
        "UNDERGROUND_ORES step must include minecraft:ore_dirt"
    );

    // Step 9 (VEGETAL_DECORATION): contains "minecraft:trees_plains".
    assert!(
        gen.features[9].contains(&"minecraft:trees_plains".to_string()),
        "VEGETAL_DECORATION step must include minecraft:trees_plains"
    );

    // Carvers: "minecraft:cave" is present.
    assert!(
        gen.carvers.contains(&"minecraft:cave".to_string()),
        "carvers must include minecraft:cave"
    );

    // --- MobSpawnSettings ---

    let mob = &biome.mob_spawn_settings;

    // creature_spawn_probability: plains.json omits the key → default 0.1.
    assert!(
        (mob.creature_spawn_probability - 0.1).abs() < 1e-6,
        "creature_spawn_probability should be the default 0.1"
    );

    // Monster category: 9 entries; minecraft:spider has weight 100.
    let monsters = mob
        .spawners
        .get(&MobCategory::Monster)
        .expect("monster spawner category must be present");
    assert_eq!(
        monsters.len(),
        9,
        "monster spawner list should have 9 entries"
    );
    let spider = monsters
        .iter()
        .find(|e| e.entity_type == "minecraft:spider")
        .expect("minecraft:spider must be in monster spawners");
    assert_eq!(spider.weight, 100, "minecraft:spider weight must be 100");
    assert_eq!(spider.min_count, 4);
    assert_eq!(spider.max_count, 4);

    // spawn_costs: empty for plains.
    assert!(
        mob.spawn_costs.is_empty(),
        "plains spawn_costs should be empty"
    );
}
