//! Live experience orb wiring: client packets, per-tick simulation and player pickup.
//!
//! Java references: `ExperienceOrb.tick`/`playerTouch`, `Player.giveExperiencePoints`,
//! `Player.take`, `ServerEntity.addPairing` (spawn bundle) and the tracker removal.
//! The pure entity model lives in `xp_orb_entity.rs`.

use super::*;
use crate::experience_system::PlayerExperience;
use crate::xp_orb_entity::{tick_orbs, XpOrbEntity, XpOrbPlayer};

/// `BuiltInRegistries.ENTITY_TYPE` id of `minecraft:experience_orb` (registration order in
/// `EntityType.java`; `minecraft:item` is 71).
pub const XP_ORB_ENTITY_TYPE_ID: i32 = 49;
/// `ExperienceOrb.DATA_VALUE` — the first synched slot after the 8 base `Entity` fields.
const XP_ORB_DATA_VALUE_INDEX: u8 = 8;
/// `EntityDataSerializers.INT` id.
const INT_SERIALIZER_ID: i32 = 1;
/// Real time between two orb simulation steps; sessions tick the shared store, so the
/// store's own clock (not the session's) decides when a step is due.
const XP_ORB_STEP: Duration = Duration::from_millis(45);
/// `ExperienceOrb.playerTouch`: `takeXpDelay` set after a pickup.
const TAKE_XP_DELAY_TICKS: i32 = 2;
/// `Player.getEyeHeight()` while standing.
const PLAYER_EYE_HEIGHT: f64 = 1.62;

/// Writes the bundle-wrapped `ADD_ENTITY` + `SET_ENTITY_DATA(DATA_VALUE)` pair for `orb`
/// (`ServerEntity.addPairing`, same shape as [`write_item_entity_spawn_packets`]).
pub(super) fn write_xp_orb_spawn_packets<W: Write>(
    writer: &mut W,
    compression: CompressionState,
    orb: &XpOrbEntity,
) -> io::Result<()> {
    let entity_id = orb.entity_id();
    let uuid_hi = (entity_id as u64).wrapping_mul(0x6C62_272E_07BB_0142);
    let uuid_lo = (entity_id as u64).wrapping_mul(0x62B8_2175_6295_C58D);
    write_framed_packet_with_compression(writer, compression, CLIENTBOUND_BUNDLE_DELIMITER_PACKET_ID, |_| Ok(()))?;
    write_framed_packet_with_compression(
        writer,
        compression,
        CLIENTBOUND_ADD_ENTITY_PACKET_ID,
        |p| {
            write_var_i32(p, entity_id)?;
            p.write_all(&uuid_hi.to_be_bytes())?;
            p.write_all(&uuid_lo.to_be_bytes())?;
            write_var_i32(p, XP_ORB_ENTITY_TYPE_ID)?;
            p.write_all(&orb.x.to_be_bytes())?;
            p.write_all(&orb.y.to_be_bytes())?;
            p.write_all(&orb.z.to_be_bytes())?;
            write_lp_vec3(p, orb.vel_x, orb.vel_y, orb.vel_z)?;
            p.write_all(&[0u8, 0u8, 0u8])?; // xRot, yRot, yHeadRot
            write_var_i32(p, 0)
        },
    )?;
    write_framed_packet_with_compression(
        writer,
        compression,
        CLIENTBOUND_SET_ENTITY_DATA_PACKET_ID,
        |p| {
            write_var_i32(p, entity_id)?;
            p.write_all(&[XP_ORB_DATA_VALUE_INDEX])?;
            write_var_i32(p, INT_SERIALIZER_ID)?;
            write_var_i32(p, orb.value())?;
            p.write_all(&[0xFFu8]) // end of metadata
        },
    )?;
    write_framed_packet_with_compression(writer, compression, CLIENTBOUND_BUNDLE_DELIMITER_PACKET_ID, |_| Ok(()))
}

fn write_remove_entities<W: Write>(
    writer: &mut W,
    compression: CompressionState,
    ids: &[i32],
) -> io::Result<()> {
    write_framed_packet_with_compression(
        writer,
        compression,
        CLIENTBOUND_REMOVE_ENTITIES_PACKET_ID,
        |p| {
            write_var_i32(p, ids.len() as i32)?;
            for id in ids {
                write_var_i32(p, *id)?;
            }
            Ok(())
        },
    )
}

/// Advances every orb one step (when the shared clock says one is due) and lets this
/// session's player collect the ones in reach.
///
/// Expired and merged orbs are announced to every session over the bus; the pickup
/// `ClientboundTakeItemEntityPacket` goes only to the collecting connection, whose
/// collector id is [`PLAYER_ENTITY_ID`].
pub(super) fn tick_xp_orbs(
    writer: &mut impl Write,
    compression: CompressionState,
    state: &mut PlaySessionState,
    world_items: &Arc<Mutex<WorldItemEntities>>,
    bus: &WorldPacketBus,
    profile: &NameAndId,
) -> io::Result<()> {
    let mut store = lock_status_mutex(world_items);
    let due = !store.xp_orbs.is_empty()
        && store
            .xp_orb_last_step
            .is_none_or(|last| last.elapsed() >= XP_ORB_STEP);
    if due {
        store.xp_orb_last_step = Some(Instant::now());
        let players = [XpOrbPlayer {
            id: &profile.uuid,
            x: state.x,
            y: state.y,
            z: state.z,
            eye_height: PLAYER_EYE_HEIGHT,
            is_spectator: state.game_mode == GameMode::Spectator,
            is_dead: state.health <= 0.0,
        }];
        let removed = tick_orbs(&mut store.xp_orbs, &players).removed;
        if !removed.is_empty() {
            publish_removed(bus, &removed)?;
        }
    }
    collect_orbs_in_reach(writer, compression, state, &mut store, bus).map(|_| ())
}

fn publish_removed(bus: &WorldPacketBus, ids: &[i32]) -> io::Result<()> {
    let mut frames = Vec::new();
    write_remove_entities(&mut frames, CompressionState::disabled(), ids)?;
    bus.publish_frames(&frames)
}

/// `Player.aiStep` → `ExperienceOrb.playerTouch` for every orb in the pickup box. Returns
/// whether any experience was gained.
fn collect_orbs_in_reach(
    writer: &mut impl Write,
    compression: CompressionState,
    state: &mut PlaySessionState,
    store: &mut WorldItemEntities,
    bus: &WorldPacketBus,
) -> io::Result<bool> {
    // Player.aiStep only touches entities while alive and not a spectator.
    if state.health <= 0.0 || state.game_mode == GameMode::Spectator {
        return Ok(false);
    }
    let mut gained = false;
    let mut removed = Vec::new();
    for orb in &mut store.xp_orbs {
        if !item_entity::in_pickup_range(state.x, state.y, state.z, orb.x, orb.y, orb.z) {
            continue;
        }
        let mut experience = PlayerExperience {
            level: state.xp_level,
            progress: state.xp_progress,
            total: state.xp_total,
            take_xp_delay: state.combat.take_xp_delay,
        };
        let value = orb.value();
        // Mending (`repairPlayerItems`) needs item durability; nothing is repairable yet.
        if !orb.orb.player_touch(&mut experience, &mut []) {
            continue;
        }
        debug_assert_eq!(experience.take_xp_delay, TAKE_XP_DELAY_TICKS);
        state.combat.take_xp_delay = experience.take_xp_delay;
        state.xp_level = experience.level;
        state.xp_progress = experience.progress;
        state.xp_total = experience.total;
        // Player.giveExperiencePoints → increaseScore(remaining).
        state.score = state.score.saturating_add(value);
        gained = true;
        write_take_xp_orb(writer, compression, orb.entity_id())?;
        if orb.orb.removed {
            removed.push(orb.entity_id());
        }
    }
    store.xp_orbs.retain(|orb| !orb.orb.removed);
    if !removed.is_empty() {
        publish_removed(bus, &removed)?;
    }
    if gained {
        write_set_experience(writer, compression, state)?;
    }
    Ok(gained)
}

/// `Player.take(entity, 1)` → `ClientboundTakeItemEntityPacket`.
fn write_take_xp_orb<W: Write>(
    writer: &mut W,
    compression: CompressionState,
    entity_id: i32,
) -> io::Result<()> {
    write_framed_packet_with_compression(
        writer,
        compression,
        CLIENTBOUND_TAKE_ITEM_ENTITY_PACKET_ID,
        |payload| {
            ClientboundTakeItemEntityPacket {
                item_entity_id: entity_id,
                collector_entity_id: PLAYER_ENTITY_ID,
                amount: 1,
            }
            .write(payload)
        },
    )
}

/// `ClientboundSetExperiencePacket(progress, total, level)` — sent when
/// `ServerPlayer.lastSentExp` is invalidated.
pub(super) fn write_set_experience<W: Write>(
    writer: &mut W,
    compression: CompressionState,
    state: &PlaySessionState,
) -> io::Result<()> {
    write_framed_packet_with_compression(
        writer,
        compression,
        CLIENTBOUND_SET_EXPERIENCE_PACKET_ID,
        |payload| {
            payload.write_all(&state.xp_progress.to_be_bytes())?;
            write_var_i32(payload, state.xp_level)?;
            write_var_i32(payload, state.xp_total)
        },
    )
}

#[cfg(test)]
mod tests;
