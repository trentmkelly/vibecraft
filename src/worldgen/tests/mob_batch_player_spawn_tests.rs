use super::*;

#[test]
fn apply_chunk_generation_mob_batch_to_chunk_queues_successful_placements() {
    let normal = super::super::resolve_world_preset("normal").unwrap();
    let mut chunk =
        super::super::generator_build_surface_for_stem(ChunkPos { x: 2, z: -3 }, &normal.overworld)
            .expect("surface chunk should generate");
    let batch = super::super::ChunkGenerationMobSpawnBatchPlan {
        category: "creature",
        entity_type: "minecraft:pig",
        count: 1,
        start_x: 37,
        start_z: -37,
    };
    let mut random = crate::random_source::RandomSourceKind::new(
        1,
        crate::random_source::RandomAlgorithm::Legacy,
    );

    let spawned = super::super::apply_chunk_generation_mob_batch_to_chunk(
        &mut chunk,
        batch,
        false,
        &mut random,
        &["00000000-0000-0000-0000-000000000456"],
    );

    assert_eq!(spawned, 1);
    assert_eq!(chunk.entities.len(), 1);
    let Tag::Compound(fields) = &chunk.entities[0] else {
        panic!("queued entity must be a compound");
    };
    assert!(fields.contains(&("id".to_string(), Tag::String("minecraft:pig".to_string()))));
    assert!(fields.contains(&(
        "UUID".to_string(),
        Tag::String("00000000-0000-0000-0000-000000000456".to_string())
    )));
}

#[test]
fn apply_chunk_generation_mob_batch_to_chunk_generates_missing_uuids() {
    let normal = super::super::resolve_world_preset("normal").unwrap();
    let mut chunk =
        super::super::generator_build_surface_for_stem(ChunkPos { x: 2, z: -3 }, &normal.overworld)
            .expect("surface chunk should generate");
    let batch = super::super::ChunkGenerationMobSpawnBatchPlan {
        category: "creature",
        entity_type: "minecraft:pig",
        count: 1,
        start_x: 37,
        start_z: -37,
    };
    let mut random = crate::random_source::RandomSourceKind::new(
        1,
        crate::random_source::RandomAlgorithm::Legacy,
    );

    let spawned = super::super::apply_chunk_generation_mob_batch_to_chunk(
        &mut chunk,
        batch,
        false,
        &mut random,
        &[],
    );

    assert_eq!(spawned, 1);
    let Tag::Compound(fields) = &chunk.entities[0] else {
        panic!("queued entity must be a compound");
    };
    let uuid = fields
        .iter()
        .find_map(|(name, value)| match (name.as_str(), value) {
            ("UUID", Tag::String(uuid)) => Some(uuid),
            _ => None,
        })
        .expect("generated entity should have a UUID");
    assert_eq!(uuid.len(), 36);
    assert_eq!(&uuid[14..15], "4");
}

#[test]
fn create_insecure_uuid_matches_mth_version_and_variant_bits() {
    let mut random = crate::random_source::RandomSourceKind::new(
        1,
        crate::random_source::RandomAlgorithm::Legacy,
    );

    assert_eq!(
        super::super::create_insecure_uuid(&mut random),
        "bb1ad573-19b8-4cd8-a8fb-0e6f684df992"
    );
}

#[test]
fn player_spawn_search_candidate_math_matches_vanilla() {
    assert_eq!(super::super::spawn_search_candidate_count(0), 1);
    assert_eq!(super::super::spawn_search_candidate_count(1), 9);
    assert_eq!(super::super::spawn_search_candidate_count(16), 1024);
    assert_eq!(super::super::spawn_search_coprime(9), 8);
    assert_eq!(super::super::spawn_search_coprime(17), 17);

    assert_eq!(super::super::spawn_search_radius(10, 20), 10);
    assert_eq!(super::super::spawn_search_radius(10, 4), 4);
    assert_eq!(super::super::spawn_search_radius(10, 1), 1);
    assert_eq!(super::super::spawn_search_radius(-5, 20), 0);

    assert_eq!(
        super::super::spawn_search_candidate(100, 200, 1, 0, 0),
        Some((99, 199))
    );
    assert_eq!(
        super::super::spawn_search_candidate(100, 200, 1, 0, 1),
        Some((101, 201))
    );
    assert_eq!(
        super::super::spawn_search_candidate(100, 200, 1, 8, 0),
        Some((101, 201))
    );
    assert_eq!(
        super::super::spawn_search_candidate(100, 200, 1, 0, 9),
        None
    );
}

#[test]
fn initial_spawn_readiness_requires_player_spawn_ticket_radius_full_chunks() {
    let center = ChunkPos { x: 2, z: -1 };
    let required = super::super::initial_spawn_required_chunks(center);
    assert_eq!(required.len(), 49);
    assert_eq!(required.first(), Some(&ChunkPos { x: -1, z: -4 }));
    assert_eq!(required.last(), Some(&ChunkPos { x: 5, z: 2 }));

    let mut snapshots: Vec<_> = required
        .iter()
        .copied()
        .map(|pos| super::super::SpawnChunkStatusSnapshot {
            pos,
            status: "minecraft:full",
        })
        .collect();
    assert!(super::super::initial_spawn_chunks_ready(center, &snapshots));

    snapshots[0].status = "minecraft:light";
    let report = super::super::initial_spawn_readiness_report(center, &snapshots);
    assert_eq!(report.required_radius, 3);
    assert_eq!(report.required_status, "minecraft:full");
    assert_eq!(report.ticket_type, "minecraft:player_spawn");
    assert_eq!(
        report.ticket_level,
        crate::chunk_ticket::FULL_CHUNK_LEVEL - 3
    );
    assert_eq!(report.missing_chunks, Vec::<ChunkPos>::new());
    assert_eq!(
        report.not_ready_chunks,
        vec![super::super::SpawnChunkStatusSnapshot {
            pos: ChunkPos { x: -1, z: -4 },
            status: "minecraft:light",
        }]
    );
    assert!(!report.is_ready());

    snapshots.pop();
    let report = super::super::initial_spawn_readiness_report(center, &snapshots);
    assert_eq!(report.missing_chunks, vec![ChunkPos { x: 5, z: 2 }]);
}

#[test]
fn overworld_respawn_candidate_rules_match_vanilla() {
    let normal_column = SpawnColumnHeights {
        top_y: 64,
        surface_y: 66,
        ocean_floor_y: 63,
        min_y: -64,
    };
    assert_eq!(
        super::super::overworld_respawn_y(
            normal_column,
            false,
            &[
                SpawnBlockKind::Air,
                SpawnBlockKind::Air,
                SpawnBlockKind::Solid
            ]
        ),
        Some(64)
    );

    assert_eq!(
        super::super::overworld_respawn_y(
            SpawnColumnHeights {
                top_y: -80,
                ..normal_column
            },
            false,
            &[SpawnBlockKind::Solid]
        ),
        None
    );
    assert_eq!(
        super::super::overworld_respawn_y(
            SpawnColumnHeights {
                surface_y: 64,
                ocean_floor_y: 62,
                ..normal_column
            },
            false,
            &[SpawnBlockKind::Solid]
        ),
        None
    );
    assert_eq!(
        super::super::overworld_respawn_y(
            normal_column,
            false,
            &[
                SpawnBlockKind::Air,
                SpawnBlockKind::Fluid,
                SpawnBlockKind::Solid
            ]
        ),
        None
    );
}

#[test]
fn spawn_height_fixup_walks_like_vanilla() {
    let blocked_until_70 = |y| y >= 70;
    assert_eq!(
        super::super::fixup_spawn_height(64, -64, 320, blocked_until_70),
        70
    );

    let air_above_ground = |y| y >= 65;
    assert_eq!(
        super::super::fixup_spawn_height(80, -64, 320, air_above_ground),
        65
    );
}
