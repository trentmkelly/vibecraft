use super::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GhastAttributes {
    pub max_health: f32,
    pub follow_range: f32,
    pub camera_distance: f32,
    pub flying_speed: f32,
    pub xp_reward: i32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GhastEntityTypeSurface {
    pub width: f32,
    pub height: f32,
    pub eye_height: f32,
    pub passenger_attachment_y: f32,
    pub riding_offset: f32,
    pub client_tracking_range: i32,
    pub fire_immune: bool,
    pub not_in_peaceful: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GhastShootTick {
    pub charge_time: i32,
    pub charging: bool,
    pub warn_level_event: Option<i32>,
    pub shoot_level_event: Option<i32>,
    pub fireball: Option<GhastFireballPlan>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GhastFireballPlan {
    pub spawn_offset: f64,
    pub y_offset_from_ghast_mid: f64,
    pub explosion_power: i32,
}

pub const GHAST_MAX_HEALTH: f32 = 10.0;
pub const GHAST_FOLLOW_RANGE: f32 = 100.0;
pub const GHAST_CAMERA_DISTANCE: f32 = 8.0;
pub const GHAST_FLYING_SPEED: f32 = 0.06;
pub const GHAST_XP_REWARD: i32 = 5;
pub const GHAST_WIDTH: f32 = 4.0;
pub const GHAST_HEIGHT: f32 = 4.0;
pub const GHAST_EYE_HEIGHT: f32 = 2.6;
pub const GHAST_PASSENGER_ATTACHMENT_Y: f32 = 4.0625;
pub const GHAST_RIDING_OFFSET: f32 = 0.5;
pub const GHAST_CLIENT_TRACKING_RANGE: i32 = 10;
pub const GHAST_DEFAULT_EXPLOSION_POWER: i32 = 1;
pub const GHAST_SOUND_VOLUME: f32 = 5.0;
pub const GHAST_TARGET_VERTICAL_RANGE: f64 = 4.0;
pub const GHAST_TARGET_CHANCE_INTERVAL: i32 = 10;
pub const GHAST_SHOOT_MAX_DISTANCE_SQR: f64 = 4096.0;
pub const GHAST_CHARGE_WARN_TICKS: i32 = 10;
pub const GHAST_CHARGE_SHOOT_TICKS: i32 = 20;
pub const GHAST_CHARGE_COOLDOWN_AFTER_SHOT: i32 = -40;
pub const GHAST_WARN_LEVEL_EVENT: i32 = 1015;
pub const GHAST_SHOOT_LEVEL_EVENT: i32 = 1016;
pub const GHAST_FIREBALL_SPAWN_OFFSET: f64 = 4.0;
pub const GHAST_FIREBALL_Y_OFFSET_FROM_MID: f64 = 0.5;
pub const GHAST_FIREBALL_ENTITY_DAMAGE: f32 = 6.0;
pub const GHAST_REFLECTED_FIREBALL_DAMAGE: f32 = 1000.0;
pub const GHAST_RANDOM_FLOAT_MAX_ATTEMPTS: i32 = 64;
pub const GHAST_RANDOM_FLOAT_RANGE: f64 = 16.0;
pub const GHAST_RANDOM_FLOAT_REACHED_DISTANCE_SQR: f64 = 1.0;
pub const GHAST_RANDOM_FLOAT_TOO_FAR_DISTANCE_SQR: f64 = 3600.0;
pub const GHAST_MOVE_FLOAT_DURATION_RANDOM_BOUND: i32 = 5;
pub const GHAST_MOVE_FLOAT_DURATION_MIN_ADD: i32 = 2;
pub const GHAST_MOVE_ACCELERATION_SCALE: f64 = 5.0 / 3.0;
pub const GHAST_LEASH_ELASTIC_DISTANCE: f64 = 10.0;
pub const GHAST_LEASH_SNAP_DISTANCE: f64 = 16.0;

pub fn ghast_attributes() -> GhastAttributes {
    GhastAttributes {
        max_health: GHAST_MAX_HEALTH,
        follow_range: GHAST_FOLLOW_RANGE,
        camera_distance: GHAST_CAMERA_DISTANCE,
        flying_speed: GHAST_FLYING_SPEED,
        xp_reward: GHAST_XP_REWARD,
    }
}

pub fn ghast_entity_type_surface() -> GhastEntityTypeSurface {
    GhastEntityTypeSurface {
        width: GHAST_WIDTH,
        height: GHAST_HEIGHT,
        eye_height: GHAST_EYE_HEIGHT,
        passenger_attachment_y: GHAST_PASSENGER_ATTACHMENT_Y,
        riding_offset: GHAST_RIDING_OFFSET,
        client_tracking_range: GHAST_CLIENT_TRACKING_RANGE,
        fire_immune: true,
        not_in_peaceful: true,
    }
}

pub fn ghast_spawn_allowed(
    peaceful_difficulty: bool,
    random_0_to_19: i32,
    mob_spawn_rules_pass: bool,
) -> bool {
    !peaceful_difficulty && random_0_to_19.rem_euclid(20) == 0 && mob_spawn_rules_pass
}

pub fn ghast_target_predicate_matches(abs_target_y_delta: f64) -> bool {
    abs_target_y_delta <= GHAST_TARGET_VERTICAL_RANGE
}

pub fn ghast_is_reflected_fireball(
    direct_entity: &'static str,
    source_entity: &'static str,
) -> bool {
    direct_entity == "minecraft:fireball" && source_entity == "minecraft:player"
}

pub fn ghast_hurt_damage(
    reflected_fireball: bool,
    invulnerable_to_source: bool,
    incoming_damage: f32,
) -> Option<f32> {
    if reflected_fireball {
        Some(GHAST_REFLECTED_FIREBALL_DAMAGE)
    } else if invulnerable_to_source {
        None
    } else {
        Some(incoming_damage)
    }
}

pub fn ghast_shoot_fireball_tick(
    charge_time: i32,
    target_present: bool,
    target_distance_sqr: f64,
    has_line_of_sight: bool,
    silent: bool,
    explosion_power: i32,
) -> GhastShootTick {
    if !target_present {
        return GhastShootTick {
            charge_time,
            charging: false,
            warn_level_event: None,
            shoot_level_event: None,
            fireball: None,
        };
    }

    if target_distance_sqr < GHAST_SHOOT_MAX_DISTANCE_SQR && has_line_of_sight {
        let next_charge_time = charge_time + 1;
        if next_charge_time == GHAST_CHARGE_SHOOT_TICKS {
            return GhastShootTick {
                charge_time: GHAST_CHARGE_COOLDOWN_AFTER_SHOT,
                charging: false,
                warn_level_event: None,
                shoot_level_event: (!silent).then_some(GHAST_SHOOT_LEVEL_EVENT),
                fireball: Some(GhastFireballPlan {
                    spawn_offset: GHAST_FIREBALL_SPAWN_OFFSET,
                    y_offset_from_ghast_mid: GHAST_FIREBALL_Y_OFFSET_FROM_MID,
                    explosion_power,
                }),
            };
        }

        GhastShootTick {
            charge_time: next_charge_time,
            charging: next_charge_time > GHAST_CHARGE_WARN_TICKS,
            warn_level_event: (next_charge_time == GHAST_CHARGE_WARN_TICKS && !silent)
                .then_some(GHAST_WARN_LEVEL_EVENT),
            shoot_level_event: None,
            fireball: None,
        }
    } else {
        let next_charge_time = if charge_time > 0 {
            charge_time - 1
        } else {
            charge_time
        };
        GhastShootTick {
            charge_time: next_charge_time,
            charging: next_charge_time > GHAST_CHARGE_WARN_TICKS,
            warn_level_event: None,
            shoot_level_event: None,
            fireball: None,
        }
    }
}

pub fn ghast_random_float_can_use(move_control_has_wanted: bool, wanted_distance_sqr: f64) -> bool {
    !move_control_has_wanted
        || wanted_distance_sqr < GHAST_RANDOM_FLOAT_REACHED_DISTANCE_SQR
        || wanted_distance_sqr > GHAST_RANDOM_FLOAT_TOO_FAR_DISTANCE_SQR
}

pub fn ghast_random_float_target(
    center: (f64, f64, f64),
    random_x: f64,
    random_y: f64,
    random_z: f64,
) -> (f64, f64, f64) {
    (
        center.0 + (random_x * 2.0 - 1.0) * GHAST_RANDOM_FLOAT_RANGE,
        center.1 + (random_y * 2.0 - 1.0) * GHAST_RANDOM_FLOAT_RANGE,
        center.2 + (random_z * 2.0 - 1.0) * GHAST_RANDOM_FLOAT_RANGE,
    )
}

pub fn ghast_move_float_duration_tick(current_duration: i32, random_0_to_4: i32) -> i32 {
    current_duration - 1
        + random_0_to_4.rem_euclid(GHAST_MOVE_FLOAT_DURATION_RANDOM_BOUND)
        + GHAST_MOVE_FLOAT_DURATION_MIN_ADD
}

pub fn large_fireball_hit_outcome(mob_griefing: bool, explosion_power: i32) -> (f32, bool, bool) {
    (explosion_power as f32, mob_griefing, true)
}

