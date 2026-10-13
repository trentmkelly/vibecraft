use super::*;

fn test_box() -> StructureBoundingBoxModel {
    StructureBoundingBoxModel {
        min_x: -8,
        min_y: 45,
        min_z: 12,
        max_x: 13,
        max_y: 54,
        max_z: 31,
    }
}

fn test_random(seed: i64) -> RandomSourceKind {
    RandomSourceKind::Legacy(LegacyRandom::new(seed))
}

#[test]
fn mesa_vertical_adjustment_uses_inclusive_surface_range_and_one_draw() {
    let aggregate = test_box();
    let sea_level = 63;
    let surface_height = 67;
    let center = aggregate.center();
    // Mth.randomBetweenInclusive(seaLevel, surfaceHeight) is nextInt(5) +
    // seaLevel, so verify both endpoint rolls and the following random value.
    for endpoint_roll in [0, 4] {
        let seed = (0..512)
            .find(|seed| {
                let mut probe = test_random(*seed);
                random_next_i32_bound(&mut probe, 5) == endpoint_roll
            })
            .unwrap_or_else(|| panic!("bounded fixture search reaches roll {endpoint_roll}"));
        let mut actual_random = test_random(seed);
        let mut expected_random = actual_random;
        let expected_roll = random_next_i32_bound(&mut expected_random, 5);
        let expected_next = random_next_i32_bound(&mut expected_random, 101);
        let mut callback_coordinates = None;
        let dy = mineshaft_vertical_adjustment_dy(
            &mut actual_random,
            aggregate,
            MineshaftTypeModel::Mesa,
            sea_level,
            -64,
            |x, z| {
                callback_coordinates = Some((x, z));
                surface_height
            },
        );

        assert_eq!(expected_roll, endpoint_roll);
        assert_eq!(dy, sea_level + endpoint_roll - center.y);
        assert_eq!(callback_coordinates, Some((center.x, center.z)));
        assert_eq!(
            random_next_i32_bound(&mut actual_random, 101),
            expected_next,
            "mesa adjustment must consume exactly the inclusive-range draw"
        );
    }
}

#[test]
fn mesa_callback_uses_java_bounding_box_center_for_negative_even_and_odd_spans() {
    let aggregate = StructureBoundingBoxModel {
        min_x: -13,
        min_y: 40,
        min_z: -22,
        max_x: 4,
        max_y: 50,
        max_z: -4,
    };
    let center = aggregate.center();
    assert_eq!((center.x, center.z), (-4, -13));

    let mut random = test_random(17);
    let mut callback_coordinates = None;
    mineshaft_vertical_adjustment_dy(
        &mut random,
        aggregate,
        MineshaftTypeModel::Mesa,
        63,
        -64,
        |x, z| {
            callback_coordinates = Some((x, z));
            63
        },
    );
    assert_eq!(callback_coordinates, Some((-4, -13)));
}

#[test]
fn mesa_surface_at_or_below_sea_level_does_not_draw_randomness() {
    let aggregate = test_box();
    let sea_level = 63;
    let center = aggregate.center();

    for surface_height in [sea_level - 1, sea_level] {
        let mut actual_random = test_random(i64::from(surface_height));
        let mut expected_random = actual_random;
        let dy = mineshaft_vertical_adjustment_dy(
            &mut actual_random,
            aggregate,
            MineshaftTypeModel::Mesa,
            sea_level,
            -64,
            |_, _| surface_height,
        );

        assert_eq!(dy, sea_level - center.y);
        assert_eq!(
            random_next_i32_bound(&mut actual_random, 97),
            random_next_i32_bound(&mut expected_random, 97),
            "surface <= sea level must not advance the random source"
        );
    }
}

#[test]
fn normal_vertical_adjustment_ignores_surface_callback_and_matches_builder_move() {
    let aggregate = test_box();
    let sea_level = 63;
    let min_y = -64;
    let mut actual_random = test_random(0x6e6f_726d);
    let mut expected_random = actual_random;

    let expected_roll = random_next_i32_bound(&mut expected_random, 106);
    let expected_dy =
        mineshaft_move_below_sea_level_dy(aggregate, sea_level, min_y, 10, expected_roll)
            .expect("computed random roll is valid");
    let expected_next = random_next_i32_bound(&mut expected_random, 89);
    let dy = mineshaft_vertical_adjustment_dy(
        &mut actual_random,
        aggregate,
        MineshaftTypeModel::Normal,
        sea_level,
        min_y,
        |_, _| {
            panic!("normal mineshafts do not query terrain height");
        },
    );

    assert_eq!(dy, expected_dy);
    assert_eq!(random_next_i32_bound(&mut actual_random, 89), expected_next);
}

#[test]
fn normal_too_tall_bounds_do_not_draw_randomness() {
    let aggregate = StructureBoundingBoxModel {
        min_x: 0,
        min_y: -64,
        min_z: 0,
        max_x: 12,
        max_y: 64,
        max_z: 12,
    };
    let mut actual_random = test_random(0x7461_6c6c);
    let mut expected_random = actual_random;

    let dy = mineshaft_vertical_adjustment_dy(
        &mut actual_random,
        aggregate,
        MineshaftTypeModel::Normal,
        63,
        -64,
        |_, _| {
            panic!("normal mineshafts do not query terrain height");
        },
    );

    // Java leaves y1Pos untouched when it is already at/above seaLevel - 10.
    let y1_pos = (aggregate.max_y - aggregate.min_y + 1) - 64 + 1;
    assert!(y1_pos >= 63 - 10);
    assert_eq!(dy, y1_pos - aggregate.max_y);
    assert_eq!(
        random_next_i32_bound(&mut actual_random, 97),
        random_next_i32_bound(&mut expected_random, 97),
        "too-tall normal bounds must not advance the random source"
    );
}

#[test]
fn generated_mesa_callback_runs_after_piece_generation_at_aggregate_center() {
    let seed = 8675309;
    let chunk_pos = ChunkPos { x: -1, z: 4 };
    let sea_level = 63;
    let pieces_without_height = mineshaft_generate_pieces_for_start(
        seed,
        chunk_pos,
        MineshaftTypeModel::Mesa,
        sea_level,
        -64,
    );
    let aggregate = pieces_without_height
        .iter()
        .map(MineshaftGeneratedPieceModel::bounding_box)
        .reduce(StructureBoundingBoxModel::union)
        .expect("mineshaft has its start room");
    assert!(
        pieces_without_height.len() > 1,
        "fixture includes child pieces beyond the start room"
    );
    assert_ne!(
        aggregate,
        pieces_without_height[0].bounding_box(),
        "fixture's aggregate bounds include child pieces"
    );
    let center = aggregate.center();
    assert_eq!(center.y, sea_level, "fallback anchors the aggregate center");
    let mut callback_calls = 0;

    let pieces = mineshaft_generate_pieces_for_start_with_surface_height(
        seed,
        chunk_pos,
        MineshaftTypeModel::Mesa,
        sea_level,
        -64,
        |x, z| {
            callback_calls += 1;
            assert_eq!((x, z), (center.x, center.z));
            sea_level
        },
    );

    assert_eq!(callback_calls, 1);
    assert_eq!(pieces, pieces_without_height);
}

#[test]
fn compatibility_entry_point_uses_sea_level_as_mesa_surface_fallback() {
    for (seed, chunk_pos) in [
        (0, ChunkPos { x: 0, z: 0 }),
        (8675309, ChunkPos { x: -1, z: 4 }),
        (-239, ChunkPos { x: 11, z: -7 }),
    ] {
        let legacy =
            mineshaft_generate_pieces_for_start(seed, chunk_pos, MineshaftTypeModel::Mesa, 63, -64);
        let explicit = mineshaft_generate_pieces_for_start_with_surface_height(
            seed,
            chunk_pos,
            MineshaftTypeModel::Mesa,
            63,
            -64,
            |_, _| 63,
        );
        assert_eq!(legacy, explicit, "seed {seed} at {chunk_pos:?}");
    }
}

#[test]
fn vertical_offset_moves_room_child_entrances_with_their_room() {
    // These two type variants consume the same random sequence while building
    // pieces. Their only difference here is the final vertical adjustment.
    // Search a small deterministic fixture set for a generated room with an
    // entrance, then verify every piece and room entrance shares that offset.
    let sea_level = 63;
    let min_y = -64;
    let mut fixture = None;
    for seed in 0_i64..32 {
        let chunk_pos = ChunkPos {
            x: seed as i32 - 12,
            z: 5 - seed as i32,
        };
        let mesa = mineshaft_generate_pieces_for_start_with_surface_height(
            seed,
            chunk_pos,
            MineshaftTypeModel::Mesa,
            sea_level,
            min_y,
            |_, _| sea_level + 20,
        );
        let has_entrances = mesa.iter().any(|piece| {
            matches!(
                piece,
                MineshaftGeneratedPieceModel::Room {
                    child_entrance_boxes,
                    ..
                } if !child_entrance_boxes.is_empty()
            )
        });
        if has_entrances {
            fixture = Some((seed, chunk_pos, mesa));
            break;
        }
    }
    let (seed, chunk_pos, mesa) = fixture.expect("bounded fixtures include room entrances");
    let normal = mineshaft_generate_pieces_for_start(
        seed,
        chunk_pos,
        MineshaftTypeModel::Normal,
        sea_level,
        min_y,
    );
    assert_eq!(normal.len(), mesa.len());

    let vertical_delta = match (&normal[0], &mesa[0]) {
        (
            MineshaftGeneratedPieceModel::Room {
                bounding_box: normal_box,
                ..
            },
            MineshaftGeneratedPieceModel::Room {
                bounding_box: mesa_box,
                ..
            },
        ) => mesa_box.min_y - normal_box.min_y,
        _ => panic!("first generated piece is the start room"),
    };

    for (normal_piece, mesa_piece) in normal.iter().zip(&mesa) {
        let normal_box = normal_piece.bounding_box();
        let mesa_box = mesa_piece.bounding_box();
        assert_eq!(mesa_box, normal_box.moved(0, vertical_delta, 0));

        match (normal_piece, mesa_piece) {
            (
                MineshaftGeneratedPieceModel::Room {
                    child_entrance_boxes: normal_entrances,
                    ..
                },
                MineshaftGeneratedPieceModel::Room {
                    child_entrance_boxes: mesa_entrances,
                    ..
                },
            ) => {
                assert_eq!(normal_entrances.len(), mesa_entrances.len());
                for (normal_entrance, mesa_entrance) in normal_entrances.iter().zip(mesa_entrances)
                {
                    assert_eq!(mesa_entrance.min_x, normal_entrance.min_x);
                    assert_eq!(mesa_entrance.max_x, normal_entrance.max_x);
                    assert_eq!(mesa_entrance.min_z, normal_entrance.min_z);
                    assert_eq!(mesa_entrance.max_z, normal_entrance.max_z);
                    assert_eq!(mesa_entrance.min_y - normal_entrance.min_y, vertical_delta);
                    assert_eq!(mesa_entrance.max_y - normal_entrance.max_y, vertical_delta);
                }
            }
            (MineshaftGeneratedPieceModel::Room { .. }, _) => {
                panic!("piece kind changed between vertical variants")
            }
            (_, MineshaftGeneratedPieceModel::Room { .. }) => {
                panic!("piece kind changed between vertical variants")
            }
            _ => assert_eq!(
                std::mem::discriminant(normal_piece),
                std::mem::discriminant(mesa_piece),
                "piece kind changed between vertical variants"
            ),
        }
    }
}
