//! Tests for the `PrimedTnt` entity simulation.

use super::*;
use crate::entity_collision::test_world::TestWorld;

fn tnt_at(x: f64, y: f64, z: f64) -> PrimedTntEntity {
    PrimedTntEntity::new(7, (x, y, z), None, 0.25)
}

#[test]
fn spawn_state_matches_the_java_constructor() {
    let tnt = PrimedTntEntity::new(7, (1.5, 64.0, 2.5), None, 0.0);
    assert_eq!(tnt.fuse, 80);
    assert_eq!(tnt.explosion_power, 4.0);
    assert_eq!(tnt.block_state, "minecraft:tnt[unstable=false]");
    // rot = 0: (-sin 0 * 0.02, 0.2F, -cos 0 * 0.02).
    assert_eq!(tnt.delta.x, 0.0);
    assert_eq!(tnt.delta.y, f64::from(0.2_f32));
    assert!((tnt.delta.z + 0.02).abs() < 1.0E-12);
    assert!(!tnt.announced);
}

#[test]
fn the_spawn_impulse_is_a_slow_horizontal_kick_in_a_random_direction() {
    for step in 0..8 {
        let tnt = PrimedTntEntity::new(1, (0.0, 0.0, 0.0), None, f64::from(step) / 8.0);
        let horizontal = (tnt.delta.x.powi(2) + tnt.delta.z.powi(2)).sqrt();
        assert!((horizontal - 0.02).abs() < 1.0E-12);
    }
}

#[test]
fn bounding_box_is_098_wide_centred_on_the_position() {
    let tnt = tnt_at(10.5, 64.0, -3.5);
    let bb = tnt.bounding_box();
    assert!((bb.max_x - bb.min_x - 0.98).abs() < 1.0E-6);
    assert!((bb.max_y - bb.min_y - 0.98).abs() < 1.0E-6);
    assert!(((bb.min_x + bb.max_x) / 2.0 - 10.5).abs() < 1.0E-12);
    assert_eq!(bb.min_y, 64.0);
    // Explosion centre: getY(0.0625).
    assert!((tnt.y_at(0.0625) - (64.0 + 0.98 * 0.0625)).abs() < 1.0E-6);
}

#[test]
fn free_fall_applies_gravity_then_drag() {
    let world = TestWorld::default();
    let mut tnt = tnt_at(0.5, 100.0, 0.5);
    tnt.delta = Vec3::ZERO;
    assert_eq!(tnt.tick(&world), TntTick::Burning);
    // gravity 0.04 added before the move, then *0.98 drag.
    assert!((tnt.pos.y - (100.0 - 0.04)).abs() < 1.0E-12);
    assert!((tnt.delta.y - -0.04 * 0.98).abs() < 1.0E-12);
    assert_eq!(tnt.fuse, 79);
}

#[test]
fn landing_stops_the_fall_and_bounces_the_velocity_the_java_way() {
    let world = TestWorld::floor(63);
    let mut tnt = tnt_at(0.5, 64.3, 0.5);
    tnt.delta = Vec3 {
        x: 0.0,
        y: -0.5,
        z: 0.0,
    };
    tnt.tick(&world);
    assert!(tnt.on_ground);
    assert!((tnt.pos.y - 64.0).abs() < 1.0E-9, "rests on the stone top: {}", tnt.pos.y);
    // updateEntityMovementAfterFallOn zeroes Y, then the on-ground multiply
    // (0.7, -0.5, 0.7) keeps it at 0.
    assert_eq!(tnt.delta.y, 0.0);
    assert!(tnt.supporting_block().is_some());
}

#[test]
fn horizontal_velocity_is_damped_on_the_ground() {
    let world = TestWorld::floor(63);
    let mut tnt = tnt_at(0.5, 64.0, 0.5);
    tnt.delta = Vec3 {
        x: 0.1,
        y: 0.0,
        z: 0.0,
    };
    tnt.tick(&world);
    // Gravity makes the box press into the floor (on ground), then *0.98 and *0.7.
    assert!((tnt.delta.x - 0.1 * 0.98 * 0.7).abs() < 1.0E-9, "{}", tnt.delta.x);
}

#[test]
fn a_wall_zeroes_the_blocked_horizontal_component() {
    let mut world = TestWorld::floor(63);
    world.set(1, 64, 0, "minecraft:stone");
    let mut tnt = tnt_at(0.5, 64.0, 0.5);
    tnt.delta = Vec3 {
        x: 0.4,
        y: 0.0,
        z: 0.0,
    };
    tnt.tick(&world);
    assert_eq!(tnt.delta.x, 0.0);
    assert!(tnt.pos.x <= 0.51 + 1.0E-9, "stopped at the wall: {}", tnt.pos.x);
}

#[test]
fn slime_blocks_bounce_at_eighty_percent() {
    let mut world = TestWorld::default();
    for x in -2..3 {
        for z in -2..3 {
            world.set(x, 63, z, "minecraft:slime_block");
        }
    }
    let mut tnt = tnt_at(0.5, 64.2, 0.5);
    tnt.delta = Vec3 {
        x: 0.0,
        y: -0.6,
        z: 0.0,
    };
    tnt.tick(&world);
    // Landing: y = 0.6 + 0.04 gravity => -0.64 collides; bounce -y * 0.8 then
    // the tick's own 0.98 drag and the on-ground (-0.5) multiply.
    let expected = (0.64 * 0.8) * 0.98 * -0.5;
    assert!((tnt.delta.y - expected).abs() < 1.0E-9, "{} vs {expected}", tnt.delta.y);
}

#[test]
fn soul_sand_slows_horizontal_motion() {
    let mut world = TestWorld::default();
    for x in -2..3 {
        for z in -2..3 {
            world.set(x, 63, z, "minecraft:soul_sand");
        }
    }
    let mut tnt = tnt_at(0.5, 63.875, 0.5);
    tnt.delta = Vec3 {
        x: 0.2,
        y: 0.0,
        z: 0.0,
    };
    tnt.tick(&world);
    assert!(tnt.delta.x < 0.2 * 0.98 * 0.7, "speed factor 0.4 applied: {}", tnt.delta.x);
}

#[test]
fn the_fuse_counts_down_and_detonates_when_it_reaches_zero() {
    let world = TestWorld::floor(63);
    let mut tnt = tnt_at(0.5, 64.0, 0.5);
    for expected_fuse in (1..80).rev() {
        assert_eq!(tnt.tick(&world), TntTick::Burning);
        assert_eq!(tnt.fuse, expected_fuse);
    }
    assert_eq!(tnt.tick(&world), TntTick::Detonate);
    assert_eq!(tnt.fuse, 0);
}

#[test]
fn a_short_fuse_detonates_on_its_first_tick() {
    let world = TestWorld::default();
    let mut tnt = tnt_at(0.5, 64.0, 0.5);
    tnt.fuse = 1;
    assert_eq!(tnt.tick(&world), TntTick::Detonate);
}

#[test]
fn push_adds_motion_and_requests_a_sync() {
    let mut tnt = tnt_at(0.0, 0.0, 0.0);
    tnt.delta = Vec3::ZERO;
    tnt.push(Vec3 {
        x: 1.0,
        y: 2.0,
        z: 3.0,
    });
    assert_eq!((tnt.delta.x, tnt.delta.y, tnt.delta.z), (1.0, 2.0, 3.0));
    assert!(tnt.needs_sync);
    tnt.needs_sync = false;
    tnt.push(Vec3 {
        x: f64::NAN,
        y: 0.0,
        z: 0.0,
    });
    assert!(!tnt.needs_sync, "non-finite impulses are ignored");
}

#[test]
fn water_currents_push_the_tnt() {
    let mut world = TestWorld::floor(63);
    world.set(0, 64, 0, "minecraft:water[level=0]");
    world.set(1, 64, 0, "minecraft:water[level=1]");
    let mut tnt = tnt_at(0.5, 64.0, 0.5);
    tnt.delta = Vec3::ZERO;
    tnt.tick(&world);
    assert!(tnt.delta.x > 0.0, "carried east by the flow: {:?}", tnt.delta);
}
