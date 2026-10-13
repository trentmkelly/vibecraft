use super::*;
use std::collections::{HashMap, HashSet};

type PositionKey = (i32, i32, i32);

fn key(pos: BlockPos) -> PositionKey {
    (pos.x, pos.y, pos.z)
}

#[derive(Debug)]
struct KelpTestWorld {
    ocean_floor_y: i32,
    initial_water: HashSet<PositionKey>,
    sturdy: HashSet<PositionKey>,
    blocks: HashMap<PositionKey, &'static str>,
    survival_overrides: HashMap<(PositionKey, &'static str), bool>,
    water_queries: Vec<PositionKey>,
    head_queries: Vec<(PositionKey, &'static str)>,
    survival_queries: Vec<(PositionKey, &'static str, Option<&'static str>)>,
    writes: Vec<(AquaticPlacementBlock, i32)>,
    reject_writes: HashSet<PositionKey>,
}

impl KelpTestWorld {
    fn new(ocean_floor_y: i32) -> Self {
        Self {
            ocean_floor_y,
            initial_water: HashSet::new(),
            sturdy: HashSet::new(),
            blocks: HashMap::new(),
            survival_overrides: HashMap::new(),
            water_queries: Vec::new(),
            head_queries: Vec::new(),
            survival_queries: Vec::new(),
            writes: Vec::new(),
            reject_writes: HashSet::new(),
        }
    }

    fn add_water_column(&mut self, x: i32, z: i32, bottom: i32, top: i32) {
        for y in bottom..=top {
            self.initial_water.insert((x, y, z));
        }
    }

    fn block_at(&self, pos: BlockPos) -> Option<&'static str> {
        self.blocks.get(&key(pos)).copied()
    }

    fn override_survival(&mut self, pos: BlockPos, state: &'static str, survives: bool) {
        self.survival_overrides.insert((key(pos), state), survives);
    }

    fn supports_growing_plant(&self, pos: BlockPos) -> bool {
        let below = BlockPos {
            y: pos.y - 1,
            ..pos
        };
        self.block_at(below).is_some_and(|state| {
            state == "minecraft:kelp_plant" || state.starts_with("minecraft:kelp[age=")
        }) || self.sturdy.contains(&key(below))
    }
}

impl KelpFeatureWorld for KelpTestWorld {
    fn ocean_floor_height(&mut self, _x: i32, _z: i32) -> i32 {
        self.ocean_floor_y
    }

    fn is_water(&mut self, pos: BlockPos) -> bool {
        self.water_queries.push(key(pos));
        self.block_at(pos).is_none() && self.initial_water.contains(&key(pos))
    }

    fn is_kelp_head(&mut self, pos: BlockPos) -> bool {
        let state = self.block_at(pos);
        self.head_queries
            .push((key(pos), state.unwrap_or("<empty>")));
        state.is_some_and(|state| state.starts_with("minecraft:kelp[age="))
    }

    fn can_survive(&mut self, pos: BlockPos, state: &'static str) -> bool {
        let below = BlockPos {
            y: pos.y - 1,
            ..pos
        };
        let observed_support = self.block_at(below);
        self.survival_queries
            .push((key(pos), state, observed_support));
        self.survival_overrides
            .get(&(key(pos), state))
            .copied()
            .unwrap_or_else(|| self.supports_growing_plant(pos))
    }

    fn set_block(&mut self, block: AquaticPlacementBlock, flags: i32) {
        self.writes.push((block, flags));
        if !self.reject_writes.contains(&key(block.pos)) {
            self.blocks.insert(key(block.pos), block.state);
        }
    }
}

fn random(seed: i64, algorithm: RandomAlgorithm) -> RandomSourceKind {
    RandomSourceKind::new(seed, algorithm)
}

fn pos(x: i32, y: i32, z: i32) -> BlockPos {
    BlockPos { x, y, z }
}

fn sampled_height(seed: i64, algorithm: RandomAlgorithm) -> i32 {
    worldgen_random_next_i32_bound(&mut random(seed, algorithm), 10) + 1
}

fn seed_with_minimum_height(algorithm: RandomAlgorithm, minimum: i32) -> i64 {
    (0..10_000)
        .find(|seed| sampled_height(*seed, algorithm) >= minimum)
        .expect("a bounded search should find a seed for each supported height")
}

fn assert_next_roll_matches(actual: &mut RandomSourceKind, expected: &mut RandomSourceKind) {
    assert_eq!(
        worldgen_random_next_i32_bound(actual, 10_000),
        worldgen_random_next_i32_bound(expected, 10_000),
        "feature consumed a different number of random draws"
    );
}

#[test]
fn kelp_replaces_origin_y_with_ocean_floor_and_initial_water_failure_consumes_no_randomness() {
    for algorithm in [RandomAlgorithm::Legacy, RandomAlgorithm::Xoroshiro] {
        let origin = pos(13, -40, -7);
        let seed = 0x5eed;
        let height = sampled_height(seed, algorithm);
        let mut world = KelpTestWorld::new(61);
        world.add_water_column(origin.x, origin.z, 61, 61 + height + 1);
        world.sturdy.insert((origin.x, 60, origin.z));
        let mut actual = random(seed, algorithm);
        let mut expected = actual;
        worldgen_random_next_i32_bound(&mut expected, 10);
        worldgen_random_next_i32_bound(&mut expected, 4);
        let result = place_kelp_feature(&mut world, &mut actual, origin);

        assert!(result);
        assert!(world.water_queries.contains(&(origin.x, 61, origin.z)));
        assert!(!world
            .water_queries
            .contains(&(origin.x, origin.y, origin.z)));
        assert_eq!(world.writes[0].0.pos, pos(origin.x, 61, origin.z));
        assert_next_roll_matches(&mut actual, &mut expected);

        let mut dry = KelpTestWorld::new(61);
        let mut dry_random = random(seed, algorithm);
        let mut untouched = dry_random;
        assert!(!place_kelp_feature(&mut dry, &mut dry_random, origin));
        assert!(dry
            .water_queries
            .iter()
            .all(|query| *query == (origin.x, 61, origin.z)));
        assert!(dry.writes.is_empty());
        assert_next_roll_matches(&mut dry_random, &mut untouched);
    }
}

#[test]
fn kelp_full_column_reads_prior_body_writes_and_places_an_aged_head() {
    for algorithm in [RandomAlgorithm::Legacy, RandomAlgorithm::Xoroshiro] {
        let origin = pos(-19, 0, 23);
        let seed = 42;
        let height = sampled_height(seed, algorithm);
        let mut world = KelpTestWorld::new(8);
        world.add_water_column(origin.x, origin.z, 8, 8 + height + 1);
        world.sturdy.insert((origin.x, 7, origin.z));
        let mut actual = random(seed, algorithm);
        let mut expected = actual;
        let expected_height = worldgen_random_next_i32_bound(&mut expected, 10) + 1;
        let age = worldgen_random_next_i32_bound(&mut expected, 4) + 20;

        assert!(place_kelp_feature(&mut world, &mut actual, origin));

        assert_eq!(height, expected_height);
        assert_eq!(world.writes.len(), height as usize + 1);
        assert!(world.writes[..height as usize]
            .iter()
            .all(|(write, flags)| write.state == "minecraft:kelp_plant" && *flags == 2));
        assert_eq!(
            world.writes.last().copied(),
            Some((
                AquaticPlacementBlock {
                    pos: pos(origin.x, 8 + height, origin.z),
                    state: kelp_state_for_age(age),
                },
                2,
            ))
        );
        for h in 1..height {
            assert!(world.survival_queries.contains(&(
                (origin.x, 8 + h, origin.z),
                "minecraft:kelp_plant",
                Some("minecraft:kelp_plant")
            )));
        }
        assert_next_roll_matches(&mut actual, &mut expected);
    }
}

#[test]
fn kelp_fallback_checks_support_below_below_after_body_writes() {
    for algorithm in [RandomAlgorithm::Legacy, RandomAlgorithm::Xoroshiro] {
        let origin = pos(4, 0, -2);
        let seed = seed_with_minimum_height(algorithm, 2);
        let height = sampled_height(seed, algorithm);
        assert!(height >= 2);
        let mut world = KelpTestWorld::new(30);
        world.add_water_column(origin.x, origin.z, 30, 30 + height + 1);
        world.sturdy.insert((origin.x, 29, origin.z));
        // The plant check fails at h=2, after bodies at h=0 and h=1 were written.
        world.override_survival(pos(origin.x, 32, origin.z), "minecraft:kelp_plant", false);
        let mut actual = random(seed, algorithm);
        let mut expected = actual;
        worldgen_random_next_i32_bound(&mut expected, 10);
        let age = worldgen_random_next_i32_bound(&mut expected, 4) + 20;

        assert!(place_kelp_feature(&mut world, &mut actual, origin));

        assert_eq!(world.writes.len(), 3);
        assert!(world.writes[..2]
            .iter()
            .all(|(write, _)| write.state == "minecraft:kelp_plant"));
        assert_eq!(world.writes[2].0.pos, pos(origin.x, 31, origin.z));
        assert_eq!(world.writes[2].0.state, kelp_state_for_age(age));
        assert!(world
            .head_queries
            .contains(&((origin.x, 30, origin.z), "minecraft:kelp_plant")));
        assert_next_roll_matches(&mut actual, &mut expected);
    }
}

#[test]
fn kelp_failure_at_origin_advances_and_later_plant_failure_can_fallback() {
    for algorithm in [RandomAlgorithm::Legacy, RandomAlgorithm::Xoroshiro] {
        let origin = pos(2, 100, 3);
        let seed = seed_with_minimum_height(algorithm, 2);
        let height = sampled_height(seed, algorithm);
        assert!(height >= 1);
        let mut world = KelpTestWorld::new(40);
        world.add_water_column(origin.x, origin.z, 40, 40 + height + 1);
        world.override_survival(pos(origin.x, 40, origin.z), "minecraft:kelp_plant", false);
        // The failed h=0 iteration advances. h=1 succeeds as a body, then h=2
        // fails its plant check and takes the surviving-head fallback.
        world.override_survival(pos(origin.x, 41, origin.z), "minecraft:kelp_plant", true);
        world.override_survival(pos(origin.x, 42, origin.z), "minecraft:kelp_plant", false);
        world.override_survival(pos(origin.x, 41, origin.z), "minecraft:kelp", true);
        let mut actual = random(seed, algorithm);
        let mut expected = actual;
        worldgen_random_next_i32_bound(&mut expected, 10);
        let age = worldgen_random_next_i32_bound(&mut expected, 4) + 20;

        assert!(place_kelp_feature(&mut world, &mut actual, origin));

        assert_eq!(world.writes.len(), 2);
        assert_eq!(world.writes[0].0.pos, pos(origin.x, 41, origin.z));
        assert_eq!(world.writes[0].0.state, "minecraft:kelp_plant");
        assert_eq!(world.writes[1].0.pos, pos(origin.x, 41, origin.z));
        assert_eq!(world.writes[1].0.state, kelp_state_for_age(age));
        assert!(world.survival_queries.contains(&(
            (origin.x, 40, origin.z),
            "minecraft:kelp_plant",
            None
        )));
        assert_next_roll_matches(&mut actual, &mut expected);
    }
}

#[test]
fn kelp_failed_origin_without_a_later_head_is_a_failure_and_only_consumes_height() {
    for algorithm in [RandomAlgorithm::Legacy, RandomAlgorithm::Xoroshiro] {
        let origin = pos(-3, 15, 6);
        let seed = -777;
        let height = sampled_height(seed, algorithm);
        let mut world = KelpTestWorld::new(50);
        world.add_water_column(origin.x, origin.z, 50, 50 + height + 1);
        world.override_survival(pos(origin.x, 50, origin.z), "minecraft:kelp_plant", false);
        for h in 1..=height {
            world.override_survival(
                pos(origin.x, 50 + h, origin.z),
                "minecraft:kelp_plant",
                false,
            );
        }
        let mut actual = random(seed, algorithm);
        let mut expected = actual;
        worldgen_random_next_i32_bound(&mut expected, 10);

        assert!(!place_kelp_feature(&mut world, &mut actual, origin));
        assert!(world.writes.is_empty());
        assert_next_roll_matches(&mut actual, &mut expected);
    }
}

#[test]
fn kelp_head_attempt_counts_as_success_even_if_write_fails() {
    for algorithm in [RandomAlgorithm::Legacy, RandomAlgorithm::Xoroshiro] {
        let origin = pos(1, 70, 1);
        let seed = 991;
        let height = sampled_height(seed, algorithm);
        let mut world = KelpTestWorld::new(12);
        world.add_water_column(origin.x, origin.z, 12, 12 + height + 1);
        world.sturdy.insert((origin.x, 11, origin.z));
        world
            .reject_writes
            .insert((origin.x, 12 + height, origin.z));
        let mut actual = random(seed, algorithm);
        let mut expected = actual;
        worldgen_random_next_i32_bound(&mut expected, 10);
        worldgen_random_next_i32_bound(&mut expected, 4);

        assert!(place_kelp_feature(&mut world, &mut actual, origin));
        assert_eq!(
            world.writes.last().unwrap().0.pos,
            pos(origin.x, 12 + height, origin.z)
        );
        assert!(world
            .block_at(pos(origin.x, 12 + height, origin.z))
            .is_none());
        assert_next_roll_matches(&mut actual, &mut expected);
    }
}

#[test]
fn kelp_does_not_attempt_a_fallback_head_when_below_cannot_survive() {
    for algorithm in [RandomAlgorithm::Legacy, RandomAlgorithm::Xoroshiro] {
        let origin = pos(0, 0, 0);
        let seed = 321;
        let height = sampled_height(seed, algorithm);
        let mut world = KelpTestWorld::new(20);
        world.add_water_column(origin.x, origin.z, 20, 20 + height + 1);
        world.sturdy.insert((origin.x, 19, origin.z));
        world.override_survival(pos(origin.x, 21, origin.z), "minecraft:kelp_plant", false);
        world.override_survival(pos(origin.x, 20, origin.z), "minecraft:kelp", false);
        let mut actual = random(seed, algorithm);
        let mut expected = actual;
        worldgen_random_next_i32_bound(&mut expected, 10);

        assert!(!place_kelp_feature(&mut world, &mut actual, origin));
        assert_eq!(world.writes.len(), 1);
        assert_eq!(world.writes[0].0.state, "minecraft:kelp_plant");
        assert!(world
            .writes
            .iter()
            .all(|(write, _)| !write.state.starts_with("minecraft:kelp[age=")));
        assert_next_roll_matches(&mut actual, &mut expected);
    }
}

#[test]
fn kelp_fallback_head_refuses_ahead_of_existing_head() {
    for algorithm in [RandomAlgorithm::Legacy, RandomAlgorithm::Xoroshiro] {
        let origin = pos(7, 5, -8);
        let seed = seed_with_minimum_height(algorithm, 1);
        let height = sampled_height(seed, algorithm);
        assert!(height >= 1);
        let mut world = KelpTestWorld::new(25);
        world.add_water_column(origin.x, origin.z, 25, 25 + height + 1);
        world.override_survival(pos(origin.x, 25, origin.z), "minecraft:kelp_plant", true);
        world.override_survival(pos(origin.x, 26, origin.z), "minecraft:kelp_plant", false);
        world
            .blocks
            .insert((origin.x, 24, origin.z), "minecraft:kelp[age=22]");
        let mut actual = random(seed, algorithm);
        let mut expected = actual;
        worldgen_random_next_i32_bound(&mut expected, 10);

        assert!(!place_kelp_feature(&mut world, &mut actual, origin));
        assert_eq!(world.writes.len(), 1);
        assert_eq!(world.writes[0].0.state, "minecraft:kelp_plant");
        assert!(world
            .head_queries
            .contains(&((origin.x, 24, origin.z), "minecraft:kelp[age=22]")));
        assert_next_roll_matches(&mut actual, &mut expected);
    }
}
