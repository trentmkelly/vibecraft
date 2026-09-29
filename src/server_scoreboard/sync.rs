//! Folding a command's scoreboard edits back into the live [`ServerScoreboard`].
//!
//! The `/scoreboard`, `/team` and `/trigger` implementations edit a flat
//! [`ScoreboardData`] snapshot held in the command state (seeded from the live scoreboard before
//! each command). [`ServerScoreboard::apply_command_changes`] diffs that snapshot against the one
//! it was seeded from and replays every difference through the same mutation hooks Java's
//! `Scoreboard` methods call, so the resulting packets, tracking and dirty state match what the
//! Java command would have produced. Diffing against the seed (not the current live state) keeps
//! concurrent changes made elsewhere while the command ran intact.

use crate::command::ScoreboardScore;
use super::display_slot::DisplaySlot;

use super::{ScoreboardData, ServerScoreboard};

impl ServerScoreboard {
    /// Replays `before -> after` onto the live scoreboard, in Java's hook order: objectives,
    /// display slots, scores, teams, then team membership.
    pub fn apply_command_changes(&mut self, before: &ScoreboardData, after: &ScoreboardData) {
        self.apply_objective_changes(before, after);
        self.apply_display_slot_changes(before, after);
        self.apply_score_changes(before, after);
        self.apply_team_changes(before, after);
        self.apply_membership_changes(before, after);
    }

    fn apply_objective_changes(&mut self, before: &ScoreboardData, after: &ScoreboardData) {
        for old in &before.objectives {
            if after.objective(&old.name).is_none() {
                self.remove_objective(&old.name);
            }
        }
        for new in &after.objectives {
            match before.objective(&new.name) {
                None => self.add_objective(new.clone()),
                Some(old) if old != new => self.change_objective(new.clone()),
                Some(_) => {}
            }
        }
    }

    fn apply_display_slot_changes(&mut self, before: &ScoreboardData, after: &ScoreboardData) {
        for slot in DisplaySlot::VALUES {
            let name = slot.serialized_name();
            let wanted = after.display_objective(name);
            if wanted != before.display_objective(name) {
                self.set_display_objective(slot, wanted);
            }
        }
    }

    fn apply_score_changes(&mut self, before: &ScoreboardData, after: &ScoreboardData) {
        self.apply_score_removals(before, after);
        for score in &after.scores {
            if before.score(&score.owner, &score.objective) != Some(score) {
                self.upsert_score(score);
            }
        }
    }

    /// `resetAllPlayerScores` for holders that lost every score, `resetSinglePlayerScore` for
    /// individual scores of holders that keep others. Scores of removed objectives are gone
    /// already.
    fn apply_score_removals(&mut self, before: &ScoreboardData, after: &ScoreboardData) {
        let mut handled: Vec<&str> = Vec::new();
        for old in &before.scores {
            let removed = after.score(&old.owner, &old.objective).is_none()
                && after.objective(&old.objective).is_some();
            if !removed || handled.contains(&old.owner.as_str()) {
                continue;
            }
            if after.scores.iter().all(|score| score.owner != old.owner) {
                handled.push(&old.owner);
                self.reset_all_player_scores(&old.owner);
            } else {
                self.reset_single_player_score(&old.owner, &old.objective);
            }
        }
    }

    /// Stores a created/changed score: a value, display or format change (or a new score)
    /// syncs to players like `ScoreAccess.sendScoreToPlayers`; a lock-only change just dirties.
    fn upsert_score(&mut self, new: &ScoreboardScore) {
        if self.data.objective(&new.objective).is_none() {
            return;
        }
        let existing = self
            .data
            .scores
            .iter_mut()
            .find(|score| score.owner == new.owner && score.objective == new.objective);
        match existing {
            Some(score) => {
                let syncs = score.value != new.value
                    || score.display_name != new.display_name
                    || score.number_format != new.number_format;
                let lock_only = !syncs && score.locked != new.locked;
                *score = new.clone();
                if syncs {
                    self.on_score_changed(&new.owner, &new.objective);
                } else if lock_only {
                    self.set_dirty();
                }
            }
            None => {
                self.data.scores.push(new.clone());
                self.on_score_changed(&new.owner, &new.objective);
            }
        }
    }

    fn apply_team_changes(&mut self, before: &ScoreboardData, after: &ScoreboardData) {
        for old in &before.teams {
            if after.team(&old.name).is_none() {
                self.remove_player_team(&old.name);
            }
        }
        for new in &after.teams {
            match before.team(&new.name) {
                None => self.add_player_team(new.clone()),
                Some(old) if old != new => self.change_player_team(new.clone()),
                Some(_) => {}
            }
        }
    }

    fn apply_membership_changes(&mut self, before: &ScoreboardData, after: &ScoreboardData) {
        for old in &before.members {
            if after.team_of(&old.player).is_none() && after.team(&old.team).is_some() {
                self.remove_player_from_team(&old.player, &old.team);
            }
        }
        for member in &after.members {
            if before.team_of(&member.player) != Some(member.team.as_str()) {
                self.add_player_to_team(&member.player, &member.team);
            }
        }
    }
}
