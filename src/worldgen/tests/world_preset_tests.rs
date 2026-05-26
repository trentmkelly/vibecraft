use super::*;

#[test]
fn flat_generator_defaults_and_presets_match_vanilla_bootstrap() {
    assert_eq!(
        FLAT_DEFAULT_LAYERS,
        &[
            FlatLayerInfo {
                height: 1,
                block: "minecraft:bedrock"
            },
            FlatLayerInfo {
                height: 2,
                block: "minecraft:dirt"
            },
            FlatLayerInfo {
                height: 1,
                block: "minecraft:grass_block"
            },
        ]
    );
    assert_eq!(
        super::super::flat_layers_total_height(FLAT_DEFAULT_LAYERS),
        4
    );
    assert_eq!(
        super::super::flat_block_at_y(FLAT_DEFAULT_LAYERS, 0),
        Some("minecraft:bedrock")
    );
    assert_eq!(
        super::super::flat_block_at_y(FLAT_DEFAULT_LAYERS, 2),
        Some("minecraft:dirt")
    );
    assert_eq!(
        super::super::flat_block_at_y(FLAT_DEFAULT_LAYERS, 3),
        Some("minecraft:grass_block")
    );
    assert_eq!(super::super::flat_block_at_y(FLAT_DEFAULT_LAYERS, 4), None);

    assert_eq!(
        FLAT_GENERATOR_PRESETS
            .iter()
            .map(|preset| preset.id)
            .collect::<Vec<_>>(),
        vec![
            "minecraft:classic_flat",
            "minecraft:tunnelers_dream",
            "minecraft:water_world",
            "minecraft:overworld",
            "minecraft:snowy_kingdom",
            "minecraft:bottomless_pit",
            "minecraft:desert",
            "minecraft:redstone_ready",
            "minecraft:the_void",
        ]
    );
    let overworld = super::super::flat_generator_preset("overworld").unwrap();
    assert_eq!(overworld.display, "minecraft:short_grass");
    assert_eq!(overworld.biome, "minecraft:plains");
    assert!(overworld.add_lakes);
    assert!(overworld.decoration);
    assert_eq!(super::super::flat_layers_total_height(overworld.layers), 64);
    assert_eq!(
        super::super::flat_block_at_y(overworld.layers, 0),
        Some("minecraft:bedrock")
    );
    assert_eq!(
        super::super::flat_block_at_y(overworld.layers, 63),
        Some("minecraft:grass_block")
    );
    assert!(overworld
        .structures
        .contains(&"minecraft:pillager_outposts"));

    let water = super::super::flat_generator_preset("minecraft:water_world").unwrap();
    assert_eq!(water.layers[0].height, 1);
    assert_eq!(water.layers[0].block, "minecraft:bedrock");
    assert_eq!(super::super::flat_layers_total_height(water.layers), 170);
    assert_eq!(
        super::super::flat_block_at_y(water.layers, 169),
        Some("minecraft:water")
    );

    let void = super::super::flat_generator_preset("the_void").unwrap();
    assert!(super::super::flat_layers_are_void(void.layers));
    assert_eq!(void.biome, "minecraft:the_void");
    assert_eq!(
        super::super::flat_block_at_y(void.layers, 0),
        Some("minecraft:air")
    );

    assert!(super::super::validate_flat_layers(FLAT_DEFAULT_LAYERS).is_ok());
    assert_eq!(
        super::super::validate_flat_layers(&[FlatLayerInfo {
            height: 385,
            block: "minecraft:stone",
        }]),
        Err("Sum of layer heights is > 384".to_string())
    );
}

#[test]
fn flat_adjusted_generation_settings_filter_biome_features_like_vanilla() {
    let plains = super::super::biome_generation_settings("plains").unwrap();
    let overworld = super::super::flat_generator_settings(
        super::super::flat_generator_preset("overworld").unwrap(),
    )
    .unwrap();
    let steps =
        super::super::flat_adjusted_generation_feature_steps(&overworld, plains.feature_steps);

    assert_eq!(
        steps[GenerationDecorationStep::Lakes as usize],
        vec![
            "minecraft:lake_lava_underground",
            "minecraft:lake_lava_surface"
        ]
    );
    assert!(steps[GenerationDecorationStep::UndergroundStructures as usize].is_empty());
    assert!(steps[GenerationDecorationStep::SurfaceStructures as usize].is_empty());
    assert!(steps[GenerationDecorationStep::VegetalDecoration as usize]
        .contains(&"minecraft:trees_plains"));
    assert!(
        steps[GenerationDecorationStep::TopLayerModification as usize]
            .contains(&"minecraft:freeze_top_layer")
    );
    assert!(
        !steps[GenerationDecorationStep::TopLayerModification as usize]
            .contains(&"minecraft:fill_layer")
    );

    let void = super::super::flat_generator_settings(
        super::super::flat_generator_preset("the_void").unwrap(),
    )
    .unwrap();
    let void_steps =
        super::super::flat_adjusted_generation_feature_steps(&void, plains.feature_steps);
    assert!(void_steps[GenerationDecorationStep::VegetalDecoration as usize].is_empty());
    assert_eq!(
        void_steps[GenerationDecorationStep::Lakes as usize],
        vec![
            "minecraft:lake_lava_underground",
            "minecraft:lake_lava_surface"
        ]
    );
    assert_eq!(
        void_steps[GenerationDecorationStep::TopLayerModification as usize],
        vec!["minecraft:fill_layer"]
    );
    assert_eq!(void.top_layer_modifications, vec![(0, "minecraft:air")]);
}

#[test]
fn world_preset_sources_match_vanilla_bootstrap_dimensions() {
    assert_eq!(
        WORLD_PRESETS
            .iter()
            .map(|preset| preset.id)
            .collect::<Vec<_>>(),
        vec![
            "minecraft:normal",
            "minecraft:flat",
            "minecraft:large_biomes",
            "minecraft:amplified",
            "minecraft:single_biome_surface",
            "minecraft:debug_all_block_states",
        ]
    );

    let normal = super::super::world_preset("normal").unwrap();
    assert_eq!(
        super::super::world_preset_dimensions_in_order(normal),
        [
            "minecraft:overworld",
            "minecraft:the_nether",
            "minecraft:the_end"
        ]
    );
    assert_eq!(normal.overworld.generator, "minecraft:noise");
    assert_eq!(normal.overworld.noise_settings, Some("minecraft:overworld"));
    assert_eq!(normal.nether.noise_settings, Some("minecraft:nether"));
    assert_eq!(normal.end.biome_source, "minecraft:the_end");

    let flat = super::super::world_preset("minecraft:flat").unwrap();
    assert_eq!(flat.overworld.generator, "minecraft:flat");
    assert_eq!(flat.overworld.biome_source, "minecraft:plains");
    assert_eq!(flat.overworld.noise_settings, None);

    let amplified = super::super::world_preset("amplified").unwrap();
    assert_eq!(
        amplified.overworld.noise_settings,
        Some("minecraft:amplified")
    );
    let single = super::super::world_preset("single_biome_surface").unwrap();
    assert_eq!(single.overworld.biome_source, "minecraft:fixed/plains");
    let debug = super::super::world_preset("debug_all_block_states").unwrap();
    assert_eq!(debug.overworld.generator, "minecraft:debug");

    assert_eq!(
        super::super::world_preset_from_overworld_generator("flat"),
        Some("minecraft:flat")
    );
    assert_eq!(
        super::super::world_preset_from_overworld_generator("minecraft:debug"),
        Some("minecraft:debug_all_block_states")
    );
    assert_eq!(
        super::super::world_preset_from_overworld_generator("minecraft:noise"),
        Some("minecraft:normal")
    );
    assert_eq!(
        super::super::world_preset_from_overworld_generator("custom"),
        None
    );

    assert!(super::super::validate_world_preset_dimensions(&[
        "minecraft:the_nether",
        "minecraft:overworld",
    ])
    .is_ok());
    assert_eq!(
        super::super::validate_world_preset_dimensions(&["minecraft:the_nether"]),
        Err("Missing overworld dimension".to_string())
    );
}

#[test]
fn chunk_generator_kind_dispatches_vanilla_codecs() {
    assert_eq!(
        super::super::chunk_generator_kind("noise"),
        Some(super::super::ChunkGeneratorKind::Noise)
    );
    assert_eq!(
        super::super::chunk_generator_kind("minecraft:flat"),
        Some(super::super::ChunkGeneratorKind::Flat)
    );
    assert_eq!(
        super::super::chunk_generator_kind("minecraft:debug"),
        Some(super::super::ChunkGeneratorKind::Debug)
    );
    assert_eq!(super::super::chunk_generator_kind("minecraft:custom"), None);
}

#[test]
fn normal_world_preset_resolves_dimension_generators_like_vanilla() {
    let normal = super::super::resolve_world_preset("normal").unwrap();
    assert_eq!(normal.id, "minecraft:normal");
    assert_eq!(normal.overworld.dimension, "minecraft:overworld");
    assert_noise_generator(
        &normal.overworld.generator,
        "minecraft:multi_noise/overworld",
        BiomeSourceModel::MultiNoisePreset {
            preset: "minecraft:overworld",
        },
        "minecraft:overworld",
    );
    assert_noise_generator(
        &normal.nether.generator,
        "minecraft:multi_noise/nether",
        BiomeSourceModel::MultiNoisePreset {
            preset: "minecraft:nether",
        },
        "minecraft:nether",
    );
    assert_noise_generator(
        &normal.end.generator,
        "minecraft:the_end",
        BiomeSourceModel::TheEnd,
        "minecraft:end",
    );
}

#[test]
fn noise_world_preset_variants_resolve_overworld_settings_like_vanilla() {
    let large = super::super::resolve_world_preset("large_biomes").unwrap();
    assert_noise_settings(&large.overworld.generator, "minecraft:large_biomes");

    let amplified = super::super::resolve_world_preset("amplified").unwrap();
    assert_noise_settings(&amplified.overworld.generator, "minecraft:amplified");

    let single = super::super::resolve_world_preset("single_biome_surface").unwrap();
    assert_noise_generator(
        &single.overworld.generator,
        "minecraft:fixed/plains",
        BiomeSourceModel::Fixed {
            biome: "minecraft:plains",
        },
        "minecraft:overworld",
    );
}

#[test]
fn flat_world_preset_resolves_flat_generator_like_vanilla() {
    let flat = super::super::resolve_world_preset("flat").unwrap();
    let super::super::ResolvedChunkGenerator::Flat {
        biome_source_model,
        settings,
    } = &flat.overworld.generator
    else {
        panic!("flat overworld should resolve to flat");
    };

    assert_eq!(
        biome_source_model,
        &BiomeSourceModel::Fixed {
            biome: "minecraft:plains",
        }
    );
    assert_eq!(settings.biome, "minecraft:plains");
    assert_eq!(
        settings.expanded_layers,
        vec![
            Some("minecraft:bedrock"),
            Some("minecraft:dirt"),
            Some("minecraft:dirt"),
            Some("minecraft:grass_block"),
        ]
    );
}

#[test]
fn debug_world_preset_resolves_debug_generator_like_vanilla() {
    let debug = super::super::resolve_world_preset("debug_all_block_states").unwrap();
    let super::super::ResolvedChunkGenerator::Debug {
        biome,
        biome_source_model,
    } = &debug.overworld.generator
    else {
        panic!("debug overworld should resolve to debug");
    };

    assert_eq!(*biome, "minecraft:plains");
    assert_eq!(
        biome_source_model,
        &BiomeSourceModel::Fixed {
            biome: "minecraft:plains",
        }
    );
}

fn assert_noise_generator(
    generator: &super::super::ResolvedChunkGenerator,
    expected_biome_source: &str,
    expected_biome_source_model: BiomeSourceModel,
    expected_noise_settings: &str,
) {
    let super::super::ResolvedChunkGenerator::Noise {
        biome_source,
        biome_source_model,
        noise_settings,
    } = generator
    else {
        panic!("world preset stem should resolve to noise");
    };

    assert_eq!(*biome_source, expected_biome_source);
    assert_eq!(*biome_source_model, expected_biome_source_model);
    assert_eq!(noise_settings.id, expected_noise_settings);
}

fn assert_noise_settings(
    generator: &super::super::ResolvedChunkGenerator,
    expected_noise_settings: &str,
) {
    let super::super::ResolvedChunkGenerator::Noise { noise_settings, .. } = generator else {
        panic!("world preset stem should resolve to noise");
    };
    assert_eq!(noise_settings.id, expected_noise_settings);
}

#[test]
fn world_preset_resolver_rejects_invalid_generator_wiring() {
    assert_eq!(
        super::super::resolve_world_preset("missing").unwrap_err(),
        "Unknown world preset missing".to_string()
    );

    assert_eq!(
        super::super::resolve_level_stem(&super::super::LevelStemPreset {
            dimension: "minecraft:overworld",
            generator: "minecraft:custom",
            biome_source: "minecraft:plains",
            noise_settings: None,
        })
        .unwrap_err(),
        "Unknown chunk generator minecraft:custom".to_string()
    );

    assert_eq!(
        super::super::resolve_level_stem(&super::super::LevelStemPreset {
            dimension: "minecraft:overworld",
            generator: "minecraft:noise",
            biome_source: "minecraft:plains",
            noise_settings: None,
        })
        .unwrap_err(),
        "Noise generator minecraft:overworld has no settings".to_string()
    );

    assert_eq!(
        super::super::resolve_level_stem(&super::super::LevelStemPreset {
            dimension: "minecraft:overworld",
            generator: "minecraft:flat",
            biome_source: "minecraft:plains",
            noise_settings: Some("minecraft:overworld"),
        })
        .unwrap_err(),
        "Flat generator minecraft:overworld must not carry noise settings".to_string()
    );
}

#[test]
fn world_preset_codecs_parse_vanilla_registry_json() {
    let normal_raw = std::fs::read_to_string(
        "../decompiled-server-26.1.2/data/minecraft/worldgen/world_preset/normal.json",
    )
    .unwrap();
    let normal = super::super::parse_world_preset_json(&normal_raw).unwrap();
    assert_eq!(
        normal
            .dimensions
            .stems
            .iter()
            .map(|(id, _)| id.as_str())
            .collect::<Vec<_>>(),
        vec![
            "minecraft:overworld",
            "minecraft:the_nether",
            "minecraft:the_end"
        ]
    );
    assert_eq!(
        normal.dimensions.stems[0].1,
        super::super::ParsedLevelStem {
            dimension_type: "minecraft:overworld".to_string(),
            generator: super::super::ParsedChunkGenerator::Noise {
                biome_source: super::super::ParsedBiomeSource::MultiNoisePreset {
                    preset: "minecraft:overworld".to_string(),
                },
                settings: "minecraft:overworld".to_string(),
            },
        }
    );
    assert_eq!(
        normal.dimensions.stems[1].1.generator,
        super::super::ParsedChunkGenerator::Noise {
            biome_source: super::super::ParsedBiomeSource::MultiNoisePreset {
                preset: "minecraft:nether".to_string(),
            },
            settings: "minecraft:nether".to_string(),
        }
    );
    assert_eq!(
        normal.dimensions.stems[2].1.generator,
        super::super::ParsedChunkGenerator::Noise {
            biome_source: super::super::ParsedBiomeSource::TheEnd,
            settings: "minecraft:end".to_string(),
        }
    );

    let flat_raw = std::fs::read_to_string(
        "../decompiled-server-26.1.2/data/minecraft/worldgen/world_preset/flat.json",
    )
    .unwrap();
    let flat = super::super::parse_world_preset_json(&flat_raw).unwrap();
    assert_eq!(
        flat.dimensions.stems[0].1.generator,
        super::super::ParsedChunkGenerator::Flat {
            settings: super::super::ParsedFlatGeneratorSettings {
                biome: "minecraft:plains".to_string(),
                structure_overrides: vec![
                    "minecraft:strongholds".to_string(),
                    "minecraft:villages".to_string(),
                ],
                add_lakes: false,
                decoration: false,
                layers: vec![
                    (1, "minecraft:bedrock".to_string()),
                    (2, "minecraft:dirt".to_string()),
                    (1, "minecraft:grass_block".to_string()),
                ],
            },
        }
    );
}

#[test]
fn worldgen_preset_registry_loads_vanilla_world_and_flat_presets() {
    let registry = super::super::load_worldgen_preset_registry(
        "../decompiled-server-26.1.2/data/minecraft/worldgen",
    )
    .unwrap();

    assert_eq!(registry.world_presets.len(), 6);
    assert_eq!(registry.flat_level_generator_presets.len(), 9);
    assert!(registry.world_presets.contains_key("minecraft:normal"));
    assert!(registry.world_presets.contains_key("minecraft:flat"));
    assert!(registry
        .flat_level_generator_presets
        .contains_key("minecraft:classic_flat"));
    assert!(registry
        .flat_level_generator_presets
        .contains_key("minecraft:bottomless_pit"));

    let normal = registry.world_presets.get("minecraft:normal").unwrap();
    assert_eq!(
        normal.dimensions.stems[0].1.generator,
        super::super::ParsedChunkGenerator::Noise {
            biome_source: super::super::ParsedBiomeSource::MultiNoisePreset {
                preset: "minecraft:overworld".to_string(),
            },
            settings: "minecraft:overworld".to_string(),
        }
    );

    let debug = registry
        .world_presets
        .get("minecraft:debug_all_block_states")
        .unwrap();
    assert_eq!(
        debug.dimensions.stems[0].1.generator,
        super::super::ParsedChunkGenerator::Debug
    );

    let bottomless_pit = registry
        .flat_level_generator_presets
        .get("minecraft:bottomless_pit")
        .unwrap();
    assert_eq!(
        bottomless_pit.structure_overrides,
        vec!["minecraft:villages".to_string()]
    );
    assert_eq!(
        bottomless_pit.layers,
        vec![
            (2, "minecraft:cobblestone".to_string()),
            (3, "minecraft:dirt".to_string()),
            (1, "minecraft:grass_block".to_string()),
        ]
    );
}

#[test]
fn template_pool_registry_loads_vanilla_nested_pool_json() {
    let registry = super::super::load_template_pool_registry(
        "../decompiled-server-26.1.2/data/minecraft/worldgen/template_pool",
    )
    .unwrap();

    assert!(registry.pools.contains_key("minecraft:empty"));
    assert!(registry
        .pools
        .contains_key("minecraft:pillager_outpost/base_plates"));
    assert!(registry
        .pools
        .contains_key("minecraft:trial_chambers/chamber/end"));
    assert!(registry.pools.len() > 100);

    let empty = registry.pools.get("minecraft:empty").unwrap();
    assert_eq!(empty.fallback, "minecraft:empty");
    assert!(empty.elements.is_empty());

    let base_plates = registry
        .pools
        .get("minecraft:pillager_outpost/base_plates")
        .unwrap();
    assert_eq!(base_plates.fallback, "minecraft:empty");
    assert_eq!(base_plates.elements.len(), 1);
    assert_eq!(base_plates.elements[0].weight, 1);
    assert_eq!(
        base_plates.elements[0].element.element_type,
        "minecraft:legacy_single_pool_element"
    );
    assert_eq!(
        base_plates.elements[0].element.location.as_deref(),
        Some("minecraft:pillager_outpost/base_plate")
    );
    assert_eq!(
        base_plates.elements[0].element.projection.as_deref(),
        Some("rigid")
    );
}

#[test]
fn processor_list_registry_loads_vanilla_processor_json() {
    let registry = super::super::load_processor_list_registry(
        "../decompiled-server-26.1.2/data/minecraft/worldgen/processor_list",
    )
    .unwrap();

    assert_eq!(registry.lists.len(), 40);
    assert!(registry.lists.contains_key("minecraft:empty"));
    assert!(registry.lists.contains_key("minecraft:mossify_20_percent"));
    assert!(registry
        .lists
        .contains_key("minecraft:trail_ruins_houses_archaeology"));

    let empty = registry.lists.get("minecraft:empty").unwrap();
    assert!(empty.processors.is_empty());

    let mossify = registry.lists.get("minecraft:mossify_20_percent").unwrap();
    assert_eq!(mossify.processors.len(), 1);
    assert_eq!(mossify.processors[0].processor_type, "minecraft:rule");
    assert_eq!(mossify.processors[0].rules.len(), 1);
    assert_eq!(
        mossify.processors[0].rules[0].input_predicate_type,
        "minecraft:random_block_match"
    );
    assert_eq!(
        mossify.processors[0].rules[0].location_predicate_type,
        "minecraft:always_true"
    );
    assert_eq!(
        mossify.processors[0].rules[0].output_state_name.as_deref(),
        Some("minecraft:mossy_cobblestone")
    );

    let archaeology = registry
        .lists
        .get("minecraft:trail_ruins_houses_archaeology")
        .unwrap();
    let capped = archaeology
        .processors
        .iter()
        .find(|processor| processor.processor_type == "minecraft:capped")
        .unwrap();
    assert_eq!(capped.limit, Some(6));
    let delegate = capped.delegate.as_ref().unwrap();
    assert_eq!(delegate.processor_type, "minecraft:rule");
    assert_eq!(
        delegate.rules[0].block_entity_modifier_type.as_deref(),
        Some("minecraft:append_loot")
    );
}

#[test]
fn worldgen_preset_registry_resolves_vanilla_generators() {
    let registry = super::super::load_worldgen_preset_registry(
        "../decompiled-server-26.1.2/data/minecraft/worldgen",
    )
    .unwrap();

    for id in [
        "normal",
        "flat",
        "large_biomes",
        "amplified",
        "single_biome_surface",
        "debug_all_block_states",
    ] {
        let resolved = super::super::resolve_world_preset_from_registry(&registry, id).unwrap();
        let builtin = super::super::resolve_world_preset(id).unwrap();
        assert_eq!(resolved.id, builtin.id);
        assert_eq!(resolved.nether, builtin.nether);
        assert_eq!(resolved.end, builtin.end);
        assert_eq!(resolved.overworld.dimension, builtin.overworld.dimension);
    }

    let normal = super::super::resolve_world_preset_from_registry(&registry, "minecraft:normal")
        .unwrap()
        .overworld
        .generator;
    let super::super::ResolvedChunkGenerator::Noise {
        biome_source_model,
        noise_settings,
        ..
    } = normal
    else {
        panic!("normal preset should resolve to a noise overworld generator");
    };
    assert_eq!(
        biome_source_model,
        BiomeSourceModel::MultiNoisePreset {
            preset: "minecraft:overworld",
        }
    );
    assert_eq!(noise_settings.id, "minecraft:overworld");

    let single_biome = super::super::resolve_world_preset_from_registry(
        &registry,
        "minecraft:single_biome_surface",
    )
    .unwrap()
    .overworld
    .generator;
    let super::super::ResolvedChunkGenerator::Noise {
        biome_source_model,
        noise_settings,
        ..
    } = single_biome
    else {
        panic!("single-biome preset should resolve to a noise overworld generator");
    };
    assert_eq!(
        biome_source_model,
        BiomeSourceModel::Fixed {
            biome: "minecraft:plains",
        }
    );
    assert_eq!(noise_settings.id, "minecraft:overworld");

    let flat =
        super::super::resolve_world_preset_from_registry(&registry, "minecraft:flat").unwrap();
    let super::super::ResolvedChunkGenerator::Flat { settings, .. } = flat.overworld.generator
    else {
        panic!("flat preset should resolve to a flat overworld generator");
    };
    assert_eq!(settings.biome, "minecraft:plains");
    assert_eq!(
        settings.structure_overrides,
        vec!["minecraft:strongholds", "minecraft:villages"]
    );
    assert_eq!(
        settings.layers,
        vec![
            super::super::FlatLayerInfo {
                height: 1,
                block: "minecraft:bedrock",
            },
            super::super::FlatLayerInfo {
                height: 2,
                block: "minecraft:dirt",
            },
            super::super::FlatLayerInfo {
                height: 1,
                block: "minecraft:grass_block",
            },
        ]
    );
}

#[test]
fn worldgen_settings_codec_parses_seed_options_and_dimensions() {
    let raw = r#"{
            "seed": 12345,
            "generate_features": true,
            "bonus_chest": false,
            "dimensions": {
                "minecraft:overworld": {
                    "type": "minecraft:overworld",
                    "generator": {
                        "type": "minecraft:noise",
                        "biome_source": {
                            "type": "minecraft:fixed",
                            "biome": "minecraft:plains"
                        },
                        "settings": "minecraft:overworld"
                    }
                }
            }
        }"#;

    let settings = super::super::parse_worldgen_settings_json(raw).unwrap();
    assert_eq!(settings.seed, 12345);
    assert!(settings.generate_structures);
    assert!(!settings.generate_bonus_chest);
    assert_eq!(settings.dimensions.stems.len(), 1);
    assert_eq!(settings.dimensions.stems[0].0, "minecraft:overworld");
    assert_eq!(
        settings.dimensions.stems[0].1.generator,
        super::super::ParsedChunkGenerator::Noise {
            biome_source: super::super::ParsedBiomeSource::Fixed {
                biome: "minecraft:plains".to_string(),
            },
            settings: "minecraft:overworld".to_string(),
        }
    );
}

#[test]
fn resolved_flat_generator_materializes_overworld_chunks() {
    let chunk = super::super::generate_overworld_chunk_for_preset(ChunkPos { x: -3, z: 5 }, "flat")
        .expect("flat preset should generate a concrete chunk");
    assert_eq!(chunk.pos, ChunkPos { x: -3, z: 5 });
    assert_eq!(chunk.status, "minecraft:full");
    assert_eq!(chunk.min_section_y, 0);
    assert_eq!(chunk.sections.len(), 1);
    assert!(chunk.heightmaps.contains_key("WORLD_SURFACE_WG"));
    assert!(chunk.heightmaps.contains_key("OCEAN_FLOOR_WG"));
}

#[test]
fn resolved_noise_generator_materializes_preview_terrain_chunks() {
    let chunk =
        super::super::generate_overworld_chunk_for_preset(ChunkPos { x: 0, z: 0 }, "normal")
            .expect("normal preset should generate preview terrain");
    assert_eq!(chunk.status, "minecraft:full");
    assert_eq!(chunk.min_section_y, -4);
    assert_eq!(chunk.sections.len(), 24);
    assert_eq!(chunk.sections[0].y, -4);
    assert_eq!(chunk.sections.last().unwrap().y, 19);
    assert!(chunk.heightmaps.contains_key("WORLD_SURFACE_WG"));
    assert!(chunk.heightmaps.contains_key("OCEAN_FLOOR_WG"));

    let overworld_settings = super::super::builtin_noise_generator_settings("overworld").unwrap();
    let low = super::super::noise_preview_terrain_height(0, 0, overworld_settings);
    let nearby = super::super::noise_preview_terrain_height(15, 15, overworld_settings);
    let far = super::super::noise_preview_terrain_height(96, -48, overworld_settings);
    assert_ne!(low, far);
    assert!((low - nearby).abs() < 40);

    let Tag::Compound(section) = &chunk.sections[8].block_states else {
        panic!("block states should be stored as a compound");
    };
    let Some((_, Tag::List(palette))) = section.iter().find(|(name, _)| name == "palette") else {
        panic!("block states should include a palette");
    };
    assert!(palette.contains(&super::super::block_state_tag("minecraft:grass_block")));
    assert!(palette.contains(&super::super::block_state_tag("minecraft:stone")));
}
