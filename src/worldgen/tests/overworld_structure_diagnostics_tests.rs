use super::*;

    #[test]
    #[ignore = "diagnostic for missing cave-air that may come from underground structures"]
    fn normal_overworld_cave_air_structure_candidate_diagnostic() {
        let fixture_json =
            include_str!("../../../harness/mineflayer/fixtures/vanilla_worldgen_block_array_target.json");
        let fixture: serde_json::Value =
            serde_json::from_str(fixture_json).expect("vanilla fixture should parse");
        let seed = fixture
            .get("seed")
            .and_then(serde_json::Value::as_str)
            .expect("vanilla fixture should include a seed")
            .parse::<i64>()
            .expect("vanilla fixture seed should parse");
        let fixture_chunks = fixture
            .get("chunks")
            .and_then(serde_json::Value::as_array)
            .expect("fixture should include chunks");

        let preset = super::super::resolve_world_preset("normal").expect("normal preset should resolve");
        let super::super::ResolvedChunkGenerator::Noise {
            biome_source_model,
            noise_settings,
            ..
        } = &preset.overworld.generator
        else {
            panic!("normal overworld should use a noise generator");
        };

        let mut generated_chunks = Vec::new();
        for fixture_chunk in fixture_chunks {
            let pos = ChunkPos {
                x: fixture_chunk
                    .get("chunkX")
                    .and_then(serde_json::Value::as_i64)
                    .expect("fixture chunk should include chunkX") as i32,
                z: fixture_chunk
                    .get("chunkZ")
                    .and_then(serde_json::Value::as_i64)
                    .expect("fixture chunk should include chunkZ") as i32,
            };
            let (base, _, _) = super::super::generate_real_surface_base_chunk(
                pos,
                biome_source_model,
                noise_settings,
                seed,
            )
            .expect("real-surface base generation should succeed");
            let mut chunk = base;
            super::super::apply_configured_carvers_for_biome_source(
                &mut chunk,
                biome_source_model,
                noise_settings,
                seed,
            );
            super::super::apply_underground_ore_decoration_to_chunk(
                &mut chunk,
                biome_source_model,
                noise_settings,
                seed,
                None,
            );
            super::super::apply_initial_tree_decoration_to_chunk(
                &mut chunk,
                biome_source_model,
                noise_settings,
                seed,
                None,
                None,
                None,
            );
            generated_chunks.push(chunk);
        }

        let mut missing_cave_air = Vec::new();
        for (chunk_index, fixture_chunk) in fixture_chunks.iter().enumerate() {
            let chunk_x = fixture_chunk
                .get("chunkX")
                .and_then(serde_json::Value::as_i64)
                .expect("fixture chunk should include chunkX") as i32;
            let chunk_z = fixture_chunk
                .get("chunkZ")
                .and_then(serde_json::Value::as_i64)
                .expect("fixture chunk should include chunkZ") as i32;
            let y_min = fixture_chunk
                .get("yMin")
                .and_then(serde_json::Value::as_i64)
                .expect("fixture chunk should include yMin") as i32;
            let blocks = fixture_chunk
                .get("blocks")
                .and_then(serde_json::Value::as_array)
                .expect("fixture chunk should include blocks");
            let generated = &generated_chunks[chunk_index];

            for (local_x, y_column) in blocks.iter().enumerate() {
                let y_column = y_column.as_array().expect("x column should be an array");
                for (y_offset, z_column) in y_column.iter().enumerate() {
                    let world_y = y_min + y_offset as i32;
                    let z_column = z_column.as_array().expect("z column should be an array");
                    for (local_z, expected) in z_column.iter().enumerate() {
                        let expected = expected
                            .as_str()
                            .expect("fixture block should be a string")
                            .split_once('[')
                            .map_or_else(|| expected.as_str().unwrap(), |(id, _)| id);
                        if expected != "minecraft:cave_air" {
                            continue;
                        }
                        let world_x = chunk_x * 16 + local_x as i32;
                        let world_z = chunk_z * 16 + local_z as i32;
                        let actual = generated
                            .get_block_state(world_x, world_y, world_z)
                            .unwrap_or_else(|| "minecraft:air".to_string());
                        if actual == "minecraft:deepslate" || actual == "minecraft:stone" {
                            missing_cave_air.push(BlockPos {
                                x: world_x,
                                y: world_y,
                                z: world_z,
                            });
                        }
                    }
                }
            }
        }

        let missing_box = missing_cave_air.iter().copied().fold(None, |acc, pos| {
            Some(acc.map_or_else(
                || super::super::StructureBoundingBoxModel {
                    min_x: pos.x,
                    min_y: pos.y,
                    min_z: pos.z,
                    max_x: pos.x,
                    max_y: pos.y,
                    max_z: pos.z,
                },
                |box_: super::super::StructureBoundingBoxModel| box_.encapsulate_pos(pos),
            ))
        });
        eprintln!(
            "[cave-air-structure] missing_cave_air={} missing_box={:?}",
            missing_cave_air.len(),
            missing_box
        );

        let mut candidate_count = 0usize;
        let mut near_missing_count = 0usize;
        let mut closest_candidates = Vec::new();
        for source_x in -64..=64 {
            for source_z in -64..=64 {
                if !super::super::structure_frequency_reducer_should_generate(
                    super::super::FrequencyReductionMethod::LegacyType3,
                    seed,
                    0,
                    source_x,
                    source_z,
                    0.004,
                )
                .expect("mineshaft frequency check should be valid")
                {
                    continue;
                }
                candidate_count += 1;

                let mut random = super::super::RandomSourceKind::Legacy(super::super::LegacyRandom::new(
                    crate::random_source::large_feature_seed(seed, source_x, source_z),
                ));
                let first_roll = super::super::random_next_f64(&mut random);
                let room = super::super::mineshaft_room(
                    ChunkPos {
                        x: source_x,
                        z: source_z,
                    },
                    super::super::MineshaftTypeModel::Normal,
                    super::super::random_next_i32_bound(&mut random, 6),
                    super::super::random_next_i32_bound(&mut random, 6),
                    super::super::random_next_i32_bound(&mut random, 6),
                )
                .expect("room rolls should be valid");
                let intersects_missing = missing_box
                    .is_some_and(|missing| room.bounding_box.inflated_by(96).intersects(missing));
                let pieces = super::super::mineshaft_generate_pieces_for_start(
                    seed,
                    ChunkPos {
                        x: source_x,
                        z: source_z,
                    },
                    super::super::MineshaftTypeModel::Normal,
                    63,
                    -64,
                );
                let intersecting_pieces = pieces
                    .iter()
                    .filter(|piece| {
                        missing_box.is_some_and(|missing| piece.bounding_box().intersects(missing))
                    })
                    .count();
                let closest_piece = missing_box.and_then(|missing| {
                    pieces
                        .iter()
                        .enumerate()
                        .map(|(index, piece)| {
                            let bb = piece.bounding_box();
                            let dx = if bb.max_x < missing.min_x {
                                missing.min_x - bb.max_x
                            } else if missing.max_x < bb.min_x {
                                bb.min_x - missing.max_x
                            } else {
                                0
                            };
                            let dy = if bb.max_y < missing.min_y {
                                missing.min_y - bb.max_y
                            } else if missing.max_y < bb.min_y {
                                bb.min_y - missing.max_y
                            } else {
                                0
                            };
                            let dz = if bb.max_z < missing.min_z {
                                missing.min_z - bb.max_z
                            } else if missing.max_z < bb.min_z {
                                bb.min_z - missing.max_z
                            } else {
                                0
                            };
                            (dx + dy + dz, index, bb)
                        })
                        .min_by_key(|(distance, _, _)| *distance)
                });
                if let Some((distance, index, bb)) = closest_piece {
                    closest_candidates.push((
                        distance,
                        source_x,
                        source_z,
                        pieces.len(),
                        index,
                        bb,
                    ));
                }
                if intersects_missing || intersecting_pieces > 0 {
                    if intersects_missing {
                        near_missing_count += 1;
                    }
                    eprintln!(
                        "[cave-air-structure-candidate] chunk=({}, {}) first_roll={:.9} room={:?} generated_pieces={} intersecting_pieces={} closest_piece={:?}",
                        source_x,
                        source_z,
                        first_roll,
                        room.bounding_box,
                        pieces.len(),
                        intersecting_pieces,
                        closest_piece
                    );
                }
            }
        }
        closest_candidates.sort_by_key(|(distance, _, _, _, _, _)| *distance);
        for (distance, source_x, source_z, piece_count, piece_index, bb) in
            closest_candidates.into_iter().take(8)
        {
            eprintln!(
                "[cave-air-structure-closest] distance={} chunk=({}, {}) generated_pieces={} piece_index={} box={:?}",
                distance, source_x, source_z, piece_count, piece_index, bb
            );
        }
        eprintln!(
            "[cave-air-structure] candidate_count={} near_missing_count={}",
            candidate_count, near_missing_count
        );
    }

    #[test]
    #[ignore = "diagnostic for the mineshaft start nearest the vanilla cave-air mismatch"]
    fn normal_overworld_mineshaft_nearest_start_diagnostic() {
        let fixture_json =
            include_str!("../../../harness/mineflayer/fixtures/vanilla_worldgen_block_array_target.json");
        let fixture: serde_json::Value =
            serde_json::from_str(fixture_json).expect("vanilla fixture should parse");
        let seed = fixture
            .get("seed")
            .and_then(serde_json::Value::as_str)
            .expect("vanilla fixture should include a seed")
            .parse::<i64>()
            .expect("vanilla fixture seed should parse");
        let preset = super::super::resolve_world_preset("normal").expect("normal preset should resolve");
        let super::super::ResolvedChunkGenerator::Noise {
            biome_source_model,
            noise_settings,
            ..
        } = &preset.overworld.generator
        else {
            panic!("normal overworld should use a noise generator");
        };
        let router =
            super::super::builtin_noise_router(super::super::noise_router_id_for_settings(**noise_settings))
                .expect("normal overworld should have a router")
                .router;
        let climate_sampler =
            super::super::ClimateSampler::from_noise_router(&router, seed, **noise_settings);

        let source_pos = ChunkPos { x: -1, z: 4 };
        let start_pos = super::super::mineshaft_start_pos(source_pos);
        let pieces = super::super::mineshaft_generate_pieces_for_start(
            seed,
            source_pos,
            super::super::MineshaftTypeModel::Normal,
            noise_settings.sea_level,
            noise_settings.noise.min_y,
        );
        let room_box = pieces[0].bounding_box();
        let y_offset = room_box.min_y - 50;
        let stub_pos = BlockPos {
            x: start_pos.x,
            y: start_pos.y + y_offset,
            z: start_pos.z,
        };
        let biome = super::super::get_biome(
            biome_source_model,
            stub_pos.x >> 2,
            stub_pos.y >> 2,
            stub_pos.z >> 2,
            &climate_sampler,
        )
        .unwrap_or("minecraft:unknown");
        eprintln!(
            "[mineshaft-nearest] source=({}, {}) seed={} start={:?} y_offset={} stub={:?} biome={} pieces={}",
            source_pos.x,
            source_pos.z,
            seed,
            start_pos,
            y_offset,
            stub_pos,
            biome,
            pieces.len()
        );

        for (index, piece) in pieces.iter().enumerate().take(40) {
            let kind = match piece {
                super::super::MineshaftGeneratedPieceModel::Room { .. } => "room",
                super::super::MineshaftGeneratedPieceModel::Corridor { .. } => "corridor",
                super::super::MineshaftGeneratedPieceModel::Crossing { .. } => "crossing",
                super::super::MineshaftGeneratedPieceModel::Stairs { .. } => "stairs",
            };
            eprintln!(
                "[mineshaft-nearest-first] index={} kind={} depth={} box={:?}",
                index,
                kind,
                piece.gen_depth(),
                piece.bounding_box()
            );
        }

        let missing = super::super::StructureBoundingBoxModel {
            min_x: 0,
            min_y: -56,
            min_z: 0,
            max_x: 22,
            max_y: -45,
            max_z: 15,
        };
        for (index, piece) in pieces.iter().enumerate() {
            let bb = piece.bounding_box();
            let dx = if bb.max_x < missing.min_x {
                missing.min_x - bb.max_x
            } else if missing.max_x < bb.min_x {
                bb.min_x - missing.max_x
            } else {
                0
            };
            let dy = if bb.max_y < missing.min_y {
                missing.min_y - bb.max_y
            } else if missing.max_y < bb.min_y {
                bb.min_y - missing.max_y
            } else {
                0
            };
            let dz = if bb.max_z < missing.min_z {
                missing.min_z - bb.max_z
            } else if missing.max_z < bb.min_z {
                bb.min_z - missing.max_z
            } else {
                0
            };
            let distance = dx + dy + dz;
            if distance <= 24 {
                let kind = match piece {
                    super::super::MineshaftGeneratedPieceModel::Room { .. } => "room",
                    super::super::MineshaftGeneratedPieceModel::Corridor { .. } => "corridor",
                    super::super::MineshaftGeneratedPieceModel::Crossing { .. } => "crossing",
                    super::super::MineshaftGeneratedPieceModel::Stairs { .. } => "stairs",
                };
                eprintln!(
                    "[mineshaft-nearest-piece] index={} kind={} depth={} distance={} box={:?}",
                    index,
                    kind,
                    piece.gen_depth(),
                    distance,
                    bb
                );
            }
        }
    }

