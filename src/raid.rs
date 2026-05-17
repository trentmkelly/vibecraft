#![allow(dead_code)]

use std::collections::{BTreeMap, BTreeSet};

use crate::block_update::BlockPos;
use crate::network::codec::Uuid;

pub const RAID_FILE_ID: &str = "minecraft:raids";
pub const MAX_RAID_OMEN_LEVEL: i32 = 5;
pub const RAID_OMEN_DURATION_TICKS: i32 = 600;
pub const RAID_DIRTY_INTERVAL_TICKS: i32 = 200;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Difficulty {
    Peaceful,
    Easy,
    Normal,
    Hard,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RaidStatus {
    Ongoing,
    Victory,
    Loss,
    Stopped,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RaidState {
    pub center: BlockPos,
    pub active: bool,
    pub started: bool,
    pub first_wave_spawned: bool,
    pub raid_omen_level: i32,
    pub status: RaidStatus,
    pub heroes_of_the_village: BTreeSet<UuidKey>,
}

impl RaidState {
    pub fn new(center: BlockPos) -> Self {
        Self {
            center,
            active: true,
            started: false,
            first_wave_spawned: false,
            raid_omen_level: 0,
            status: RaidStatus::Ongoing,
            heroes_of_the_village: BTreeSet::new(),
        }
    }

    pub fn is_active(&self) -> bool {
        self.active && !matches!(self.status, RaidStatus::Stopped)
    }

    pub fn stop(&mut self) {
        self.active = false;
        self.status = RaidStatus::Stopped;
    }

    pub fn absorb_raid_omen(&mut self, amplifier: Option<i32>) -> RaidOmenAbsorbResult {
        let Some(amplifier) = amplifier else {
            return RaidOmenAbsorbResult::NoEffect;
        };
        let before = self.raid_omen_level;
        self.raid_omen_level = (self.raid_omen_level + amplifier + 1).clamp(0, MAX_RAID_OMEN_LEVEL);
        RaidOmenAbsorbResult::Absorbed {
            before,
            after: self.raid_omen_level,
            award_raid_trigger: !self.first_wave_spawned,
        }
    }

    pub fn add_hero_of_the_village(&mut self, uuid: Uuid) {
        self.heroes_of_the_village.insert(UuidKey::from(uuid));
    }

    pub fn num_groups(difficulty: Difficulty) -> i32 {
        match difficulty {
            Difficulty::Peaceful => 0,
            Difficulty::Easy => 3,
            Difficulty::Normal => 5,
            Difficulty::Hard => 7,
        }
    }

    pub fn enchant_odds(&self) -> f32 {
        match self.raid_omen_level {
            2 => 0.1,
            3 => 0.25,
            4 => 0.5,
            5 => 0.75,
            _ => 0.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RaidOmenAbsorbResult {
    NoEffect,
    Absorbed {
        before: i32,
        after: i32,
        award_raid_trigger: bool,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RaidStore {
    raids: BTreeMap<i32, RaidState>,
    next_id: i32,
    tick: i32,
    dirty: bool,
}

impl RaidStore {
    pub fn new() -> Self {
        Self {
            raids: BTreeMap::new(),
            next_id: 1,
            tick: 0,
            dirty: true,
        }
    }

    pub fn tick(&mut self, raids_rule: bool) -> RaidStoreTick {
        self.tick += 1;
        let mut removed = Vec::new();
        for (id, raid) in self.raids.iter_mut() {
            if !raids_rule {
                raid.stop();
            }
            if matches!(raid.status, RaidStatus::Stopped) {
                removed.push(*id);
            }
        }
        for id in &removed {
            self.raids.remove(id);
        }
        if !removed.is_empty() || self.tick % RAID_DIRTY_INTERVAL_TICKS == 0 {
            self.dirty = true;
        }
        RaidStoreTick {
            tick: self.tick,
            removed,
            dirty: self.dirty,
        }
    }

    pub fn create_or_extend_raid(
        &mut self,
        player: RaidPlayerState,
        raid_position: BlockPos,
        village_pois: &[BlockPos],
        existing_raid_id: Option<i32>,
    ) -> CreateRaidResult {
        if player.spectator {
            return CreateRaidResult::Rejected(CreateRaidRejectReason::Spectator);
        }
        if !player.raids_rule {
            return CreateRaidResult::Rejected(CreateRaidRejectReason::RaidsDisabled);
        }
        if !player.can_start_raid_here {
            return CreateRaidResult::Rejected(CreateRaidRejectReason::EnvironmentDisabled);
        }

        let center = raid_center(raid_position, village_pois);
        let id = existing_raid_id
            .filter(|id| self.raids.contains_key(id))
            .unwrap_or_else(|| self.unique_id());
        let raid = self
            .raids
            .entry(id)
            .or_insert_with(|| RaidState::new(center));
        if !raid.started {
            raid.center = center;
        }
        let absorption = if !raid.started || raid.raid_omen_level < MAX_RAID_OMEN_LEVEL {
            raid.absorb_raid_omen(player.raid_omen_amplifier)
        } else {
            RaidOmenAbsorbResult::NoEffect
        };
        self.dirty = true;
        CreateRaidResult::CreatedOrExtended {
            id,
            center: raid.center,
            absorption,
        }
    }

    pub fn get(&self, id: i32) -> Option<&RaidState> {
        self.raids.get(&id)
    }

    pub fn get_nearby_raid(&self, pos: BlockPos, max_dist_sqr: i32) -> Option<(i32, &RaidState)> {
        self.raids
            .iter()
            .filter(|(_, raid)| raid.is_active())
            .filter_map(|(id, raid)| {
                let dist = dist_sqr(pos, raid.center);
                (dist < max_dist_sqr).then_some((*id, raid, dist))
            })
            .min_by_key(|(_, _, dist)| *dist)
            .map(|(id, raid, _)| (id, raid))
    }

    pub fn next_id(&self) -> i32 {
        self.next_id
    }

    pub fn tick_count(&self) -> i32 {
        self.tick
    }

    fn unique_id(&mut self) -> i32 {
        self.next_id += 1;
        self.next_id
    }
}

impl Default for RaidStore {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RaidStoreTick {
    pub tick: i32,
    pub removed: Vec<i32>,
    pub dirty: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RaidPlayerState {
    pub spectator: bool,
    pub raids_rule: bool,
    pub can_start_raid_here: bool,
    pub raid_omen_amplifier: Option<i32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CreateRaidRejectReason {
    Spectator,
    RaidsDisabled,
    EnvironmentDisabled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CreateRaidResult {
    Rejected(CreateRaidRejectReason),
    CreatedOrExtended {
        id: i32,
        center: BlockPos,
        absorption: RaidOmenAbsorbResult,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OmenEffectAction {
    KeepEffect,
    ConvertBadOmenToRaidOmen {
        duration_ticks: i32,
        amplifier: i32,
        raid_omen_position: BlockPos,
    },
    CreateOrExtendRaid {
        raid_omen_position: BlockPos,
    },
}

pub fn bad_omen_tick(
    player_spectator: bool,
    difficulty: Difficulty,
    in_village: bool,
    current_raid_omen_level: Option<i32>,
    player_pos: BlockPos,
    amplifier: i32,
) -> OmenEffectAction {
    if !player_spectator
        && difficulty != Difficulty::Peaceful
        && in_village
        && current_raid_omen_level.unwrap_or(-1) < MAX_RAID_OMEN_LEVEL
    {
        OmenEffectAction::ConvertBadOmenToRaidOmen {
            duration_ticks: RAID_OMEN_DURATION_TICKS,
            amplifier,
            raid_omen_position: player_pos,
        }
    } else {
        OmenEffectAction::KeepEffect
    }
}

pub fn raid_omen_tick(
    remaining_duration: i32,
    player_spectator: bool,
    raid_omen_position: Option<BlockPos>,
) -> OmenEffectAction {
    if remaining_duration == 1 && !player_spectator {
        if let Some(raid_omen_position) = raid_omen_position {
            return OmenEffectAction::CreateOrExtendRaid { raid_omen_position };
        }
    }
    OmenEffectAction::KeepEffect
}

pub fn raid_center(raid_position: BlockPos, village_pois: &[BlockPos]) -> BlockPos {
    if village_pois.is_empty() {
        return raid_position;
    }
    let count = village_pois.len() as i32;
    BlockPos {
        x: village_pois.iter().map(|pos| pos.x).sum::<i32>() / count,
        y: village_pois.iter().map(|pos| pos.y).sum::<i32>() / count,
        z: village_pois.iter().map(|pos| pos.z).sum::<i32>() / count,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct UuidKey([u8; 16]);

impl From<Uuid> for UuidKey {
    fn from(uuid: Uuid) -> Self {
        Self(uuid.0)
    }
}

fn dist_sqr(left: BlockPos, right: BlockPos) -> i32 {
    let dx = left.x - right.x;
    let dy = left.y - right.y;
    let dz = left.z - right.z;
    dx * dx + dy * dy + dz * dz
}

#[cfg(test)]
mod tests {
    use super::{
        bad_omen_tick, raid_center, raid_omen_tick, CreateRaidRejectReason, CreateRaidResult,
        Difficulty, OmenEffectAction, RaidOmenAbsorbResult, RaidPlayerState, RaidState, RaidStatus,
        RaidStore, MAX_RAID_OMEN_LEVEL, RAID_DIRTY_INTERVAL_TICKS, RAID_FILE_ID,
        RAID_OMEN_DURATION_TICKS,
    };
    use crate::block_update::BlockPos;
    use crate::network::codec::Uuid;

    fn pos(x: i32, y: i32, z: i32) -> BlockPos {
        BlockPos { x, y, z }
    }

    #[test]
    fn bad_omen_converts_to_raid_omen_only_for_non_spectator_village_non_peaceful() {
        assert_eq!(
            bad_omen_tick(false, Difficulty::Normal, true, None, pos(1, 64, 1), 2),
            OmenEffectAction::ConvertBadOmenToRaidOmen {
                duration_ticks: RAID_OMEN_DURATION_TICKS,
                amplifier: 2,
                raid_omen_position: pos(1, 64, 1)
            }
        );
        assert_eq!(
            bad_omen_tick(false, Difficulty::Peaceful, true, None, pos(1, 64, 1), 2),
            OmenEffectAction::KeepEffect
        );
        assert_eq!(
            bad_omen_tick(
                false,
                Difficulty::Hard,
                true,
                Some(MAX_RAID_OMEN_LEVEL),
                pos(1, 64, 1),
                2
            ),
            OmenEffectAction::KeepEffect
        );
    }

    #[test]
    fn raid_omen_fires_only_on_final_tick_and_clears_position_after_creation() {
        assert_eq!(
            raid_omen_tick(2, false, Some(pos(3, 64, 3))),
            OmenEffectAction::KeepEffect
        );
        assert_eq!(
            raid_omen_tick(1, false, Some(pos(3, 64, 3))),
            OmenEffectAction::CreateOrExtendRaid {
                raid_omen_position: pos(3, 64, 3)
            }
        );
        assert_eq!(
            raid_omen_tick(1, true, Some(pos(3, 64, 3))),
            OmenEffectAction::KeepEffect
        );
    }

    #[test]
    fn raid_absorbs_omen_amplifier_clamps_and_awards_only_before_first_wave() {
        let mut raid = RaidState::new(pos(0, 64, 0));
        assert_eq!(
            raid.absorb_raid_omen(Some(2)),
            RaidOmenAbsorbResult::Absorbed {
                before: 0,
                after: 3,
                award_raid_trigger: true
            }
        );
        raid.first_wave_spawned = true;
        assert_eq!(
            raid.absorb_raid_omen(Some(9)),
            RaidOmenAbsorbResult::Absorbed {
                before: 3,
                after: MAX_RAID_OMEN_LEVEL,
                award_raid_trigger: false
            }
        );
        assert_eq!(raid.absorb_raid_omen(None), RaidOmenAbsorbResult::NoEffect);
    }

    #[test]
    fn raid_difficulty_groups_and_enchant_odds_match_vanilla() {
        assert_eq!(RaidState::num_groups(Difficulty::Peaceful), 0);
        assert_eq!(RaidState::num_groups(Difficulty::Easy), 3);
        assert_eq!(RaidState::num_groups(Difficulty::Normal), 5);
        assert_eq!(RaidState::num_groups(Difficulty::Hard), 7);
        let mut raid = RaidState::new(pos(0, 64, 0));
        assert_eq!(raid.enchant_odds(), 0.0);
        raid.raid_omen_level = 2;
        assert_eq!(raid.enchant_odds(), 0.1);
        raid.raid_omen_level = 5;
        assert_eq!(raid.enchant_odds(), 0.75);
    }

    #[test]
    fn raid_store_creates_extends_uses_average_village_center_and_unique_ids() {
        assert_eq!(RAID_FILE_ID, "minecraft:raids");
        let mut store = RaidStore::new();
        let player = RaidPlayerState {
            spectator: false,
            raids_rule: true,
            can_start_raid_here: true,
            raid_omen_amplifier: Some(1),
        };
        let result = store.create_or_extend_raid(
            player,
            pos(100, 64, 100),
            &[pos(96, 64, 96), pos(104, 64, 104)],
            None,
        );

        assert_eq!(
            result,
            CreateRaidResult::CreatedOrExtended {
                id: 2,
                center: pos(100, 64, 100),
                absorption: RaidOmenAbsorbResult::Absorbed {
                    before: 0,
                    after: 2,
                    award_raid_trigger: true
                }
            }
        );
        assert_eq!(store.next_id(), 2);
        assert_eq!(
            store
                .get_nearby_raid(pos(101, 64, 101), 64)
                .map(|(id, _)| id),
            Some(2)
        );

        let extended = store.create_or_extend_raid(player, pos(120, 64, 120), &[], Some(2));
        assert!(matches!(
            extended,
            CreateRaidResult::CreatedOrExtended { id: 2, .. }
        ));
        assert_eq!(store.get(2).unwrap().raid_omen_level, 4);
    }

    #[test]
    fn raid_store_rejects_invalid_players_and_removes_stopped_raids_on_tick() {
        let mut store = RaidStore::new();
        let spectator = RaidPlayerState {
            spectator: true,
            raids_rule: true,
            can_start_raid_here: true,
            raid_omen_amplifier: Some(0),
        };
        assert_eq!(
            store.create_or_extend_raid(spectator, pos(0, 64, 0), &[], None),
            CreateRaidResult::Rejected(CreateRaidRejectReason::Spectator)
        );

        let player = RaidPlayerState {
            spectator: false,
            raids_rule: true,
            can_start_raid_here: true,
            raid_omen_amplifier: Some(0),
        };
        store.create_or_extend_raid(player, pos(0, 64, 0), &[], None);
        let tick = store.tick(false);
        assert_eq!(tick.removed, vec![2]);
        assert!(store.get(2).is_none());

        for _ in 1..RAID_DIRTY_INTERVAL_TICKS {
            store.tick(true);
        }
        assert_eq!(store.tick_count(), RAID_DIRTY_INTERVAL_TICKS);
    }

    #[test]
    fn raid_tracks_heroes_and_stop_status() {
        let mut raid = RaidState::new(pos(0, 64, 0));
        raid.add_hero_of_the_village(Uuid([7; 16]));
        assert_eq!(raid.heroes_of_the_village.len(), 1);
        raid.stop();
        assert_eq!(raid.status, RaidStatus::Stopped);
        assert!(!raid.is_active());
    }

    #[test]
    fn raid_center_falls_back_to_omen_position_without_village_pois() {
        assert_eq!(raid_center(pos(5, 70, 5), &[]), pos(5, 70, 5));
        assert_eq!(
            raid_center(pos(5, 70, 5), &[pos(0, 64, 0), pos(10, 66, 10)]),
            pos(5, 65, 5)
        );
    }
}
