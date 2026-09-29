//! Live wiring of the shared [`ServerScoreboard`](crate::server_scoreboard::ServerScoreboard)
//! into the status runtime: the join sequence and persistence.

use super::player_damage::armor_totals;
use super::PlaySessionState;
use crate::player_access::NameAndId;
use crate::network::world_broadcast::WorldPacketBus;
use crate::server_scoreboard::criteria::PlayerVitals;
use crate::server_scoreboard::live::{global, save_if_dirty, send_join_packets};

/// `MinecraftServer.saveAllChunks` writing the dirty `scoreboard` `SavedData`.
pub(super) fn save_scoreboard() {
    if let Err(err) = save_if_dirty() {
        eprintln!("Failed to save scoreboard.dat: {err}");
    }
}

/// `PlayerList.updateEntireScoreboard` for the session registered on the bus as `token`.
pub(super) fn send_join_scoreboard(bus: &WorldPacketBus, token: u64) {
    send_join_packets(&global(), bus, token);
}

/// `ServerPlayer.doTick`: publishes changed health/food/air/armor/xp/level to the matching
/// criteria objectives and broadcasts the resulting score packets.
pub(super) fn update_player_criteria(
    state: &mut PlaySessionState,
    profile: &NameAndId,
    bus: &WorldPacketBus,
) {
    let vitals = PlayerVitals {
        health_and_absorption: state.health + state.combat.hurt.absorption,
        food_level: state.food_level,
        air_supply: state.air_supply,
        armor: armor_totals(state).0,
        experience: state.xp_total,
        level: state.xp_level,
    };
    let scoreboard = global();
    scoreboard.lock().update_vitals(&mut state.combat.recorded_vitals, &profile.name, vitals);
    scoreboard.flush_to(bus);
}

/// `ServerPlayer.die`: `forAllObjectives(DEATH_COUNT, this, ScoreAccess::increment)`.
pub(super) fn record_player_death(profile: &NameAndId, bus: &WorldPacketBus) {
    let scoreboard = global();
    scoreboard.lock().increment_death_count(&profile.name);
    scoreboard.flush_to(bus);
}
