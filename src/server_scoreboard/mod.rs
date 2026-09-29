//! The server's single scoreboard (Java `ServerScoreboard extends Scoreboard`).
//!
//! Java keeps one `ServerScoreboard` on the `MinecraftServer`; the `/scoreboard`, `/team` and
//! `/trigger` commands, criteria updates (`ServerPlayer.awardStat`, `die`, ...) and the join
//! sequence (`PlayerList.updateEntireScoreboard`) all read and mutate it, and it persists as the
//! `scoreboard` `SavedData` (`data/minecraft/scoreboard.dat`).
//!
//! This module owns that state ([`ServerScoreboard`]) with the same mutation hooks Java overrides
//! (`onScoreChanged`, `setDisplayObjective`, `addPlayerToTeam`, ...). Each hook queues the
//! clientbound packet Java broadcasts to `PlayerList.broadcastAll` in an outbox that the caller
//! drains onto the world packet bus, and marks the data dirty for the next save.
//!
//! Score holders are plain names (players by profile name, entities by UUID string).
//! `Scoreboard.entityRemoved` is not ported: live mobs have no UUID yet.
//!
//! * [`live`] is the process-wide handle plus the command/join/save integration.
//! * [`codec`] is the `ScoreboardSaveData.Packed` NBT codec.
//! * [`criteria`] drives criteria-based scores (`deathCount`, `health`, ...).

use std::collections::BTreeSet;

use crate::command::{ScoreboardDisplaySlot, ScoreboardObjective, ScoreboardScore, TeamState};
use display_slot::DisplaySlot;

// TODO(entity-score-holders): drive `Scoreboard.entityRemoved` (drop a dead non-player entity's
// scores and team) once live mobs carry UUIDs.

pub mod codec;
pub mod criteria;
pub mod live;
mod display_slot;
mod number_format;
mod packets;
mod sync;

use packets::ObjectiveMethodKind;

/// One team member: a `ScoreHolder.getScoreboardName()` (player name or entity UUID string).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TeamMember {
    pub player: String,
    pub team: String,
}

/// Everything `ScoreboardSaveData.Packed` stores: objectives, scores, display slots, teams.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ScoreboardData {
    pub objectives: Vec<ScoreboardObjective>,
    pub scores: Vec<ScoreboardScore>,
    pub display_slots: Vec<ScoreboardDisplaySlot>,
    pub teams: Vec<TeamState>,
    pub members: Vec<TeamMember>,
}

impl ScoreboardData {
    fn objective(&self, name: &str) -> Option<&ScoreboardObjective> {
        self.objectives.iter().find(|objective| objective.name == name)
    }

    /// `Scoreboard.getDisplayObjective(slot)` as an objective name.
    fn display_objective(&self, slot: &str) -> Option<&str> {
        self.display_slots
            .iter()
            .find(|entry| entry.slot == slot)
            .map(|entry| entry.objective.as_str())
    }

    fn score(&self, owner: &str, objective: &str) -> Option<&ScoreboardScore> {
        self.scores
            .iter()
            .find(|score| score.owner == owner && score.objective == objective)
    }

    fn team(&self, name: &str) -> Option<&TeamState> {
        self.teams.iter().find(|team| team.name == name)
    }

    /// `Scoreboard.getPlayersTeam(name)`.
    fn team_of(&self, player: &str) -> Option<&str> {
        self.members
            .iter()
            .find(|member| member.player == player)
            .map(|member| member.team.as_str())
    }

    fn team_players(&self, team: &str) -> Vec<String> {
        self.members
            .iter()
            .filter(|member| member.team == team)
            .map(|member| member.player.clone())
            .collect()
    }

    /// `Scoreboard.getObjectiveDisplaySlotCount(objective)` (`ServerScoreboard`).
    fn display_slot_count(&self, objective: &str) -> usize {
        self.display_slots
            .iter()
            .filter(|entry| entry.objective == objective)
            .count()
    }
}

/// Java `ServerScoreboard`: the scoreboard data plus the tracked-objective set, dirty flag and
/// the packets its hooks have queued for broadcast.
#[derive(Debug, Default)]
pub struct ServerScoreboard {
    data: ScoreboardData,
    /// Java `trackedObjectives`: objectives whose state has been sent to every player.
    tracked_objectives: BTreeSet<String>,
    dirty: bool,
    outbox: Vec<Vec<u8>>,
}

impl ServerScoreboard {
    /// `ServerScoreboard.load(ScoreboardSaveData.Packed)`: objectives, scores, display slots,
    /// then teams. Loading queues no packets (nobody is online yet) but, exactly like Java's
    /// `setDisplayObjective`, marks displayed objectives as tracked.
    pub fn load(packed: ScoreboardData) -> Self {
        let mut scoreboard = Self::default();
        scoreboard.data.objectives = packed.objectives;
        for score in packed.scores {
            if scoreboard.data.objective(&score.objective).is_some() {
                scoreboard.data.scores.push(score);
            }
        }
        for entry in &packed.display_slots {
            if let Some(slot) = DisplaySlot::by_name(&entry.slot) {
                let known = scoreboard.data.objective(&entry.objective).is_some();
                scoreboard.set_display_objective(slot, known.then_some(entry.objective.as_str()));
            }
        }
        for team in packed.teams {
            scoreboard.data.teams.push(team);
        }
        for member in packed.members {
            scoreboard.add_player_to_team(&member.player, &member.team);
        }
        scoreboard.outbox.clear();
        scoreboard.dirty = false;
        scoreboard
    }

    /// The current data (`ServerScoreboard.store()` before it is packed).
    pub fn data(&self) -> &ScoreboardData {
        &self.data
    }

    /// Takes the packets queued since the last call, in send order.
    pub fn take_outbox(&mut self) -> Vec<Vec<u8>> {
        std::mem::take(&mut self.outbox)
    }

    /// Whether anything changed since the last [`Self::take_dirty_data`].
    #[cfg(test)]
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// `storeToSaveDataIfDirty`: the data to persist if dirty, clearing the flag.
    pub fn take_dirty_data(&mut self) -> Option<ScoreboardData> {
        std::mem::take(&mut self.dirty).then(|| self.data.clone())
    }

    fn set_dirty(&mut self) {
        self.dirty = true;
    }

    // ---- objectives ------------------------------------------------------------------------

    /// `Scoreboard.addObjective` + `ServerScoreboard.onObjectiveAdded` (dirty only).
    pub fn add_objective(&mut self, objective: ScoreboardObjective) {
        self.data.objectives.push(objective);
        self.set_dirty();
    }

    /// `ServerScoreboard.onObjectiveChanged`: tracked objectives broadcast method CHANGE.
    pub fn change_objective(&mut self, objective: ScoreboardObjective) {
        let Some(slot) = self
            .data
            .objectives
            .iter_mut()
            .find(|entry| entry.name == objective.name)
        else {
            return;
        };
        *slot = objective.clone();
        if self.tracked_objectives.contains(&objective.name) {
            self.outbox
                .push(packets::set_objective(&objective, ObjectiveMethodKind::Change));
        }
        self.set_dirty();
    }

    /// `Scoreboard.removeObjective` + `ServerScoreboard.onObjectiveRemoved`: displayed slots are
    /// cleared first (each through `setDisplayObjective(slot, null)`), scores dropped, and a
    /// tracked objective is stopped.
    pub fn remove_objective(&mut self, name: &str) {
        if self.data.objective(name).is_none() {
            return;
        }
        self.data.objectives.retain(|entry| entry.name != name);
        for slot in DisplaySlot::VALUES {
            if self.data.display_objective(slot.serialized_name()) == Some(name) {
                self.set_display_objective(slot, None);
            }
        }
        self.data.scores.retain(|score| score.objective != name);
        if self.tracked_objectives.contains(name) {
            self.stop_tracking_objective(name);
        }
        self.set_dirty();
    }

    /// `ServerScoreboard.setDisplayObjective`.
    pub fn set_display_objective(&mut self, slot: DisplaySlot, objective: Option<&str>) {
        let slot_name = slot.serialized_name();
        let old = self.data.display_objective(slot_name).map(str::to_string);
        self.data.display_slots.retain(|entry| entry.slot != slot_name);
        if let Some(name) = objective {
            self.data.display_slots.push(ScoreboardDisplaySlot {
                slot: slot_name.to_string(),
                objective: name.to_string(),
            });
        }
        if let Some(old) = old.as_deref().filter(|old| Some(*old) != objective) {
            if self.data.display_slot_count(old) > 0 {
                self.outbox.push(packets::set_display_objective(slot, objective));
            } else {
                self.stop_tracking_objective(old);
            }
        }
        if let Some(name) = objective {
            if self.tracked_objectives.contains(name) {
                self.outbox.push(packets::set_display_objective(slot, Some(name)));
            } else {
                self.start_tracking_objective(name);
            }
        }
        self.set_dirty();
    }

    // ---- tracking --------------------------------------------------------------------------

    /// `ServerScoreboard.getStartTrackingPackets`: the objective, its display slots, its scores.
    pub fn start_tracking_packets(&self, objective: &str) -> Vec<Vec<u8>> {
        let Some(entry) = self.data.objective(objective) else {
            return Vec::new();
        };
        let mut queued = vec![packets::set_objective(entry, ObjectiveMethodKind::Add)];
        for slot in DisplaySlot::VALUES {
            if self.data.display_objective(slot.serialized_name()) == Some(objective) {
                queued.push(packets::set_display_objective(slot, Some(objective)));
            }
        }
        for score in self.data.scores.iter().filter(|score| score.objective == objective) {
            queued.push(packets::set_score(
                &score.owner,
                objective,
                score.value,
                score.display_name.as_deref(),
                score.number_format.as_deref(),
            ));
        }
        queued
    }

    /// `ServerScoreboard.startTrackingObjective`.
    fn start_tracking_objective(&mut self, objective: &str) {
        let queued = self.start_tracking_packets(objective);
        self.outbox.extend(queued);
        self.tracked_objectives.insert(objective.to_string());
    }

    /// `ServerScoreboard.getStopTrackingPackets`: REMOVE, then the slots still showing it.
    fn stop_tracking_packets(&self, objective: &str) -> Vec<Vec<u8>> {
        let mut queued = vec![packets::remove_objective(objective)];
        for slot in DisplaySlot::VALUES {
            if self.data.display_objective(slot.serialized_name()) == Some(objective) {
                queued.push(packets::set_display_objective(slot, Some(objective)));
            }
        }
        queued
    }

    /// `ServerScoreboard.stopTrackingObjective`.
    fn stop_tracking_objective(&mut self, objective: &str) {
        let queued = self.stop_tracking_packets(objective);
        self.outbox.extend(queued);
        self.tracked_objectives.remove(objective);
    }

    /// `PlayerList.updateEntireScoreboard`: what a joining player is sent — every team, then the
    /// start-tracking packets of each distinct displayed objective in display-slot order.
    pub fn join_packets(&self) -> Vec<Vec<u8>> {
        let mut queued: Vec<Vec<u8>> = self
            .data
            .teams
            .iter()
            .map(|team| packets::team_add_or_modify(team, Some(self.data.team_players(&team.name))))
            .collect();
        let mut sent: Vec<&str> = Vec::new();
        for slot in DisplaySlot::VALUES {
            if let Some(objective) = self.data.display_objective(slot.serialized_name()) {
                if !sent.contains(&objective) {
                    queued.extend(self.start_tracking_packets(objective));
                    sent.push(objective);
                }
            }
        }
        queued
    }

    // ---- scores ----------------------------------------------------------------------------

    /// `ServerScoreboard.onScoreChanged`: tracked objectives broadcast the score; always dirty.
    fn on_score_changed(&mut self, owner: &str, objective: &str) {
        if self.tracked_objectives.contains(objective) {
            if let Some(score) = self.data.score(owner, objective) {
                self.outbox.push(packets::set_score(
                    owner,
                    objective,
                    score.value,
                    score.display_name.as_deref(),
                    score.number_format.as_deref(),
                ));
            }
        }
        self.set_dirty();
    }

    /// `Scoreboard.getOrCreatePlayerScore(holder, objective, true)` followed by `set(value)`
    /// (`ScoreAccess.set`): a new score always syncs, and an auto-updating objective refreshes
    /// the score's display name from the holder's display name.
    pub fn set_score(
        &mut self,
        owner: &str,
        objective: &str,
        value: i32,
        holder_display_name: Option<&str>,
    ) {
        let Some(entry) = self.data.objective(objective) else {
            return;
        };
        let auto_update = entry.display_auto_update;
        let mut changed = false;
        if self.data.score(owner, objective).is_none() {
            self.data.scores.push(new_score(owner, objective));
            changed = true;
        }
        let Some(score) = self
            .data
            .scores
            .iter_mut()
            .find(|score| score.owner == owner && score.objective == objective)
        else {
            return;
        };
        if let Some(display) = holder_display_name.filter(|_| auto_update) {
            if score.display_name.as_deref() != Some(display) {
                score.display_name = Some(display.to_string());
                changed = true;
            }
        }
        if score.value != value {
            score.value = value;
            changed = true;
        }
        if changed {
            self.on_score_changed(owner, objective);
        }
    }

    /// The current value of `owner`'s score (`Scoreboard.getPlayerScoreInfo`).
    pub fn score_value(&self, owner: &str, objective: &str) -> Option<i32> {
        self.data.score(owner, objective).map(|score| score.value)
    }

    /// `Scoreboard.resetAllPlayerScores` + `onPlayerRemoved`.
    pub fn reset_all_player_scores(&mut self, owner: &str) {
        let before = self.data.scores.len();
        self.data.scores.retain(|score| score.owner != owner);
        if self.data.scores.len() != before {
            self.outbox.push(packets::reset_score(owner, None));
            self.set_dirty();
        }
    }

    /// `Scoreboard.resetSinglePlayerScore`: removing a holder's last score resets the holder.
    pub fn reset_single_player_score(&mut self, owner: &str, objective: &str) {
        let held = self.data.scores.iter().filter(|score| score.owner == owner).count();
        if held == 0 {
            return;
        }
        let before = self.data.scores.len();
        self.data
            .scores
            .retain(|score| !(score.owner == owner && score.objective == objective));
        let removed = self.data.scores.len() != before;
        if held == usize::from(removed) {
            self.outbox.push(packets::reset_score(owner, None));
            self.set_dirty();
        } else if removed {
            if self.tracked_objectives.contains(objective) {
                self.outbox.push(packets::reset_score(owner, Some(objective)));
            }
            self.set_dirty();
        }
    }

    // ---- teams -----------------------------------------------------------------------------

    /// `Scoreboard.addPlayerTeam` + `onTeamAdded`.
    pub fn add_player_team(&mut self, team: TeamState) {
        if self.data.team(&team.name).is_some() {
            return;
        }
        let create = packets::team_add_or_modify(&team, Some(Vec::new()));
        self.data.teams.push(team);
        self.outbox.push(create);
        self.set_dirty();
    }

    /// `onTeamChanged`: broadcasts the updated parameters.
    pub fn change_player_team(&mut self, team: TeamState) {
        let Some(slot) = self.data.teams.iter_mut().find(|entry| entry.name == team.name) else {
            return;
        };
        *slot = team.clone();
        self.outbox.push(packets::team_add_or_modify(&team, None));
        self.set_dirty();
    }

    /// `Scoreboard.removePlayerTeam` + `onTeamRemoved` (members leave silently).
    pub fn remove_player_team(&mut self, name: &str) {
        if self.data.team(name).is_none() {
            return;
        }
        self.data.teams.retain(|team| team.name != name);
        self.data.members.retain(|member| member.team != name);
        self.outbox.push(packets::team_remove(name));
        self.set_dirty();
    }

    /// `ServerScoreboard.addPlayerToTeam`: leaves any current team first (broadcasting the
    /// leave), then joins and broadcasts. False if the team does not exist.
    pub fn add_player_to_team(&mut self, player: &str, team: &str) -> bool {
        if self.data.team(team).is_none() {
            return false;
        }
        if let Some(current) = self.data.team_of(player).map(str::to_string) {
            self.remove_player_from_team(player, &current);
        }
        self.data.members.push(TeamMember {
            player: player.to_string(),
            team: team.to_string(),
        });
        self.outbox.push(packets::team_player(team, player, true));
        self.set_dirty();
        true
    }

    /// `ServerScoreboard.removePlayerFromTeam(player, team)`.
    pub fn remove_player_from_team(&mut self, player: &str, team: &str) {
        if self.data.team_of(player) != Some(team) {
            return;
        }
        self.data.members.retain(|member| member.player != player);
        self.outbox.push(packets::team_player(team, player, false));
        self.set_dirty();
    }
}

/// A fresh `Score` (`value 0`, locked, no display or number format).
fn new_score(owner: &str, objective: &str) -> ScoreboardScore {
    ScoreboardScore {
        owner: owner.to_string(),
        objective: objective.to_string(),
        value: 0,
        locked: true,
        display_name: None,
        number_format: None,
    }
}

#[cfg(test)]
mod tests;
