//! Tests for the entity fluid interaction port.

use super::*;
use crate::entity_collision::test_world::TestWorld;

fn tnt_box(x: f64, y: f64, z: f64) -> Aabb {
    Aabb::new(x - 0.49, y, z - 0.49, x + 0.49, y + 0.98, z + 0.49)
}

#[test]
fn a_dry_box_touches_no_fluid() {
    let world = TestWorld::floor(63);
    let interaction = update_fluid_interaction(&world, &tnt_box(0.5, 64.0, 0.5), (0, 0), 64.15, false);
    assert!(!interaction.water.in_fluid());
    assert!(!interaction.lava.in_fluid());
}

#[test]
fn submersion_depth_is_the_fluid_top_above_the_box_bottom() {
    let mut world = TestWorld::floor(63);
    world.set(0, 64, 0, "minecraft:water[level=0]");
    let interaction = update_fluid_interaction(&world, &tnt_box(0.5, 64.0, 0.5), (0, 0), 64.15, false);
    // A source is 8/9 tall.
    assert!((interaction.water.height - 8.0 / 9.0).abs() < 1.0E-6, "{:?}", interaction.water);
    assert!(interaction.water.eyes_inside);
    assert!(!interaction.lava.in_fluid());
}

#[test]
fn stacked_water_reaches_the_top_of_the_block() {
    let mut world = TestWorld::floor(63);
    world.set(0, 64, 0, "minecraft:water[level=0]");
    world.set(0, 65, 0, "minecraft:water[level=0]");
    let interaction = update_fluid_interaction(&world, &tnt_box(0.5, 64.0, 0.5), (0, 0), 64.15, false);
    // The block under the box reports a full height because water sits above it.
    assert!((interaction.water.height - 1.0).abs() < 1.0E-6, "{:?}", interaction.water);
}

#[test]
fn lava_is_tracked_separately_from_water() {
    let mut world = TestWorld::floor(63);
    world.set(0, 64, 0, "minecraft:lava[level=0]");
    let interaction = update_fluid_interaction(&world, &tnt_box(0.5, 64.0, 0.5), (0, 0), 64.15, false);
    assert!(interaction.lava.in_fluid());
    assert!(!interaction.water.in_fluid());
}

#[test]
fn flowing_water_pushes_toward_the_lower_neighbour() {
    let mut world = TestWorld::floor(63);
    // A source with a level-1 flowing block to the east: the current runs east.
    world.set(0, 64, 0, "minecraft:water[level=0]");
    world.set(1, 64, 0, "minecraft:water[level=1]");
    let interaction = update_fluid_interaction(&world, &tnt_box(0.5, 64.0, 0.5), (0, 0), 64.15, false);
    let impulse = interaction
        .water
        .current_impulse(Vec3::ZERO, WATER_CURRENT_SCALE)
        .expect("a current");
    assert!(impulse.x > 0.0, "pushed east: {impulse:?}");
    assert!(impulse.z.abs() < 1.0E-9);
}

#[test]
fn still_water_has_no_current_and_ignore_current_skips_it() {
    let mut world = TestWorld::floor(63);
    world.set(0, 64, 0, "minecraft:water[level=0]");
    let still = update_fluid_interaction(&world, &tnt_box(0.5, 64.0, 0.5), (0, 0), 64.15, false);
    assert!(still.water.current_impulse(Vec3::ZERO, WATER_CURRENT_SCALE).is_none());

    world.set(1, 64, 0, "minecraft:water[level=1]");
    let ignored = update_fluid_interaction(&world, &tnt_box(0.5, 64.0, 0.5), (0, 0), 64.15, true);
    assert!(ignored.water.current_impulse(Vec3::ZERO, WATER_CURRENT_SCALE).is_none());
}

#[test]
fn slow_entities_get_the_minimum_current_impulse() {
    let mut world = TestWorld::floor(63);
    world.set(0, 64, 0, "minecraft:water[level=0]");
    world.set(1, 64, 0, "minecraft:water[level=1]");
    let interaction = update_fluid_interaction(&world, &tnt_box(0.5, 64.0, 0.5), (0, 0), 64.15, false);
    // 0.014 scale is above the 0.0045 minimum, so it is used as is.
    let impulse = interaction.water.current_impulse(Vec3::ZERO, WATER_CURRENT_SCALE).unwrap();
    assert!((impulse.x - 0.014).abs() < 1.0E-9, "{impulse:?}");
    // The lava scale (0.0023) is below the minimum for a nearly still entity.
    let mut lava_world = TestWorld::floor(63);
    lava_world.set(0, 64, 0, "minecraft:lava[level=0]");
    lava_world.set(1, 64, 0, "minecraft:lava[level=1]");
    let lava = update_fluid_interaction(&lava_world, &tnt_box(0.5, 64.0, 0.5), (0, 0), 64.15, false);
    let lava_impulse = lava.lava.current_impulse(Vec3::ZERO, LAVA_CURRENT_SCALE).unwrap();
    assert!((lava_impulse.x - 0.0045).abs() < 1.0E-9, "{lava_impulse:?}");
}
