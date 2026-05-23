use super::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BlazeAttributes {
    pub attack_damage: f32,
    pub movement_speed: f32,
    pub follow_range: f32,
    pub water_pathfinding_malus: f32,
    pub lava_pathfinding_malus: f32,
    pub fire_neighbor_pathfinding_malus: f32,
    pub fire_pathfinding_malus: f32,
    pub xp_reward: i32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BlazeAiStep {
    pub delta_y: f32,
    pub needs_sync: bool,
    pub allowed_height_offset: f32,
    pub next_height_offset_change_tick: i32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BlazeAttackState {
    pub attack_step: i32,
    pub attack_time: i32,
    pub last_seen: i32,
    pub charged: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlazeAttackAction {
    None,
    Melee {
        cooldown_ticks: i32,
    },
    Charge {
        charge_ticks: i32,
    },
    ShootSmallFireball {
        cooldown_ticks: i32,
        level_event: i32,
    },
    Cooldown {
        cooldown_ticks: i32,
    },
    MoveTowardTarget,
}

pub const BLAZE_ATTACK_DAMAGE: f32 = 6.0;
pub const BLAZE_MOVEMENT_SPEED: f32 = 0.23;
pub const BLAZE_FOLLOW_RANGE: f32 = 48.0;
pub const BLAZE_WATER_PATHFINDING_MALUS: f32 = -1.0;
pub const BLAZE_LAVA_PATHFINDING_MALUS: f32 = 8.0;
pub const BLAZE_FIRE_PATHFINDING_MALUS: f32 = 0.0;
pub const BLAZE_XP_REWARD: i32 = 10;
pub const BLAZE_DEFAULT_ALLOWED_HEIGHT_OFFSET: f32 = 0.5;
pub const BLAZE_HEIGHT_OFFSET_REFRESH_TICKS: i32 = 100;
pub const BLAZE_FALL_DAMPING: f32 = 0.6;
pub const BLAZE_ASCEND_TARGET_Y_SPEED: f32 = 0.3;
pub const BLAZE_ASCEND_ACCELERATION: f32 = 0.3;
pub const BLAZE_MELEE_RANGE_SQR: f32 = 4.0;
pub const BLAZE_MELEE_COOLDOWN_TICKS: i32 = 20;
pub const BLAZE_CHARGE_TICKS: i32 = 60;
pub const BLAZE_FIREBALL_BURST_COOLDOWN_TICKS: i32 = 6;
pub const BLAZE_ATTACK_CYCLE_COOLDOWN_TICKS: i32 = 100;
pub const BLAZE_LOST_SIGHT_CHASE_TICKS: i32 = 5;
pub const BLAZE_SHOOT_LEVEL_EVENT: i32 = 1018;
pub const BLAZE_FIREBALL_INACCURACY: f32 = 2.297;
pub const BLAZE_FIREBALL_SPREAD_SCALE: f32 = 0.5;
pub const BLAZE_LOOT_ITEM: &str = "minecraft:blaze_rod";

pub fn blaze_attributes() -> BlazeAttributes {
    BlazeAttributes {
        attack_damage: BLAZE_ATTACK_DAMAGE,
        movement_speed: BLAZE_MOVEMENT_SPEED,
        follow_range: BLAZE_FOLLOW_RANGE,
        water_pathfinding_malus: BLAZE_WATER_PATHFINDING_MALUS,
        lava_pathfinding_malus: BLAZE_LAVA_PATHFINDING_MALUS,
        fire_neighbor_pathfinding_malus: BLAZE_FIRE_PATHFINDING_MALUS,
        fire_pathfinding_malus: BLAZE_FIRE_PATHFINDING_MALUS,
        xp_reward: BLAZE_XP_REWARD,
    }
}

pub fn blaze_ai_step(
    on_ground: bool,
    delta_y: f32,
    next_height_offset_change_tick: i32,
    sampled_allowed_height_offset: f32,
    target_present: bool,
    target_eye_y_above_self_eye_y: f32,
    can_attack_target: bool,
) -> BlazeAiStep {
    let mut next_delta_y = if !on_ground && delta_y < 0.0 {
        delta_y * BLAZE_FALL_DAMPING
    } else {
        delta_y
    };
    let mut next_tick = next_height_offset_change_tick - 1;
    let mut allowed_height_offset = BLAZE_DEFAULT_ALLOWED_HEIGHT_OFFSET;
    if next_tick <= 0 {
        next_tick = BLAZE_HEIGHT_OFFSET_REFRESH_TICKS;
        allowed_height_offset = sampled_allowed_height_offset;
    }

    let should_ascend = target_present
        && target_eye_y_above_self_eye_y > allowed_height_offset
        && can_attack_target;
    if should_ascend {
        next_delta_y += (BLAZE_ASCEND_TARGET_Y_SPEED - next_delta_y) * BLAZE_ASCEND_ACCELERATION;
    }

    BlazeAiStep {
        delta_y: next_delta_y,
        needs_sync: should_ascend,
        allowed_height_offset,
        next_height_offset_change_tick: next_tick,
    }
}

pub fn blaze_attack_goal_can_use(
    target_present: bool,
    target_alive: bool,
    can_attack: bool,
) -> bool {
    target_present && target_alive && can_attack
}

pub fn blaze_attack_goal_start() -> BlazeAttackState {
    BlazeAttackState {
        attack_step: 0,
        attack_time: 0,
        last_seen: 0,
        charged: false,
    }
}

pub fn blaze_attack_goal_stop(mut state: BlazeAttackState) -> BlazeAttackState {
    state.charged = false;
    state.last_seen = 0;
    state
}

pub fn blaze_attack_goal_tick(
    mut state: BlazeAttackState,
    target_present: bool,
    has_line_of_sight: bool,
    distance_sqr: f32,
    follow_range: f32,
    silent: bool,
) -> (BlazeAttackState, BlazeAttackAction) {
    state.attack_time -= 1;
    if !target_present {
        return (state, BlazeAttackAction::None);
    }

    if has_line_of_sight {
        state.last_seen = 0;
    } else {
        state.last_seen += 1;
    }

    if distance_sqr < BLAZE_MELEE_RANGE_SQR {
        if !has_line_of_sight {
            return (state, BlazeAttackAction::None);
        }
        if state.attack_time <= 0 {
            state.attack_time = BLAZE_MELEE_COOLDOWN_TICKS;
            return (
                state,
                BlazeAttackAction::Melee {
                    cooldown_ticks: BLAZE_MELEE_COOLDOWN_TICKS,
                },
            );
        }
        return (state, BlazeAttackAction::MoveTowardTarget);
    }

    if distance_sqr < follow_range * follow_range && has_line_of_sight {
        if state.attack_time <= 0 {
            state.attack_step += 1;
            if state.attack_step == 1 {
                state.attack_time = BLAZE_CHARGE_TICKS;
                state.charged = true;
                return (
                    state,
                    BlazeAttackAction::Charge {
                        charge_ticks: BLAZE_CHARGE_TICKS,
                    },
                );
            }
            if state.attack_step <= 4 {
                state.attack_time = BLAZE_FIREBALL_BURST_COOLDOWN_TICKS;
                return (
                    state,
                    BlazeAttackAction::ShootSmallFireball {
                        cooldown_ticks: BLAZE_FIREBALL_BURST_COOLDOWN_TICKS,
                        level_event: if silent { 0 } else { BLAZE_SHOOT_LEVEL_EVENT },
                    },
                );
            }

            state.attack_time = BLAZE_ATTACK_CYCLE_COOLDOWN_TICKS;
            state.attack_step = 0;
            state.charged = false;
            return (
                state,
                BlazeAttackAction::Cooldown {
                    cooldown_ticks: BLAZE_ATTACK_CYCLE_COOLDOWN_TICKS,
                },
            );
        }
        return (state, BlazeAttackAction::None);
    }

    if state.last_seen < BLAZE_LOST_SIGHT_CHASE_TICKS {
        return (state, BlazeAttackAction::MoveTowardTarget);
    }

    (state, BlazeAttackAction::None)
}

pub fn blaze_fireball_spread(distance_sqr: f32) -> f32 {
    distance_sqr.sqrt().sqrt() * BLAZE_FIREBALL_SPREAD_SCALE
}

pub fn blaze_is_on_fire(charged: bool) -> bool {
    charged
}

pub fn blaze_is_sensitive_to_water() -> bool {
    true
}

pub fn blaze_light_level_dependent_magic_value() -> f32 {
    1.0
}

pub fn blaze_loot_roll(
    killed_by_player: bool,
    base_roll_0_or_1: i32,
    looting_roll_0_to_level: i32,
) -> i32 {
    if killed_by_player {
        base_roll_0_or_1.clamp(0, 1) + looting_roll_0_to_level.max(0)
    } else {
        0
    }
}

