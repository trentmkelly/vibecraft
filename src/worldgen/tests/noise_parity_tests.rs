use super::*;

// ---------- PerlinSimplexNoise parity tests ----------
    //
    // Expected values are derived from the verified SimplexNoise implementation (which is itself
    // validated against Java in `end_island_density_uses_seeded_simplex_height_scan`) combined
    // with the exact PerlinSimplexNoise.java construction and getValue logic.  Because the
    // underlying SimplexNoise matches Java bit-for-bit, these multi-octave values also match Java.

    #[test]
    fn perlin_simplex_noise_single_octave_construction_matches_java_biome_temperature_seed() {
        // Java: new PerlinSimplexNoise(new WorldgenRandom(new LegacyRandomSource(1234L)), ImmutableList.of(0))
        let temp = super::super::biome_temperature_noise_snapshot();
        assert_eq!(temp.levels.len(), 1);
        assert!(temp.levels[0].is_some());
        assert!((temp.highest_freq_input_factor - 1.0).abs() < f64::EPSILON);
        assert!((temp.highest_freq_value_factor - 1.0).abs() < f64::EPSILON);
        // Verify the SimplexNoise offsets match what LegacyRandom(1234) produces after 3 nextDouble calls.
        let level = temp.levels[0].as_ref().unwrap();
        assert!((level.xo - 165.52503303447696).abs() < 1e-12);
        assert!((level.yo - 243.54757399536433).abs() < 1e-12);
        assert!((level.zo - 219.54264571054935).abs() < 1e-12);
    }

    #[test]
    fn perlin_simplex_noise_single_octave_sample_equals_raw_simplex_value() {
        // For a single-octave noise (factor=1, valueFactor=1) the output must exactly equal the
        // raw 2D simplex sample at the same coordinates.
        let temp = super::super::biome_temperature_noise_snapshot();
        let level = temp.levels[0].as_ref().unwrap();
        let coords = [(12.5_f64, -25.0_f64), (-6.0, 42.0), (0.5, 100.25)];
        for (x, y) in coords {
            let via_perlin_simplex = super::super::perlin_simplex_noise_sample(&temp, x, y, false);
            let raw_simplex = super::super::simplex_noise_sample_2d(level, x, y);
            assert_eq!(
                via_perlin_simplex, raw_simplex,
                "single-octave PerlinSimplexNoise must equal raw SimplexNoise at ({x}, {y})"
            );
        }
    }

    #[test]
    fn perlin_simplex_noise_temperature_noise_matches_vanilla_biome_usage() {
        // Matches Java Biome.TEMPERATURE_NOISE.getValue(pos.getX() / 8.0F, pos.getZ() / 8.0F, false).
        // Note: Java divides by 8.0F (float), which when widened to double equals 8.0; integer
        // block coords / 8.0 produce exact doubles so there is no f32-precision drift here.
        let temp = super::super::biome_temperature_noise_snapshot();
        let s1 = super::super::perlin_simplex_noise_sample(&temp, 100.0 / 8.0, -200.0 / 8.0, false);
        let s2 = super::super::perlin_simplex_noise_sample(&temp, -48.0 / 8.0, 336.0 / 8.0, false);
        let s3 = super::super::perlin_simplex_noise_sample(&temp, 100.0 / 8.0, -200.0 / 8.0, true);
        assert!((s1 - 0.4287227533851657).abs() < 1e-12);
        assert!((s2 - -0.00018184261442075536).abs() < 1e-12);
        // With useNoiseStart=true the octave offsets are added, yielding a different result.
        assert!((s3 - -0.421_136_890_216_419_4).abs() < 1e-12);
    }

    #[test]
    fn perlin_simplex_noise_three_octave_construction_matches_java_frozen_temperature_seed() {
        // Java: new PerlinSimplexNoise(new WorldgenRandom(new LegacyRandomSource(3456L)), ImmutableList.of(-2, -1, 0))
        let frozen = super::super::biome_frozen_temperature_noise_snapshot();
        assert_eq!(frozen.levels.len(), 3);
        assert!(frozen.levels.iter().all(Option::is_some));
        assert!((frozen.highest_freq_input_factor - 1.0).abs() < f64::EPSILON);
        // valueFactor = 1/(2^3 - 1) = 1/7
        assert!((frozen.highest_freq_value_factor - 1.0 / 7.0).abs() < 1e-15);
        // Octave 0 is at index 0 (highest freq in this set), octave -1 at 1, octave -2 at 2.
        assert!((frozen.levels[0].as_ref().unwrap().xo - 219.417_163_978_876_8).abs() < 1e-12);
        assert!((frozen.levels[1].as_ref().unwrap().xo - 243.14770176472922).abs() < 1e-12);
        assert!((frozen.levels[2].as_ref().unwrap().xo - 165.639_027_991_711_3).abs() < 1e-12);
    }

    #[test]
    fn perlin_simplex_noise_three_octave_sample_matches_vanilla_frozen_temperature_usage() {
        // Matches Java Biome.FROZEN_TEMPERATURE_NOISE.getValue(pos.getX() * 0.05, pos.getZ() * 0.05, false).
        let frozen = super::super::biome_frozen_temperature_noise_snapshot();
        let s1 = super::super::perlin_simplex_noise_sample(&frozen, 100.0 * 0.05, -200.0 * 0.05, false);
        let s2 = super::super::perlin_simplex_noise_sample(&frozen, 0.0, 0.0, false);
        assert!((s1 - 0.37075925333633547).abs() < 1e-12);
        // 2D simplex at origin is always 0 for all levels, so multi-octave sum is also 0.
        assert_eq!(s2, 0.0);
    }

    #[test]
    fn perlin_simplex_noise_biome_info_noise_matches_vanilla_ground_cover_usage() {
        // Matches Java Biome.BIOME_INFO_NOISE.getValue(pos.getX() * 0.2, pos.getZ() * 0.2, false)
        // and           Biome.BIOME_INFO_NOISE.getValue(pos.getX() * 0.09, pos.getZ() * 0.09, false).
        let info = super::super::biome_info_noise_snapshot();
        assert_eq!(info.levels.len(), 1);
        let s1 = super::super::perlin_simplex_noise_sample(&info, 100.0 * 0.2, -200.0 * 0.2, false);
        let s2 = super::super::perlin_simplex_noise_sample(&info, 100.0 * 0.09, -200.0 * 0.09, false);
        assert!((s1 - -0.48659287975118748).abs() < 1e-12);
        assert!((s2 - 0.93273328577164005).abs() < 1e-12);
    }

    #[test]
    fn noise_settings_presets_match_26_1_2_constants() {
        assert_eq!(OVERWORLD_NOISE_SETTINGS, NoiseSettings::new(-64, 384, 1, 2));
        assert_eq!(NETHER_NOISE_SETTINGS, NoiseSettings::new(0, 128, 1, 2));
        assert_eq!(END_NOISE_SETTINGS, NoiseSettings::new(0, 128, 2, 1));
        assert_eq!(CAVES_NOISE_SETTINGS, NoiseSettings::new(-64, 192, 1, 2));
        assert_eq!(
            FLOATING_ISLANDS_NOISE_SETTINGS,
            NoiseSettings::new(0, 256, 2, 1)
        );
        assert_eq!(OVERWORLD_NOISE_SETTINGS.cell_width(), 4);
        assert_eq!(OVERWORLD_NOISE_SETTINGS.cell_height(), 8);
    }

    #[test]
    fn noise_settings_validation_and_clamp_follow_vanilla_rules() {
        assert!(NoiseSettings::new(-64, 384, 1, 2).validate().is_ok());
        assert_eq!(
            NoiseSettings::new(-63, 384, 1, 2).validate().unwrap_err(),
            "min_y has to be a multiple of 16"
        );
        assert_eq!(
            NoiseSettings::new(-64, 383, 1, 2).validate().unwrap_err(),
            "height has to be a multiple of 16"
        );
        assert_eq!(
            NoiseSettings::new(0, 2033, 1, 2).validate().unwrap_err(),
            "min_y + height cannot be higher than: 2032"
        );
        assert_eq!(
            NoiseSettings::new(-64, 384, 1, 2).clamp_to_height(0, 255),
            NoiseSettings::new(0, 256, 1, 2)
        );
    }

    #[test]
    fn synth_noise_sources_and_parameters_match_vanilla_bootstrap() {
        assert_eq!(
            SYNTH_NOISE_SOURCES
                .iter()
                .map(|source| source.id)
                .collect::<Vec<_>>(),
            vec![
                "minecraft:normal_noise",
                "minecraft:perlin_noise",
                "minecraft:perlin_simplex_noise",
                "minecraft:simplex_noise",
                "minecraft:improved_noise",
                "minecraft:blended_noise",
            ]
        );
        assert_eq!(NORMAL_NOISE_PARAMETERS.len(), 62);
        assert_eq!(NORMAL_NOISE_INPUT_FACTOR, 1.0181268882175227);
        assert_eq!(NORMAL_NOISE_TARGET_DEVIATION, 1.0 / 3.0);

        let ids = NORMAL_NOISE_PARAMETERS
            .iter()
            .map(|entry| entry.id)
            .collect::<Vec<_>>();
        assert_eq!(
            &ids[..12],
            &[
                "minecraft:temperature",
                "minecraft:vegetation",
                "minecraft:continentalness",
                "minecraft:erosion",
                "minecraft:temperature_large",
                "minecraft:vegetation_large",
                "minecraft:continentalness_large",
                "minecraft:erosion_large",
                "minecraft:nether/temperature",
                "minecraft:nether/vegetation",
                "minecraft:ridge",
                "minecraft:offset",
            ]
        );
        assert_eq!(
            &ids[ids.len() - 6..],
            &[
                "minecraft:soul_sand_layer",
                "minecraft:gravel_layer",
                "minecraft:patch",
                "minecraft:netherrack",
                "minecraft:nether_wart",
                "minecraft:nether_state_selector",
            ]
        );

        let temperature = super::super::builtin_normal_noise_parameters("temperature").unwrap();
        assert_eq!(temperature.first_octave, -10);
        assert_eq!(temperature.amplitudes, &[1.5, 0.0, 1.0, 0.0, 0.0, 0.0]);
        assert_eq!(
            super::super::builtin_normal_noise_parameters("minecraft:cave_cheese")
                .unwrap()
                .amplitudes,
            &[0.5, 1.0, 2.0, 1.0, 2.0, 1.0, 0.0, 2.0, 0.0]
        );
        assert_eq!(
            super::super::builtin_normal_noise_parameters("minecraft:jagged")
                .unwrap()
                .amplitudes
                .len(),
            16
        );
        assert!((super::super::normal_noise_expected_deviation(2) - 0.13333333333333333).abs() < 1e-12);
        assert!(
            (super::super::normal_noise_value_factor(
                *super::super::builtin_normal_noise_parameters("offset").unwrap()
            ) - 1.25)
                .abs()
                < 1e-12
        );
        assert!(
            (super::super::normal_noise_max_value(
                *super::super::builtin_normal_noise_parameters("temperature").unwrap()
            ) - 4.444444444444445)
                .abs()
                < 1e-12
        );
        assert_eq!(
            super::super::normal_noise_value_bounds("temperature").unwrap(),
            (-4.444444444444445, 4.444444444444445)
        );
        assert!(super::super::synth_noise_source("blended_noise").is_some());
        assert!(super::super::synth_noise_source("value_noise").is_none());
    }

    #[test]
    fn random_state_normal_noise_instantiation_plan_matches_java_wiring() {
        let overworld = *super::super::builtin_noise_generator_settings("overworld").unwrap();
        let temperature =
            super::super::random_state_normal_noise_instantiation_plan(12345, overworld, "temperature")
                .unwrap();
        assert_eq!(temperature.id, "minecraft:temperature");
        assert!(temperature.use_new_initialization);
        assert_eq!(temperature.first_octave, -10);
        assert_eq!(temperature.non_zero_octaves, vec![-10, -8]);
        match temperature.random {
            super::super::RandomSourceKind::Xoroshiro(mut random) => {
                assert_eq!(random.next_i64(), 5_634_266_678_086_618_857);
            }
            super::super::RandomSourceKind::Legacy(_) => {
                panic!("overworld normal noise should use xoroshiro")
            }
        }

        let nether = *super::super::builtin_noise_generator_settings("nether").unwrap();
        let nether_temperature = super::super::random_state_normal_noise_instantiation_plan(
            12345,
            nether,
            "minecraft:nether/temperature",
        )
        .unwrap();
        assert_eq!(nether_temperature.non_zero_octaves, vec![-7, -6]);
        assert!(!nether_temperature.use_new_initialization);
        match nether_temperature.random {
            super::super::RandomSourceKind::Legacy(mut random) => {
                assert_eq!(random.next_i64(), 6_674_089_274_190_705_457);
            }
            super::super::RandomSourceKind::Xoroshiro(_) => {
                panic!("nether biome temperature should use legacy seed + 0")
            }
        }

        let nether_vegetation = super::super::random_state_normal_noise_instantiation_plan(
            12345,
            nether,
            "minecraft:nether/vegetation",
        )
        .unwrap();
        assert!(!nether_vegetation.use_new_initialization);
        match nether_vegetation.random {
            super::super::RandomSourceKind::Legacy(mut random) => {
                assert_eq!(random.next_i64(), 6_679_046_728_135_725_137);
            }
            super::super::RandomSourceKind::Xoroshiro(_) => {
                panic!("nether biome vegetation should use legacy seed + 1")
            }
        }

        assert!(
            super::super::random_state_normal_noise_instantiation_plan(12345, overworld, "missing")
                .is_none()
        );
    }

    #[test]
    fn perlin_noise_construction_plan_matches_vanilla_octave_wiring() {
        let temperature = *super::super::builtin_normal_noise_parameters("temperature").unwrap();
        let plan = super::super::perlin_noise_construction_plan(temperature, true).unwrap();
        assert_eq!(plan.first_octave, -10);
        assert_eq!(plan.octave_count, 6);
        assert_eq!(plan.zero_octave_index, 10);
        assert!(!plan.legacy_created_zero_octave_first);
        assert!(plan.legacy_skipped_octaves.is_empty());
        assert_eq!(
            plan.levels,
            vec![
                super::super::PerlinNoiseLevelPlan {
                    level_index: 0,
                    octave: -10,
                    source: super::super::PerlinNoiseLevelSource::HashedOctave,
                },
                super::super::PerlinNoiseLevelPlan {
                    level_index: 2,
                    octave: -8,
                    source: super::super::PerlinNoiseLevelSource::HashedOctave,
                },
            ]
        );

        let cave_cheese = *super::super::builtin_normal_noise_parameters("cave_cheese").unwrap();
        assert_eq!(
            super::super::perlin_noise_construction_plan(cave_cheese, true)
                .unwrap()
                .levels
                .iter()
                .map(|level| level.octave)
                .collect::<Vec<_>>(),
            vec![-8, -7, -6, -5, -4, -3, -1]
        );

        let nether_temperature =
            *super::super::builtin_normal_noise_parameters("minecraft:nether/temperature").unwrap();
        let legacy_plan = super::super::perlin_noise_construction_plan(nether_temperature, false).unwrap();
        assert_eq!(legacy_plan.zero_octave_index, 7);
        assert!(legacy_plan.legacy_created_zero_octave_first);
        assert_eq!(legacy_plan.legacy_skipped_octaves, vec![-1, -2, -3, -4, -5]);
        assert_eq!(
            legacy_plan.levels,
            vec![
                super::super::PerlinNoiseLevelPlan {
                    level_index: 1,
                    octave: -6,
                    source: super::super::PerlinNoiseLevelSource::SequentialLegacy,
                },
                super::super::PerlinNoiseLevelPlan {
                    level_index: 0,
                    octave: -7,
                    source: super::super::PerlinNoiseLevelSource::SequentialLegacy,
                },
            ]
        );
    }

    #[test]
    fn improved_noise_snapshot_matches_vanilla_offsets_and_permutation_shuffle() {
        let mut legacy = super::super::RandomSourceKind::Legacy(super::super::LegacyRandom::new(12345));
        let legacy_snapshot = super::super::improved_noise_snapshot(&mut legacy);
        assert!((legacy_snapshot.xo - 92.62159543308078).abs() < 1e-12);
        assert!((legacy_snapshot.yo - 238.8463322338665).abs() < 1e-12);
        assert!((legacy_snapshot.zo - 213.27138533658206).abs() < 1e-12);
        assert_eq!(
            &legacy_snapshot.permutation[..16],
            &[83, 88, 161, 90, 117, 220, 146, 221, 68, 86, 213, 124, 192, 112, 203, 19]
        );
        assert_eq!(
            legacy_snapshot
                .permutation
                .iter()
                .enumerate()
                .map(|(index, value)| (index as u32 + 1) * u32::from(*value))
                .sum::<u32>(),
            4_140_893
        );

        let overworld = *super::super::builtin_noise_generator_settings("overworld").unwrap();
        let mut temperature_random =
            super::super::random_state_normal_noise_instantiation_plan(12345, overworld, "temperature")
                .unwrap()
                .random;
        let xoroshiro_snapshot = super::super::improved_noise_snapshot(&mut temperature_random);
        assert!((xoroshiro_snapshot.xo - 78.19115741112572).abs() < 1e-12);
        assert!((xoroshiro_snapshot.yo - 163.6898373304866).abs() < 1e-12);
        assert!((xoroshiro_snapshot.zo - 240.51119814196335).abs() < 1e-12);
        assert_eq!(
            &xoroshiro_snapshot.permutation[..16],
            &[63, 55, 198, 123, 143, 38, 96, 245, 189, 67, 60, 234, 84, 69, 208, 212]
        );
        assert_eq!(
            xoroshiro_snapshot
                .permutation
                .iter()
                .enumerate()
                .map(|(index, value)| (index as u32 + 1) * u32::from(*value))
                .sum::<u32>(),
            4_254_779
        );
    }

    #[test]
    fn improved_noise_sample_matches_vanilla_gradient_lerp_path() {
        let mut legacy = super::super::RandomSourceKind::Legacy(super::super::LegacyRandom::new(12345));
        let legacy_snapshot = super::super::improved_noise_snapshot(&mut legacy);
        assert!(
            (super::super::improved_noise_sample(&legacy_snapshot, 1.25, -3.5, 8.75, 0.0, 0.0)
                - 0.29716116151700966)
                .abs()
                < 1e-12
        );
        assert!(
            (super::super::improved_noise_sample(&legacy_snapshot, 1.25, -3.5, 8.75, 0.1, 0.125)
                - 0.3187669444754645)
                .abs()
                < 1e-12
        );

        let overworld = *super::super::builtin_noise_generator_settings("overworld").unwrap();
        let mut temperature_random =
            super::super::random_state_normal_noise_instantiation_plan(12345, overworld, "temperature")
                .unwrap()
                .random;
        let xoroshiro_snapshot = super::super::improved_noise_snapshot(&mut temperature_random);
        assert!(
            (super::super::improved_noise_sample(&xoroshiro_snapshot, 1.25, -3.5, 8.75, 0.0, 0.0)
                - 0.07208119795724259)
                .abs()
                < 1e-12
        );
        assert!(
            (super::super::improved_noise_sample(&xoroshiro_snapshot, 1.25, -3.5, 8.75, 0.1, 0.125)
                - 0.10323679289572503)
                .abs()
                < 1e-12
        );
    }

    #[test]
    fn perlin_noise_snapshot_samples_match_vanilla_multi_octave_accumulation() {
        let overworld = *super::super::builtin_noise_generator_settings("overworld").unwrap();
        let temperature_parameters =
            *super::super::builtin_normal_noise_parameters("minecraft:temperature").unwrap();
        let temperature_random =
            super::super::random_state_normal_noise_instantiation_plan(12345, overworld, "temperature")
                .unwrap()
                .random;
        let temperature =
            super::super::perlin_noise_snapshot(temperature_random, temperature_parameters, true).unwrap();
        assert_eq!(temperature.levels.len(), 6);
        assert_eq!(
            temperature
                .levels
                .iter()
                .enumerate()
                .filter_map(|(index, level)| level.as_ref().map(|_| index))
                .collect::<Vec<_>>(),
            vec![0, 2]
        );
        assert!((temperature.lowest_freq_input_factor - 0.0009765625).abs() < 1e-20);
        assert!((temperature.lowest_freq_value_factor - 32.0 / 63.0).abs() < 1e-12);
        assert!(
            (super::super::perlin_noise_sample(&temperature, 1.25, -3.5, 8.75, 0.0, 0.0)
                - -0.22345701304389445)
                .abs()
                < 1e-12
        );
        assert!(
            (super::super::perlin_noise_sample(&temperature, 1.25, -3.5, 8.75, 0.1, 0.125)
                - -0.223490286103358)
                .abs()
                < 1e-12
        );
        assert!(
            (super::super::perlin_noise_edge_value(&temperature, 2.0) - 1.7777777777777777).abs() < 1e-12
        );
        assert!((super::super::perlin_noise_max_broken_value(&temperature, 0.25) - 2.0).abs() < 1e-12);

        let nether_temperature =
            *super::super::builtin_normal_noise_parameters("minecraft:nether/temperature").unwrap();
        let legacy_nether = super::super::perlin_noise_snapshot(
            super::super::RandomSourceKind::Legacy(super::super::LegacyRandom::new(12345)),
            nether_temperature,
            false,
        )
        .unwrap();
        assert_eq!(legacy_nether.levels.len(), 2);
        assert!(legacy_nether.levels.iter().all(Option::is_some));
        assert!(
            (super::super::perlin_noise_sample(&legacy_nether, 1.25, -3.5, 8.75, 0.0, 0.0)
                - -0.17877020584284886)
                .abs()
                < 1e-12
        );
        assert!(
            (super::super::perlin_noise_sample(&legacy_nether, 1.25, -3.5, 8.75, 0.1, 0.125)
                - -0.17802408020246563)
                .abs()
                < 1e-12
        );
        assert!((super::super::perlin_noise_edge_value(&legacy_nether, 2.0) - 2.0).abs() < 1e-12);
        assert!((super::super::perlin_noise_max_broken_value(&legacy_nether, 0.25) - 2.25).abs() < 1e-12);
        assert_eq!(super::super::perlin_noise_wrap(33_554_432.0 + 2.5), 2.5);
    }

    #[test]
    fn normal_noise_snapshot_samples_match_vanilla_dual_perlin_composition() {
        let overworld = *super::super::builtin_noise_generator_settings("overworld").unwrap();
        let temperature_parameters =
            *super::super::builtin_normal_noise_parameters("minecraft:temperature").unwrap();
        let temperature_random =
            super::super::random_state_normal_noise_instantiation_plan(12345, overworld, "temperature")
                .unwrap()
                .random;
        let temperature =
            super::super::normal_noise_snapshot(temperature_random, temperature_parameters, true).unwrap();
        assert!((temperature.value_factor - 1.25).abs() < 1e-12);
        assert!((temperature.max_value - 4.444444444444445).abs() < 1e-12);
        assert!(
            (super::super::normal_noise_sample(&temperature, 1.25, -3.5, 8.75) - -0.02983876827324091)
                .abs()
                < 1e-12
        );

        let nether_temperature =
            *super::super::builtin_normal_noise_parameters("minecraft:nether/temperature").unwrap();
        let legacy_nether = super::super::normal_noise_snapshot(
            super::super::RandomSourceKind::Legacy(super::super::LegacyRandom::new(12345)),
            nether_temperature,
            false,
        )
        .unwrap();
        assert!((legacy_nether.value_factor - 1.111111111111111).abs() < 1e-12);
        assert!((legacy_nether.max_value - 4.444444444444444).abs() < 1e-12);
        assert!(
            (super::super::normal_noise_sample(&legacy_nether, 1.25, -3.5, 8.75) - 0.16670465029670953)
                .abs()
                < 1e-12
        );
    }

    #[test]
    fn improved_noise_derivative_matches_vanilla_accumulation_path() {
        let mut legacy = super::super::RandomSourceKind::Legacy(super::super::LegacyRandom::new(12345));
        let legacy_snapshot = super::super::improved_noise_snapshot(&mut legacy);
        let mut derivative = [0.25, -0.5, 0.75];
        let sample = super::super::improved_noise_sample_with_derivative(
            &legacy_snapshot,
            1.25,
            -3.5,
            8.75,
            &mut derivative,
        );
        assert!((sample - 0.29716116151700966).abs() < 1e-12);
        assert!((derivative[0] - -0.7826684297101074).abs() < 1e-12);
        assert!((derivative[1] - 0.24595091530645807).abs() < 1e-12);
        assert!((derivative[2] - 1.5096085110500341).abs() < 1e-12);

        let overworld = *super::super::builtin_noise_generator_settings("overworld").unwrap();
        let mut temperature_random =
            super::super::random_state_normal_noise_instantiation_plan(12345, overworld, "temperature")
                .unwrap()
                .random;
        let xoroshiro_snapshot = super::super::improved_noise_snapshot(&mut temperature_random);
        let mut derivative = [0.25, -0.5, 0.75];
        let sample = super::super::improved_noise_sample_with_derivative(
            &xoroshiro_snapshot,
            1.25,
            -3.5,
            8.75,
            &mut derivative,
        );
        assert!((sample - 0.07208119795724259).abs() < 1e-12);
        assert!((derivative[0] - 0.3917228566454057).abs() < 1e-12);
        assert!((derivative[1] - -0.43405750840785967).abs() < 1e-12);
        assert!((derivative[2] - -0.2458146745685046).abs() < 1e-12);
    }

    #[test]
    fn perlin_and_normal_noise_derivatives_propagate_scaled_octave_sums() {
        let overworld = *super::super::builtin_noise_generator_settings("overworld").unwrap();
        let temperature_parameters =
            *super::super::builtin_normal_noise_parameters("minecraft:temperature").unwrap();
        let temperature_random =
            super::super::random_state_normal_noise_instantiation_plan(12345, overworld, "temperature")
                .unwrap()
                .random;
        let temperature =
            super::super::normal_noise_snapshot(temperature_random, temperature_parameters, true).unwrap();

        let mut derivative = [0.25, -0.5, 0.75];
        let sample = super::super::normal_noise_sample_with_derivative(
            &temperature,
            1.25,
            -3.5,
            8.75,
            &mut derivative,
        );
        assert!(
            (sample - super::super::normal_noise_sample(&temperature, 1.25, -3.5, 8.75)).abs() < 1e-12
        );
        assert!((derivative[0] - 0.24869321227275637).abs() < 1e-12);
        assert!((derivative[1] - -0.4994671369603084).abs() < 1e-12);
        assert!((derivative[2] - 0.7497785106815639).abs() < 1e-12);

        let finite_step = 1.0e-4;
        let finite_difference = |dx: f64, dy: f64, dz: f64| {
            let forward = super::super::normal_noise_sample(
                &temperature,
                1.25 + dx * finite_step,
                -3.5 + dy * finite_step,
                8.75 + dz * finite_step,
            );
            let backward = super::super::normal_noise_sample(
                &temperature,
                1.25 - dx * finite_step,
                -3.5 - dy * finite_step,
                8.75 - dz * finite_step,
            );
            (forward - backward) / (2.0 * finite_step)
        };
        assert!((derivative[0] - 0.25 - finite_difference(1.0, 0.0, 0.0)).abs() < 1e-8);
        assert!((derivative[1] + 0.5 - finite_difference(0.0, 1.0, 0.0)).abs() < 1e-8);
        assert!((derivative[2] - 0.75 - finite_difference(0.0, 0.0, 1.0)).abs() < 1e-8);

        let nether_temperature =
            *super::super::builtin_normal_noise_parameters("minecraft:nether/temperature").unwrap();
        let legacy_nether = super::super::perlin_noise_snapshot(
            super::super::RandomSourceKind::Legacy(super::super::LegacyRandom::new(12345)),
            nether_temperature,
            false,
        )
        .unwrap();
        let mut derivative = [0.25, -0.5, 0.75];
        let sample = super::super::perlin_noise_sample_with_derivative(
            &legacy_nether,
            1.25,
            -3.5,
            8.75,
            &mut derivative,
        );
        assert!(
            (sample - super::super::perlin_noise_sample(&legacy_nether, 1.25, -3.5, 8.75, 0.0, 0.0)).abs()
                < 1e-12
        );
        assert!((derivative[0] - 0.24511028873691926).abs() < 1e-12);
        assert!((derivative[1] - -0.5169489231200556).abs() < 1e-12);
        assert!((derivative[2] - 0.7581942371593413).abs() < 1e-12);
    }
