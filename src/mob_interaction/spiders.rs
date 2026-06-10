#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CaveSpiderAttributes {
    pub max_health: f32,
    pub movement_speed: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CaveSpiderEntityTypeSurface {
    pub width: f32,
    pub height: f32,
    pub eye_height: f32,
    pub client_tracking_range: i32,
    pub not_in_peaceful: bool,
}

pub const SPIDER_MAX_HEALTH: f32 = 16.0;
pub const SPIDER_MOVEMENT_SPEED: f32 = 0.3;
pub const SPIDER_WIDTH: f32 = 1.4;
pub const SPIDER_HEIGHT: f32 = 0.9;
pub const SPIDER_EYE_HEIGHT: f32 = 0.65;
pub const SPIDER_CLIENT_TRACKING_RANGE: i32 = 8;
pub const SPIDER_CLIMBING_FLAG: u8 = 1;
pub const SPIDER_SPECIAL_EFFECT_CHANCE: f32 = 0.1;
pub const SPIDER_JOCKEY_RANDOM_BOUND: i32 = 100;
pub const SPIDER_JOCKEY_RANDOM_HIT: i32 = 0;
pub const SPIDER_ATTACK_LIGHT_BREAK_THRESHOLD: f32 = 0.5;
pub const SPIDER_ATTACK_LIGHT_BREAK_RANDOM_BOUND: i32 = 100;
pub const SPIDER_VEHICLE_ATTACHMENT_Y: f32 = 0.3125;
pub const SPIDER_POISON_IMMUNE: bool = true;
pub const SPIDER_AVOID_ARMADILLO_DISTANCE: f32 = 6.0;
pub const SPIDER_AVOID_ARMADILLO_WALK_SPEED: f32 = 1.0;
pub const SPIDER_AVOID_ARMADILLO_SPRINT_SPEED: f32 = 1.2;
pub const SPIDER_LEAP_AT_TARGET_POWER: f32 = 0.4;
pub const SPIDER_RANDOM_STROLL_SPEED: f32 = 0.8;
pub const SPIDER_LOOK_AT_PLAYER_RANGE: f32 = 8.0;
pub const CAVE_SPIDER_MAX_HEALTH: f32 = 12.0;
pub const CAVE_SPIDER_WIDTH: f32 = 0.7;
pub const CAVE_SPIDER_HEIGHT: f32 = 0.5;
pub const CAVE_SPIDER_EYE_HEIGHT: f32 = 0.45;
pub const CAVE_SPIDER_CLIENT_TRACKING_RANGE: i32 = 8;
pub const CAVE_SPIDER_NOT_IN_PEACEFUL: bool = true;
pub const CAVE_SPIDER_POISON_SECONDS_NORMAL: i32 = 7;
pub const CAVE_SPIDER_POISON_SECONDS_HARD: i32 = 15;
pub const CAVE_SPIDER_POISON_AMPLIFIER: u8 = 0;
pub const CAVE_SPIDER_VEHICLE_ATTACHMENT_Y: f32 = 0.21875;

pub fn cave_spider_attributes() -> CaveSpiderAttributes {
    CaveSpiderAttributes {
        max_health: CAVE_SPIDER_MAX_HEALTH,
        movement_speed: SPIDER_MOVEMENT_SPEED,
    }
}

pub fn cave_spider_entity_type_surface() -> CaveSpiderEntityTypeSurface {
    CaveSpiderEntityTypeSurface {
        width: CAVE_SPIDER_WIDTH,
        height: CAVE_SPIDER_HEIGHT,
        eye_height: CAVE_SPIDER_EYE_HEIGHT,
        client_tracking_range: CAVE_SPIDER_CLIENT_TRACKING_RANGE,
        not_in_peaceful: CAVE_SPIDER_NOT_IN_PEACEFUL,
    }
}

pub fn cave_spider_poison_duration_ticks(
    difficulty: &str,
    super_hurt_succeeded: bool,
) -> Option<i32> {
    if !super_hurt_succeeded {
        return None;
    }
    match difficulty {
        "normal" => Some(CAVE_SPIDER_POISON_SECONDS_NORMAL * 20),
        "hard" => Some(CAVE_SPIDER_POISON_SECONDS_HARD * 20),
        _ => None,
    }
}

pub fn cave_spider_finalize_spawn_preserves_group_data<T>(group_data: Option<T>) -> Option<T> {
    group_data
}

pub fn cave_spider_vehicle_attachment_y(
    vehicle_width: f32,
    cave_spider_width: f32,
    scale: f32,
) -> Option<f32> {
    (vehicle_width <= cave_spider_width).then_some(CAVE_SPIDER_VEHICLE_ATTACHMENT_Y * scale)
}

pub fn spider_set_climbing_flags(flags: u8, climbing: bool) -> u8 {
    if climbing {
        flags | SPIDER_CLIMBING_FLAG
    } else {
        flags & !SPIDER_CLIMBING_FLAG
    }
}

pub fn spider_is_climbing(flags: u8) -> bool {
    flags & SPIDER_CLIMBING_FLAG != 0
}

pub fn spider_tick_climbing_flags(flags: u8, horizontal_collision: bool) -> u8 {
    spider_set_climbing_flags(flags, horizontal_collision)
}

pub fn spider_attack_goal_can_use(super_can_use: bool, is_vehicle: bool) -> bool {
    super_can_use && !is_vehicle
}

pub fn spider_target_goal_can_use(light_value: f32, super_can_use: bool) -> bool {
    light_value < SPIDER_ATTACK_LIGHT_BREAK_THRESHOLD && super_can_use
}

pub fn spider_avoids_armadillo(armadillo_is_scared: bool) -> bool {
    !armadillo_is_scared
}

pub fn spider_can_be_affected(effect_id: &str) -> bool {
    effect_id != "minecraft:poison"
}

pub fn spider_jockey_from_finalize_spawn(random_0_to_99: i32) -> bool {
    random_0_to_99.rem_euclid(SPIDER_JOCKEY_RANDOM_BOUND) == SPIDER_JOCKEY_RANDOM_HIT
}

pub fn spider_should_drop_target_in_light(light_value: f32, random_0_to_99: i32) -> bool {
    light_value >= SPIDER_ATTACK_LIGHT_BREAK_THRESHOLD
        && random_0_to_99.rem_euclid(SPIDER_ATTACK_LIGHT_BREAK_RANDOM_BOUND) == 0
}

pub fn spider_effect_from_group_data_selection(random_0_to_4: i32) -> &'static str {
    match random_0_to_4.rem_euclid(5) {
        0 | 1 => "minecraft:speed",
        2 => "minecraft:strength",
        3 => "minecraft:regeneration",
        _ => "minecraft:invisibility",
    }
}

pub fn spider_should_roll_special_effect(
    difficulty: &str,
    random_float: f32,
    special_multiplier: f32,
) -> bool {
    difficulty == "hard" && random_float < SPIDER_SPECIAL_EFFECT_CHANCE * special_multiplier
}
