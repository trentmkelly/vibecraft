//! Live `ServerLevel.explode`: [`ServerExplosion`](crate::server_explosion)
//! applied to the running level.
//!
//! The shape follows Java `ServerExplosion.explode()`: calculate the exploded
//! positions, hurt entities, destroy blocks (collecting drops), create fire,
//! then send `ClientboundExplodePacket` to the players within 64 blocks.
//! World changes are written to a frame buffer that the shared world tick
//! publishes on the [`WorldPacketBus`]; players are hurt through
//! [`ExplosionHit`]s applied by their own session
//! (`player_explosion_live`).
//!
//! Entities modelled: players, primed TNT, dropped items and experience
//! orbs. TODO(explosion-mobs): mobs, falling blocks, boats/minecarts and the
//! other `Entity` kinds join `hurtEntities` when their live models exist.
//! TODO(game-events-live): `level.gameEvent(source, EXPLODE, center)` (sculk
//! vibration) has no live game-event bus.
//! TODO(explosion-block-triggers): the `TRIGGER_BLOCK` behaviours (bell, door,
//! button, lever, ... `onExplosionHit` overrides) have no live source yet.
//! TODO(block-break-experience): `spawnAfterBreak` ore experience is not
//! dropped by any live block-removal path.
//! TODO(block-entity-contents-drop): container contents are not scattered when
//! their block is removed (also true of player block breaking).

use std::io::{self, Write};
use std::sync::{Arc, Mutex};

use super::block_placement_live::{
    run_live_shape_cascade, write_block_update, LiveBlockWorld, LiveCascade,
};
use super::chunk_d_2::{evaluate_block_loot_in_context, write_item_entity_spawn_packets};
use std::sync::MutexGuard;

use crate::random_source::LegacyRandom;
use super::*;
use crate::block_behavior::BlockStateModel;
use crate::block_survival::SurvivalWorld;
use crate::block_update::BlockPos;
use crate::collision_shape::Aabb;
use crate::damage_type::DamageEntityRef;
use crate::entity_collision::aabb_intersects;
use crate::entity_physics::ExplosionBlockInteraction;
use crate::fire_block::fire_tick_delay;
use crate::item_entity::{DroppedItem, DEFAULT_PICKUP_DELAY};
use crate::live_block_entities::registry_ids::{particle_type_protocol_id, sound_event_protocol_id};
use crate::network::play::{
    ClientboundExplodePacket, ClientboundSetEntityMotionPacket, ExplosionParticleInfo,
    RawParticleOptions, SoundEventHolder, Vec3, WeightedExplosionParticle,
    CLIENTBOUND_EXPLODE_PACKET_ID, CLIENTBOUND_SET_ENTITY_MOTION_PACKET_ID,
};
use crate::network::world_broadcast::ExplosionHit;
use crate::primed_tnt::PrimedTntEntity;
use crate::server_explosion::{
    add_or_append_stack, calculate_exploded, entity_impact, explosion_distance, get_seen_percent,
    interaction_affects_blocklike_entities, shuffle, BlockCalculator, CollectedStack,
    EntityImpact, EntityTarget, ExplosionRandom, ExplosionRules, LevelExplosionInteraction,
};

/// `ServerLevel.explode` only notifies players closer than this
/// (`player.distanceToSqr(center) < 4096.0`).
const EXPLOSION_NOTIFY_DISTANCE_SQR: f64 = 4096.0;
/// `Level.setBlock` flags 3: `UPDATE_NEIGHBORS | UPDATE_CLIENTS`.
const ITEM_HALF_HEIGHT: f64 = 0.125;
/// `EntityType.ITEM` `sized(0.25F, 0.25F)`.
const ITEM_SIZE: f64 = 0.25;
/// `EntityType.ITEM` `eyeHeight(0.2125F)`.
const ITEM_EYE_HEIGHT: f64 = 0.2125_f32 as f64;
/// `EntityType.EXPERIENCE_ORB` `sized(0.5F, 0.5F)`.
const ORB_SIZE: f64 = 0.5;
/// `EntityDimensions` default eye height: `height * 0.85F`.
const ORB_EYE_HEIGHT: f64 = (0.5_f32 * 0.85_f32) as f64;

impl ExplosionRandom for LegacyRandom {
    fn next_float(&mut self) -> f32 {
        self.next_f32()
    }

    fn next_int(&mut self, bound: i32) -> i32 {
        self.next_i32_bound(bound)
    }

    fn next_double(&mut self) -> f64 {
        self.next_f64()
    }
}

impl crate::fire_block::FireRandom for LegacyRandom {
    fn next_int(&mut self, bound: i32) -> i32 {
        self.next_i32_bound(bound)
    }

    fn next_float(&mut self) -> f32 {
        self.next_f32()
    }
}

/// The level state an explosion reads and mutates.
pub(super) struct ExplosionEnv<'a> {
    pub layout: &'a WorldLayout,
    pub seed: i64,
    pub cache: &'a GeneratedChunkCache,
    pub world_items: &'a Arc<Mutex<WorldItemEntities>>,
    pub bus: &'a WorldPacketBus,
    pub rules: ExplosionRules,
    pub game_time: i64,
    /// Java `DedicatedServerProperties.maxChainedNeighborUpdates`.
    pub max_chained_neighbor_updates: i32,
}

/// Packets addressed to individual connections (`ServerPlayer.connection.send`),
/// published after the tick's broadcast frames so clients see the block and
/// entity changes before the explosion effects that describe them.
#[derive(Default)]
pub(super) struct DirectedPackets(Vec<(u64, Vec<u8>)>);

impl DirectedPackets {
    /// Queues `payload` (`VarInt id` + body) for the connection `token`.
    pub(super) fn push(&mut self, token: u64, payload: Vec<u8>) {
        self.0.push((token, payload));
    }

    /// Delivers everything queued through the bus.
    pub(super) fn publish(self, bus: &WorldPacketBus) {
        for (token, payload) in self.0 {
            bus.publish_to(token, &payload);
        }
    }
}

/// What an explosion writes its results to besides the frame buffer: the
/// scheduled-tick queues the block changes feed into and the per-connection
/// packet queue.
pub(super) struct ExplosionSinks<'a> {
    pub block: &'a mut LiveBlockTicks,
    pub fluid: &'a mut LiveFluidTicks,
    pub packets: &'a mut DirectedPackets,
}

/// The exploding entity (`ServerExplosion.source`).
#[derive(Clone, Copy, Debug)]
pub(super) struct ExplosionSource {
    pub entity_id: i32,
    /// `Entity.position()`, the damage source position.
    pub position: Vec3,
    /// `Explosion.getIndirectSourceEntity`: the responsible living entity.
    pub owner: Option<DamageEntityRef>,
}

/// The arguments of `ServerLevel.explode`.
pub(super) struct ExplosionRequest {
    pub source: Option<ExplosionSource>,
    pub center: Vec3,
    pub radius: f32,
    pub fire: bool,
    pub interaction: LevelExplosionInteraction,
    pub calculator: BlockCalculator,
}

/// One `ServerExplosion` in flight.
struct ServerExplosion<'a, 'b> {
    env: &'a ExplosionEnv<'a>,
    ticks: &'a mut ExplosionSinks<'b>,
    tnts: &'a mut Vec<PrimedTntEntity>,
    request: &'a ExplosionRequest,
    block_interaction: ExplosionBlockInteraction,
    /// `Level.random`, locked for the duration of the explosion.
    random: MutexGuard<'a, LegacyRandom>,
    /// `ServerExplosion.hitPlayers`: session token -> knockback.
    hit_players: Vec<(u64, Vec3)>,
    /// Positions whose neighbours need the shape-update cascade.
    changed: Vec<BlockPos>,
}

/// `ServerLevel.explode(source, damageSource, calculator, x, y, z, r, fire,
/// interactionType, ..)` with the default particles and sound.
pub(super) fn explode<W: Write>(
    writer: &mut W,
    env: &ExplosionEnv<'_>,
    ticks: &mut ExplosionSinks<'_>,
    tnts: &mut Vec<PrimedTntEntity>,
    request: &ExplosionRequest,
) -> io::Result<()> {
    let mut explosion = ServerExplosion {
        env,
        ticks,
        tnts,
        request,
        block_interaction: request.interaction.block_interaction(&env.rules),
        random: env.cache.level_random.lock(),
        hit_players: Vec::new(),
        changed: Vec::new(),
    };
    let block_count = explosion.explode(writer)?;
    explosion.send_explode_packets(block_count)
}

impl<'a> ServerExplosion<'a, '_> {
    fn world(&self) -> LiveBlockWorld<'a> {
        LiveBlockWorld {
            layout: self.env.layout,
            seed: self.env.seed,
            cache: self.env.cache,
        }
    }

    /// `ServerExplosion.explode()`; returns the number of exploded positions.
    fn explode<W: Write>(&mut self, writer: &mut W) -> io::Result<i32> {
        let request = self.request;
        // TODO(game-events-live): `level.gameEvent(source, EXPLODE, center)`.
        let mut to_blow = calculate_exploded(
            &self.world(),
            request.center,
            request.radius,
            request.calculator,
            &mut *self.random,
        );
        self.hurt_entities(writer)?;
        if self.block_interaction != ExplosionBlockInteraction::Keep {
            self.interact_with_blocks(writer, &mut to_blow)?;
        }
        if request.fire {
            self.create_fire(writer, &to_blow)?;
        }
        Ok(to_blow.len() as i32)
    }

    /// `ServerExplosion.shouldAffectBlocklikeEntities()` for a source that is
    /// not a wind charge.
    fn should_affect_blocklike_entities(&self) -> bool {
        if self.env.rules.mob_griefing {
            true
        } else {
            interaction_affects_blocklike_entities(self.block_interaction)
        }
    }

    /// The `DamageSource` refs of the exploding entity and its owner.
    fn damage_refs(&self) -> (Option<DamageEntityRef>, Option<DamageEntityRef>) {
        let source = self.request.source;
        (
            source.map(|source| DamageEntityRef::non_living(source.entity_id)),
            source.and_then(|source| source.owner),
        )
    }

    /// The exposure and impact on an entity, or `None` when out of range
    /// (`dist > 1`). `exposure` is only computed for entities in range.
    fn impact_on(&self, target: &EntityTarget, bb: &Aabb) -> Option<EntityImpact> {
        let request = self.request;
        if explosion_distance(request.center, request.radius, target.position) > 1.0 {
            return None;
        }
        let exposure = get_seen_percent(&self.world(), request.center, bb);
        entity_impact(request.center, request.radius, target, exposure, 1.0)
    }

    /// `ServerExplosion.hurtEntities()`.
    fn hurt_entities<W: Write>(&mut self, writer: &mut W) -> io::Result<()> {
        let request = self.request;
        if request.radius < 1.0E-5 {
            return Ok(());
        }
        let double_radius = f64::from(request.radius * 2.0);
        let c = request.center;
        let region = Aabb::new(
            (c.x - double_radius - 1.0).floor(),
            (c.y - double_radius - 1.0).floor(),
            (c.z - double_radius - 1.0).floor(),
            (c.x + double_radius + 1.0).floor(),
            (c.y + double_radius + 1.0).floor(),
            (c.z + double_radius + 1.0).floor(),
        );
        self.hurt_players(&region);
        self.hurt_primed_tnts(&region);
        self.hurt_items(writer, &region)?;
        self.hurt_experience_orbs(writer, &region)
    }

    /// Players: queue the damage for their session and remember the
    /// knockback for the `ClientboundExplodePacket`.
    fn hurt_players(&mut self, region: &Aabb) {
        let (direct, causing) = self.damage_refs();
        let source_position = self
            .request
            .source
            .map_or([self.request.center.x, self.request.center.y, self.request.center.z], |source| {
                [source.position.x, source.position.y, source.position.z]
            });
        for (token, player) in self.env.bus.players() {
            // `Level.getEntities` uses `EntitySelector.NO_SPECTATORS`.
            if player.spectator {
                continue;
            }
            let [x, y, z] = player.position;
            let half = player.width / 2.0;
            let bb = Aabb::new(x - half, y, z - half, x + half, y + player.height, z + half);
            if !aabb_intersects(&bb, region) {
                continue;
            }
            let target = EntityTarget {
                position: Vec3 { x, y, z },
                origin: Vec3 {
                    x,
                    y: y + player.eye_height,
                    z,
                },
                knockback_resistance: player.explosion_knockback_resistance,
            };
            let Some(impact) = self.impact_on(&target, &bb) else {
                continue;
            };
            self.env.bus.push_explosion_hit(
                token,
                ExplosionHit {
                    damage: impact.damage,
                    direct,
                    causing,
                    causing_type: causing
                        .filter(|owner| owner.is_player)
                        .map(|_| "minecraft:player"),
                    source_position,
                },
            );
            if !player.spectator && (!player.creative || !player.flying) {
                self.hit_players.push((token, impact.knockback));
            }
        }
    }

    /// Other primed TNT is pushed (`PrimedTnt.hurtServer` is always false).
    fn hurt_primed_tnts(&mut self, region: &Aabb) {
        let source_id = self.request.source.map(|source| source.entity_id);
        for index in 0..self.tnts.len() {
            let tnt = &self.tnts[index];
            if Some(tnt.entity_id) == source_id || !aabb_intersects(&tnt.bounding_box(), region) {
                continue;
            }
            // `entity instanceof PrimedTnt ? entity.position() : getEyePosition()`.
            let target = EntityTarget {
                position: tnt.pos,
                origin: tnt.pos,
                knockback_resistance: 0.0,
            };
            let bb = tnt.bounding_box();
            if let Some(impact) = self.impact_on(&target, &bb) {
                self.tnts[index].push(impact.knockback);
            }
        }
    }

    /// `ItemEntity.hurtServer` + `push`.
    fn hurt_items<W: Write>(&mut self, writer: &mut W, region: &Aabb) -> io::Result<()> {
        // `ItemEntity.ignoreExplosion`.
        if !self.should_affect_blocklike_entities() {
            return Ok(());
        }
        let mut store = lock_status_mutex(self.env.world_items);
        let mut removed = Vec::new();
        let mut pushed = Vec::new();
        for item in &mut store.entities {
            let half = ITEM_SIZE / 2.0;
            let bb = Aabb::new(item.x - half, item.y, item.z - half, item.x + half, item.y + ITEM_SIZE, item.z + half);
            if !aabb_intersects(&bb, region) {
                continue;
            }
            let position = Vec3 {
                x: item.x,
                y: item.y,
                z: item.z,
            };
            let target = EntityTarget {
                position,
                origin: Vec3 {
                    y: item.y + ITEM_EYE_HEIGHT,
                    ..position
                },
                knockback_resistance: 0.0,
            };
            let Some(impact) = self.impact_on(&target, &bb) else {
                continue;
            };
            // `ItemStack.canBeHurtBy`: the nether star is `DamageResistant(is_explosion)`.
            if item.item != "minecraft:nether_star" {
                item.health = (item.health as f32 - impact.damage) as i32;
                if item.health <= 0 {
                    removed.push(item.entity_id);
                    continue;
                }
            }
            item.vel_x += impact.knockback.x;
            item.vel_y += impact.knockback.y;
            item.vel_z += impact.knockback.z;
            pushed.push((item.entity_id, Vec3 { x: item.vel_x, y: item.vel_y, z: item.vel_z }));
        }
        store.entities.retain(|item| !removed.contains(&item.entity_id));
        drop(store);
        write_removed_and_pushed(writer, &removed, &pushed)
    }

    /// `ExperienceOrb.hurtServer` + `push`.
    fn hurt_experience_orbs<W: Write>(&mut self, writer: &mut W, region: &Aabb) -> io::Result<()> {
        let mut store = lock_status_mutex(self.env.world_items);
        let mut removed = Vec::new();
        let mut pushed = Vec::new();
        for orb in &mut store.xp_orbs {
            let half = ORB_SIZE / 2.0;
            let bb = Aabb::new(orb.x - half, orb.y, orb.z - half, orb.x + half, orb.y + ORB_SIZE, orb.z + half);
            if !aabb_intersects(&bb, region) {
                continue;
            }
            let position = Vec3 {
                x: orb.x,
                y: orb.y,
                z: orb.z,
            };
            let target = EntityTarget {
                position,
                origin: Vec3 {
                    y: orb.y + ORB_EYE_HEIGHT,
                    ..position
                },
                knockback_resistance: 0.0,
            };
            let Some(impact) = self.impact_on(&target, &bb) else {
                continue;
            };
            orb.orb.health = (orb.orb.health as f32 - impact.damage) as i32;
            if orb.orb.health <= 0 {
                orb.orb.removed = true;
                removed.push(orb.entity_id());
                continue;
            }
            orb.vel_x += impact.knockback.x;
            orb.vel_y += impact.knockback.y;
            orb.vel_z += impact.knockback.z;
            pushed.push((orb.entity_id(), Vec3 { x: orb.vel_x, y: orb.vel_y, z: orb.vel_z }));
        }
        store.xp_orbs.retain(|orb| !orb.orb.removed);
        drop(store);
        write_removed_and_pushed(writer, &removed, &pushed)
    }

    /// `ServerExplosion.interactWithBlocks`.
    fn interact_with_blocks<W: Write>(
        &mut self,
        writer: &mut W,
        to_blow: &mut [BlockPos],
    ) -> io::Result<()> {
        let mut stacks: Vec<CollectedStack> = Vec::new();
        shuffle(to_blow, &mut *self.random);
        for pos in to_blow.iter() {
            let state = self.world().state_at(*pos);
            self.on_explosion_hit(writer, &state, *pos, &mut stacks)?;
        }
        self.flush_block_changes(writer)?;
        for stack in stacks {
            self.pop_resource(writer, &stack)?;
        }
        Ok(())
    }

    /// `BlockBehaviour.onExplosionHit` and `Block.wasExploded`.
    fn on_explosion_hit<W: Write>(
        &mut self,
        writer: &mut W,
        state: &BlockStateModel,
        pos: BlockPos,
        stacks: &mut Vec<CollectedStack>,
    ) -> io::Result<()> {
        if state.is_air() || self.block_interaction == ExplosionBlockInteraction::TriggerBlock {
            return Ok(());
        }
        let is_tnt = state.registry_id == "minecraft:tnt";
        // `TntBlock.dropFromExplosion` is false; every other block drops.
        if !is_tnt {
            let radius = (self.block_interaction == ExplosionBlockInteraction::DestroyWithDecay)
                .then_some(self.request.radius);
            let seed = (u64::from(self.random.next_int(i32::MAX) as u32) << 32)
                | u64::from(self.random.next_int(i32::MAX) as u32);
            for (item, count) in
                evaluate_block_loot_in_context(&state.state_name(), seed, None, true, radius)
            {
                add_or_append_stack(stacks, item, count, pos);
            }
        }
        // `level.setBlock(pos, AIR, 3)`.
        self.env
            .cache
            .set_block(self.env.layout.root(), self.env.seed, pos, "minecraft:air");
        write_block_update(writer, CompressionState::disabled(), pos, 0)?;
        self.changed.push(pos);
        if is_tnt && self.env.rules.tnt_explodes {
            self.tnt_was_exploded(pos);
        }
        Ok(())
    }

    /// `TntBlock.wasExploded`: a short-fused `PrimedTnt` owned by the
    /// explosion's responsible entity.
    fn tnt_was_exploded(&mut self, pos: BlockPos) {
        let owner = self.request.source.and_then(|source| source.owner);
        let entity_id = lock_status_mutex(self.env.world_items).alloc_entity_id();
        let angle = self.random.next_double();
        let mut tnt = PrimedTntEntity::new(
            entity_id,
            (f64::from(pos.x) + 0.5, f64::from(pos.y), f64::from(pos.z) + 0.5),
            owner,
            angle,
        );
        let fuse = tnt.fuse;
        tnt.fuse = self.random.next_int(fuse / 4) + fuse / 8;
        self.tnts.push(tnt);
    }

    /// The `updateNeighborShapes` cascade for the positions changed so far.
    fn flush_block_changes<W: Write>(&mut self, writer: &mut W) -> io::Result<()> {
        let changed = std::mem::take(&mut self.changed);
        if changed.is_empty() {
            return Ok(());
        }
        let mut cascade = LiveCascade {
            layout: self.env.layout,
            seed: self.env.seed,
            cache: self.env.cache,
            fluid_ticks: &mut *self.ticks.fluid,
            block_ticks: &mut *self.ticks.block,
            game_time: self.env.game_time,
            random_roll: (self.env.game_time as i32).rem_euclid(40),
            max_chained_neighbor_updates: self.env.max_chained_neighbor_updates,
        };
        run_live_shape_cascade(writer, CompressionState::disabled(), &mut cascade, changed)
    }

    /// `Block.popResource(level, pos, stack)`.
    fn pop_resource<W: Write>(&mut self, writer: &mut W, stack: &CollectedStack) -> io::Result<()> {
        if !self.env.rules.block_drops || stack.count <= 0 {
            return Ok(());
        }
        let Some(item_id) = item_protocol_id(stack.item) else {
            return Ok(());
        };
        let jitter = |random: &mut LegacyRandom| -0.25 + random.next_double() * 0.5;
        let x = f64::from(stack.pos.x) + 0.5 + jitter(&mut self.random);
        let y = f64::from(stack.pos.y) + 0.5 + jitter(&mut self.random) - ITEM_HALF_HEIGHT;
        let z = f64::from(stack.pos.z) + 0.5 + jitter(&mut self.random);
        // `ItemEntity(level, x, y, z, stack)`: random yaw, then the motion.
        let _yaw = self.random.next_float();
        let vel_x = self.random.next_double() * 0.2 - 0.1;
        let vel_z = self.random.next_double() * 0.2 - 0.1;
        let entity_id = lock_status_mutex(self.env.world_items).alloc_entity_id();
        let item = DroppedItem {
            entity_id,
            item: stack.item,
            count: stack.count,
            x,
            y,
            z,
            vel_x,
            vel_y: 0.2,
            vel_z,
            pickup_delay: DEFAULT_PICKUP_DELAY,
            age: 0,
            target_uuid: None,
            health: crate::item_entity::ITEM_DEFAULT_HEALTH,
        };
        write_item_entity_spawn_packets(writer, CompressionState::disabled(), &item, item_id)?;
        lock_status_mutex(self.env.world_items).entities.push(item);
        Ok(())
    }

    /// `ServerExplosion.createFire`.
    fn create_fire<W: Write>(&mut self, writer: &mut W, to_blow: &[BlockPos]) -> io::Result<()> {
        for pos in to_blow {
            if self.random.next_int(3) != 0 {
                continue;
            }
            let world = self.world();
            let below_solid = crate::block_properties::state_physics_by_name(
                &world.state_at(pos.relative(crate::block_update::Direction::Down)).state_name(),
            )
            .is_some_and(|physics| physics.is_solid_render);
            if !world.state_at(*pos).is_air() || !below_solid {
                continue;
            }
            // `BaseFireBlock.getState(level, pos)` then `setBlockAndUpdate`.
            let fire = crate::block_placement::connecting::base_fire_state(&world, *pos);
            let id = crate::block_states::network_id_for_block_state(&fire.state_name()).unwrap_or(0);
            self.env
                .cache
                .set_block(self.env.layout.root(), self.env.seed, *pos, &fire.state_name());
            write_block_update(writer, CompressionState::disabled(), *pos, id)?;
            self.changed.push(*pos);
            // `FireBlock.onPlace` schedules the first spread tick.
            if fire.registry_id == "minecraft:fire" {
                let delay = fire_tick_delay(&mut *self.random);
                self.ticks
                    .block
                    .schedule(self.env.game_time, *pos, "minecraft:fire", delay);
            }
        }
        self.flush_block_changes(writer)
    }

    /// The `ClientboundExplodePacket` loop of `ServerLevel.explode`.
    fn send_explode_packets(&mut self, block_count: i32) -> io::Result<()> {
        let request = self.request;
        let small = request.radius < 2.0 || self.block_interaction == ExplosionBlockInteraction::Keep;
        let particle_name = if small {
            "minecraft:explosion"
        } else {
            "minecraft:explosion_emitter"
        };
        let simple_particle = |name: &str| -> RawParticleOptions {
            RawParticleOptions {
                particle_id: particle_type_protocol_id(name).unwrap_or(0),
                data: Vec::new(),
            }
        };
        // `Level.DEFAULT_EXPLOSION_BLOCK_PARTICLES`.
        let block_particles = vec![
            WeightedExplosionParticle {
                value: ExplosionParticleInfo {
                    particle: simple_particle("minecraft:poof"),
                    scaling: 0.5,
                    speed: 1.0,
                },
                weight: 1,
            },
            WeightedExplosionParticle {
                value: ExplosionParticleInfo {
                    particle: simple_particle("minecraft:smoke"),
                    scaling: 1.0,
                    speed: 1.0,
                },
                weight: 1,
            },
        ];
        let sound = SoundEventHolder::Registered {
            id: sound_event_protocol_id("minecraft:entity.generic.explode").unwrap_or(0),
        };
        for (token, player) in self.env.bus.players() {
            let d = |axis: usize, c: f64| player.position[axis] - c;
            let distance_sqr = d(0, request.center.x).powi(2)
                + d(1, request.center.y).powi(2)
                + d(2, request.center.z).powi(2);
            if distance_sqr >= EXPLOSION_NOTIFY_DISTANCE_SQR {
                continue;
            }
            let packet = ClientboundExplodePacket {
                center: request.center,
                radius: request.radius,
                block_count,
                player_knockback: self
                    .hit_players
                    .iter()
                    .find(|(hit_token, _)| *hit_token == token)
                    .map(|(_, knockback)| *knockback),
                explosion_particle: simple_particle(particle_name),
                explosion_sound: sound.clone(),
                block_particles: block_particles.clone(),
            };
            let mut payload = Vec::new();
            write_var_i32(&mut payload, CLIENTBOUND_EXPLODE_PACKET_ID)?;
            packet.write(&mut payload)?;
            self.ticks.packets.push(token, payload);
        }
        Ok(())
    }
}

/// `ClientboundRemoveEntitiesPacket` for discarded entities and
/// `ClientboundSetEntityMotionPacket` for pushed ones (`ServerEntity` sends
/// the motion when `Entity.push` sets `needsSync`).
fn write_removed_and_pushed<W: Write>(
    writer: &mut W,
    removed: &[i32],
    pushed: &[(i32, Vec3)],
) -> io::Result<()> {
    let compression = CompressionState::disabled();
    if !removed.is_empty() {
        write_framed_packet_with_compression(
            writer,
            compression,
            CLIENTBOUND_REMOVE_ENTITIES_PACKET_ID,
            |p| {
                write_var_i32(p, removed.len() as i32)?;
                removed.iter().try_for_each(|id| write_var_i32(p, *id))
            },
        )?;
    }
    for (id, movement) in pushed {
        write_framed_packet_with_compression(
            writer,
            compression,
            CLIENTBOUND_SET_ENTITY_MOTION_PACKET_ID,
            |p| ClientboundSetEntityMotionPacket::new(*id, *movement).write(p),
        )?;
    }
    Ok(())
}
