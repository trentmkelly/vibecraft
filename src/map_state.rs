#![allow(dead_code)]

use std::collections::BTreeMap;

pub const MAP_SIZE: usize = 128;
pub const MAX_TRACKED_DECORATIONS: usize = 256;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MapDecorationKind {
    Player,
    PlayerOffMap,
    PlayerOffLimits,
    Frame,
    Banner(DyeColor),
    Target,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DyeColor {
    White,
    Orange,
    Magenta,
    LightBlue,
    Yellow,
    Lime,
    Pink,
    Gray,
    LightGray,
    Cyan,
    Purple,
    Blue,
    Brown,
    Green,
    Red,
    Black,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MapDecoration {
    pub kind: MapDecorationKind,
    pub x: i8,
    pub y: i8,
    pub rot: i8,
    pub name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MapBanner {
    pub pos: BlockPos,
    pub color: DyeColor,
    pub name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MapFrame {
    pub pos: BlockPos,
    pub rotation: i32,
    pub entity_id: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MapPatch {
    pub start_x: usize,
    pub start_y: usize,
    pub width: usize,
    pub height: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MapUpdatePacket {
    pub scale: u8,
    pub locked: bool,
    pub decorations: Option<Vec<(String, MapDecoration)>>,
    pub patch: Option<MapPatch>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MapDecorationUpdate {
    pub kind: MapDecorationKind,
    pub key: String,
    pub x_pos: f64,
    pub z_pos: f64,
    pub y_rot: f64,
    pub name: Option<String>,
    pub game_time: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MapState {
    pub center_x: i32,
    pub center_z: i32,
    pub scale: u8,
    pub dimension: String,
    pub tracking_position: bool,
    pub unlimited_tracking: bool,
    pub locked: bool,
    pub colors: Vec<u8>,
    pub decorations: BTreeMap<String, MapDecoration>,
    pub banner_markers: BTreeMap<String, MapBanner>,
    pub frame_markers: BTreeMap<String, MapFrame>,
    tracked_decoration_count: usize,
    dirty_data: bool,
    dirty_decorations: bool,
    min_dirty_x: usize,
    min_dirty_y: usize,
    max_dirty_x: usize,
    max_dirty_y: usize,
    update_tick: u32,
}

impl MapBanner {
    pub fn id(&self) -> String {
        format!("banner-{},{},{}", self.pos.x, self.pos.y, self.pos.z)
    }

    pub fn decoration_kind(&self) -> MapDecorationKind {
        MapDecorationKind::Banner(self.color)
    }
}

impl MapFrame {
    pub fn id(&self) -> String {
        frame_id(self.pos)
    }
}

impl MapState {
    pub fn new(
        center_x: i32,
        center_z: i32,
        scale: u8,
        dimension: impl Into<String>,
        tracking_position: bool,
        unlimited_tracking: bool,
        locked: bool,
    ) -> Self {
        Self {
            center_x,
            center_z,
            scale,
            dimension: dimension.into(),
            tracking_position,
            unlimited_tracking,
            locked,
            colors: vec![0; MAP_SIZE * MAP_SIZE],
            decorations: BTreeMap::new(),
            banner_markers: BTreeMap::new(),
            frame_markers: BTreeMap::new(),
            tracked_decoration_count: 0,
            dirty_data: true,
            dirty_decorations: true,
            min_dirty_x: 0,
            min_dirty_y: 0,
            max_dirty_x: MAP_SIZE - 1,
            max_dirty_y: MAP_SIZE - 1,
            update_tick: 0,
        }
    }

    pub fn is_tracked_count_over_limit(&self, limit: usize) -> bool {
        self.tracked_decoration_count >= limit
    }

    pub fn update_color(&mut self, x: usize, y: usize, new_color: u8) -> bool {
        let index = x + y * MAP_SIZE;
        if self.colors[index] != new_color {
            self.colors[index] = new_color;
            self.mark_colors_dirty(x, y);
            true
        } else {
            false
        }
    }

    pub fn add_decoration(&mut self, update: MapDecorationUpdate) -> bool {
        let scale = 1_i32 << self.scale;
        let x_delta = ((update.x_pos - f64::from(self.center_x)) / f64::from(scale)) as f32;
        let y_delta = ((update.z_pos - f64::from(self.center_z)) / f64::from(scale)) as f32;
        let Some((kind, rot)) = self.decoration_location(
            update.kind,
            x_delta,
            y_delta,
            update.y_rot,
            update.game_time,
        ) else {
            return self.remove_decoration(&update.key);
        };
        let decoration = MapDecoration {
            kind,
            x: clamp_map_coordinate(x_delta),
            y: clamp_map_coordinate(y_delta),
            rot,
            name: update.name,
        };
        let previous = self.decorations.insert(update.key, decoration.clone());
        if previous.as_ref() != Some(&decoration) {
            self.adjust_tracked_count(previous.as_ref(), -1);
            self.adjust_tracked_count(Some(&decoration), 1);
            self.dirty_decorations = true;
            return true;
        }
        false
    }

    pub fn remove_decoration(&mut self, key: &str) -> bool {
        if let Some(decoration) = self.decorations.remove(key) {
            self.adjust_tracked_count(Some(&decoration), -1);
            self.dirty_decorations = true;
            true
        } else {
            false
        }
    }

    pub fn toggle_banner(&mut self, banner: Option<MapBanner>) -> bool {
        let Some(banner) = banner else {
            return false;
        };
        let x_pos = f64::from(banner.pos.x) + 0.5;
        let z_pos = f64::from(banner.pos.z) + 0.5;
        let scale = 1_i32 << self.scale;
        let x_delta = (x_pos - f64::from(self.center_x)) / f64::from(scale);
        let y_delta = (z_pos - f64::from(self.center_z)) / f64::from(scale);
        if !inside_map(x_delta as f32, y_delta as f32) {
            return false;
        }

        let id = banner.id();
        if self.banner_markers.get(&id) == Some(&banner) {
            self.banner_markers.remove(&id);
            self.remove_decoration(&id);
            return true;
        }

        if self.is_tracked_count_over_limit(MAX_TRACKED_DECORATIONS) {
            return false;
        }
        self.add_decoration(MapDecorationUpdate {
            kind: banner.decoration_kind(),
            key: id.clone(),
            x_pos,
            z_pos,
            y_rot: 180.0,
            name: banner.name.clone(),
            game_time: 0,
        });
        self.banner_markers.insert(id, banner);
        true
    }

    pub fn check_banner_column(
        &mut self,
        x: i32,
        z: i32,
        current: impl Fn(BlockPos) -> Option<MapBanner>,
    ) {
        let ids = self
            .banner_markers
            .values()
            .filter(|banner| banner.pos.x == x && banner.pos.z == z)
            .map(MapBanner::id)
            .collect::<Vec<_>>();
        for id in ids {
            let expected = self.banner_markers.get(&id).cloned();
            if expected.as_ref().and_then(|banner| current(banner.pos)) != expected {
                self.banner_markers.remove(&id);
                self.remove_decoration(&id);
            }
        }
    }

    pub fn add_frame(&mut self, frame: MapFrame) -> bool {
        let key = frame_key(frame.entity_id);
        let changed = self.add_decoration(MapDecorationUpdate {
            kind: MapDecorationKind::Frame,
            key,
            x_pos: f64::from(frame.pos.x),
            z_pos: f64::from(frame.pos.z),
            y_rot: f64::from(frame.rotation),
            name: None,
            game_time: 0,
        });
        let old = self.frame_markers.insert(frame.id(), frame.clone());
        changed || old.as_ref() != Some(&frame)
    }

    pub fn removed_from_frame(&mut self, pos: BlockPos, entity_id: i32) {
        self.remove_decoration(&frame_key(entity_id));
        self.frame_markers.remove(&frame_id(pos));
    }

    pub fn add_target_decoration(
        &mut self,
        key: impl Into<String>,
        pos: BlockPos,
        kind: MapDecorationKind,
    ) -> bool {
        self.add_decoration(MapDecorationUpdate {
            kind,
            key: key.into(),
            x_pos: f64::from(pos.x),
            z_pos: f64::from(pos.z),
            y_rot: 180.0,
            name: None,
            game_time: 0,
        })
    }

    pub fn next_update_packet(&mut self) -> Option<MapUpdatePacket> {
        let patch = self.dirty_data.then(|| {
            self.dirty_data = false;
            MapPatch {
                start_x: self.min_dirty_x,
                start_y: self.min_dirty_y,
                width: self.max_dirty_x + 1 - self.min_dirty_x,
                height: self.max_dirty_y + 1 - self.min_dirty_y,
            }
        });
        // Java: `if (this.dirtyDecorations && this.tick++ % 5 == 0)`. The `tick++`
        // sits inside the `&&` short-circuit, so the tick counter only advances on
        // calls where decorations are dirty — it must NOT increment unconditionally,
        // or the every-5th-tick send cadence desyncs.
        let decorations = if self.dirty_decorations {
            let at_interval = self.update_tick.is_multiple_of(5);
            self.update_tick += 1;
            if at_interval {
                self.dirty_decorations = false;
                Some(
                    self.decorations
                        .iter()
                        .map(|(key, value)| (key.clone(), value.clone()))
                        .collect::<Vec<_>>(),
                )
            } else {
                None
            }
        } else {
            None
        };

        if patch.is_none() && decorations.is_none() {
            None
        } else {
            Some(MapUpdatePacket {
                scale: self.scale,
                locked: self.locked,
                decorations,
                patch,
            })
        }
    }

    fn decoration_location(
        &self,
        kind: MapDecorationKind,
        x_delta: f32,
        y_delta: f32,
        y_rot: f64,
        game_time: i64,
    ) -> Option<(MapDecorationKind, i8)> {
        if kind == MapDecorationKind::Player {
            if inside_map(x_delta, y_delta) {
                Some((kind, self.calculate_rotation(y_rot, game_time)))
            } else if x_delta.abs() < 320.0 && y_delta.abs() < 320.0 {
                Some((MapDecorationKind::PlayerOffMap, 0))
            } else if self.unlimited_tracking {
                Some((MapDecorationKind::PlayerOffLimits, 0))
            } else {
                None
            }
        } else if !inside_map(x_delta, y_delta) && !self.unlimited_tracking {
            None
        } else {
            Some((kind, self.calculate_rotation(y_rot, game_time)))
        }
    }

    fn calculate_rotation(&self, y_rot: f64, game_time: i64) -> i8 {
        if self.dimension == "minecraft:the_nether" {
            let s = (game_time / 10) as i32;
            ((s * s * 34_187_121 + s * 121) >> 15 & 15) as i8
        } else {
            let adjusted = if y_rot < 0.0 {
                y_rot - 8.0
            } else {
                y_rot + 8.0
            };
            (adjusted * 16.0 / 360.0) as i8
        }
    }

    fn mark_colors_dirty(&mut self, x: usize, y: usize) {
        if self.dirty_data {
            self.min_dirty_x = self.min_dirty_x.min(x);
            self.min_dirty_y = self.min_dirty_y.min(y);
            self.max_dirty_x = self.max_dirty_x.max(x);
            self.max_dirty_y = self.max_dirty_y.max(y);
        } else {
            self.dirty_data = true;
            self.min_dirty_x = x;
            self.min_dirty_y = y;
            self.max_dirty_x = x;
            self.max_dirty_y = y;
        }
    }

    fn adjust_tracked_count(&mut self, decoration: Option<&MapDecoration>, amount: i32) {
        if decoration.is_some_and(|decoration| decoration.kind.tracks_count()) {
            if amount < 0 {
                self.tracked_decoration_count = self.tracked_decoration_count.saturating_sub(1);
            } else {
                self.tracked_decoration_count += 1;
            }
        }
    }
}

impl MapDecorationKind {
    fn tracks_count(self) -> bool {
        matches!(
            self,
            MapDecorationKind::Player
                | MapDecorationKind::PlayerOffMap
                | MapDecorationKind::PlayerOffLimits
                | MapDecorationKind::Frame
                | MapDecorationKind::Banner(_)
                | MapDecorationKind::Target
        )
    }
}

pub fn frame_id(pos: BlockPos) -> String {
    format!("frame-{},{},{}", pos.x, pos.y, pos.z)
}

pub fn frame_key(entity_id: i32) -> String {
    format!("frame-{entity_id}")
}

pub fn item_frame_y_rotation(
    direction_2d: i32,
    item_rotation: i32,
    vertical_axis_step: Option<i32>,
) -> i32 {
    let rotation_correction = vertical_axis_step.map(|step| 90 * step).unwrap_or(0);
    wrap_degrees(180 + direction_2d * 90 + item_rotation * 45 + rotation_correction)
}

fn wrap_degrees(mut degrees: i32) -> i32 {
    degrees %= 360;
    if degrees >= 180 {
        degrees -= 360;
    }
    if degrees < -180 {
        degrees += 360;
    }
    degrees
}

fn inside_map(x_delta: f32, y_delta: f32) -> bool {
    x_delta >= -63.0 && y_delta >= -63.0 && x_delta <= 63.0 && y_delta <= 63.0
}

fn clamp_map_coordinate(delta: f32) -> i8 {
    if delta <= -63.0 {
        -128
    } else if delta >= 63.0 {
        127
    } else {
        (delta * 2.0 + 0.5) as i8
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decorations_use_vanilla_coordinate_clamp_rotation_and_off_map_rules() {
        let mut map = MapState::new(0, 0, 0, "minecraft:overworld", true, false, false);
        assert!(map.add_decoration(MapDecorationUpdate {
            kind: MapDecorationKind::Player,
            key: "Steve".to_string(),
            x_pos: 10.0,
            z_pos: -63.0,
            y_rot: 90.0,
            name: None,
            game_time: 0,
        }));
        assert_eq!(
            map.decorations["Steve"],
            MapDecoration {
                kind: MapDecorationKind::Player,
                x: 20,
                y: -128,
                rot: 4,
                name: None
            }
        );

        assert!(map.add_decoration(MapDecorationUpdate {
            kind: MapDecorationKind::Player,
            key: "Alex".to_string(),
            x_pos: 100.0,
            z_pos: 0.0,
            y_rot: 0.0,
            name: None,
            game_time: 0,
        }));
        assert_eq!(
            map.decorations["Alex"].kind,
            MapDecorationKind::PlayerOffMap
        );
        assert!(!map.add_decoration(MapDecorationUpdate {
            kind: MapDecorationKind::Player,
            key: "Far".to_string(),
            x_pos: 400.0,
            z_pos: 0.0,
            y_rot: 0.0,
            name: None,
            game_time: 0,
        }));
        assert!(!map.decorations.contains_key("Far"));

        let mut unlimited = MapState::new(0, 0, 0, "minecraft:overworld", true, true, false);
        assert!(unlimited.add_decoration(MapDecorationUpdate {
            kind: MapDecorationKind::Player,
            key: "Far".to_string(),
            x_pos: 400.0,
            z_pos: 0.0,
            y_rot: 0.0,
            name: None,
            game_time: 0,
        }));
        assert_eq!(
            unlimited.decorations["Far"].kind,
            MapDecorationKind::PlayerOffLimits
        );
    }

    #[test]
    fn banners_toggle_only_inside_map_and_remove_when_world_banner_changes() {
        let mut map = MapState::new(0, 0, 0, "minecraft:overworld", true, false, false);
        let banner = MapBanner {
            pos: BlockPos { x: 1, y: 64, z: 2 },
            color: DyeColor::Red,
            name: Some("Base".to_string()),
        };
        assert!(map.toggle_banner(Some(banner.clone())));
        assert!(map.banner_markers.contains_key(&banner.id()));
        assert_eq!(
            map.decorations[&banner.id()].kind,
            MapDecorationKind::Banner(DyeColor::Red)
        );

        assert!(map.toggle_banner(Some(banner.clone())));
        assert!(!map.banner_markers.contains_key(&banner.id()));
        assert!(!map.decorations.contains_key(&banner.id()));

        assert!(map.toggle_banner(Some(banner.clone())));
        map.check_banner_column(1, 2, |_| None);
        assert!(!map.banner_markers.contains_key(&banner.id()));
    }

    #[test]
    fn frames_mark_decorations_and_use_frame_ids_and_rotation_formula() {
        let mut map = MapState::new(0, 0, 0, "minecraft:overworld", true, false, false);
        let frame = MapFrame {
            pos: BlockPos { x: 4, y: 70, z: 5 },
            rotation: 90,
            entity_id: 42,
        };
        assert!(map.add_frame(frame.clone()));
        assert!(map.frame_markers.contains_key("frame-4,70,5"));
        assert!(map.decorations.contains_key("frame-42"));
        assert_eq!(map.decorations["frame-42"].rot, 4);
        assert_eq!(item_frame_y_rotation(1, 2, None), 0);
        assert_eq!(item_frame_y_rotation(0, 0, Some(-1)), 90);

        map.removed_from_frame(frame.pos, frame.entity_id);
        assert!(!map.frame_markers.contains_key("frame-4,70,5"));
        assert!(!map.decorations.contains_key("frame-42"));
    }

    #[test]
    fn dirty_color_and_decoration_updates_follow_holding_player_packet_rules() {
        let mut map = MapState::new(0, 0, 2, "minecraft:overworld", true, false, true);
        let initial = map.next_update_packet().unwrap();
        assert_eq!(
            initial.patch,
            Some(MapPatch {
                start_x: 0,
                start_y: 0,
                width: 128,
                height: 128
            })
        );
        assert!(initial.decorations.is_some());

        assert!(map.next_update_packet().is_none());
        assert!(map.update_color(10, 20, 5));
        assert!(!map.update_color(10, 20, 5));
        let color_update = map.next_update_packet().unwrap();
        assert_eq!(
            color_update.patch,
            Some(MapPatch {
                start_x: 10,
                start_y: 20,
                width: 1,
                height: 1
            })
        );

        assert!(map.add_target_decoration(
            "target",
            BlockPos { x: 3, y: 0, z: 4 },
            MapDecorationKind::Target
        ));
        // Java flushes decorations only on the call where the per-view tick counter
        // is a multiple of 5, and that counter advances ONLY while decorations are
        // dirty (`dirtyDecorations && tick++ % 5 == 0`). So once dirty it takes up to
        // five calls to flush; the intermediate calls return no packet here (the
        // colour patch was already sent and cleared above).
        let mut flush_calls = 0;
        let decoration_update = loop {
            flush_calls += 1;
            assert!(flush_calls <= 5, "decorations must flush within 5 ticks");
            if let Some(packet) = map.next_update_packet() {
                if packet.decorations.is_some() {
                    break packet;
                }
            }
        };
        assert_eq!(flush_calls, 5);
        assert!(decoration_update
            .decorations
            .unwrap()
            .iter()
            .any(|(key, _)| key == "target"));
    }

    #[test]
    fn nether_rotation_and_tracking_limit_match_saved_data_edges() {
        let mut map = MapState::new(0, 0, 0, "minecraft:the_nether", true, false, false);
        assert!(map.add_decoration(MapDecorationUpdate {
            kind: MapDecorationKind::Player,
            key: "Steve".to_string(),
            x_pos: 0.0,
            z_pos: 0.0,
            y_rot: 0.0,
            name: None,
            game_time: 20,
        }));
        assert_eq!(map.decorations["Steve"].rot, 13);

        map.tracked_decoration_count = MAX_TRACKED_DECORATIONS;
        let banner = MapBanner {
            pos: BlockPos { x: 1, y: 64, z: 1 },
            color: DyeColor::White,
            name: None,
        };
        assert!(!map.toggle_banner(Some(banner)));
    }
}
