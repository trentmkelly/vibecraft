use super::*;
use std::collections::HashSet;

#[derive(Default)]
struct TestWorld {
    occupied: HashSet<(i32, i32, i32)>,
    emptiness_queries: Vec<BlockPos>,
    writes: Vec<(VineColumnBlock, i32)>,
    reject_writes: bool,
}

impl TestWorld {
    fn occupy(&mut self, pos: BlockPos) {
        self.occupied.insert((pos.x, pos.y, pos.z));
    }
}

impl NetherVineColumnWorld for TestWorld {
    fn is_empty_block(&mut self, pos: BlockPos) -> bool {
        self.emptiness_queries.push(pos);
        !self.occupied.contains(&(pos.x, pos.y, pos.z))
    }

    fn set_block(&mut self, block: VineColumnBlock, flags: i32) {
        if !self.reject_writes {
            self.occupied
                .insert((block.pos.x, block.pos.y, block.pos.z));
        }
        self.writes.push((block, flags));
    }
}

fn pos(y: i32) -> BlockPos {
    BlockPos { x: 4, y, z: -8 }
}

#[test]
fn twisting_last_step_places_head_without_querying_above_and_preserves_cursor() {
    let mut world = TestWorld::default();
    world.occupy(pos(11));
    let mut cursor = pos(10);
    let mut random = RandomSourceKind::new(17, RandomAlgorithm::Legacy);

    place_twisting_vines_column(&mut world, &mut random, &mut cursor, 1, 23, 25);

    assert_eq!(world.emptiness_queries, vec![pos(10)]);
    assert_eq!(cursor, pos(10));
    assert_eq!(world.writes.len(), 1);
    assert_eq!(world.writes[0].0.pos, pos(10));
    assert_eq!(world.writes[0].0.state, "minecraft:twisting_vines");
    assert_eq!(world.writes[0].0.kind, VineColumnBlockKind::Head);
    assert!((23..=25).contains(&world.writes[0].0.age.unwrap_or_default()));
    assert_eq!(world.writes[0].1, 2);
}

#[test]
fn adjacent_obstruction_turns_the_current_empty_step_into_a_head() {
    let mut twisting_world = TestWorld::default();
    twisting_world.occupy(pos(11));
    let mut twisting_cursor = pos(10);
    let mut twisting_random = RandomSourceKind::new(170, RandomAlgorithm::Legacy);

    place_twisting_vines_column(
        &mut twisting_world,
        &mut twisting_random,
        &mut twisting_cursor,
        3,
        17,
        25,
    );

    assert_eq!(twisting_world.emptiness_queries, vec![pos(10), pos(11)]);
    assert_eq!(twisting_cursor, pos(10));
    assert_eq!(twisting_world.writes.len(), 1);
    assert_eq!(twisting_world.writes[0].0.kind, VineColumnBlockKind::Head);
    assert_eq!(twisting_world.writes[0].0.pos, pos(10));

    let mut weeping_world = TestWorld::default();
    weeping_world.occupy(pos(19));
    let mut weeping_cursor = pos(20);
    let mut weeping_random = RandomSourceKind::new(171, RandomAlgorithm::Xoroshiro);

    place_weeping_vines_column(
        &mut weeping_world,
        &mut weeping_random,
        &mut weeping_cursor,
        2,
        23,
        25,
    );

    assert_eq!(weeping_world.emptiness_queries, vec![pos(20), pos(19)]);
    assert_eq!(weeping_cursor, pos(20));
    assert_eq!(weeping_world.writes.len(), 1);
    assert_eq!(weeping_world.writes[0].0.kind, VineColumnBlockKind::Head);
    assert_eq!(weeping_world.writes[0].0.pos, pos(20));
}

#[test]
fn occupied_steps_advance_mutable_cursors_in_both_directions() {
    let mut twisting_world = TestWorld::default();
    for y in 10..13 {
        twisting_world.occupy(pos(y));
    }
    let mut twisting_cursor = pos(10);
    let mut twisting_random = RandomSourceKind::new(1, RandomAlgorithm::Legacy);
    place_twisting_vines_column(
        &mut twisting_world,
        &mut twisting_random,
        &mut twisting_cursor,
        3,
        17,
        25,
    );
    assert_eq!(twisting_cursor, pos(13));
    assert!(twisting_world.writes.is_empty());
    assert_eq!(
        twisting_world.emptiness_queries,
        vec![pos(10), pos(11), pos(12)]
    );

    let mut weeping_world = TestWorld::default();
    for y in 20..23 {
        weeping_world.occupy(pos(y));
    }
    let mut weeping_cursor = pos(22);
    let mut weeping_random = RandomSourceKind::new(2, RandomAlgorithm::Xoroshiro);
    place_weeping_vines_column(
        &mut weeping_world,
        &mut weeping_random,
        &mut weeping_cursor,
        2,
        17,
        25,
    );
    assert_eq!(weeping_cursor, pos(19));
    assert!(weeping_world.writes.is_empty());
    assert_eq!(
        weeping_world.emptiness_queries,
        vec![pos(22), pos(21), pos(20)]
    );
}

#[test]
fn twisting_column_writes_body_then_head_and_leaves_cursor_at_head() {
    for algorithm in [RandomAlgorithm::Legacy, RandomAlgorithm::Xoroshiro] {
        let mut twisting_world = TestWorld::default();
        let mut twisting_cursor = pos(0);
        let mut twisting_random = RandomSourceKind::new(0x1357, algorithm);
        let mut expected_twisting_random = twisting_random;
        let expected_twisting_age =
            17 + worldgen_random_next_i32_bound(&mut expected_twisting_random, 9);
        let expected_twisting_followup =
            worldgen_random_next_i32_bound(&mut expected_twisting_random, 31);

        place_twisting_vines_column(
            &mut twisting_world,
            &mut twisting_random,
            &mut twisting_cursor,
            3,
            17,
            25,
        );

        assert_eq!(twisting_cursor, pos(2));
        assert_eq!(
            twisting_world.emptiness_queries,
            vec![pos(0), pos(1), pos(1), pos(2), pos(2)]
        );
        assert_eq!(twisting_world.writes.len(), 3);
        assert_eq!(twisting_world.writes[0].0.pos, pos(0));
        assert_eq!(twisting_world.writes[0].0.kind, VineColumnBlockKind::Plant);
        assert_eq!(twisting_world.writes[1].0.pos, pos(1));
        assert_eq!(twisting_world.writes[1].0.kind, VineColumnBlockKind::Plant);
        assert_eq!(twisting_world.writes[2].0.pos, pos(2));
        assert_eq!(twisting_world.writes[2].0.kind, VineColumnBlockKind::Head);
        assert_eq!(twisting_world.writes[2].0.age, Some(expected_twisting_age));
        assert!(twisting_world.writes.iter().all(|(_, flags)| *flags == 2));
        assert!(twisting_world.occupied.contains(&(4, 0, -8)));
        assert!(twisting_world.occupied.contains(&(4, 1, -8)));
        assert!(twisting_world.occupied.contains(&(4, 2, -8)));
        assert_eq!(
            worldgen_random_next_i32_bound(&mut twisting_random, 31),
            expected_twisting_followup
        );
    }
}

#[test]
fn weeping_column_writes_body_then_head_and_leaves_cursor_at_head() {
    for algorithm in [RandomAlgorithm::Legacy, RandomAlgorithm::Xoroshiro] {
        let mut weeping_world = TestWorld::default();
        let mut weeping_cursor = pos(3);
        let mut weeping_random = RandomSourceKind::new(0x2468, algorithm);
        let mut expected_weeping_random = weeping_random;
        let expected_weeping_age =
            23 + worldgen_random_next_i32_bound(&mut expected_weeping_random, 3);
        let expected_weeping_followup =
            worldgen_random_next_i32_bound(&mut expected_weeping_random, 31);

        place_weeping_vines_column(
            &mut weeping_world,
            &mut weeping_random,
            &mut weeping_cursor,
            3,
            23,
            25,
        );

        assert_eq!(weeping_cursor, pos(0));
        assert_eq!(
            weeping_world.emptiness_queries,
            vec![pos(3), pos(2), pos(2), pos(1), pos(1), pos(0), pos(0)]
        );
        assert_eq!(weeping_world.writes.len(), 4);
        for ((block, _flags), expected_pos) in
            weeping_world.writes[..3]
                .iter()
                .zip([pos(3), pos(2), pos(1)])
        {
            assert_eq!(block.pos, expected_pos);
            assert_eq!(block.kind, VineColumnBlockKind::Plant);
            assert!(weeping_world.occupied.contains(&(
                expected_pos.x,
                expected_pos.y,
                expected_pos.z
            )));
        }
        assert_eq!(weeping_world.writes[3].0.pos, pos(0));
        assert_eq!(weeping_world.writes[3].0.kind, VineColumnBlockKind::Head);
        assert_eq!(weeping_world.writes[3].0.age, Some(expected_weeping_age));
        assert!(weeping_world.writes.iter().all(|(_, flags)| *flags == 2));
        assert!(weeping_world.occupied.contains(&(4, 0, -8)));
        assert_eq!(
            worldgen_random_next_i32_bound(&mut weeping_random, 31),
            expected_weeping_followup
        );
    }
}

#[test]
fn weeping_zero_height_places_head_without_querying_below() {
    let mut world = TestWorld::default();
    world.occupy(pos(29));
    let mut cursor = pos(30);
    let mut random = RandomSourceKind::new(3, RandomAlgorithm::Xoroshiro);

    place_weeping_vines_column(&mut world, &mut random, &mut cursor, 0, 17, 25);

    assert_eq!(world.emptiness_queries, vec![pos(30)]);
    assert_eq!(cursor, pos(30));
    assert_eq!(world.writes.len(), 1);
    assert_eq!(world.writes[0].0.kind, VineColumnBlockKind::Head);
    assert_eq!(world.writes[0].0.state, "minecraft:weeping_vines");
    assert_eq!(world.writes[0].1, 2);
}

#[test]
fn negative_heights_do_not_query_write_or_move() {
    for algorithm in [RandomAlgorithm::Legacy, RandomAlgorithm::Xoroshiro] {
        let mut twisting_world = TestWorld::default();
        let mut twisting_cursor = pos(7);
        let mut twisting_random = RandomSourceKind::new(8, algorithm);
        place_twisting_vines_column(
            &mut twisting_world,
            &mut twisting_random,
            &mut twisting_cursor,
            0,
            17,
            25,
        );
        assert_eq!(twisting_cursor, pos(7));
        assert!(twisting_world.emptiness_queries.is_empty());
        assert!(twisting_world.writes.is_empty());

        let mut weeping_world = TestWorld::default();
        let mut weeping_cursor = pos(7);
        let mut weeping_random = RandomSourceKind::new(8, algorithm);
        place_weeping_vines_column(
            &mut weeping_world,
            &mut weeping_random,
            &mut weeping_cursor,
            -1,
            17,
            25,
        );
        assert_eq!(weeping_cursor, pos(7));
        assert!(weeping_world.emptiness_queries.is_empty());
        assert!(weeping_world.writes.is_empty());
    }
}

#[test]
fn head_age_ranges_match_java_and_only_heads_advance_rng_for_both_algorithms() {
    for algorithm in [RandomAlgorithm::Legacy, RandomAlgorithm::Xoroshiro] {
        for (min_age, max_age) in [(17, 25), (23, 25)] {
            let mut world = TestWorld::default();
            let mut cursor = pos(40);
            let mut random = RandomSourceKind::new(0x5eed, algorithm);
            let mut expected_random = random;
            let expected_age = min_age
                + worldgen_random_next_i32_bound(&mut expected_random, max_age - min_age + 1);

            place_weeping_vines_column(&mut world, &mut random, &mut cursor, 0, min_age, max_age);

            assert_eq!(world.writes[0].0.age, Some(expected_age));
            assert_eq!(random, expected_random);
        }

        let mut world = TestWorld::default();
        let mut cursor = pos(50);
        let mut random = RandomSourceKind::new(0xabc, algorithm);
        let expected_random = random;
        place_twisting_vines_column(&mut world, &mut random, &mut cursor, 0, 17, 25);
        assert_eq!(random, expected_random);
        assert!(world.writes.is_empty());

        let mut occupied_world = TestWorld::default();
        occupied_world.occupy(pos(60));
        let mut occupied_cursor = pos(60);
        let mut occupied_random = RandomSourceKind::new(0xdef, algorithm);
        let expected_random = occupied_random;
        place_weeping_vines_column(
            &mut occupied_world,
            &mut occupied_random,
            &mut occupied_cursor,
            0,
            23,
            25,
        );
        assert_eq!(occupied_random, expected_random);
        assert!(occupied_world.writes.is_empty());
    }
}

#[test]
fn age_range_with_equal_bounds_does_not_consume_randomness() {
    for algorithm in [RandomAlgorithm::Legacy, RandomAlgorithm::Xoroshiro] {
        for (min_age, max_age) in [(24, 24), (25, 23)] {
            let mut world = TestWorld::default();
            let mut cursor = pos(0);
            let mut random = RandomSourceKind::new(1234, algorithm);
            let expected_random = random;

            place_twisting_vines_column(&mut world, &mut random, &mut cursor, 1, min_age, max_age);

            assert_eq!(world.writes[0].0.age, Some(min_age));
            assert_eq!(random, expected_random);
        }
    }
}

#[test]
fn rejected_head_write_still_breaks_with_cursor_at_head_and_consumes_age_draw() {
    for algorithm in [RandomAlgorithm::Legacy, RandomAlgorithm::Xoroshiro] {
        let mut world = TestWorld {
            reject_writes: true,
            ..TestWorld::default()
        };
        let mut cursor = pos(70);
        let mut random = RandomSourceKind::new(789, algorithm);
        let mut expected_random = random;
        let expected_age = 23 + worldgen_random_next_i32_bound(&mut expected_random, 3);

        place_weeping_vines_column(&mut world, &mut random, &mut cursor, 0, 23, 25);

        assert_eq!(world.writes.len(), 1);
        assert_eq!(world.writes[0].0.age, Some(expected_age));
        assert_eq!(world.writes[0].1, 2);
        assert_eq!(cursor, pos(70));
        assert_eq!(random, expected_random);
        assert!(!world.occupied.contains(&(4, 70, -8)));
    }
}
