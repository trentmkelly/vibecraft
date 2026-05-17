#![allow(dead_code)]

use std::collections::{BTreeMap, BTreeSet};

pub const MAX_WAYPOINT_RANGE: f64 = 60_000_000.0;
pub const REALLY_FAR_DISTANCE: f64 = 332.0;
pub const AZIMUTH_UPDATE_EPSILON_RADIANS: f32 = 0.008726646;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WaypointIcon {
    pub style: String,
    pub color: Option<i32>,
}

impl Default for WaypointIcon {
    fn default() -> Self {
        Self {
            style: "minecraft:default".to_string(),
            color: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WaypointEntity {
    pub id: String,
    pub dimension: String,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub chunk_x: i32,
    pub chunk_z: i32,
    pub transmit_range: f64,
    pub receive_range: f64,
    pub spectator: bool,
    pub first_tick: bool,
    pub icon: WaypointIcon,
    pub team_color: Option<i32>,
    pub passengers: BTreeSet<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WaypointConnectionKind {
    Block,
    Chunk,
    Azimuth,
}

#[derive(Debug, Clone, PartialEq)]
pub enum WaypointPacket {
    Track {
        receiver: String,
        source: String,
        kind: WaypointConnectionKind,
        icon: WaypointIcon,
    },
    Update {
        receiver: String,
        source: String,
        kind: WaypointConnectionKind,
        icon: WaypointIcon,
    },
    Untrack {
        receiver: String,
        source: String,
    },
}

#[derive(Debug, Clone, PartialEq)]
struct ActiveConnection {
    kind: WaypointConnectionKind,
    icon: WaypointIcon,
    last_x: f64,
    last_y: f64,
    last_z: f64,
    last_chunk_x: i32,
    last_chunk_z: i32,
    last_angle: f32,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct WaypointManager {
    transmitters: BTreeSet<String>,
    receivers: BTreeSet<String>,
    connections: BTreeMap<(String, String), ActiveConnection>,
}

impl WaypointIcon {
    pub fn has_data(&self) -> bool {
        self.style != "minecraft:default" || self.color.is_some()
    }

    pub fn clone_and_assign_style(&self, team_color: Option<i32>) -> Self {
        Self {
            style: self.style.clone(),
            color: self.color.or(team_color.map(|color| if color == 0 { -13_619_152 } else { color })),
        }
    }
}

impl WaypointEntity {
    pub fn is_transmitting_waypoint(&self) -> bool {
        self.transmit_range > 0.0
    }

    pub fn distance_to(&self, other: &Self) -> f64 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        let dz = self.z - other.z;
        (dx * dx + dy * dy + dz * dz).sqrt()
    }

    pub fn has_indirect_passenger(&self, receiver: &Self) -> bool {
        self.passengers.contains(&receiver.id)
    }
}

impl WaypointManager {
    pub fn add_player(
        &mut self,
        player: &WaypointEntity,
        entities: &BTreeMap<String, WaypointEntity>,
        locator_bar_enabled: bool,
    ) -> Vec<WaypointPacket> {
        self.receivers.insert(player.id.clone());
        let mut packets = Vec::new();
        for source_id in self.transmitters.clone() {
            if let Some(source) = entities.get(&source_id) {
                packets.extend(self.create_connection(player, source, locator_bar_enabled));
            }
        }
        if player.is_transmitting_waypoint() {
            packets.extend(self.track_waypoint(player, entities, locator_bar_enabled));
        }
        packets
    }

    pub fn remove_player(&mut self, player_id: &str) -> Vec<WaypointPacket> {
        let mut packets = Vec::new();
        let keys = self
            .connections
            .keys()
            .filter(|(receiver, source)| receiver == player_id || source == player_id)
            .cloned()
            .collect::<Vec<_>>();
        for (receiver, source) in keys {
            self.connections.remove(&(receiver.clone(), source.clone()));
            packets.push(WaypointPacket::Untrack { receiver, source });
        }
        self.transmitters.remove(player_id);
        self.receivers.remove(player_id);
        packets
    }

    pub fn track_waypoint(
        &mut self,
        source: &WaypointEntity,
        entities: &BTreeMap<String, WaypointEntity>,
        locator_bar_enabled: bool,
    ) -> Vec<WaypointPacket> {
        self.transmitters.insert(source.id.clone());
        let mut packets = Vec::new();
        for receiver_id in self.receivers.clone() {
            if let Some(receiver) = entities.get(&receiver_id) {
                packets.extend(self.create_connection(receiver, source, locator_bar_enabled));
            }
        }
        packets
    }

    pub fn update_waypoint(
        &mut self,
        source: &WaypointEntity,
        entities: &BTreeMap<String, WaypointEntity>,
        locator_bar_enabled: bool,
    ) -> Vec<WaypointPacket> {
        if !self.transmitters.contains(&source.id) {
            return Vec::new();
        }
        let mut packets = Vec::new();
        for receiver_id in self.receivers.clone() {
            let Some(receiver) = entities.get(&receiver_id) else {
                continue;
            };
            let key = (receiver.id.clone(), source.id.clone());
            if let Some(connection) = self.connections.get(&key).cloned() {
                if connection_is_broken(&connection, receiver, source, locator_bar_enabled) {
                    self.connections.remove(&key);
                    packets.push(WaypointPacket::Untrack {
                        receiver: receiver.id.clone(),
                        source: source.id.clone(),
                    });
                    packets.extend(self.create_connection(receiver, source, locator_bar_enabled));
                } else if connection_changed(&connection, receiver, source) {
                    let next = active_connection(receiver, source, connection.kind);
                    self.connections.insert(key, next.clone());
                    packets.push(WaypointPacket::Update {
                        receiver: receiver.id.clone(),
                        source: source.id.clone(),
                        kind: next.kind,
                        icon: next.icon,
                    });
                }
            } else {
                packets.extend(self.create_connection(receiver, source, locator_bar_enabled));
            }
        }
        packets
    }

    pub fn break_all_connections(&mut self) -> Vec<WaypointPacket> {
        let packets = self
            .connections
            .keys()
            .map(|(receiver, source)| WaypointPacket::Untrack {
                receiver: receiver.clone(),
                source: source.clone(),
            })
            .collect();
        self.connections.clear();
        packets
    }

    fn create_connection(
        &mut self,
        receiver: &WaypointEntity,
        source: &WaypointEntity,
        locator_bar_enabled: bool,
    ) -> Vec<WaypointPacket> {
        let key = (receiver.id.clone(), source.id.clone());
        if receiver.id == source.id || !locator_bar_enabled {
            self.connections.remove(&key);
            return Vec::new();
        }
        let Some(kind) = make_connection_kind(receiver, source, true) else {
            if self.connections.remove(&key).is_some() {
                return vec![WaypointPacket::Untrack {
                    receiver: receiver.id.clone(),
                    source: source.id.clone(),
                }];
            }
            return Vec::new();
        };
        let connection = active_connection(receiver, source, kind);
        self.connections.insert(key, connection.clone());
        vec![WaypointPacket::Track {
            receiver: receiver.id.clone(),
            source: source.id.clone(),
            kind,
            icon: connection.icon,
        }]
    }
}

pub fn does_source_ignore_receiver(source: &WaypointEntity, receiver: &WaypointEntity) -> bool {
    if receiver.spectator {
        false
    } else if !source.spectator && !source.has_indirect_passenger(receiver) {
        let range = source.transmit_range.min(receiver.receive_range).min(MAX_WAYPOINT_RANGE);
        source.distance_to(receiver) >= range
    } else {
        true
    }
}

pub fn is_really_far(source: &WaypointEntity, receiver: &WaypointEntity) -> bool {
    source.distance_to(receiver) > REALLY_FAR_DISTANCE
}

pub fn is_chunk_visible(source: &WaypointEntity, receiver: &WaypointEntity, view_distance: i32) -> bool {
    (source.chunk_x - receiver.chunk_x).abs().max((source.chunk_z - receiver.chunk_z).abs())
        <= view_distance
}

pub fn make_connection_kind(
    receiver: &WaypointEntity,
    source: &WaypointEntity,
    source_chunk_visible: bool,
) -> Option<WaypointConnectionKind> {
    if source.first_tick || receiver.id == source.id || does_source_ignore_receiver(source, receiver)
    {
        None
    } else if is_really_far(source, receiver) {
        Some(WaypointConnectionKind::Azimuth)
    } else if !source_chunk_visible {
        Some(WaypointConnectionKind::Chunk)
    } else {
        Some(WaypointConnectionKind::Block)
    }
}

fn active_connection(
    receiver: &WaypointEntity,
    source: &WaypointEntity,
    kind: WaypointConnectionKind,
) -> ActiveConnection {
    ActiveConnection {
        kind,
        icon: source.icon.clone_and_assign_style(source.team_color),
        last_x: source.x,
        last_y: source.y,
        last_z: source.z,
        last_chunk_x: source.chunk_x,
        last_chunk_z: source.chunk_z,
        last_angle: azimuth_angle(source, receiver),
    }
}

fn connection_is_broken(
    connection: &ActiveConnection,
    receiver: &WaypointEntity,
    source: &WaypointEntity,
    locator_bar_enabled: bool,
) -> bool {
    if !locator_bar_enabled || does_source_ignore_receiver(source, receiver) {
        return true;
    }
    match connection.kind {
        WaypointConnectionKind::Block => {
            manhattan((connection.last_x, connection.last_y, connection.last_z), (source.x, source.y, source.z)) > 1.0
        }
        WaypointConnectionKind::Chunk => {
            chessboard(
                (connection.last_chunk_x, connection.last_chunk_z),
                (source.chunk_x, source.chunk_z),
            ) > 1
        }
        WaypointConnectionKind::Azimuth => false,
    }
}

fn connection_changed(
    connection: &ActiveConnection,
    receiver: &WaypointEntity,
    source: &WaypointEntity,
) -> bool {
    match connection.kind {
        WaypointConnectionKind::Block => {
            manhattan((connection.last_x, connection.last_y, connection.last_z), (source.x, source.y, source.z)) > 0.0
        }
        WaypointConnectionKind::Chunk => {
            chessboard(
                (connection.last_chunk_x, connection.last_chunk_z),
                (source.chunk_x, source.chunk_z),
            ) > 0
        }
        WaypointConnectionKind::Azimuth => {
            (azimuth_angle(source, receiver) - connection.last_angle).abs()
                > AZIMUTH_UPDATE_EPSILON_RADIANS
        }
    }
}

fn azimuth_angle(source: &WaypointEntity, receiver: &WaypointEntity) -> f32 {
    let dx = receiver.x - source.x;
    let dz = receiver.z - source.z;
    dz.atan2(-dx) as f32
}

fn manhattan(a: (f64, f64, f64), b: (f64, f64, f64)) -> f64 {
    (a.0 - b.0).abs() + (a.1 - b.1).abs() + (a.2 - b.2).abs()
}

fn chessboard(a: (i32, i32), b: (i32, i32)) -> i32 {
    (a.0 - b.0).abs().max((a.1 - b.1).abs())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entity(id: &str, x: f64, z: f64) -> WaypointEntity {
        WaypointEntity {
            id: id.to_string(),
            dimension: "minecraft:overworld".to_string(),
            x,
            y: 64.0,
            z,
            chunk_x: (x as i32).div_euclid(16),
            chunk_z: (z as i32).div_euclid(16),
            transmit_range: 1024.0,
            receive_range: 1024.0,
            spectator: false,
            first_tick: false,
            icon: WaypointIcon::default(),
            team_color: None,
            passengers: BTreeSet::new(),
        }
    }

    #[test]
    fn connection_kind_matches_range_distance_and_visibility_rules() {
        let receiver = entity("receiver", 0.0, 0.0);
        let mut source = entity("source", 16.0, 0.0);

        assert_eq!(
            make_connection_kind(&receiver, &source, true),
            Some(WaypointConnectionKind::Block)
        );
        assert_eq!(
            make_connection_kind(&receiver, &source, false),
            Some(WaypointConnectionKind::Chunk)
        );
        source.x = 400.0;
        assert_eq!(
            make_connection_kind(&receiver, &source, false),
            Some(WaypointConnectionKind::Azimuth)
        );
        source.transmit_range = 10.0;
        assert_eq!(make_connection_kind(&receiver, &source, true), None);
    }

    #[test]
    fn spectators_and_passengers_follow_ignore_receiver_rules() {
        let mut receiver = entity("receiver", 100.0, 0.0);
        let mut source = entity("source", 0.0, 0.0);
        source.transmit_range = 10.0;

        assert!(does_source_ignore_receiver(&source, &receiver));
        receiver.spectator = true;
        assert!(!does_source_ignore_receiver(&source, &receiver));
        receiver.spectator = false;
        source.passengers.insert(receiver.id.clone());
        assert!(does_source_ignore_receiver(&source, &receiver));
    }

    #[test]
    fn manager_tracks_updates_breaks_and_removes_connections() {
        let receiver = entity("receiver", 0.0, 0.0);
        let mut source = entity("source", 16.0, 0.0);
        let mut entities = BTreeMap::from([
            (receiver.id.clone(), receiver.clone()),
            (source.id.clone(), source.clone()),
        ]);
        let mut manager = WaypointManager::default();
        manager.receivers.insert(receiver.id.clone());

        assert_eq!(
            manager.track_waypoint(&source, &entities, true),
            vec![WaypointPacket::Track {
                receiver: receiver.id.clone(),
                source: source.id.clone(),
                kind: WaypointConnectionKind::Block,
                icon: WaypointIcon::default(),
            }]
        );

        source.x = 16.5;
        entities.insert(source.id.clone(), source.clone());
        assert!(matches!(
            manager.update_waypoint(&source, &entities, true).as_slice(),
            [WaypointPacket::Update {
                kind: WaypointConnectionKind::Block,
                ..
            }]
        ));

        source.x = 19.0;
        entities.insert(source.id.clone(), source.clone());
        let packets = manager.update_waypoint(&source, &entities, true);
        assert!(matches!(packets[0], WaypointPacket::Untrack { .. }));
        assert!(matches!(packets[1], WaypointPacket::Track { .. }));

        assert_eq!(manager.break_all_connections().len(), 1);
        assert!(manager.break_all_connections().is_empty());
    }

    #[test]
    fn icon_data_team_color_and_chunk_view_follow_vanilla_helpers() {
        let mut icon = WaypointIcon::default();
        assert!(!icon.has_data());
        icon.color = Some(0x123456);
        assert!(icon.has_data());
        assert_eq!(WaypointIcon::default().clone_and_assign_style(Some(0)).color, Some(-13_619_152));

        let receiver = entity("receiver", 0.0, 0.0);
        let source = entity("source", 48.0, 48.0);
        assert!(is_chunk_visible(&source, &receiver, 3));
        assert!(!is_chunk_visible(&source, &receiver, 2));
    }
}
