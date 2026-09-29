//! Live `TntBlock` priming and `PrimedTnt` entities.
//!
//! * [`prime_tnt`] is `TntBlock.prime(level, pos, source)`: it adds a
//!   [`PrimedTntEntity`] to the shared entity store. The entity's spawn is
//!   announced by the next world tick ([`announce_pending_tnts`]), which
//!   mirrors `Level.addFreshEntity` + `ChunkMap.addEntity` tracking.
//! * [`tick_primed_tnts`] runs `PrimedTnt.tick` for every lit TNT from the
//!   shared world tick, synchronises it to clients like `ServerEntity` and
//!   detonates it through [`super::explosion_live::explode`].
//!
//! TODO(entity-tracking-range): Java sends an entity only to players inside
//! its tracking range (`EntityType.TNT` `clientTrackingRange(10)`) and pairs
//! it with players who come into range later (`ServerEntity.addPairing`); the
//! bus broadcasts the spawn once to every connection present at that moment.
//! TODO(tnt-persistence): primed TNT is not written to or restored from the
//! world's entity storage (`PrimedTnt.addAdditionalSaveData`), which is
//! blocked on the 26.1.2 entity storage layout.
//! TODO(game-events-live): `GameEvent.PRIME_FUSE` is not raised.

use std::io::{self, Write};
use std::sync::{Arc, Mutex};

use super::explosion_live::{
    explode, DirectedPackets, ExplosionEnv, ExplosionRequest, ExplosionSinks, ExplosionSource,
};
use super::*;
use crate::block_update::BlockPos;
use crate::damage_type::DamageEntityRef;
use crate::live_block_entities::registry_ids::{entity_type_protocol_id, sound_event_protocol_id};
use crate::network::codec::Uuid;
use crate::network::play::{
    AddEntityPacketInput, ClientboundAddEntityPacket, ClientboundEntityPositionSyncPacket,
    ClientboundMoveEntityPacket, ClientboundSetEntityDataPacket, ClientboundSetEntityMotionPacket,
    ClientboundSoundPacket, EntityDataValue, EntityMetadataValue, SoundEventHolder, SoundSource,
    Vec3, CLIENTBOUND_ENTITY_POSITION_SYNC_PACKET_ID, CLIENTBOUND_MOVE_ENTITY_POS_PACKET_ID,
    CLIENTBOUND_SET_ENTITY_MOTION_PACKET_ID, CLIENTBOUND_SOUND_PACKET_ID,
};
use crate::primed_tnt::{
    PrimedTntEntity, TntTick, DEFAULT_BLOCK_STATE, DEFAULT_FUSE_TIME, TNT_UPDATE_INTERVAL,
};
use crate::server_entity_sync::{EntitySync, PositionUpdate};
use crate::server_explosion::{BlockCalculator, LevelExplosionInteraction};

/// `PrimedTnt.DATA_FUSE_ID`: the first synched slot after the 8 base `Entity`
/// fields.
const DATA_FUSE_INDEX: u8 = 8;
/// `TntBlock.prime` plays `SoundEvents.TNT_PRIMED` for 16 blocks
/// (`volume 1.0`).
const PRIME_SOUND_RADIUS: f64 = 16.0;
/// `TntBlock.prime` spawns at `pos.getY()` (block-aligned) and `+0.5` on X/Z.
const CENTER_OFFSET: f64 = 0.5;

/// `TntBlock.prime(level, pos, source)`: lights the TNT at `pos`. Returns
/// whether the entity was created (`GameRules.TNT_EXPLODES`); the caller
/// removes the block when it was.
pub(super) fn prime_tnt(
    world_items: &Arc<Mutex<WorldItemEntities>>,
    level_random: &LevelRandomSource,
    pos: BlockPos,
    owner: Option<DamageEntityRef>,
) -> bool {
    let mut store = lock_status_mutex(world_items);
    if !store.explosion_rules.tnt_explodes {
        return false;
    }
    let entity_id = store.alloc_entity_id();
    let angle = level_random.lock().next_f64();
    let mut tnt = PrimedTntEntity::new(
        entity_id,
        (
            f64::from(pos.x) + CENTER_OFFSET,
            f64::from(pos.y),
            f64::from(pos.z) + CENTER_OFFSET,
        ),
        owner,
        angle,
    );
    tnt.play_prime_sound = true;
    store.primed_tnts.push(tnt);
    true
}

/// The TNT entity's network UUID (derived from the entity id like the other
/// live entities).
fn tnt_uuid(entity_id: i32) -> Uuid {
    let hi = (entity_id as u64).wrapping_mul(0x6C62_272E_07BB_0142);
    let lo = (entity_id as u64).wrapping_mul(0x62B8_2175_6295_C58D);
    let mut bytes = [0_u8; 16];
    bytes[..8].copy_from_slice(&hi.to_be_bytes());
    bytes[8..].copy_from_slice(&lo.to_be_bytes());
    Uuid(bytes)
}

fn fuse_data(entity_id: i32, fuse: i32) -> io::Result<ClientboundSetEntityDataPacket> {
    Ok(ClientboundSetEntityDataPacket {
        id: entity_id,
        packed_items: vec![EntityDataValue::typed(
            DATA_FUSE_INDEX,
            EntityMetadataValue::VarInt(fuse),
        )?],
    })
}

/// `ServerEntity.addPairing` for a TNT: the add-entity packet plus its
/// non-default synched data, in a bundle.
fn write_spawn<W: Write>(writer: &mut W, tnt: &PrimedTntEntity, sync: &EntitySync) -> io::Result<()> {
    let compression = CompressionState::disabled();
    let bundle = |writer: &mut W| {
        write_framed_packet_with_compression(
            writer,
            compression,
            CLIENTBOUND_BUNDLE_DELIMITER_PACKET_ID,
            |_| Ok(()),
        )
    };
    bundle(writer)?;
    let add = ClientboundAddEntityPacket::new(AddEntityPacketInput {
        id: tnt.entity_id,
        uuid: tnt_uuid(tnt.entity_id),
        entity_type: entity_type_protocol_id("minecraft:tnt").unwrap_or(0),
        position: sync.position_base(),
        movement: sync.last_sent_movement(),
        rotation: (0.0, 0.0),
        y_head_rot: 0.0,
        data: 0,
    });
    write_framed_packet_with_compression(writer, compression, CLIENTBOUND_ADD_ENTITY_PACKET_ID, |p| {
        add.write(p)
    })?;
    // `SynchedEntityData.getNonDefaultValues`: only a changed fuse is sent.
    if tnt.fuse != DEFAULT_FUSE_TIME {
        let data = fuse_data(tnt.entity_id, tnt.fuse)?;
        write_framed_packet_with_compression(
            writer,
            compression,
            CLIENTBOUND_SET_ENTITY_DATA_PACKET_ID,
            |p| data.write(p),
        )?;
    }
    debug_assert_eq!(tnt.block_state, DEFAULT_BLOCK_STATE);
    bundle(writer)
}

/// `SoundEvents.TNT_PRIMED` broadcast to players within 16 blocks
/// (`level.playSound(null, x, y, z, .., SoundSource.BLOCKS, 1.0F, 1.0F)`).
fn queue_prime_sound(
    bus: &WorldPacketBus,
    packets: &mut DirectedPackets,
    tnt: &PrimedTntEntity,
    seed: i64,
) -> io::Result<()> {
    let Some(id) = sound_event_protocol_id("minecraft:entity.tnt.primed") else {
        return Ok(());
    };
    let packet = ClientboundSoundPacket {
        sound: SoundEventHolder::Registered { id },
        source_id: SoundSource::Blocks as i32,
        position: tnt.pos,
        volume: 1.0,
        pitch: 1.0,
        seed,
        entity_id: None,
    };
    let mut payload = Vec::new();
    write_var_i32(&mut payload, CLIENTBOUND_SOUND_PACKET_ID)?;
    packet.write_position(&mut payload)?;
    for token in bus.tokens_within([tnt.pos.x, tnt.pos.y, tnt.pos.z], PRIME_SOUND_RADIUS) {
        packets.push(token, payload.clone());
    }
    Ok(())
}

/// Announces every TNT that has not been sent to clients yet.
pub(super) fn announce_pending_tnts<W: Write>(
    writer: &mut W,
    bus: &WorldPacketBus,
    level_random: &LevelRandomSource,
    packets: &mut DirectedPackets,
    tnts: &mut [PrimedTntEntity],
) -> io::Result<()> {
    for tnt in tnts.iter_mut().filter(|tnt| !tnt.announced) {
        let sync = EntitySync::new(TNT_UPDATE_INTERVAL, true, tnt.pos, tnt.delta, tnt.on_ground);
        write_spawn(writer, tnt, &sync)?;
        tnt.sync = Some(sync);
        tnt.announced = true;
        if tnt.play_prime_sound {
            tnt.play_prime_sound = false;
            // `level.playSound`: `seed = level.random.nextLong()`.
            let seed = level_random.lock().next_i64();
            queue_prime_sound(bus, packets, tnt, seed)?;
        }
    }
    Ok(())
}

/// `ServerEntity.sendChanges()` for a burning TNT whose fuse changed.
fn write_sync_changes<W: Write>(writer: &mut W, tnt: &mut PrimedTntEntity) -> io::Result<()> {
    let compression = CompressionState::disabled();
    let Some(sync) = tnt.sync.as_mut() else {
        return Ok(());
    };
    let output = sync.send_changes(tnt.pos, tnt.delta, tnt.on_ground, tnt.needs_sync, true);
    tnt.needs_sync = false;
    let id = tnt.entity_id;
    if let Some(movement) = output.motion {
        write_framed_packet_with_compression(
            writer,
            compression,
            CLIENTBOUND_SET_ENTITY_MOTION_PACKET_ID,
            |p| ClientboundSetEntityMotionPacket::new(id, movement).write(p),
        )?;
    }
    match output.position {
        Some(PositionUpdate::Sync) => write_framed_packet_with_compression(
            writer,
            compression,
            CLIENTBOUND_ENTITY_POSITION_SYNC_PACKET_ID,
            |p| {
                ClientboundEntityPositionSyncPacket {
                    id,
                    position: tnt.pos,
                    movement: tnt.delta,
                    y_rot: 0.0,
                    x_rot: 0.0,
                    on_ground: tnt.on_ground,
                }
                .write(p)
            },
        )?,
        Some(PositionUpdate::Relative(delta)) => write_framed_packet_with_compression(
            writer,
            compression,
            CLIENTBOUND_MOVE_ENTITY_POS_PACKET_ID,
            |p| ClientboundMoveEntityPacket::pos(id, delta, tnt.on_ground).write_pos(p),
        )?,
        None => {}
    }
    // `sendDirtyEntityData`: the fuse changes every tick.
    let data = fuse_data(id, tnt.fuse)?;
    write_framed_packet_with_compression(
        writer,
        compression,
        CLIENTBOUND_SET_ENTITY_DATA_PACKET_ID,
        |p| data.write(p),
    )
}

/// `PrimedTnt.explode()`: the TNT explosion at the entity's `getY(0.0625)`.
fn detonate<W: Write>(
    writer: &mut W,
    env: &ExplosionEnv<'_>,
    ticks: &mut ExplosionSinks<'_>,
    tnts: &mut Vec<PrimedTntEntity>,
    tnt: &PrimedTntEntity,
) -> io::Result<()> {
    // `Level.explode(this, .., usedPortal ? USED_PORTAL_DAMAGE_CALCULATOR : null,
    // x, getY(0.0625), z, explosionPower, false, ExplosionInteraction.TNT)`.
    let request = ExplosionRequest {
        source: Some(ExplosionSource {
            entity_id: tnt.entity_id,
            position: tnt.pos,
            owner: tnt.owner,
        }),
        center: Vec3 {
            x: tnt.pos.x,
            y: tnt.y_at(0.0625),
            z: tnt.pos.z,
        },
        radius: tnt.explosion_power,
        fire: false,
        interaction: LevelExplosionInteraction::Tnt,
        calculator: if tnt.used_portal {
            BlockCalculator::UsedPortal
        } else {
            BlockCalculator::Default
        },
    };
    explode(writer, env, ticks, tnts, &request)
}

/// Runs `PrimedTnt.tick` for every lit TNT that existed at the start of the
/// tick (TNT spawned by an explosion during the tick first ticks next tick).
pub(super) fn tick_primed_tnts<W: Write>(
    writer: &mut W,
    env: &ExplosionEnv<'_>,
    ticks: &mut ExplosionSinks<'_>,
) -> io::Result<()> {
    let mut tnts = {
        let mut store = lock_status_mutex(env.world_items);
        store.explosion_rules = env.rules;
        std::mem::take(&mut store.primed_tnts)
    };
    let result = tick_all(writer, env, ticks, &mut tnts);
    // Entities primed by other threads while this tick ran queued up in the
    // store; keep them after the ones simulated here.
    let mut store = lock_status_mutex(env.world_items);
    tnts.append(&mut store.primed_tnts);
    store.primed_tnts = tnts;
    result
}

fn tick_all<W: Write>(
    writer: &mut W,
    env: &ExplosionEnv<'_>,
    ticks: &mut ExplosionSinks<'_>,
    tnts: &mut Vec<PrimedTntEntity>,
) -> io::Result<()> {
    announce_pending_tnts(writer, env.bus, &env.cache.level_random, ticks.packets, tnts)?;
    let world = super::block_placement_live::LiveBlockWorld {
        layout: env.layout,
        seed: env.seed,
        cache: env.cache,
    };
    let ids: Vec<i32> = tnts.iter().map(|tnt| tnt.entity_id).collect();
    for id in ids {
        let Some(index) = tnts.iter().position(|tnt| tnt.entity_id == id) else {
            continue;
        };
        match tnts[index].tick(&world) {
            TntTick::Burning => write_sync_changes(writer, &mut tnts[index])?,
            TntTick::Detonate => {
                let tnt = tnts.remove(index);
                write_framed_packet_with_compression(
                    writer,
                    CompressionState::disabled(),
                    CLIENTBOUND_REMOVE_ENTITIES_PACKET_ID,
                    |p| {
                        write_var_i32(p, 1)?;
                        write_var_i32(p, tnt.entity_id)
                    },
                )?;
                if env.rules.tnt_explodes {
                    detonate(writer, env, ticks, tnts, &tnt)?;
                    announce_pending_tnts(writer, env.bus, &env.cache.level_random, ticks.packets, tnts)?;
                }
            }
        }
    }
    Ok(())
}
