use super::*;

fn extracted_density_function_ids_from_json_tree() -> Vec<String> {
    fn visit_density_function_jsons(root: &Path, dir: &Path, ids: &mut Vec<String>) {
        let entries = std::fs::read_dir(dir)
            .unwrap_or_else(|err| panic!("failed to read {}: {err}", dir.display()));
        for entry in entries {
            let entry = entry.unwrap_or_else(|err| {
                panic!("failed to read entry under {}: {err}", dir.display())
            });
            let path = entry.path();
            if path.is_dir() {
                visit_density_function_jsons(root, &path, ids);
                continue;
            }
            if path.extension().and_then(|extension| extension.to_str()) != Some("json") {
                continue;
            }
            let relative = path
                .strip_prefix(root)
                .unwrap_or_else(|err| panic!("failed to relativize {}: {err}", path.display()))
                .with_extension("");
            let key = relative
                .components()
                .map(|component| component.as_os_str().to_string_lossy())
                .collect::<Vec<_>>()
                .join("/");
            ids.push(format!("minecraft:{key}"));
        }
    }

    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../decompiled-server-26.1.2/data/minecraft/worldgen/density_function");
    let mut ids = Vec::new();
    visit_density_function_jsons(&root, &root, &mut ids);
    ids.sort_unstable();
    ids
}

#[test]
fn noise_router_density_function_bootstrap_keys_match_vanilla_prefix() {
    assert_density_function_registry_keys_match_vanilla();
    assert_density_function_registry_order_matches_bootstrap();
    assert_core_overworld_density_functions();
    assert_overworld_terrain_density_functions();
    assert_cave_density_functions();
}

fn assert_density_function_registry_keys_match_vanilla() {
    let mut extracted_density_function_ids = extracted_density_function_ids_from_json_tree();
    extracted_density_function_ids.push("minecraft:overworld/final_density".to_string());
    extracted_density_function_ids.sort_unstable();
    let mut actual_density_function_ids = BUILTIN_DENSITY_FUNCTIONS
        .iter()
        .map(|entry| entry.id.to_string())
        .collect::<Vec<_>>();
    actual_density_function_ids.sort_unstable();
    assert_eq!(actual_density_function_ids, extracted_density_function_ids);
}

fn assert_density_function_registry_order_matches_bootstrap() {
    assert_eq!(
        BUILTIN_DENSITY_FUNCTIONS
            .iter()
            .map(|entry| entry.id)
            .collect::<Vec<_>>(),
        vec![
            "minecraft:zero",
            "minecraft:y",
            "minecraft:shift_x",
            "minecraft:shift_z",
            "minecraft:overworld/base_3d_noise",
            "minecraft:nether/base_3d_noise",
            "minecraft:end/base_3d_noise",
            "minecraft:overworld/continents",
            "minecraft:overworld/erosion",
            "minecraft:overworld/ridges",
            "minecraft:overworld/ridges_folded",
            "minecraft:overworld/offset",
            "minecraft:overworld/factor",
            "minecraft:overworld/jaggedness",
            "minecraft:overworld/depth",
            "minecraft:overworld/sloped_cheese",
            "minecraft:overworld_large_biomes/continents",
            "minecraft:overworld_large_biomes/erosion",
            "minecraft:overworld_large_biomes/offset",
            "minecraft:overworld_large_biomes/factor",
            "minecraft:overworld_large_biomes/jaggedness",
            "minecraft:overworld_large_biomes/depth",
            "minecraft:overworld_large_biomes/sloped_cheese",
            "minecraft:overworld_amplified/offset",
            "minecraft:overworld_amplified/factor",
            "minecraft:overworld_amplified/jaggedness",
            "minecraft:overworld_amplified/depth",
            "minecraft:overworld_amplified/sloped_cheese",
            "minecraft:end/sloped_cheese",
            "minecraft:overworld/caves/spaghetti_2d_thickness_modulator",
            "minecraft:overworld/caves/spaghetti_roughness_function",
            "minecraft:overworld/caves/pillars",
            "minecraft:overworld/caves/spaghetti_2d",
            "minecraft:overworld/caves/noodle",
            "minecraft:overworld/caves/entrances",
            "minecraft:overworld/final_density",
        ]
    );
}

fn assert_core_overworld_density_functions() {
    assert_eq!(
        builtin_density_function("overworld/base_3d_noise")
            .unwrap()
            .function
            .type_name(),
        "old_blended_noise"
    );
    assert_eq!(
        builtin_density_function("shift_x")
            .unwrap()
            .function
            .type_name(),
        DensityMarker::FlatCache.serialized_name()
    );
    let ridges_folded = builtin_density_function("overworld/ridges_folded")
        .unwrap()
        .function;
    assert_eq!(ridges_folded.type_name(), "mul");
    assert_eq!(ridges_folded.value_bounds(), (-14.14285714285714, 1.0));
    let offset = builtin_density_function("overworld/offset")
        .unwrap()
        .function;
    assert_eq!(
        offset.type_name(),
        DensityMarker::FlatCache.serialized_name()
    );
    assert_eq!(offset, super::super::OVERWORLD_OFFSET_DENSITY);
    assert_eq!(
        super::super::OVERWORLD_OFFSET_BLENDED_DENSITY,
        DensityFunction::Binary {
            kind: BinaryDensityFunction::Add,
            argument1: &super::super::OVERWORLD_OFFSET_BLEND_TARGET_DENSITY,
            argument2: &super::super::OVERWORLD_OFFSET_SPLINE_WEIGHTED_DENSITY,
        }
    );
    assert_eq!(
        super::super::OVERWORLD_OFFSET_BLEND_TARGET_DENSITY,
        DensityFunction::Binary {
            kind: BinaryDensityFunction::Mul,
            argument1: &super::super::BLEND_OFFSET_DENSITY,
            argument2: &super::super::BLEND_ALPHA_INVERSE_DENSITY,
        }
    );
    assert_eq!(
        super::super::OVERWORLD_OFFSET_SPLINE_WEIGHTED_DENSITY,
        DensityFunction::Binary {
            kind: BinaryDensityFunction::Mul,
            argument1: &super::super::OVERWORLD_OFFSET_SPLINE_WITH_OFFSET_DENSITY,
            argument2: &super::super::BLEND_ALPHA_CACHE_ONCE_DENSITY,
        }
    );
    assert_eq!(
        offset.value_bounds(),
        (-1.3037500262260437, 1.5722867486489562)
    );
    assert_eq!(
        builtin_density_function("overworld/factor")
            .unwrap()
            .function,
        super::super::OVERWORLD_FACTOR_DENSITY
    );
    assert_eq!(
        super::super::OVERWORLD_FACTOR_BLENDED_DENSITY,
        DensityFunction::Binary {
            kind: BinaryDensityFunction::Add,
            argument1: &super::super::BLENDING_FACTOR_DENSITY,
            argument2: &super::super::OVERWORLD_FACTOR_SPLINE_WEIGHTED_DENSITY,
        }
    );
    assert_eq!(
        super::super::OVERWORLD_FACTOR_SPLINE_WEIGHTED_DENSITY,
        DensityFunction::Binary {
            kind: BinaryDensityFunction::Mul,
            argument1: &super::super::BLEND_ALPHA_DENSITY,
            argument2: &super::super::OVERWORLD_FACTOR_SPLINE_DELTA_DENSITY,
        }
    );
    assert_eq!(
        builtin_density_function("overworld/jaggedness")
            .unwrap()
            .function,
        super::super::OVERWORLD_JAGGEDNESS_DENSITY
    );
    assert_eq!(
        super::super::OVERWORLD_JAGGEDNESS_BLENDED_DENSITY,
        DensityFunction::Binary {
            kind: BinaryDensityFunction::Add,
            argument1: &super::super::BLENDING_JAGGEDNESS_DENSITY,
            argument2: &super::super::OVERWORLD_JAGGEDNESS_SPLINE_WEIGHTED_DENSITY,
        }
    );
}

fn assert_overworld_terrain_density_functions() {
    for id in [
        "overworld/factor",
        "overworld/jaggedness",
        "overworld_large_biomes/factor",
        "overworld_large_biomes/jaggedness",
        "overworld_amplified/factor",
        "overworld_amplified/jaggedness",
    ] {
        let entry = builtin_density_function(id).unwrap().function;
        assert_eq!(
            entry.type_name(),
            DensityMarker::FlatCache.serialized_name()
        );
        let bounds = entry.value_bounds();
        assert!(bounds.0.is_finite(), "{id} min bound should be finite");
        assert!(bounds.1.is_finite(), "{id} max bound should be finite");
        assert!(bounds.0 <= bounds.1, "{id} bounds should be ordered");
    }
    let depth = builtin_density_function("overworld/depth")
        .unwrap()
        .function;
    assert_eq!(depth.type_name(), "add");
    let depth_bounds = depth.value_bounds();
    assert!(depth_bounds.0.is_finite());
    assert!(depth_bounds.1.is_finite());
    assert_eq!(
        builtin_density_function("overworld_large_biomes/depth")
            .unwrap()
            .function
            .type_name(),
        "add"
    );
    assert_eq!(
        builtin_density_function("overworld_amplified/depth")
            .unwrap()
            .function
            .type_name(),
        "add"
    );
    for id in [
        "overworld/sloped_cheese",
        "overworld_large_biomes/sloped_cheese",
        "overworld_amplified/sloped_cheese",
    ] {
        let entry = builtin_density_function(id).unwrap().function;
        assert_eq!(entry.type_name(), "add");
        let bounds = entry.value_bounds();
        assert!(bounds.0.is_finite(), "{id} min bound should be finite");
        assert!(bounds.1.is_finite(), "{id} max bound should be finite");
    }
}

fn assert_cave_density_functions() {
    assert_eq!(
        builtin_density_function("overworld/caves/spaghetti_2d_thickness_modulator")
            .unwrap()
            .function
            .type_name(),
        "cache_once"
    );
    let roughness = builtin_density_function("overworld/caves/spaghetti_roughness_function")
        .unwrap()
        .function;
    assert_eq!(roughness.type_name(), "cache_once");
    assert_eq!(
        roughness.value_bounds(),
        (-0.6355555555555555, 0.34222222222222215)
    );
    let pillars = builtin_density_function("overworld/caves/pillars")
        .unwrap()
        .function;
    assert_eq!(pillars.type_name(), "cache_once");
    assert_eq!(
        pillars.value_bounds(),
        (-179.00238323045266, 151.9263924897119)
    );
    let spaghetti_2d = builtin_density_function("overworld/caves/spaghetti_2d")
        .unwrap()
        .function;
    assert_eq!(spaghetti_2d.type_name(), "clamp");
    assert_eq!(spaghetti_2d.value_bounds(), (-1.0, 1.0));
    let noodle = builtin_density_function("overworld/caves/noodle")
        .unwrap()
        .function;
    assert_eq!(noodle.type_name(), "range_choice");
    assert_eq!(noodle.value_bounds(), (-0.15833333333333333, 64.0));
    let entrances = builtin_density_function("overworld/caves/entrances")
        .unwrap()
        .function;
    assert_eq!(entrances.type_name(), "cache_once");
    assert_eq!(
        entrances.value_bounds(),
        (-2.201428571428571, 1.3422222222222222)
    );
}

#[test]
fn noise_router_record_shape_and_presets_match_noise_router_data() {
    assert_eq!(
        BUILTIN_NOISE_ROUTERS
            .iter()
            .map(|entry| entry.id)
            .collect::<Vec<_>>(),
        vec![
            "minecraft:overworld",
            "minecraft:large_biomes",
            "minecraft:amplified",
            "minecraft:nether",
            "minecraft:end",
            "minecraft:caves",
            "minecraft:floating_islands",
            "minecraft:none",
        ]
    );

    let overworld = builtin_noise_router("overworld").unwrap().router;
    assert_eq!(
        overworld.field_type_names(),
        [
            "noise",            // barrier
            "noise",            // fluid_level_floodedness
            "noise",            // fluid_level_spread
            "noise",            // lava
            "shifted_noise",    // temperature
            "shifted_noise",    // vegetation
            "reference",        // continents
            "reference",        // erosion
            "reference",        // depth
            "reference",        // ridges
            "find_top_surface", // preliminary_surface_level (inline tree, not a registry reference)
            "reference",        // final_density
            "interpolated",     // vein_toggle (yLimitedInterpolatable wraps rangeChoice)
            "add",              // vein_ridged (add(-0.08f, max(veinA, veinB)))
            "noise",            // vein_gap
        ]
    );
    assert_eq!(
        overworld.barrier,
        DensityFunction::Noise {
            noise: "minecraft:aquifer_barrier",
            xz_scale: 1.0,
            y_scale: 0.5,
        }
    );
    assert_eq!(
        overworld.final_density,
        DensityFunction::Reference("minecraft:overworld/final_density")
    );

    let large = builtin_noise_router("large_biomes").unwrap().router;
    assert_eq!(
        large.temperature,
        DensityFunction::ShiftedNoise {
            shift_x: &super::super::SHIFT_X_DENSITY,
            shift_y: &super::super::ZERO_DENSITY,
            shift_z: &super::super::SHIFT_Z_DENSITY,
            xz_scale: 0.25,
            y_scale: 0.0,
            noise: "minecraft:temperature_large",
        }
    );
    assert_eq!(
        large.continents,
        DensityFunction::Reference("minecraft:overworld_large_biomes/continents")
    );

    let nether = builtin_noise_router("nether").unwrap().router;
    assert_eq!(
        nether.temperature,
        DensityFunction::ShiftedNoise {
            shift_x: &super::super::ZERO_DENSITY,
            shift_y: &super::super::ZERO_DENSITY,
            shift_z: &super::super::ZERO_DENSITY,
            xz_scale: 0.25,
            y_scale: 0.0,
            noise: "minecraft:nether/temperature",
        }
    );
    assert_eq!(
        nether.vegetation,
        DensityFunction::ShiftedNoise {
            shift_x: &super::super::ZERO_DENSITY,
            shift_y: &super::super::ZERO_DENSITY,
            shift_z: &super::super::ZERO_DENSITY,
            xz_scale: 0.25,
            y_scale: 0.0,
            noise: "minecraft:nether/vegetation",
        }
    );
    assert_eq!(
        builtin_noise_router("end")
            .unwrap()
            .router
            .erosion
            .type_name(),
        "cache_2d"
    );
    assert_eq!(
        builtin_noise_router("none").unwrap().router.final_density,
        DensityFunction::Constant(0.0)
    );
}

#[test]
fn surface_rule_codecs_and_presets_match_surface_rule_data() {
    assert_eq!(
        SURFACE_RULE_TYPES
            .iter()
            .map(|kind| kind.id)
            .collect::<Vec<_>>(),
        vec!["bandlands", "block", "sequence", "condition"]
    );
    assert_eq!(
        SURFACE_CONDITION_TYPES
            .iter()
            .map(|kind| kind.id)
            .collect::<Vec<_>>(),
        vec![
            "biome",
            "noise_threshold",
            "vertical_gradient",
            "y_above",
            "water",
            "stone_depth",
            "not",
            "steep",
            "hole",
            "above_preliminary_surface",
            "temperature",
        ]
    );

    assert_eq!(
        BUILTIN_SURFACE_RULE_PRESETS
            .iter()
            .map(|preset| preset.id)
            .collect::<Vec<_>>(),
        vec![
            "minecraft:overworld",
            "minecraft:caves",
            "minecraft:floating_islands",
            "minecraft:nether",
            "minecraft:end",
            "minecraft:air",
        ]
    );

    let overworld = super::super::builtin_surface_rule_preset("overworld").unwrap();
    assert_eq!(
        overworld.rule,
        SurfaceRuleKind::OverworldLike {
            preliminary_surface_check: true,
            bedrock_roof: false,
            bedrock_floor: true,
            deepslate: true,
        }
    );
    assert!(overworld.blocks.contains(&"minecraft:grass_block"));
    assert!(overworld.blocks.contains(&"minecraft:deepslate"));
    assert!(overworld.blocks.contains(&"minecraft:powder_snow"));
    assert!(overworld.conditions.contains(&"above_preliminary_surface"));
    assert!(overworld.conditions.contains(&"temperature"));

    let caves = super::super::builtin_surface_rule_preset("caves").unwrap();
    assert_eq!(
        caves.rule,
        SurfaceRuleKind::OverworldLike {
            preliminary_surface_check: false,
            bedrock_roof: true,
            bedrock_floor: true,
            deepslate: true,
        }
    );
    let floating = super::super::builtin_surface_rule_preset("floating_islands").unwrap();
    assert_eq!(
        floating.rule,
        SurfaceRuleKind::OverworldLike {
            preliminary_surface_check: false,
            bedrock_roof: false,
            bedrock_floor: false,
            deepslate: true,
        }
    );

    let nether = super::super::builtin_surface_rule_preset("nether").unwrap();
    assert_eq!(nether.rule, SurfaceRuleKind::Nether);
    assert!(nether.blocks.contains(&"minecraft:netherrack"));
    assert!(nether.blocks.contains(&"minecraft:warped_nylium"));
    assert!(nether.blocks.contains(&"minecraft:crimson_nylium"));
    assert!(nether
        .conditions
        .contains(&"bedrock_roof_vertical_gradient"));
    assert!(nether.conditions.contains(&"noise_threshold"));

    assert_eq!(
        super::super::builtin_surface_rule_preset("end")
            .unwrap()
            .rule,
        SurfaceRuleKind::State("minecraft:end_stone")
    );
    assert_eq!(
        super::super::builtin_surface_rule_preset("air")
            .unwrap()
            .rule,
        SurfaceRuleKind::State("minecraft:air")
    );
}

fn is_land_surface_block(block: &str) -> bool {
    !matches!(
        block,
        "minecraft:air"
            | "minecraft:cave_air"
            | "minecraft:void_air"
            | "minecraft:water"
            | "minecraft:lava"
    )
}

fn chunk_has_land_column(
    chunk: &crate::storage::chunk::LevelChunk,
    min_y: i32,
    max_y: i32,
    sea_level: i32,
) -> bool {
    (0..16_i32).any(|local_z| {
        (0..16_i32).any(|local_x| {
            let block_x = chunk.pos.x * 16 + local_x;
            let block_z = chunk.pos.z * 16 + local_z;
            (min_y..=max_y).rev().any(|y| {
                chunk
                    .get_block_state(block_x, y, block_z)
                    .as_deref()
                    .is_some_and(|block| is_land_surface_block(block) && y > sea_level)
            })
        })
    })
}

fn first_land_column_top(
    chunk: &crate::storage::chunk::LevelChunk,
    min_y: i32,
    max_y: i32,
    sea_level: i32,
) -> Option<(i32, i32, i32, String)> {
    (0..16_i32)
        .flat_map(|local_z| (0..16_i32).map(move |local_x| (local_x, local_z)))
        .find_map(|(local_x, local_z)| {
            let block_x = chunk.pos.x * 16 + local_x;
            let block_z = chunk.pos.z * 16 + local_z;
            (min_y..=max_y).rev().find_map(|y| {
                chunk
                    .get_block_state(block_x, y, block_z)
                    .and_then(|block| {
                        if is_land_surface_block(&block) && y > sea_level {
                            Some((block_x, y, block_z, block))
                        } else {
                            None
                        }
                    })
            })
        })
}

/// Parity test: after applying the overworld surface rule, any land column (top solid
/// block above sea level) should have grass on top, dirt directly below, and stone
/// several blocks further down.  Because chunk (0,0) at seed 0 may be partially or
/// fully underwater, we scan all 256 columns and pick the first one that is land.
///
/// Verifies items 155-158 of CHECKLIST_WORLDGEN.md.
#[test]
fn overworld_surface_rules_place_grass_dirt_stone_in_plains_column() {
    use super::super::{
        builtin_noise_generator_settings, builtin_noise_router, fill_noise_and_build_surface,
        load_surface_rule, noise_router_id_for_settings, ChunkPos, NONE_NOISE_ROUTER,
    };
    use crate::biome::BiomeSourceModel;

    let settings = builtin_noise_generator_settings("minecraft:overworld")
        .expect("overworld noise settings must exist");
    let router_id = noise_router_id_for_settings(*settings);
    let noise_router = builtin_noise_router(router_id)
        .map(|e| e.router)
        .unwrap_or(NONE_NOISE_ROUTER);
    let rule = load_surface_rule("minecraft:overworld")
        .expect("overworld surface rule must load from JSON");

    let sea_level = settings.sea_level;
    let min_y = settings.noise.min_y;
    let max_y = min_y + settings.noise.height - 1;

    // Try a range of nearby chunks until we find one with at least one land column.
    // Seed 0 overworld terrain around the origin includes both ocean and land chunks.
    let chunk = 'found: {
        for cz in 0..4_i32 {
            for cx in 0..4_i32 {
                let c = fill_noise_and_build_surface(
                    ChunkPos { x: cx, z: cz },
                    &BiomeSourceModel::Fixed {
                        biome: "minecraft:plains",
                    },
                    settings,
                    0,
                    noise_router,
                    &rule,
                );
                if chunk_has_land_column(&c, min_y, max_y, sea_level) {
                    break 'found c;
                }
            }
        }
        panic!("no land column found in any of the 16 chunks near the origin — terrain generation may be broken");
    };

    let land_column = first_land_column_top(&chunk, min_y, max_y, sea_level)
        .expect("a land column must exist in the chunk we selected");

    let (bx, top_y, bz, top_block_name) = land_column;

    // The surface block should be grass for the fixed-plains biome source.
    assert_eq!(
        top_block_name, "minecraft:grass_block",
        "top solid block at ({bx},{top_y},{bz}) should be grass_block (got {top_block_name})"
    );

    // The block directly below should be dirt.
    let below = chunk.get_block_state(bx, top_y - 1, bz).unwrap_or_default();
    assert_eq!(
        below,
        "minecraft:dirt",
        "block below grass at ({bx},{},{bz}) should be dirt (got {below})",
        top_y - 1
    );

    // The exact dirt-depth depends on vanilla surface noise, but the column
    // should transition back to stone below the generated soil layer.
    let has_stone_below_soil = ((min_y + 1)..=(top_y - 2)).rev().any(|y| {
        chunk
            .get_block_state(bx, y, bz)
            .is_some_and(|block| block == "minecraft:stone")
    });
    assert!(
        has_stone_below_soil,
        "column at ({bx},{bz}) should contain stone below the plains soil layer"
    );
}

#[test]
fn overworld_surface_rule_has_builtin_fallback_without_decompiled_json() {
    use super::super::{
        builtin_noise_generator_settings, builtin_noise_router, fill_noise_and_build_surface,
        load_surface_rule_uncached, noise_router_id_for_settings, ChunkPos, NONE_NOISE_ROUTER,
    };
    use crate::biome::BiomeSourceModel;

    let settings = builtin_noise_generator_settings("minecraft:overworld")
        .expect("overworld noise settings must exist");
    let router_id = noise_router_id_for_settings(*settings);
    let noise_router = builtin_noise_router(router_id)
        .map(|e| e.router)
        .unwrap_or(NONE_NOISE_ROUTER);
    let rule = load_surface_rule_uncached("minecraft:overworld", None)
        .expect("overworld surface rule must fall back when optional JSON is absent");

    let sea_level = settings.sea_level;
    let min_y = settings.noise.min_y;
    let max_y = min_y + settings.noise.height - 1;
    let chunk = 'found: {
        for cz in 0..4_i32 {
            for cx in 0..4_i32 {
                let chunk = fill_noise_and_build_surface(
                    ChunkPos { x: cx, z: cz },
                    &BiomeSourceModel::Fixed {
                        biome: "minecraft:plains",
                    },
                    settings,
                    0,
                    noise_router,
                    &rule,
                );
                if chunk_has_land_column(&chunk, min_y, max_y, sea_level) {
                    break 'found chunk;
                }
            }
        }
        panic!("fallback surface rule did not produce any land columns near the origin");
    };

    let (block_x, top_y, block_z, top_block_name) =
        first_land_column_top(&chunk, min_y, max_y, sea_level)
            .expect("selected fallback chunk must contain land");
    assert_eq!(
        top_block_name, "minecraft:grass_block",
        "fallback top solid block at ({block_x},{top_y},{block_z}) should not remain bare stone"
    );
}

#[test]
fn material_rule_sources_evaluate_surface_conditions_in_vanilla_order() {
    assert_material_rule_source_registries();
    let heights = sample_surface_rule_heights();
    let plains_surface = sample_plains_surface_context();
    let quiet_desert_floor =
        assert_surface_rule_sequence_matches_vanilla_order(&heights, plains_surface);
    assert_y_above_condition(&heights, quiet_desert_floor);
    assert_water_conditions(&heights, quiet_desert_floor);
    assert_dynamic_water_condition(&heights, quiet_desert_floor);
    assert_stone_depth_conditions(&heights, quiet_desert_floor);
    assert_vertical_gradient_conditions(&heights, quiet_desert_floor);
    assert_bandlands_material_rule(&heights, quiet_desert_floor);
}

static PLAINS_OR_FOREST: SurfaceConditionSource =
    SurfaceConditionSource::Biome(&["minecraft:plains", "minecraft:forest"]);
static DRY_NOISE: SurfaceConditionSource = SurfaceConditionSource::NoiseThreshold {
    min: -0.25,
    max: 0.25,
};
static NOT_DRY_NOISE: SurfaceConditionSource = SurfaceConditionSource::Not(&DRY_NOISE);
static FLOOR: SurfaceConditionSource = SurfaceConditionSource::StoneDepth {
    offset: 0,
    add_surface_depth: true,
    secondary_depth_range: 0,
    surface: CaveSurface::Floor,
};
static GRASS: SurfaceRuleSource = SurfaceRuleSource::Block("minecraft:grass_block");
static DIRT: SurfaceRuleSource = SurfaceRuleSource::Block("minecraft:dirt");
static STONE: SurfaceRuleSource = SurfaceRuleSource::Block("minecraft:stone");
static PLAINS_GRASS: SurfaceRuleSource = SurfaceRuleSource::Condition {
    condition: &PLAINS_OR_FOREST,
    rule: &GRASS,
};
static DRY_STONE: SurfaceRuleSource = SurfaceRuleSource::Condition {
    condition: &NOT_DRY_NOISE,
    rule: &STONE,
};
static FLOOR_DIRT: SurfaceRuleSource = SurfaceRuleSource::Condition {
    condition: &FLOOR,
    rule: &DIRT,
};
static RULES: &[SurfaceRuleSource] = &[PLAINS_GRASS, DRY_STONE, FLOOR_DIRT];
static SEQUENCE: SurfaceRuleSource = SurfaceRuleSource::Sequence(RULES);

fn assert_material_rule_source_registries() {
    assert!(super::super::surface_rule_type("block").is_some());
    assert!(super::super::surface_rule_type("sequence").is_some());
    assert!(super::super::surface_rule_type("condition").is_some());
    assert!(super::super::surface_condition_type("biome").is_some());
    assert!(super::super::surface_condition_type("noise_threshold").is_some());
    assert!(super::super::surface_condition_type("stone_depth").is_some());
}

fn sample_surface_rule_heights() -> WorldGenerationHeightContext {
    WorldGenerationHeightContext {
        min_y: -64,
        height: 384,
    }
}

fn sample_plains_surface_context() -> SurfaceMaterialContext {
    SurfaceMaterialContext {
        seed: 12345,
        random_algorithm: super::super::RandomAlgorithm::Xoroshiro,
        x: 12,
        y: 64,
        z: -4,
        biome: "minecraft:plains",
        stone_depth_above: 0,
        stone_depth_below: 3,
        surface_depth: 3,
        preliminary_surface_y: 62,
        water_height: 63,
        temperature: 0.8,
        noise: 0.0,
        steep: false,
        hole: false,
    }
}

fn assert_surface_rule_sequence_matches_vanilla_order(
    heights: &WorldGenerationHeightContext,
    plains_surface: SurfaceMaterialContext,
) -> SurfaceMaterialContext {
    assert_eq!(
        super::super::surface_rule_apply(&SEQUENCE, &plains_surface, heights),
        Some("minecraft:grass_block")
    );

    let noisy_desert = SurfaceMaterialContext {
        biome: "minecraft:desert",
        noise: 0.6,
        ..plains_surface
    };
    assert_eq!(
        super::super::surface_rule_apply(&SEQUENCE, &noisy_desert, heights),
        Some("minecraft:stone")
    );

    let quiet_desert_floor = SurfaceMaterialContext {
        biome: "minecraft:desert",
        noise: 0.0,
        stone_depth_above: 2,
        ..plains_surface
    };
    assert_eq!(
        super::super::surface_rule_apply(&SEQUENCE, &quiet_desert_floor, heights),
        Some("minecraft:dirt")
    );
    quiet_desert_floor
}

fn assert_y_above_condition(
    heights: &WorldGenerationHeightContext,
    quiet_desert_floor: SurfaceMaterialContext,
) {
    assert!(super::super::surface_condition_test(
        &SurfaceConditionSource::YAbove {
            anchor: VerticalAnchor::Absolute(59),
            surface_depth_multiplier: 1,
            add_stone_depth: true,
        },
        &quiet_desert_floor,
        heights
    ));
}

fn assert_water_conditions(
    heights: &WorldGenerationHeightContext,
    quiet_desert_floor: SurfaceMaterialContext,
) {
    assert!(super::super::surface_condition_test(
        &SurfaceConditionSource::Water {
            offset: 2,
            surface_depth_multiplier: 0,
            add_stone_depth: false,
        },
        &SurfaceMaterialContext {
            y: 65,
            ..quiet_desert_floor
        },
        heights
    ));
    assert!(!super::super::surface_condition_test(
        &SurfaceConditionSource::Water {
            offset: 2,
            surface_depth_multiplier: 0,
            add_stone_depth: false,
        },
        &quiet_desert_floor,
        heights
    ));
    assert!(super::super::surface_condition_test(
        &SurfaceConditionSource::Water {
            offset: 0,
            surface_depth_multiplier: 0,
            add_stone_depth: false,
        },
        &SurfaceMaterialContext {
            water_height: i32::MIN,
            ..quiet_desert_floor
        },
        heights
    ));
    assert!(super::super::surface_condition_test(
        &SurfaceConditionSource::Water {
            offset: 0,
            surface_depth_multiplier: 0,
            add_stone_depth: true,
        },
        &SurfaceMaterialContext {
            y: 61,
            water_height: 63,
            stone_depth_above: 2,
            ..quiet_desert_floor
        },
        heights
    ));
    assert!(!super::super::surface_condition_test(
        &SurfaceConditionSource::Water {
            offset: 1,
            surface_depth_multiplier: 1,
            add_stone_depth: true,
        },
        &SurfaceMaterialContext {
            y: 63,
            water_height: 63,
            surface_depth: 2,
            stone_depth_above: 2,
            ..quiet_desert_floor
        },
        heights
    ));
}

fn assert_stone_depth_conditions(
    heights: &WorldGenerationHeightContext,
    quiet_desert_floor: SurfaceMaterialContext,
) {
    assert!(super::super::surface_condition_test(
        &SurfaceConditionSource::StoneDepth {
            offset: 0,
            add_surface_depth: true,
            secondary_depth_range: 6,
            surface: CaveSurface::Ceiling,
        },
        &SurfaceMaterialContext {
            stone_depth_below: 7,
            noise: 0.0,
            ..quiet_desert_floor
        },
        heights
    ));
    assert!(!super::super::surface_condition_test(
        &SurfaceConditionSource::StoneDepth {
            offset: 0,
            add_surface_depth: false,
            secondary_depth_range: 0,
            surface: CaveSurface::Ceiling,
        },
        &SurfaceMaterialContext {
            stone_depth_below: 2,
            ..quiet_desert_floor
        },
        heights
    ));
}

fn assert_dynamic_water_condition(
    heights: &WorldGenerationHeightContext,
    quiet_desert_floor: SurfaceMaterialContext,
) {
    let dynamic_water_state = dynamic_surface_state_from_context(
        *heights,
        quiet_desert_floor,
        DynamicSurfacePosition {
            block_y: 61,
            water_height: 63,
            surface_depth: 0,
            stone_depth_above: 2,
        },
    );
    assert!(super::super::dyn_surface_condition_test(
        &super::super::DynSurfaceCondition::Water {
            offset: 0,
            surface_depth_multiplier: 0,
            add_stone_depth: true,
        },
        &dynamic_water_state,
        BUILTIN_NOISE_GENERATOR_SETTINGS[0],
    ));
}

#[derive(Clone, Copy)]
struct DynamicSurfacePosition {
    block_y: i32,
    water_height: i32,
    surface_depth: i32,
    stone_depth_above: i32,
}

fn dynamic_surface_state_from_context(
    heights: WorldGenerationHeightContext,
    surface: SurfaceMaterialContext,
    position: DynamicSurfacePosition,
) -> super::super::BuildSurfaceColumnState {
    super::super::BuildSurfaceColumnState {
        seed: surface.seed,
        algorithm: surface.random_algorithm,
        heights,
        last_update_xz: 1,
        last_update_y: 1,
        condition_cache: std::cell::RefCell::new(std::collections::HashMap::new()),
        profile: None,
        block_x: surface.x,
        block_z: surface.z,
        surface_depth: position.surface_depth,
        surface_secondary: surface.noise,
        steep: surface.steep,
        hole: surface.hole,
        min_surface_level: surface.preliminary_surface_y,
        block_y: position.block_y,
        water_height: position.water_height,
        stone_depth_above: position.stone_depth_above,
        stone_depth_below: surface.stone_depth_below,
        biome: surface.biome,
        temperature: surface.temperature,
        biome_needs_update: false,
    }
}

fn assert_vertical_gradient_conditions(
    heights: &WorldGenerationHeightContext,
    quiet_desert_floor: SurfaceMaterialContext,
) {
    let vertical_gradient = SurfaceConditionSource::VerticalGradient {
        random_name: "minecraft:bedrock_floor",
        true_at_and_below: VerticalAnchor::Absolute(60),
        false_at_and_above: VerticalAnchor::Absolute(70),
    };
    assert!(super::super::surface_condition_test(
        &vertical_gradient,
        &SurfaceMaterialContext {
            y: 60,
            noise: 1.0,
            ..quiet_desert_floor
        },
        heights
    ));
    assert!(!super::super::surface_condition_test(
        &vertical_gradient,
        &SurfaceMaterialContext {
            y: 70,
            noise: -1.0,
            ..quiet_desert_floor
        },
        heights
    ));
    let random_float = super::super::surface_positional_random_float(
        quiet_desert_floor.seed,
        quiet_desert_floor.random_algorithm,
        "minecraft:bedrock_floor",
        quiet_desert_floor.x,
        65,
        quiet_desert_floor.z,
    );
    assert!((random_float - 0.96467084).abs() < f32::EPSILON);
    assert!(!super::super::surface_condition_test(
        &vertical_gradient,
        &SurfaceMaterialContext {
            y: 65,
            noise: -1.0,
            ..quiet_desert_floor
        },
        heights
    ));
    assert!(!super::super::dyn_surface_condition_test(
        &super::super::DynSurfaceCondition::VerticalGradient {
            random_name: "minecraft:bedrock_floor".to_string(),
            true_at_and_below: VerticalAnchor::Absolute(60),
            false_at_and_above: VerticalAnchor::Absolute(70),
        },
        &dynamic_surface_state_from_context(
            *heights,
            quiet_desert_floor,
            DynamicSurfacePosition {
                block_y: 65,
                water_height: 63,
                surface_depth: 0,
                stone_depth_above: 2,
            },
        ),
        BUILTIN_NOISE_GENERATOR_SETTINGS[0],
    ));
}

fn assert_bandlands_material_rule(
    heights: &WorldGenerationHeightContext,
    quiet_desert_floor: SurfaceMaterialContext,
) {
    assert_eq!(
        super::super::surface_rule_apply(
            &SurfaceRuleSource::Bandlands,
            &quiet_desert_floor,
            heights
        ),
        Some("minecraft:red_sand")
    );
}
