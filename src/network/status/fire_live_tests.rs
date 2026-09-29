//! Tests of the live fire tick's player-dependent spread gate
//! (`ServerLevel.canSpreadFireAround`).

use super::fire_live::{fire_spread_players, run_live_fire_tick, FireEnvironment};
use super::tnt_live_tests::{presence_at, World};
use super::*;
use crate::block_update::BlockPos;

const Y: i32 = 220;
const FIRE: &str = "minecraft:fire[age=0,east=false,north=false,south=false,up=false,west=false]";

/// Ticks a fire next to a TNT block up to `ticks` times, re-lighting the fire
/// each time so it cannot burn out, and returns whether TNT got primed.
fn fire_burns_tnt(world: &World, radius: i32, players: &[[f64; 3]], ticks: i64) -> bool {
    let fire = BlockPos { x: 0, y: Y + 1, z: 0 };
    let mut block_ticks = LiveBlockTicks::new();
    let mut sink = Vec::new();
    let environment = FireEnvironment {
        raining: false,
        difficulty_id: 2,
        spread_radius: radius,
    };
    for game_time in 0..ticks {
        world.set(1, Y + 1, 0, "minecraft:tnt");
        world.set(0, Y + 1, 0, FIRE);
        let state = read_live_block_model_at(&world.cache, &world.layout, 42, fire);
        run_live_fire_tick(
            &mut sink,
            CompressionState::disabled(),
            super::block_placement_live::LiveBlockWorld {
                layout: &world.layout,
                seed: 42,
                cache: &world.cache,
            },
            &mut block_ticks,
            game_time,
            environment,
            &world.items,
            players,
            state,
            fire,
        )
        .unwrap();
        if world.tnt_count() > 0 {
            return true;
        }
    }
    false
}

#[test]
fn unlimited_spread_radius_burns_without_any_player_nearby() {
    let world = World::new("spread-unlimited");
    world.platform(4);
    assert!(fire_burns_tnt(&world, -1, &[], 20_000));
}

#[test]
fn a_finite_radius_freezes_fire_when_no_player_is_close() {
    let world = World::new("spread-frozen");
    world.platform(4);
    assert!(!fire_burns_tnt(&world, 32, &[], 3_000), "no player: the fire tick does nothing");
    assert!(!fire_burns_tnt(&world, 32, &[[0.5, f64::from(Y + 1), 40.5]], 3_000), "40 blocks away");
}

#[test]
fn a_finite_radius_lets_fire_burn_near_a_player() {
    let world = World::new("spread-near");
    world.platform(4);
    assert!(fire_burns_tnt(&world, 32, &[[0.5, f64::from(Y + 1), 10.5]], 20_000));
}

#[test]
fn spectators_do_not_count_as_nearby_players() {
    let bus = WorldPacketBus::default();
    let survivor = bus.subscribe(1);
    survivor.update_presence(presence_at(1.0, 64.0, 2.0));
    let spectator = bus.subscribe(2);
    let mut watching = presence_at(50.0, 64.0, 50.0);
    watching.spectator = true;
    spectator.update_presence(watching);
    assert_eq!(fire_spread_players(&bus), vec![[1.0, 64.0, 2.0]]);
}
