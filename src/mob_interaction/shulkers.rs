
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShulkerDirection {
    Down,
    Up,
    North,
    South,
    West,
    East,
}

impl ShulkerDirection {
    pub fn axis(self) -> ShulkerAxis {
        match self {
            ShulkerDirection::Down | ShulkerDirection::Up => ShulkerAxis::Y,
            ShulkerDirection::North | ShulkerDirection::South => ShulkerAxis::Z,
            ShulkerDirection::West | ShulkerDirection::East => ShulkerAxis::X,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShulkerAxis {
    X,
    Y,
    Z,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShulkerAttributes {
    pub max_health: f32,
    pub covered_armor_bonus: f32,
    pub xp_reward: i32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShulkerEntityTypeSurface {
    pub width: f32,
    pub height: f32,
    pub eye_height: f32,
    pub client_tracking_range: i32,
    pub fire_immune: bool,
    pub can_spawn_far_from_player: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShulkerBulletSurface {
    pub width: f32,
    pub height: f32,
    pub client_tracking_range: i32,
    pub no_loot_table: bool,
    pub no_physics: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShulkerAttackTick {
    pub attack_time: i32,
    pub raw_peek: i32,
    pub shoot_bullet: bool,
    pub clear_target: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShulkerHitByBullet {
    pub should_spawn_baby: bool,
    pub failure_chance: f32,
}

pub const SHULKER_MAX_HEALTH: f32 = 30.0;
pub const SHULKER_XP_REWARD: i32 = 5;
pub const SHULKER_WIDTH: f32 = 1.0;
pub const SHULKER_HEIGHT: f32 = 1.0;
pub const SHULKER_EYE_HEIGHT: f32 = 0.5;
pub const SHULKER_CLIENT_TRACKING_RANGE: i32 = 10;
pub const SHULKER_DEFAULT_ATTACH_FACE: ShulkerDirection = ShulkerDirection::Down;
pub const SHULKER_DEFAULT_PEEK: i32 = 0;
pub const SHULKER_ATTACK_PEEK: i32 = 100;
pub const SHULKER_IDLE_PEEK: i32 = 30;
pub const SHULKER_NO_COLOR: i32 = 16;
pub const SHULKER_DEFAULT_COLOR: i32 = 16;
pub const SHULKER_COVERED_ARMOR_BONUS: f32 = 20.0;
pub const SHULKER_TELEPORT_STEPS: i32 = 6;
pub const SHULKER_MAX_TELEPORT_DISTANCE: i32 = 8;
pub const SHULKER_TELEPORT_ATTEMPTS: i32 = 5;
pub const SHULKER_OTHER_SCAN_RADIUS: f32 = 8.0;
pub const SHULKER_OTHER_LIMIT: f32 = 5.0;
pub const SHULKER_PEEK_PER_TICK: f32 = 0.05;
pub const SHULKER_MAX_SCALE: f32 = 3.0;
pub const SHULKER_MAX_LID_OPEN: f32 = 1.0;
pub const SHULKER_LOOK_AT_PLAYER_RANGE: f32 = 8.0;
pub const SHULKER_LOOK_AT_PLAYER_PROBABILITY: f32 = 0.02;
pub const SHULKER_TARGET_RANGE_SQR: f32 = 400.0;
pub const SHULKER_ATTACK_START_TICKS: i32 = 20;
pub const SHULKER_ATTACK_RANDOM_STEP: i32 = 10;
pub const SHULKER_ATTACK_RANDOM_MULTIPLIER: i32 = 10;
pub const SHULKER_PEEK_GOAL_ROLL: i32 = 40;
pub const SHULKER_PEEK_BASE_SECONDS: i32 = 20;
pub const SHULKER_BULLET_SPEED: f32 = 0.15;
pub const SHULKER_BULLET_GRAVITY: f32 = 0.04;
pub const SHULKER_BULLET_DAMAGE: f32 = 4.0;
pub const SHULKER_BULLET_LEVITATION_TICKS: i32 = 200;
pub const SHULKER_BULLET_WIDTH: f32 = 0.3125;
pub const SHULKER_BULLET_HEIGHT: f32 = 0.3125;
pub const SHULKER_BULLET_CLIENT_TRACKING_RANGE: i32 = 8;
pub const SHULKER_MAX_HEAD_X_ROT: i32 = 180;
pub const SHULKER_MAX_HEAD_Y_ROT: i32 = 180;
pub const SHULKER_RENDER_DISTANCE_SQR: f32 = 16384.0;

pub fn shulker_attributes() -> ShulkerAttributes {
    ShulkerAttributes {
        max_health: SHULKER_MAX_HEALTH,
        covered_armor_bonus: SHULKER_COVERED_ARMOR_BONUS,
        xp_reward: SHULKER_XP_REWARD,
    }
}

pub fn shulker_entity_type_surface() -> ShulkerEntityTypeSurface {
    ShulkerEntityTypeSurface {
        width: SHULKER_WIDTH,
        height: SHULKER_HEIGHT,
        eye_height: SHULKER_EYE_HEIGHT,
        client_tracking_range: SHULKER_CLIENT_TRACKING_RANGE,
        fire_immune: true,
        can_spawn_far_from_player: true,
    }
}

pub fn shulker_bullet_surface() -> ShulkerBulletSurface {
    ShulkerBulletSurface {
        width: SHULKER_BULLET_WIDTH,
        height: SHULKER_BULLET_HEIGHT,
        client_tracking_range: SHULKER_BULLET_CLIENT_TRACKING_RANGE,
        no_loot_table: true,
        no_physics: true,
    }
}

pub fn shulker_update_peek_amount(current: f32, raw_peek: i32) -> f32 {
    let target = raw_peek as f32 * 0.01;
    if current > target {
        (current - SHULKER_PEEK_PER_TICK).clamp(target, SHULKER_MAX_LID_OPEN)
    } else if current < target {
        (current + SHULKER_PEEK_PER_TICK).clamp(0.0, target)
    } else {
        current
    }
}

pub fn shulker_raw_peek_armor_bonus(raw_peek: i32) -> Option<f32> {
    (raw_peek == 0).then_some(SHULKER_COVERED_ARMOR_BONUS)
}

pub fn shulker_color_from_data(color: i32) -> Option<i32> {
    (0..=15).contains(&color).then_some(color)
}

pub fn shulker_sanitized_scale(scale: f32) -> f32 {
    scale.min(SHULKER_MAX_SCALE)
}

pub fn shulker_hurt_allowed(raw_peek: i32, direct_entity_type: &'static str) -> bool {
    !(raw_peek == 0 && direct_entity_type == "minecraft:arrow")
}

pub fn shulker_should_teleport_after_hurt(
    health: f32,
    max_health: f32,
    random_0_to_3: i32,
) -> bool {
    health < max_health * 0.5 && random_0_to_3.rem_euclid(4) == 0
}

pub fn shulker_hit_by_bullet(
    raw_peek: i32,
    teleport_succeeded: bool,
    nearby_shulker_count: i32,
    random_float_0_to_1: f32,
) -> ShulkerHitByBullet {
    let failure_chance = (nearby_shulker_count - 1) as f32 / SHULKER_OTHER_LIMIT;
    ShulkerHitByBullet {
        should_spawn_baby: raw_peek != 0
            && teleport_succeeded
            && random_float_0_to_1 >= failure_chance,
        failure_chance,
    }
}

pub fn shulker_attack_can_use(target_alive: bool, peaceful_difficulty: bool) -> bool {
    target_alive && !peaceful_difficulty
}

pub fn shulker_attack_tick(
    attack_time: i32,
    peaceful_difficulty: bool,
    target_alive: bool,
    distance_sqr: f32,
    random_0_to_9: i32,
) -> ShulkerAttackTick {
    if peaceful_difficulty || !target_alive {
        return ShulkerAttackTick {
            attack_time,
            raw_peek: SHULKER_ATTACK_PEEK,
            shoot_bullet: false,
            clear_target: false,
        };
    }

    let next = attack_time - 1;
    if distance_sqr >= SHULKER_TARGET_RANGE_SQR {
        return ShulkerAttackTick {
            attack_time: next,
            raw_peek: SHULKER_ATTACK_PEEK,
            shoot_bullet: false,
            clear_target: true,
        };
    }

    if next <= 0 {
        ShulkerAttackTick {
            attack_time: 20 + random_0_to_9.rem_euclid(10) * SHULKER_ATTACK_RANDOM_MULTIPLIER / 2,
            raw_peek: SHULKER_ATTACK_PEEK,
            shoot_bullet: true,
            clear_target: false,
        }
    } else {
        ShulkerAttackTick {
            attack_time: next,
            raw_peek: SHULKER_ATTACK_PEEK,
            shoot_bullet: false,
            clear_target: false,
        }
    }
}

pub fn shulker_defense_search_inflate(
    attach_face: ShulkerDirection,
    follow_distance: f32,
) -> (f32, f32, f32) {
    match attach_face.axis() {
        ShulkerAxis::X => (4.0, follow_distance, follow_distance),
        ShulkerAxis::Z => (follow_distance, follow_distance, 4.0),
        ShulkerAxis::Y => (follow_distance, 4.0, follow_distance),
    }
}

pub fn shulker_bullet_on_hit_entity(
    target_hurt: bool,
    target_is_living: bool,
) -> Option<(f32, i32)> {
    (target_hurt && target_is_living)
        .then_some((SHULKER_BULLET_DAMAGE, SHULKER_BULLET_LEVITATION_TICKS))
}

pub fn shulker_bullet_discards_in_peaceful(peaceful_difficulty: bool) -> bool {
    peaceful_difficulty
}

