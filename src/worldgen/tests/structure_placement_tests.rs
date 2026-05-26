use super::*;

#[test]
fn structure_registries_and_sets_match_vanilla_bootstrap() {
    assert_eq!(STRUCTURE_TYPES.len(), 16);
    assert_eq!(BUILTIN_STRUCTURES.len(), 34);
    assert_eq!(BUILTIN_STRUCTURE_SETS.len(), 20);
    assert_eq!(
        BUILTIN_STRUCTURE_SETS
            .iter()
            .map(|set| set.id)
            .collect::<Vec<_>>(),
        vec![
            "minecraft:villages",
            "minecraft:desert_pyramids",
            "minecraft:igloos",
            "minecraft:jungle_temples",
            "minecraft:swamp_huts",
            "minecraft:pillager_outposts",
            "minecraft:ancient_cities",
            "minecraft:ocean_monuments",
            "minecraft:woodland_mansions",
            "minecraft:buried_treasures",
            "minecraft:mineshafts",
            "minecraft:ruined_portals",
            "minecraft:shipwrecks",
            "minecraft:ocean_ruins",
            "minecraft:nether_complexes",
            "minecraft:nether_fossils",
            "minecraft:end_cities",
            "minecraft:strongholds",
            "minecraft:trail_ruins",
            "minecraft:trial_chambers",
        ]
    );

    let villages = &BUILTIN_STRUCTURE_SETS[0];
    assert_eq!(villages.structures.len(), 5);
    assert_eq!(
        villages.placement,
        StructurePlacementKind::RandomSpread {
            spacing: 34,
            separation: 8,
            salt: 10387312,
            spread_type: RandomSpreadType::Linear,
        }
    );

    let strongholds = BUILTIN_STRUCTURE_SETS
        .iter()
        .find(|set| set.id == "minecraft:strongholds")
        .unwrap();
    assert_eq!(
        strongholds.placement,
        StructurePlacementKind::ConcentricRings {
            distance: 32,
            spread: 3,
            count: 128,
        }
    );

    let mansions = BUILTIN_STRUCTURE_SETS
        .iter()
        .find(|set| set.id == "minecraft:woodland_mansions")
        .unwrap();
    assert_eq!(
        mansions.placement,
        StructurePlacementKind::RandomSpread {
            spacing: 80,
            separation: 20,
            salt: 10387319,
            spread_type: RandomSpreadType::Triangular,
        }
    );
}

#[test]
fn random_spread_structure_placement_uses_vanilla_grid_and_salt_math() {
    assert_eq!(
        super::super::validate_random_spread_placement(32, 32).unwrap_err(),
        "Spacing has to be larger than separation".to_string()
    );
    assert_eq!(
        super::super::validate_random_spread_placement(4097, 0).unwrap_err(),
        "Random spread spacing and separation must be in 0..=4096".to_string()
    );

    let village = super::super::random_spread_potential_structure_chunk(
        12345,
        0,
        0,
        34,
        8,
        10387312,
        RandomSpreadType::Linear,
    )
    .unwrap();
    assert_eq!(village, ChunkPos { x: 21, z: 5 });
    assert!(!super::super::random_spread_is_placement_chunk(
        12345,
        0,
        0,
        34,
        8,
        10387312,
        RandomSpreadType::Linear,
    )
    .unwrap());
    assert!(super::super::random_spread_is_placement_chunk(
        12345,
        village.x,
        village.z,
        34,
        8,
        10387312,
        RandomSpreadType::Linear,
    )
    .unwrap());

    assert_eq!(
        super::super::random_spread_potential_structure_chunk(
            12345,
            -1,
            -1,
            80,
            20,
            10387319,
            RandomSpreadType::Triangular,
        )
        .unwrap(),
        ChunkPos { x: -37, z: -54 }
    );
}

#[test]
fn structure_frequency_reducers_match_vanilla_methods() {
    assert_eq!(
        super::super::FrequencyReductionMethod::Default.id(),
        "default"
    );
    assert_eq!(
        super::super::FrequencyReductionMethod::LegacyType1.id(),
        "legacy_type_1"
    );
    assert_eq!(
        super::super::FrequencyReductionMethod::LegacyType2.id(),
        "legacy_type_2"
    );
    assert_eq!(
        super::super::FrequencyReductionMethod::LegacyType3.id(),
        "legacy_type_3"
    );
    assert_eq!(
        super::super::validate_structure_frequency(1.25).unwrap_err(),
        "Structure placement frequency must be in 0.0..=1.0".to_string()
    );

    assert!(super::super::structure_frequency_reducer_should_generate(
        super::super::FrequencyReductionMethod::Default,
        12345,
        10387312,
        21,
        5,
        1.0,
    )
    .unwrap());
    assert!(!super::super::structure_frequency_reducer_should_generate(
        super::super::FrequencyReductionMethod::Default,
        12345,
        10387312,
        21,
        5,
        0.0,
    )
    .unwrap());
    assert!(!super::super::structure_frequency_reducer_should_generate(
        super::super::FrequencyReductionMethod::Default,
        12345,
        10387312,
        21,
        5,
        0.5,
    )
    .unwrap());
    assert!(super::super::structure_frequency_reducer_should_generate(
        super::super::FrequencyReductionMethod::LegacyType1,
        12345,
        10387312,
        21,
        5,
        0.5,
    )
    .unwrap());
    assert!(super::super::structure_frequency_reducer_should_generate(
        super::super::FrequencyReductionMethod::LegacyType2,
        12345,
        10387312,
        21,
        5,
        0.5,
    )
    .unwrap());
    assert!(!super::super::structure_frequency_reducer_should_generate(
        super::super::FrequencyReductionMethod::LegacyType3,
        12345,
        10387312,
        21,
        5,
        0.5,
    )
    .unwrap());
}

#[test]
fn structure_locate_pos_uses_chunk_min_block_and_validated_offset() {
    assert_eq!(
        super::super::structure_locate_pos(
            ChunkPos { x: 21, z: -5 },
            BlockPos { x: 8, y: 0, z: 8 }
        )
        .unwrap(),
        BlockPos {
            x: 344,
            y: 0,
            z: -72
        }
    );
    assert_eq!(
        super::super::structure_locate_pos(
            ChunkPos { x: -2, z: 3 },
            BlockPos {
                x: -16,
                y: 16,
                z: 16
            }
        )
        .unwrap(),
        BlockPos {
            x: -48,
            y: 16,
            z: 64
        }
    );
    assert_eq!(
        super::super::structure_locate_pos(ChunkPos { x: 0, z: 0 }, BlockPos { x: 17, y: 0, z: 0 })
            .unwrap_err(),
        "Structure locate offset components must be in -16..=16".to_string()
    );
}

#[test]
fn concentric_rings_initial_candidates_follow_vanilla_ring_progression() {
    assert_eq!(
        super::super::validate_concentric_rings_placement(1024, 3, 128).unwrap_err(),
        "Concentric rings distance and spread must be in 0..=1023".to_string()
    );
    assert_eq!(
        super::super::validate_concentric_rings_placement(32, 3, 0).unwrap_err(),
        "Concentric rings count must be in 1..=4095".to_string()
    );

    let candidates = super::super::concentric_ring_initial_candidates(12345, 32, 3, 8).unwrap();
    assert_eq!(candidates.len(), 8);
    assert_eq!(
        candidates[0],
        super::super::ConcentricRingPlacementCandidate {
            index: 0,
            circle: 0,
            chunk_pos: ChunkPos { x: -105, z: 124 },
        }
    );
    assert_eq!(
        candidates[2],
        super::super::ConcentricRingPlacementCandidate {
            index: 2,
            circle: 0,
            chunk_pos: ChunkPos { x: 114, z: 21 },
        }
    );
    assert_eq!(candidates[3].circle, 1);
    let ring_positions = candidates
        .iter()
        .map(|candidate| candidate.chunk_pos)
        .collect::<Vec<_>>();
    assert!(super::super::concentric_rings_is_placement_chunk(
        &ring_positions,
        -105,
        124
    ));
    assert!(!super::super::concentric_rings_is_placement_chunk(
        &ring_positions,
        0,
        0
    ));

    assert_eq!(
        super::super::concentric_ring_candidate_search_center(candidates[0]),
        BlockPos {
            x: -1672,
            y: 0,
            z: 1992,
        }
    );
    assert_eq!(
        super::super::concentric_ring_adjusted_position(
            candidates[0],
            Some(super::super::ConcentricRingBiomeSearchResult {
                block_x: -1601,
                block_z: 2047,
            }),
        ),
        ChunkPos { x: -101, z: 127 }
    );
    assert_eq!(
        super::super::concentric_ring_adjusted_position(candidates[1], None),
        candidates[1].chunk_pos
    );
    assert_eq!(
        super::super::concentric_ring_adjusted_positions(
            &candidates[0..3],
            &[
                Some(super::super::ConcentricRingBiomeSearchResult {
                    block_x: -1601,
                    block_z: 2047,
                }),
                None,
                Some(super::super::ConcentricRingBiomeSearchResult {
                    block_x: 0,
                    block_z: -1,
                }),
            ],
        ),
        vec![
            ChunkPos { x: -101, z: 127 },
            candidates[1].chunk_pos,
            ChunkPos { x: 0, z: -1 },
        ]
    );
}

#[test]
fn structure_exclusion_zone_checks_square_chunk_range() {
    let zone = super::super::StructureExclusionZoneModel {
        other_set: "minecraft:villages",
        chunk_count: 3,
    };
    assert_eq!(
        super::super::validate_structure_exclusion_zone(zone),
        Ok(zone)
    );
    assert_eq!(
        super::super::validate_structure_exclusion_zone(
            super::super::StructureExclusionZoneModel {
                other_set: "minecraft:villages",
                chunk_count: 17,
            }
        )
        .unwrap_err(),
        "Structure exclusion zone chunk_count must be in 1..=16".to_string()
    );

    let other_chunks = [
        ChunkPos { x: 10, z: -4 },
        ChunkPos { x: -12, z: 8 },
        ChunkPos { x: 40, z: 40 },
    ];
    assert!(super::super::structure_has_chunk_in_range(
        &other_chunks,
        7,
        -1,
        3
    ));
    assert!(!super::super::structure_has_chunk_in_range(
        &other_chunks,
        6,
        -1,
        3
    ));
    assert!(super::super::structure_exclusion_zone_forbids(zone, &other_chunks, 7, -1).unwrap());
    assert!(!super::super::structure_exclusion_zone_forbids(zone, &other_chunks, 0, 0).unwrap());
}

#[test]
fn chunk_generator_structure_state_caches_placeable_sets_and_ring_positions() {
    let placeable = ["minecraft:stronghold", "minecraft:village_plains"];
    let mut normal = super::super::ChunkGeneratorStructureStateModel::create_for_normal(
        12345,
        super::super::BUILTIN_STRUCTURE_SETS,
        &placeable,
    );
    assert_eq!(normal.level_seed, 12345);
    assert_eq!(normal.concentric_rings_seed, 12345);
    assert!(normal
        .possible_structure_sets
        .iter()
        .any(|set| set.id == "minecraft:villages"));
    assert!(normal
        .possible_structure_sets
        .iter()
        .any(|set| set.id == "minecraft:strongholds"));
    assert!(!normal
        .possible_structure_sets
        .iter()
        .any(|set| set.id == "minecraft:desert_pyramids"));

    assert_eq!(
        normal.get_placements_for_structure("minecraft:village_plains", &placeable),
        vec![super::super::StructurePlacementKind::RandomSpread {
            spacing: 34,
            separation: 8,
            salt: 10387312,
            spread_type: super::super::RandomSpreadType::Linear,
        }]
    );
    assert!(normal
        .get_placements_for_structure("minecraft:desert_pyramid", &placeable)
        .is_empty());
    let rings = normal
        .get_ring_positions_for("minecraft:strongholds", &placeable)
        .unwrap();
    assert_eq!(rings.len(), 128);
    assert_eq!(rings[0], ChunkPos { x: -105, z: 124 });
    assert!(normal
        .has_structure_chunk_in_range("minecraft:strongholds", -105, 124, 0, &placeable)
        .unwrap());

    let village_chunk = super::super::random_spread_potential_structure_chunk(
        normal.level_seed,
        0,
        0,
        34,
        8,
        10387312,
        super::super::RandomSpreadType::Linear,
    )
    .unwrap();
    assert!(normal
        .has_structure_chunk_in_range(
            "minecraft:villages",
            village_chunk.x - 1,
            village_chunk.z,
            1,
            &placeable,
        )
        .unwrap());
    assert!(!normal
        .has_structure_chunk_in_range("minecraft:desert_pyramids", 0, 0, 10, &placeable)
        .unwrap());

    let mut flat = super::super::ChunkGeneratorStructureStateModel::create_for_flat(
        12345,
        super::super::BUILTIN_STRUCTURE_SETS,
        &["minecraft:stronghold"],
    );
    assert_eq!(flat.concentric_rings_seed, 0);
    assert_ne!(
        flat.get_ring_positions_for("minecraft:strongholds", &["minecraft:stronghold"])
            .unwrap()[0],
        rings[0]
    );
}

#[test]
fn structure_start_validity_references_tags_and_piece_queries_match_vanilla() {
    let invalid = super::super::StructureStartModel::invalid();
    assert!(!invalid.is_valid());
    assert_eq!(
        invalid.create_tag(ChunkPos { x: 4, z: -7 }),
        super::super::StructureStartTagModel {
            id: "INVALID",
            chunk_x: None,
            chunk_z: None,
            references: None,
            children: 0,
        }
    );

    let first_piece = super::super::StructurePieceModel {
        bounding_box: super::super::StructureBoundingBoxModel {
            min_x: 32,
            min_y: 20,
            min_z: -16,
            max_x: 47,
            max_y: 35,
            max_z: -1,
        },
    };
    let second_piece = super::super::StructurePieceModel {
        bounding_box: super::super::StructureBoundingBoxModel {
            min_x: 48,
            min_y: 18,
            min_z: -8,
            max_x: 63,
            max_y: 30,
            max_z: 7,
        },
    };
    let mut start = super::super::StructureStartModel {
        structure: Some("minecraft:village_plains"),
        chunk_pos: ChunkPos { x: 2, z: -1 },
        references: 0,
        pieces: vec![first_piece, second_piece],
    };

    assert!(start.is_valid());
    assert!(start.can_be_referenced());
    start.add_reference();
    assert_eq!(start.references, 1);
    assert!(!start.can_be_referenced());
    assert_eq!(
        start.bounding_box(),
        Some(super::super::StructureBoundingBoxModel {
            min_x: 32,
            min_y: 18,
            min_z: -16,
            max_x: 63,
            max_y: 35,
            max_z: 7,
        })
    );
    assert_eq!(
        start.create_tag(ChunkPos { x: 2, z: -1 }),
        super::super::StructureStartTagModel {
            id: "minecraft:village_plains",
            chunk_x: Some(2),
            chunk_z: Some(-1),
            references: Some(1),
            children: 2,
        }
    );
    assert_eq!(
        super::super::structure_start_reference_pos(first_piece),
        BlockPos {
            x: 40,
            y: 20,
            z: -8,
        }
    );
    assert_eq!(
        super::super::structure_pieces_intersecting_chunk(
            &start,
            super::super::StructureBoundingBoxModel {
                min_x: 48,
                min_y: -64,
                min_z: -16,
                max_x: 63,
                max_y: 320,
                max_z: -1,
            }
        ),
        vec![second_piece]
    );
}

#[test]
fn structure_manager_position_queries_use_union_and_piece_boxes() {
    let first_piece = super::super::StructurePieceModel {
        bounding_box: super::super::StructureBoundingBoxModel {
            min_x: 0,
            min_y: 10,
            min_z: 0,
            max_x: 4,
            max_y: 20,
            max_z: 4,
        },
    };
    let second_piece = super::super::StructurePieceModel {
        bounding_box: super::super::StructureBoundingBoxModel {
            min_x: 10,
            min_y: 10,
            min_z: 10,
            max_x: 14,
            max_y: 20,
            max_z: 14,
        },
    };
    let start = super::super::StructureStartModel {
        structure: Some("minecraft:stronghold"),
        chunk_pos: ChunkPos { x: 0, z: 0 },
        references: 0,
        pieces: vec![first_piece, second_piece],
    };
    let inside_first_edge = BlockPos { x: 4, y: 20, z: 4 };
    let inside_union_gap = BlockPos { x: 7, y: 15, z: 7 };
    let outside = BlockPos {
        x: 15,
        y: 15,
        z: 15,
    };

    assert!(first_piece.bounding_box.is_inside(inside_first_edge));
    assert!(super::super::structure_start_contains_pos(
        inside_first_edge,
        &start
    ));
    assert!(super::super::structure_has_piece_at(
        inside_first_edge,
        &start
    ));

    assert!(super::super::structure_start_contains_pos(
        inside_union_gap,
        &start
    ));
    assert!(!super::super::structure_has_piece_at(
        inside_union_gap,
        &start
    ));
    assert!(!super::super::structure_start_contains_pos(outside, &start));
    assert!(!super::super::structure_has_piece_at(outside, &start));

    let invalid = super::super::StructureStartModel::invalid();
    assert_eq!(
        super::super::first_structure_start_containing_pos(
            inside_union_gap,
            &[invalid.clone(), start.clone()]
        ),
        Some(start.clone())
    );
    assert_eq!(
        super::super::first_structure_start_with_piece_at(
            inside_union_gap,
            std::slice::from_ref(&start),
        ),
        None
    );
    assert_eq!(
        super::super::first_structure_start_with_piece_at(
            inside_first_edge,
            &[invalid, start.clone()]
        ),
        Some(start)
    );
}

#[test]
fn chunk_generator_mob_lookup_prefers_matching_structure_spawn_overrides() {
    let start = swamp_hut_spawn_override_start();
    let piece_override = super::super::StructureSpawnOverrideModel {
        category: "monster",
        bounding_box: super::super::StructureSpawnBoundingBoxTypeModel::Piece,
        spawns: &["minecraft:witch"],
    };
    let full_override = super::super::StructureSpawnOverrideModel {
        category: "monster",
        bounding_box: super::super::StructureSpawnBoundingBoxTypeModel::Full,
        spawns: &["minecraft:guardian"],
    };
    let inside_piece = BlockPos { x: 4, y: 20, z: 4 };
    let inside_union_gap = BlockPos { x: 7, y: 15, z: 7 };
    let biome_spawns = &["minecraft:zombie", "minecraft:skeleton"];

    assert_structure_spawn_override_bounds_match_vanilla(
        &start,
        piece_override,
        full_override,
        inside_piece,
        inside_union_gap,
    );
    assert_chunk_generator_mob_override_lookup_matches_vanilla(
        &start,
        piece_override,
        full_override,
        inside_piece,
        inside_union_gap,
        biome_spawns,
    );
}

fn swamp_hut_spawn_override_start() -> super::super::StructureStartModel {
    let first_piece = super::super::StructurePieceModel {
        bounding_box: super::super::StructureBoundingBoxModel {
            min_x: 0,
            min_y: 10,
            min_z: 0,
            max_x: 4,
            max_y: 20,
            max_z: 4,
        },
    };
    let second_piece = super::super::StructurePieceModel {
        bounding_box: super::super::StructureBoundingBoxModel {
            min_x: 10,
            min_y: 10,
            min_z: 10,
            max_x: 14,
            max_y: 20,
            max_z: 14,
        },
    };
    super::super::StructureStartModel {
        structure: Some("minecraft:swamp_hut"),
        chunk_pos: ChunkPos { x: 0, z: 0 },
        references: 0,
        pieces: vec![first_piece, second_piece],
    }
}

fn assert_structure_spawn_override_bounds_match_vanilla(
    start: &super::super::StructureStartModel,
    piece_override: super::super::StructureSpawnOverrideModel,
    full_override: super::super::StructureSpawnOverrideModel,
    inside_piece: BlockPos,
    inside_union_gap: BlockPos,
) {
    assert!(super::super::structure_spawn_override_applies(
        inside_piece,
        start,
        piece_override
    ));
    assert!(!super::super::structure_spawn_override_applies(
        inside_union_gap,
        start,
        piece_override
    ));
    assert!(super::super::structure_spawn_override_applies(
        inside_union_gap,
        start,
        full_override
    ));
}

fn assert_chunk_generator_mob_override_lookup_matches_vanilla(
    start: &super::super::StructureStartModel,
    piece_override: super::super::StructureSpawnOverrideModel,
    full_override: super::super::StructureSpawnOverrideModel,
    inside_piece: BlockPos,
    inside_union_gap: BlockPos,
    biome_spawns: &[&'static str],
) {
    assert_eq!(
        super::super::chunk_generator_mobs_at(
            biome_spawns,
            "monster",
            inside_piece,
            &[super::super::StructureSpawnCandidateModel {
                structure: "minecraft:swamp_hut",
                start,
                override_model: Some(piece_override),
            }]
        ),
        vec!["minecraft:witch"]
    );
    assert_eq!(
        super::super::chunk_generator_mobs_at(
            biome_spawns,
            "monster",
            inside_union_gap,
            &[super::super::StructureSpawnCandidateModel {
                structure: "minecraft:swamp_hut",
                start,
                override_model: Some(piece_override),
            }]
        ),
        biome_spawns.to_vec()
    );
    assert_eq!(
        super::super::chunk_generator_mobs_at(
            biome_spawns,
            "monster",
            inside_union_gap,
            &[super::super::StructureSpawnCandidateModel {
                structure: "minecraft:ocean_monument",
                start,
                override_model: Some(full_override),
            }]
        ),
        vec!["minecraft:guardian"]
    );
    assert_eq!(
        super::super::chunk_generator_mobs_at(
            biome_spawns,
            "creature",
            inside_piece,
            &[super::super::StructureSpawnCandidateModel {
                structure: "minecraft:swamp_hut",
                start,
                override_model: Some(piece_override),
            }]
        ),
        biome_spawns.to_vec()
    );
}
