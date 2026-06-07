#![allow(dead_code)]

use std::collections::BTreeSet;
use std::path::Path;

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
            Self::Normal => "minecraft:chests/trial_chambers/reward",
            Self::Ominous => "minecraft:chests/trial_chambers/reward_ominous",
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerDetectorKind {
    NoCreativePlayers,
    IncludingCreativePlayers,
    Sheep,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlayerDetectorCandidate {
    pub uuid: String,
    pub entity_type: String,
    pub distance: f64,
    pub creative: bool,
    pub spectator: bool,
    pub alive: bool,
    pub line_of_sight: bool,
}

pub fn detect_trial_spawner_players(
    kind: PlayerDetectorKind,
    candidates: &[PlayerDetectorCandidate],
    required_player_range: f64,
    require_line_of_sight: bool,
) -> Vec<String> {
    candidates
        .iter()
        .filter(|candidate| {
            candidate.distance < required_player_range
                && (!require_line_of_sight || candidate.line_of_sight)
                && match kind {
                    PlayerDetectorKind::NoCreativePlayers => {
                        candidate.entity_type == "minecraft:player"
                            && !candidate.creative
                            && !candidate.spectator
                    }
                    PlayerDetectorKind::IncludingCreativePlayers => {
                        candidate.entity_type == "minecraft:player" && !candidate.spectator
                    }
                    PlayerDetectorKind::Sheep => {
                        candidate.entity_type == "minecraft:sheep" && candidate.alive
                    }
                }
        })
        .map(|candidate| candidate.uuid.clone())
        .collect()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TrialSpawnerEvent {
    Idle,
    DetectPlayer {
        mode: TrialMode,
        player_count: usize,
    },
    SpawnMob {
        mode: TrialMode,
        entity: String,
    },
    EjectReward {
        item: String,
        loot_table: String,
    },
    Cooldown {
        until_tick: i64,
    },
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
            .filter(|player| {
                !player.spectator && player.distance <= self.config.required_player_range
            })
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

#[derive(Debug, Clone, PartialEq)]
pub struct TrialSpawnerConfigResource {
    pub spawn_range: i32,
    pub total_mobs: f64,
    pub simultaneous_mobs: f64,
    pub total_mobs_added_per_player: f64,
    pub simultaneous_mobs_added_per_player: f64,
    pub ticks_between_spawn: i32,
    pub spawn_potentials: Vec<WeightedTrialSpawnerData>,
    pub loot_tables_to_eject: Vec<WeightedLootTable>,
    pub items_to_drop_when_ominous: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedTrialSpawnerData {
    pub entity_id: String,
    pub entity_data: serde_json::Map<String, serde_json::Value>,
    pub weight: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WeightedLootTable {
    pub id: String,
    pub weight: i32,
}

pub fn parse_trial_spawner_config_resource(
    raw: &str,
) -> Result<TrialSpawnerConfigResource, String> {
    let value: serde_json::Value =
        serde_json::from_str(raw).map_err(|err| format!("invalid trial spawner JSON: {err}"))?;
    let object = value
        .as_object()
        .ok_or_else(|| "trial spawner config must be a JSON object".to_string())?;

    Ok(TrialSpawnerConfigResource {
        spawn_range: optional_i32_range(object, "spawn_range", 4, 1, 128)?,
        total_mobs: optional_f64_min(object, "total_mobs", 6.0, 0.0)?,
        simultaneous_mobs: optional_f64_min(object, "simultaneous_mobs", 2.0, 0.0)?,
        total_mobs_added_per_player: optional_f64_min(
            object,
            "total_mobs_added_per_player",
            2.0,
            0.0,
        )?,
        simultaneous_mobs_added_per_player: optional_f64_min(
            object,
            "simultaneous_mobs_added_per_player",
            1.0,
            0.0,
        )?,
        ticks_between_spawn: optional_i32_range(object, "ticks_between_spawn", 40, 0, i32::MAX)?,
        spawn_potentials: parse_spawn_potentials(object.get("spawn_potentials"))?,
        loot_tables_to_eject: match object.get("loot_tables_to_eject") {
            Some(value) => parse_weighted_loot_tables(value)?,
            None => vec![
                WeightedLootTable {
                    id: "minecraft:spawners/trial_chamber/consumables".to_string(),
                    weight: 1,
                },
                WeightedLootTable {
                    id: "minecraft:spawners/trial_chamber/key".to_string(),
                    weight: 1,
                },
            ],
        },
        items_to_drop_when_ominous: object
            .get("items_to_drop_when_ominous")
            .map(json_string_value)
            .transpose()?
            .unwrap_or_else(|| {
                "minecraft:spawners/trial_chamber/items_to_drop_when_ominous".to_string()
            }),
    })
}

pub fn load_trial_spawner_config_resource(
    path: impl AsRef<Path>,
) -> Result<TrialSpawnerConfigResource, String> {
    let raw = std::fs::read_to_string(path.as_ref())
        .map_err(|err| format!("failed to read {}: {err}", path.as_ref().display()))?;
    parse_trial_spawner_config_resource(&raw)
}

fn parse_spawn_potentials(
    value: Option<&serde_json::Value>,
) -> Result<Vec<WeightedTrialSpawnerData>, String> {
    let Some(value) = value else {
        return Ok(Vec::new());
    };
    let entries = value
        .as_array()
        .ok_or_else(|| "spawn_potentials must be a list".to_string())?;
    entries
        .iter()
        .map(|entry| {
            let object = entry
                .as_object()
                .ok_or_else(|| "spawn potential must be an object".to_string())?;
            let data = object
                .get("data")
                .and_then(serde_json::Value::as_object)
                .ok_or_else(|| "spawn potential data must be an object".to_string())?;
            let entity = data
                .get("entity")
                .and_then(serde_json::Value::as_object)
                .ok_or_else(|| "spawn potential data.entity must be an object".to_string())?;
            Ok(WeightedTrialSpawnerData {
                entity_id: json_string(entity, "id")?,
                entity_data: entity.clone(),
                weight: required_i32_range(object, "weight", 1, i32::MAX)?,
            })
        })
        .collect()
}

fn parse_weighted_loot_tables(value: &serde_json::Value) -> Result<Vec<WeightedLootTable>, String> {
    let entries = value
        .as_array()
        .ok_or_else(|| "loot_tables_to_eject must be a list".to_string())?;
    entries
        .iter()
        .map(|entry| {
            let object = entry
                .as_object()
                .ok_or_else(|| "loot table entry must be an object".to_string())?;
            Ok(WeightedLootTable {
                id: object
                    .get("data")
                    .map(json_string_value)
                    .transpose()?
                    .ok_or_else(|| "loot table entry requires data".to_string())?,
                weight: required_i32_range(object, "weight", 1, i32::MAX)?,
            })
        })
        .collect()
}

fn optional_i32_range(
    object: &serde_json::Map<String, serde_json::Value>,
    field: &str,
    default: i32,
    min: i32,
    max: i32,
) -> Result<i32, String> {
    match object.get(field) {
        Some(_) => required_i32_range(object, field, min, max),
        None => Ok(default),
    }
}

fn required_i32_range(
    object: &serde_json::Map<String, serde_json::Value>,
    field: &str,
    min: i32,
    max: i32,
) -> Result<i32, String> {
    let value = object
        .get(field)
        .and_then(serde_json::Value::as_i64)
        .ok_or_else(|| format!("{field} must be an integer"))?;
    let value = i32::try_from(value).map_err(|_| format!("{field} is out of i32 range"))?;
    if value < min || value > max {
        return Err(format!("{field} must be in {min}..={max}"));
    }
    Ok(value)
}

fn optional_f64_min(
    object: &serde_json::Map<String, serde_json::Value>,
    field: &str,
    default: f64,
    min: f64,
) -> Result<f64, String> {
    let Some(value) = object.get(field) else {
        return Ok(default);
    };
    let value = value
        .as_f64()
        .ok_or_else(|| format!("{field} must be numeric"))?;
    if value < min {
        return Err(format!("{field} must be >= {min}"));
    }
    Ok(value)
}

fn json_string(
    object: &serde_json::Map<String, serde_json::Value>,
    field: &str,
) -> Result<String, String> {
    object
        .get(field)
        .map(json_string_value)
        .transpose()?
        .ok_or_else(|| format!("{field} must be a string"))
}

fn json_string_value(value: &serde_json::Value) -> Result<String, String> {
    value
        .as_str()
        .map(ToString::to_string)
        .ok_or_else(|| "value must be a string".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

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
    fn player_detector_variants_match_trial_spawner_java_filters() {
        let candidates = vec![
            PlayerDetectorCandidate {
                uuid: "survival".to_string(),
                entity_type: "minecraft:player".to_string(),
                distance: 13.0,
                creative: false,
                spectator: false,
                alive: true,
                line_of_sight: true,
            },
            PlayerDetectorCandidate {
                uuid: "creative".to_string(),
                entity_type: "minecraft:player".to_string(),
                distance: 13.0,
                creative: true,
                spectator: false,
                alive: true,
                line_of_sight: true,
            },
            PlayerDetectorCandidate {
                uuid: "spectator".to_string(),
                entity_type: "minecraft:player".to_string(),
                distance: 13.0,
                creative: false,
                spectator: true,
                alive: true,
                line_of_sight: true,
            },
            PlayerDetectorCandidate {
                uuid: "blocked".to_string(),
                entity_type: "minecraft:player".to_string(),
                distance: 13.0,
                creative: false,
                spectator: false,
                alive: true,
                line_of_sight: false,
            },
            PlayerDetectorCandidate {
                uuid: "edge".to_string(),
                entity_type: "minecraft:player".to_string(),
                distance: 14.0,
                creative: false,
                spectator: false,
                alive: true,
                line_of_sight: true,
            },
            PlayerDetectorCandidate {
                uuid: "sheep".to_string(),
                entity_type: "minecraft:sheep".to_string(),
                distance: 14.0,
                creative: false,
                spectator: false,
                alive: true,
                line_of_sight: true,
            },
            PlayerDetectorCandidate {
                uuid: "dead-sheep".to_string(),
                entity_type: "minecraft:sheep".to_string(),
                distance: 4.0,
                creative: false,
                spectator: false,
                alive: false,
                line_of_sight: true,
            },
        ];

        assert_eq!(
            detect_trial_spawner_players(
                PlayerDetectorKind::NoCreativePlayers,
                &candidates,
                14.0,
                true
            ),
            vec!["survival".to_string()]
        );
        assert_eq!(
            detect_trial_spawner_players(
                PlayerDetectorKind::IncludingCreativePlayers,
                &candidates,
                14.0,
                true
            ),
            vec!["survival".to_string(), "creative".to_string()]
        );
        assert_eq!(
            detect_trial_spawner_players(
                PlayerDetectorKind::NoCreativePlayers,
                &candidates,
                14.0,
                false
            ),
            vec!["survival".to_string(), "blocked".to_string()]
        );
        assert_eq!(
            detect_trial_spawner_players(PlayerDetectorKind::Sheep, &candidates, 15.0, true),
            vec!["sheep".to_string()]
        );
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
                loot_table: "minecraft:chests/trial_chambers/reward".to_string()
            }
        );
        assert_eq!(
            state.cooldown_until,
            20 + i64::from(default_target_cooldown())
        );
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
                loot_table: "minecraft:chests/trial_chambers/reward".to_string()
            }
        );
        assert_eq!(
            normal.try_unlock("player-a", "minecraft:trial_key"),
            VaultUnlockResult::AlreadyRewarded
        );

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

    #[test]
    fn trial_spawner_registry_resources_decode_all_vanilla_configs() {
        let root = std::path::Path::new("../decompiled-server-26.1.2/data/minecraft/trial_spawner");
        let mut paths = Vec::new();
        collect_json_paths(root, &mut paths);
        paths.sort();

        assert_eq!(paths.len(), 28);
        let mut entity_ids = BTreeSet::new();
        let mut ominous_overrides = 0;
        for path in &paths {
            let config = load_trial_spawner_config_resource(path).unwrap_or_else(|err| {
                panic!("{} failed to decode: {err}", path.display());
            });
            assert!((1..=128).contains(&config.spawn_range));
            assert!(
                !config.spawn_potentials.is_empty(),
                "{} has no spawn potentials",
                path.display()
            );
            assert!(
                !config.loot_tables_to_eject.is_empty(),
                "{} has no eject loot tables",
                path.display()
            );
            for potential in config.spawn_potentials {
                assert!(potential.entity_id.starts_with("minecraft:"));
                assert!(potential.weight > 0);
                entity_ids.insert(potential.entity_id);
            }
            if path.ends_with("ominous.json") {
                ominous_overrides += 1;
                assert!(
                    config
                        .loot_tables_to_eject
                        .iter()
                        .any(|table| table.id.contains("/ominous/")),
                    "{} should use ominous ejection loot",
                    path.display()
                );
            }
        }

        assert_eq!(ominous_overrides, 14);
        assert!(entity_ids.contains("minecraft:breeze"));
        assert!(entity_ids.contains("minecraft:zombie"));
        assert!(entity_ids.contains("minecraft:slime"));
    }

    #[test]
    fn trial_spawner_resource_defaults_and_range_checks_match_java_codec() {
        let config = parse_trial_spawner_config_resource(
            r#"{"spawn_potentials":[{"data":{"entity":{"id":"minecraft:zombie"}},"weight":1}]}"#,
        )
        .unwrap();

        assert_eq!(config.spawn_range, 4);
        assert_eq!(config.total_mobs, 6.0);
        assert_eq!(config.simultaneous_mobs, 2.0);
        assert_eq!(config.total_mobs_added_per_player, 2.0);
        assert_eq!(config.simultaneous_mobs_added_per_player, 1.0);
        assert_eq!(config.ticks_between_spawn, 40);
        assert_eq!(config.loot_tables_to_eject.len(), 2);
        assert_eq!(
            config.items_to_drop_when_ominous,
            "minecraft:spawners/trial_chamber/items_to_drop_when_ominous"
        );

        assert!(parse_trial_spawner_config_resource(r#"{"spawn_range":0}"#).is_err());
        assert!(parse_trial_spawner_config_resource(r#"{"ticks_between_spawn":-1}"#).is_err());
        assert!(parse_trial_spawner_config_resource(r#"{"total_mobs":-0.1}"#).is_err());
    }

    fn collect_json_paths(root: &std::path::Path, paths: &mut Vec<std::path::PathBuf>) {
        for entry in fs::read_dir(root).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_dir() {
                collect_json_paths(&path, paths);
            } else if path.extension().and_then(|ext| ext.to_str()) == Some("json") {
                paths.push(path);
            }
        }
    }
}
