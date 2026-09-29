//! Wire-format tests for the packets an explosion and a lit TNT put on the bus.

use super::explosion_live::{
    explode, DirectedPackets, ExplosionEnv, ExplosionRequest, ExplosionSinks,
};
use super::tnt_live_tests::{drain, presence_at, World};
use super::*;
use crate::live_block_entities::registry_ids::{
    entity_type_protocol_id, particle_type_protocol_id, sound_event_protocol_id,
};
use crate::network::play::{Vec3, CLIENTBOUND_EXPLODE_PACKET_ID};
use crate::network::varint::read_var_i32;
use crate::server_explosion::{BlockCalculator, ExplosionRules, LevelExplosionInteraction};

#[test]
fn the_registry_ids_the_packets_use_all_resolve() {
    for particle in [
        "minecraft:explosion",
        "minecraft:explosion_emitter",
        "minecraft:poof",
        "minecraft:smoke",
    ] {
        assert!(particle_type_protocol_id(particle).is_some(), "{particle}");
    }
    assert!(sound_event_protocol_id("minecraft:entity.generic.explode").is_some());
    assert!(sound_event_protocol_id("minecraft:entity.tnt.primed").is_some());
    assert!(entity_type_protocol_id("minecraft:tnt").is_some());
    // The registry agrees with the entity type ids the other live entities hard-code.
    assert_eq!(entity_type_protocol_id("minecraft:falling_block"), Some(51));
    assert_eq!(entity_type_protocol_id("minecraft:experience_orb"), Some(49));
    assert_eq!(entity_type_protocol_id("minecraft:item"), Some(71));
}

/// Explodes a radius-`radius` blast over a platform and returns the explode
/// packet a player at `player_x` received (payload after the packet id).
fn explode_packet(name: &str, radius: f32, interaction: LevelExplosionInteraction) -> Vec<u8> {
    let world = World::new(name);
    world.platform(3);
    let player = world.bus.subscribe(1);
    player.update_presence(presence_at(20.0, 221.0, 0.5));
    let request = ExplosionRequest {
        source: None,
        center: Vec3 {
            x: 0.5,
            y: 221.5,
            z: 0.5,
        },
        radius,
        fire: false,
        interaction,
        calculator: BlockCalculator::Default,
    };
    let mut packets = DirectedPackets::default();
    let mut frames = Vec::new();
    {
        let mut guard = lock_status_mutex(&world.ticks);
        let WorldTicks { fluid, block, .. } = &mut *guard;
        explode(
            &mut frames,
            &ExplosionEnv {
                layout: &world.layout,
                seed: 42,
                cache: &world.cache,
                world_items: &world.items,
                bus: &world.bus,
                rules: ExplosionRules::default(),
                game_time: 1,
                max_chained_neighbor_updates: 1_000_000,
            },
            &mut ExplosionSinks {
                block,
                fluid,
                packets: &mut packets,
            },
            &mut Vec::new(),
            &request,
        )
        .unwrap();
    }
    packets.publish(&world.bus);
    let bytes = drain(&player);
    let mut rest = bytes.as_slice();
    while !rest.is_empty() {
        let length = read_var_i32(&mut rest).unwrap() as usize;
        let (mut payload, tail) = rest.split_at(length);
        if read_var_i32(&mut payload).unwrap() == CLIENTBOUND_EXPLODE_PACKET_ID {
            return payload.to_vec();
        }
        rest = tail;
    }
    panic!("no explode packet");
}

/// One `ExplosionParticleInfo` with its weight.
#[derive(Debug, PartialEq)]
struct BlockParticle {
    id: i32,
    scaling: f32,
    speed: f32,
    weight: i32,
}

/// A decoded `ClientboundExplodePacket` without knockback.
struct DecodedExplosion {
    particle: i32,
    sound: i32,
    block_particles: Vec<BlockParticle>,
}

fn decode(mut payload: &[u8]) -> DecodedExplosion {
    let take = |payload: &mut &[u8], n: usize| {
        let (head, tail) = payload.split_at(n);
        *payload = tail;
        head.to_vec()
    };
    take(&mut payload, 24 + 4 + 4); // centre, radius, block count
    assert_eq!(take(&mut payload, 1), vec![0], "no knockback for a distant player");
    let particle = read_var_i32(&mut payload).unwrap();
    let sound = read_var_i32(&mut payload).unwrap();
    let count = read_var_i32(&mut payload).unwrap();
    let mut block_particles = Vec::new();
    for _ in 0..count {
        let id = read_var_i32(&mut payload).unwrap();
        let scaling = f32::from_be_bytes(take(&mut payload, 4).try_into().unwrap());
        let speed = f32::from_be_bytes(take(&mut payload, 4).try_into().unwrap());
        let weight = read_var_i32(&mut payload).unwrap();
        block_particles.push(BlockParticle {
            id,
            scaling,
            speed,
            weight,
        });
    }
    assert!(payload.is_empty(), "no trailing bytes");
    DecodedExplosion {
        particle,
        sound,
        block_particles,
    }
}

#[test]
fn a_large_destroying_explosion_uses_the_emitter_particle_and_default_sound() {
    let packet = decode(&explode_packet("packet-large", 4.0, LevelExplosionInteraction::Tnt));
    assert_eq!(Some(packet.particle), particle_type_protocol_id("minecraft:explosion_emitter"));
    // `SoundEvent.STREAM_CODEC` writes registered holders as `id + 1`.
    assert_eq!(
        Some(packet.sound - 1),
        sound_event_protocol_id("minecraft:entity.generic.explode")
    );
    // Level.DEFAULT_EXPLOSION_BLOCK_PARTICLES: poof (0.5, 1.0) and smoke (1.0, 1.0), weight 1.
    assert_eq!(
        packet.block_particles,
        vec![
            BlockParticle {
                id: particle_type_protocol_id("minecraft:poof").unwrap(),
                scaling: 0.5,
                speed: 1.0,
                weight: 1,
            },
            BlockParticle {
                id: particle_type_protocol_id("minecraft:smoke").unwrap(),
                scaling: 1.0,
                speed: 1.0,
                weight: 1,
            },
        ]
    );
}

#[test]
fn small_or_non_destroying_explosions_use_the_plain_explosion_particle() {
    let small = decode(&explode_packet("packet-small", 1.5, LevelExplosionInteraction::Tnt));
    assert_eq!(Some(small.particle), particle_type_protocol_id("minecraft:explosion"));
    let keep = decode(&explode_packet("packet-keep", 4.0, LevelExplosionInteraction::None));
    assert_eq!(Some(keep.particle), particle_type_protocol_id("minecraft:explosion"));
}
