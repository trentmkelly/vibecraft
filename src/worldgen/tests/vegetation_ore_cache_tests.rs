use super::*;

#[test]
    fn final_client_heightmaps_are_computed_from_blocks() {
        let mut chunk = crate::storage::chunk::LevelChunk::empty(ChunkPos { x: 0, z: 0 });
        let mut block_states = crate::storage::chunk::PalettedContainer::single(
            super::super::block_state_tag("minecraft:air"),
            crate::storage::chunk::SECTION_VOLUME,
        );
        block_states.set_entry(
            1 * 256 + 0 * 16 + 0,
            super::super::block_state_tag("minecraft:dirt"),
        );
        block_states.set_entry(
            2 * 256 + 0 * 16 + 0,
            super::super::block_state_tag("minecraft:oak_leaves"),
        );
        chunk.sections.push(crate::storage::chunk::ChunkSection {
            y: 0,
            block_states: block_states.to_nbt(),
            biomes: crate::storage::chunk::PalettedContainer::single(
                Tag::String("minecraft:plains".to_string()),
                crate::storage::chunk::BIOME_SECTION_VOLUME,
            )
            .to_nbt(),
            block_light: None,
            sky_light: None,
        });

        super::super::add_client_heightmaps_from_blocks(&mut chunk);

        let Tag::LongArray(world_surface) = chunk.heightmaps.get("WORLD_SURFACE").unwrap() else {
            panic!("WORLD_SURFACE should be stored as a long array");
        };
        let Tag::LongArray(motion_blocking) = chunk.heightmaps.get("MOTION_BLOCKING").unwrap()
        else {
            panic!("MOTION_BLOCKING should be stored as a long array");
        };
        let Tag::LongArray(motion_blocking_no_leaves) =
            chunk.heightmaps.get("MOTION_BLOCKING_NO_LEAVES").unwrap()
        else {
            panic!("MOTION_BLOCKING_NO_LEAVES should be stored as a long array");
        };

        assert_eq!(unpack_heightmap_column(world_surface, 0), 3);
        assert_eq!(unpack_heightmap_column(motion_blocking, 0), 3);
        assert_eq!(unpack_heightmap_column(motion_blocking_no_leaves, 0), 2);
    }

    #[test]
    fn placed_simple_vegetation_models_cover_common_plains_features() {
        let grass = super::super::placed_simple_vegetation_feature("minecraft:patch_grass_plain")
            .expect("patch_grass_plain should have a simple vegetation model");
        assert_eq!(grass.configured_feature, "minecraft:grass");
        assert!(matches!(
            grass.placement.as_slice(),
            [
                super::super::PlacementModifier::NoiseThresholdCount { .. },
                super::super::PlacementModifier::InSquare,
                super::super::PlacementModifier::Heightmap { .. },
                super::super::PlacementModifier::BiomeFilter,
                super::super::PlacementModifier::Count { count: 32 },
                super::super::PlacementModifier::RandomOffset { .. },
                super::super::PlacementModifier::BlockPredicateFilter { .. },
            ]
        ));

        let flower = super::super::placed_simple_vegetation_feature("flower_plains")
            .expect("flower_plains should have a simple vegetation model");
        assert_eq!(flower.configured_feature, "minecraft:flower_plain");
        let leaf_litter = super::super::placed_simple_vegetation_feature("patch_leaf_litter")
            .expect("patch_leaf_litter should share the simple vegetation executor");
        assert_eq!(leaf_litter.configured_feature, "minecraft:leaf_litter");
        assert!(matches!(
            leaf_litter.placement.as_slice(),
            [
                super::super::PlacementModifier::Count { count: 2 },
                super::super::PlacementModifier::InSquare,
                super::super::PlacementModifier::Heightmap {
                    heightmap: HeightmapKind::WorldSurface
                },
                super::super::PlacementModifier::BiomeFilter,
                super::super::PlacementModifier::Count { count: 32 },
                super::super::PlacementModifier::RandomOffset { .. },
                super::super::PlacementModifier::BlockPredicateFilter { .. },
            ]
        ));
        assert!(super::super::placed_simple_vegetation_feature("trees_plains").is_none());
    }

    #[test]
    fn simple_vegetation_phase_matches_vanilla_forest_feature_order() {
        assert_eq!(
            super::super::simple_vegetation_phase("minecraft:forest_flowers"),
            super::super::SimpleVegetationPhase::BeforeTrees,
            "forest.json lists forest_flowers before trees_birch_and_oak_leaf_litter"
        );
        assert_eq!(
            super::super::simple_vegetation_phase("minecraft:patch_bush"),
            super::super::SimpleVegetationPhase::AfterTrees,
            "forest.json lists patch_bush after trees_birch_and_oak_leaf_litter"
        );
        assert_eq!(
            super::super::simple_vegetation_phase("minecraft:flower_default"),
            super::super::SimpleVegetationPhase::AfterTrees,
            "forest.json lists flower_default after tree placement"
        );
    }

    #[test]
    fn configured_simple_vegetation_places_single_and_double_plants() {
        let mut chunk = LevelChunk::empty(ChunkPos { x: 0, z: 0 });
        chunk.min_section_y = 0;
        let mut block_states =
            PalettedContainer::single(super::super::block_state_tag("minecraft:air"), SECTION_VOLUME);
        block_states.set_entry(0, super::super::block_state_tag("minecraft:grass_block"));
        chunk.sections.push(ChunkSection {
            y: 0,
            block_states: block_states.to_nbt(),
            biomes: PalettedContainer::single(
                Tag::String("minecraft:plains".to_string()),
                BIOME_SECTION_VOLUME,
            )
            .to_nbt(),
            block_light: None,
            sky_light: None,
        });

        let settings = super::super::builtin_noise_generator_settings("overworld").unwrap();
        let mut random = super::super::RandomSourceKind::new(1, super::super::RandomAlgorithm::Xoroshiro);
        assert_eq!(
            super::super::place_configured_simple_vegetation_in_target_chunk(
                &mut chunk,
                settings,
                "minecraft:grass",
                BlockPos { x: 0, y: 1, z: 0 },
                &mut random,
            ),
            1
        );
        assert_eq!(
            chunk.get_block_state_name(0, 1, 0),
            Some("minecraft:short_grass")
        );

        chunk.set_block_state(1, 0, 0, "minecraft:grass_block");
        let mut random = super::super::RandomSourceKind::new(2, super::super::RandomAlgorithm::Xoroshiro);
        assert_eq!(
            super::super::place_configured_simple_vegetation_in_target_chunk(
                &mut chunk,
                settings,
                "minecraft:sunflower",
                BlockPos { x: 1, y: 1, z: 0 },
                &mut random,
            ),
            2
        );
        assert_eq!(
            chunk.get_block_state_name(1, 1, 0),
            Some("minecraft:sunflower")
        );
        assert_eq!(
            chunk.get_block_state_name(1, 2, 0),
            Some("minecraft:sunflower")
        );
    }

    #[test]
    fn simple_vegetation_air_filter_rejects_property_bearing_tree_logs() {
        let log = super::super::carver_static_block_name("minecraft:oak_log[axis=y]")
            .expect("tree logs with properties should normalize for placement predicates");
        let context = super::super::block_predicate_context_for_state(log, -64, 384);

        assert!(!super::super::block_predicate_test(
            super::super::BlockPredicate::MatchingBlockTag {
                tag: "minecraft:air"
            },
            context,
            64,
        ));
    }

    #[test]
    fn simple_vegetation_block_predicate_filter_reads_below_block() {
        let settings = super::super::builtin_noise_generator_settings("minecraft:overworld")
            .expect("overworld noise settings should exist");
        let mut chunk = LevelChunk::empty(ChunkPos { x: 0, z: 0 });
        chunk.min_section_y = 0;
        let mut block_states =
            PalettedContainer::single(super::super::block_state_tag("minecraft:air"), SECTION_VOLUME);
        block_states.set_entry(
            0 * 256 + 1 * 16 + 1,
            super::super::block_state_tag("minecraft:grass_block"),
        );
        chunk.sections.push(ChunkSection {
            y: 0,
            block_states: block_states.to_nbt(),
            biomes: PalettedContainer::single(
                Tag::String("minecraft:forest".to_string()),
                BIOME_SECTION_VOLUME,
            )
            .to_nbt(),
            block_light: None,
            sky_light: None,
        });

        let predicate = BlockPredicate::AllOf {
            predicates: &[
                BlockPredicate::MatchingBlockTag {
                    tag: "minecraft:air",
                },
                BlockPredicate::MatchingBlocksAt {
                    offset_y: -1,
                    blocks: &["minecraft:grass_block"],
                },
            ],
        };

        assert!(super::super::block_predicate_test_in_chunk(
            &chunk,
            settings,
            predicate,
            BlockPos { x: 1, y: 1, z: 1 },
        ));
        assert!(!super::super::block_predicate_test_in_chunk(
            &chunk,
            settings,
            predicate,
            BlockPos { x: 2, y: 1, z: 1 },
        ));
    }

    #[test]
    fn simple_vegetation_region_predicates_and_writes_cross_chunk_edges() {
        let settings = super::super::builtin_noise_generator_settings("minecraft:overworld")
            .expect("overworld noise settings should exist");
        let mut chunks = BTreeMap::new();
        let source_pos = ChunkPos { x: 0, z: 0 };
        let target_pos = ChunkPos { x: 1, z: 0 };
        chunks.insert(source_pos, LevelChunk::empty(source_pos));

        let mut target = LevelChunk::empty(target_pos);
        target.min_section_y = 0;
        let mut block_states =
            PalettedContainer::single(super::super::block_state_tag("minecraft:air"), SECTION_VOLUME);
        block_states.set_entry(0, super::super::block_state_tag("minecraft:grass_block"));
        target.sections.push(ChunkSection {
            y: 0,
            block_states: block_states.to_nbt(),
            biomes: PalettedContainer::single(
                Tag::String("minecraft:forest".to_string()),
                BIOME_SECTION_VOLUME,
            )
            .to_nbt(),
            block_light: None,
            sky_light: None,
        });
        chunks.insert(target_pos, target);

        let predicate = BlockPredicate::AllOf {
            predicates: &[
                BlockPredicate::MatchingBlockTag {
                    tag: "minecraft:air",
                },
                BlockPredicate::MatchingBlocksAt {
                    offset_y: -1,
                    blocks: &["minecraft:grass_block"],
                },
            ],
        };
        let edge_position = BlockPos { x: 16, y: 1, z: 0 };
        assert!(super::super::block_predicate_test_in_region(
            &chunks,
            settings,
            predicate,
            edge_position,
        ));

        let mut random = super::super::RandomSourceKind::new(1, super::super::RandomAlgorithm::Xoroshiro);
        assert_eq!(
            super::super::place_configured_simple_vegetation_in_region(
                &mut chunks,
                source_pos,
                settings,
                "minecraft:grass",
                edge_position,
                &mut random,
            ),
            1
        );
        assert_eq!(
            chunks
                .get(&target_pos)
                .and_then(|chunk| chunk.get_block_state_name(16, 1, 0)),
            Some("minecraft:short_grass")
        );

        let far_position = BlockPos { x: 32, y: 1, z: 0 };
        assert_eq!(
            super::super::place_configured_simple_vegetation_in_region(
                &mut chunks,
                source_pos,
                settings,
                "minecraft:grass",
                far_position,
                &mut random,
            ),
            0,
            "WorldGenRegion only permits feature writes within one chunk of the source"
        );
    }

    #[test]
    fn tree_final_write_filter_allows_trunks_to_replace_grass() {
        assert!(super::super::tree_placement_block_can_replace(
            TreePlacementBlockKind::Log,
            "minecraft:short_grass",
        ));
        assert!(super::super::tree_placement_block_can_replace(
            TreePlacementBlockKind::Log,
            "minecraft:tall_grass",
        ));
        assert!(!super::super::tree_placement_block_can_replace(
            TreePlacementBlockKind::Log,
            "minecraft:stone",
        ));
    }

    #[test]
    fn ore_block_cache_uses_world_coordinates_without_chunk_wrapping() {
        let pos = ChunkPos { x: 2, z: -3 };
        let origin_x = pos.x * 16;
        let origin_z = pos.z * 16;
        let mut chunk = LevelChunk::empty(pos);
        chunk.min_section_y = 0;

        let mut block_states =
            PalettedContainer::single(super::super::block_state_tag("minecraft:stone"), SECTION_VOLUME);
        block_states.set_entry(
            1 * 256 + 5 * 16 + 15,
            super::super::block_state_tag("minecraft:dirt"),
        );
        chunk.sections.push(ChunkSection {
            y: 0,
            block_states: block_states.to_nbt(),
            biomes: PalettedContainer::single(
                Tag::String("minecraft:plains".to_string()),
                BIOME_SECTION_VOLUME,
            )
            .to_nbt(),
            block_light: None,
            sky_light: None,
        });

        let mut block_cache = super::super::OreBlockCache::from_chunk(&chunk);
        assert_eq!(
            block_cache.block_state_name(origin_x + 15, 1, origin_z + 5),
            Some("minecraft:dirt")
        );
        assert_eq!(
            block_cache.block_state_name(origin_x - 1, 1, origin_z + 5),
            None,
            "neighboring chunk reads must not wrap onto local x=15"
        );

        block_cache.set_block_state(origin_x - 1, 1, origin_z + 5, "minecraft:gold_ore");
        block_cache.set_block_state(origin_x + 1, 1, origin_z + 1, "minecraft:iron_ore");
        block_cache.flush_to_chunk(&mut chunk);

        assert_eq!(
            chunk
                .get_block_state(origin_x + 15, 1, origin_z + 5)
                .as_deref(),
            Some("minecraft:dirt"),
            "neighboring chunk writes must not wrap onto local x=15"
        );
        assert_eq!(
            chunk
                .get_block_state(origin_x + 1, 1, origin_z + 1)
                .as_deref(),
            Some("minecraft:iron_ore")
        );
    }

    #[test]
    fn ore_region_cache_reads_and_flushes_neighboring_chunks() {
        let center_pos = ChunkPos { x: 2, z: -3 };
        let west_pos = ChunkPos { x: 1, z: -3 };
        let far_west_pos = ChunkPos { x: 0, z: -3 };
        let mut chunks = BTreeMap::new();
        for (pos, marker) in [
            (center_pos, "minecraft:stone"),
            (west_pos, "minecraft:dirt"),
            (far_west_pos, "minecraft:deepslate"),
        ] {
            let mut chunk = LevelChunk::empty(pos);
            chunk.min_section_y = 0;
            chunk.sections.push(ChunkSection {
                y: 0,
                block_states: PalettedContainer::single(
                    super::super::block_state_tag(marker),
                    SECTION_VOLUME,
                )
                .to_nbt(),
                biomes: PalettedContainer::single(
                    Tag::String("minecraft:plains".to_string()),
                    BIOME_SECTION_VOLUME,
                )
                .to_nbt(),
                block_light: None,
                sky_light: None,
            });
            chunks.insert(pos, chunk);
        }

        let center_min_x = center_pos.x * 16;
        let center_min_z = center_pos.z * 16;
        let west_world_x = center_min_x - 1;
        let far_west_world_x = center_min_x - 17;
        let world_z = center_min_z + 5;
        let mut block_cache =
            super::super::OreBlockCache::from_region_chunks(center_pos, &chunks).unwrap();

        assert_eq!(
            block_cache.block_state_name(west_world_x, 1, world_z),
            Some("minecraft:dirt")
        );
        assert_eq!(
            block_cache.block_state_name(far_west_world_x, 1, world_z),
            Some("minecraft:deepslate")
        );
        block_cache.set_block_state(west_world_x, 1, world_z, "minecraft:gold_ore");
        block_cache.set_block_state(far_west_world_x, 1, world_z, "minecraft:diamond_ore");
        block_cache.set_block_state(center_min_x + 1, 1, world_z, "minecraft:iron_ore");
        block_cache.flush_to_chunks(&mut chunks);

        assert_eq!(
            chunks[&west_pos]
                .get_block_state(west_world_x, 1, world_z)
                .as_deref(),
            Some("minecraft:gold_ore")
        );
        assert_eq!(
            chunks[&far_west_pos]
                .get_block_state(far_west_world_x, 1, world_z)
                .as_deref(),
            Some("minecraft:deepslate"),
            "FEATURES region writes must be rejected outside the source chunk's block-state write radius"
        );
        assert_eq!(
            chunks[&center_pos]
                .get_block_state(center_min_x + 1, 1, world_z)
                .as_deref(),
            Some("minecraft:iron_ore")
        );
    }

