#![allow(dead_code)]

use std::collections::{BTreeMap, BTreeSet};

use crate::storage::region::ChunkPos;

pub const FLAG_PERSIST: u8 = 1;
pub const FLAG_LOADING: u8 = 2;
pub const FLAG_SIMULATION: u8 = 4;
pub const FLAG_KEEP_DIMENSION_ACTIVE: u8 = 8;
pub const FLAG_CAN_EXPIRE_IF_UNLOADED: u8 = 16;
pub const FULL_CHUNK_LEVEL: i32 = 33;
pub const BLOCK_TICKING_LEVEL: i32 = 32;
pub const ENTITY_TICKING_LEVEL: i32 = 31;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TicketTypeEntry {
    pub id: &'static str,
    pub timeout: i64,
    pub flags: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ticket {
    pub ticket_type: TicketTypeEntry,
    pub level: i32,
    pub ticks_left: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FullChunkStatus {
    Inaccessible,
    Full,
    BlockTicking,
    EntityTicking,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct TicketStore {
    tickets: BTreeMap<ChunkPos, Vec<Ticket>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChunkTrackingView {
    pub center: Option<ChunkPos>,
    pub view_distance: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChunkTrackingDiff {
    pub entered: Vec<ChunkPos>,
    pub left: Vec<ChunkPos>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerChunkTracker {
    players: BTreeMap<String, ChunkPos>,
    views: BTreeMap<String, ChunkTrackingView>,
    pub view_distance: i32,
    pub simulation_distance: i32,
}

pub const TICKET_TYPES: &[TicketTypeEntry] = &[
    ticket_type("minecraft:player_spawn", 20, FLAG_LOADING),
    ticket_type("minecraft:spawn_search", 1, FLAG_LOADING),
    ticket_type("minecraft:dragon", 0, FLAG_LOADING | FLAG_SIMULATION),
    ticket_type("minecraft:player_loading", 0, FLAG_LOADING),
    ticket_type(
        "minecraft:player_simulation",
        0,
        FLAG_SIMULATION | FLAG_KEEP_DIMENSION_ACTIVE,
    ),
    ticket_type(
        "minecraft:forced",
        0,
        FLAG_PERSIST | FLAG_LOADING | FLAG_SIMULATION | FLAG_KEEP_DIMENSION_ACTIVE,
    ),
    ticket_type(
        "minecraft:portal",
        300,
        FLAG_PERSIST | FLAG_LOADING | FLAG_SIMULATION | FLAG_KEEP_DIMENSION_ACTIVE,
    ),
    ticket_type(
        "minecraft:ender_pearl",
        40,
        FLAG_LOADING | FLAG_SIMULATION | FLAG_KEEP_DIMENSION_ACTIVE,
    ),
    ticket_type(
        "minecraft:unknown",
        1,
        FLAG_LOADING | FLAG_CAN_EXPIRE_IF_UNLOADED,
    ),
];

const fn ticket_type(id: &'static str, timeout: i64, flags: u8) -> TicketTypeEntry {
    TicketTypeEntry { id, timeout, flags }
}

impl TicketTypeEntry {
    pub fn persist(self) -> bool {
        self.flags & FLAG_PERSIST != 0
    }

    pub fn does_load(self) -> bool {
        self.flags & FLAG_LOADING != 0
    }

    pub fn does_simulate(self) -> bool {
        self.flags & FLAG_SIMULATION != 0
    }

    pub fn should_keep_dimension_active(self) -> bool {
        self.flags & FLAG_KEEP_DIMENSION_ACTIVE != 0
    }

    pub fn can_expire_if_unloaded(self) -> bool {
        self.flags & FLAG_CAN_EXPIRE_IF_UNLOADED != 0
    }

    pub fn has_timeout(self) -> bool {
        self.timeout != 0
    }
}

impl Ticket {
    pub fn new(ticket_type: TicketTypeEntry, level: i32) -> Self {
        Self {
            ticket_type,
            level,
            ticks_left: ticket_type.timeout,
        }
    }

    pub fn reset_ticks_left(&mut self) {
        self.ticks_left = self.ticket_type.timeout;
    }

    pub fn decrease_ticks_left(&mut self) {
        if self.ticket_type.has_timeout() {
            self.ticks_left -= 1;
        }
    }

    pub fn is_timed_out(self) -> bool {
        self.ticket_type.has_timeout() && self.ticks_left < 0
    }
}

impl FullChunkStatus {
    pub fn by_level(level: i32) -> Self {
        if level <= ENTITY_TICKING_LEVEL {
            Self::EntityTicking
        } else if level <= BLOCK_TICKING_LEVEL {
            Self::BlockTicking
        } else if level <= FULL_CHUNK_LEVEL {
            Self::Full
        } else {
            Self::Inaccessible
        }
    }

    pub fn level(self) -> i32 {
        match self {
            Self::Inaccessible => i32::MAX,
            Self::Full => FULL_CHUNK_LEVEL,
            Self::BlockTicking => BLOCK_TICKING_LEVEL,
            Self::EntityTicking => ENTITY_TICKING_LEVEL,
        }
    }
}

impl TicketStore {
    pub fn add_ticket(&mut self, pos: ChunkPos, ticket: Ticket) -> bool {
        let tickets = self.tickets.entry(pos).or_default();
        if let Some(existing) = tickets
            .iter_mut()
            .find(|existing| same_type_and_level(**existing, ticket))
        {
            existing.reset_ticks_left();
            false
        } else {
            tickets.push(ticket);
            true
        }
    }

    pub fn add_ticket_with_radius(
        &mut self,
        pos: ChunkPos,
        ticket_type: TicketTypeEntry,
        radius: i32,
    ) -> bool {
        self.add_ticket(pos, Ticket::new(ticket_type, FULL_CHUNK_LEVEL - radius))
    }

    pub fn remove_ticket(&mut self, pos: ChunkPos, ticket: Ticket) -> bool {
        let Some(tickets) = self.tickets.get_mut(&pos) else {
            return false;
        };
        let old_len = tickets.len();
        tickets.retain(|existing| !same_type_and_level(*existing, ticket));
        let removed = tickets.len() != old_len;
        if tickets.is_empty() {
            self.tickets.remove(&pos);
        }
        removed
    }

    pub fn tick(&mut self) {
        for tickets in self.tickets.values_mut() {
            for ticket in tickets.iter_mut() {
                ticket.decrease_ticks_left();
            }
            tickets.retain(|ticket| !ticket.is_timed_out());
        }
        self.tickets.retain(|_, tickets| !tickets.is_empty());
    }

    pub fn tickets(&self, pos: ChunkPos) -> &[Ticket] {
        self.tickets.get(&pos).map(Vec::as_slice).unwrap_or(&[])
    }

    pub fn effective_level(&self, pos: ChunkPos, simulation: bool) -> Option<i32> {
        self.tickets(pos)
            .iter()
            .filter(|ticket| {
                if simulation {
                    ticket.ticket_type.does_simulate()
                } else {
                    ticket.ticket_type.does_load()
                }
            })
            .map(|ticket| ticket.level)
            .min()
    }

    pub fn replace_ticket_level_of_type(&mut self, new_level: i32, ticket_type: TicketTypeEntry) {
        for tickets in self.tickets.values_mut() {
            for ticket in tickets {
                if ticket.ticket_type.id == ticket_type.id {
                    ticket.level = new_level;
                }
            }
        }
    }

    pub fn keeps_dimension_active(&self) -> bool {
        self.tickets
            .values()
            .flatten()
            .any(|ticket| ticket.ticket_type.should_keep_dimension_active())
    }
}

impl ChunkTrackingView {
    pub const EMPTY: Self = Self {
        center: None,
        view_distance: 0,
    };

    pub fn positioned(center: ChunkPos, view_distance: i32) -> Self {
        Self {
            center: Some(center),
            view_distance,
        }
    }

    pub fn contains(&self, chunk: ChunkPos, include_neighbors: bool) -> bool {
        self.center.is_some_and(|center| {
            is_within_view_distance(center, self.view_distance, chunk, include_neighbors)
        })
    }

    pub fn is_in_view_distance(&self, chunk: ChunkPos) -> bool {
        self.contains(chunk, false)
    }

    pub fn chunks(&self) -> BTreeSet<ChunkPos> {
        let mut chunks = BTreeSet::new();
        let Some(center) = self.center else {
            return chunks;
        };
        for x in self.min_x()..=self.max_x() {
            for z in self.min_z()..=self.max_z() {
                let pos = ChunkPos { x, z };
                if self.contains(pos, true) {
                    chunks.insert(pos);
                }
            }
        }
        if chunks.is_empty() {
            chunks.insert(center);
        }
        chunks
    }

    pub fn diff(&self, next: &Self) -> ChunkTrackingDiff {
        let from = self.chunks();
        let to = next.chunks();
        ChunkTrackingDiff {
            entered: to.difference(&from).copied().collect(),
            left: from.difference(&to).copied().collect(),
        }
    }

    fn min_x(&self) -> i32 {
        self.center
            .map(|center| center.x - self.view_distance - 1)
            .unwrap_or(0)
    }

    fn min_z(&self) -> i32 {
        self.center
            .map(|center| center.z - self.view_distance - 1)
            .unwrap_or(0)
    }

    fn max_x(&self) -> i32 {
        self.center
            .map(|center| center.x + self.view_distance + 1)
            .unwrap_or(-1)
    }

    fn max_z(&self) -> i32 {
        self.center
            .map(|center| center.z + self.view_distance + 1)
            .unwrap_or(-1)
    }
}

impl PlayerChunkTracker {
    pub fn new(view_distance: i32, simulation_distance: i32) -> Self {
        Self {
            players: BTreeMap::new(),
            views: BTreeMap::new(),
            view_distance,
            simulation_distance,
        }
    }

    pub fn add_or_move_player(
        &mut self,
        player_id: impl Into<String>,
        chunk: ChunkPos,
    ) -> ChunkTrackingDiff {
        let player_id = player_id.into();
        let previous = self
            .views
            .get(&player_id)
            .cloned()
            .unwrap_or(ChunkTrackingView::EMPTY);
        let next = ChunkTrackingView::positioned(chunk, self.view_distance);
        let diff = previous.diff(&next);
        self.players.insert(player_id.clone(), chunk);
        self.views.insert(player_id, next);
        diff
    }

    pub fn remove_player(&mut self, player_id: &str) -> ChunkTrackingDiff {
        self.players.remove(player_id);
        let previous = self
            .views
            .remove(player_id)
            .unwrap_or(ChunkTrackingView::EMPTY);
        previous.diff(&ChunkTrackingView::EMPTY)
    }

    pub fn update_view_distance(&mut self, view_distance: i32) -> Vec<(String, ChunkTrackingDiff)> {
        self.view_distance = view_distance;
        let player_positions: Vec<_> = self
            .players
            .iter()
            .map(|(player, pos)| (player.clone(), *pos))
            .collect();
        player_positions
            .into_iter()
            .map(|(player, pos)| {
                let previous = self
                    .views
                    .get(&player)
                    .cloned()
                    .unwrap_or(ChunkTrackingView::EMPTY);
                let next = ChunkTrackingView::positioned(pos, view_distance);
                let diff = previous.diff(&next);
                self.views.insert(player.clone(), next);
                (player, diff)
            })
            .collect()
    }

    pub fn player_simulation_ticket_level(&self) -> i32 {
        (ENTITY_TICKING_LEVEL - self.simulation_distance).max(0)
    }

    pub fn update_simulation_distance(
        &mut self,
        simulation_distance: i32,
        tickets: &mut TicketStore,
    ) {
        self.simulation_distance = simulation_distance;
        if let Some(ticket_type) = ticket_type_by_id("player_simulation") {
            tickets
                .replace_ticket_level_of_type(self.player_simulation_ticket_level(), ticket_type);
        }
    }
}

pub fn is_within_view_distance(
    center: ChunkPos,
    view_distance: i32,
    chunk: ChunkPos,
    include_neighbors: bool,
) -> bool {
    let buffer_range = if include_neighbors { 2 } else { 1 };
    let delta_x = 0.max((chunk.x - center.x).abs() - buffer_range) as i64;
    let delta_z = 0.max((chunk.z - center.z).abs() - buffer_range) as i64;
    let distance_squared = delta_x * delta_x + delta_z * delta_z;
    distance_squared < i64::from(view_distance * view_distance)
}

pub fn ticket_type_by_id(id: &str) -> Option<TicketTypeEntry> {
    let name = id.strip_prefix("minecraft:").unwrap_or(id);
    TICKET_TYPES.iter().copied().find(|entry| {
        entry
            .id
            .strip_prefix("minecraft:")
            .is_some_and(|entry_name| entry_name == name)
    })
}

fn same_type_and_level(a: Ticket, b: Ticket) -> bool {
    a.ticket_type.id == b.ticket_type.id && a.level == b.level
}

#[cfg(test)]
mod tests {
    use super::{
        is_within_view_distance, ticket_type_by_id, ChunkTrackingView, FullChunkStatus,
        PlayerChunkTracker, Ticket, TicketStore, BLOCK_TICKING_LEVEL, ENTITY_TICKING_LEVEL,
        FULL_CHUNK_LEVEL, TICKET_TYPES,
    };
    use crate::storage::region::ChunkPos;

    #[test]
    fn ticket_types_match_vanilla_flags_and_timeouts() {
        assert_eq!(TICKET_TYPES.len(), 9);
        let player_spawn = ticket_type_by_id("player_spawn").unwrap();
        assert_eq!(player_spawn.timeout, 20);
        assert!(player_spawn.does_load());
        assert!(!player_spawn.does_simulate());

        let forced = ticket_type_by_id("minecraft:forced").unwrap();
        assert!(forced.persist());
        assert!(forced.does_load());
        assert!(forced.does_simulate());
        assert!(forced.should_keep_dimension_active());
        assert!(!forced.has_timeout());

        let unknown = ticket_type_by_id("unknown").unwrap();
        assert!(unknown.does_load());
        assert!(unknown.can_expire_if_unloaded());
        assert_eq!(unknown.timeout, 1);
    }

    #[test]
    fn tickets_reset_expire_and_compute_effective_levels() {
        let mut store = TicketStore::default();
        let pos = ChunkPos { x: 0, z: 0 };
        let spawn_search = ticket_type_by_id("spawn_search").unwrap();
        let forced = ticket_type_by_id("forced").unwrap();

        assert!(store.add_ticket_with_radius(pos, spawn_search, 0));
        assert_eq!(store.effective_level(pos, false), Some(FULL_CHUNK_LEVEL));
        assert_eq!(store.effective_level(pos, true), None);
        assert!(!store.add_ticket_with_radius(pos, spawn_search, 0));
        assert_eq!(store.tickets(pos)[0].ticks_left, 1);

        store.tick();
        assert_eq!(store.tickets(pos)[0].ticks_left, 0);
        store.tick();
        assert!(store.tickets(pos).is_empty());

        assert!(store.add_ticket_with_radius(pos, forced, 2));
        assert_eq!(store.effective_level(pos, false), Some(31));
        assert_eq!(store.effective_level(pos, true), Some(31));
        assert!(store.keeps_dimension_active());
        assert!(store.remove_ticket(pos, Ticket::new(forced, FULL_CHUNK_LEVEL - 2)));
        assert!(!store.keeps_dimension_active());
    }

    #[test]
    fn full_chunk_status_thresholds_match_vanilla_levels() {
        assert_eq!(FULL_CHUNK_LEVEL, 33);
        assert_eq!(BLOCK_TICKING_LEVEL, 32);
        assert_eq!(ENTITY_TICKING_LEVEL, 31);
        assert_eq!(
            FullChunkStatus::by_level(31),
            FullChunkStatus::EntityTicking
        );
        assert_eq!(FullChunkStatus::by_level(32), FullChunkStatus::BlockTicking);
        assert_eq!(FullChunkStatus::by_level(33), FullChunkStatus::Full);
        assert_eq!(FullChunkStatus::by_level(34), FullChunkStatus::Inaccessible);
        assert_eq!(FullChunkStatus::Full.level(), 33);
    }

    #[test]
    fn player_simulation_ticket_levels_can_be_replaced() {
        let mut store = TicketStore::default();
        let pos = ChunkPos { x: 2, z: -3 };
        let player_simulation = ticket_type_by_id("player_simulation").unwrap();
        store.add_ticket(pos, Ticket::new(player_simulation, 21));
        store.replace_ticket_level_of_type(25, player_simulation);
        assert_eq!(store.effective_level(pos, true), Some(25));
    }

    #[test]
    fn chunk_tracking_view_matches_vanilla_distance_buffer() {
        let center = ChunkPos { x: 0, z: 0 };
        assert!(is_within_view_distance(
            center,
            5,
            ChunkPos { x: 5, z: 0 },
            true
        ));
        assert!(is_within_view_distance(
            center,
            5,
            ChunkPos { x: 5, z: 0 },
            false
        ));
        assert!(is_within_view_distance(
            center,
            5,
            ChunkPos { x: 6, z: 0 },
            true
        ));
        assert!(!is_within_view_distance(
            center,
            5,
            ChunkPos { x: 6, z: 0 },
            false
        ));
        assert!(!is_within_view_distance(
            center,
            5,
            ChunkPos { x: 7, z: 0 },
            true
        ));

        let view = ChunkTrackingView::positioned(center, 2);
        assert!(view.contains(ChunkPos { x: 3, z: 0 }, true));
        assert!(!view.is_in_view_distance(ChunkPos { x: 3, z: 0 }));
        assert!(view.chunks().contains(&center));
    }

    #[test]
    fn player_chunk_tracker_reports_enter_leave_and_separate_simulation_distance() {
        let mut tracker = PlayerChunkTracker::new(2, 10);
        let first = tracker.add_or_move_player("trent", ChunkPos { x: 0, z: 0 });
        assert!(!first.entered.is_empty());
        assert!(first.left.is_empty());

        let moved = tracker.add_or_move_player("trent", ChunkPos { x: 4, z: 0 });
        assert!(!moved.entered.is_empty());
        assert!(!moved.left.is_empty());

        let resized = tracker.update_view_distance(3);
        assert_eq!(resized.len(), 1);
        assert!(resized[0].1.entered.len() > resized[0].1.left.len());

        assert_eq!(tracker.player_simulation_ticket_level(), 21);
        let mut tickets = TicketStore::default();
        let player_simulation = ticket_type_by_id("player_simulation").unwrap();
        tickets.add_ticket(ChunkPos { x: 4, z: 0 }, Ticket::new(player_simulation, 21));
        tracker.update_simulation_distance(6, &mut tickets);
        assert_eq!(tracker.view_distance, 3);
        assert_eq!(tracker.player_simulation_ticket_level(), 25);
        assert_eq!(
            tickets.effective_level(ChunkPos { x: 4, z: 0 }, true),
            Some(25)
        );

        let removed = tracker.remove_player("trent");
        assert!(!removed.left.is_empty());
        assert!(removed.entered.is_empty());
    }
}
