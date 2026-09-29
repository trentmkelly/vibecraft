//! Process-wide live scoreboard: the shared instance, command integration, the join sequence and
//! `scoreboard.dat` persistence.
//!
//! Java's `MinecraftServer` owns one `ServerScoreboard` (loaded from the overworld's
//! `SavedDataStorage` at startup, saved when dirty during `saveAllChunks`/shutdown). The
//! equivalent here is [`global`], initialised once by [`initialize`] and reached by:
//!
//! * commands: [`CommandScoreboardSeed::seed`] copies the scoreboard into the command state before
//!   a command runs and [`CommandScoreboardSeed::apply`] folds the command's edits back in and
//!   broadcasts the resulting packets;
//! * joining players: [`send_join_packets`] (`PlayerList.updateEntireScoreboard`);
//! * saving: [`save_if_dirty`].

use std::io;
use std::sync::{Arc, Mutex, MutexGuard, OnceLock};

use crate::command::{ServerCommandState, TeamMembership};
use crate::player_access::NameAndId;
use crate::network::world_broadcast::WorldPacketBus;
use crate::storage::world::WorldLayout;

use super::codec::{from_saved_data_tag, to_saved_data_tag};
use super::{ScoreboardData, ServerScoreboard, TeamMember};

/// The shared `ServerScoreboard`.
#[derive(Clone, Default)]
pub struct SharedScoreboard(Arc<Mutex<ServerScoreboard>>);

impl SharedScoreboard {
    /// Locks the scoreboard; a poisoned lock only means another thread panicked mid-hook, and the
    /// data is still structurally valid.
    pub fn lock(&self) -> MutexGuard<'_, ServerScoreboard> {
        self.0.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// Publishes every packet the scoreboard's hooks queued to all online players
    /// (`PlayerList.broadcastAll`).
    pub fn flush_to(&self, bus: &WorldPacketBus) {
        let mut scoreboard = self.lock();
        for payload in scoreboard.take_outbox() {
            bus.publish(&payload);
        }
    }
}

struct LiveScoreboard {
    scoreboard: SharedScoreboard,
    /// Where `scoreboard.dat` lives; `None` until [`initialize`] runs (unit tests, tools).
    layout: Mutex<Option<WorldLayout>>,
}

fn live() -> &'static LiveScoreboard {
    static LIVE: OnceLock<LiveScoreboard> = OnceLock::new();
    LIVE.get_or_init(|| LiveScoreboard {
        scoreboard: SharedScoreboard::default(),
        layout: Mutex::new(None),
    })
}

/// The process-wide scoreboard.
pub fn global() -> SharedScoreboard {
    live().scoreboard.clone()
}

/// Loads `data/minecraft/scoreboard.dat` (a missing or undecodable file yields an empty
/// scoreboard, as `SavedDataStorage` does for a `SavedDataType` constructor) into the global
/// scoreboard.
pub fn initialize(layout: WorldLayout) -> io::Result<()> {
    let data = match layout.load_scoreboard() {
        Ok(tag) => from_saved_data_tag(&tag).unwrap_or_else(|error| {
            eprintln!("Failed to parse scoreboard.dat, starting empty: {error}");
            ScoreboardData::default()
        }),
        Err(error) if error.kind() == io::ErrorKind::NotFound => ScoreboardData::default(),
        Err(error) => return Err(error),
    };
    *live().scoreboard.lock() = ServerScoreboard::load(data);
    *live().layout.lock().unwrap_or_else(|p| p.into_inner()) = Some(layout);
    Ok(())
}

/// `ServerScoreboard.storeToSaveDataIfDirty` + the saved-data write: persists the scoreboard if
/// anything changed since the last save.
pub fn save_if_dirty() -> io::Result<()> {
    let layout = live().layout.lock().unwrap_or_else(|p| p.into_inner()).clone();
    let Some(layout) = layout else {
        return Ok(());
    };
    let dirty = live().scoreboard.lock().take_dirty_data();
    match dirty {
        Some(data) => layout.save_scoreboard(&to_saved_data_tag(&data)),
        None => Ok(()),
    }
}

/// `PlayerList.updateEntireScoreboard`: queues the scoreboard's state for the joining player
/// registered under `token`.
pub fn send_join_packets(scoreboard: &SharedScoreboard, bus: &WorldPacketBus, token: u64) {
    for payload in scoreboard.lock().join_packets() {
        bus.publish_to(token, &payload);
    }
}

/// The scoreboard snapshot a command state was seeded with, used to diff the command's edits.
pub struct CommandScoreboardSeed {
    baseline: ScoreboardData,
}

impl CommandScoreboardSeed {
    /// Copies the scoreboard into the command state (`state.scoreboard_*`, `state.teams`,
    /// `state.player_teams`). Team members that are online resolve to their real profile so
    /// `/team join`/`leave` compare consistently.
    pub fn seed(scoreboard: &SharedScoreboard, state: &mut ServerCommandState) -> Self {
        let baseline = scoreboard.lock().data().clone();
        state.scoreboard_objectives = baseline.objectives.clone();
        state.scoreboard_scores = baseline.scores.clone();
        state.scoreboard_display_slots = baseline.display_slots.clone();
        state.teams = baseline.teams.clone();
        state.player_teams = baseline
            .members
            .iter()
            .map(|member| TeamMembership {
                player: resolve_member(state, &member.player),
                team: member.team.clone(),
            })
            .collect();
        Self { baseline }
    }

    /// Folds the command's scoreboard edits into the live scoreboard and broadcasts the packets.
    pub fn apply(
        &self,
        scoreboard: &SharedScoreboard,
        state: &ServerCommandState,
        bus: &WorldPacketBus,
    ) {
        let after = ScoreboardData {
            objectives: state.scoreboard_objectives.clone(),
            scores: state.scoreboard_scores.clone(),
            display_slots: state.scoreboard_display_slots.clone(),
            teams: state.teams.clone(),
            members: members_from_state(state),
        };
        if after == self.baseline {
            return;
        }
        scoreboard.lock().apply_command_changes(&self.baseline, &after);
        scoreboard.flush_to(bus);
    }
}

/// Later memberships of the same holder replace earlier ones (a holder is on at most one team).
fn members_from_state(state: &ServerCommandState) -> Vec<TeamMember> {
    let mut members: Vec<TeamMember> = Vec::new();
    for membership in &state.player_teams {
        members.retain(|member| member.player != membership.player.name);
        members.push(TeamMember {
            player: membership.player.name.clone(),
            team: membership.team.clone(),
        });
    }
    members
}

fn resolve_member(state: &ServerCommandState, name: &str) -> NameAndId {
    state
        .online_players
        .iter()
        .find(|player| player.name == name)
        .cloned()
        .unwrap_or_else(|| NameAndId::create_offline(name))
}
