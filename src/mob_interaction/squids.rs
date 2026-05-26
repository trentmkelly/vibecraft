
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SquidState {
    pub movement_vector: (f64, f64, f64),
    pub tentacle_movement: f32,
    pub tentacle_speed: f32,
    pub tentacle_angle: f32,
    pub rotate_speed: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GlowSquidState {
    pub dark_ticks_remaining: i32,
}

pub const SQUID_MAX_HEALTH: f32 = 10.0;
pub const SQUID_DEFAULT_GRAVITY: f64 = 0.08;
pub const SQUID_SOUND_VOLUME: f32 = 0.4;
pub const SQUID_BABY_WIDTH: f32 = 0.5;
pub const SQUID_BABY_HEIGHT: f32 = 0.63;
pub const SQUID_BABY_EYE_HEIGHT: f32 = 0.37;
pub const SQUID_INK_PARTICLE_COUNT: i32 = 30;
pub const SQUID_INK_BABY_OFFSET_SCALE: f32 = 0.1;
pub const SQUID_INK_ADULT_OFFSET_SCALE: f32 = 0.3;
pub const SQUID_FLEE_SPEED: f64 = 3.0;
pub const SQUID_FLEE_MIN_DISTANCE: f64 = 5.0;
pub const SQUID_FLEE_MAX_DISTANCE: f64 = 10.0;
pub const SQUID_FLEE_DISTANCE_SQUARED: f64 = 100.0;
pub const SQUID_FLEE_VECTOR_SCALE: f64 = 20.0;
pub const SQUID_BUBBLE_INTERVAL_TICKS: i32 = 10;
pub const SQUID_BUBBLE_PHASE_TICK: i32 = 5;
pub const GLOW_SQUID_DEFAULT_DARK_TICKS_REMAINING: i32 = 0;
pub const GLOW_SQUID_DARK_TICKS_ON_HURT: i32 = 100;
pub const GLOW_SQUID_SPAWN_SEA_LEVEL_OFFSET: i32 = 33;

impl SquidState {
    pub fn new(tentacle_speed_random_float: f32) -> Self {
        Self {
            movement_vector: (0.0, 0.0, 0.0),
            tentacle_movement: 0.0,
            tentacle_speed: 1.0 / (tentacle_speed_random_float.clamp(0.0, 1.0) + 1.0) * 0.2,
            tentacle_angle: 0.0,
            rotate_speed: 0.0,
        }
    }

    pub fn has_movement_vector(self) -> bool {
        let (x, y, z) = self.movement_vector;
        x * x + y * y + z * z > 1.0E-5
    }

    pub fn handle_entity_event(&mut self, event_id: u8) -> bool {
        if event_id == 19 {
            self.tentacle_movement = 0.0;
            true
        } else {
            false
        }
    }
}

impl GlowSquidState {
    pub fn new() -> Self {
        Self {
            dark_ticks_remaining: GLOW_SQUID_DEFAULT_DARK_TICKS_REMAINING,
        }
    }

    pub fn ai_step(&mut self) {
        if self.dark_ticks_remaining > 0 {
            self.dark_ticks_remaining -= 1;
        }
    }

    pub fn on_hurt(&mut self, hurt: bool) {
        if hurt {
            self.dark_ticks_remaining = GLOW_SQUID_DARK_TICKS_ON_HURT;
        }
    }
}

pub fn squid_hurt_spawns_ink(super_hurt: bool, last_hurt_by_mob_present: bool) -> bool {
    super_hurt && last_hurt_by_mob_present
}

pub fn squid_flee_can_use(
    in_water: bool,
    last_hurt_by_mob_present: bool,
    distance_squared: f64,
) -> bool {
    in_water && last_hurt_by_mob_present && distance_squared < SQUID_FLEE_DISTANCE_SQUARED
}

pub fn squid_flee_vector(
    squid_pos: (f64, f64, f64),
    attacker_pos: (f64, f64, f64),
    target_block_is_water: bool,
    target_block_is_air: bool,
) -> Option<(f64, f64, f64)> {
    if !target_block_is_water && !target_block_is_air {
        return None;
    }
    let mut x = squid_pos.0 - attacker_pos.0;
    let mut y = squid_pos.1 - attacker_pos.1;
    let mut z = squid_pos.2 - attacker_pos.2;
    let length = (x * x + y * y + z * z).sqrt();
    if length > 0.0 {
        x /= length;
        y /= length;
        z /= length;
        let mut avoid_speed = SQUID_FLEE_SPEED;
        if length > SQUID_FLEE_MIN_DISTANCE {
            avoid_speed -= (length - SQUID_FLEE_MIN_DISTANCE) / SQUID_FLEE_MIN_DISTANCE;
        }
        if avoid_speed > 0.0 {
            x *= avoid_speed;
            y *= avoid_speed;
            z *= avoid_speed;
        }
    }
    if target_block_is_air {
        y = 0.0;
    }
    Some((
        x / SQUID_FLEE_VECTOR_SCALE,
        y / SQUID_FLEE_VECTOR_SCALE,
        z / SQUID_FLEE_VECTOR_SCALE,
    ))
}

pub fn squid_flee_emits_bubble(flee_ticks: i32) -> bool {
    flee_ticks.rem_euclid(SQUID_BUBBLE_INTERVAL_TICKS) == SQUID_BUBBLE_PHASE_TICK
}

pub fn glow_squid_spawn_allowed(
    y: i32,
    sea_level: i32,
    raw_brightness: i32,
    block_is_water: bool,
) -> bool {
    y <= sea_level - GLOW_SQUID_SPAWN_SEA_LEVEL_OFFSET && raw_brightness == 0 && block_is_water
}

