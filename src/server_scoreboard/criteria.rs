//! Criteria-driven scores (Java `ObjectiveCriteria`, `Scoreboard.forAllObjectives`,
//! `ServerPlayer.doTick`/`die`).
//!
//! `dummy` and `trigger` objectives are only changed by commands. The built-in criteria update
//! automatically: `deathCount` when the player dies (`ServerPlayer.die`), and `health`, `food`,
//! `air`, `armor`, `xp` and `level` whenever the tracked value differs from what was last
//! recorded (`ServerPlayer.doTick`).
// TODO(pvp-kill-criteria): `playerKillCount`, `totalKillCount`, `teamkill.*` and
// `killedByTeam.*` (`ServerPlayer.awardKillScore`) need a live player-kills-entity event; there
// is no player attack pipeline yet, so nothing raises them.
// TODO(stat-criteria): `<stat_type>:<value>` criteria (`ServerPlayer.awardStat`) need the live
// statistics events wired to the scoreboard.

use super::ServerScoreboard;

/// `ObjectiveCriteria.DEATH_COUNT`.
pub const DEATH_COUNT: &str = "deathCount";
const HEALTH: &str = "health";
const FOOD: &str = "food";
const AIR: &str = "air";
const ARMOR: &str = "armor";
const EXPERIENCE: &str = "xp";
const LEVEL: &str = "level";

impl ServerScoreboard {
    /// `Scoreboard.forAllObjectives(criteria, holder, score -> score.set(f(score.get())))`:
    /// every objective using `criteria` gets the holder's score created if needed and updated.
    pub fn for_all_objectives(
        &mut self,
        criteria: &str,
        holder: &str,
        holder_display_name: Option<&str>,
        update: impl Fn(i32) -> i32,
    ) {
        let objectives: Vec<String> = self
            .data
            .objectives
            .iter()
            .filter(|objective| objective.criteria == criteria)
            .map(|objective| objective.name.clone())
            .collect();
        for objective in objectives {
            let current = self.score_value(holder, &objective).unwrap_or(0);
            self.set_score(holder, &objective, update(current), holder_display_name);
        }
    }

    /// Runs [`RecordedVitals::update`] for `holder`.
    pub fn update_vitals(&mut self, recorded: &mut RecordedVitals, holder: &str, vitals: PlayerVitals) {
        recorded.update(self, holder, vitals);
    }

    /// `forAllObjectives(DEATH_COUNT, player, ScoreAccess::increment)`.
    pub fn increment_death_count(&mut self, holder: &str) {
        self.for_all_objectives(DEATH_COUNT, holder, Some(holder), |score| score.wrapping_add(1));
    }
}

/// The values `ServerPlayer.doTick` compares against its `lastRecorded*` fields.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlayerVitals {
    /// `getHealth() + getAbsorptionAmount()`.
    pub health_and_absorption: f32,
    pub food_level: i32,
    pub air_supply: i32,
    /// `getArmorValue()`.
    pub armor: f32,
    /// `totalExperience`.
    pub experience: i32,
    pub level: i32,
}

/// Java's `lastRecordedHealthAndAbsorption`, `lastRecordedFoodLevel`, ... sentinels: they start
/// at values no real vitals can equal, so the first tick publishes every criterion.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RecordedVitals {
    health_and_absorption: f32,
    food_level: i32,
    air_supply: i32,
    armor: f32,
    experience: i32,
    level: i32,
}

impl Default for RecordedVitals {
    fn default() -> Self {
        Self {
            health_and_absorption: -1.0e8,
            food_level: -99_999_999,
            air_supply: -99_999_999,
            armor: -99_999_999.0,
            experience: -99_999_999,
            level: -99_999_999,
        }
    }
}

impl RecordedVitals {
    /// `ServerPlayer.doTick`'s six `updateScoreForCriteria` blocks, in order.
    pub fn update(
        &mut self,
        scoreboard: &mut ServerScoreboard,
        holder: &str,
        current: PlayerVitals,
    ) {
        let mut publish = |criteria: &str, value: i32| {
            scoreboard.for_all_objectives(criteria, holder, Some(holder), |_| value);
        };
        if current.health_and_absorption != self.health_and_absorption {
            self.health_and_absorption = current.health_and_absorption;
            publish(HEALTH, current.health_and_absorption.ceil() as i32);
        }
        if current.food_level != self.food_level {
            self.food_level = current.food_level;
            publish(FOOD, current.food_level);
        }
        if current.air_supply != self.air_supply {
            self.air_supply = current.air_supply;
            publish(AIR, current.air_supply);
        }
        if current.armor != self.armor {
            self.armor = current.armor;
            publish(ARMOR, current.armor.ceil() as i32);
        }
        if current.experience != self.experience {
            self.experience = current.experience;
            publish(EXPERIENCE, current.experience);
        }
        if current.level != self.level {
            self.level = current.level;
            publish(LEVEL, current.level);
        }
    }
}
