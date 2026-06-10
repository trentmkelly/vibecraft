use super::{
    default_biomes_container, default_block_states_container, pack_postprocessing_offset,
    saved_tick_tag, string_field, unpack_postprocessing_offset, BlockStateEntry,
    ChunkInitializeLightPlan, ChunkLightCompletionPlan, ChunkSection, LevelChunk, LightLayer,
    LightSectionStatusUpdate, QueuedSectionLightData, TickPriority, LIGHT_DATA_LAYER_LENGTH,
    LIGHT_DATA_LAYER_NIBBLE_COUNT, LIGHT_DATA_LAYER_ROW_SIZE, LIGHT_DATA_LAYER_WIDTH,
};
use crate::storage::datafix::TARGET_DATA_VERSION;
use crate::storage::nbt::Tag;
use crate::storage::region::ChunkPos;
use std::collections::BTreeMap;

#[test]
fn level_chunk_round_trips_vanilla_storage_sections_and_side_payloads() {
    let pos = ChunkPos { x: 4, z: -2 };
    let mut heightmaps = BTreeMap::new();
    heightmaps.insert("WORLD_SURFACE".to_string(), Tag::LongArray(vec![1, 2, 3]));
    let chunk = LevelChunk {
        pos,
        min_section_y: -4,
        last_update: 1234,
        status: "minecraft:spawn".to_string(),
        inhabited_time: 42,
        sections: vec![ChunkSection {
            y: 0,
            block_states: Tag::Compound(vec![("palette".to_string(), Tag::List(Vec::new()))]),
            biomes: Tag::Compound(vec![("palette".to_string(), Tag::List(Vec::new()))]),
            block_light: Some(vec![0; 2048]),
            sky_light: Some(vec![15; 2048]),
        }],
        heightmaps,
        block_entities: vec![Tag::Compound(vec![(
            "id".to_string(),
            Tag::String("minecraft:chest".to_string()),
        )])],
        entities: vec![Tag::Compound(vec![(
            "id".to_string(),
            Tag::String("minecraft:pig".to_string()),
        )])],
        structures: Tag::Compound(vec![("starts".to_string(), Tag::Compound(Vec::new()))]),
        upgrade_data: Some(Tag::Compound(vec![("Sides".to_string(), Tag::Int(0))])),
        blending_data: Some(Tag::Compound(vec![
            ("min_section".to_string(), Tag::Int(-4)),
            ("max_section".to_string(), Tag::Int(20)),
        ])),
        below_zero_retrogen: Some(Tag::Compound(vec![(
            "target_status".to_string(),
            Tag::String("minecraft:noise".to_string()),
        )])),
        carving_mask: Some(vec![7, 8, 9]),
        block_ticks: vec![saved_tick_tag(
            "minecraft:stone".to_string(),
            64,
            70,
            -31,
            4,
            TickPriority::Normal,
        )],
        fluid_ticks: vec![saved_tick_tag(
            "minecraft:water".to_string(),
            65,
            63,
            -32,
            2,
            TickPriority::High,
        )],
        post_processing: vec![Tag::List(vec![Tag::Short(1), Tag::Short(2)])],
        light_correct: true,
    };

    let decoded = LevelChunk::from_nbt(pos, &chunk.to_nbt(TARGET_DATA_VERSION)).unwrap();

    assert_eq!(decoded.pos, pos);
    assert_eq!(decoded.min_section_y, -4);
    assert_eq!(decoded.last_update, 1234);
    assert_eq!(decoded.status, "minecraft:spawn");
    assert_eq!(
        decoded.sections[0].block_light.as_ref().unwrap().len(),
        2048
    );
    assert_eq!(decoded.sections[0].sky_light.as_ref().unwrap()[0], 15);
    assert!(decoded.heightmaps.contains_key("WORLD_SURFACE"));
    assert_eq!(decoded.block_entities.len(), 1);
    assert_eq!(decoded.entities.len(), 1);
    assert!(decoded.upgrade_data.is_some());
    assert!(decoded.blending_data.is_some());
    assert!(decoded.below_zero_retrogen.is_some());
    assert_eq!(decoded.carving_mask, Some(vec![7, 8, 9]));
    assert_eq!(decoded.block_ticks.len(), 1);
    assert_eq!(decoded.fluid_ticks.len(), 1);
    assert_eq!(decoded.post_processing.len(), 1);
    assert!(decoded.light_correct);
}

#[test]
fn full_chunk_serialization_omits_proto_only_entity_and_carver_fields() {
    let mut chunk = LevelChunk::empty(ChunkPos { x: 0, z: 0 });
    chunk.status = "minecraft:full".to_string();
    chunk.entities = vec![Tag::Compound(vec![(
        "id".to_string(),
        Tag::String("minecraft:pig".to_string()),
    )])];
    chunk.carving_mask = Some(vec![1, 2, 3]);

    let encoded = chunk.to_nbt(TARGET_DATA_VERSION);
    let Tag::Compound(fields) = &encoded else {
        panic!("chunk should encode as a compound");
    };
    assert!(fields.iter().all(|(name, _)| name != "entities"));
    assert!(fields.iter().all(|(name, _)| name != "carving_mask"));

    let decoded = LevelChunk::from_nbt(chunk.pos, &encoded).unwrap();
    assert!(decoded.entities.is_empty());
    assert!(decoded.carving_mask.is_none());
}

#[test]
fn level_chunk_promote_to_full_migrates_proto_only_payloads() {
    let mut chunk = LevelChunk::empty(ChunkPos { x: 0, z: 0 });
    let pig = Tag::Compound(vec![(
        "id".to_string(),
        Tag::String("minecraft:pig".to_string()),
    )]);
    chunk.status = "minecraft:spawn".to_string();
    chunk.entities = vec![pig.clone()];
    chunk.carving_mask = Some(vec![1, 2, 3]);
    chunk.below_zero_retrogen = Some(Tag::Compound(vec![
        (
            "target_status".to_string(),
            Tag::String("minecraft:carvers".to_string()),
        ),
        ("missing_bedrock".to_string(), Tag::LongArray(Vec::new())),
    ]));

    let migrated_entities = chunk.promote_to_full_chunk();
    let encoded = chunk.to_nbt(TARGET_DATA_VERSION);
    let Tag::Compound(fields) = &encoded else {
        panic!("chunk should encode as a compound");
    };

    assert_eq!(migrated_entities, vec![pig]);
    assert_eq!(chunk.status, "minecraft:full");
    assert!(chunk.entities.is_empty());
    assert!(chunk.carving_mask.is_none());
    assert!(chunk.below_zero_retrogen.is_none());
    assert!(fields.iter().all(|(name, _)| name != "entities"));
    assert!(fields.iter().all(|(name, _)| name != "carving_mask"));
}

#[test]
fn level_chunk_updates_inhabited_time_like_chunk_access() {
    let mut chunk = LevelChunk::empty(ChunkPos { x: 0, z: 0 });

    chunk.set_inhabited_time(40);
    chunk.increment_inhabited_time(2);

    assert_eq!(chunk.inhabited_time, 42);

    let decoded = LevelChunk::from_nbt(chunk.pos, &chunk.to_nbt(TARGET_DATA_VERSION)).unwrap();

    assert_eq!(decoded.inhabited_time, 42);
}

#[test]
fn level_chunk_validates_blending_data_payload_shape() {
    let pos = ChunkPos { x: 0, z: 0 };
    let mut encoded = LevelChunk::empty(pos).to_nbt(TARGET_DATA_VERSION);
    let Tag::Compound(fields) = &mut encoded else {
        panic!("chunk should encode as a compound");
    };
    fields.push((
        "blending_data".to_string(),
        Tag::Compound(vec![
            ("min_section".to_string(), Tag::Int(-4)),
            ("max_section".to_string(), Tag::Int(20)),
            (
                "heights".to_string(),
                Tag::List((0..16).map(|value| Tag::Double(f64::from(value))).collect()),
            ),
        ]),
    ));

    let decoded = LevelChunk::from_nbt(pos, &encoded).unwrap();

    assert!(decoded.blending_data.is_some());
}

#[test]
fn level_chunk_rejects_invalid_blending_data_payload_shape() {
    let pos = ChunkPos { x: 0, z: 0 };

    let mut missing_sections = LevelChunk::empty(pos).to_nbt(TARGET_DATA_VERSION);
    let Tag::Compound(fields) = &mut missing_sections else {
        panic!("chunk should encode as a compound");
    };
    fields.push((
        "blending_data".to_string(),
        Tag::Compound(vec![("heights".to_string(), Tag::List(Vec::new()))]),
    ));
    let err = LevelChunk::from_nbt(pos, &missing_sections).unwrap_err();
    assert!(err.contains("missing NBT field min_section"));

    let mut wrong_heights_len = LevelChunk::empty(pos).to_nbt(TARGET_DATA_VERSION);
    let Tag::Compound(fields) = &mut wrong_heights_len else {
        panic!("chunk should encode as a compound");
    };
    fields.push((
        "blending_data".to_string(),
        Tag::Compound(vec![
            ("min_section".to_string(), Tag::Int(-4)),
            ("max_section".to_string(), Tag::Int(20)),
            (
                "heights".to_string(),
                Tag::List(vec![Tag::Double(0.0), Tag::Double(1.0)]),
            ),
        ]),
    ));
    let err = LevelChunk::from_nbt(pos, &wrong_heights_len).unwrap_err();
    assert!(err.contains("heights has to be of length 16"));

    let mut wrong_heights_type = LevelChunk::empty(pos).to_nbt(TARGET_DATA_VERSION);
    let Tag::Compound(fields) = &mut wrong_heights_type else {
        panic!("chunk should encode as a compound");
    };
    fields.push((
        "blending_data".to_string(),
        Tag::Compound(vec![
            ("min_section".to_string(), Tag::Int(-4)),
            ("max_section".to_string(), Tag::Int(20)),
            ("heights".to_string(), Tag::List(vec![Tag::Int(0)])),
        ]),
    ));
    let err = LevelChunk::from_nbt(pos, &wrong_heights_type).unwrap_err();
    assert!(err.contains("NBT field heights must be a double list"));
}

#[test]
fn level_chunk_accepts_valid_below_zero_retrogen_payload() {
    let pos = ChunkPos { x: 0, z: 0 };
    let mut encoded = LevelChunk::empty(pos).to_nbt(TARGET_DATA_VERSION);
    let Tag::Compound(fields) = &mut encoded else {
        panic!("chunk should encode as a compound");
    };
    fields.push((
        "below_zero_retrogen".to_string(),
        Tag::Compound(vec![
            (
                "target_status".to_string(),
                Tag::String("minecraft:noise".to_string()),
            ),
            ("missing_bedrock".to_string(), Tag::LongArray(vec![3])),
        ]),
    ));

    let decoded = LevelChunk::from_nbt(pos, &encoded).unwrap();

    assert!(decoded.below_zero_retrogen.is_some());
}

#[test]
fn level_chunk_highest_generated_status_accounts_for_below_zero_retrogen() {
    let mut chunk = LevelChunk::empty(ChunkPos { x: 0, z: 0 });
    chunk.status = "minecraft:noise".to_string();
    chunk.below_zero_retrogen = Some(Tag::Compound(vec![
        (
            "target_status".to_string(),
            Tag::String("minecraft:carvers".to_string()),
        ),
        ("missing_bedrock".to_string(), Tag::LongArray(Vec::new())),
    ]));

    assert_eq!(chunk.highest_generated_status(), Some("minecraft:carvers"));

    chunk.status = "minecraft:features".to_string();

    assert_eq!(chunk.highest_generated_status(), Some("minecraft:features"));
}

#[test]
fn level_chunk_set_persisted_status_clears_completed_below_zero_retrogen() {
    let mut chunk = LevelChunk::empty(ChunkPos { x: 0, z: 0 });
    chunk.below_zero_retrogen = Some(Tag::Compound(vec![
        (
            "target_status".to_string(),
            Tag::String("minecraft:carvers".to_string()),
        ),
        ("missing_bedrock".to_string(), Tag::LongArray(Vec::new())),
    ]));

    assert!(chunk.set_persisted_status("noise"));
    assert_eq!(chunk.status, "minecraft:noise");
    assert!(chunk.below_zero_retrogen.is_some());
    assert!(chunk.set_persisted_status("minecraft:carvers"));
    assert_eq!(chunk.status, "minecraft:carvers");
    assert!(chunk.below_zero_retrogen.is_none());
    assert!(!chunk.set_persisted_status("minecraft:not_a_status"));
    assert_eq!(chunk.status, "minecraft:carvers");
}

#[test]
fn level_chunk_rejects_invalid_below_zero_retrogen_payloads() {
    let pos = ChunkPos { x: 0, z: 0 };

    let mut empty_status = LevelChunk::empty(pos).to_nbt(TARGET_DATA_VERSION);
    let Tag::Compound(fields) = &mut empty_status else {
        panic!("chunk should encode as a compound");
    };
    fields.push((
        "below_zero_retrogen".to_string(),
        Tag::Compound(vec![(
            "target_status".to_string(),
            Tag::String("minecraft:empty".to_string()),
        )]),
    ));
    let err = LevelChunk::from_nbt(pos, &empty_status).unwrap_err();
    assert!(err.contains("target_status cannot be empty"));

    let mut unknown_status = LevelChunk::empty(pos).to_nbt(TARGET_DATA_VERSION);
    let Tag::Compound(fields) = &mut unknown_status else {
        panic!("chunk should encode as a compound");
    };
    fields.push((
        "below_zero_retrogen".to_string(),
        Tag::Compound(vec![(
            "target_status".to_string(),
            Tag::String("minecraft:not_a_status".to_string()),
        )]),
    ));
    let err = LevelChunk::from_nbt(pos, &unknown_status).unwrap_err();
    assert!(err.contains("not a known chunk status"));

    let mut wrong_bedrock_shape = LevelChunk::empty(pos).to_nbt(TARGET_DATA_VERSION);
    let Tag::Compound(fields) = &mut wrong_bedrock_shape else {
        panic!("chunk should encode as a compound");
    };
    fields.push((
        "below_zero_retrogen".to_string(),
        Tag::Compound(vec![
            (
                "target_status".to_string(),
                Tag::String("minecraft:noise".to_string()),
            ),
            ("missing_bedrock".to_string(), Tag::List(Vec::new())),
        ]),
    ));
    let err = LevelChunk::from_nbt(pos, &wrong_bedrock_shape).unwrap_err();
    assert!(err.contains("NBT field missing_bedrock must be a long array"));
}

#[test]
fn empty_level_chunk_uses_vanilla_structures_payload_shape() {
    let chunk = LevelChunk::empty(ChunkPos { x: 0, z: 0 });
    let Tag::Compound(fields) = &chunk.structures else {
        panic!("structures payload should be a compound");
    };
    assert!(matches!(
        fields.iter().find(|(name, _)| name == "starts"),
        Some((_, Tag::Compound(starts))) if starts.is_empty()
    ));
    assert!(matches!(
        fields.iter().find(|(name, _)| name == "References"),
        Some((_, Tag::Compound(references))) if references.is_empty()
    ));
}

#[test]
fn level_chunk_stores_structure_starts_and_nearby_references() {
    let mut chunk = LevelChunk::empty(ChunkPos { x: 10, z: -10 });
    let mineshaft = Tag::Compound(vec![(
        "id".to_string(),
        Tag::String("minecraft:mineshaft".to_string()),
    )]);
    let village = Tag::Compound(vec![(
        "id".to_string(),
        Tag::String("minecraft:village".to_string()),
    )]);

    assert!(chunk.set_structure_start_nbt("minecraft:mineshaft", mineshaft));
    assert!(chunk.set_structure_start_nbt("minecraft:mineshaft", village.clone()));
    assert!(!chunk.set_structure_start_nbt("minecraft:bad", Tag::List(Vec::new())));
    assert!(chunk.add_structure_reference("minecraft:village", ChunkPos { x: 18, z: -2 }));
    assert!(chunk.add_structure_reference("minecraft:village", ChunkPos { x: 18, z: -2 }));
    assert!(!chunk.add_structure_reference("minecraft:village", ChunkPos { x: 19, z: -10 }));

    let Tag::Compound(fields) = &chunk.structures else {
        panic!("structures payload should be a compound");
    };
    assert!(matches!(
        fields.iter().find(|(name, _)| name == "starts"),
        Some((_, Tag::Compound(starts)))
            if starts == &vec![("minecraft:mineshaft".to_string(), village)]
    ));
    assert!(matches!(
        fields.iter().find(|(name, _)| name == "References"),
        Some((_, Tag::Compound(references)))
            if references == &vec![(
                "minecraft:village".to_string(),
                Tag::LongArray(vec![super::pack_chunk_pos_as_long(ChunkPos { x: 18, z: -2 })])
            )]
    ));
    assert_eq!(
        super::chunk_pos_chessboard_distance(ChunkPos { x: 10, z: -10 }, ChunkPos { x: 18, z: -2 }),
        8
    );
}

#[test]
fn level_chunk_marks_postprocessing_offsets_by_section() {
    let mut chunk = LevelChunk::empty(ChunkPos { x: 3, z: -2 });
    chunk.min_section_y = -1;
    chunk.sections = vec![
        ChunkSection {
            y: -1,
            block_states: Tag::Compound(Vec::new()),
            biomes: Tag::Compound(Vec::new()),
            block_light: None,
            sky_light: None,
        },
        ChunkSection {
            y: 0,
            block_states: Tag::Compound(Vec::new()),
            biomes: Tag::Compound(Vec::new()),
            block_light: None,
            sky_light: None,
        },
    ];

    assert!(chunk.mark_pos_for_postprocessing(48, -1, -17));
    assert!(chunk.mark_pos_for_postprocessing(63, 0, -32));
    assert!(!chunk.mark_pos_for_postprocessing(48, 16, -17));

    assert_eq!(
        chunk.post_processing,
        vec![
            Tag::List(vec![Tag::Short(pack_postprocessing_offset(48, -1, -17))]),
            Tag::List(vec![Tag::Short(pack_postprocessing_offset(63, 0, -32))]),
        ]
    );
}

#[test]
fn postprocessing_offsets_unpack_to_world_coordinates() {
    let chunk_pos = ChunkPos { x: 3, z: -2 };
    let packed = pack_postprocessing_offset(63, -1, -17);

    assert_eq!(
        unpack_postprocessing_offset(packed, -1, chunk_pos),
        (63, -1, -17)
    );
    assert_eq!(
        unpack_postprocessing_offset(pack_postprocessing_offset(48, 0, -32), 0, chunk_pos),
        (48, 0, -32)
    );
}

#[test]
fn level_chunk_sets_pending_block_entity_nbt_by_position() {
    let mut chunk = LevelChunk::empty(ChunkPos { x: 1, z: 1 });
    let chest = Tag::Compound(vec![
        ("id".to_string(), Tag::String("minecraft:chest".to_string())),
        ("x".to_string(), Tag::Int(20)),
        ("y".to_string(), Tag::Int(64)),
        ("z".to_string(), Tag::Int(23)),
        ("keep".to_string(), Tag::Byte(1)),
    ]);
    let barrel = Tag::Compound(vec![
        (
            "id".to_string(),
            Tag::String("minecraft:barrel".to_string()),
        ),
        ("x".to_string(), Tag::Int(20)),
        ("y".to_string(), Tag::Int(64)),
        ("z".to_string(), Tag::Int(23)),
    ]);
    let malformed = Tag::Compound(vec![(
        "id".to_string(),
        Tag::String("minecraft:furnace".to_string()),
    )]);

    assert!(chunk.set_block_entity_nbt(chest));
    assert!(chunk.set_block_entity_nbt(barrel.clone()));
    assert!(!chunk.set_block_entity_nbt(malformed));

    assert_eq!(chunk.block_entities, vec![barrel]);
}

#[test]
fn level_chunk_adds_proto_entity_nbt_in_generation_order() {
    let mut chunk = LevelChunk::empty(ChunkPos { x: 0, z: 0 });
    let pig = Tag::Compound(vec![(
        "id".to_string(),
        Tag::String("minecraft:pig".to_string()),
    )]);
    let cow = Tag::Compound(vec![(
        "id".to_string(),
        Tag::String("minecraft:cow".to_string()),
    )]);

    assert!(chunk.add_entity_nbt(pig.clone()));
    assert!(!chunk.add_entity_nbt(Tag::String("minecraft:bat".to_string())));
    assert!(chunk.add_entity_nbt(cow.clone()));

    assert_eq!(chunk.entities, vec![pig, cow]);
}

#[test]
fn level_chunk_schedules_block_and_fluid_ticks_for_own_chunk() {
    let mut chunk = LevelChunk::empty(ChunkPos { x: -2, z: 3 });

    assert!(chunk.schedule_block_tick("minecraft:oak_sapling", -17, 65, 48, 7, TickPriority::High));
    assert!(chunk.schedule_fluid_tick("minecraft:water", -32, -5, 63, 1, TickPriority::VeryLow));
    assert!(!chunk.schedule_block_tick("minecraft:stone", -33, 65, 48, 0, TickPriority::Normal));

    assert_eq!(
        chunk.block_ticks,
        vec![Tag::Compound(vec![
            (
                "i".to_string(),
                Tag::String("minecraft:oak_sapling".to_string())
            ),
            ("x".to_string(), Tag::Int(-17)),
            ("y".to_string(), Tag::Int(65)),
            ("z".to_string(), Tag::Int(48)),
            ("t".to_string(), Tag::Int(7)),
            ("p".to_string(), Tag::Int(-1)),
        ])]
    );
    assert_eq!(
        chunk.fluid_ticks,
        vec![Tag::Compound(vec![
            ("i".to_string(), Tag::String("minecraft:water".to_string())),
            ("x".to_string(), Tag::Int(-32)),
            ("y".to_string(), Tag::Int(-5)),
            ("z".to_string(), Tag::Int(63)),
            ("t".to_string(), Tag::Int(1)),
            ("p".to_string(), Tag::Int(2)),
        ])]
    );
}

#[test]
fn level_chunk_load_filters_saved_ticks_to_own_chunk() {
    let pos = ChunkPos { x: -2, z: 3 };
    let mut chunk = LevelChunk::empty(pos);
    chunk.schedule_block_tick("minecraft:oak_sapling", -17, 65, 48, 7, TickPriority::High);
    chunk.block_ticks.push(saved_tick_tag(
        "minecraft:stone".to_string(),
        -33,
        65,
        48,
        0,
        TickPriority::Normal,
    ));
    chunk.schedule_fluid_tick("minecraft:water", -32, -5, 63, 1, TickPriority::VeryLow);
    chunk.fluid_ticks.push(saved_tick_tag(
        "minecraft:lava".to_string(),
        -17,
        20,
        64,
        0,
        TickPriority::Normal,
    ));

    let decoded = LevelChunk::from_nbt(pos, &chunk.to_nbt(TARGET_DATA_VERSION)).unwrap();

    assert_eq!(decoded.block_ticks.len(), 1);
    assert_eq!(decoded.fluid_ticks.len(), 1);
    assert!(matches!(
        &decoded.block_ticks[0],
        Tag::Compound(fields)
            if string_field(fields, "i").unwrap() == "minecraft:oak_sapling"
    ));
    assert!(matches!(
        &decoded.fluid_ticks[0],
        Tag::Compound(fields) if string_field(fields, "i").unwrap() == "minecraft:water"
    ));
}

#[test]
fn level_chunk_sets_valid_section_light_arrays() {
    let mut chunk = LevelChunk::empty(ChunkPos { x: 0, z: 0 });
    chunk.sections = vec![
        ChunkSection {
            y: -1,
            block_states: Tag::Compound(Vec::new()),
            biomes: Tag::Compound(Vec::new()),
            block_light: None,
            sky_light: None,
        },
        ChunkSection {
            y: 0,
            block_states: Tag::Compound(Vec::new()),
            biomes: Tag::Compound(Vec::new()),
            block_light: None,
            sky_light: None,
        },
    ];

    assert!(!chunk.set_section_light_arrays(-1, Some(vec![0; 17]), None));
    assert!(!chunk.set_section_light_arrays(1, Some(vec![0; LIGHT_DATA_LAYER_LENGTH]), None));
    assert!(chunk.set_section_light_arrays(-1, Some(vec![0; LIGHT_DATA_LAYER_LENGTH]), None));
    assert!(!chunk.light_correct);
    assert!(chunk.set_section_light_arrays(0, None, Some(vec![15; LIGHT_DATA_LAYER_LENGTH])));

    assert_eq!(
        chunk.sections[0].block_light.as_ref().unwrap().len(),
        LIGHT_DATA_LAYER_LENGTH
    );
    assert_eq!(chunk.sections[1].sky_light.as_ref().unwrap()[0], 15);
    assert!(chunk.light_correct);
}

#[test]
fn level_chunk_light_handoff_plan_matches_vanilla_section_queueing() {
    let mut chunk = LevelChunk::empty(ChunkPos { x: 0, z: 0 });
    chunk.sections = vec![
        ChunkSection {
            y: -1,
            block_states: Tag::Compound(Vec::new()),
            biomes: Tag::Compound(Vec::new()),
            block_light: Some(vec![1; LIGHT_DATA_LAYER_LENGTH]),
            sky_light: Some(vec![15; LIGHT_DATA_LAYER_LENGTH]),
        },
        ChunkSection {
            y: 0,
            block_states: Tag::Compound(Vec::new()),
            biomes: Tag::Compound(Vec::new()),
            block_light: None,
            sky_light: Some(vec![7; LIGHT_DATA_LAYER_LENGTH]),
        },
        ChunkSection {
            y: 1,
            block_states: Tag::Compound(Vec::new()),
            biomes: Tag::Compound(Vec::new()),
            block_light: None,
            sky_light: None,
        },
    ];

    let overworld_plan = chunk.light_handoff_plan(true);

    assert!(overworld_plan.retain_data);
    assert_eq!(
        overworld_plan.queued_sections,
        vec![
            QueuedSectionLightData {
                layer: LightLayer::Block,
                section_y: -1,
                data: vec![1; LIGHT_DATA_LAYER_LENGTH],
            },
            QueuedSectionLightData {
                layer: LightLayer::Sky,
                section_y: -1,
                data: vec![15; LIGHT_DATA_LAYER_LENGTH],
            },
            QueuedSectionLightData {
                layer: LightLayer::Sky,
                section_y: 0,
                data: vec![7; LIGHT_DATA_LAYER_LENGTH],
            },
        ]
    );

    let nether_plan = chunk.light_handoff_plan(false);

    assert!(nether_plan.retain_data);
    assert_eq!(
        nether_plan.queued_sections,
        vec![QueuedSectionLightData {
            layer: LightLayer::Block,
            section_y: -1,
            data: vec![1; LIGHT_DATA_LAYER_LENGTH],
        }]
    );
    assert!(
        !LevelChunk::empty(ChunkPos { x: 1, z: 1 })
            .light_handoff_plan(true)
            .retain_data
    );
}

#[test]
fn light_data_layer_nibble_indexing_matches_vanilla() {
    assert_eq!(LIGHT_DATA_LAYER_WIDTH, 16);
    assert_eq!(LIGHT_DATA_LAYER_ROW_SIZE, 128);
    assert_eq!(LIGHT_DATA_LAYER_LENGTH, 2048);
    assert_eq!(LIGHT_DATA_LAYER_NIBBLE_COUNT, 4096);

    let index = super::light_data_layer_index(2, 3, 4);
    assert_eq!(index, 0x342);
    assert_eq!(super::light_data_layer_byte_index(index), 0x1a1);
    assert_eq!(super::light_data_layer_nibble_index(index), 0);

    let odd_index = super::light_data_layer_index(3, 3, 4);
    assert_eq!(odd_index, 0x343);
    assert_eq!(super::light_data_layer_byte_index(odd_index), 0x1a1);
    assert_eq!(super::light_data_layer_nibble_index(odd_index), 1);

    assert_eq!(super::light_data_layer_pack_filled(0), 0x00);
    assert_eq!(super::light_data_layer_pack_filled(15), -1);
    assert_eq!(super::light_data_layer_pack_filled(18), 0x22);

    let mut data = vec![0_i8; LIGHT_DATA_LAYER_LENGTH];
    data[0x1a1] = 0xab_u8 as i8;
    assert_eq!(super::light_data_layer_get(Some(&data), 0, 2, 3, 4), 0x0b);
    assert_eq!(super::light_data_layer_get(Some(&data), 0, 3, 3, 4), 0x0a);
    assert_eq!(super::light_data_layer_get(None, 15, 3, 3, 4), 15);
    assert_eq!(super::light_data_layer_get(None, 18, 3, 3, 4), 2);
}

#[test]
fn light_data_layer_set_materializes_and_masks_like_vanilla() {
    let mut data = None;

    super::light_data_layer_set(&mut data, 15, 2, 3, 4, 0);
    let data = data.as_mut().expect("setting should materialize the layer");

    assert_eq!(data.len(), LIGHT_DATA_LAYER_LENGTH);
    assert_eq!(super::light_data_layer_get(Some(data), 15, 2, 3, 4), 0);
    assert_eq!(super::light_data_layer_get(Some(data), 15, 3, 3, 4), 15);

    let mut data = Some(data.clone());
    super::light_data_layer_set(&mut data, 15, 3, 3, 4, 18);
    let data = data.as_ref().unwrap();

    assert_eq!(super::light_data_layer_get(Some(data), 15, 3, 3, 4), 2);
    assert_eq!(super::light_data_layer_get(Some(data), 15, 2, 3, 4), 0);
}

#[test]
fn light_data_layer_homogenous_predicates_match_vanilla() {
    assert!(super::light_data_layer_is_definitely_homogenous(None));
    assert!(super::light_data_layer_is_definitely_filled_with(
        None, 15, 31
    ));
    assert!(super::light_data_layer_is_empty(None, 0));
    assert!(!super::light_data_layer_is_empty(None, 15));

    let data = super::light_data_layer_materialize(0);

    assert!(!super::light_data_layer_is_definitely_homogenous(Some(
        &data
    )));
    assert!(!super::light_data_layer_is_definitely_filled_with(
        Some(&data),
        0,
        0
    ));
    assert!(!super::light_data_layer_is_empty(Some(&data), 0));
}

#[test]
fn level_chunk_finds_block_light_sources_by_section() {
    let mut chunk = LevelChunk::empty(ChunkPos { x: -2, z: 3 });
    chunk.min_section_y = -1;
    chunk.sections = vec![
        ChunkSection {
            y: -1,
            block_states: default_block_states_container(),
            biomes: default_biomes_container(),
            block_light: None,
            sky_light: None,
        },
        ChunkSection {
            y: 0,
            block_states: default_block_states_container(),
            biomes: default_biomes_container(),
            block_light: None,
            sky_light: None,
        },
    ];

    chunk.set_block_state(-31, -1, 48, "minecraft:torch");
    chunk.set_block_state(-17, 2, 63, "minecraft:stone");
    chunk.set_block_state(-32, 3, 63, "minecraft:torch");

    assert_eq!(
        chunk.find_block_light_sources(),
        vec![
            (-31, -1, 48, "minecraft:torch".to_string(), 14),
            (-32, 3, 63, "minecraft:torch".to_string(), 14),
        ]
    );
}

#[test]
fn level_chunk_reports_light_section_status_from_air_only_sections() {
    let mut chunk = LevelChunk::empty(ChunkPos { x: 0, z: 0 });
    chunk.min_section_y = -1;
    chunk.sections = vec![
        ChunkSection {
            y: -1,
            block_states: default_block_states_container(),
            biomes: default_biomes_container(),
            block_light: None,
            sky_light: None,
        },
        ChunkSection {
            y: 0,
            block_states: default_block_states_container(),
            biomes: default_biomes_container(),
            block_light: None,
            sky_light: None,
        },
        ChunkSection {
            y: 1,
            block_states: Tag::Compound(vec![(
                "palette".to_string(),
                Tag::List(vec![BlockStateEntry::new("minecraft:stone").to_nbt()]),
            )]),
            biomes: default_biomes_container(),
            block_light: None,
            sky_light: None,
        },
    ];

    chunk.set_block_state(1, 3, 1, "minecraft:torch");

    assert!(chunk.sections[0].has_only_air());
    assert_eq!(
        chunk.light_section_status_updates(),
        vec![
            LightSectionStatusUpdate {
                section_y: -1,
                has_only_air: true,
            },
            LightSectionStatusUpdate {
                section_y: 0,
                has_only_air: false,
            },
            LightSectionStatusUpdate {
                section_y: 1,
                has_only_air: false,
            },
        ]
    );
}

#[test]
fn level_chunk_initialize_light_plan_matches_threaded_engine_order() {
    let mut chunk = LevelChunk::empty(ChunkPos { x: 4, z: -3 });
    chunk.min_section_y = -1;
    chunk.sections = vec![
        ChunkSection {
            y: -1,
            block_states: default_block_states_container(),
            biomes: default_biomes_container(),
            block_light: None,
            sky_light: None,
        },
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
    chunk.set_block_state(64, 4, -48, "minecraft:stone");
    chunk.set_block_state(65, 17, -47, "minecraft:torch");

    assert_eq!(
        chunk.initialize_light_plan(true),
        ChunkInitializeLightPlan {
            pre_update_section_statuses: vec![
                LightSectionStatusUpdate {
                    section_y: 0,
                    has_only_air: false,
                },
                LightSectionStatusUpdate {
                    section_y: 1,
                    has_only_air: false,
                },
            ],
            post_update_light_enabled: true,
            post_update_retain_data: false,
        }
    );
    assert_eq!(
        LevelChunk::empty(ChunkPos { x: 0, z: 0 }).initialize_light_plan(false),
        ChunkInitializeLightPlan {
            pre_update_section_statuses: Vec::new(),
            post_update_light_enabled: false,
            post_update_retain_data: false,
        }
    );
}

#[test]
fn level_chunk_light_completion_plan_matches_threaded_engine_order() {
    let chunk = LevelChunk::empty(ChunkPos { x: 2, z: 5 });

    assert_eq!(
        chunk.light_completion_plan(false),
        ChunkLightCompletionPlan {
            initial_light_correct: false,
            pre_update_propagate_light_sources: true,
            completed_light_correct: true,
        }
    );
    assert_eq!(
        chunk.light_completion_plan(true),
        ChunkLightCompletionPlan {
            initial_light_correct: false,
            pre_update_propagate_light_sources: false,
            completed_light_correct: true,
        }
    );
}

#[test]
fn level_chunk_rejects_misplaced_payloads() {
    let tag = LevelChunk::empty(ChunkPos { x: 9, z: 9 }).to_nbt(TARGET_DATA_VERSION);
    let err = LevelChunk::from_nbt(ChunkPos { x: 0, z: 0 }, &tag).unwrap_err();
    assert!(err.contains("wrong position"));
}

#[test]
fn level_chunk_rejects_missing_or_unsupported_data_versions() {
    let pos = ChunkPos { x: 0, z: 0 };
    let mut missing = LevelChunk::empty(pos).to_nbt(TARGET_DATA_VERSION);
    if let Tag::Compound(values) = &mut missing {
        values.retain(|(name, _)| name != "DataVersion");
    }
    let err = LevelChunk::from_nbt(pos, &missing).unwrap_err();
    assert!(err.contains("missing DataVersion"));

    let unsupported =
        LevelChunk::empty(pos).to_nbt(crate::storage::datafix::TARGET_DATA_VERSION - 1);
    let err = LevelChunk::from_nbt(pos, &unsupported).unwrap_err();
    assert!(err.contains("Unsupported chunk DataVersion"));
}

#[test]
fn level_chunk_normalizes_unknown_status_like_vanilla_storage_codec() {
    let pos = ChunkPos { x: 0, z: 0 };
    let mut tag = LevelChunk::empty(pos).to_nbt(TARGET_DATA_VERSION);
    let Tag::Compound(fields) = &mut tag else {
        panic!("chunk should encode as a compound");
    };
    let (_, status) = fields
        .iter_mut()
        .find(|(name, _)| name == "Status")
        .expect("Status should be present");
    *status = Tag::String("minecraft:not_a_status".to_string());

    let decoded = LevelChunk::from_nbt(pos, &tag).unwrap();

    assert_eq!(decoded.status, "minecraft:empty");
}

#[test]
fn level_chunk_rejects_empty_status_like_vanilla_parse_null_path() {
    let pos = ChunkPos { x: 0, z: 0 };
    let mut tag = LevelChunk::empty(pos).to_nbt(TARGET_DATA_VERSION);
    let Tag::Compound(fields) = &mut tag else {
        panic!("chunk should encode as a compound");
    };
    let (_, status) = fields
        .iter_mut()
        .find(|(name, _)| name == "Status")
        .expect("Status should be present");
    *status = Tag::String(String::new());

    let err = LevelChunk::from_nbt(pos, &tag).unwrap_err();

    assert!(err.contains("Status cannot be empty"));
}

mod generation_tests;
