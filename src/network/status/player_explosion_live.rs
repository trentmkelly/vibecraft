//! The session half of explosions: publishing this player's
//! [`PlayerPresence`] for the world tick and applying the
//! [`ExplosionHit`]s it queued.
//!
//! Java's `ServerExplosion.hurtEntities` calls `ServerPlayer.hurtServer`
//! directly; here the world tick cannot touch another thread's
//! `PlaySessionState`, so the hit travels through the [`WorldPacketBus`] and
//! this module runs it through the normal hurt pipeline
//! ([`hurt_server`]) on the session's own tick. The knockback reaches the
//! client in `ClientboundExplodePacket` (sent by the world tick).

use super::player_damage::{explosion_knockback_resistance, hurt_server, HurtOrigin};
use super::player_environment::{
    CROUCHING_EYE_HEIGHT, CROUCHING_HEIGHT, PLAYER_WIDTH_F, STANDING_EYE_HEIGHT, STANDING_HEIGHT,
};
use super::*;
use crate::network::world_broadcast::PlayerPresence;

/// This player as `ServerLevel.players()` exposes it to the level tick.
pub(super) fn presence_of(state: &PlaySessionState) -> PlayerPresence {
    let (height, eye_height) = if state.input_shift {
        (CROUCHING_HEIGHT, CROUCHING_EYE_HEIGHT)
    } else {
        (STANDING_HEIGHT, STANDING_EYE_HEIGHT)
    };
    PlayerPresence {
        entity_id: PLAYER_ENTITY_ID,
        position: [state.x, state.y, state.z],
        width: PLAYER_WIDTH_F,
        height,
        eye_height,
        spectator: state.game_mode == GameMode::Spectator,
        creative: state.game_mode == GameMode::Creative,
        flying: state.abilities.flying,
        explosion_knockback_resistance: explosion_knockback_resistance(state),
    }
}

/// Publishes the presence and applies queued explosion damage; returns
/// whether any hit was accepted (so the vitals need syncing).
pub(super) fn sync_with_world(state: &mut PlaySessionState, subscription: &Subscription) -> bool {
    subscription.update_presence(presence_of(state));
    let mut hurt = false;
    for hit in subscription.take_explosion_hits() {
        // `DamageSources.explosion(directEntity, causingEntity)`.
        let source = crate::damage_type::explosion_source(hit.direct, hit.causing);
        hurt |= hurt_server(
            state,
            source,
            hit.damage,
            HurtOrigin {
                attacker_type: hit.causing_type,
                position: Some(hit.source_position),
            },
        );
    }
    hurt
}
