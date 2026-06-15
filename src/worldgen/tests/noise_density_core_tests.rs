use super::*;

const TERRAIN_PROVIDER_JAVA: &str = vibecraft_java_source!("/net/minecraft/data/worldgen/TerrainProvider.java");
const CUBIC_SPLINE_JAVA: &str =
    vibecraft_java_source!("/net/minecraft/util/CubicSpline.java");

fn occurrence_count(source: &str, needle: &str) -> usize {
    source.matches(needle).count()
}

fn assert_close(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() < 1e-12,
        "expected {expected}, got {actual}"
    );
}

#[test]
fn noise_generator_settings_bootstrap_matches_vanilla_order_and_flags() {
    assert_eq!(
        BUILTIN_NOISE_GENERATOR_SETTINGS
            .iter()
            .map(|settings| settings.id)
            .collect::<Vec<_>>(),
        vec![
            "minecraft:overworld",
            "minecraft:large_biomes",
            "minecraft:amplified",
            "minecraft:nether",
            "minecraft:end",
            "minecraft:caves",
            "minecraft:floating_islands",
        ]
    );

    let overworld = builtin_noise_generator_settings("overworld").unwrap();
    assert_eq!(overworld.default_block, "minecraft:stone");
    assert_eq!(overworld.default_fluid, "minecraft:water");
    assert_eq!(
        overworld.noise_router,
        NoiseRouterPreset::Overworld {
            large_biomes: false,
            amplified: false
        }
    );
    assert_eq!(overworld.surface_rule, SurfaceRulePreset::Overworld);
    assert_eq!(overworld.sea_level, 63);
    assert!(overworld.aquifers_enabled);
    assert!(overworld.ore_veins_enabled);
    assert!(!overworld.legacy_random_source);

    let large = builtin_noise_generator_settings("large_biomes").unwrap();
    assert_eq!(
        large.noise_router,
        NoiseRouterPreset::Overworld {
            large_biomes: true,
            amplified: false
        }
    );
    let amplified = builtin_noise_generator_settings("amplified").unwrap();
    assert_eq!(
        amplified.noise_router,
        NoiseRouterPreset::Overworld {
            large_biomes: false,
            amplified: true
        }
    );

    let nether = builtin_noise_generator_settings("nether").unwrap();
    assert_eq!(nether.default_block, "minecraft:netherrack");
    assert_eq!(nether.default_fluid, "minecraft:lava");
    assert_eq!(nether.sea_level, 32);
    assert!(nether.legacy_random_source);

    let end = builtin_noise_generator_settings("end").unwrap();
    assert_eq!(end.default_block, "minecraft:end_stone");
    assert_eq!(end.default_fluid, "minecraft:air");
    assert!(end.disable_mob_generation);
    assert_eq!(end.sea_level, 0);
}

#[test]
fn noise_generator_settings_cover_extracted_registry_jsons() {
    assert_eq!(
        super::super::EXTRACTED_NOISE_SETTINGS_REGISTRY_EXPECTATIONS
            .iter()
            .map(|entry| entry.id)
            .collect::<Vec<_>>(),
        vec![
            "minecraft:amplified",
            "minecraft:caves",
            "minecraft:end",
            "minecraft:floating_islands",
            "minecraft:large_biomes",
            "minecraft:nether",
            "minecraft:overworld",
        ]
    );
    assert_eq!(
        super::super::EXTRACTED_NOISE_SETTINGS_REGISTRY_EXPECTATIONS.len(),
        BUILTIN_NOISE_GENERATOR_SETTINGS.len()
    );

    for expected in super::super::EXTRACTED_NOISE_SETTINGS_REGISTRY_EXPECTATIONS {
        let settings = builtin_noise_generator_settings(expected.id)
            .unwrap_or_else(|| panic!("missing {}", expected.id));
        assert_eq!(settings.noise, expected.noise, "{}", expected.id);
        assert_eq!(
            settings.default_block, expected.default_block,
            "{}",
            expected.id
        );
        assert_eq!(
            settings.default_fluid, expected.default_fluid,
            "{}",
            expected.id
        );
        assert_eq!(
            super::super::noise_router_id_for_settings(*settings),
            expected.router_id,
            "{}",
            expected.id
        );
        assert_eq!(
            settings.surface_rule, expected.surface_rule,
            "{}",
            expected.id
        );
        assert_eq!(
            settings.spawn_target.len(),
            expected.spawn_target_len,
            "{}",
            expected.id
        );
        assert_eq!(settings.sea_level, expected.sea_level, "{}", expected.id);
        assert_eq!(
            settings.disable_mob_generation, expected.disable_mob_generation,
            "{}",
            expected.id
        );
        assert_eq!(
            settings.aquifers_enabled, expected.aquifers_enabled,
            "{}",
            expected.id
        );
        assert_eq!(
            settings.ore_veins_enabled, expected.ore_veins_enabled,
            "{}",
            expected.id
        );
        assert_eq!(
            settings.legacy_random_source, expected.legacy_random_source,
            "{}",
            expected.id
        );
        assert!(builtin_noise_router(expected.router_id).is_some());
    }
}

#[test]
fn overworld_spawn_target_matches_overworld_biome_builder() {
    assert_eq!(OVERWORLD_SPAWN_TARGET.len(), 2);
    assert_eq!(
        OVERWORLD_SPAWN_TARGET[0].continentalness.min,
        quantize_coord(-0.11)
    );
    assert_eq!(
        OVERWORLD_SPAWN_TARGET[0].continentalness.max,
        quantize_coord(1.0)
    );
    assert_eq!(
        OVERWORLD_SPAWN_TARGET[0].weirdness.min,
        quantize_coord(-1.0)
    );
    assert_eq!(
        OVERWORLD_SPAWN_TARGET[0].weirdness.max,
        quantize_coord(-0.16)
    );
    assert_eq!(
        OVERWORLD_SPAWN_TARGET[1].weirdness.min,
        quantize_coord(0.16)
    );
    assert_eq!(OVERWORLD_SPAWN_TARGET[1].weirdness.max, quantize_coord(1.0));
}

#[test]
fn density_function_type_registry_matches_densityfunctions_bootstrap_order() {
    assert_eq!(DENSITY_FUNCTION_TYPES.len(), 34);
    assert_eq!(
        DENSITY_FUNCTION_TYPES
            .iter()
            .map(|kind| kind.id)
            .collect::<Vec<_>>(),
        vec![
            "blend_alpha",
            "blend_offset",
            "beardifier",
            "old_blended_noise",
            "interpolated",
            "flat_cache",
            "cache_2d",
            "cache_once",
            "cache_all_in_cell",
            "noise",
            "end_islands",
            "weird_scaled_sampler",
            "shifted_noise",
            "range_choice",
            "shift_a",
            "shift_b",
            "shift",
            "blend_density",
            "clamp",
            "abs",
            "square",
            "cube",
            "half_negative",
            "quarter_negative",
            "invert",
            "squeeze",
            "add",
            "mul",
            "min",
            "max",
            "spline",
            "constant",
            "y_clamped_gradient",
            "find_top_surface",
        ]
    );
    assert!(density_function_type("shifted_noise").is_some());
    assert!(density_function_type("missing").is_none());
}

#[test]
fn cache_all_in_cell_uses_wrapper_owned_cell_values_for_scalar_and_array_reads() {
    let settings = *builtin_noise_generator_settings("overworld").unwrap();
    let router = super::super::NoiseRouter {
        barrier: DensityFunction::Constant(0.0),
        fluid_level_floodedness: DensityFunction::Constant(0.0),
        fluid_level_spread: DensityFunction::Constant(0.0),
        lava: DensityFunction::Constant(0.0),
        temperature: DensityFunction::Constant(0.0),
        vegetation: DensityFunction::Constant(0.0),
        continents: DensityFunction::Constant(0.0),
        erosion: DensityFunction::Constant(0.0),
        depth: DensityFunction::Constant(0.0),
        ridges: DensityFunction::Constant(0.0),
        preliminary_surface_level: DensityFunction::Constant(0.0),
        final_density: super::super::TEST_CACHE_ALL_IN_CELL_DENSITY,
        vein_toggle: DensityFunction::Constant(0.0),
        vein_ridged: DensityFunction::Constant(0.0),
        vein_gap: DensityFunction::Constant(0.0),
    };
    let mut chunk = super::super::NoiseChunk::new(0, 0, settings, 0, router);

    chunk.advance_cell_x(0);
    chunk.select_cell_yz(0, 0);
    chunk.update_for_x(0, 0.0);
    chunk.update_for_y(settings.noise.min_y, 0.0);
    chunk.update_for_z(0, 0.0);

    let value_index = chunk
        .cache_all_cell_index()
        .expect("selected block should be inside the current noise cell");
    assert_eq!(chunk.cache_all_in_cell.len(), 1);
    chunk.cache_all_in_cell[0].values[value_index] = 42.25;

    assert_eq!(
        super::super::eval_density_fn_with_interp(
            super::super::TEST_CACHE_ALL_IN_CELL_DENSITY,
            &chunk,
            0,
            settings.noise.min_y,
            0,
        ),
        42.25,
        "CacheAllInCell scalar compute should read the wrapper-owned cell cache"
    );

    let mut output = vec![0.0; chunk.cache_all_in_cell[0].values.len()];
    super::super::fill_density_array_with_interp(
        super::super::TEST_CACHE_ALL_IN_CELL_DENSITY,
        &mut chunk,
        &mut output,
        super::super::DensityArrayFillMode::Cell,
    );
    assert_eq!(
        output, chunk.cache_all_in_cell[0].values,
        "CacheAllInCell fillArray should copy the wrapper-owned cell cache"
    );
}

#[test]
fn density_function_core_evaluators_follow_vanilla_transform_rules() {
    assert_y_density_and_mapped_transforms();
    assert_binary_density_function_rules();
    assert_find_top_surface_density_rules();
    assert_sampler_value_bounds();
    assert_range_choice_density_rules();
}

fn assert_y_density_and_mapped_transforms() {
    assert_eq!(Y_DENSITY.compute(-5000), -4064.0);
    assert_eq!(Y_DENSITY.compute(5000), 4062.0);
    assert_eq!(Y_DENSITY.compute(0), 0.0);

    assert_eq!(MappedDensityFunction::Abs.transform(-2.0), 2.0);
    assert_eq!(MappedDensityFunction::Square.transform(-2.0), 4.0);
    assert_eq!(MappedDensityFunction::Cube.transform(-2.0), -8.0);
    assert_eq!(MappedDensityFunction::HalfNegative.transform(-2.0), -1.0);
    assert_eq!(MappedDensityFunction::QuarterNegative.transform(-2.0), -0.5);
    assert_eq!(MappedDensityFunction::Invert.transform(2.0), 0.5);
    assert!((MappedDensityFunction::Squeeze.transform(1.0) - 0.4583333333333333).abs() < 1e-12);
}

fn assert_binary_density_function_rules() {
    let add = DensityFunction::Binary {
        kind: BinaryDensityFunction::Add,
        argument1: &TEST_NEGATIVE_DENSITY,
        argument2: &TEST_POSITIVE_DENSITY,
    };
    assert_eq!(add.compute(0), 1.0);
    assert_eq!(BinaryDensityFunction::Mul.apply(-2.0, 3.0), -6.0);
    assert_eq!(BinaryDensityFunction::Min.apply(-2.0, 3.0), -2.0);
    assert_eq!(BinaryDensityFunction::Max.apply(-2.0, 3.0), 3.0);
    assert_eq!(super::super::SHIFT_A_DENSITY.type_name(), "shift_a");
    assert_eq!(super::super::SHIFT_B_DENSITY.type_name(), "shift_b");
    let shift_bounds = super::super::SHIFT_A_DENSITY.value_bounds();
    let shift_noise_bounds = super::super::normal_noise_value_bounds("minecraft:offset").unwrap();
    assert_eq!(
        shift_bounds,
        (shift_noise_bounds.0 * 4.0, shift_noise_bounds.1 * 4.0)
    );
    assert_eq!(
        BinaryDensityFunction::Mul.apply_lazy(0.0, (3.0, 3.0), || panic!(
            "mul should skip zero second argument"
        )),
        0.0
    );
    assert_eq!(
        BinaryDensityFunction::Min.apply_lazy(-5.0, (3.0, 3.0), || panic!(
            "min should skip higher second argument"
        )),
        -5.0
    );
    assert_eq!(
        BinaryDensityFunction::Max.apply_lazy(5.0, (-3.0, -3.0), || panic!(
            "max should skip lower second argument"
        )),
        5.0
    );
    assert_eq!(Y_DENSITY.value_bounds(), (-4064.0, 4062.0));
    assert_eq!(
        DensityFunction::Binary {
            kind: BinaryDensityFunction::Add,
            argument1: &TEST_NEGATIVE_DENSITY,
            argument2: &TEST_POSITIVE_DENSITY,
        }
        .value_bounds(),
        (1.0, 1.0)
    );
}

fn assert_find_top_surface_density_rules() {
    assert_eq!(
        super::super::find_top_surface_compute(
            |sample_y| if sample_y <= 72 { 0.25 } else { -0.25 },
            83.9,
            -64,
            4,
        ),
        72.0
    );
    assert_eq!(
        super::super::find_top_surface_compute(|_| -0.25, -80.0, -64, 4),
        -64.0
    );
    assert_eq!(
        DensityFunction::FindTopSurface {
            density: &TEST_POSITIVE_DENSITY,
            upper_bound: &TEST_POSITIVE_DENSITY,
            lower_bound: -64,
            cell_height: 4,
        }
        .type_name(),
        "find_top_surface"
    );
    assert_eq!(
        DensityFunction::FindTopSurface {
            density: &TEST_POSITIVE_DENSITY,
            upper_bound: &TEST_POSITIVE_DENSITY,
            lower_bound: -64,
            cell_height: 4,
        }
        .value_bounds(),
        (-64.0, 3.0)
    );
}

fn assert_sampler_value_bounds() {
    assert_eq!(super::super::RarityValueMapper::Type1.max_rarity(), 2.0);
    assert_eq!(super::super::RarityValueMapper::Type2.max_rarity(), 3.0);
    let weird_bounds = DensityFunction::WeirdScaledSampler {
        input: &TEST_POSITIVE_DENSITY,
        noise: "minecraft:spaghetti_3d_1",
        rarity_mapper: super::super::RarityValueMapper::Type1,
    }
    .value_bounds();
    assert_eq!(weird_bounds.0, 0.0);
    assert!(weird_bounds.1.is_finite());
    assert!(weird_bounds.1 > 0.0);
    assert_eq!(
        DensityFunction::Noise {
            noise: "minecraft:temperature",
            xz_scale: 0.25,
            y_scale: 0.0,
        }
        .value_bounds(),
        (-4.444444444444445, 4.444444444444445)
    );
    assert_eq!(
        DensityFunction::ShiftedNoise {
            shift_x: &super::super::SHIFT_X_DENSITY,
            shift_y: &super::super::ZERO_DENSITY,
            shift_z: &super::super::SHIFT_Z_DENSITY,
            xz_scale: 0.25,
            y_scale: 0.0,
            noise: "minecraft:temperature",
        }
        .value_bounds(),
        (-4.444444444444445, 4.444444444444445)
    );
}

fn assert_range_choice_density_rules() {
    assert_eq!(
        super::super::TEST_RANGE_CHOICE_DENSITY.type_name(),
        "range_choice"
    );
    assert_eq!(super::super::TEST_RANGE_CHOICE_DENSITY.compute(0), 3.0);
    assert_eq!(super::super::TEST_RANGE_CHOICE_DENSITY.compute(-64), -2.0);
    assert_eq!(super::super::TEST_RANGE_CHOICE_DENSITY.compute(64), -2.0);
    let overworld = *super::super::builtin_noise_generator_settings("overworld").unwrap();
    assert_eq!(
        super::super::TEST_RANGE_CHOICE_DENSITY.compute_with_noise(12345, overworld, 0, 0, 0),
        3.0
    );
    assert_eq!(
        super::super::TEST_RANGE_CHOICE_DENSITY.compute_with_noise(12345, overworld, 0, 64, 0),
        -2.0
    );
}

#[test]
fn density_function_noise_evaluators_resolve_random_state_noise_holders() {
    let overworld = *super::super::builtin_noise_generator_settings("overworld").unwrap();
    let noise = DensityFunction::Noise {
        noise: "minecraft:temperature",
        xz_scale: 0.25,
        y_scale: 0.0,
    };
    let shifted = DensityFunction::ShiftedNoise {
        shift_x: &super::super::SHIFT_X_DENSITY,
        shift_y: &super::super::ZERO_DENSITY,
        shift_z: &super::super::SHIFT_Z_DENSITY,
        xz_scale: 0.25,
        y_scale: 0.0,
        noise: "minecraft:temperature",
    };
    let weird = DensityFunction::WeirdScaledSampler {
        input: &TEST_POSITIVE_DENSITY,
        noise: "minecraft:spaghetti_3d_1",
        rarity_mapper: super::super::RarityValueMapper::Type1,
    };

    let noise_value = noise.compute_with_noise(12345, overworld, 16, 64, -32);
    let shifted_value = shifted.compute_with_noise(12345, overworld, 16, 64, -32);
    let weird_value = weird.compute_with_noise(12345, overworld, 16, 64, -32);
    let shift_a_value =
        super::super::SHIFT_A_DENSITY.compute_with_noise(12345, overworld, 16, 64, -32);
    assert!((noise_value - -0.02846337681055331).abs() < 1e-12);
    assert!((shifted_value - -0.02854944566757223).abs() < 1e-12);
    assert!((weird_value - 0.5833159524778098).abs() < 1e-12);
    assert!(
        (shift_a_value
            - super::super::density_shift_noise_sample(
                12345,
                overworld,
                "minecraft:offset",
                16.0,
                0.0,
                -32.0,
            ))
        .abs()
            < 1e-12
    );

    assert_eq!(
        super::super::RarityValueMapper::Type1.map_value(-0.75),
        0.75
    );
    assert_eq!(super::super::RarityValueMapper::Type1.map_value(0.25), 1.5);
    assert_eq!(super::super::RarityValueMapper::Type2.map_value(-0.8), 0.5);
    assert_eq!(super::super::RarityValueMapper::Type2.map_value(0.8), 3.0);
}

#[test]
fn blended_noise_evaluator_uses_vanilla_legacy_octave_stack() {
    let overworld = *super::super::builtin_noise_generator_settings("overworld").unwrap();
    let snapshot = super::super::blended_noise_snapshot(
        super::super::random_state_terrain_random(12345, overworld),
        0.25,
        0.125,
        80.0,
        160.0,
        8.0,
    )
    .unwrap();
    assert_eq!(snapshot.min_limit_noise.levels.len(), 16);
    assert_eq!(snapshot.max_limit_noise.levels.len(), 16);
    assert_eq!(snapshot.main_noise.levels.len(), 8);
    assert!(snapshot.min_limit_noise.levels.iter().all(Option::is_some));
    assert!(snapshot.max_limit_noise.levels.iter().all(Option::is_some));
    assert!(snapshot.main_noise.levels.iter().all(Option::is_some));
    assert!((snapshot.max_value - super::super::blended_noise_max_value(0.125)).abs() < 1e-12);
    assert_eq!(
        super::super::BASE_3D_NOISE_OVERWORLD_DENSITY.value_bounds(),
        (-snapshot.max_value, snapshot.max_value)
    );

    let sample = super::super::blended_noise_sample(&snapshot, 16.0, 64.0, -32.0);
    let density_sample = super::super::BASE_3D_NOISE_OVERWORLD_DENSITY
        .compute_with_noise(12345, overworld, 16, 64, -32);
    assert!((sample - -0.12125330841368123).abs() < 1e-12);
    assert!((sample - density_sample).abs() < 1e-12);

    let nether = *super::super::builtin_noise_generator_settings("nether").unwrap();
    let nether_sample =
        super::super::BASE_3D_NOISE_NETHER_DENSITY.compute_with_noise(12345, nether, 16, 64, -32);
    assert!((nether_sample - 0.287_241_937_427_688_8).abs() < 1e-12);
}

#[test]
fn end_island_density_uses_seeded_simplex_height_scan() {
    let mut random = super::super::RandomSourceKind::Legacy(super::super::LegacyRandom::new(12345));
    random.consume_count(17_292);
    let simplex = super::super::simplex_noise_snapshot(&mut random);
    assert!((simplex.xo - 217.28203870227557).abs() < 1e-12);
    assert!((simplex.yo - 16.202358521842683).abs() < 1e-12);
    assert!((simplex.zo - 80.38840973560625).abs() < 1e-12);
    assert_eq!(
        &simplex.permutation[0..8],
        &[133, 54, 101, 16, 13, 4, 149, 66]
    );

    let simplex_value = super::super::simplex_noise_sample_2d(&simplex, 65.0, -71.0);
    let height = super::super::end_island_height_value(&simplex, 16 / 8, -32 / 8);
    let density = super::super::end_island_density_sample(12345, 16, -32);
    let end = *super::super::builtin_noise_generator_settings("end").unwrap();
    let density_function_value =
        super::super::END_ISLANDS_DENSITY.compute_with_noise(12345, end, 16, 64, -32);
    assert!((simplex_value - -0.40209723583721246).abs() < 1e-12);
    assert!((height - 64.222916).abs() < 1e-5);
    assert!((density - 0.43924152851104736).abs() < 1e-12);
    assert!((density - density_function_value).abs() < 1e-12);
}

#[test]
fn simplex_noise_2d_sampling_covers_both_skew_branches() {
    let mut random = super::super::RandomSourceKind::Legacy(super::super::LegacyRandom::new(12345));
    let simplex = super::super::simplex_noise_snapshot(&mut random);
    let lower_triangle = super::super::simplex_noise_sample_2d(&simplex, 1.25, -3.5);
    let upper_triangle = super::super::simplex_noise_sample_2d(&simplex, -3.5, 1.25);
    assert!((lower_triangle - -0.18271295126292367).abs() < 1e-12);
    assert!((upper_triangle - -0.45718095628667355).abs() < 1e-12);
}

#[test]
fn simplex_noise_3d_sampling_matches_vanilla_corner_path() {
    let mut random = super::super::RandomSourceKind::Legacy(super::super::LegacyRandom::new(12345));
    let simplex = super::super::simplex_noise_snapshot(&mut random);
    let first = super::super::simplex_noise_sample_3d(&simplex, 1.25, -3.5, 8.75);
    let second = super::super::simplex_noise_sample_3d(&simplex, -12.125, 0.5, 33.25);
    assert!((first - 0.124169920267489).abs() < 1e-12);
    assert!((second - 0.29129930814264227).abs() < 1e-12);
}

#[test]
fn simplex_noise_3d_sampling_covers_all_rank_order_corner_paths() {
    let mut random = super::super::RandomSourceKind::Legacy(super::super::LegacyRandom::new(12345));
    let simplex = super::super::simplex_noise_snapshot(&mut random);
    let samples = [
        ((-5.0, -4.5, -2.75), -0.2787272376543217),
        ((-5.0, -4.75, -2.5), 0.16925000482253008),
        ((-5.0, -4.5, -3.75), -0.17981134259259166),
        ((-5.0, -4.75, -4.5), -0.05465644531249998),
        ((-5.0, -4.5, -4.75), 0.1639693359374999),
        ((-5.0, -4.75, -4.25), 0.07816971450617194),
    ];
    for ((x, y, z), expected) in samples {
        let actual = super::super::simplex_noise_sample_3d(&simplex, x, y, z);
        assert!(
            (actual - expected).abs() < 1e-12,
            "simplex sample at ({x}, {y}, {z})"
        );
    }
}

#[test]
fn terrain_provider_java_source_shape_matches_rust_spline_model() {
    assert_eq!(TERRAIN_PROVIDER_JAVA.lines().count(), 310);
    for sentinel in [
        "DEEP_OCEAN_CONTINENTALNESS = -0.51F",
        "OCEAN_CONTINENTALNESS = -0.4F",
        "PLAINS_CONTINENTALNESS = 0.1F",
        "BEACH_CONTINENTALNESS = -0.15F",
        "AMPLIFIED_OFFSET = BoundedFloatFunction.createUnlimited",
        "AMPLIFIED_FACTOR = BoundedFloatFunction.createUnlimited",
        "AMPLIFIED_JAGGEDNESS = BoundedFloatFunction.createUnlimited",
    ] {
        assert!(
            TERRAIN_PROVIDER_JAVA.contains(sentinel),
            "missing TerrainProvider sentinel {sentinel}"
        );
    }
    for method in [
        "overworldOffset(",
        "overworldFactor(",
        "overworldJaggedness(",
        "buildErosionOffsetSpline(",
        "buildErosionJaggednessSpline(",
        "buildRidgeJaggednessSpline(",
        "buildWeirdnessJaggednessSpline(",
        "getErosionFactor(",
        "buildMountainRidgeSplineWithPoints(",
        "mountainContinentalness(",
        "calculateMountainRidgeZeroContinentalnessPoint(",
        "ridgeSpline(",
    ] {
        assert!(
            TERRAIN_PROVIDER_JAVA.contains(method),
            "missing TerrainProvider method {method}"
        );
    }
    assert_eq!(occurrence_count(TERRAIN_PROVIDER_JAVA, ".addPoint("), 87);
    assert_eq!(
        occurrence_count(TERRAIN_PROVIDER_JAVA, "CubicSpline.builder"),
        2
    );
    assert_eq!(
        occurrence_count(TERRAIN_PROVIDER_JAVA, "CubicSpline.<C, I>builder"),
        16
    );
    assert_eq!(
        occurrence_count(TERRAIN_PROVIDER_JAVA, "NoiseRouterData.peaksAndValleys"),
        2
    );
    assert_eq!(occurrence_count(TERRAIN_PROVIDER_JAVA, "Mth.lerp"), 4);
    assert_eq!(
        occurrence_count(TERRAIN_PROVIDER_JAVA, "calculateSlope("),
        4
    );
}

#[test]
fn cubic_spline_java_source_shape_matches_terrain_spline_evaluator() {
    assert_eq!(CUBIC_SPLINE_JAVA.lines().count(), 307);
    for sentinel in [
        "int start = findIntervalStart(this.locations, input);",
        "return Mth.binarySearch(0, locations.length, i -> input < locations[i]) - 1;",
        "float a = d1 * (x2 - x1) - (y2 - y1);",
        "float b = -d2 * (x2 - x1) + (y2 - y1);",
        "return Mth.lerp(t, y1, y2) + t * (1.0F - t) * Mth.lerp(t, a, b);",
        "private static float linearExtend",
        "float minValue = Float.POSITIVE_INFINITY;",
        "float maxValue = Float.NEGATIVE_INFINITY;",
    ] {
        assert!(
            CUBIC_SPLINE_JAVA.contains(sentinel),
            "missing CubicSpline sentinel {sentinel}"
        );
    }
}

#[test]
fn terrain_spline_holders_evaluate_vanilla_terrain_provider_shapes() {
    let context = super::super::TerrainSplineContext {
        continents: 0.2,
        erosion: -0.45,
        weirdness: 0.35,
        ridges: super::super::peaks_and_valleys(0.35),
    };
    let offset = super::super::terrain_spline(super::super::TerrainSplineKind::Offset);
    let factor = super::super::terrain_spline(super::super::TerrainSplineKind::Factor);
    let jaggedness = super::super::terrain_spline(super::super::TerrainSplineKind::Jaggedness);
    let amplified_offset =
        super::super::terrain_spline(super::super::TerrainSplineKind::AmplifiedOffset);
    let amplified_factor =
        super::super::terrain_spline(super::super::TerrainSplineKind::AmplifiedFactor);
    let amplified_jaggedness =
        super::super::terrain_spline(super::super::TerrainSplineKind::AmplifiedJaggedness);

    assert!((offset.apply(context) - 0.3207085726435902).abs() < 1e-12);
    assert!((factor.apply(context) - 3.1937037037037035).abs() < 1e-12);
    assert!((jaggedness.apply(context) - 0.0).abs() < 1e-12);
    assert!((amplified_offset.apply(context) - 0.6257336663769374).abs() < 1e-12);
    assert!((amplified_factor.apply(context) - 0.47917681702730064).abs() < 1e-12);
    assert!((amplified_jaggedness.apply(context) - 0.0).abs() < 1e-12);

    for spline in [
        &offset,
        &factor,
        &jaggedness,
        &amplified_offset,
        &amplified_factor,
        &amplified_jaggedness,
    ] {
        let bounds = spline.bounds();
        assert!(bounds.0.is_finite());
        assert!(bounds.1.is_finite());
        assert!(bounds.0 <= bounds.1);
    }
}

#[test]
fn terrain_spline_variants_match_terrain_provider_regression_samples() {
    let offset = super::super::terrain_spline(super::super::TerrainSplineKind::Offset);
    let factor = super::super::terrain_spline(super::super::TerrainSplineKind::Factor);
    let jaggedness = super::super::terrain_spline(super::super::TerrainSplineKind::Jaggedness);
    let amplified_offset =
        super::super::terrain_spline(super::super::TerrainSplineKind::AmplifiedOffset);
    let amplified_factor =
        super::super::terrain_spline(super::super::TerrainSplineKind::AmplifiedFactor);
    let amplified_jaggedness =
        super::super::terrain_spline(super::super::TerrainSplineKind::AmplifiedJaggedness);
    for (
        context,
        expected_offset,
        expected_factor,
        expected_jaggedness,
        expected_amplified_offset,
        expected_amplified_factor,
        expected_amplified_jaggedness,
    ) in [
        (
            super::super::TerrainSplineContext {
                continents: -0.6,
                erosion: 0.15,
                weirdness: -0.7,
                ridges: super::super::peaks_and_valleys(-0.7),
            },
            -0.2222,
            3.95,
            0.0,
            -0.2222,
            3.95,
            0.0,
        ),
        (
            super::super::TerrainSplineContext {
                continents: 0.8,
                erosion: -0.8,
                weirdness: 0.05,
                ridges: super::super::peaks_and_valleys(0.05),
            },
            0.31091066101544634,
            5.1994140625,
            0.0,
            0.6320881393178455,
            0.6340821017071245,
            0.0,
        ),
        (
            super::super::TerrainSplineContext {
                continents: 0.8,
                erosion: -0.95,
                weirdness: -0.02,
                ridges: 1.0,
            },
            1.452639938561368,
            5.6153474999999995,
            0.63,
            2.905279877122736,
            0.6578232451574016,
            1.26,
        ),
    ] {
        assert_close(offset.apply(context), expected_offset);
        assert_close(factor.apply(context), expected_factor);
        assert_close(jaggedness.apply(context), expected_jaggedness);
        assert_close(amplified_offset.apply(context), expected_amplified_offset);
        assert_close(amplified_factor.apply(context), expected_amplified_factor);
        assert_close(
            amplified_jaggedness.apply(context),
            expected_amplified_jaggedness,
        );
        assert_close(
            super::super::terrain_spline(super::super::TerrainSplineKind::LargeBiomesOffset)
                .apply(context),
            expected_offset,
        );
        assert_close(
            super::super::terrain_spline(super::super::TerrainSplineKind::LargeBiomesFactor)
                .apply(context),
            expected_factor,
        );
        assert_close(
            super::super::terrain_spline(super::super::TerrainSplineKind::LargeBiomesJaggedness)
                .apply(context),
            expected_jaggedness,
        );
    }
}
