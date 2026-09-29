//! Player presence and cross-thread world hits on the [`WorldPacketBus`].
//!
//! Java's `ServerLevel.players()` lets the level tick read every player's
//! position and hurt them directly (`ServerExplosion.hurtEntities`,
//! `PlayerList.broadcast(x, y, z, radius, ..)`). VibeCraft runs one thread per
//! connection, so each session publishes a [`PlayerPresence`] snapshot every
//! tick and the world tick queues [`ExplosionHit`]s that the owning session
//! applies to its own `PlaySessionState`.

use super::{Inbox, Subscription, WorldPacketBus};
use crate::damage_type::DamageEntityRef;

/// One player's position and body as `ServerLevel.players()` exposes it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PlayerPresence {
    /// `Entity.getId()`.
    pub entity_id: i32,
    /// `Entity.position()` (feet).
    pub position: [f64; 3],
    /// `EntityDimensions.width()`.
    pub width: f64,
    /// `EntityDimensions.height()`.
    pub height: f64,
    /// `Entity.getEyeHeight()`.
    pub eye_height: f64,
    /// `Entity.isSpectator()`.
    pub spectator: bool,
    /// `Player.isCreative()`.
    pub creative: bool,
    /// `Abilities.flying`.
    pub flying: bool,
    /// `LivingEntity.getAttributeValue(EXPLOSION_KNOCKBACK_RESISTANCE)`.
    pub explosion_knockback_resistance: f64,
}

/// A `Entity.hurtServer(explosion damage)` the world tick queued for a player.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ExplosionHit {
    /// The amount from `ExplosionDamageCalculator.getEntityDamageAmount`.
    pub damage: f32,
    /// `DamageSource.getDirectEntity()` (the exploding entity).
    pub direct: Option<DamageEntityRef>,
    /// `DamageSource.getEntity()` (the responsible living entity).
    pub causing: Option<DamageEntityRef>,
    /// Registry name of the causing entity for death messages.
    pub causing_type: Option<&'static str>,
    /// Position of the direct entity (`DamageSource.getSourcePosition()`).
    pub source_position: [f64; 3],
}

impl WorldPacketBus {
    /// The presence snapshot of every in-play connection, with its token.
    pub fn players(&self) -> Vec<(u64, PlayerPresence)> {
        let mut players: Vec<(u64, PlayerPresence)> = self
            .lock()
            .iter()
            .filter_map(|(token, inbox)| inbox.presence.map(|presence| (*token, presence)))
            .collect();
        players.sort_by_key(|(token, _)| *token);
        players
    }

    /// Queues an explosion hit for `token`'s session; false when it left.
    pub fn push_explosion_hit(&self, token: u64, hit: ExplosionHit) -> bool {
        match self.lock().get_mut(&token) {
            Some(inbox) if !inbox.closing => {
                inbox.explosion_hits.push(hit);
                true
            }
            _ => false,
        }
    }

    /// The connections whose player is closer than `radius` to `center`
    /// (`PlayerList.broadcast(null, x, y, z, radius, ..)`).
    pub fn tokens_within(&self, center: [f64; 3], radius: f64) -> Vec<u64> {
        self.players()
            .into_iter()
            .filter(|(_, presence)| {
                let d = |i: usize| presence.position[i] - center[i];
                d(0) * d(0) + d(1) * d(1) + d(2) * d(2) < radius * radius
            })
            .map(|(token, _)| token)
            .collect()
    }
}

impl Subscription {
    /// Publishes this session's current [`PlayerPresence`].
    pub fn update_presence(&self, presence: PlayerPresence) {
        if let Some(inbox) = self.bus.lock().get_mut(&self.token) {
            inbox.presence = Some(presence);
        }
    }

    /// Takes the explosion hits queued for this session since the last call.
    pub fn take_explosion_hits(&self) -> Vec<ExplosionHit> {
        self.bus
            .lock()
            .get_mut(&self.token)
            .map(|inbox: &mut Inbox| std::mem::take(&mut inbox.explosion_hits))
            .unwrap_or_default()
    }
}
