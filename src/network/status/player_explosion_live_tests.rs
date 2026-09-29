//! Tests of the session half of explosions: presence publishing and applying
//! the [`ExplosionHit`]s the world tick queued.

// `PlaySessionState` has ~60 fields; tests set only the few they care about.
#![allow(clippy::field_reassign_with_default)]

use super::player_explosion_live::{presence_of, sync_with_world};
use super::*;
use crate::damage_type::DamageEntityRef;
use crate::network::world_broadcast::ExplosionHit;

fn hit(damage: f32) -> ExplosionHit {
    ExplosionHit {
        damage,
        direct: Some(DamageEntityRef::non_living(500)),
        causing: None,
        causing_type: None,
        source_position: [0.0, 64.0, 0.0],
    }
}

#[test]
fn a_session_publishes_its_presence_and_applies_queued_hits() {
    let bus = WorldPacketBus::default();
    let subscription = bus.subscribe(7);
    let mut state = PlaySessionState::default();
    state.x = 3.5;
    state.y = 70.0;
    state.z = -1.5;
    assert!(bus.push_explosion_hit(7, hit(6.0)));
    assert!(sync_with_world(&mut state, &subscription));
    assert_eq!(state.health, 14.0, "the explosion damage went through the hurt pipeline");
    let (token, presence) = bus.players()[0];
    assert_eq!(token, 7);
    assert_eq!(presence.position, [3.5, 70.0, -1.5]);
    assert_eq!((presence.width, presence.height), (0.6, 1.8));
    // The hits were consumed.
    assert!(!sync_with_world(&mut state, &subscription));
}

#[test]
fn sneaking_players_are_shorter() {
    let mut state = PlaySessionState::default();
    state.input_shift = true;
    let presence = presence_of(&state);
    assert_eq!((presence.height, presence.eye_height), (1.5, 1.27));
}

#[test]
fn creative_players_are_invulnerable_to_ordinary_explosions() {
    let bus = WorldPacketBus::default();
    let subscription = bus.subscribe(1);
    let mut state = PlaySessionState::default();
    state.abilities.invulnerable = true;
    state.game_mode = GameMode::Creative;
    bus.push_explosion_hit(1, hit(10.0));
    assert!(!sync_with_world(&mut state, &subscription));
    assert_eq!(state.health, 20.0);
    assert!(bus.players()[0].1.creative);
}

#[test]
fn hits_for_departed_sessions_are_dropped() {
    let bus = WorldPacketBus::default();
    assert!(!bus.push_explosion_hit(99, hit(1.0)));
}

#[test]
fn the_attributed_source_is_the_responsible_entity() {
    let bus = WorldPacketBus::default();
    let subscription = bus.subscribe(3);
    let mut state = PlaySessionState::default();
    let mut owned = hit(4.0);
    owned.causing = Some(DamageEntityRef::player(2, false));
    owned.causing_type = Some("minecraft:player");
    bus.push_explosion_hit(3, owned);
    assert!(sync_with_world(&mut state, &subscription));
    assert_eq!(state.health, 16.0);
}
