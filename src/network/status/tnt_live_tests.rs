//! End-to-end tests of live TNT: priming, the world tick's `PrimedTnt.tick`,
//! detonation, chain reactions and the player/entity effects, driven through
//! the shared [`tick_server_world`].

use super::fire_live::FireEnvironment;
use super::tnt_live::prime_tnt;
use super::*;
use crate::block_update::BlockPos;
use crate::damage_type::DamageEntityRef;
use crate::network::play::{
    CLIENTBOUND_ADD_ENTITY_PACKET_ID, CLIENTBOUND_EXPLODE_PACKET_ID,
    CLIENTBOUND_REMOVE_ENTITIES_PACKET_ID, CLIENTBOUND_SOUND_PACKET_ID,
};
use crate::network::varint::read_var_i32;
use crate::network::world_broadcast::PlayerPresence;
use crate::random_tick::LevelRandom;
use crate::server_explosion::ExplosionRules;

/// Altitude of the test platforms: above the generated terrain.
const Y: i32 = 220;

fn temp_root(name: &str) -> std::path::PathBuf {
    let root = std::env::temp_dir().join(format!("vibecraft-tnt-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();
    root
}

pub(super) struct World {
    pub layout: WorldLayout,
    pub cache: GeneratedChunkCache,
    pub items: Arc<Mutex<WorldItemEntities>>,
    pub bus: WorldPacketBus,
    pub ticks: SharedWorldTicks,
    random: LevelRandom,
    pub rules: ExplosionRules,
    pub game_time: i64,
}

impl World {
    pub(super) fn new(name: &str) -> Self {
        Self {
            layout: WorldLayout::new(temp_root(name)),
            cache: GeneratedChunkCache::default(),
            items: Arc::new(Mutex::new(WorldItemEntities::default())),
            bus: WorldPacketBus::default(),
            ticks: Arc::new(Mutex::new(WorldTicks::new(0))),
            random: LevelRandom::new(11),
            rules: ExplosionRules::default(),
            game_time: 0,
        }
    }

    pub(super) fn set(&self, x: i32, y: i32, z: i32, block: &str) {
        self.cache
            .set_block(self.layout.root(), 42, BlockPos { x, y, z }, block);
    }

    pub(super) fn block(&self, x: i32, y: i32, z: i32) -> String {
        read_live_block_model_at(&self.cache, &self.layout, 42, BlockPos { x, y, z }).registry_id
    }

    /// A `size`-wide stone platform under `y + 1` centred on the origin.
    pub(super) fn platform(&self, size: i32) {
        for x in -size..=size {
            for z in -size..=size {
                self.set(x, Y, z, "minecraft:stone");
            }
        }
    }

    /// One `ServerLevel.tick` at the next game time.
    pub(super) fn tick(&mut self) {
        self.game_time += 1;
        tick_server_world(
            &self.ticks,
            &mut self.random,
            &ServerWorldTick {
                game_time: self.game_time,
                layout: &self.layout,
                seed: 42,
                cache: &self.cache,
                world_items: &self.items,
                max_chained_neighbor_updates: 1_000_000,
                random_tick_speed: 0,
                spread_vines: true,
                fire: FireEnvironment {
                    raining: false,
                    difficulty_id: 2,
                    spread_radius: -1,
                },
                explosion_rules: self.rules,
                bus: &self.bus,
            },
        )
        .unwrap();
    }

    pub(super) fn tnt_count(&self) -> usize {
        lock_status_mutex(&self.items).primed_tnts.len()
    }
}

/// The packet ids in a drained frame buffer.
pub(super) fn packet_ids(frames: &[u8]) -> Vec<i32> {
    let mut rest = frames;
    let mut ids = Vec::new();
    while !rest.is_empty() {
        let length = read_var_i32(&mut rest).unwrap() as usize;
        let (mut payload, tail) = rest.split_at(length);
        ids.push(read_var_i32(&mut payload).unwrap());
        rest = tail;
    }
    ids
}

pub(super) fn drain(subscription: &Subscription) -> Vec<u8> {
    let mut out = Vec::new();
    subscription
        .drain_into(&mut out, CompressionState::disabled())
        .unwrap();
    out
}

pub(super) fn presence_at(x: f64, y: f64, z: f64) -> PlayerPresence {
    PlayerPresence {
        entity_id: 1,
        position: [x, y, z],
        width: 0.6,
        height: 1.8,
        eye_height: 1.62,
        spectator: false,
        creative: false,
        flying: false,
        explosion_knockback_resistance: 0.0,
    }
}

fn prime(world: &World, x: i32, z: i32) {
    assert!(prime_tnt(
        &world.items,
        &world.cache.level_random,
        BlockPos { x, y: Y + 1, z },
        None
    ));
}

#[test]
fn priming_creates_an_unannounced_entity_at_the_block_centre() {
    let world = World::new("prime");
    let owner = DamageEntityRef::player(1, false);
    assert!(prime_tnt(&world.items, &world.cache.level_random, BlockPos { x: 3, y: 70, z: -2 }, Some(owner)));
    let store = lock_status_mutex(&world.items);
    let tnt = &store.primed_tnts[0];
    assert_eq!((tnt.pos.x, tnt.pos.y, tnt.pos.z), (3.5, 70.0, -1.5));
    assert_eq!(tnt.fuse, 80);
    assert_eq!(tnt.owner, Some(owner));
    assert!(!tnt.announced && tnt.play_prime_sound);
}

#[test]
fn priming_is_refused_when_tnt_explodes_is_off() {
    let world = World::new("prime-off");
    lock_status_mutex(&world.items).explosion_rules.tnt_explodes = false;
    assert!(!prime_tnt(&world.items, &world.cache.level_random, BlockPos { x: 0, y: 70, z: 0 }, None));
    assert_eq!(world.tnt_count(), 0);
}

#[test]
fn the_first_world_tick_announces_the_tnt_and_plays_the_prime_sound_nearby() {
    let mut world = World::new("announce");
    world.platform(3);
    let near = world.bus.subscribe(1);
    near.update_presence(presence_at(0.5, f64::from(Y + 1), 3.5));
    let far = world.bus.subscribe(2);
    far.update_presence(presence_at(500.0, f64::from(Y + 1), 0.5));
    prime(&world, 0, 0);
    world.tick();

    let near_ids = packet_ids(&drain(&near));
    assert!(near_ids.contains(&CLIENTBOUND_ADD_ENTITY_PACKET_ID), "{near_ids:?}");
    assert!(near_ids.contains(&CLIENTBOUND_SOUND_PACKET_ID), "{near_ids:?}");
    let far_ids = packet_ids(&drain(&far));
    assert!(far_ids.contains(&CLIENTBOUND_ADD_ENTITY_PACKET_ID));
    assert!(!far_ids.contains(&CLIENTBOUND_SOUND_PACKET_ID), "16-block sound radius");
    let store = lock_status_mutex(&world.items);
    assert!(store.primed_tnts[0].announced);
    assert_eq!(store.primed_tnts[0].fuse, 79);
}

#[test]
fn a_burning_tnt_syncs_its_fuse_every_tick() {
    let mut world = World::new("sync");
    world.platform(3);
    let watcher = world.bus.subscribe(1);
    prime(&world, 0, 0);
    world.tick();
    drain(&watcher);
    world.tick();
    let ids = packet_ids(&drain(&watcher));
    assert!(
        ids.contains(&crate::network::play::CLIENTBOUND_SET_ENTITY_DATA_PACKET_ID),
        "fuse metadata each tick: {ids:?}"
    );
}

#[test]
fn the_tnt_falls_settles_and_explodes_after_exactly_eighty_ticks() {
    let mut world = World::new("fuse");
    world.platform(4);
    prime(&world, 0, 0);
    for tick in 1..80 {
        world.tick();
        assert_eq!(world.tnt_count(), 1, "still burning after {tick} ticks");
        assert_eq!(world.block(0, Y, 0), "minecraft:stone", "no explosion before the fuse ends");
    }
    world.tick();
    assert_eq!(world.tnt_count(), 0, "the entity is discarded on the 80th tick");
    assert_eq!(world.block(0, Y, 0), "minecraft:air", "the blast dug the platform");
}

#[test]
fn detonation_removes_the_entity_and_notifies_nearby_players_with_the_explode_packet() {
    let mut world = World::new("notify");
    world.platform(4);
    let player = world.bus.subscribe(1);
    player.update_presence(presence_at(0.5, f64::from(Y + 1), 30.0));
    prime(&world, 0, 0);
    lock_status_mutex(&world.items).primed_tnts[0].fuse = 1;
    world.tick();
    let ids = packet_ids(&drain(&player));
    assert!(ids.contains(&CLIENTBOUND_REMOVE_ENTITIES_PACKET_ID), "{ids:?}");
    assert!(ids.contains(&CLIENTBOUND_EXPLODE_PACKET_ID), "{ids:?}");
    // The block updates precede the explode packet.
    let explode_at = ids.iter().position(|id| *id == CLIENTBOUND_EXPLODE_PACKET_ID).unwrap();
    let block_update_at = ids
        .iter()
        .position(|id| *id == crate::network::play::CLIENTBOUND_BLOCK_UPDATE_PACKET_ID)
        .unwrap();
    assert!(block_update_at < explode_at, "{ids:?}");
}

#[test]
fn players_beyond_64_blocks_get_no_explode_packet() {
    let mut world = World::new("far");
    world.platform(4);
    let player = world.bus.subscribe(1);
    player.update_presence(presence_at(0.5, f64::from(Y + 1), 100.0));
    prime(&world, 0, 0);
    lock_status_mutex(&world.items).primed_tnts[0].fuse = 1;
    world.tick();
    assert!(!packet_ids(&drain(&player)).contains(&CLIENTBOUND_EXPLODE_PACKET_ID));
}

#[test]
fn nothing_explodes_when_tnt_explodes_is_off_but_the_entity_is_still_discarded() {
    let mut world = World::new("rule-off");
    world.platform(4);
    prime(&world, 0, 0);
    lock_status_mutex(&world.items).primed_tnts[0].fuse = 1;
    world.rules.tnt_explodes = false;
    world.tick();
    assert_eq!(world.tnt_count(), 0);
    assert_eq!(world.block(0, Y, 0), "minecraft:stone");
}

#[test]
fn a_tnt_block_caught_in_the_blast_becomes_a_short_fused_primed_tnt() {
    let mut world = World::new("chain");
    world.platform(6);
    world.set(2, Y + 1, 0, "minecraft:tnt");
    world.set(-2, Y + 1, 0, "minecraft:tnt");
    prime(&world, 0, 0);
    lock_status_mutex(&world.items).primed_tnts[0].fuse = 1;
    world.tick();
    assert_eq!(world.block(2, Y + 1, 0), "minecraft:air");
    assert_eq!(world.block(-2, Y + 1, 0), "minecraft:air");
    let store = lock_status_mutex(&world.items);
    assert_eq!(store.primed_tnts.len(), 2, "each TNT block became an entity");
    for tnt in &store.primed_tnts {
        // wasExploded: nextInt(80 / 4) + 80 / 8 = 10..29 (announced this tick,
        // first ticked on the next one).
        assert!((10..30).contains(&tnt.fuse), "fuse {}", tnt.fuse);
        assert!(tnt.announced);
    }
    assert!(store.entities.iter().all(|item| item.item != "minecraft:tnt"), "TNT never drops from explosions");
}

#[test]
fn chain_reactions_propagate_through_successive_detonations() {
    let mut world = World::new("chain-deep");
    world.platform(12);
    world.set(3, Y + 1, 0, "minecraft:tnt");
    world.set(6, Y + 1, 0, "minecraft:tnt");
    prime(&world, 0, 0);
    lock_status_mutex(&world.items).primed_tnts[0].fuse = 1;
    for _ in 0..120 {
        world.tick();
    }
    assert_eq!(world.tnt_count(), 0, "all chained TNT eventually detonated");
    assert_eq!(world.block(3, Y + 1, 0), "minecraft:air");
    assert_eq!(world.block(6, Y + 1, 0), "minecraft:air");
}

#[test]
fn blocks_drop_their_loot_as_item_entities() {
    let mut world = World::new("drops");
    world.platform(4);
    prime(&world, 0, 0);
    lock_status_mutex(&world.items).primed_tnts[0].fuse = 1;
    world.tick();
    let store = lock_status_mutex(&world.items);
    let cobblestone: i32 = store
        .entities
        .iter()
        .filter(|item| item.item == "minecraft:cobblestone")
        .map(|item| item.count)
        .sum();
    assert!(cobblestone > 0, "destroyed stone drops cobblestone");
    assert!(
        store.entities.iter().all(|item| item.count <= 16),
        "explosion drops combine into stacks of at most 16"
    );
}

#[test]
fn the_block_drops_rule_suppresses_the_item_entities() {
    let mut world = World::new("no-drops");
    world.platform(4);
    world.rules.block_drops = false;
    prime(&world, 0, 0);
    lock_status_mutex(&world.items).primed_tnts[0].fuse = 1;
    world.tick();
    assert!(lock_status_mutex(&world.items).entities.is_empty());
    assert_eq!(world.block(0, Y, 0), "minecraft:air", "blocks are still destroyed");
}

#[test]
fn a_nearby_player_gets_an_explosion_hit_and_knockback() {
    let mut world = World::new("hit");
    world.platform(8);
    let player = world.bus.subscribe(1);
    player.update_presence(presence_at(3.5, f64::from(Y + 1), 0.5));
    prime(&world, 0, 0);
    lock_status_mutex(&world.items).primed_tnts[0].fuse = 1;
    world.tick();
    let hits = player.take_explosion_hits();
    assert_eq!(hits.len(), 1);
    assert!(hits[0].damage > 5.0, "3 blocks from a power-4 blast: {}", hits[0].damage);
    assert!(hits[0].direct.is_some() && hits[0].causing.is_none());
    let frames = drain(&player);
    // The explode packet carries the knockback: payload = id + centre(24) +
    // radius(4) + block count(4), then the optional flag.
    let (payload, _) = find_packet(&frames, CLIENTBOUND_EXPLODE_PACKET_ID);
    assert_eq!(payload[1 + 32], 1, "knockback present");
}

#[test]
fn creative_flying_players_are_hurt_but_not_knocked_back() {
    let mut world = World::new("creative");
    world.platform(8);
    let player = world.bus.subscribe(1);
    let mut presence = presence_at(3.5, f64::from(Y + 1), 0.5);
    presence.creative = true;
    presence.flying = true;
    player.update_presence(presence);
    prime(&world, 0, 0);
    lock_status_mutex(&world.items).primed_tnts[0].fuse = 1;
    world.tick();
    assert_eq!(player.take_explosion_hits().len(), 1);
    let (payload, _) = find_packet(&drain(&player), CLIENTBOUND_EXPLODE_PACKET_ID);
    assert_eq!(payload[1 + 32], 0, "no knockback for a flying creative player");
}

#[test]
fn spectators_are_ignored_by_the_blast() {
    let mut world = World::new("spectator");
    world.platform(8);
    let player = world.bus.subscribe(1);
    let mut presence = presence_at(3.5, f64::from(Y + 1), 0.5);
    presence.spectator = true;
    player.update_presence(presence);
    prime(&world, 0, 0);
    lock_status_mutex(&world.items).primed_tnts[0].fuse = 1;
    world.tick();
    assert!(player.take_explosion_hits().is_empty());
}

#[test]
fn a_wall_between_the_blast_and_the_player_shields_them() {
    let mut world = World::new("shield");
    world.platform(8);
    for y in Y + 1..Y + 6 {
        for z in -2..=2 {
            world.set(6, y, z, "minecraft:obsidian");
        }
    }
    let player = world.bus.subscribe(1);
    player.update_presence(presence_at(7.5, f64::from(Y + 1), 0.5));
    prime(&world, 0, 0);
    lock_status_mutex(&world.items).primed_tnts[0].fuse = 1;
    world.tick();
    // Exposure 0 leaves only the `+ 1.0` base of the damage formula.
    let hits = player.take_explosion_hits();
    assert!(hits.iter().all(|hit| hit.damage < 2.0), "{hits:?}");
}

#[test]
fn dropped_items_in_range_are_destroyed_or_pushed() {
    let mut world = World::new("items");
    world.platform(8);
    let item = |id: i32, x: f64, name: &'static str| crate::item_entity::DroppedItem {
        entity_id: id,
        item: name,
        count: 1,
        x,
        y: f64::from(Y + 1),
        z: 0.5,
        vel_x: 0.0,
        vel_y: 0.0,
        vel_z: 0.0,
        pickup_delay: 0,
        age: 0,
        target_uuid: None,
        health: crate::item_entity::ITEM_DEFAULT_HEALTH,
    };
    {
        let mut store = lock_status_mutex(&world.items);
        store.entities.push(item(9001, 1.5, "minecraft:apple"));
        store.entities.push(item(9002, 1.5, "minecraft:nether_star"));
        store.entities.push(item(9003, 8.0, "minecraft:apple"));
    }
    prime(&world, 0, 0);
    lock_status_mutex(&world.items).primed_tnts[0].fuse = 1;
    world.tick();
    let store = lock_status_mutex(&world.items);
    assert!(!store.entities.iter().any(|item| item.entity_id == 9001), "close item destroyed");
    let star = store.entities.iter().find(|item| item.entity_id == 9002).expect("nether star resists");
    assert!(star.vel_x > 0.0, "pushed away from the blast");
    let far = store.entities.iter().find(|item| item.entity_id == 9003).expect("far item survives");
    // dist 0.94: damage 2.86 wears the 5 health down to 2 without killing it.
    assert_eq!(far.health, 2);
}

#[test]
fn a_neighbouring_primed_tnt_is_pushed_by_the_blast() {
    let mut world = World::new("push");
    world.platform(8);
    prime(&world, 0, 0);
    prime(&world, 3, 0);
    {
        let mut store = lock_status_mutex(&world.items);
        store.primed_tnts[0].fuse = 1;
        store.primed_tnts[1].delta = crate::network::play::Vec3::ZERO;
    }
    world.tick();
    let store = lock_status_mutex(&world.items);
    assert_eq!(store.primed_tnts.len(), 1);
    assert!(store.primed_tnts[0].delta.x > 0.1, "shoved away: {:?}", store.primed_tnts[0].delta);
}

#[test]
fn burning_tnt_next_to_fire_is_primed_instead_of_vanishing() {
    // FireBlock.checkBurnOut: the burnt TNT block turns into a PrimedTnt.
    let world = World::new("fire-prime");
    world.platform(4);
    let tnt = BlockPos { x: 1, y: Y + 1, z: 0 };
    let fire = BlockPos { x: 0, y: Y + 1, z: 0 };
    let mut blocks = LiveBlockTicks::new();
    let mut sink = Vec::new();
    let environment = FireEnvironment {
        raining: false,
        difficulty_id: 2,
        spread_radius: -1,
    };
    let mut primed = false;
    for game_time in 0..20_000 {
        world.set(1, Y + 1, 0, "minecraft:tnt");
        world.set(0, Y + 1, 0, "minecraft:fire[age=0,east=false,north=false,south=false,up=false,west=false]");
        let state = read_live_block_model_at(&world.cache, &world.layout, 42, fire);
        super::fire_live::run_live_fire_tick(
            &mut sink,
            CompressionState::disabled(),
            super::block_placement_live::LiveBlockWorld {
                layout: &world.layout,
                seed: 42,
                cache: &world.cache,
            },
            &mut blocks,
            game_time,
            environment,
            &world.items,
            &[],
            state,
            fire,
        )
        .unwrap();
        if world.tnt_count() > 0 {
            primed = true;
            break;
        }
    }
    assert!(primed, "fire eventually burns the TNT");
    let store = lock_status_mutex(&world.items);
    assert_eq!(
        (store.primed_tnts[0].pos.x, store.primed_tnts[0].pos.z),
        (f64::from(tnt.x) + 0.5, f64::from(tnt.z) + 0.5)
    );
    drop(store);
    assert_ne!(world.block(1, Y + 1, 0), "minecraft:tnt", "the burnt block is gone");
}

/// Finds the first packet with `id`, returning its payload (id included).
fn find_packet(frames: &[u8], id: i32) -> (Vec<u8>, usize) {
    let mut rest = frames;
    let mut index = 0;
    while !rest.is_empty() {
        let length = read_var_i32(&mut rest).unwrap() as usize;
        let (payload, tail) = rest.split_at(length);
        let mut peek = payload;
        if read_var_i32(&mut peek).unwrap() == id {
            return (payload.to_vec(), index);
        }
        rest = tail;
        index += 1;
    }
    panic!("packet {id} not found");
}
