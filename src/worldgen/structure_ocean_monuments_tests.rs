use super::*;

#[test]
fn monument_graph_has_vanilla_sparse_cells_and_claimed_core() {
    for seed in [0, 1, -1, 0x1234_5678] {
        let mut random = RandomSourceKind::new(seed, RandomAlgorithm::Legacy);
        let graph = ocean_monument_room_graph(&mut random).unwrap();

        assert_eq!(graph.rooms.len(), 49);
        assert_eq!(graph.rooms[graph.source_room].index, 2);
        assert!(graph.rooms[graph.source_room].is_source);
        assert_eq!(graph.rooms[graph.core_room].index / 5 % 5, 2);
        assert!((0..4).contains(&(graph.rooms[graph.core_room].index % 5)));
        assert!(graph.rooms[graph.core_room].claimed);

        let regular: Vec<_> = graph.rooms.iter().filter(|room| room.index < 75).collect();
        assert_eq!(regular.len(), 46);
        for (room_index, room) in graph.rooms.iter().enumerate().take(46) {
            assert!(monument_room_reaches_source(
                &graph.rooms,
                room_index,
                1,
                &mut vec![0; graph.rooms.len()],
            ));
            assert!(room.index < 75);
        }

        let core_claim_count = regular.iter().filter(|room| room.claimed).count();
        assert_eq!(core_claim_count, 8);
        assert!(graph
            .rooms
            .iter()
            .filter(|room| room.index >= 1000)
            .all(|room| room.claimed));
    }
}

#[test]
fn monument_room_graph_preserves_vanilla_special_room_opening_asymmetry() {
    let mut random = RandomSourceKind::new(19, RandomAlgorithm::Legacy);
    let graph = ocean_monument_room_graph(&mut random).unwrap();
    let by_id = |id| {
        graph
            .rooms
            .iter()
            .position(|room| room.index == id)
            .unwrap_or(usize::MAX)
    };
    let roof = by_id(1003);
    let left_wing = by_id(1001);
    let right_wing = by_id(1002);

    assert!(roof < graph.rooms.len());
    assert!(left_wing < graph.rooms.len());
    assert!(right_wing < graph.rooms.len());
    assert!(graph.rooms[roof].openings[0]);
    assert!(graph.rooms[roof].connections[0].is_some());
    let left_room = graph
        .rooms
        .iter()
        .position(|room| room.index == 25)
        .unwrap();
    let right_room = graph
        .rooms
        .iter()
        .position(|room| room.index == 29)
        .unwrap();
    assert!(graph.rooms[left_room].openings[3]);
    assert!(graph.rooms[right_room].openings[3]);
    assert_eq!(graph.room_order[46..], [roof, left_wing, right_wing]);
    let mut regular_order = graph.room_order[..46].to_vec();
    regular_order.sort_unstable();
    assert_eq!(regular_order, (0..46).collect::<Vec<_>>());
}

#[test]
fn monument_graph_and_random_continuation_are_deterministic() {
    let seed = 0x5eed;
    let mut first_random = RandomSourceKind::new(seed, RandomAlgorithm::Legacy);
    let mut second_random = RandomSourceKind::new(seed, RandomAlgorithm::Legacy);
    let first_graph = ocean_monument_room_graph(&mut first_random).unwrap();
    let second_graph = ocean_monument_room_graph(&mut second_random).unwrap();

    assert_eq!(first_graph, second_graph);
    assert_eq!(
        worldgen_random_next_i32_bound(&mut first_random, 10_000),
        worldgen_random_next_i32_bound(&mut second_random, 10_000),
    );
}

#[test]
fn monument_room_graph_matches_java_seed_zero_fixture() {
    // Captured by invoking generateRoomGraph via reflection in the actual 26.1.2
    // server jar with RandomSource.create(seed), before constructor fitter draws.
    // The continuation values are the immediately following nextInt(10_000).
    assert_java_graph_fixture(
        0,
        12,
        &[
            27, 28, 53, 39, 25, 44, 56, 13, 38, 43, 26, 51, 16, 7, 18, 9, 10, 52, 19, 30, 6, 31,
            35, 11, 29, 58, 41, 0, 34, 15, 8, 36, 42, 12, 33, 17, 57, 2, 3, 1, 14, 40, 5, 32, 4,
            37, 1003, 1001, 1002,
        ],
        &[
            (0, 32),
            (1, 18),
            (2, 32),
            (3, 50),
            (4, 20),
            (5, 32),
            (6, 22),
            (7, 6),
            (8, 2),
            (9, 12),
            (10, 36),
            (11, 24),
            (12, 44),
            (13, 16),
            (14, 14),
            (15, 10),
            (16, 32),
            (17, 58),
            (18, 16),
            (19, 8),
            (25, 12),
            (26, 5),
            (27, 38),
            (28, 17),
            (29, 12),
            (30, 12),
            (31, 11),
            (32, 47),
            (33, 51),
            (34, 24),
            (35, 12),
            (36, 36),
            (37, 24),
            (38, 4),
            (39, 5),
            (40, 41),
            (41, 24),
            (42, 33),
            (43, 24),
            (44, 8),
            (51, 32),
            (52, 19),
            (53, 4),
            (56, 1),
            (57, 1),
            (58, 9),
            (1001, 4),
            (1002, 0),
            (1003, 1),
        ],
        6935,
    );
}

#[test]
fn monument_room_graph_matches_java_seed_one_fixture() {
    assert_java_graph_fixture(
        1,
        12,
        &[
            4, 1, 28, 36, 26, 2, 44, 7, 33, 40, 34, 3, 19, 57, 0, 9, 41, 35, 43, 31, 53, 38, 37,
            11, 10, 14, 25, 5, 16, 56, 8, 51, 58, 32, 12, 30, 15, 18, 27, 6, 17, 39, 13, 52, 42,
            29, 1003, 1001, 1002,
        ],
        &[
            (0, 36),
            (1, 16),
            (2, 34),
            (3, 18),
            (4, 6),
            (5, 10),
            (6, 6),
            (7, 36),
            (8, 16),
            (9, 8),
            (10, 34),
            (11, 24),
            (12, 42),
            (13, 52),
            (14, 16),
            (15, 34),
            (16, 48),
            (17, 50),
            (18, 58),
            (19, 16),
            (25, 12),
            (26, 36),
            (27, 23),
            (28, 33),
            (29, 29),
            (30, 13),
            (31, 15),
            (32, 8),
            (33, 36),
            (34, 24),
            (35, 9),
            (36, 40),
            (37, 17),
            (38, 40),
            (39, 20),
            (40, 1),
            (41, 32),
            (42, 17),
            (43, 1),
            (44, 8),
            (51, 32),
            (52, 23),
            (53, 4),
            (56, 1),
            (57, 40),
            (58, 24),
            (1001, 4),
            (1002, 0),
            (1003, 1),
        ],
        8720,
    );
}

#[test]
fn monument_room_graph_matches_java_seed_42_fixture() {
    assert_java_graph_fixture(
        42,
        12,
        &[
            35, 4, 8, 34, 40, 29, 6, 37, 52, 13, 39, 44, 25, 36, 15, 5, 11, 53, 10, 57, 33, 19, 3,
            41, 1, 42, 28, 9, 7, 0, 38, 27, 2, 14, 26, 51, 58, 17, 56, 43, 31, 30, 12, 16, 18, 32,
            1003, 1001, 1002,
        ],
        &[
            (0, 36),
            (1, 54),
            (2, 16),
            (3, 2),
            (4, 2),
            (5, 8),
            (6, 8),
            (7, 4),
            (8, 2),
            (9, 4),
            (10, 36),
            (11, 48),
            (12, 30),
            (13, 4),
            (14, 10),
            (15, 42),
            (16, 16),
            (17, 42),
            (18, 58),
            (19, 16),
            (25, 12),
            (26, 37),
            (27, 50),
            (28, 51),
            (29, 25),
            (30, 12),
            (31, 42),
            (32, 52),
            (33, 49),
            (34, 16),
            (35, 12),
            (36, 32),
            (37, 25),
            (38, 36),
            (39, 17),
            (40, 9),
            (41, 32),
            (42, 17),
            (43, 41),
            (44, 16),
            (51, 4),
            (52, 7),
            (53, 1),
            (56, 9),
            (57, 40),
            (58, 16),
            (1001, 4),
            (1002, 4),
            (1003, 1),
        ],
        8510,
    );
}

fn assert_java_graph_fixture(
    seed: i64,
    core_id: i32,
    order: &[i32],
    masks: &[(i32, i32)],
    next: i32,
) {
    let mut random = RandomSourceKind::new(seed, RandomAlgorithm::Legacy);
    let graph = ocean_monument_room_graph(&mut random).unwrap();
    let actual_order: Vec<_> = graph
        .room_order
        .iter()
        .map(|&room| graph.rooms[room].index)
        .collect();
    let mut actual_masks: Vec<_> = graph
        .rooms
        .iter()
        .map(|room| {
            let mask = room
                .openings
                .iter()
                .enumerate()
                .fold(0, |mask, (direction, &opening)| {
                    if opening {
                        mask | (1 << direction)
                    } else {
                        mask
                    }
                });
            (room.index, mask)
        })
        .collect();
    actual_masks.sort_unstable_by_key(|&(room_id, _)| room_id);

    assert_eq!(
        graph.rooms[graph.core_room].index, core_id,
        "seed {seed} core"
    );
    assert_eq!(actual_order, order, "seed {seed} room order");
    assert_eq!(actual_masks, masks, "seed {seed} opening masks");
    assert_eq!(
        worldgen_random_next_i32_bound(&mut random, 10_000),
        next,
        "seed {seed} random continuation",
    );
}
