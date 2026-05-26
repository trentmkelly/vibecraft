
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EndermanAttributes {
    pub max_health: f32,
    pub movement_speed: f32,
    pub attacking_speed_bonus: f32,
    pub attack_damage: f32,
    pub follow_range: f32,
    pub step_height: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EndermanEntityTypeSurface {
    pub width: f32,
    pub height: f32,
    pub eye_height: f32,
    pub passenger_attachment_y: f32,
    pub client_tracking_range: i32,
    pub not_in_peaceful: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EndermanTargetState {
    pub target_change_time: i32,
    pub creepy: bool,
    pub stared_at: bool,
    pub speed_modifier_present: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EndermanHurtResponse {
    NormalHurt,
    NormalHurtAndMaybeTeleport,
    WaterPotionHurtAndTryTeleport64,
    ProjectileTryTeleport64,
}

pub const ENDERMAN_MAX_HEALTH: f32 = 40.0;
pub const ENDERMAN_MOVEMENT_SPEED: f32 = 0.3;
pub const ENDERMAN_ATTACKING_SPEED_BONUS: f32 = 0.15;
pub const ENDERMAN_ATTACK_DAMAGE: f32 = 7.0;
pub const ENDERMAN_FOLLOW_RANGE: f32 = 64.0;
pub const ENDERMAN_STEP_HEIGHT: f32 = 1.0;
pub const ENDERMAN_WIDTH: f32 = 0.6;
pub const ENDERMAN_HEIGHT: f32 = 2.9;
pub const ENDERMAN_EYE_HEIGHT: f32 = 2.55;
pub const ENDERMAN_PASSENGER_ATTACHMENT_Y: f32 = 2.80625;
pub const ENDERMAN_CLIENT_TRACKING_RANGE: i32 = 8;
pub const ENDERMAN_WATER_PATHFINDING_MALUS: f32 = -1.0;
pub const ENDERMAN_STARE_SOUND_COOLDOWN: i32 = 400;
pub const ENDERMAN_MIN_DEAGGRESSION_TIME: i32 = 600;
pub const ENDERMAN_PERSISTENT_ANGER_MIN_SECONDS: i32 = 20;
pub const ENDERMAN_PERSISTENT_ANGER_MAX_SECONDS: i32 = 39;
pub const ENDERMAN_LOOK_AT_PLAYER_RANGE: f32 = 8.0;
pub const ENDERMAN_FREEZE_STARE_DISTANCE_SQR: f32 = 256.0;
pub const ENDERMAN_CLOSE_STARE_TELEPORT_DISTANCE_SQR: f32 = 16.0;
pub const ENDERMAN_FAR_TARGET_TELEPORT_DISTANCE_SQR: f32 = 256.0;
pub const ENDERMAN_FAR_TARGET_TELEPORT_DELAY: i32 = 30;
pub const ENDERMAN_AGGRO_TIME: i32 = 5;
pub const ENDERMAN_STARE_DOT_THRESHOLD: f64 = 0.025;
pub const ENDERMAN_RANDOM_TELEPORT_HORIZONTAL_RANGE: f64 = 64.0;
pub const ENDERMAN_RANDOM_TELEPORT_VERTICAL_RANGE: i32 = 64;
pub const ENDERMAN_TELEPORT_TOWARDS_DISTANCE: f64 = 16.0;
pub const ENDERMAN_TELEPORT_TOWARDS_RANDOM_HORIZONTAL: f64 = 8.0;
pub const ENDERMAN_TELEPORT_TOWARDS_RANDOM_VERTICAL: i32 = 16;
pub const ENDERMAN_PROJECTILE_TELEPORT_ATTEMPTS: i32 = 64;
pub const ENDERMAN_NON_LIVING_DAMAGE_TELEPORT_ROLL: i32 = 10;
pub const ENDERMAN_TAKE_BLOCK_ROLL: i32 = 20;
pub const ENDERMAN_LEAVE_BLOCK_ROLL: i32 = 2000;

pub fn enderman_attributes() -> EndermanAttributes {
    EndermanAttributes {
        max_health: ENDERMAN_MAX_HEALTH,
        movement_speed: ENDERMAN_MOVEMENT_SPEED,
        attacking_speed_bonus: ENDERMAN_ATTACKING_SPEED_BONUS,
        attack_damage: ENDERMAN_ATTACK_DAMAGE,
        follow_range: ENDERMAN_FOLLOW_RANGE,
        step_height: ENDERMAN_STEP_HEIGHT,
    }
}

pub fn enderman_entity_type_surface() -> EndermanEntityTypeSurface {
    EndermanEntityTypeSurface {
        width: ENDERMAN_WIDTH,
        height: ENDERMAN_HEIGHT,
        eye_height: ENDERMAN_EYE_HEIGHT,
        passenger_attachment_y: ENDERMAN_PASSENGER_ATTACHMENT_Y,
        client_tracking_range: ENDERMAN_CLIENT_TRACKING_RANGE,
        not_in_peaceful: true,
    }
}

pub fn enderman_set_target_state(target_present: bool, tick_count: i32) -> EndermanTargetState {
    if target_present {
        EndermanTargetState {
            target_change_time: tick_count,
            creepy: true,
            stared_at: false,
            speed_modifier_present: true,
        }
    } else {
        EndermanTargetState {
            target_change_time: 0,
            creepy: false,
            stared_at: false,
            speed_modifier_present: false,
        }
    }
}

pub fn enderman_stare_sound_allowed(tick_count: i32, last_stare_sound: i32) -> bool {
    tick_count >= last_stare_sound + ENDERMAN_STARE_SOUND_COOLDOWN
}

pub fn enderman_should_daylight_deaggro_and_teleport(
    bright_outside: bool,
    tick_count: i32,
    target_change_time: i32,
    light_magic: f32,
    can_see_sky: bool,
    random_float_0_to_1: f32,
) -> bool {
    bright_outside
        && tick_count >= target_change_time + ENDERMAN_MIN_DEAGGRESSION_TIME
        && light_magic > 0.5
        && can_see_sky
        && random_float_0_to_1 * 30.0 < (light_magic - 0.4) * 2.0
}

pub fn enderman_hurt_response(
    projectile_damage: bool,
    thrown_potion: bool,
    potion_is_water: bool,
    source_entity_is_living: bool,
    random_0_to_9: i32,
) -> EndermanHurtResponse {
    if projectile_damage || thrown_potion {
        if thrown_potion && potion_is_water {
            EndermanHurtResponse::WaterPotionHurtAndTryTeleport64
        } else {
            EndermanHurtResponse::ProjectileTryTeleport64
        }
    } else if !source_entity_is_living && random_0_to_9.rem_euclid(10) != 0 {
        EndermanHurtResponse::NormalHurtAndMaybeTeleport
    } else {
        EndermanHurtResponse::NormalHurt
    }
}

pub fn enderman_freeze_when_looked_at(
    player_target: bool,
    distance_sqr: f32,
    stared_by_player: bool,
) -> bool {
    player_target && distance_sqr <= ENDERMAN_FREEZE_STARE_DISTANCE_SQR && stared_by_player
}

pub fn enderman_look_goal_starts_aggro(pending_target_present: bool) -> Option<i32> {
    pending_target_present.then_some(ENDERMAN_AGGRO_TIME)
}

pub fn enderman_look_goal_tick(
    pending_aggro_time: Option<i32>,
    target_present: bool,
    being_stared_by_target: bool,
    target_distance_sqr: f32,
    teleport_time: i32,
    is_passenger: bool,
) -> (Option<i32>, bool, i32, bool) {
    if let Some(aggro_time) = pending_aggro_time {
        let next = aggro_time - 1;
        return (Some(next), next <= 0, teleport_time, false);
    }
    if !target_present || is_passenger {
        return (None, false, teleport_time, false);
    }
    if being_stared_by_target {
        return (
            None,
            false,
            0,
            target_distance_sqr < ENDERMAN_CLOSE_STARE_TELEPORT_DISTANCE_SQR,
        );
    }
    let next_teleport_time = if target_distance_sqr > ENDERMAN_FAR_TARGET_TELEPORT_DISTANCE_SQR {
        teleport_time + 1
    } else {
        teleport_time
    };
    (
        None,
        false,
        next_teleport_time,
        target_distance_sqr > ENDERMAN_FAR_TARGET_TELEPORT_DISTANCE_SQR
            && teleport_time >= ENDERMAN_FAR_TARGET_TELEPORT_DELAY,
    )
}

pub fn enderman_take_block_can_use(
    carried_block_present: bool,
    mob_griefing: bool,
    random_0_to_19: i32,
) -> bool {
    !carried_block_present
        && mob_griefing
        && random_0_to_19.rem_euclid(ENDERMAN_TAKE_BLOCK_ROLL) == 0
}

pub fn enderman_leave_block_can_use(
    carried_block_present: bool,
    mob_griefing: bool,
    random_0_to_1999: i32,
) -> bool {
    carried_block_present
        && mob_griefing
        && random_0_to_1999.rem_euclid(ENDERMAN_LEAVE_BLOCK_ROLL) == 0
}

pub fn enderman_can_place_carried_block(
    target_is_air: bool,
    below_is_air: bool,
    below_is_bedrock: bool,
    below_full_collision: bool,
    carried_can_survive: bool,
    entity_collision_empty: bool,
) -> bool {
    target_is_air
        && !below_is_air
        && !below_is_bedrock
        && below_full_collision
        && carried_can_survive
        && entity_collision_empty
}

pub fn enderman_requires_custom_persistence(
    super_requires: bool,
    carried_block_present: bool,
) -> bool {
    super_requires || carried_block_present
}

