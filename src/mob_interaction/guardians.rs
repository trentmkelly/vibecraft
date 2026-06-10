#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GuardianAttributes {
    pub max_health: f32,
    pub movement_speed: f32,
    pub attack_damage: f32,
    pub xp_reward: i32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GuardianEntityTypeSurface {
    pub width: f32,
    pub height: f32,
    pub eye_height: f32,
    pub passenger_attachment_y: f32,
    pub client_tracking_range: i32,
    pub not_in_peaceful: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GuardianAttackTick {
    pub attack_time: i32,
    pub active_attack_target: Option<i32>,
    pub broadcast_event: Option<u8>,
    pub magic_damage: Option<f32>,
    pub melee_hit: bool,
    pub clear_target: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ElderGuardianEffectPulse {
    pub mining_fatigue_ticks: i32,
    pub amplifier: i32,
    pub radius: f64,
    pub display_limit_ticks: i32,
    pub game_event_strength: f32,
}

pub const GUARDIAN_MAX_HEALTH: f32 = 30.0;
pub const GUARDIAN_MOVEMENT_SPEED: f32 = 0.5;
pub const GUARDIAN_ATTACK_DAMAGE: f32 = 6.0;
pub const GUARDIAN_XP_REWARD: i32 = 10;
pub const GUARDIAN_WIDTH: f32 = 0.85;
pub const GUARDIAN_HEIGHT: f32 = 0.85;
pub const GUARDIAN_EYE_HEIGHT: f32 = 0.425;
pub const GUARDIAN_PASSENGER_ATTACHMENT_Y: f32 = 0.975;
pub const GUARDIAN_CLIENT_TRACKING_RANGE: i32 = 8;
pub const GUARDIAN_ATTACK_DURATION_TICKS: i32 = 80;
pub const GUARDIAN_ATTACK_START_TICKS: i32 = -10;
pub const GUARDIAN_ATTACK_EVENT_ID: u8 = 21;
pub const GUARDIAN_ATTACK_SELECTOR_MIN_DISTANCE_SQR: f32 = 9.0;
pub const GUARDIAN_TARGET_SCAN_INTERVAL: i32 = 10;
pub const GUARDIAN_WATER_PATHFINDING_MALUS: f32 = 0.0;
pub const GUARDIAN_AMBIENT_SOUND_INTERVAL: i32 = 160;
pub const GUARDIAN_WATER_WALK_TARGET_BASE: f32 = 10.0;
pub const GUARDIAN_AIR_SUPPLY_IN_WATER: i32 = 300;
pub const GUARDIAN_LAND_FLOP_Y_PUSH: f64 = 0.5;
pub const GUARDIAN_LAND_FLOP_XZ_SCALE: f64 = 0.4;
pub const GUARDIAN_THORNS_DAMAGE: f32 = 2.0;
pub const GUARDIAN_TRAVEL_WATER_RELATIVE: f32 = 0.1;
pub const GUARDIAN_TRAVEL_WATER_DAMPING: f64 = 0.9;
pub const GUARDIAN_IDLE_SINKING_Y: f64 = -0.005;
pub const GUARDIAN_MAX_HEAD_X_ROT: i32 = 180;

pub const ELDER_GUARDIAN_MAX_HEALTH: f32 = 80.0;
pub const ELDER_GUARDIAN_MOVEMENT_SPEED: f32 = 0.3;
pub const ELDER_GUARDIAN_ATTACK_DAMAGE: f32 = 8.0;
pub const ELDER_GUARDIAN_WIDTH: f32 = 1.9975;
pub const ELDER_GUARDIAN_HEIGHT: f32 = 1.9975;
pub const ELDER_GUARDIAN_EYE_HEIGHT: f32 = 0.99875;
pub const ELDER_GUARDIAN_PASSENGER_ATTACHMENT_Y: f32 = 2.350625;
pub const ELDER_GUARDIAN_CLIENT_TRACKING_RANGE: i32 = 10;
pub const ELDER_GUARDIAN_ATTACK_DURATION_TICKS: i32 = 60;
pub const ELDER_GUARDIAN_RANDOM_STROLL_INTERVAL: i32 = 400;
pub const ELDER_GUARDIAN_EFFECT_INTERVAL: i32 = 1200;
pub const ELDER_GUARDIAN_EFFECT_RADIUS: f64 = 50.0;
pub const ELDER_GUARDIAN_EFFECT_DURATION: i32 = 6000;
pub const ELDER_GUARDIAN_EFFECT_AMPLIFIER: i32 = 2;
pub const ELDER_GUARDIAN_EFFECT_DISPLAY_LIMIT: i32 = 1200;
pub const ELDER_GUARDIAN_HOME_RADIUS: i32 = 16;

pub fn guardian_attributes() -> GuardianAttributes {
    GuardianAttributes {
        max_health: GUARDIAN_MAX_HEALTH,
        movement_speed: GUARDIAN_MOVEMENT_SPEED,
        attack_damage: GUARDIAN_ATTACK_DAMAGE,
        xp_reward: GUARDIAN_XP_REWARD,
    }
}

pub fn elder_guardian_attributes() -> GuardianAttributes {
    GuardianAttributes {
        max_health: ELDER_GUARDIAN_MAX_HEALTH,
        movement_speed: ELDER_GUARDIAN_MOVEMENT_SPEED,
        attack_damage: ELDER_GUARDIAN_ATTACK_DAMAGE,
        xp_reward: GUARDIAN_XP_REWARD,
    }
}

pub fn guardian_entity_type_surface() -> GuardianEntityTypeSurface {
    GuardianEntityTypeSurface {
        width: GUARDIAN_WIDTH,
        height: GUARDIAN_HEIGHT,
        eye_height: GUARDIAN_EYE_HEIGHT,
        passenger_attachment_y: GUARDIAN_PASSENGER_ATTACHMENT_Y,
        client_tracking_range: GUARDIAN_CLIENT_TRACKING_RANGE,
        not_in_peaceful: true,
    }
}

pub fn elder_guardian_entity_type_surface() -> GuardianEntityTypeSurface {
    GuardianEntityTypeSurface {
        width: ELDER_GUARDIAN_WIDTH,
        height: ELDER_GUARDIAN_HEIGHT,
        eye_height: ELDER_GUARDIAN_EYE_HEIGHT,
        passenger_attachment_y: ELDER_GUARDIAN_PASSENGER_ATTACHMENT_Y,
        client_tracking_range: ELDER_GUARDIAN_CLIENT_TRACKING_RANGE,
        not_in_peaceful: true,
    }
}

pub fn guardian_spawn_allowed(
    random_0_to_19: i32,
    can_see_sky_from_below_water: bool,
    peaceful_difficulty: bool,
    spawn_reason_is_spawner: bool,
    current_fluid_is_water: bool,
    below_fluid_is_water: bool,
) -> bool {
    (random_0_to_19.rem_euclid(20) == 0 || !can_see_sky_from_below_water)
        && !peaceful_difficulty
        && (spawn_reason_is_spawner || current_fluid_is_water)
        && below_fluid_is_water
}

pub fn guardian_attack_selector_matches(
    target_entity_type: &'static str,
    distance_sqr: f32,
) -> bool {
    matches!(
        target_entity_type,
        "minecraft:player" | "minecraft:squid" | "minecraft:axolotl"
    ) && distance_sqr > GUARDIAN_ATTACK_SELECTOR_MIN_DISTANCE_SQR
}

pub fn guardian_attack_can_continue(
    elder: bool,
    target_present: bool,
    target_distance_sqr: f32,
    super_can_continue: bool,
) -> bool {
    super_can_continue
        && (elder
            || (target_present && target_distance_sqr > GUARDIAN_ATTACK_SELECTOR_MIN_DISTANCE_SQR))
}

pub fn guardian_attack_tick(
    attack_time: i32,
    target_id: i32,
    has_line_of_sight: bool,
    silent: bool,
    hard_difficulty: bool,
    elder: bool,
) -> GuardianAttackTick {
    if !has_line_of_sight {
        return GuardianAttackTick {
            attack_time,
            active_attack_target: None,
            broadcast_event: None,
            magic_damage: None,
            melee_hit: false,
            clear_target: true,
        };
    }

    let next = attack_time + 1;
    if next == 0 {
        return GuardianAttackTick {
            attack_time: next,
            active_attack_target: Some(target_id),
            broadcast_event: (!silent).then_some(GUARDIAN_ATTACK_EVENT_ID),
            magic_damage: None,
            melee_hit: false,
            clear_target: false,
        };
    }

    let duration = if elder {
        ELDER_GUARDIAN_ATTACK_DURATION_TICKS
    } else {
        GUARDIAN_ATTACK_DURATION_TICKS
    };
    if next >= duration {
        let mut damage = 1.0;
        if hard_difficulty {
            damage += 2.0;
        }
        if elder {
            damage += 2.0;
        }
        GuardianAttackTick {
            attack_time: next,
            active_attack_target: None,
            broadcast_event: None,
            magic_damage: Some(damage),
            melee_hit: true,
            clear_target: true,
        }
    } else {
        GuardianAttackTick {
            attack_time: next,
            active_attack_target: None,
            broadcast_event: None,
            magic_damage: None,
            melee_hit: false,
            clear_target: false,
        }
    }
}

pub fn guardian_thorns_damage(
    moving: bool,
    source_avoids_guardian_thorns: bool,
    source_is_thorns: bool,
    direct_entity_is_living: bool,
) -> Option<f32> {
    (!moving && !source_avoids_guardian_thorns && !source_is_thorns && direct_entity_is_living)
        .then_some(GUARDIAN_THORNS_DAMAGE)
}

pub fn guardian_walk_target_value(in_water: bool, light_level_cost: f32, fallback: f32) -> f32 {
    if in_water {
        GUARDIAN_WATER_WALK_TARGET_BASE + light_level_cost
    } else {
        fallback
    }
}

pub fn elder_guardian_effect_pulse(
    tick_count: i32,
    entity_id: i32,
    silent: bool,
) -> Option<ElderGuardianEffectPulse> {
    ((tick_count + entity_id).rem_euclid(ELDER_GUARDIAN_EFFECT_INTERVAL) == 0).then_some(
        ElderGuardianEffectPulse {
            mining_fatigue_ticks: ELDER_GUARDIAN_EFFECT_DURATION,
            amplifier: ELDER_GUARDIAN_EFFECT_AMPLIFIER,
            radius: ELDER_GUARDIAN_EFFECT_RADIUS,
            display_limit_ticks: ELDER_GUARDIAN_EFFECT_DISPLAY_LIMIT,
            game_event_strength: if silent { 0.0 } else { 1.0 },
        },
    )
}

pub fn elder_guardian_sets_home_when_missing(has_home: bool) -> Option<i32> {
    (!has_home).then_some(ELDER_GUARDIAN_HOME_RADIUS)
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RavagerAttributes {
    pub max_health: f32,
    pub movement_speed: f32,
    pub knockback_resistance: f32,
    pub attack_damage: f32,
    pub attack_knockback: f32,
    pub follow_range: f32,
    pub step_height: f32,
    pub xp_reward: i32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RavagerEntityTypeSurface {
    pub width: f32,
    pub height: f32,
    pub passenger_attachment_y: f32,
    pub passenger_attachment_z: f32,
    pub client_tracking_range: i32,
    pub not_in_peaceful: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RavagerAiStep {
    pub movement_speed: f32,
    pub attack_tick: i32,
    pub stunned_tick: i32,
    pub roar_tick: i32,
    pub roar_now: bool,
    pub start_roar_sound: bool,
    pub should_jump_after_leaf_collision: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RavagerBlockedByItem {
    pub stunned_tick: i32,
    pub roar_tick: i32,
    pub stun_event: Option<u8>,
    pub strong_knockback: bool,
    pub defender_hurt_marked: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RavagerRoarEffect {
    pub damage: Option<f32>,
    pub strong_knockback: bool,
    pub include_armor_stand: bool,
    pub event: Option<u8>,
}

pub const RAVAGER_MAX_HEALTH: f32 = 100.0;
pub const RAVAGER_BASE_MOVEMENT_SPEED: f32 = 0.3;
pub const RAVAGER_ATTACK_MOVEMENT_SPEED: f32 = 0.35;
pub const RAVAGER_KNOCKBACK_RESISTANCE: f32 = 0.75;
pub const RAVAGER_ATTACK_DAMAGE: f32 = 12.0;
pub const RAVAGER_ATTACK_KNOCKBACK: f32 = 1.5;
pub const RAVAGER_FOLLOW_RANGE: f32 = 32.0;
pub const RAVAGER_STEP_HEIGHT: f32 = 1.0;
pub const RAVAGER_XP_REWARD: i32 = 20;
pub const RAVAGER_WIDTH: f32 = 1.95;
pub const RAVAGER_HEIGHT: f32 = 2.2;
pub const RAVAGER_PASSENGER_ATTACHMENT_Y: f32 = 2.2625;
pub const RAVAGER_PASSENGER_ATTACHMENT_Z: f32 = -0.0625;
pub const RAVAGER_CLIENT_TRACKING_RANGE: i32 = 10;
pub const RAVAGER_ATTACK_DURATION: i32 = 10;
pub const RAVAGER_STUN_DURATION: i32 = 40;
pub const RAVAGER_ROAR_WINDUP_TICKS: i32 = 20;
pub const RAVAGER_ROAR_DAMAGE_TICK: i32 = 10;
pub const RAVAGER_ROAR_RADIUS: f32 = 4.0;
pub const RAVAGER_ROAR_DAMAGE: f32 = 6.0;
pub const RAVAGER_ATTACK_EVENT_ID: u8 = 4;
pub const RAVAGER_STUN_EVENT_ID: u8 = 39;
pub const RAVAGER_ROAR_EVENT_ID: u8 = 69;
pub const RAVAGER_LEAVES_PATHFINDING_MALUS: f32 = 0.0;
pub const RAVAGER_MAX_HEAD_Y_ROT: i32 = 45;
pub const RAVAGER_ATTACK_BB_DEFLATE_XZ: f64 = 0.05;
pub const RAVAGER_RANDOM_STROLL_SPEED: f32 = 0.4;
pub const RAVAGER_LOOK_AT_PLAYER_RANGE: f32 = 6.0;
pub const RAVAGER_LOOK_AT_MOB_RANGE: f32 = 8.0;

pub fn ravager_attributes() -> RavagerAttributes {
    RavagerAttributes {
        max_health: RAVAGER_MAX_HEALTH,
        movement_speed: RAVAGER_BASE_MOVEMENT_SPEED,
        knockback_resistance: RAVAGER_KNOCKBACK_RESISTANCE,
        attack_damage: RAVAGER_ATTACK_DAMAGE,
        attack_knockback: RAVAGER_ATTACK_KNOCKBACK,
        follow_range: RAVAGER_FOLLOW_RANGE,
        step_height: RAVAGER_STEP_HEIGHT,
        xp_reward: RAVAGER_XP_REWARD,
    }
}
