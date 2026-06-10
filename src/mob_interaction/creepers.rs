#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CreeperState {
    pub old_swell: i32,
    pub swell: i32,
    pub max_swell: i32,
    pub explosion_radius: i32,
    pub swell_dir: i32,
    pub powered: bool,
    pub ignited: bool,
    pub dropped_skulls: bool,
    pub alive: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CreeperTickResult {
    pub primed_sound: bool,
    pub prime_fuse_game_event: bool,
    pub explosion_radius: Option<f32>,
    pub lingering_cloud: Option<CreeperLingeringCloud>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CreeperLingeringCloud {
    pub radius: f32,
    pub radius_on_use: f32,
    pub wait_time: i32,
    pub duration: i32,
    pub potion_duration_scale: f32,
    pub radius_per_tick: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CreeperIgniterUse {
    NotIgniter,
    ConsumeOne,
    DamageOne,
}

pub const CREEPER_DEFAULT_SWELL_DIR: i32 = -1;
pub const CREEPER_DEFAULT_POWERED: bool = false;
pub const CREEPER_DEFAULT_IGNITED: bool = false;
pub const CREEPER_DEFAULT_MAX_SWELL: i32 = 30;
pub const CREEPER_DEFAULT_EXPLOSION_RADIUS: i32 = 3;
pub const CREEPER_MOVEMENT_SPEED: f32 = 0.25;
pub const CREEPER_SWELL_FALL_DAMAGE_FACTOR: f64 = 1.5;
pub const CREEPER_SWELL_FALL_DAMAGE_CLAMP_OFFSET: i32 = 5;
pub const CREEPER_POWERED_EXPLOSION_MULTIPLIER: f32 = 2.0;
pub const CREEPER_NORMAL_EXPLOSION_MULTIPLIER: f32 = 1.0;
pub const CREEPER_SWELL_RENDER_DENOMINATOR_OFFSET: i32 = 2;
pub const CREEPER_CAT_AVOID_DISTANCE: f32 = 6.0;
pub const CREEPER_CAT_AVOID_WALK_SPEED: f32 = 1.0;
pub const CREEPER_CAT_AVOID_SPRINT_SPEED: f32 = 1.2;
pub const CREEPER_STROLL_SPEED: f32 = 0.8;
pub const CREEPER_LOOK_AT_PLAYER_DISTANCE: f32 = 8.0;
pub const CREEPER_LINGERING_CLOUD_RADIUS: f32 = 2.5;
pub const CREEPER_LINGERING_CLOUD_RADIUS_ON_USE: f32 = -0.5;
pub const CREEPER_LINGERING_CLOUD_WAIT_TIME: i32 = 10;
pub const CREEPER_LINGERING_CLOUD_DURATION: i32 = 300;
pub const CREEPER_LINGERING_CLOUD_POTION_DURATION_SCALE: f32 = 0.25;

impl CreeperState {
    pub fn new() -> Self {
        Self {
            old_swell: 0,
            swell: 0,
            max_swell: CREEPER_DEFAULT_MAX_SWELL,
            explosion_radius: CREEPER_DEFAULT_EXPLOSION_RADIUS,
            swell_dir: CREEPER_DEFAULT_SWELL_DIR,
            powered: CREEPER_DEFAULT_POWERED,
            ignited: CREEPER_DEFAULT_IGNITED,
            dropped_skulls: false,
            alive: true,
        }
    }

    pub fn read_save_data(
        powered: bool,
        fuse: Option<i16>,
        explosion_radius: Option<i8>,
        ignited: bool,
    ) -> Self {
        let mut state = Self::new();
        state.powered = powered;
        state.max_swell = fuse.unwrap_or(CREEPER_DEFAULT_MAX_SWELL as i16) as i32;
        state.explosion_radius =
            explosion_radius.unwrap_or(CREEPER_DEFAULT_EXPLOSION_RADIUS as i8) as i32;
        if ignited {
            state.ignite();
        }
        state
    }

    pub fn ignite(&mut self) {
        self.ignited = true;
    }

    pub fn thunder_hit(&mut self) {
        self.powered = true;
    }

    pub fn set_swell_dir(&mut self, dir: i32) {
        self.swell_dir = dir;
    }

    pub fn cause_fall_damage(&mut self, fall_distance: f64) {
        self.swell += (fall_distance * CREEPER_SWELL_FALL_DAMAGE_FACTOR) as i32;
        let max_preloaded_swell = self.max_swell - CREEPER_SWELL_FALL_DAMAGE_CLAMP_OFFSET;
        if self.swell > max_preloaded_swell {
            self.swell = max_preloaded_swell;
        }
    }

    pub fn tick(&mut self, active_effect_count: usize) -> CreeperTickResult {
        let mut result = CreeperTickResult {
            primed_sound: false,
            prime_fuse_game_event: false,
            explosion_radius: None,
            lingering_cloud: None,
        };
        if !self.alive {
            return result;
        }

        self.old_swell = self.swell;
        if self.ignited {
            self.set_swell_dir(1);
        }

        if self.swell_dir > 0 && self.swell == 0 {
            result.primed_sound = true;
            result.prime_fuse_game_event = true;
        }

        self.swell += self.swell_dir;
        if self.swell < 0 {
            self.swell = 0;
        }
        if self.swell >= self.max_swell {
            self.swell = self.max_swell;
            result.explosion_radius = Some(self.effective_explosion_radius());
            if active_effect_count > 0 {
                result.lingering_cloud = Some(creeper_lingering_cloud());
            }
            self.alive = false;
        }

        result
    }

    pub fn effective_explosion_radius(self) -> f32 {
        let multiplier = if self.powered {
            CREEPER_POWERED_EXPLOSION_MULTIPLIER
        } else {
            CREEPER_NORMAL_EXPLOSION_MULTIPLIER
        };
        self.explosion_radius as f32 * multiplier
    }

    pub fn swelling(self, partial_tick: f32) -> f32 {
        let interpolated =
            self.old_swell as f32 + (self.swell - self.old_swell) as f32 * partial_tick;
        interpolated / (self.max_swell - CREEPER_SWELL_RENDER_DENOMINATOR_OFFSET) as f32
    }

    pub fn killed_entity_drops_charged_creeper_loot(&mut self, should_drop_loot: bool) -> bool {
        if should_drop_loot && self.powered && !self.dropped_skulls {
            self.dropped_skulls = true;
            true
        } else {
            false
        }
    }

    pub fn can_target(self, target_is_goat: bool) -> bool {
        !target_is_goat
    }
}

pub fn creeper_igniter_use(is_igniter: bool, is_damageable_item: bool) -> CreeperIgniterUse {
    if !is_igniter {
        CreeperIgniterUse::NotIgniter
    } else if is_damageable_item {
        CreeperIgniterUse::DamageOne
    } else {
        CreeperIgniterUse::ConsumeOne
    }
}

pub fn creeper_lingering_cloud() -> CreeperLingeringCloud {
    CreeperLingeringCloud {
        radius: CREEPER_LINGERING_CLOUD_RADIUS,
        radius_on_use: CREEPER_LINGERING_CLOUD_RADIUS_ON_USE,
        wait_time: CREEPER_LINGERING_CLOUD_WAIT_TIME,
        duration: CREEPER_LINGERING_CLOUD_DURATION,
        potion_duration_scale: CREEPER_LINGERING_CLOUD_POTION_DURATION_SCALE,
        radius_per_tick: -CREEPER_LINGERING_CLOUD_RADIUS / CREEPER_LINGERING_CLOUD_DURATION as f32,
    }
}
