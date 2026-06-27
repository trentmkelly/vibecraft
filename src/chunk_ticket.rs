#![allow(dead_code)]

use std::collections::{BTreeMap, BTreeSet};

use crate::storage::{
    chunk::unpack_chunk_pos_from_long,
    nbt::Tag,
    region::ChunkPos,
};

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChunkMapUpdateKind {
    MarkPendingToSend,
    Drop,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChunkMapUpdate {
    pub player_id: String,
    pub chunk: ChunkPos,
    pub kind: ChunkMapUpdateKind,
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
    pub fn is_or_after(self, step: Self) -> bool {
        self.ordinal() >= step.ordinal()
    }

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

    fn ordinal(self) -> u8 {
        match self {
            Self::Inaccessible => 0,
            Self::Full => 1,
            Self::BlockTicking => 2,
            Self::EntityTicking => 3,
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

    pub fn update_chunk_forced(&mut self, pos: ChunkPos, forced: bool) -> bool {
        let Some(ticket_type) = ticket_type_by_id("minecraft:forced") else {
            return false;
        };
        let ticket = Ticket::new(ticket_type, ENTITY_TICKING_LEVEL);
        if forced {
            self.add_ticket(pos, ticket)
        } else {
            self.remove_ticket(pos, ticket)
        }
    }

    pub fn force_loaded_chunks(&self) -> BTreeSet<ChunkPos> {
        self.tickets
            .iter()
            .filter_map(|(pos, tickets)| {
                tickets
                    .iter()
                    .any(|ticket| ticket.ticket_type.id == "minecraft:forced")
                    .then_some(*pos)
            })
            .collect()
    }

    pub fn to_ticket_storage_tag(&self) -> Tag {
        let tickets = self
            .tickets
            .iter()
            .flat_map(|(pos, tickets)| {
                tickets
                    .iter()
                    .filter(|ticket| ticket.ticket_type.persist())
                    .map(|ticket| ticket_entry_to_tag(*pos, *ticket))
            })
            .collect();
        Tag::Compound(vec![("tickets".to_string(), Tag::List(tickets))])
    }

    pub fn from_ticket_storage_tag(tag: &Tag) -> Self {
        let Tag::Compound(fields) = tag else {
            return Self::default();
        };
        let tickets = fields
            .iter()
            .find_map(|(name, tag)| (name == "tickets").then_some(tag))
            .and_then(|tag| match tag {
                Tag::List(tickets) => Some(tickets.as_slice()),
                _ => None,
            })
            .unwrap_or_default();
        let mut store = Self::default();
        // Java `TicketStorage.fromPacked` initially loads persisted tickets into
        // deactivatedTickets; `MinecraftServer.prepareLevels` activates them after
        // chunk listeners are installed. VibeCraft has no listener/deactivation split
        // yet, so persisted tickets are made active immediately.
        for tag in tickets {
            if let Some((pos, ticket)) = ticket_entry_from_tag(tag) {
                store.add_ticket(pos, ticket);
            }
        }
        store
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

pub fn chunk_map_updates_for_diff(
    player_id: &str,
    diff: &ChunkTrackingDiff,
    pending_to_send: &BTreeSet<ChunkPos>,
) -> Vec<ChunkMapUpdate> {
    let mut updates = Vec::with_capacity(diff.entered.len() + diff.left.len());
    for chunk in &diff.entered {
        if !pending_to_send.contains(chunk) {
            updates.push(ChunkMapUpdate {
                player_id: player_id.to_string(),
                chunk: *chunk,
                kind: ChunkMapUpdateKind::MarkPendingToSend,
            });
        }
    }
    for chunk in &diff.left {
        updates.push(ChunkMapUpdate {
            player_id: player_id.to_string(),
            chunk: *chunk,
            kind: ChunkMapUpdateKind::Drop,
        });
    }
    updates
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

fn ticket_entry_to_tag(pos: ChunkPos, ticket: Ticket) -> Tag {
    let mut fields = vec![
        (
            "chunk_pos".to_string(),
            Tag::List(vec![Tag::Int(pos.x), Tag::Int(pos.z)]),
        ),
        (
            "type".to_string(),
            Tag::String(ticket.ticket_type.id.to_string()),
        ),
        ("level".to_string(), Tag::Int(ticket.level)),
    ];
    if ticket.ticks_left != 0 {
        fields.push(("ticks_left".to_string(), Tag::Long(ticket.ticks_left)));
    }
    Tag::Compound(fields)
}

fn ticket_entry_from_tag(tag: &Tag) -> Option<(ChunkPos, Ticket)> {
    let Tag::Compound(fields) = tag else {
        return None;
    };
    let pos = chunk_pos_field(fields, "chunk_pos")?;
    let ticket_type = string_field(fields, "type").and_then(ticket_type_by_id)?;
    let level = int_field(fields, "level")?;
    if level < 0 {
        return None;
    }
    let ticks_left = long_field(fields, "ticks_left").unwrap_or(0);
    Some((
        pos,
        Ticket {
            ticket_type,
            level,
            ticks_left,
        },
    ))
}

fn chunk_pos_field(fields: &[(String, Tag)], name: &str) -> Option<ChunkPos> {
    match &fields.iter().find(|(field, _)| field == name)?.1 {
        Tag::List(values) if values.len() == 2 => {
            let [Tag::Int(x), Tag::Int(z)] = values.as_slice() else {
                return None;
            };
            Some(ChunkPos { x: *x, z: *z })
        }
        Tag::Long(value) => Some(unpack_chunk_pos_from_long(*value)),
        _ => None,
    }
}

fn int_field(fields: &[(String, Tag)], name: &str) -> Option<i32> {
    match fields.iter().find(|(field, _)| field == name)?.1 {
        Tag::Int(value) => Some(value),
        _ => None,
    }
}

fn long_field(fields: &[(String, Tag)], name: &str) -> Option<i64> {
    match fields.iter().find(|(field, _)| field == name)?.1 {
        Tag::Long(value) => Some(value),
        _ => None,
    }
}

fn string_field<'a>(fields: &'a [(String, Tag)], name: &str) -> Option<&'a str> {
    match fields.iter().find(|(field, _)| field == name)?.1 {
        Tag::String(ref value) => Some(value),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        chunk_map_updates_for_diff, is_within_view_distance, ticket_type_by_id, ChunkMapUpdateKind,
        ChunkTrackingView, FullChunkStatus, PlayerChunkTracker, Ticket, TicketStore,
        BLOCK_TICKING_LEVEL, ENTITY_TICKING_LEVEL, FULL_CHUNK_LEVEL, TICKET_TYPES,
    };
    use crate::storage::chunk::{pack_chunk_pos_as_long, unpack_chunk_pos_from_long};
    use crate::storage::nbt::Tag;
    use crate::storage::region::ChunkPos;
    use std::collections::BTreeSet;

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
    fn forced_chunk_ticket_storage_tag_matches_java_codec_shape() {
        let mut store = TicketStore::default();
        let forced = ticket_type_by_id("minecraft:forced").unwrap();
        let portal = ticket_type_by_id("minecraft:portal").unwrap();
        let spawn_search = ticket_type_by_id("minecraft:spawn_search").unwrap();
        assert!(store.update_chunk_forced(ChunkPos { x: 2, z: -3 }, true));
        assert!(!store.update_chunk_forced(ChunkPos { x: 2, z: -3 }, true));
        store.add_ticket(ChunkPos { x: -1, z: 4 }, Ticket::new(portal, 30));
        store.add_ticket(ChunkPos { x: 9, z: 9 }, Ticket::new(spawn_search, 33));

        let Tag::Compound(fields) = store.to_ticket_storage_tag() else {
            panic!("ticket storage should encode as compound");
        };
        let ticket_field = fields
            .iter()
            .find(|(name, _)| name == "tickets")
            .map(|(_, tag)| tag);
        let Some(Tag::List(tickets)) = ticket_field
        else {
            panic!("tickets should encode as list");
        };
        assert_eq!(tickets.len(), 2, "only persistent ticket types are saved");
        assert!(tickets.contains(&Tag::Compound(vec![
            (
                "chunk_pos".to_string(),
                Tag::List(vec![Tag::Int(2), Tag::Int(-3)])
            ),
            ("type".to_string(), Tag::String(forced.id.to_string())),
            ("level".to_string(), Tag::Int(ENTITY_TICKING_LEVEL)),
        ])));
    }

    #[test]
    fn ticket_storage_decodes_forced_chunks_and_optional_ticks_left() {
        let packed = Tag::Compound(vec![(
            "tickets".to_string(),
            Tag::List(vec![
                Tag::Compound(vec![
                    (
                        "chunk_pos".to_string(),
                        Tag::List(vec![Tag::Int(5), Tag::Int(-7)]),
                    ),
                    (
                        "type".to_string(),
                        Tag::String("minecraft:forced".to_string()),
                    ),
                    ("level".to_string(), Tag::Int(ENTITY_TICKING_LEVEL)),
                ]),
                Tag::Compound(vec![
                    (
                        "chunk_pos".to_string(),
                        Tag::Long(pack_chunk_pos_as_long(ChunkPos { x: -9, z: 11 })),
                    ),
                    (
                        "type".to_string(),
                        Tag::String("minecraft:portal".to_string()),
                    ),
                    ("level".to_string(), Tag::Int(30)),
                    ("ticks_left".to_string(), Tag::Long(17)),
                ]),
            ]),
        )]);

        let store = TicketStore::from_ticket_storage_tag(&packed);
        assert_eq!(
            store.force_loaded_chunks(),
            BTreeSet::from([ChunkPos { x: 5, z: -7 }])
        );
        assert_eq!(store.tickets(ChunkPos { x: 5, z: -7 })[0].ticks_left, 0);
        let portal_pos = unpack_chunk_pos_from_long(pack_chunk_pos_as_long(ChunkPos {
            x: -9,
            z: 11,
        }));
        assert_eq!(store.tickets(portal_pos)[0].ticks_left, 17);
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
        assert!(FullChunkStatus::EntityTicking.is_or_after(FullChunkStatus::Full));
        assert!(FullChunkStatus::BlockTicking.is_or_after(FullChunkStatus::BlockTicking));
        assert!(!FullChunkStatus::Full.is_or_after(FullChunkStatus::BlockTicking));
        assert!(!FullChunkStatus::Inaccessible.is_or_after(FullChunkStatus::Full));
    }

    #[test]
    #[cfg(vibecraft_has_decompiled_sources)]
    fn full_chunk_status_source_matches_java_26_1_2() {
        const FULL_CHUNK_STATUS: &str =
            vibecraft_java_source!("/net/minecraft/server/level/FullChunkStatus.java");

        for sentinel in [
            "public enum FullChunkStatus",
            "INACCESSIBLE,",
            "FULL,",
            "BLOCK_TICKING,",
            "ENTITY_TICKING;",
            "public boolean isOrAfter(final FullChunkStatus step)",
            "return this.ordinal() >= step.ordinal();",
        ] {
            assert!(
                FULL_CHUNK_STATUS.contains(sentinel),
                "FullChunkStatus.java is missing sentinel: {sentinel}"
            );
        }
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

    #[test]
    fn chunk_map_updates_mark_enters_drop_leaves_and_skip_pending_sends() {
        let previous = ChunkTrackingView::positioned(ChunkPos { x: 0, z: 0 }, 1);
        let next = ChunkTrackingView::positioned(ChunkPos { x: 3, z: 0 }, 1);
        let diff = previous.diff(&next);
        let already_pending = diff
            .entered
            .iter()
            .copied()
            .take(1)
            .collect::<BTreeSet<_>>();

        let updates = chunk_map_updates_for_diff("trent", &diff, &already_pending);
        assert!(updates
            .iter()
            .any(|update| update.kind == ChunkMapUpdateKind::Drop));
        assert!(updates
            .iter()
            .any(|update| update.kind == ChunkMapUpdateKind::MarkPendingToSend));
        assert!(!updates.iter().any(|update| {
            update.kind == ChunkMapUpdateKind::MarkPendingToSend
                && already_pending.contains(&update.chunk)
        }));
        assert!(updates.iter().all(|update| update.player_id == "trent"));
    }
}
