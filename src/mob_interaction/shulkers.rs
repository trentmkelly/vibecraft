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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShulkerClassSurface {
    pub goal_priorities: &'static [(i32, &'static str)],
    pub target_priorities: &'static [(i32, &'static str)],
    pub movement_emission: &'static str,
    pub sound_source: &'static str,
    pub ambient_sound: &'static str,
    pub death_sound: &'static str,
    pub hurt_sound_open: &'static str,
    pub hurt_sound_closed: &'static str,
    pub look_control: &'static str,
    pub body_control: &'static str,
    pub interpolation: Option<&'static str>,
    pub push_entities: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShulkerRawPeekSideEffects {
    pub armor_bonus: Option<i32>,
    pub sound: &'static str,
    pub game_event: &'static str,
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
pub const SHULKER_GOAL_PRIORITIES: &[(i32, &str)] = &[
    (1, "LookAtPlayerGoal(Player,8.0,0.02,true)"),
    (4, "ShulkerAttackGoal"),
    (7, "ShulkerPeekGoal"),
    (8, "RandomLookAroundGoal"),
];
pub const SHULKER_TARGET_PRIORITIES: &[(i32, &str)] = &[
    (1, "HurtByTargetGoal(alert_others_same_class)"),
    (2, "ShulkerNearestAttackGoal"),
    (3, "ShulkerDefenseAttackGoal"),
];
pub const SHULKER_MOVEMENT_EMISSION: &str = "none";
pub const SHULKER_SOUND_SOURCE: &str = "hostile";
pub const SHULKER_AMBIENT_SOUND: &str = "minecraft:entity.shulker.ambient";
pub const SHULKER_DEATH_SOUND: &str = "minecraft:entity.shulker.death";
pub const SHULKER_HURT_SOUND: &str = "minecraft:entity.shulker.hurt";
pub const SHULKER_HURT_CLOSED_SOUND: &str = "minecraft:entity.shulker.hurt_closed";
pub const SHULKER_OPEN_SOUND: &str = "minecraft:entity.shulker.open";
pub const SHULKER_CLOSE_SOUND: &str = "minecraft:entity.shulker.close";
pub const SHULKER_TELEPORT_SOUND: &str = "minecraft:entity.shulker.teleport";
pub const SHULKER_SHOOT_SOUND: &str = "minecraft:entity.shulker.shoot";
pub const SHULKER_GAME_EVENT_OPEN: &str = "minecraft:container_open";
pub const SHULKER_GAME_EVENT_CLOSE: &str = "minecraft:container_close";
pub const SHULKER_GAME_EVENT_TELEPORT: &str = "minecraft:teleport";
pub const SHULKER_PEEK_GOAL_MIN_TICKS: i32 = 20;
pub const SHULKER_PEEK_GOAL_RANDOM_BOUND: i32 = 3;
pub const SHULKER_TELEPORT_MIN_OFFSET: i32 = -8;
pub const SHULKER_TELEPORT_MAX_OFFSET: i32 = 8;
pub const SHULKER_MOVING_PISTON_BLOCK: &str = "minecraft:moving_piston";

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

pub fn shulker_class_surface() -> ShulkerClassSurface {
    ShulkerClassSurface {
        goal_priorities: SHULKER_GOAL_PRIORITIES,
        target_priorities: SHULKER_TARGET_PRIORITIES,
        movement_emission: SHULKER_MOVEMENT_EMISSION,
        sound_source: SHULKER_SOUND_SOURCE,
        ambient_sound: SHULKER_AMBIENT_SOUND,
        death_sound: SHULKER_DEATH_SOUND,
        hurt_sound_open: SHULKER_HURT_SOUND,
        hurt_sound_closed: SHULKER_HURT_CLOSED_SOUND,
        look_control: "ShulkerLookControl",
        body_control: "ShulkerBodyRotationControl",
        interpolation: None,
        push_entities: false,
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

pub fn shulker_raw_peek_side_effects(raw_peek: i32) -> ShulkerRawPeekSideEffects {
    if raw_peek == 0 {
        ShulkerRawPeekSideEffects {
            armor_bonus: Some(SHULKER_COVERED_ARMOR_BONUS as i32),
            sound: SHULKER_CLOSE_SOUND,
            game_event: SHULKER_GAME_EVENT_CLOSE,
        }
    } else {
        ShulkerRawPeekSideEffects {
            armor_bonus: None,
            sound: SHULKER_OPEN_SOUND,
            game_event: SHULKER_GAME_EVENT_OPEN,
        }
    }
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

pub fn shulker_can_be_collided_with(is_alive: bool) -> bool {
    is_alive
}

pub fn shulker_play_ambient_sound(is_closed: bool) -> bool {
    !is_closed
}

pub fn shulker_set_pos_centered(x: f64, y: f64, z: f64, is_passenger: bool) -> (f64, f64, f64) {
    if is_passenger {
        (x, y, z)
    } else {
        (x.floor() + 0.5, (y + 0.5).floor() + 0.5, z.floor() + 0.5)
    }
}

pub fn shulker_set_pos_resets_peek(old_pos: (i32, i32, i32), new_pos: (i32, i32, i32), tick_count: i32) -> bool {
    tick_count != 0 && old_pos != new_pos
}

pub fn shulker_start_riding_attach_face() -> ShulkerDirection {
    ShulkerDirection::Down
}

pub fn shulker_finalize_spawn_rotations() -> (f32, f32) {
    (0.0, 0.0)
}

pub fn shulker_move_triggers_teleport(mover_type: &'static str) -> bool {
    mover_type == "shulker_box"
}

pub fn shulker_position_blocked(block: &'static str, target_is_current_position: bool) -> bool {
    block != "minecraft:air" && !(block == SHULKER_MOVING_PISTON_BLOCK && target_is_current_position)
}

pub fn shulker_peek_goal_can_use(
    has_target: bool,
    random_0_to_39: i32,
    can_stay_at_attachment: bool,
) -> bool {
    !has_target && random_0_to_39.rem_euclid(SHULKER_PEEK_GOAL_ROLL) == 0 && can_stay_at_attachment
}

pub fn shulker_peek_goal_start_ticks(random_0_to_2: i32) -> i32 {
    SHULKER_PEEK_BASE_SECONDS * (1 + random_0_to_2.rem_euclid(SHULKER_PEEK_GOAL_RANDOM_BOUND))
}

pub fn shulker_peek_goal_can_continue(has_target: bool, peek_time: i32) -> bool {
    !has_target && peek_time > 0
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
