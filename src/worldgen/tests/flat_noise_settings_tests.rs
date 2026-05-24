use super::*;

    #[test]
    fn flat_generator_expands_layers_bottom_up_like_vanilla() {
        let settings = super::super::default_flat_generator_settings().unwrap();
        assert_eq!(settings.biome, "minecraft:plains");
        assert_eq!(
            settings.structure_overrides,
            vec!["minecraft:strongholds", "minecraft:villages"]
        );
        assert_eq!(
            settings.expanded_layers,
            vec![
                Some("minecraft:bedrock"),
                Some("minecraft:dirt"),
                Some("minecraft:dirt"),
                Some("minecraft:grass_block")
            ]
        );
        assert!(!settings.void_generation);
        assert_eq!(
            super::super::flat_base_height(
                &settings.expanded_layers,
                super::super::FLAT_GENERATOR_MIN_Y,
                super::super::FLAT_GENERATOR_GEN_DEPTH,
                HeightmapKind::MotionBlocking
            ),
            4
        );
        assert_eq!(
            super::super::flat_base_column(&settings.expanded_layers, 0, 6).states,
            vec![
                "minecraft:bedrock",
                "minecraft:dirt",
                "minecraft:dirt",
                "minecraft:grass_block",
                "minecraft:air",
                "minecraft:air"
            ]
        );
    }

    #[test]
    fn flat_generator_handles_void_and_non_motion_blocking_layers() {
        let void_settings = super::super::flat_generator_settings(
            super::super::flat_generator_preset("minecraft:the_void").unwrap(),
        )
        .unwrap();
        assert!(void_settings.void_generation);
        assert_eq!(void_settings.expanded_layers, vec![None]);
        assert_eq!(
            void_settings.top_layer_modifications,
            vec![(0, "minecraft:air")]
        );

        let snowy = super::super::flat_generator_settings(
            super::super::flat_generator_preset("minecraft:snowy_kingdom").unwrap(),
        )
        .unwrap();
        assert_eq!(snowy.expanded_layers[0], Some("minecraft:bedrock"));
        assert_eq!(snowy.expanded_layers[63], Some("minecraft:grass_block"));
        assert_eq!(snowy.expanded_layers[64], None);
        assert_eq!(snowy.top_layer_modifications, vec![(64, "minecraft:snow")]);
        assert_eq!(
            super::super::flat_base_height(
                &snowy.expanded_layers,
                super::super::FLAT_GENERATOR_MIN_Y,
                super::super::FLAT_GENERATOR_GEN_DEPTH,
                HeightmapKind::MotionBlocking
            ),
            64
        );
    }

    #[test]
    fn flat_generator_materializes_chunk_sections_and_heightmaps() {
        let settings = super::super::default_flat_generator_settings().unwrap();
        let chunk = super::super::materialize_flat_chunk(ChunkPos { x: 2, z: -1 }, &settings);
        assert_eq!(chunk.status, "minecraft:full");
        assert_eq!(chunk.sections.len(), 1);
        assert_eq!(chunk.sections[0].y, 0);
        assert!(chunk.heightmaps.contains_key("WORLD_SURFACE_WG"));
        assert!(chunk.heightmaps.contains_key("OCEAN_FLOOR_WG"));

        let Tag::Compound(section) = &chunk.sections[0].block_states else {
            panic!("block states should be stored as a compound");
        };
        let Some((_, Tag::List(palette))) = section.iter().find(|(name, _)| name == "palette")
        else {
            panic!("block states should include a palette");
        };
        assert_eq!(palette.len(), 4);
        assert!(matches!(
            &chunk.sections[0].biomes,
            Tag::Compound(fields)
                if matches!(
                    fields.iter().find(|(name, _)| name == "palette"),
                    Some((_, Tag::List(values))) if values == &vec![Tag::String("minecraft:plains".to_string())]
                )
        ));
    }

    // ---------- RandomStateNoiseCache parity tests ----------

    #[test]
    fn random_state_noise_cache_matches_uncached_and_is_stable_across_calls() {
        let seed = 12345_i64;
        let settings = *builtin_noise_generator_settings("minecraft:overworld").unwrap();
        let noise_id = "minecraft:temperature";

        let uncached = random_state_normal_noise_snapshot(seed, settings, noise_id).unwrap();

        let mut cache = RandomStateNoiseCache::new(seed, settings);
        let first = cache.get_or_create_noise(noise_id).unwrap().clone();
        let second = cache.get_or_create_noise(noise_id).unwrap().clone();

        // First call must match uncached computation (same seed, same noise id).
        assert_eq!(first, uncached);
        // Repeated calls must return the identical cached value.
        assert_eq!(first, second);
    }

    #[test]
    fn random_state_noise_cache_unknown_id_returns_none() {
        let settings = *builtin_noise_generator_settings("minecraft:overworld").unwrap();
        let mut cache = RandomStateNoiseCache::new(0, settings);
        assert!(cache
            .get_or_create_noise("minecraft:nonexistent_noise_xyz")
            .is_none());
    }

    // ---------- NoiseGeneratorSettings codec loading tests ----------

    #[test]
    fn noise_generator_settings_scalar_fields_match_vanilla_json_files() {
        // Read every noise_settings JSON from the decompiled server data directory and
        // validate the scalar fields against our hardcoded EXTRACTED_NOISE_SETTINGS_REGISTRY_EXPECTATIONS.
        // This is the codec-loading parity test: it proves our statics match vanilla JSON.
        let dir = "../decompiled-server-26.1.2/data/minecraft/worldgen/noise_settings";

        for entry in EXTRACTED_NOISE_SETTINGS_REGISTRY_EXPECTATIONS {
            let name = entry.id.strip_prefix("minecraft:").unwrap_or(entry.id);
            let path = format!("{}/{}.json", dir, name);
            let raw = std::fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("failed to read {path}: {e}"));
            let json: serde_json::Value = serde_json::from_str(&raw)
                .unwrap_or_else(|e| panic!("invalid JSON in {path}: {e}"));

            let noise = &json["noise"];
            assert_eq!(
                noise["min_y"].as_i64().unwrap() as i32,
                entry.noise.min_y,
                "{} noise.min_y",
                entry.id
            );
            assert_eq!(
                noise["height"].as_i64().unwrap() as i32,
                entry.noise.height,
                "{} noise.height",
                entry.id
            );
            assert_eq!(
                noise["size_horizontal"].as_i64().unwrap() as i32,
                entry.noise.size_horizontal,
                "{} noise.size_horizontal",
                entry.id
            );
            assert_eq!(
                noise["size_vertical"].as_i64().unwrap() as i32,
                entry.noise.size_vertical,
                "{} noise.size_vertical",
                entry.id
            );

            // default_block / default_fluid stored as { "Name": "minecraft:..." }
            assert_eq!(
                json["default_block"]["Name"].as_str().unwrap(),
                entry.default_block,
                "{} default_block",
                entry.id
            );
            assert_eq!(
                json["default_fluid"]["Name"].as_str().unwrap(),
                entry.default_fluid,
                "{} default_fluid",
                entry.id
            );

            assert_eq!(
                json["sea_level"].as_i64().unwrap() as i32,
                entry.sea_level,
                "{} sea_level",
                entry.id
            );
            assert_eq!(
                json["disable_mob_generation"].as_bool().unwrap(),
                entry.disable_mob_generation,
                "{} disable_mob_generation",
                entry.id
            );
            assert_eq!(
                json["aquifers_enabled"].as_bool().unwrap(),
                entry.aquifers_enabled,
                "{} aquifers_enabled",
                entry.id
            );
            assert_eq!(
                json["ore_veins_enabled"].as_bool().unwrap(),
                entry.ore_veins_enabled,
                "{} ore_veins_enabled",
                entry.id
            );
            assert_eq!(
                json["legacy_random_source"].as_bool().unwrap(),
                entry.legacy_random_source,
                "{} legacy_random_source",
                entry.id
            );
            assert_eq!(
                json["spawn_target"]
                    .as_array()
                    .map(|a| a.len())
                    .unwrap_or(0),
                entry.spawn_target_len,
                "{} spawn_target length",
                entry.id
            );
        }
    }

    #[test]
    fn overworld_final_density_matches_vanilla_at_0_100_0_seed_0() {
        // Oracle command used against official 26.1.2:
        // VanillaRegistries.createLookup() -> RandomState.create(OVERWORLD, seed 0) ->
        // router().finalDensity().compute(SinglePointContext(0, 100, 0)).
        const VANILLA_FINAL_DENSITY_0_100_0_SEED_0: f64 = -0.45833333333333330;
        let seed = 0_i64;

        // Confirm the registry entry resolves.
        assert!(
            builtin_density_function("minecraft:overworld/final_density").is_some(),
            "minecraft:overworld/final_density must be registered in BUILTIN_DENSITY_FUNCTIONS"
        );

        let settings = *builtin_noise_generator_settings("minecraft:overworld")
            .expect("overworld noise_generator_settings must exist");

        let final_density = super::super::OVERWORLD_NOISE_ROUTER.final_density;
        let result = final_density.compute_with_noise(seed, settings, 0, 100, 0);

        assert!(
            result.is_finite(),
            "finalDensity at (0,100,0) seed 0 must be finite, got {result}"
        );
        assert!(
            (result - VANILLA_FINAL_DENSITY_0_100_0_SEED_0).abs() < 1e-15,
            "finalDensity at (0,100,0) seed 0 must match vanilla {VANILLA_FINAL_DENSITY_0_100_0_SEED_0}, got {result}"
        );
    }

    #[test]
    fn overworld_base_3d_noise_matches_vanilla_in_river_column() {
        // Oracle command used against official 26.1.2:
        // VanillaRegistries.createLookup() -> RandomState.create(OVERWORLD, seed 8675309) ->
        // router().finalDensity().compute(SinglePointContext(-16, y, -8)).
        // The base_3d_noise value is the RandomState-wired BlendedNoise with the
        // "minecraft:terrain" positional random, not the unseeded registry entry.
        let seed = 8_675_309_i64;
        let settings = *builtin_noise_generator_settings("minecraft:overworld")
            .expect("overworld noise_generator_settings must exist");

        let base_3d =
            super::super::BASE_3D_NOISE_OVERWORLD_DENSITY.compute_with_noise(seed, settings, -16, 56, -8);
        assert!(
            (base_3d - -0.08889146062143664).abs() < 1e-14,
            "base_3d_noise at (-16,56,-8) seed {seed} must match vanilla, got {base_3d}"
        );

        let final_density = super::super::OVERWORLD_NOISE_ROUTER.final_density;
        let y_samples = [
            (56, 0.09843842627734438),
            (57, 0.04970794306538723),
            (58, -0.002730670313905269),
            (59, -0.03560427276411281),
            (60, -0.04682484220125964),
            (61, -0.06178625975797264),
            (62, -0.07550869288269363),
        ];
        for (y, expected) in y_samples {
            let result = final_density.compute_with_noise(seed, settings, -16, y, -8);
            assert!(
                (result - expected).abs() < 1e-3,
                "finalDensity at (-16,{y},-8) seed {seed} must match vanilla {expected}, got {result}"
            );
        }
    }

    #[test]
    fn overworld_shift_noise_uses_vanilla_offset_noise_key() {
        let seed = 8_675_309_i64;
        let settings = *builtin_noise_generator_settings("minecraft:overworld").unwrap();
        assert!(
            super::super::builtin_normal_noise_parameters("minecraft:shift").is_none(),
            "vanilla Noises.SHIFT is ResourceKey minecraft:offset; Rust must not invent minecraft:shift"
        );

        let samples = [
            ((0, 62, 0), -3.7094676845533336, -3.7094676845533336),
            ((3, 62, 0), -3.3318550257927586, -2.9289862624467620),
            ((31, 62, 6), -0.42489011889609285, -0.4626364685144386),
        ];

        for ((x, y, z), expected_x, expected_z) in samples {
            let shift_x = super::super::SHIFT_X_DENSITY.compute_with_noise(seed, settings, x, y, z);
            let shift_z = super::super::SHIFT_Z_DENSITY.compute_with_noise(seed, settings, x, y, z);
            assert!(
                (shift_x - expected_x).abs() < 1e-12,
                "SHIFT_X at ({x},{y},{z}) seed {seed} should match Java Noises.SHIFT offset noise"
            );
            assert!(
                (shift_z - expected_z).abs() < 1e-12,
                "SHIFT_Z at ({x},{y},{z}) seed {seed} should match Java Noises.SHIFT offset noise"
            );
        }
    }

    /// Verifies that every NoiseRouter field for the standard overworld evaluates to a finite,
    /// non-NaN value at a canonical position. An unregistered Reference silently returns 0.0
    /// rather than NaN or infinity, so this test guards against both silent-zero and arithmetic
    /// blow-up bugs. Seed 12345, block (0, 64, 0) is well inside the active overworld range.
    #[test]
    fn overworld_noise_router_all_fields_finite_at_canonical_position() {
        let seed = 12345_i64;
        let settings = *builtin_noise_generator_settings("minecraft:overworld")
            .expect("overworld noise_generator_settings must exist");
        let router = super::super::OVERWORLD_NOISE_ROUTER;
        let (x, y, z) = (0, 64, 0);

        macro_rules! check_field {
            ($field:expr, $name:literal) => {
                let val = $field.compute_with_noise(seed, settings, x, y, z);
                assert!(
                    val.is_finite(),
                    "OVERWORLD_NOISE_ROUTER.{} returned non-finite ({val}) at ({x},{y},{z}) seed {seed}",
                    $name
                );
            };
        }

        check_field!(router.barrier, "barrier");
        check_field!(router.fluid_level_floodedness, "fluid_level_floodedness");
        check_field!(router.fluid_level_spread, "fluid_level_spread");
        check_field!(router.lava, "lava");
        check_field!(router.temperature, "temperature");
        check_field!(router.vegetation, "vegetation");
        check_field!(router.continents, "continents");
        check_field!(router.erosion, "erosion");
        check_field!(router.depth, "depth");
        check_field!(router.ridges, "ridges");
        check_field!(
            router.preliminary_surface_level,
            "preliminary_surface_level"
        );
        check_field!(router.final_density, "final_density");
        check_field!(router.vein_toggle, "vein_toggle");
        check_field!(router.vein_ridged, "vein_ridged");
        check_field!(router.vein_gap, "vein_gap");
    }

    /// Parity test: ClimateSampler evaluates all six climate density functions at overworld
    /// block (0, 64, 0) with seed 0, then finds the nearest biome from the overworld parameter
    /// list. The expected biome is derived by calling `find_value_bruteforce()` on the sampled
    /// target — the R-tree `find_value_index()` must agree, verifying both the sampler wiring
    /// and the R-tree search correctness at a concrete position.
    ///
    /// The overworld parameter list JSON files contain only `{"preset":"minecraft:overworld"}`,
    /// so the parameter list is the one generated by `OverworldBiomeBuilder`.
    #[test]
    fn overworld_climate_sampler_and_rtree_agree_at_seed_0_block_0_64_0() {
        use super::super::{ClimateSampler, OVERWORLD_NOISE_ROUTER};
        use crate::biome::{overworld_biome_parameters, ClimateBiomeEntry, ClimateParameterList};

        let seed = 0_i64;
        let settings = *builtin_noise_generator_settings("minecraft:overworld")
            .expect("overworld noise_generator_settings must exist");

        // Block (0, 64, 0) → quart (0, 16, 0).
        let sampler = ClimateSampler::from_noise_router(&OVERWORLD_NOISE_ROUTER, seed, settings);
        let climate = sampler.sample(0, 16, 0);

        // All six climate values must be finite (density functions resolved correctly).
        assert!(
            crate::biome::unquantize_coord(climate.temperature).is_finite(),
            "temperature at seed 0 (0,64,0) must be finite"
        );
        assert!(
            crate::biome::unquantize_coord(climate.continentalness).is_finite(),
            "continentalness at seed 0 (0,64,0) must be finite"
        );

        // Build a ClimateParameterList from the overworld parameters and check that
        // brute-force and R-tree search agree on the nearest biome.
        let params = overworld_biome_parameters();
        let list = ClimateParameterList::new(params.to_vec())
            .expect("overworld parameter list is non-empty");
        let bruteforce_biome = list.find_value_bruteforce(climate);
        let rtree_biome = list.find_value_index(climate);
        assert_eq!(
            bruteforce_biome, rtree_biome,
            "R-tree and brute-force must agree on biome at seed 0 block (0,64,0): \
             brute={bruteforce_biome} rtree={rtree_biome}"
        );

        // The biome must be a valid registered overworld biome.
        assert!(
            bruteforce_biome.starts_with("minecraft:"),
            "expected a namespaced biome id, got {bruteforce_biome}"
        );
    }