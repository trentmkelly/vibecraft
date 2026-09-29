//! Position/motion synchronisation of a tracked entity: the decision logic of
//! Java `ServerEntity.sendChanges` for a non-passenger, non-minecart entity
//! that never rotates (e.g. `PrimedTnt`).
//!
//! The caller feeds the entity's current state each tick and receives which
//! movement packets to broadcast; entity-data (`SynchedEntityData`) dirtiness
//! is reported in by the caller and written out by it.

use crate::network::play::{VecDeltaCodec, Vec3};

/// `ServerEntity.TOLERANCE_LEVEL_POSITION` (`7.6293945E-6F`).
const TOLERANCE_LEVEL_POSITION: f64 = 7.629_394_5E-6;
/// `ServerEntity` forces a full teleport-style sync after this many
/// relative updates.
const FORCED_TELEPORT_PERIOD: i32 = 400;
/// `ServerEntity.FORCED_POS_UPDATE_PERIOD`.
const FORCED_POS_UPDATE_PERIOD: i32 = 60;

/// The position packet `sendChanges` chose.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PositionUpdate {
    /// `ClientboundEntityPositionSyncPacket.of(entity)`.
    Sync,
    /// `ClientboundMoveEntityPacket.Pos` with the encoded deltas.
    Relative([i16; 3]),
}

/// What one `sendChanges` call decided to send.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SyncOutput {
    pub position: Option<PositionUpdate>,
    /// `ClientboundSetEntityMotionPacket` payload.
    pub motion: Option<Vec3>,
}

/// `ServerEntity` state that persists between ticks.
#[derive(Clone, Debug, PartialEq)]
pub struct EntitySync {
    update_interval: i32,
    track_delta: bool,
    position_codec: VecDeltaCodec,
    last_sent_movement: Vec3,
    tick_count: i32,
    teleport_delay: i32,
    was_on_ground: bool,
}

impl EntitySync {
    /// `new ServerEntity(level, entity, updateInterval, trackDelta, ..)`.
    pub fn new(
        update_interval: i32,
        track_delta: bool,
        position: Vec3,
        movement: Vec3,
        on_ground: bool,
    ) -> Self {
        let mut position_codec = VecDeltaCodec::new();
        position_codec.set_base(position);
        Self {
            update_interval,
            track_delta,
            position_codec,
            last_sent_movement: movement,
            tick_count: 0,
            teleport_delay: 0,
            was_on_ground: on_ground,
        }
    }

    /// `ServerEntity.getPositionBase()`.
    pub fn position_base(&self) -> Vec3 {
        self.position_codec.base()
    }

    /// `ServerEntity.getLastSentMovement()`.
    pub fn last_sent_movement(&self) -> Vec3 {
        self.last_sent_movement
    }

    /// `ServerEntity.sendChanges()`.
    ///
    /// `needs_sync` is `Entity.needsSync` (set by `Entity.push`) and is
    /// consumed by the caller afterwards; `data_dirty` is
    /// `SynchedEntityData.isDirty()`.
    pub fn send_changes(
        &mut self,
        position: Vec3,
        movement: Vec3,
        on_ground: bool,
        needs_sync: bool,
        data_dirty: bool,
    ) -> SyncOutput {
        let mut output = SyncOutput::default();
        if self.tick_count % self.update_interval == 0 || needs_sync || data_dirty {
            self.teleport_delay += 1;
            let delta = self.position_codec.delta(position);
            let position_changed =
                delta.x * delta.x + delta.y * delta.y + delta.z * delta.z >= TOLERANCE_LEVEL_POSITION;
            let pos = position_changed || self.tick_count % FORCED_POS_UPDATE_PERIOD == 0;
            let xa = self.position_codec.encode_x(position);
            let ya = self.position_codec.encode_y(position);
            let za = self.position_codec.encode_z(position);
            let delta_too_big = [xa, ya, za]
                .iter()
                .any(|v| *v < i64::from(i16::MIN) || *v > i64::from(i16::MAX));
            let mut sent_position = false;
            if delta_too_big
                || self.teleport_delay > FORCED_TELEPORT_PERIOD
                || self.was_on_ground != on_ground
            {
                self.was_on_ground = on_ground;
                self.teleport_delay = 0;
                output.position = Some(PositionUpdate::Sync);
                sent_position = true;
            } else if pos {
                output.position = Some(PositionUpdate::Relative([xa as i16, ya as i16, za as i16]));
                sent_position = true;
            }
            if needs_sync || self.track_delta {
                let d = Vec3 {
                    x: movement.x - self.last_sent_movement.x,
                    y: movement.y - self.last_sent_movement.y,
                    z: movement.z - self.last_sent_movement.z,
                };
                let diff = d.x * d.x + d.y * d.y + d.z * d.z;
                let movement_sqr =
                    movement.x * movement.x + movement.y * movement.y + movement.z * movement.z;
                if diff > 1.0E-7 || (diff > 0.0 && movement_sqr == 0.0) {
                    self.last_sent_movement = movement;
                    output.motion = Some(movement);
                }
            }
            if sent_position {
                self.position_codec.set_base(position);
            }
        }
        self.tick_count += 1;
        output
    }
}

#[cfg(test)]
mod tests;
