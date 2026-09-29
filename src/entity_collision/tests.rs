//! Tests for the entity collision and ray-clip port.

use super::test_world::TestWorld;
use super::*;

fn unit_box_at(x: f64, y: f64, z: f64) -> Aabb {
    Aabb::new(x - 0.49, y, z - 0.49, x + 0.49, y + 0.98, z + 0.49)
}

fn v(x: f64, y: f64, z: f64) -> Vec3 {
    Vec3 { x, y, z }
}

#[test]
fn falling_entity_stops_on_the_block_below() {
    let world = TestWorld::floor(63);
    // Feet at y=64.5, moving down 2 blocks: stone top is y=64.
    let resolved = collide_bounding_box(&world, &unit_box_at(0.5, 64.5, 0.5), v(0.0, -2.0, 0.0));
    assert!((resolved.y - -0.5).abs() < 1.0E-9, "stopped at the stone top: {resolved:?}");
    assert_eq!((resolved.x, resolved.z), (0.0, 0.0));
}

#[test]
fn horizontal_movement_is_clipped_by_a_wall_and_axes_resolve_independently() {
    let mut world = TestWorld::floor(63);
    world.set(2, 64, 0, "minecraft:stone");
    // Standing on the floor at x=0.5; the wall's west face is x=2.
    let resolved = collide_bounding_box(&world, &unit_box_at(0.5, 64.0, 0.5), v(3.0, 0.0, 0.5));
    assert!((resolved.x - 1.01).abs() < 1.0E-9, "x clipped to the wall: {resolved:?}");
    assert!((resolved.z - 0.5).abs() < 1.0E-9, "z is unobstructed: {resolved:?}");
}

#[test]
fn zero_movement_is_returned_unchanged() {
    let world = TestWorld::floor(63);
    assert_eq!(
        collide_bounding_box(&world, &unit_box_at(0.5, 64.0, 0.5), Vec3::ZERO),
        Vec3::ZERO
    );
}

#[test]
fn shapes_without_collision_do_not_block() {
    let mut world = TestWorld::floor(63);
    world.set(0, 64, 0, "minecraft:torch");
    let resolved = collide_bounding_box(&world, &unit_box_at(-1.5, 64.0, 0.5), v(3.0, 0.0, 0.0));
    assert_eq!(resolved.x, 3.0);
}

#[test]
fn partial_shapes_use_their_own_height() {
    let mut world = TestWorld::floor(63);
    world.set(0, 64, 0, "minecraft:stone_slab[type=bottom,waterlogged=false]");
    // Falling onto a bottom slab stops at y=64.5.
    let resolved = collide_bounding_box(&world, &unit_box_at(0.5, 65.0, 0.5), v(0.0, -1.0, 0.0));
    assert!((resolved.y - -0.5).abs() < 1.0E-9, "slab top at 64.5: {resolved:?}");
}

#[test]
fn clip_hits_a_solid_block_between_the_points_and_misses_around_it() {
    let mut world = TestWorld::default();
    world.set(3, 64, 0, "minecraft:stone");
    let through = clip_hits_collider(&world, v(0.5, 64.5, 0.5), v(6.5, 64.5, 0.5));
    assert!(through, "the stone blocks the segment");
    let above = clip_hits_collider(&world, v(0.5, 66.5, 0.5), v(6.5, 66.5, 0.5));
    assert!(!above, "a segment above the block is clear");
    let short = clip_hits_collider(&world, v(0.5, 64.5, 0.5), v(2.5, 64.5, 0.5));
    assert!(!short, "the segment ends before the block");
}

#[test]
fn clip_ignores_blocks_without_collision_and_degenerate_segments() {
    let mut world = TestWorld::default();
    world.set(3, 64, 0, "minecraft:torch");
    assert!(!clip_hits_collider(&world, v(0.5, 64.5, 0.5), v(6.5, 64.5, 0.5)));
    let same = v(1.5, 64.5, 0.5);
    assert!(!clip_hits_collider(&world, same, same));
}

#[test]
fn clip_starting_inside_a_block_counts_as_a_hit() {
    let mut world = TestWorld::default();
    world.set(0, 64, 0, "minecraft:stone");
    assert!(clip_hits_collider(&world, v(0.5, 64.5, 0.5), v(5.5, 64.5, 0.5)));
}

#[test]
fn clip_walks_diagonal_segments_through_the_correct_cells() {
    let mut world = TestWorld::default();
    world.set(2, 65, 2, "minecraft:stone");
    assert!(clip_hits_collider(&world, v(0.5, 63.5, 0.5), v(4.5, 67.5, 4.5)));
    assert!(!clip_hits_collider(&world, v(0.5, 63.5, 0.5), v(4.5, 63.5, 4.5)));
}

#[test]
fn supporting_block_is_the_collidable_block_nearest_the_entity() {
    let world = TestWorld::floor(63);
    let bb = unit_box_at(0.5, 64.0, 0.5);
    let test_area = Aabb::new(bb.min_x, bb.min_y - 1.0E-6, bb.min_z, bb.max_x, bb.min_y, bb.max_z);
    let support = find_supporting_block(&world, &test_area, v(0.5, 64.0, 0.5));
    assert_eq!(support, Some(BlockPos { x: 0, y: 63, z: 0 }));
    let airborne = Aabb::new(bb.min_x, 70.0 - 1.0E-6, bb.min_z, bb.max_x, 70.0, bb.max_z);
    assert_eq!(find_supporting_block(&world, &airborne, v(0.5, 70.0, 0.5)), None);
}

#[test]
fn aabb_helpers_match_java_semantics() {
    let bb = Aabb::new(0.0, 0.0, 0.0, 1.0, 1.0, 1.0);
    let moved = aabb_offset(bb, 1.0, -1.0, 0.5);
    assert_eq!((moved.min_x, moved.max_y, moved.min_z), (1.0, 0.0, 0.5));
    let stretched = aabb_expand_towards(bb, v(2.0, -1.0, 0.0));
    assert_eq!((stretched.min_y, stretched.max_x, stretched.min_x), (-1.0, 3.0, 0.0));
    assert!(aabb_intersects(&bb, &Aabb::new(0.5, 0.5, 0.5, 2.0, 2.0, 2.0)));
    assert!(!aabb_intersects(&bb, &Aabb::new(1.0, 0.0, 0.0, 2.0, 1.0, 1.0)), "touching is not intersecting");
}
