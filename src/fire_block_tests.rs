//! Tests for [`crate::fire_block`] against a scripted world and RNG.

use std::collections::HashMap;

use crate::block_behavior::BlockStateModel;
use crate::block_placement::PlacementWorld;
use crate::block_survival::SurvivalWorld;
use crate::block_update::BlockPos;
use crate::fire_block::*;

/// RNG returning a fixed value clamped into each requested bound.
struct FixedRandom {
    int: i32,
    float: f32,
    int_calls: Vec<i32>,
}

impl FixedRandom {
    fn new(int: i32, float: f32) -> Self {
        Self {
            int,
            float,
            int_calls: Vec::new(),
        }
    }
}

impl FireRandom for FixedRandom {
    fn next_int(&mut self, bound: i32) -> i32 {
        self.int_calls.push(bound);
        self.int.min(bound - 1)
    }
    fn next_float(&mut self) -> f32 {
        self.float
    }
}

#[derive(Default)]
struct MockWorld {
    blocks: HashMap<(i32, i32, i32), BlockStateModel>,
    raining: bool,
    difficulty: i32,
    burnout: bool,
    spread_allowed: bool,
    scheduled: Vec<(BlockPos, i32)>,
    primed: Vec<BlockPos>,
    writes: Vec<(BlockPos, String, i32)>,
}

impl MockWorld {
    fn new() -> Self {
        Self {
            spread_allowed: true,
            difficulty: 2,
            ..Self::default()
        }
    }
    fn with(mut self, pos: (i32, i32, i32), id: &str) -> Self {
        self.blocks.insert(pos, BlockStateModel::new(id));
        self
    }
}

const POS: BlockPos = BlockPos { x: 0, y: 65, z: 0 };

impl SurvivalWorld for MockWorld {
    fn state_at(&self, pos: BlockPos) -> BlockStateModel {
        self.blocks
            .get(&(pos.x, pos.y, pos.z))
            .cloned()
            .unwrap_or_else(BlockStateModel::air)
    }
    fn raw_brightness(&self, _pos: BlockPos) -> i32 {
        15
    }
}

impl PlacementWorld for MockWorld {
    fn has_neighbor_signal(&self, _pos: BlockPos) -> bool {
        false
    }
}

impl FireWorld for MockWorld {
    fn can_spread_fire_around(&self, _pos: BlockPos) -> bool {
        self.spread_allowed
    }
    fn is_raining(&self) -> bool {
        self.raining
    }
    fn is_raining_at(&self, _pos: BlockPos) -> bool {
        self.raining
    }
    fn is_infiniburn(&self, state: &BlockStateModel) -> bool {
        matches!(
            state.registry_id.as_str(),
            "minecraft:netherrack" | "minecraft:magma_block"
        )
    }
    fn increased_fire_burnout(&self, _pos: BlockPos) -> bool {
        self.burnout
    }
    fn difficulty_id(&self) -> i32 {
        self.difficulty
    }
    fn is_face_sturdy_up(&self, pos: BlockPos) -> bool {
        matches!(
            self.state_at(pos).registry_id.as_str(),
            "minecraft:stone" | "minecraft:netherrack"
        )
    }
    fn set_block(&mut self, pos: BlockPos, state: BlockStateModel, flags: i32) {
        self.writes.push((pos, state.state_name(), flags));
        self.blocks.insert((pos.x, pos.y, pos.z), state);
    }
    fn remove_block(&mut self, pos: BlockPos) {
        self.writes.push((pos, "minecraft:air".into(), 3));
        self.blocks.remove(&(pos.x, pos.y, pos.z));
    }
    fn prime_tnt(&mut self, pos: BlockPos) {
        self.primed.push(pos);
    }
    fn schedule_fire_tick(&mut self, pos: BlockPos, delay: i32) {
        self.scheduled.push((pos, delay));
    }
}

fn fire(age: i32) -> BlockStateModel {
    BlockStateModel::new("minecraft:fire").with_property("age", age.to_string())
}

#[test]
fn flammability_registry_matches_fire_block_bootstrap() {
    assert_eq!(FLAMMABLE_BLOCK_COUNT_FOR_TESTS, 207);
    for (id, ignite, burn) in [
        ("minecraft:oak_planks", 5, 20),
        ("minecraft:oak_log", 5, 5),
        ("minecraft:mangrove_roots", 5, 20),
        ("minecraft:oak_leaves", 30, 60),
        ("minecraft:bookshelf", 30, 20),
        ("minecraft:tnt", 15, 100),
        ("minecraft:short_grass", 60, 100),
        ("minecraft:hay_block", 60, 20),
        ("minecraft:white_carpet", 60, 20),
        ("minecraft:vine", 15, 100),
        ("minecraft:coal_block", 5, 5),
        ("minecraft:oak_shelf", 30, 20),
    ] {
        assert_eq!(flammable_block_odds(id), Some((ignite, burn)), "{id}");
    }
    // Not registered in bootStrap even though the old heuristic matched them.
    assert_eq!(flammable_block_odds("minecraft:stone"), None);
    assert_eq!(flammable_block_odds("minecraft:oak_sign"), None);
}

#[test]
fn waterlogged_states_have_zero_odds() {
    let slab = BlockStateModel::new("minecraft:oak_slab").with_property("waterlogged", "true");
    assert_eq!(state_ignite_odds(&slab), 0);
    assert_eq!(state_burn_odds(&slab), 0);
    assert!(!can_burn(&slab));
    assert!(can_burn(&BlockStateModel::new("minecraft:oak_slab")));
}

#[test]
fn tick_always_reschedules_with_thirty_plus_random_delay_first() {
    let mut world = MockWorld::new().with((0, 64, 0), "minecraft:stone");
    world.spread_allowed = false;
    let mut random = FixedRandom::new(7, 0.0);
    fire_tick(&mut world, fire(0), POS, &mut random);
    assert_eq!(world.scheduled, vec![(POS, 37)]);
    // canSpreadFireAround == false stops right after the reschedule.
    assert_eq!(random.int_calls, vec![10]);
    assert!(world.writes.is_empty());
}

#[test]
fn unsupported_fire_without_burnable_neighbour_is_removed() {
    let mut world = MockWorld::new();
    let mut random = FixedRandom::new(0, 0.0);
    fire_tick(&mut world, fire(0), POS, &mut random);
    assert!(world.writes.iter().any(|(pos, name, _)| *pos == POS && name == "minecraft:air"));
}

#[test]
fn infiniburn_fire_ages_silently_and_never_burns_out() {
    let mut world = MockWorld::new().with((0, 64, 0), "minecraft:netherrack");
    // nextInt(3)/2 with nextInt=2 -> +1 age.
    let mut random = FixedRandom::new(2, 0.9);
    fire_tick(&mut world, fire(4), POS, &mut random);
    let age_write = world
        .writes
        .iter()
        .find(|(pos, _, _)| *pos == POS)
        .expect("age write");
    assert_eq!(age_write.2, SET_BLOCK_FLAGS_AGE_ONLY);
    assert!(age_write.1.contains("age=5"));
}

#[test]
fn rain_puts_out_fire_on_non_infiniburn_ground() {
    let mut world = MockWorld::new().with((0, 64, 0), "minecraft:stone");
    world.raining = true;
    // nextFloat 0.0 < 0.2 + age*0.03
    let mut random = FixedRandom::new(0, 0.0);
    fire_tick(&mut world, fire(3), POS, &mut random);
    assert!(world.writes.iter().any(|(pos, name, _)| *pos == POS && name == "minecraft:air"));
    assert!(!world.blocks.contains_key(&(0, 65, 0)));
}

#[test]
fn rain_does_not_put_out_infiniburn_fire() {
    let mut world = MockWorld::new().with((0, 64, 0), "minecraft:netherrack");
    world.raining = true;
    let mut random = FixedRandom::new(0, 0.0);
    fire_tick(&mut world, fire(3), POS, &mut random);
    assert!(!world.writes.iter().any(|(_, name, _)| name == "minecraft:air"));
}

#[test]
fn fire_burns_adjacent_flammable_block_into_fire_or_air() {
    // nextInt always 0: burn-out check passes (0 < 5), age roll 0 < 5 and not
    // raining -> the planks become fire with getStateWithAge(0).
    let mut world = MockWorld::new()
        .with((0, 64, 0), "minecraft:stone")
        .with((1, 65, 0), "minecraft:oak_planks");
    let mut random = FixedRandom::new(0, 0.9);
    fire_tick(&mut world, fire(0), POS, &mut random);
    let east = BlockPos { x: 1, y: 65, z: 0 };
    let replaced = world.state_at(east);
    assert_eq!(replaced.registry_id, "minecraft:fire");
    assert_eq!(replaced.property("age"), Some("0"));
}

#[test]
fn burning_tnt_is_primed() {
    let mut world = MockWorld::new()
        .with((0, 64, 0), "minecraft:stone")
        .with((1, 65, 0), "minecraft:tnt");
    struct Script(Vec<i32>);
    impl FireRandom for Script {
        fn next_int(&mut self, _bound: i32) -> i32 {
            self.0.remove(0)
        }
        fn next_float(&mut self) -> f32 {
            0.9
        }
    }
    // delay, age step, east: burn passes (0), age roll fails (9) -> remove;
    // remaining rolls are large so nothing else burns.
    let mut script = Script(vec![0, 0, 0, 9]);
    script.0.extend(std::iter::repeat_n(299, 64));
    fire_tick(&mut world, fire(0), POS, &mut script);
    assert_eq!(world.primed, vec![BlockPos { x: 1, y: 65, z: 0 }]);
    assert_eq!(
        world.state_at(BlockPos { x: 1, y: 65, z: 0 }).registry_id,
        "minecraft:air"
    );
}

#[test]
fn increased_burnout_lowers_chances_and_halves_spread_odds() {
    let mut world = MockWorld::new()
        .with((0, 64, 0), "minecraft:stone")
        .with((0, 66, 0), "minecraft:oak_planks");
    world.burnout = true;
    let mut random = FixedRandom::new(0, 0.9);
    // Only checks it runs the same path without panicking and consumes the
    // reduced 250 / 200 bounds for the burn-out checks.
    fire_tick(&mut world, fire(0), POS, &mut random);
    assert!(random.int_calls.contains(&250));
    assert!(random.int_calls.contains(&200));
}
