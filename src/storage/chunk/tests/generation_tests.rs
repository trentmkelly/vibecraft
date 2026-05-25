use super::super::*;
use crate::storage::datafix::TARGET_DATA_VERSION;

#[test]
fn level_chunk_defaults_absent_vanilla_optional_collections() {
    let pos = ChunkPos { x: 0, z: 0 };
    let mut tag = LevelChunk::empty(pos).to_nbt(TARGET_DATA_VERSION);
    let Tag::Compound(fields) = &mut tag else {
        panic!("chunk should encode as a compound");
    };
    fields.retain(|(name, _)| {
        matches!(
            name.as_str(),
            "DataVersion" | "xPos" | "zPos" | "Status" | "LastUpdate"
        )
    });

    let decoded = LevelChunk::from_nbt(pos, &tag).unwrap();

    assert_eq!(decoded.inhabited_time, 0);
    assert!(decoded.sections.is_empty());
    assert!(decoded.heightmaps.is_empty());
    assert!(decoded.block_entities.is_empty());
    assert!(decoded.entities.is_empty());
    assert!(decoded.block_ticks.is_empty());
    assert!(decoded.fluid_ticks.is_empty());
    assert!(decoded.post_processing.is_empty());
    assert!(matches!(decoded.structures, Tag::Compound(_)));
}

#[test]
fn level_chunk_loads_only_heightmaps_valid_for_status() {
    let pos = ChunkPos { x: 0, z: 0 };
    let mut chunk = LevelChunk::empty(pos);
    chunk.status = "minecraft:noise".to_string();
    chunk
        .heightmaps
        .insert("WORLD_SURFACE_WG".to_string(), Tag::LongArray(vec![1]));
    chunk
        .heightmaps
        .insert("MOTION_BLOCKING".to_string(), Tag::LongArray(vec![2]));
    chunk
        .heightmaps
        .insert("OCEAN_FLOOR_WG".to_string(), Tag::Int(3));

    let decoded = LevelChunk::from_nbt(pos, &chunk.to_nbt(TARGET_DATA_VERSION)).unwrap();

    assert!(decoded.heightmaps.contains_key("WORLD_SURFACE_WG"));
    assert!(!decoded.heightmaps.contains_key("OCEAN_FLOOR_WG"));
    assert!(!decoded.heightmaps.contains_key("MOTION_BLOCKING"));
}

#[test]
fn level_chunk_reports_missing_status_heightmaps_to_prime() {
    let mut chunk = LevelChunk::empty(ChunkPos { x: 0, z: 0 });
    chunk.status = "minecraft:full".to_string();
    chunk
        .heightmaps
        .insert("WORLD_SURFACE".to_string(), Tag::LongArray(vec![1]));
    chunk
        .heightmaps
        .insert("OCEAN_FLOOR".to_string(), Tag::LongArray(vec![2]));

    assert_eq!(
        chunk.heightmaps_to_prime(),
        vec![
            HeightmapKind::MotionBlocking,
            HeightmapKind::MotionBlockingNoLeaves,
        ]
    );

    chunk.status = "minecraft:surface".to_string();

    assert_eq!(
        chunk.heightmaps_to_prime(),
        vec![HeightmapKind::OceanFloorWg, HeightmapKind::WorldSurfaceWg,]
    );

    chunk.status = "minecraft:not_a_status".to_string();

    assert!(chunk.heightmaps_to_prime().is_empty());
}

#[test]
fn level_chunk_primes_heightmaps_from_block_sections() {
    let mut chunk = LevelChunk::empty(ChunkPos { x: 2, z: -3 });
    chunk.min_section_y = 0;
    chunk.status = "minecraft:full".to_string();
    chunk.sections = vec![
        ChunkSection {
            y: 0,
            block_states: default_block_states_container(),
            biomes: default_biomes_container(),
            block_light: None,
            sky_light: None,
        },
        ChunkSection {
            y: 1,
            block_states: default_block_states_container(),
            biomes: default_biomes_container(),
            block_light: None,
            sky_light: None,
        },
    ];
    let world_x = chunk.pos.x * CHUNK_WIDTH + 3;
    let world_z = chunk.pos.z * CHUNK_WIDTH + 5;
    chunk.set_block_state(world_x, 20, world_z, "minecraft:water");
    chunk.set_block_state(world_x, 10, world_z, "minecraft:oak_leaves");
    chunk.set_block_state(world_x, 4, world_z, "minecraft:stone");

    let column_index = 5 * 16 + 3;

    assert_eq!(
        chunk.compute_heightmap_values(HeightmapKind::WorldSurface)[column_index],
        21
    );
    assert_eq!(
        chunk.compute_heightmap_values(HeightmapKind::MotionBlocking)[column_index],
        21
    );
    assert_eq!(
        chunk.compute_heightmap_values(HeightmapKind::MotionBlockingNoLeaves)[column_index],
        21
    );

    chunk.prime_missing_heightmaps();

    assert!(chunk.heightmaps_to_prime().is_empty());
    assert_eq!(
        chunk
            .heightmaps
            .get("WORLD_SURFACE")
            .and_then(|tag| match tag {
                Tag::LongArray(values) => Some(values.len()),
                _ => None,
            }),
        Some(36)
    );
    assert!(chunk.heightmaps.contains_key("MOTION_BLOCKING"));
    assert!(chunk.heightmaps.contains_key("MOTION_BLOCKING_NO_LEAVES"));
}

#[test]
fn level_chunk_updates_existing_heightmaps_after_block_changes() {
    let mut chunk = LevelChunk::empty(ChunkPos { x: 0, z: 0 });
    chunk.min_section_y = 0;
    chunk.sections = vec![ChunkSection {
        y: 0,
        block_states: default_block_states_container(),
        biomes: default_biomes_container(),
        block_light: None,
        sky_light: None,
    }];
    chunk.prime_heightmaps(&[HeightmapKind::WorldSurface]);
    let column_index = 4 * 16 + 2;

    chunk.set_block_state(2, 3, 4, "minecraft:stone");
    let values = match chunk.heightmaps.get("WORLD_SURFACE").unwrap() {
        Tag::LongArray(values) => super::super::unpack_heightmap_values(values),
        _ => panic!("heightmap should be a long array"),
    };
    assert_eq!(values[column_index], 4);

    chunk.set_block_state(2, 8, 4, "minecraft:stone");
    let values = match chunk.heightmaps.get("WORLD_SURFACE").unwrap() {
        Tag::LongArray(values) => super::super::unpack_heightmap_values(values),
        _ => panic!("heightmap should be a long array"),
    };
    assert_eq!(values[column_index], 9);

    chunk.set_block_state(2, 8, 4, "minecraft:air");
    let values = match chunk.heightmaps.get("WORLD_SURFACE").unwrap() {
        Tag::LongArray(values) => super::super::unpack_heightmap_values(values),
        _ => panic!("heightmap should be a long array"),
    };
    assert_eq!(values[column_index], 4);
}

#[test]
fn level_chunk_set_block_state_creates_missing_section() {
    let mut chunk = LevelChunk::empty(ChunkPos { x: 0, z: 0 });
    chunk.min_section_y = -4;

    chunk.set_block_state(2, 70, 4, "minecraft:oak_planks");

    assert_eq!(
        chunk.get_block_state(2, 70, 4).as_deref(),
        Some("minecraft:oak_planks")
    );
    assert!(
        chunk.sections.iter().any(|section| section.y == 4),
        "world Y=70 belongs to section Y=4"
    );
}

#[test]
fn level_chunk_set_block_state_preserves_block_state_properties() {
    let mut chunk = LevelChunk::empty(ChunkPos { x: 0, z: 0 });
    chunk.set_block_state(2, 70, 4, "minecraft:water[level=8]");

    let state = chunk.get_block_state_model(2, 70, 4).unwrap();
    assert_eq!(state.name, "minecraft:water");
    assert_eq!(state.properties.get("level").map(String::as_str), Some("8"));
}

#[test]
fn level_chunk_load_ignores_non_compound_entity_entries() {
    let pos = ChunkPos { x: 0, z: 0 };
    let mut chunk = LevelChunk::empty(pos);
    chunk.status = "minecraft:spawn".to_string();
    chunk.entities = vec![
        Tag::String("not-an-entity".to_string()),
        Tag::Compound(vec![(
            "id".to_string(),
            Tag::String("minecraft:pig".to_string()),
        )]),
    ];
    chunk.block_entities = vec![
        Tag::Int(7),
        Tag::Compound(vec![(
            "id".to_string(),
            Tag::String("minecraft:chest".to_string()),
        )]),
    ];

    let decoded = LevelChunk::from_nbt(pos, &chunk.to_nbt(TARGET_DATA_VERSION)).unwrap();

    assert_eq!(decoded.entities.len(), 1);
    assert_eq!(decoded.block_entities.len(), 1);
    assert!(matches!(&decoded.entities[0], Tag::Compound(_)));
    assert!(matches!(&decoded.block_entities[0], Tag::Compound(_)));
}

#[test]
fn level_chunk_load_normalizes_postprocessing_sections() {
    let pos = ChunkPos { x: 0, z: 0 };
    let mut chunk = LevelChunk::empty(pos);
    chunk.post_processing = vec![
        Tag::List(vec![Tag::Short(12), Tag::Int(99)]),
        Tag::String("not-a-section-list".to_string()),
    ];

    let decoded = LevelChunk::from_nbt(pos, &chunk.to_nbt(TARGET_DATA_VERSION)).unwrap();

    assert_eq!(
        decoded.post_processing,
        vec![
            Tag::List(vec![Tag::Short(12), Tag::Short(0)]),
            Tag::List(Vec::new()),
        ]
    );
}

#[test]
fn level_chunk_load_defaults_sparse_section_payloads() {
    let pos = ChunkPos { x: 0, z: 0 };
    let mut tag = LevelChunk::empty(pos).to_nbt(TARGET_DATA_VERSION);
    let Tag::Compound(fields) = &mut tag else {
        panic!("chunk should encode as a compound");
    };
    let (_, sections) = fields
        .iter_mut()
        .find(|(name, _)| name == "sections")
        .expect("sections should be present");
    *sections = Tag::List(vec![
        Tag::String("not-a-section".to_string()),
        Tag::Compound(Vec::new()),
    ]);

    let decoded = LevelChunk::from_nbt(pos, &tag).unwrap();

    assert_eq!(decoded.sections.len(), 1);
    assert_eq!(decoded.sections[0].y, 0);
    assert_eq!(
        decoded.sections[0].block_states,
        default_block_states_container()
    );
    assert_eq!(decoded.sections[0].biomes, default_biomes_container());
}

#[test]
fn level_chunk_defaults_wrong_type_optional_compounds_on_load() {
    let pos = ChunkPos { x: 0, z: 0 };
    let mut tag = LevelChunk::empty(pos).to_nbt(TARGET_DATA_VERSION);
    let Tag::Compound(fields) = &mut tag else {
        panic!("chunk should encode as a compound");
    };
    let (_, structures) = fields
        .iter_mut()
        .find(|(name, _)| name == "structures")
        .expect("structures should be present");
    *structures = Tag::List(Vec::new());
    let (_, sections) = fields
        .iter_mut()
        .find(|(name, _)| name == "sections")
        .expect("sections should be present");
    *sections = Tag::List(vec![Tag::Compound(vec![
        ("Y".to_string(), Tag::Byte(0)),
        (
            "block_states".to_string(),
            Tag::String("wrong-type".to_string()),
        ),
        ("biomes".to_string(), Tag::Int(7)),
    ])]);

    let decoded = LevelChunk::from_nbt(pos, &tag).unwrap();

    assert_eq!(decoded.structures, empty_structures_payload());
    assert_eq!(
        decoded.sections[0].block_states,
        default_block_states_container()
    );
    assert_eq!(decoded.sections[0].biomes, default_biomes_container());
}

#[test]
fn level_chunk_load_normalizes_structure_payloads() {
    let pos = ChunkPos { x: 10, z: -10 };
    let mut tag = LevelChunk::empty(pos).to_nbt(TARGET_DATA_VERSION);
    let Tag::Compound(fields) = &mut tag else {
        panic!("chunk should encode as a compound");
    };
    let (_, structures) = fields
        .iter_mut()
        .find(|(name, _)| name == "structures")
        .expect("structures should be present");
    *structures = Tag::Compound(vec![
        (
            "starts".to_string(),
            Tag::Compound(vec![
                (
                    "minecraft:village".to_string(),
                    Tag::Compound(vec![("id".to_string(), Tag::String("village".to_string()))]),
                ),
                ("minecraft:bad".to_string(), Tag::List(Vec::new())),
            ]),
        ),
        (
            "References".to_string(),
            Tag::Compound(vec![
                (
                    "minecraft:village".to_string(),
                    Tag::LongArray(vec![
                        super::super::pack_chunk_pos_as_long(ChunkPos { x: 18, z: -2 }),
                        super::super::pack_chunk_pos_as_long(ChunkPos { x: 19, z: -10 }),
                    ]),
                ),
                ("minecraft:bad".to_string(), Tag::String("bad".to_string())),
            ]),
        ),
    ]);

    let decoded = LevelChunk::from_nbt(pos, &tag).unwrap();

    assert_eq!(
        decoded.structures,
        Tag::Compound(vec![
            (
                "starts".to_string(),
                Tag::Compound(vec![(
                    "minecraft:village".to_string(),
                    Tag::Compound(vec![("id".to_string(), Tag::String("village".to_string()))])
                )])
            ),
            (
                "References".to_string(),
                Tag::Compound(vec![(
                    "minecraft:village".to_string(),
                    Tag::LongArray(vec![super::super::pack_chunk_pos_as_long(ChunkPos {
                        x: 18,
                        z: -2
                    })])
                )])
            ),
        ])
    );
    assert_eq!(
        super::super::unpack_chunk_pos_from_long(super::super::pack_chunk_pos_as_long(ChunkPos {
            x: -1,
            z: -2
        })),
        ChunkPos { x: -1, z: -2 }
    );
}

#[test]
fn level_chunk_rejects_invalid_section_light_arrays_on_load() {
    let pos = ChunkPos { x: 0, z: 0 };
    let mut tag = LevelChunk::empty(pos).to_nbt(TARGET_DATA_VERSION);
    let Tag::Compound(fields) = &mut tag else {
        panic!("chunk should encode as a compound");
    };
    let (_, sections) = fields
        .iter_mut()
        .find(|(name, _)| name == "sections")
        .expect("sections should be present");
    *sections = Tag::List(vec![Tag::Compound(vec![
        ("Y".to_string(), Tag::Byte(0)),
        ("BlockLight".to_string(), Tag::ByteArray(vec![0; 17])),
    ])]);

    let err = LevelChunk::from_nbt(pos, &tag).unwrap_err();

    assert!(err.contains("DataLayer should be 2048 bytes not: 17"));
    assert!(err.contains("BlockLight"));
}

#[test]
fn section_positions_and_palettes_match_vanilla_shapes() {
    assert_eq!(SECTION_VOLUME, 4096);
    assert_eq!(BIOME_SECTION_VOLUME, 64);
    assert!(SectionBlockPos::new(16, 0, 0).is_none());

    let pos = SectionBlockPos::new(3, 5, 7).unwrap();
    assert_eq!(pos.block_state_index(), 5 * 256 + 7 * 16 + 3);
    assert_eq!(pos.biome_index(), 1 * 16 + 1 * 4);

    let mut oak = BlockStateEntry::new("minecraft:oak_log");
    oak.properties.insert("axis".to_string(), "y".to_string());
    let block_states = PalettedContainer::single(oak.to_nbt(), SECTION_VOLUME);
    let decoded = PalettedContainer::from_nbt(&block_states.to_nbt(), SECTION_VOLUME).unwrap();
    assert_eq!(decoded.palette.len(), 1);
    assert!(decoded.data.is_none());
    assert_eq!(decoded.expected_entries, SECTION_VOLUME);
}

#[test]
fn chunk_status_pipeline_matches_vanilla_order_and_dependencies() {
    assert_eq!(CHUNK_STATUS_PIPELINE.len(), 12);
    assert_eq!(CHUNK_STATUS_PIPELINE[0].id, "minecraft:empty");
    assert_eq!(CHUNK_STATUS_PIPELINE[0].parent, "minecraft:empty");
    assert_eq!(CHUNK_STATUS_PIPELINE[11].id, "minecraft:full");
    assert_eq!(CHUNK_STATUS_PIPELINE[11].parent, "minecraft:spawn");
    assert_eq!(CHUNK_STATUS_PIPELINE[11].chunk_type, ChunkType::LevelChunk);
    assert_eq!(chunk_status("full").unwrap().index, 11);
    assert_eq!(
        chunk_status("minecraft:carvers").unwrap().parent,
        "minecraft:surface"
    );
    assert_eq!(chunk_status_is_or_after("features", "carvers"), Some(true));
    assert_eq!(chunk_status_is_or_after("noise", "features"), Some(false));
    assert_eq!(chunk_status_is_after("features", "carvers"), Some(true));
    assert_eq!(chunk_status_is_after("carvers", "carvers"), Some(false));
    assert_eq!(chunk_status_is_before("noise", "features"), Some(true));
    assert_eq!(chunk_status_is_before("features", "features"), Some(false));
    assert_eq!(
        chunk_status_is_or_before("features", "features"),
        Some(true)
    );
    assert_eq!(chunk_status_is_or_before("full", "spawn"), Some(false));
    assert_eq!(
        chunk_status_max("noise", "features"),
        Some("minecraft:features")
    );
    assert_eq!(chunk_status_max("full", "spawn"), Some("minecraft:full"));
    assert_eq!(chunk_status_max("bad", "spawn"), None);
    assert_eq!(
        chunk_status_list(),
        vec![
            "minecraft:empty",
            "minecraft:structure_starts",
            "minecraft:structure_references",
            "minecraft:biomes",
            "minecraft:noise",
            "minecraft:surface",
            "minecraft:carvers",
            "minecraft:features",
            "minecraft:initialize_light",
            "minecraft:light",
            "minecraft:spawn",
            "minecraft:full",
        ]
    );

    assert_eq!(
        WORLDGEN_HEIGHTMAPS
            .iter()
            .map(|kind| kind.storage_name())
            .collect::<Vec<_>>(),
        vec!["OCEAN_FLOOR_WG", "WORLD_SURFACE_WG"]
    );
    assert_eq!(
        FINAL_HEIGHTMAPS
            .iter()
            .map(|kind| kind.storage_name())
            .collect::<Vec<_>>(),
        vec![
            "OCEAN_FLOOR",
            "WORLD_SURFACE",
            "MOTION_BLOCKING",
            "MOTION_BLOCKING_NO_LEAVES"
        ]
    );
    assert_eq!(
        chunk_status("features").unwrap().heightmaps_after,
        FINAL_HEIGHTMAPS
    );
    assert_eq!(
        chunk_status("structure_starts").unwrap().task,
        ChunkStatusTaskKind::GenerateStructureStarts
    );
    assert_eq!(
        chunk_status("structure_references").unwrap().task,
        ChunkStatusTaskKind::GenerateStructureReferences
    );
    assert_eq!(
        chunk_status("structure_references")
            .unwrap()
            .region_dependencies,
        8
    );
    assert_eq!(
        chunk_status("structure_references").unwrap().requirements,
        super::super::STRUCTURE_STARTS_DISTANCE_8_REQUIREMENT
    );
    assert_eq!(
        chunk_status("biomes").unwrap().task,
        ChunkStatusTaskKind::GenerateBiomes
    );
    assert_eq!(chunk_status("biomes").unwrap().region_dependencies, 8);
    assert_eq!(
        chunk_status("biomes").unwrap().requirements,
        super::super::STRUCTURE_STARTS_DISTANCE_8_REQUIREMENT
    );
    assert_eq!(
        chunk_status("noise").unwrap().requirements,
        super::super::STRUCTURE_STARTS_DISTANCE_8_AND_BIOMES_DISTANCE_1_REQUIREMENTS
    );
    assert_eq!(chunk_status("noise").unwrap().block_state_write_radius, 0);
    assert_eq!(
        chunk_status("features").unwrap().task,
        ChunkStatusTaskKind::GenerateFeatures
    );
    assert_eq!(
        chunk_status("features").unwrap().requirements,
        super::super::STRUCTURE_STARTS_DISTANCE_8_AND_CARVERS_DISTANCE_1_REQUIREMENTS
    );
    assert_eq!(
        chunk_status("features").unwrap().block_state_write_radius,
        1
    );
    assert_eq!(
        chunk_status("light").unwrap().requirements,
        super::super::INITIALIZE_LIGHT_DISTANCE_1_REQUIREMENT
    );
    assert_eq!(
        chunk_status("spawn").unwrap().requirements,
        super::super::BIOMES_DISTANCE_1_REQUIREMENT
    );
    assert_eq!(
        chunk_status("full").unwrap().task,
        ChunkStatusTaskKind::Full
    );
    assert_eq!(chunk_status("full").unwrap().region_dependencies, 0);
    assert_eq!(
        chunk_status("biomes").unwrap().heightmaps_after,
        WORLDGEN_HEIGHTMAPS
    );
    assert_eq!(
        HeightmapKind::MotionBlockingNoLeaves.storage_name(),
        "MOTION_BLOCKING_NO_LEAVES"
    );
}

#[test]
fn chunk_pyramid_dependencies_match_generation_and_loading_radii() {
    assert_eq!(
        super::super::chunk_pyramid_direct_dependencies(ChunkPyramidKind::Generation, "features"),
        Some(vec![
            "minecraft:carvers",
            "minecraft:carvers",
            "minecraft:structure_starts",
            "minecraft:structure_starts",
            "minecraft:structure_starts",
            "minecraft:structure_starts",
            "minecraft:structure_starts",
            "minecraft:structure_starts",
            "minecraft:structure_starts",
        ])
    );
    assert_eq!(
        super::super::chunk_pyramid_accumulated_dependencies(ChunkPyramidKind::Generation, "features"),
        Some(vec![
            "minecraft:carvers",
            "minecraft:carvers",
            "minecraft:biomes",
            "minecraft:structure_starts",
            "minecraft:structure_starts",
            "minecraft:structure_starts",
            "minecraft:structure_starts",
            "minecraft:structure_starts",
            "minecraft:structure_starts",
            "minecraft:structure_starts",
            "minecraft:structure_starts",
        ])
    );
    assert_eq!(
        super::super::chunk_pyramid_accumulated_dependencies(ChunkPyramidKind::Loading, "features"),
        Some(vec!["minecraft:carvers"])
    );

    assert_eq!(
        super::super::chunk_generation_task_worst_case_radius("full"),
        Some(11)
    );
    assert_eq!(
        super::super::chunk_generation_task_layer_radius("full", "structure_starts", true),
        Some(11)
    );
    assert_eq!(
        super::super::chunk_generation_task_layer_radius("full", "features", true),
        Some(1)
    );
    assert_eq!(
        super::super::chunk_generation_task_layer_radius("full", "features", false),
        Some(1)
    );
    assert_eq!(
        super::super::chunk_generation_task_layer_radius("light", "initialize_light", false),
        Some(1)
    );
    assert_eq!(
        super::super::chunk_generation_task_layer_radius("light", "empty", false),
        Some(1)
    );
    assert_eq!(
        super::super::chunk_generation_task_worst_case_radius("unknown"),
        None
    );
}

#[test]
fn worldgen_region_access_plan_matches_java_features_step_contract() {
    let center = ChunkPos { x: 4, z: -7 };
    let plan =
        super::super::worldgen_region_access_plan(ChunkPyramidKind::Generation, "features", center)
            .expect("features access plan should exist");

    assert_eq!(plan.target_status, "minecraft:features");
    assert_eq!(plan.block_state_write_radius, 1);
    assert_eq!(plan.chunks.len(), 17 * 17);

    let center_entry = plan
        .chunks
        .iter()
        .find(|entry| entry.chunk == center)
        .expect("center chunk should be in access plan");
    assert_eq!(center_entry.max_read_status, "minecraft:carvers");
    assert!(center_entry.can_write_blocks);

    let carver_neighbor = plan
        .chunks
        .iter()
        .find(|entry| entry.chunk == (ChunkPos { x: 5, z: -7 }))
        .expect("distance-1 chunk should be in access plan");
    assert_eq!(carver_neighbor.distance, 1);
    assert_eq!(carver_neighbor.max_read_status, "minecraft:carvers");
    assert!(carver_neighbor.can_write_blocks);

    let far_structure_chunk = plan
        .chunks
        .iter()
        .find(|entry| entry.chunk == (ChunkPos { x: 12, z: -7 }))
        .expect("distance-8 chunk should be in access plan");
    assert_eq!(far_structure_chunk.distance, 8);
    assert_eq!(
        far_structure_chunk.max_read_status,
        "minecraft:structure_starts"
    );
    assert!(!far_structure_chunk.can_write_blocks);

    assert_eq!(
        super::super::worldgen_region_can_read_status(
            ChunkPyramidKind::Generation,
            "features",
            center,
            ChunkPos { x: 5, z: -7 },
            "minecraft:carvers",
        ),
        Some(true)
    );
    assert_eq!(
        super::super::worldgen_region_can_read_status(
            ChunkPyramidKind::Generation,
            "features",
            center,
            ChunkPos { x: 12, z: -7 },
            "minecraft:carvers",
        ),
        Some(false)
    );
    assert_eq!(
        super::super::worldgen_region_can_write_block(
            ChunkPyramidKind::Generation,
            "features",
            center,
            ChunkPos { x: 6, z: -7 },
        ),
        Some(false)
    );
}

#[test]
fn chunk_generation_task_can_load_without_generation_matches_java_gate() {
    assert_eq!(
        super::super::chunk_generation_task_can_load_without_generation("empty", 0, 0, |_, _| None),
        Some(true)
    );
    assert_eq!(
        super::super::chunk_generation_task_can_load_without_generation("features", 0, 0, |x, z| {
            if x == 0 && z == 0 {
                Some("minecraft:carvers")
            } else {
                Some("minecraft:features")
            }
        }),
        Some(false)
    );
    assert_eq!(
        super::super::chunk_generation_task_can_load_without_generation("light", 0, 0, |x, z| {
            if x == 0 && z == 0 {
                Some("minecraft:light")
            } else {
                Some("minecraft:initialize_light")
            }
        }),
        Some(true)
    );
    assert_eq!(
        super::super::chunk_generation_task_can_load_without_generation("light", 0, 0, |x, z| {
            if x == 0 && z == 0 {
                Some("minecraft:light")
            } else if x == 1 && z == 0 {
                Some("minecraft:features")
            } else {
                Some("minecraft:initialize_light")
            }
        }),
        Some(false)
    );
    assert_eq!(
        super::super::chunk_generation_task_can_load_without_generation("light", 0, 0, |x, z| {
            if x == 0 && z == 0 {
                Some("minecraft:light")
            } else if x == 1 && z == 0 {
                None
            } else {
                Some("minecraft:initialize_light")
            }
        }),
        None
    );
}

#[test]
fn chunk_generation_task_next_layer_matches_java_schedule_next_layer() {
    assert_eq!(
        super::super::chunk_generation_task_next_layer(None, false, false),
        Some(super::super::ChunkGenerationLayerPlan {
            status: "minecraft:empty",
            needs_generation: false,
        })
    );
    assert_eq!(
        super::super::chunk_generation_task_next_layer(Some("minecraft:empty"), false, false),
        Some(super::super::ChunkGenerationLayerPlan {
            status: "minecraft:empty",
            needs_generation: true,
        })
    );
    assert_eq!(
        super::super::chunk_generation_task_next_layer(Some("minecraft:empty"), false, true),
        Some(super::super::ChunkGenerationLayerPlan {
            status: "minecraft:structure_starts",
            needs_generation: false,
        })
    );
    assert_eq!(
        super::super::chunk_generation_task_next_layer(Some("minecraft:empty"), true, false),
        Some(super::super::ChunkGenerationLayerPlan {
            status: "minecraft:structure_starts",
            needs_generation: true,
        })
    );
    assert_eq!(
        super::super::chunk_generation_task_next_layer(Some("minecraft:carvers"), true, true),
        Some(super::super::ChunkGenerationLayerPlan {
            status: "minecraft:features",
            needs_generation: true,
        })
    );
    assert_eq!(
        super::super::chunk_generation_task_next_layer(Some("minecraft:full"), false, true),
        None
    );
    assert_eq!(
        super::super::chunk_generation_task_next_layer(Some("missing"), false, true),
        None
    );
}

#[test]
fn chunk_generation_task_chunk_step_matches_java_schedule_chunk_in_layer_gate() {
    assert_eq!(
        super::super::chunk_generation_task_chunk_step("minecraft:features", Some("carvers"), true),
        Some(super::super::ChunkGenerationChunkStepPlan::Apply {
            pyramid: ChunkPyramidKind::Generation,
            generate: true,
        })
    );
    assert_eq!(
        super::super::chunk_generation_task_chunk_step("minecraft:features", Some("carvers"), false),
        Some(super::super::ChunkGenerationChunkStepPlan::UnexpectedGeneration)
    );
    assert_eq!(
        super::super::chunk_generation_task_chunk_step("minecraft:features", Some("features"), false),
        Some(super::super::ChunkGenerationChunkStepPlan::Apply {
            pyramid: ChunkPyramidKind::Loading,
            generate: false,
        })
    );
    assert_eq!(
        super::super::chunk_generation_task_chunk_step("minecraft:features", Some("full"), true),
        Some(super::super::ChunkGenerationChunkStepPlan::Apply {
            pyramid: ChunkPyramidKind::Loading,
            generate: false,
        })
    );
    assert_eq!(
        super::super::chunk_generation_task_chunk_step("minecraft:features", None, true),
        Some(super::super::ChunkGenerationChunkStepPlan::Apply {
            pyramid: ChunkPyramidKind::Loading,
            generate: false,
        })
    );
    assert_eq!(
        super::super::chunk_generation_task_chunk_step("missing", Some("features"), true),
        None
    );
    assert_eq!(
        super::super::chunk_generation_task_chunk_step("minecraft:features", Some("missing"), true),
        Some(super::super::ChunkGenerationChunkStepPlan::Apply {
            pyramid: ChunkPyramidKind::Loading,
            generate: false,
        })
    );
}

#[test]
fn chunk_generation_task_wait_for_scheduled_layer_matches_java_stack_polling() {
    use super::super::ChunkGenerationFutureState::{Failure, Pending, Success};

    assert_eq!(
        super::super::chunk_generation_task_wait_for_scheduled_layer(&[]),
        super::super::ChunkGenerationWaitPlan {
            waiting_for_index: None,
            remaining_layer: vec![],
            marked_for_cancellation: false,
        }
    );
    assert_eq!(
        super::super::chunk_generation_task_wait_for_scheduled_layer(&[Success, Pending, Success]),
        super::super::ChunkGenerationWaitPlan {
            waiting_for_index: Some(1),
            remaining_layer: vec![Success, Pending],
            marked_for_cancellation: false,
        }
    );
    assert_eq!(
        super::super::chunk_generation_task_wait_for_scheduled_layer(&[Pending, Failure, Success]),
        super::super::ChunkGenerationWaitPlan {
            waiting_for_index: Some(0),
            remaining_layer: vec![Pending],
            marked_for_cancellation: true,
        }
    );
    assert_eq!(
        super::super::chunk_generation_task_wait_for_scheduled_layer(&[Success, Failure]),
        super::super::ChunkGenerationWaitPlan {
            waiting_for_index: None,
            remaining_layer: vec![],
            marked_for_cancellation: true,
        }
    );
}

#[test]
fn chunk_generation_task_schedule_layer_positions_match_java_traversal() {
    assert_eq!(
        super::super::chunk_generation_task_schedule_layer_positions(
            "minecraft:light",
            "minecraft:initialize_light",
            false,
            5,
            -2,
            std::iter::repeat(true),
        ),
        Some(super::super::ChunkGenerationScheduleLayerPlan {
            radius: 1,
            visited_positions: vec![
                (4, -3),
                (4, -2),
                (4, -1),
                (5, -3),
                (5, -2),
                (5, -1),
                (6, -3),
                (6, -2),
                (6, -1),
            ],
            stopped_early: false,
        })
    );
    assert_eq!(
        super::super::chunk_generation_task_schedule_layer_positions(
            "minecraft:light",
            "minecraft:initialize_light",
            false,
            0,
            0,
            [true, true, false, true].into_iter(),
        ),
        Some(super::super::ChunkGenerationScheduleLayerPlan {
            radius: 1,
            visited_positions: vec![(-1, -1), (-1, 0), (-1, 1)],
            stopped_early: true,
        })
    );
    assert_eq!(
        super::super::chunk_generation_task_schedule_layer_positions(
            "missing",
            "minecraft:empty",
            false,
            0,
            0,
            std::iter::repeat(true),
        ),
        None
    );
}

#[test]
fn chunk_generation_task_run_until_wait_decision_matches_java_ordering() {
    use super::super::ChunkGenerationFutureState::{Failure, Pending, Success};

    assert_eq!(
        super::super::chunk_generation_task_run_until_wait_decision(
            "minecraft:features",
            Some("minecraft:features"),
            false,
            false,
            true,
            &[Pending],
        ),
        Some(super::super::ChunkGenerationRunPlan::Waiting {
            waiting_for_index: 0,
            remaining_layer: vec![Pending],
            marked_for_cancellation: false,
        })
    );
    assert_eq!(
        super::super::chunk_generation_task_run_until_wait_decision(
            "minecraft:features",
            Some("minecraft:features"),
            false,
            false,
            true,
            &[Success],
        ),
        Some(super::super::ChunkGenerationRunPlan::Released)
    );
    assert_eq!(
        super::super::chunk_generation_task_run_until_wait_decision(
            "minecraft:features",
            Some("minecraft:carvers"),
            false,
            false,
            true,
            &[Failure],
        ),
        Some(super::super::ChunkGenerationRunPlan::Released)
    );
    assert_eq!(
        super::super::chunk_generation_task_run_until_wait_decision(
            "minecraft:features",
            Some("minecraft:empty"),
            false,
            false,
            false,
            &[],
        ),
        Some(super::super::ChunkGenerationRunPlan::Schedule(
            super::super::ChunkGenerationLayerPlan {
                status: "minecraft:empty",
                needs_generation: true,
            }
        ))
    );
    assert_eq!(
        super::super::chunk_generation_task_run_until_wait_decision(
            "minecraft:features",
            None,
            false,
            false,
            true,
            &[],
        ),
        Some(super::super::ChunkGenerationRunPlan::Schedule(
            super::super::ChunkGenerationLayerPlan {
                status: "minecraft:empty",
                needs_generation: false,
            }
        ))
    );
    assert_eq!(
        super::super::chunk_generation_task_run_until_wait_decision(
            "missing",
            None,
            false,
            false,
            true,
            &[],
        ),
        None
    );
}
