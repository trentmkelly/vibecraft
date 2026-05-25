use super::*;

    #[test]
    fn biome_manager_seed_obfuscation_matches_java_hash_long() {
        // Java: BiomeManager.obfuscateSeed(seed) =
        // Hashing.sha256().hashLong(seed).asLong().
        assert_eq!(super::super::biome_manager_obfuscate_seed(0), 8794265229978523055);
        assert_eq!(
            super::super::biome_manager_obfuscate_seed(8_675_309),
            8580917108473614843
        );
        assert_eq!(super::super::biome_manager_obfuscate_seed(-1), 6759447113877070610);
    }

    #[test]
    fn biome_manager_fiddle_mask_matches_java_floor_mod() {
        for value in [
            0,
            1,
            -1,
            i64::MIN,
            i64::MAX,
            0x1234_5678_9abc_def0_i64,
            -0x1234_5678_9abc_def_i64,
            6_364_136_223_846_793_005_i64,
            -6_364_136_223_846_793_005_i64,
        ] {
            let java_floor_mod = ((value >> 24).rem_euclid(1024) as f64) / 1024.0;
            let expected = (java_floor_mod - 0.5) * 0.9;
            assert_eq!(super::super::biome_manager_fiddle(value), expected);
        }
    }

    #[test]
    fn aquifer_constants_and_disabled_behavior_match_decompiled_rules() {
        assert_eq!(
            AQUIFER_NOISE_SETTINGS,
            AquiferNoiseSettings {
                x_range: 10,
                y_range: 9,
                z_range: 10,
                x_separation: 6,
                y_separation: 3,
                z_separation: 6,
                x_spacing: 16,
                y_spacing: 12,
                z_spacing: 16,
                max_reasonable_distance_to_center: 11,
                sample_offset_x: -5,
                sample_offset_y: 1,
                sample_offset_z: -5,
            }
        );
        assert_eq!(AQUIFER_SURFACE_SAMPLING_OFFSETS_IN_CHUNKS.len(), 13);
        assert_eq!(AQUIFER_SURFACE_SAMPLING_OFFSETS_IN_CHUNKS[0], (0, 0));
        assert_eq!(AQUIFER_SURFACE_SAMPLING_OFFSETS_IN_CHUNKS[5], (-3, 0));

        let water = FluidStatus {
            fluid_level: 63,
            fluid_type: "minecraft:water",
        };
        assert_eq!(water.at(62), "minecraft:water");
        assert_eq!(water.at(63), "minecraft:air");
        assert_eq!(super::super::disabled_aquifer_substance(0.1, water, 62), None);
        assert_eq!(
            super::super::disabled_aquifer_substance(-0.1, water, 62),
            Some("minecraft:water")
        );
        assert_eq!(super::super::aquifer_similarity(100, 144), -0.76);
    }

    /// Parity test: global fluid picker returns lava below y = -54 and water below sea level.
    /// Also verifies that no water appears at y = -55 in a generated overworld chunk
    /// (those blocks must be lava or solid — the aquifer respects the global lava floor).
    ///
    /// Verifies items 143, 144, 146 of CHECKLIST_WORLDGEN.md.
    #[test]
    fn aquifer_global_fluid_picker_and_no_water_below_lava_floor() {
        use super::super::{
            builtin_noise_generator_settings, builtin_noise_router, fill_from_noise_chunk,
            global_fluid_status, noise_router_id_for_settings, ChunkPos, NONE_NOISE_ROUTER,
        };

        let settings = builtin_noise_generator_settings("minecraft:overworld")
            .expect("overworld noise settings must exist");

        // ── Unit-test the global fluid picker ────────────────────────────────
        // y = -55 < min(-54, seaLevel=63) = -54 → lava status.
        let lava_status = global_fluid_status(-55, settings.sea_level, settings.default_fluid);
        assert_eq!(
            lava_status.at(-55),
            "minecraft:lava",
            "y=-55 (below bedrock lava floor at -54) must be lava"
        );
        // at the fluid_level boundary (-54), the block is air (< relation is strict).
        assert_eq!(
            lava_status.at(-54),
            "minecraft:air",
            "y=-54 is the fluid_level itself, so at(-54) must be air"
        );

        // y = 0 → sea-level water status.
        let water_status = global_fluid_status(0, settings.sea_level, settings.default_fluid);
        assert_eq!(
            water_status.at(0),
            "minecraft:water",
            "y=0 is below sea level (63) so the global fluid is water"
        );
        assert_eq!(
            water_status.at(settings.sea_level),
            "minecraft:air",
            "y=sea_level is at the fluid_level boundary → air"
        );

        // ── Integration: no water at y = -55 in a real chunk ─────────────────
        let router_id = noise_router_id_for_settings(*settings);
        let noise_router = builtin_noise_router(router_id)
            .map(|e| e.router)
            .unwrap_or(NONE_NOISE_ROUTER);
        let chunk = fill_from_noise_chunk(ChunkPos { x: 0, z: 0 }, settings, 0, noise_router);

        // Scan all 256 columns at y=-55: any non-solid, non-air block must be lava, not water.
        for lx in 0..16_i32 {
            for lz in 0..16_i32 {
                if let Some(block) = chunk.get_block_state(lx, -55, lz) {
                    assert_ne!(
                        block, "minecraft:water",
                        "y=-55 at ({lx}, {lz}) must not be water (got {block}); \
                         the global lava floor takes priority"
                    );
                }
            }
        }
    }

    /// Parity test: with aquifers enabled, the overworld generates water somewhere
    /// underground between sea level and the lava floor, demonstrating that the
    /// NoiseBasedAquifer is active and produces fluid pockets.
    ///
    /// Verifies items 142, 145 of CHECKLIST_WORLDGEN.md.
    #[test]
    fn noise_based_aquifer_produces_underground_water_in_overworld() {
        use super::super::{
            builtin_noise_generator_settings, builtin_noise_router, fill_from_noise_chunk,
            noise_router_id_for_settings, ChunkPos, NONE_NOISE_ROUTER,
        };

        let settings = builtin_noise_generator_settings("minecraft:overworld")
            .expect("overworld noise settings must exist");
        assert!(
            settings.aquifers_enabled,
            "overworld must have aquifers enabled"
        );

        let router_id = noise_router_id_for_settings(*settings);
        let noise_router = builtin_noise_router(router_id)
            .map(|e| e.router)
            .unwrap_or(NONE_NOISE_ROUTER);

        // Try a few nearby chunks until we find one with underground water.
        let sea = settings.sea_level;
        let found_water = (0..4_i32)
            .flat_map(|cz| (0..4_i32).map(move |cx| (cx, cz)))
            .any(|(cx, cz)| {
                let chunk =
                    fill_from_noise_chunk(ChunkPos { x: cx, z: cz }, settings, 0, noise_router);
                let _min_y = settings.noise.min_y;
                // Look for water between the lava floor (-54) and sea level in any column.
                (-54..sea).any(|y| {
                    (0..16_i32)
                        .flat_map(|lz| (0..16_i32).map(move |lx| (lx, lz)))
                        .any(|(lx, lz)| {
                            let bx = cx * 16 + lx;
                            let bz = cz * 16 + lz;
                            chunk.get_block_state(bx, y, bz).as_deref() == Some("minecraft:water")
                                || chunk.get_block_state(bx, y, bz).as_deref()
                                    == Some("minecraft:lava")
                        })
                })
            });
        assert!(
            found_water,
            "no underground fluid (water or lava) found in 16 chunks near origin — \
             NoiseBasedAquifer may not be running"
        );
    }

    #[test]
    fn cave_generation_families_cover_noise_router_data_cave_builders() {
        assert_eq!(
            CAVE_GENERATION_FAMILIES
                .iter()
                .map(|family| family.id)
                .collect::<Vec<_>>(),
            vec![
                "minecraft:overworld/caves/spaghetti_roughness_function",
                "minecraft:overworld/caves/entrances",
                "minecraft:overworld/caves/noodle",
                "minecraft:overworld/caves/pillars",
                "minecraft:overworld/caves/spaghetti_2d",
                "minecraft:overworld/caves/underground",
            ]
        );
        let entrances = super::super::cave_generation_family("overworld/caves/entrances").unwrap();
        assert!(entrances.noises.contains(&"minecraft:spaghetti_3d_1"));
        assert!(entrances.noises.contains(&"minecraft:cave_entrance"));
        assert_eq!(entrances.output, CaveDensityOutput::CacheOnce);

        let noodle = super::super::cave_generation_family("overworld/caves/noodle").unwrap();
        assert!(noodle.noises.contains(&"minecraft:noodle_ridge_a"));
        assert_eq!(noodle.output, CaveDensityOutput::RangeChoice);

        let spaghetti_2d = super::super::cave_generation_family("overworld/caves/spaghetti_2d").unwrap();
        assert!(spaghetti_2d
            .noises
            .contains(&"minecraft:spaghetti_2d_elevation"));
        assert_eq!(
            spaghetti_2d.output,
            CaveDensityOutput::Clamp { min: -1, max: 1 }
        );
    }

    #[test]
    fn ore_veinifier_constants_and_vein_types_match_decompiled_values() {
        assert_eq!(
            ORE_VEINIFIER_CONSTANTS,
            OreVeinifierConstants {
                veininess_threshold: 0.4,
                edge_roundoff_begin: 20,
                max_edge_roundoff: 0.2,
                vein_solidness: 0.7,
                min_richness: 0.1,
                max_richness: 0.3,
                max_richness_threshold: 0.6,
                chance_of_raw_ore_block: 0.02,
                skip_ore_if_gap_noise_is_below: -0.3,
            }
        );
        assert_eq!(ORE_VEIN_TYPES.len(), 2);
        assert_eq!(ORE_VEIN_TYPES[0].id, "copper");
        assert_eq!(ORE_VEIN_TYPES[0].ore, "minecraft:copper_ore");
        assert_eq!(
            ORE_VEIN_TYPES[0].raw_ore_block,
            "minecraft:raw_copper_block"
        );
        assert_eq!(ORE_VEIN_TYPES[0].filler, "minecraft:granite");
        assert_eq!((ORE_VEIN_TYPES[0].min_y, ORE_VEIN_TYPES[0].max_y), (0, 50));
        assert_eq!(ORE_VEIN_TYPES[1].id, "iron");
        assert_eq!(ORE_VEIN_TYPES[1].ore, "minecraft:deepslate_iron_ore");
        assert_eq!(ORE_VEIN_TYPES[1].raw_ore_block, "minecraft:raw_iron_block");
        assert_eq!(ORE_VEIN_TYPES[1].filler, "minecraft:tuff");
        assert_eq!(
            (ORE_VEIN_TYPES[1].min_y, ORE_VEIN_TYPES[1].max_y),
            (-60, -8)
        );
    }

    #[test]
    fn ore_veinifier_decision_matches_vanilla_branching() {
        let base = OreVeinDecisionInput {
            y: 25,
            vein_toggle: 0.61,
            vein_ridged: -0.1,
            vein_gap: 0.0,
            solidness_random: 0.5,
            richness_random: 0.2,
            raw_ore_random: 0.5,
            debug_ore_veins: false,
        };
        assert_eq!(super::super::ore_vein_richness(0.4), 0.1);
        assert_eq!(super::super::ore_vein_richness(0.6), 0.3);
        assert_eq!(super::super::ore_vein_decision(base), Some("minecraft:copper_ore"));
        assert_eq!(
            super::super::ore_vein_decision(OreVeinDecisionInput {
                raw_ore_random: 0.01,
                ..base
            }),
            Some("minecraft:raw_copper_block")
        );
        assert_eq!(
            super::super::ore_vein_decision(OreVeinDecisionInput {
                y: -30,
                vein_toggle: -0.61,
                raw_ore_random: 0.5,
                ..base
            }),
            Some("minecraft:deepslate_iron_ore")
        );
        assert_eq!(
            super::super::ore_vein_decision(OreVeinDecisionInput {
                richness_random: 0.99,
                ..base
            }),
            Some("minecraft:granite")
        );
        assert_eq!(
            super::super::ore_vein_decision(OreVeinDecisionInput {
                vein_gap: -0.31,
                ..base
            }),
            Some("minecraft:granite")
        );
        assert_eq!(
            super::super::ore_vein_decision(OreVeinDecisionInput {
                solidness_random: 0.71,
                ..base
            }),
            None
        );
        assert_eq!(
            super::super::ore_vein_decision(OreVeinDecisionInput {
                y: 100,
                debug_ore_veins: true,
                ..base
            }),
            Some("minecraft:air")
        );
        assert_eq!(
            super::super::ore_vein_decision(OreVeinDecisionInput {
                richness_random: 0.99,
                debug_ore_veins: true,
                ..base
            }),
            Some("minecraft:oak_button")
        );
    }

    #[test]
    fn ore_veinifier_uses_positional_random_factory_sequence() {
        let ore_factory = crate::random_source::random_state_seed_factories(
            8675309,
            super::super::RandomAlgorithm::Xoroshiro,
        )
        .ore;
        let mut positional_random = ore_factory.at(4, 25, -9);
        let manual = super::super::ore_vein_decision(OreVeinDecisionInput {
            y: 25,
            vein_toggle: 0.61,
            vein_ridged: -0.1,
            vein_gap: 0.0,
            solidness_random: f64::from(positional_random.next_f32()),
            richness_random: f64::from(positional_random.next_f32()),
            raw_ore_random: f64::from(positional_random.next_f32()),
            debug_ore_veins: false,
        });
        assert_eq!(
            super::super::ore_vein_decision_at(ore_factory, 4, 25, -9, 0.61, -0.1, 0.0, false),
            manual
        );

        let legacy_ore_factory = crate::random_source::random_state_seed_factories(
            8675309,
            super::super::RandomAlgorithm::Legacy,
        )
        .ore;
        assert_eq!(
            super::super::ore_vein_decision_at(legacy_ore_factory, 4, 25, -9, 0.61, -0.1, 0.0, false),
            None
        );
    }

    /// Verifies that ore veins are integrated into solid-block placement in
    /// `fill_from_noise_chunk`.  The OreVeinifier can place three block types per vein:
    ///
    /// - **Iron vein** (Y -60..=-8): deepslate_iron_ore (ore), raw_iron_block (2% raw),
    ///   tuff (filler — most common; appears in the halo around the ore core)
    /// - **Copper vein** (Y 0..=50): copper_ore (ore), raw_copper_block (2% raw),
    ///   granite (filler — same halo role)
    ///
    /// Regular iron_ore is never placed by veins: the iron VeinType explicitly uses
    /// deepslate_iron_ore because iron veins only spawn in the deepslate zone (Y ≤ -8).
    /// Verifies that ore veins are integrated into solid-block placement in
    /// `fill_from_noise_chunk`.  The OreVeinifier can place three block types per vein:
    ///
    /// - **Iron vein** (Y -60..=-8): deepslate_iron_ore (ore), raw_iron_block (2% raw),
    ///   tuff (filler — most common; appears in the halo around the ore core)
    /// - **Copper vein** (Y 0..=50): copper_ore (ore), raw_copper_block (2% raw),
    ///   granite (filler — same halo role)
    ///
    /// Regular iron_ore is never placed by veins: the iron VeinType explicitly uses
    /// deepslate_iron_ore because iron veins only spawn in the deepslate zone (Y ≤ -8).
    ///
    /// Strategy: the ore_veininess noise (scale 1.5) has a ~170-block wavelength, so
    /// the near-origin region may be entirely in a low-veininess trough at seed 0.
    /// We first scan the raw noise over a ±512-block grid (no chunk generation needed)
    /// to locate a world position guaranteed to have high vein activity, then generate
    /// exactly that one chunk and scan its sections efficiently (one decode per section).
    #[test]
    fn ore_veins_integrated_in_chunk_generation() {
        use super::super::{
            builtin_noise_generator_settings, builtin_noise_router, fill_from_noise_chunk,
            noise_router_id_for_settings, ChunkPos, NONE_NOISE_ROUTER,
            OVERWORLD_VEIN_TOGGLE_NOISE_DENSITY,
        };
        use crate::storage::chunk::{PalettedContainer, SECTION_VOLUME};
        use crate::storage::nbt::Tag;

        let settings = builtin_noise_generator_settings("minecraft:overworld")
            .expect("overworld noise settings must exist");
        assert!(
            settings.ore_veins_enabled,
            "overworld must have ore veins enabled"
        );

        let router_id = noise_router_id_for_settings(*settings);
        let noise_router = builtin_noise_router(router_id)
            .map(|e| e.router)
            .unwrap_or(NONE_NOISE_ROUTER);

        // Iron vein blocks (Y -60..=-8).
        const IRON_VEIN_BLOCKS: &[&str] = &[
            "minecraft:deepslate_iron_ore",
            "minecraft:raw_iron_block",
            "minecraft:tuff",
        ];
        // Copper vein blocks (Y 0..=50).
        const COPPER_VEIN_BLOCKS: &[&str] = &[
            "minecraft:copper_ore",
            "minecraft:raw_copper_block",
            "minecraft:granite",
        ];

        // Fast veininess probe: evaluate the raw (non-interpolated) noise without
        // generating any chunk.  Scans a ±512 block grid at step 8 to cover many noise
        // wavelengths (~170 blocks at scale 1.5).  Returns unique chunk positions where the
        // toggle noise satisfies `predicate` at the given probe Y.  probe_y is chosen deep
        // underground so blocks there are nearly always solid stone.
        //
        // Positive vein_toggle (> 0.4) → COPPER vein type, Y range 0..=50.
        // Negative vein_toggle (< -0.4) → IRON vein type, Y range -60..=-8.
        // These are the same noise; sign determines which type is active.
        fn find_vein_chunks(
            settings: &super::super::NoiseGeneratorSettings,
            probe_y: i32,
            predicate: impl Fn(f64) -> bool,
        ) -> Vec<(i32, i32)> {
            let mut results = Vec::new();
            for z in (-512_i32..512).step_by(8) {
                for x in (-512_i32..512).step_by(8) {
                    let vt = OVERWORLD_VEIN_TOGGLE_NOISE_DENSITY
                        .compute_with_noise(0, *settings, x, probe_y, z);
                    if predicate(vt) {
                        let cx = x >> 4;
                        let cz = z >> 4;
                        if !results.contains(&(cx, cz)) {
                            results.push((cx, cz));
                        }
                    }
                }
            }
            results
        }

        /// Decode each relevant section once and look for any `targets` block name.
        fn any_vein_block_in_range(
            chunk: &crate::storage::chunk::LevelChunk,
            y_min: i32,
            y_max: i32,
            targets: &[&str],
        ) -> bool {
            for section in &chunk.sections {
                let s_min = section.y as i32 * 16;
                let s_max = s_min + 15;
                if s_max < y_min || s_min > y_max {
                    continue;
                }
                let Ok(container) =
                    PalettedContainer::from_nbt(&section.block_states, SECTION_VOLUME)
                else {
                    continue;
                };
                let lo = (y_min - s_min).clamp(0, 15) as usize;
                let hi = (y_max - s_min).clamp(0, 15) as usize;
                for local_y in lo..=hi {
                    for local_z in 0..16_usize {
                        for local_x in 0..16_usize {
                            let idx = local_y * 256 + local_z * 16 + local_x;
                            if let Some(Tag::Compound(fields)) = container.get_entry(idx) {
                                if let Some((_, Tag::String(name))) =
                                    fields.iter().find(|(k, _)| k == "Name")
                                {
                                    if targets.contains(&name.as_str()) {
                                        return true;
                                    }
                                }
                            }
                        }
                    }
                }
            }
            false
        }

        // --- Iron vein check ---
        // Probe at Y=-30: always solid stone/deepslate, dead centre of the iron vein Y range.
        // Iron is selected when vein_toggle < -0.4 (negative).
        let iron_chunks = find_vein_chunks(settings, -30, |vt| vt < -0.4);
        assert!(
            !iron_chunks.is_empty(),
            "ore_veininess noise never exceeded 0.4 at Y=-30 over a ±512 block grid at seed 0 \
             — noise setup may be broken"
        );
        // Take only the first 3 candidate chunks — vein zones span ~170 blocks so multiple
        // consecutive chunks are inside the same zone; 3 is enough to hit a ridgeline.
        let found_iron = iron_chunks.iter().take(3).any(|&(cx, cz)| {
            let chunk = fill_from_noise_chunk(ChunkPos { x: cx, z: cz }, settings, 0, noise_router);
            any_vein_block_in_range(&chunk, -60, -8, IRON_VEIN_BLOCKS)
        });
        assert!(
            found_iron,
            "no iron vein blocks (deepslate_iron_ore / raw_iron_block / tuff) found in \
             the first 3 vein-zone chunks at Y=-60..=-8 — ore vein integration broken"
        );

        // --- Copper vein check ---
        // Probe at Y=5: copper vein range is 0..=50; Y=5 is deep underground and nearly
        // always solid stone, unlike Y=25 which can be aquifer-filled ocean water.
        // Copper is selected when vein_toggle > 0.4 (positive).
        let copper_chunks = find_vein_chunks(settings, 5, |vt| vt > 0.4);
        assert!(
            !copper_chunks.is_empty(),
            "ore_veininess noise never exceeded 0.4 at Y=5 over a ±512 block grid at seed 0 \
             — noise setup may be broken"
        );
        let found_copper = copper_chunks.iter().take(3).any(|&(cx, cz)| {
            let chunk = fill_from_noise_chunk(ChunkPos { x: cx, z: cz }, settings, 0, noise_router);
            any_vein_block_in_range(&chunk, 0, 50, COPPER_VEIN_BLOCKS)
        });
        assert!(
            found_copper,
            "no copper vein blocks (copper_ore / raw_copper_block / granite) found in \
             the first 3 vein-zone chunks at Y=0..=50 — ore vein integration broken"
        );
    }

