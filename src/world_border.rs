#![allow(dead_code)]

pub const WORLD_BORDER_MAX_SIZE: f64 = 5.999997E7;
pub const WORLD_BORDER_MAX_CENTER_COORDINATE: f64 = 2.9999984E7;
pub const WORLD_BORDER_DEFAULT_ABSOLUTE_MAX_SIZE: i32 = 29_999_984;
const BORDER_EPSILON: f64 = 1.0E-5;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WorldBorderSettings {
    pub center_x: f64,
    pub center_z: f64,
    pub damage_per_block: f64,
    pub safe_zone: f64,
    pub warning_blocks: i32,
    pub warning_time: i32,
    pub size: f64,
    pub lerp_time: i64,
    pub lerp_target: f64,
}

impl Default for WorldBorderSettings {
    fn default() -> Self {
        Self {
            center_x: 0.0,
            center_z: 0.0,
            damage_per_block: 0.2,
            safe_zone: 5.0,
            warning_blocks: 5,
            warning_time: 300,
            size: WORLD_BORDER_MAX_SIZE,
            lerp_time: 0,
            lerp_target: 0.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BorderStatus {
    Stationary,
    Growing,
    Shrinking,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BorderExtent {
    Static {
        size: f64,
    },
    Moving {
        from: f64,
        to: f64,
        lerp_duration: i64,
        lerp_progress: i64,
        previous_size: f64,
        size: f64,
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WorldBorder {
    pub center_x: f64,
    pub center_z: f64,
    pub damage_per_block: f64,
    pub safe_zone: f64,
    pub warning_blocks: i32,
    pub warning_time: i32,
    pub absolute_max_size: i32,
    pub extent: BorderExtent,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BorderBox {
    pub min_x: f64,
    pub min_z: f64,
    pub max_x: f64,
    pub max_z: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RespawnData2d {
    pub x: i32,
    pub z: i32,
    pub yaw: f32,
    pub pitch: f32,
}

impl Default for WorldBorder {
    fn default() -> Self {
        Self::from_settings(WorldBorderSettings::default(), 0)
    }
}

impl WorldBorder {
    pub fn from_settings(settings: WorldBorderSettings, _game_time: i64) -> Self {
        let mut border = Self {
            center_x: settings.center_x,
            center_z: settings.center_z,
            damage_per_block: settings.damage_per_block,
            safe_zone: settings.safe_zone,
            warning_blocks: settings.warning_blocks,
            warning_time: settings.warning_time,
            absolute_max_size: WORLD_BORDER_DEFAULT_ABSOLUTE_MAX_SIZE,
            extent: BorderExtent::Static {
                size: settings.size,
            },
        };
        if settings.lerp_time > 0 {
            border.lerp_size_between(settings.size, settings.lerp_target, settings.lerp_time);
        }
        border
    }

    pub fn set_center(&mut self, x: f64, z: f64) {
        self.center_x = x;
        self.center_z = z;
    }

    pub fn set_size(&mut self, size: f64) {
        self.extent = BorderExtent::Static { size };
    }

    pub fn lerp_size_between(&mut self, from: f64, to: f64, ticks: i64) {
        self.extent = if from == to {
            BorderExtent::Static { size: to }
        } else {
            BorderExtent::Moving {
                from,
                to,
                lerp_duration: ticks,
                lerp_progress: ticks,
                previous_size: from,
                size: from,
            }
        };
    }

    pub fn tick(&mut self) {
        if let BorderExtent::Moving {
            from,
            to,
            lerp_duration,
            lerp_progress,
            size,
            ..
        } = self.extent
        {
            let next_progress = lerp_progress - 1;
            let progress = (lerp_duration - next_progress) as f64 / lerp_duration as f64;
            let next_size = if progress < 1.0 {
                lerp(progress, from, to)
            } else {
                to
            };
            self.extent = if next_progress <= 0 {
                BorderExtent::Static { size: to }
            } else {
                BorderExtent::Moving {
                    from,
                    to,
                    lerp_duration,
                    lerp_progress: next_progress,
                    previous_size: size,
                    size: next_size,
                }
            };
        }
    }

    pub fn size(self) -> f64 {
        match self.extent {
            BorderExtent::Static { size } | BorderExtent::Moving { size, .. } => size,
        }
    }

    pub fn lerp_time(self) -> i64 {
        match self.extent {
            BorderExtent::Static { .. } => 0,
            BorderExtent::Moving { lerp_progress, .. } => lerp_progress,
        }
    }

    pub fn lerp_target(self) -> f64 {
        match self.extent {
            BorderExtent::Static { size } => size,
            BorderExtent::Moving { to, .. } => to,
        }
    }

    pub fn lerp_speed(self) -> f64 {
        match self.extent {
            BorderExtent::Static { .. } => 0.0,
            BorderExtent::Moving {
                from,
                to,
                lerp_duration,
                ..
            } => (from - to).abs() / lerp_duration as f64,
        }
    }

    pub fn status(self) -> BorderStatus {
        match self.extent {
            BorderExtent::Static { .. } => BorderStatus::Stationary,
            BorderExtent::Moving { from, to, .. } if to < from => BorderStatus::Shrinking,
            BorderExtent::Moving { .. } => BorderStatus::Growing,
        }
    }

    pub fn bounds(self) -> BorderBox {
        let half = self.size() / 2.0;
        let max = f64::from(self.absolute_max_size);
        BorderBox {
            min_x: (self.center_x - half).clamp(-max, max),
            min_z: (self.center_z - half).clamp(-max, max),
            max_x: (self.center_x + half).clamp(-max, max),
            max_z: (self.center_z + half).clamp(-max, max),
        }
    }

    pub fn is_within_bounds(self, x: f64, z: f64) -> bool {
        self.is_within_bounds_with_margin(x, z, 0.0)
    }

    pub fn is_within_bounds_with_margin(self, x: f64, z: f64, margin: f64) -> bool {
        let bounds = self.bounds();
        x >= bounds.min_x - margin
            && x < bounds.max_x + margin
            && z >= bounds.min_z - margin
            && z < bounds.max_z + margin
    }

    pub fn is_box_within_bounds(self, min_x: f64, min_z: f64, max_x: f64, max_z: f64) -> bool {
        self.is_within_bounds(min_x, min_z)
            && self.is_within_bounds(max_x - BORDER_EPSILON, max_z - BORDER_EPSILON)
    }

    pub fn is_chunk_within_bounds(self, chunk_x: i32, chunk_z: i32) -> bool {
        let min_x = f64::from(chunk_x * 16);
        let min_z = f64::from(chunk_z * 16);
        let max_x = f64::from(chunk_x * 16 + 15);
        let max_z = f64::from(chunk_z * 16 + 15);
        self.is_within_bounds(min_x, min_z) && self.is_within_bounds(max_x, max_z)
    }

    pub fn clamp_vec3_to_bound(self, x: f64, y: f64, z: f64) -> (f64, f64, f64) {
        let bounds = self.bounds();
        (
            x.clamp(bounds.min_x, bounds.max_x - BORDER_EPSILON),
            y,
            z.clamp(bounds.min_z, bounds.max_z - BORDER_EPSILON),
        )
    }

    pub fn distance_to_border(self, x: f64, z: f64) -> f64 {
        let bounds = self.bounds();
        let from_north = z - bounds.min_z;
        let from_south = bounds.max_z - z;
        let from_west = x - bounds.min_x;
        let from_east = bounds.max_x - x;
        from_west.min(from_east).min(from_north).min(from_south)
    }

    pub fn out_of_border_damage(self, x: f64, z: f64) -> Option<i32> {
        let distance_with_safe_zone = self.distance_to_border(x, z) + self.safe_zone;
        if distance_with_safe_zone < 0.0 && self.damage_per_block > 0.0 {
            Some(1.max((-distance_with_safe_zone * self.damage_per_block).floor() as i32))
        } else {
            None
        }
    }

    pub fn adjusted_respawn(
        self,
        respawn: RespawnData2d,
        center_heightmap: impl Fn(i32, i32) -> (i32, i32, i32),
    ) -> RespawnData2d {
        if self.is_within_bounds(f64::from(respawn.x), f64::from(respawn.z)) {
            respawn
        } else {
            let (x, _y, z) =
                center_heightmap(self.center_x.floor() as i32, self.center_z.floor() as i32);
            RespawnData2d {
                x,
                z,
                yaw: respawn.yaw,
                pitch: respawn.pitch,
            }
        }
    }
}

fn lerp(progress: f64, from: f64, to: f64) -> f64 {
    from + progress * (to - from)
}

#[cfg(test)]
mod tests {
    use super::{
        BorderStatus, RespawnData2d, WorldBorder, WorldBorderSettings,
        WORLD_BORDER_DEFAULT_ABSOLUTE_MAX_SIZE, WORLD_BORDER_MAX_CENTER_COORDINATE,
        WORLD_BORDER_MAX_SIZE,
    };

    #[test]
    fn default_world_border_settings_match_vanilla() {
        let settings = WorldBorderSettings::default();
        assert_eq!(settings.center_x, 0.0);
        assert_eq!(settings.center_z, 0.0);
        assert_eq!(settings.damage_per_block, 0.2);
        assert_eq!(settings.safe_zone, 5.0);
        assert_eq!(settings.warning_blocks, 5);
        assert_eq!(settings.warning_time, 300);
        assert_eq!(settings.size, WORLD_BORDER_MAX_SIZE);
        assert_eq!(settings.lerp_time, 0);
        assert_eq!(settings.lerp_target, 0.0);
        assert_eq!(WORLD_BORDER_MAX_CENTER_COORDINATE, 29_999_984.0);
        assert_eq!(WORLD_BORDER_DEFAULT_ABSOLUTE_MAX_SIZE, 29_999_984);
    }

    #[test]
    fn bounds_use_half_size_exclusive_max_and_absolute_clamp() {
        let mut border = WorldBorder::default();
        border.set_size(20.0);
        border.set_center(10.0, -10.0);
        let bounds = border.bounds();
        assert_eq!(bounds.min_x, 0.0);
        assert_eq!(bounds.max_x, 20.0);
        assert_eq!(bounds.min_z, -20.0);
        assert_eq!(bounds.max_z, 0.0);

        assert!(border.is_within_bounds(0.0, -20.0));
        assert!(border.is_within_bounds(19.99999, -0.00001));
        assert!(!border.is_within_bounds(20.0, -10.0));
        assert!(!border.is_within_bounds(10.0, 0.0));

        assert_eq!(
            border.clamp_vec3_to_bound(25.0, 70.0, 5.0),
            (19.99999, 70.0, -0.00001)
        );

        border.set_size(WORLD_BORDER_MAX_SIZE);
        border.set_center(29_999_980.0, 0.0);
        assert_eq!(border.bounds().max_x, 29_999_984.0);
    }

    #[test]
    fn chunk_and_box_bounds_match_vanilla_edge_rules() {
        let mut border = WorldBorder::default();
        border.set_size(32.0);
        border.set_center(16.0, 16.0);

        assert!(border.is_chunk_within_bounds(0, 0));
        assert!(border.is_chunk_within_bounds(1, 1));
        assert!(!border.is_chunk_within_bounds(2, 0));
        assert!(border.is_box_within_bounds(0.0, 0.0, 32.0, 32.0));
        assert!(!border.is_box_within_bounds(0.0, 0.0, 32.00002, 32.0));
    }

    #[test]
    fn moving_border_lerps_and_becomes_static_after_duration() {
        let mut border = WorldBorder::default();
        border.lerp_size_between(10.0, 20.0, 4);

        assert_eq!(border.status(), BorderStatus::Growing);
        assert_eq!(border.lerp_time(), 4);
        assert_eq!(border.lerp_target(), 20.0);
        assert_eq!(border.lerp_speed(), 2.5);
        assert_eq!(border.size(), 10.0);

        border.tick();
        assert_eq!(border.size(), 12.5);
        assert_eq!(border.lerp_time(), 3);
        border.tick();
        assert_eq!(border.size(), 15.0);
        border.tick();
        assert_eq!(border.size(), 17.5);
        border.tick();
        assert_eq!(border.size(), 20.0);
        assert_eq!(border.status(), BorderStatus::Stationary);
        assert_eq!(border.lerp_time(), 0);
    }

    #[test]
    fn border_distance_damage_and_respawn_adjustment_match_vanilla() {
        let mut border = WorldBorder::default();
        border.set_size(20.0);
        border.set_center(0.0, 0.0);
        border.safe_zone = 5.0;
        border.damage_per_block = 0.2;

        assert_eq!(border.distance_to_border(0.0, 0.0), 10.0);
        assert_eq!(border.distance_to_border(13.0, 0.0), -3.0);
        assert_eq!(border.out_of_border_damage(13.0, 0.0), None);
        assert_eq!(border.out_of_border_damage(20.0, 0.0), Some(1));
        assert_eq!(border.out_of_border_damage(30.0, 0.0), Some(3));

        let respawn = RespawnData2d {
            x: 30,
            z: 0,
            yaw: 45.0,
            pitch: 10.0,
        };
        assert_eq!(
            border.adjusted_respawn(respawn, |x, z| (x, 72, z)),
            RespawnData2d {
                x: 0,
                z: 0,
                yaw: 45.0,
                pitch: 10.0
            }
        );
    }
}
