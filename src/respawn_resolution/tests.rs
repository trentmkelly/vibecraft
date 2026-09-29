use std::collections::HashMap;

use super::*;

/// In-memory world: unlisted positions are air; `floor_y` and below are stone.
struct TestWorld {
    blocks: HashMap<(i32, i32, i32), BlockStateModel>,
    floor_y: i32,
    anchor_works: bool,
}

impl TestWorld {
    fn flat(floor_y: i32) -> Self {
        Self { blocks: HashMap::new(), floor_y, anchor_works: false }
    }

    fn put(&mut self, x: i32, y: i32, z: i32, state: BlockStateModel) {
        self.blocks.insert((x, y, z), state);
    }
}

impl RespawnWorld for TestWorld {
    fn block_state(&self, pos: BlockPos) -> BlockStateModel {
        if let Some(state) = self.blocks.get(&(pos.x, pos.y, pos.z)) {
            return state.clone();
        }
        if pos.y <= self.floor_y {
            BlockStateModel::new("minecraft:stone")
        } else {
            BlockStateModel::air()
        }
    }

    fn respawn_anchor_works(&self, _pos: BlockPos) -> bool {
        self.anchor_works
    }

    fn bed_can_set_spawn(&self, _pos: BlockPos) -> bool {
        true
    }

    fn set_block_state(&mut self, pos: BlockPos, state: BlockStateModel) {
        self.blocks.insert((pos.x, pos.y, pos.z), state);
    }
}

fn bed(facing: &str, part: &str) -> BlockStateModel {
    BlockStateModel::new("minecraft:red_bed")
        .with_property("facing", facing)
        .with_property("occupied", "false")
        .with_property("part", part)
}

fn config(x: i32, y: i32, z: i32, forced: bool) -> RespawnConfig {
    RespawnConfig { pos: BlockPos { x, y, z }, yaw: 0.0, pitch: 0.0, forced }
}

#[test]
fn bed_respawn_stands_next_to_the_bed_looking_at_it() {
    let mut world = TestWorld::flat(63);
    world.put(0, 64, 0, bed("south", "foot"));
    world.put(0, 64, 1, bed("south", "head"));
    let found = find_respawn_and_use_spawn_block(&mut world, config(0, 64, 0, false), true)
        .expect("bed with free surroundings resolves");
    // The bed block itself has a 9/16 tall collision; standing spots start beside it.
    let (x, y, z) = found.position;
    assert_eq!(y, 64.0, "on the ground next to the bed");
    assert!((x - 0.5).abs() > 0.9 || (z - 0.5).abs() > 0.9, "not on top of the bed: {x},{z}");
    // Yaw points from the stand-up position at the bed (Mth.atan2 - 90).
    let expected = wrap_degrees((0.5 - z).atan2(0.5 - x).to_degrees() - 90.0) as f32;
    assert!((found.yaw - expected).abs() < 1e-3, "{} vs {expected}", found.yaw);
    assert_eq!(found.pitch, 0.0);
}

#[test]
fn bed_with_all_neighbours_blocked_reports_missing() {
    let mut world = TestWorld::flat(63);
    world.put(0, 64, 0, bed("south", "foot"));
    world.put(0, 64, 1, bed("south", "head"));
    // Wall of stone around and above so no dismount position is free.
    for x in -3..=3 {
        for z in -3..=4 {
            for y in 64..=67 {
                if !(x == 0 && (z == 0 || z == 1) && y == 64) {
                    world.put(x, y, z, BlockStateModel::new("minecraft:stone"));
                }
            }
        }
    }
    assert_eq!(find_respawn_and_use_spawn_block(&mut world, config(0, 64, 0, false), true), None);
}

#[test]
fn missing_spawn_block_is_reported_unless_forced() {
    let mut world = TestWorld::flat(63);
    assert_eq!(find_respawn_and_use_spawn_block(&mut world, config(5, 64, 5, false), true), None);
    let forced = find_respawn_and_use_spawn_block(&mut world, config(5, 64, 5, true), true).unwrap();
    // Forced: the position itself when feet and head blocks are passable.
    assert_eq!(forced.position, (5.5, 64.1, 5.5));
}

#[test]
fn forced_spawn_inside_solid_blocks_is_rejected() {
    let mut world = TestWorld::flat(63);
    world.put(5, 65, 5, BlockStateModel::new("minecraft:stone"));
    assert_eq!(find_respawn_and_use_spawn_block(&mut world, config(5, 64, 5, true), true), None);
}

#[test]
fn respawn_anchor_is_ignored_where_it_does_not_work() {
    let mut world = TestWorld::flat(63);
    world.put(
        0,
        64,
        0,
        BlockStateModel::new("minecraft:respawn_anchor").with_property("charge", "2"),
    );
    // Overworld: RESPAWN_ANCHOR_WORKS is false, so an unforced anchor is treated as missing.
    assert_eq!(find_respawn_and_use_spawn_block(&mut world, config(0, 64, 0, false), true), None);
}

#[test]
fn working_respawn_anchor_spends_one_charge_only_when_not_forced() {
    let anchor = BlockStateModel::new("minecraft:respawn_anchor").with_property("charge", "2");
    let mut world = TestWorld::flat(63);
    world.anchor_works = true;
    world.put(0, 64, 0, anchor.clone());
    let found = find_respawn_and_use_spawn_block(&mut world, config(0, 64, 0, false), true);
    assert!(found.is_some());
    assert_eq!(world.block_state(BlockPos { x: 0, y: 64, z: 0 }).property("charge"), Some("1"));
    // consumeSpawnBlock = false (keepAllPlayerData) leaves the anchor alone.
    world.put(0, 64, 0, anchor.clone());
    find_respawn_and_use_spawn_block(&mut world, config(0, 64, 0, false), false).unwrap();
    assert_eq!(world.block_state(BlockPos { x: 0, y: 64, z: 0 }).property("charge"), Some("2"));
    // A forced anchor is never spent.
    find_respawn_and_use_spawn_block(&mut world, config(0, 64, 0, true), true).unwrap();
    assert_eq!(world.block_state(BlockPos { x: 0, y: 64, z: 0 }).property("charge"), Some("2"));
}

#[test]
fn uncharged_working_anchor_is_missing_unless_forced() {
    let mut world = TestWorld::flat(63);
    world.anchor_works = true;
    world.put(
        0,
        64,
        0,
        BlockStateModel::new("minecraft:respawn_anchor").with_property("charge", "0"),
    );
    assert_eq!(find_respawn_and_use_spawn_block(&mut world, config(0, 64, 0, false), true), None);
    assert!(find_respawn_and_use_spawn_block(&mut world, config(0, 64, 0, true), true).is_some());
}

#[test]
fn dismount_prefers_safe_spots_and_rejects_dangerous_blocks() {
    let mut world = TestWorld::flat(63);
    world.put(0, 64, 0, BlockStateModel::new("minecraft:cactus"));
    assert_eq!(find_safe_dismount_location(&world, BlockPos { x: 0, y: 64, z: 0 }, true), None);
    world.put(1, 64, 0, BlockStateModel::new("minecraft:fire"));
    assert_eq!(find_safe_dismount_location(&world, BlockPos { x: 1, y: 64, z: 0 }, true), None);
    assert_eq!(
        find_safe_dismount_location(&world, BlockPos { x: 2, y: 64, z: 0 }, true),
        Some((2.5, 64.0, 0.5))
    );
}

#[test]
fn dismount_requires_headroom() {
    let mut world = TestWorld::flat(63);
    world.put(0, 65, 0, BlockStateModel::new("minecraft:stone"));
    assert_eq!(find_safe_dismount_location(&world, BlockPos { x: 0, y: 64, z: 0 }, false), None);
}

#[test]
fn dismount_over_an_air_gap_has_no_floor() {
    let world = TestWorld::flat(40);
    assert_eq!(find_safe_dismount_location(&world, BlockPos { x: 0, y: 64, z: 0 }, false), None);
}

#[test]
fn dismount_rejects_end_portals() {
    let mut world = TestWorld::flat(63);
    world.put(0, 64, 0, BlockStateModel::new("minecraft:end_portal"));
    assert_eq!(find_safe_dismount_location(&world, BlockPos { x: 0, y: 64, z: 0 }, false), None);
}

#[test]
fn bed_offsets_match_java_layout() {
    // forward = SOUTH (0,1), side = EAST (1,0).
    let surround = bed_surround_offsets(Direction::South, Direction::East);
    assert_eq!(
        surround,
        vec![(1, 0), (1, -1), (1, -2), (0, -2), (-1, -2), (-1, -1), (-1, 0), (-1, 1), (0, 1), (1, 1)]
    );
    assert_eq!(bed_above_offsets(Direction::South), vec![(0, 0), (0, -1)]);
    assert_eq!(bed_stand_up_offsets(Direction::South, Direction::East).len(), 12);
}

#[test]
fn facing_angle_and_clockwise_follow_java() {
    assert_eq!(clockwise(Direction::North), Direction::East);
    assert_eq!(clockwise(Direction::West), Direction::North);
    // yaw 0 faces south (+z): SOUTH.isFacingAngle(0) is true, NORTH is false.
    assert!(is_facing_angle(Direction::South, 0.0));
    assert!(!is_facing_angle(Direction::North, 0.0));
    assert!(is_facing_angle(Direction::West, 90.0));
}

#[test]
fn anchor_offsets_are_ring_then_below_ring_then_above_ring_then_up() {
    let offsets = anchor_respawn_offsets();
    assert_eq!(offsets.len(), 25);
    assert_eq!(offsets[0], (0, 0, -1));
    assert_eq!(offsets[8], (0, -1, -1));
    assert_eq!(offsets[16], (0, 1, -1));
    assert_eq!(offsets[24], (0, 1, 0));
}

#[test]
fn wrap_degrees_matches_mth() {
    assert_eq!(wrap_degrees(190.0), -170.0);
    assert_eq!(wrap_degrees(-190.0), 170.0);
    assert_eq!(wrap_degrees(180.0), -180.0);
}
