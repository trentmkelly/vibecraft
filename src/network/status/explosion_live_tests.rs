//! Tests of `ServerLevel.explode` applied to the live level, without the
//! `PrimedTnt` entity around it.

use super::explosion_live::{
    explode, DirectedPackets, ExplosionEnv, ExplosionRequest, ExplosionSinks,
};
use super::tnt_live_tests::{drain, packet_ids, presence_at, World};
use super::*;
use crate::network::play::{Vec3, CLIENTBOUND_EXPLODE_PACKET_ID};
use crate::primed_tnt::PrimedTntEntity;
use crate::server_explosion::{BlockCalculator, ExplosionRules, LevelExplosionInteraction};

const Y: i32 = 220;

fn request(interaction: LevelExplosionInteraction, fire: bool) -> ExplosionRequest {
    ExplosionRequest {
        source: None,
        center: Vec3 {
            x: 0.5,
            y: f64::from(Y) + 1.5,
            z: 0.5,
        },
        radius: 4.0,
        fire,
        interaction,
        calculator: BlockCalculator::Default,
    }
}

/// Runs one explosion against `world`, returning the frame buffer and the
/// primed TNT the blast created.
fn run(world: &World, rules: ExplosionRules, request: &ExplosionRequest) -> (Vec<u8>, Vec<PrimedTntEntity>) {
    let mut frames = Vec::new();
    let mut tnts = Vec::new();
    let mut packets = DirectedPackets::default();
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
                rules,
                game_time: 1,
                max_chained_neighbor_updates: 1_000_000,
            },
            &mut ExplosionSinks {
                block,
                fluid,
                packets: &mut packets,
            },
            &mut tnts,
            request,
        )
        .unwrap();
    }
    packets.publish(&world.bus);
    (frames, tnts)
}

fn count_items(world: &World, name: &str) -> i32 {
    lock_status_mutex(&world.items)
        .entities
        .iter()
        .filter(|item| item.item == name)
        .map(|item| item.count)
        .sum()
}

#[test]
fn tnt_interaction_destroys_solid_blocks_and_writes_block_updates() {
    let world = World::new("destroy");
    world.platform(6);
    let (frames, _) = run(&world, ExplosionRules::default(), &request(LevelExplosionInteraction::Tnt, false));
    assert_eq!(world.block(0, Y, 0), "minecraft:air");
    assert!(packet_ids(&frames).contains(&crate::network::play::CLIENTBOUND_BLOCK_UPDATE_PACKET_ID));
    // Far corners of the platform are out of range.
    assert_eq!(world.block(6, Y, 6), "minecraft:stone");
}

#[test]
fn obsidian_and_bedrock_like_resistance_survives() {
    let world = World::new("obsidian");
    for x in -6..=6 {
        for z in -6..=6 {
            world.set(x, Y, z, "minecraft:obsidian");
        }
    }
    run(&world, ExplosionRules::default(), &request(LevelExplosionInteraction::Tnt, false));
    assert_eq!(world.block(0, Y, 0), "minecraft:obsidian");
}

#[test]
fn keep_interaction_hurts_entities_but_leaves_the_blocks() {
    let world = World::new("keep");
    world.platform(6);
    let rules = ExplosionRules {
        mob_griefing: false,
        ..ExplosionRules::default()
    };
    let player = world.bus.subscribe(1);
    player.update_presence(presence_at(2.5, f64::from(Y + 1), 0.5));
    run(&world, rules, &request(LevelExplosionInteraction::Mob, false));
    assert_eq!(world.block(0, Y, 0), "minecraft:stone", "mobGriefing=false keeps blocks");
    assert_eq!(player.take_explosion_hits().len(), 1);
    assert!(count_items(&world, "minecraft:cobblestone") == 0);
}

/// A platform of coal ore, whose loot applies `ApplyExplosionDecay`.
fn coal_platform(world: &World) {
    for x in -6..=6 {
        for z in -6..=6 {
            world.set(x, Y, z, "minecraft:coal_ore");
        }
    }
}

#[test]
fn destroy_with_decay_thins_the_drops_by_the_explosion_radius() {
    let plain_world = World::new("decay-off");
    coal_platform(&plain_world);
    run(&plain_world, ExplosionRules::default(), &request(LevelExplosionInteraction::Tnt, false));
    let plain = count_items(&plain_world, "minecraft:coal");

    let decay_world = World::new("decay-on");
    coal_platform(&decay_world);
    let rules = ExplosionRules {
        tnt_explosion_drop_decay: true,
        ..ExplosionRules::default()
    };
    run(&decay_world, rules, &request(LevelExplosionInteraction::Tnt, false));
    let decayed = count_items(&decay_world, "minecraft:coal");
    assert!(plain > 20, "Destroy (no decay) keeps every drop: {plain}");
    // Each drop survives with probability 1 / radius = 1/4.
    assert!(decayed < plain / 2, "decay thins the drops: {decayed} vs {plain}");
}

#[test]
fn fire_explosions_light_air_above_solid_ground() {
    let world = World::new("fire");
    world.platform(6);
    let rules = ExplosionRules::default();
    // Keep the ground: only fire and entity effects.
    let mut fire = request(LevelExplosionInteraction::None, true);
    fire.radius = 4.0;
    run(&world, rules, &fire);
    let mut lit = 0;
    for x in -4..=4 {
        for z in -4..=4 {
            if world.block(x, Y + 1, z) == "minecraft:fire" {
                lit += 1;
            }
        }
    }
    assert!(lit > 0, "some air positions above the platform caught fire");
    assert_eq!(world.block(0, Y, 0), "minecraft:stone");
    // FireBlock.onPlace scheduled a spread tick for each fire.
    assert!(lock_status_mutex(&world.ticks).block.queues.count() >= lit);
}

#[test]
fn a_used_portal_tnt_leaves_nether_portal_blocks_alone() {
    let default_world = World::new("portal-default");
    let used_world = World::new("portal-used");
    for world in [&default_world, &used_world] {
        world.platform(4);
        world.set(1, Y + 1, 0, "minecraft:nether_portal[axis=x]");
    }
    run(&default_world, ExplosionRules::default(), &request(LevelExplosionInteraction::Tnt, false));
    let mut used = request(LevelExplosionInteraction::Tnt, false);
    used.calculator = BlockCalculator::UsedPortal;
    run(&used_world, ExplosionRules::default(), &used);
    assert_eq!(default_world.block(1, Y + 1, 0), "minecraft:air");
    assert_eq!(used_world.block(1, Y + 1, 0), "minecraft:nether_portal");
}

#[test]
fn only_players_within_64_blocks_receive_the_explode_packet() {
    let world = World::new("range");
    world.platform(2);
    let near = world.bus.subscribe(1);
    near.update_presence(presence_at(40.0, f64::from(Y), 0.0));
    let far = world.bus.subscribe(2);
    far.update_presence(presence_at(70.0, f64::from(Y), 0.0));
    run(&world, ExplosionRules::default(), &request(LevelExplosionInteraction::Tnt, false));
    assert!(packet_ids(&drain(&near)).contains(&CLIENTBOUND_EXPLODE_PACKET_ID));
    assert!(!packet_ids(&drain(&far)).contains(&CLIENTBOUND_EXPLODE_PACKET_ID));
}

#[test]
fn the_explode_packet_reports_the_number_of_exploded_positions() {
    let world = World::new("count");
    world.platform(6);
    let player = world.bus.subscribe(1);
    player.update_presence(presence_at(0.5, f64::from(Y + 30), 0.5));
    run(&world, ExplosionRules::default(), &request(LevelExplosionInteraction::Tnt, false));
    let frames = drain(&player);
    let mut rest = frames.as_slice();
    let mut count = None;
    while !rest.is_empty() {
        let length = crate::network::varint::read_var_i32(&mut rest).unwrap() as usize;
        let (payload, tail) = rest.split_at(length);
        if payload[0] == CLIENTBOUND_EXPLODE_PACKET_ID as u8 {
            // id, centre (3 doubles), radius, then the block count.
            count = Some(i32::from_be_bytes(payload[29..33].try_into().unwrap()));
        }
        rest = tail;
    }
    assert!(count.unwrap() > 20, "air positions are counted too: {count:?}");
}

#[test]
fn tnt_blocks_in_range_become_primed_entities_owned_by_the_responsible_entity() {
    let world = World::new("owner");
    world.platform(4);
    world.set(2, Y + 1, 0, "minecraft:tnt");
    let owner = crate::damage_type::DamageEntityRef::player(1, false);
    let mut with_owner = request(LevelExplosionInteraction::Tnt, false);
    with_owner.source = Some(super::explosion_live::ExplosionSource {
        entity_id: 500,
        position: with_owner.center,
        owner: Some(owner),
    });
    let (_, tnts) = run(&world, ExplosionRules::default(), &with_owner);
    assert_eq!(tnts.len(), 1);
    assert_eq!(tnts[0].owner, Some(owner));
    assert_eq!((tnts[0].pos.x, tnts[0].pos.y, tnts[0].pos.z), (2.5, f64::from(Y + 1), 0.5));
}

#[test]
fn tnt_blocks_are_not_primed_when_tnt_explodes_is_off() {
    let world = World::new("tnt-off");
    world.platform(4);
    world.set(2, Y + 1, 0, "minecraft:tnt");
    let rules = ExplosionRules {
        tnt_explodes: false,
        ..ExplosionRules::default()
    };
    let (_, tnts) = run(&world, rules, &request(LevelExplosionInteraction::Tnt, false));
    assert!(tnts.is_empty());
    assert_eq!(world.block(2, Y + 1, 0), "minecraft:air", "the block is still destroyed");
}
