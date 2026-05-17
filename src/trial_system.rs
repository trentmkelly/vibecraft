#![allow(dead_code)]

use std::collections::BTreeSet;

use crate::spawning::{
    validate_trial_spawner_spawn, TrialSpawnerConfigSummary, TrialSpawnerRejection,
    TrialSpawnerSpawnContext, TRIAL_SPAWNER_DEFAULT_TARGET_COOLDOWN,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrialMode {
    Normal,
    Ominous,
}

impl TrialMode {
    pub fn key_item(self) -> &'static str {
        match self {
            Self::Normal => "minecraft:trial_key",
            Self::Ominous => "minecraft:ominous_trial_key",
        }
    }

    pub fn reward_table(self) -> &'static str {
        match self {
            Self::Normal => "minecraft:trial_chambers/reward",
            Self::Ominous => "minecraft:trial_chambers/reward_ominous",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrialPlayer {
    pub uuid: String,
    pub has_trial_omen: bool,
    pub spectator: bool,
    pub distance: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TrialSpawnerEvent {
    Idle,
    DetectPlayer { mode: TrialMode, player_count: usize },
    SpawnMob { mode: TrialMode, entity: String },
    EjectReward { item: String, loot_table: String },
    Cooldown { until_tick: i64 },
    Rejected(TrialSpawnerRejection),
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrialSpawnerState {
    pub config: TrialSpawnerConfigSummary,
    pub mode: TrialMode,
    pub cooldown_until: i64,
    pub spawned_mobs: i32,
    pub total_mobs: i32,
    pub players_seen: BTreeSet<String>,
    pub reward_ejected: bool,
}

impl TrialSpawnerState {
    pub fn new(config: TrialSpawnerConfigSummary) -> Self {
        Self {
            config,
            mode: TrialMode::Normal,
            cooldown_until: 0,
            spawned_mobs: 0,
            total_mobs: 0,
            players_seen: BTreeSet::new(),
            reward_ejected: false,
        }
    }

    pub fn tick(
        &mut self,
        game_time: i64,
        players: &[TrialPlayer],
        spawn_context: TrialSpawnerSpawnContext,
    ) -> TrialSpawnerEvent {
        if game_time < self.cooldown_until {
            return TrialSpawnerEvent::Cooldown {
                until_tick: self.cooldown_until,
            };
        }

        let eligible: Vec<&TrialPlayer> = players
            .iter()
            .filter(|player| !player.spectator && player.distance <= self.config.required_player_range)
            .collect();
        if eligible.is_empty() {
            return TrialSpawnerEvent::Idle;
        }
        for player in &eligible {
            self.players_seen.insert(player.uuid.clone());
        }
        if eligible.iter().any(|player| player.has_trial_omen) {
            self.mode = TrialMode::Ominous;
        }
        if self.spawned_mobs == 0 {
            return TrialSpawnerEvent::DetectPlayer {
                mode: self.mode,
                player_count: eligible.len(),
            };
        }

        match validate_trial_spawner_spawn(self.config, spawn_context) {
            Ok(()) => {
                self.spawned_mobs += 1;
                self.total_mobs += 1;
                TrialSpawnerEvent::SpawnMob {
                    mode: self.mode,
                    entity: if self.mode == TrialMode::Ominous {
                        "minecraft:breeze".to_string()
                    } else {
                        "minecraft:trial_mob".to_string()
                    },
                }
            }
            Err(rejection) => TrialSpawnerEvent::Rejected(rejection),
        }
    }

    pub fn complete_wave(&mut self, game_time: i64) -> TrialSpawnerEvent {
        self.cooldown_until = game_time + self.config.target_cooldown_length as i64;
        self.reward_ejected = true;
        TrialSpawnerEvent::EjectReward {
            item: self.mode.key_item().to_string(),
            loot_table: self.mode.reward_table().to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VaultState {
    pub mode: TrialMode,
    pub reward_table: String,
    pub unlocking_players: BTreeSet<String>,
    pub rewarded_players: BTreeSet<String>,
    pub cooldown_ticks: i32,
}

impl VaultState {
    pub fn new(mode: TrialMode) -> Self {
        Self {
            mode,
            reward_table: mode.reward_table().to_string(),
            unlocking_players: BTreeSet::new(),
            rewarded_players: BTreeSet::new(),
            cooldown_ticks: 0,
        }
    }

    pub fn try_unlock(&mut self, player: impl Into<String>, key_item: &str) -> VaultUnlockResult {
        let player = player.into();
        if self.rewarded_players.contains(&player) {
            return VaultUnlockResult::AlreadyRewarded;
        }
        if key_item != self.mode.key_item() {
            return VaultUnlockResult::WrongKey {
                expected: self.mode.key_item(),
            };
        }
        self.unlocking_players.insert(player.clone());
        self.rewarded_players.insert(player.clone());
        self.cooldown_ticks = 14;
        VaultUnlockResult::Unlocked {
            player,
            loot_table: self.reward_table.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VaultUnlockResult {
    Unlocked { player: String, loot_table: String },
    WrongKey { expected: &'static str },
    AlreadyRewarded,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OminousBottleUse {
    pub amplifier: i32,
    pub duration_ticks: i32,
}

impl OminousBottleUse {
    pub fn trial_omen(amplifier: i32) -> Self {
        Self {
            amplifier: amplifier.clamp(0, 4),
            duration_ticks: 120_000,
        }
    }
}

pub fn default_trial_spawner_state() -> TrialSpawnerState {
    TrialSpawnerState::new(TrialSpawnerConfigSummary::default())
}

pub fn default_target_cooldown() -> i32 {
    TRIAL_SPAWNER_DEFAULT_TARGET_COOLDOWN
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_context() -> TrialSpawnerSpawnContext {
        TrialSpawnerSpawnContext {
            spawner_blocks_work: true,
            override_peaceful_and_mob_spawn_rule: false,
            peaceful: false,
            spawn_mobs_rule: true,
            collision_free: true,
            line_of_sight: true,
            placement_rules_ok: true,
            custom_rules_ok: true,
            obstruction_free: true,
            tracked_mobs: 0,
        }
    }

    #[test]
    fn trial_spawner_detects_players_ominous_mode_and_spawns_with_existing_validation() {
        let mut state = default_trial_spawner_state();
        let players = vec![TrialPlayer {
            uuid: "player-a".to_string(),
            has_trial_omen: true,
            spectator: false,
            distance: 4,
        }];

        assert_eq!(
            state.tick(100, &players, valid_context()),
            TrialSpawnerEvent::DetectPlayer {
                mode: TrialMode::Ominous,
                player_count: 1
            }
        );

        state.spawned_mobs = 1;
        assert_eq!(
            state.tick(101, &players, valid_context()),
            TrialSpawnerEvent::SpawnMob {
                mode: TrialMode::Ominous,
                entity: "minecraft:breeze".to_string()
            }
        );
        assert!(state.players_seen.contains("player-a"));
    }

    #[test]
    fn trial_spawner_rejections_and_cooldown_use_vanilla_config_values() {
        let mut state = default_trial_spawner_state();
        state.spawned_mobs = 1;
        let players = vec![TrialPlayer {
            uuid: "player-a".to_string(),
            has_trial_omen: false,
            spectator: false,
            distance: 4,
        }];
        let mut blocked = valid_context();
        blocked.tracked_mobs = state.config.simultaneous_mobs;

        assert_eq!(
            state.tick(10, &players, blocked),
            TrialSpawnerEvent::Rejected(TrialSpawnerRejection::MobCap)
        );

        let reward = state.complete_wave(20);
        assert_eq!(
            reward,
            TrialSpawnerEvent::EjectReward {
                item: "minecraft:trial_key".to_string(),
                loot_table: "minecraft:trial_chambers/reward".to_string()
            }
        );
        assert_eq!(state.cooldown_until, 20 + i64::from(default_target_cooldown()));
        assert!(matches!(
            state.tick(21, &players, valid_context()),
            TrialSpawnerEvent::Cooldown { .. }
        ));
    }

    #[test]
    fn vault_unlocks_once_per_player_and_requires_matching_trial_key() {
        let mut normal = VaultState::new(TrialMode::Normal);
        assert_eq!(
            normal.try_unlock("player-a", "minecraft:ominous_trial_key"),
            VaultUnlockResult::WrongKey {
                expected: "minecraft:trial_key"
            }
        );
        assert_eq!(
            normal.try_unlock("player-a", "minecraft:trial_key"),
            VaultUnlockResult::Unlocked {
                player: "player-a".to_string(),
                loot_table: "minecraft:trial_chambers/reward".to_string()
            }
        );
        assert_eq!(normal.try_unlock("player-a", "minecraft:trial_key"), VaultUnlockResult::AlreadyRewarded);

        let mut ominous = VaultState::new(TrialMode::Ominous);
        assert!(matches!(
            ominous.try_unlock("player-b", "minecraft:ominous_trial_key"),
            VaultUnlockResult::Unlocked { .. }
        ));
    }

    #[test]
    fn ominous_bottle_clamps_amplifier_and_uses_trial_omen_duration() {
        assert_eq!(
            OminousBottleUse::trial_omen(9),
            OminousBottleUse {
                amplifier: 4,
                duration_ticks: 120_000
            }
        );
    }
}
