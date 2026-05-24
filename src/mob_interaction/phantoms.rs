use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhantomBlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhantomAttackPhase {
    Circle,
    Swoop,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PhantomState {
    pub size: i32,
    pub anchor_point: Option<PhantomBlockPos>,
    pub attack_phase: PhantomAttackPhase,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PhantomAttributes {
    pub attack_damage: f32,
    pub xp_reward: i32,
    pub dimensions_scale: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhantomAttackStrategy {
    pub attack_phase: PhantomAttackPhase,
    pub next_sweep_tick: i32,
    pub anchor_point: Option<PhantomBlockPos>,
    pub played_swoop_sound: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhantomSwoopContinuation {
    Continue,
    StopNoTarget,
    StopDeadTarget,
    StopCreativeOrSpectatorPlayer,
    StopNoLongerSwooping,
    StopScaredOfCat,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhantomSwoopTick {
    Flying,
    HitTarget { level_event: Option<i32> },
    CancelledToCircle,
}

pub const PHANTOM_MIN_SIZE: i32 = 0;
pub const PHANTOM_MAX_SIZE: i32 = 64;
pub const PHANTOM_DEFAULT_SIZE: i32 = 0;
pub const PHANTOM_XP_REWARD: i32 = 5;
pub const PHANTOM_BASE_ATTACK_DAMAGE: f32 = 6.0;
pub const PHANTOM_DIMENSIONS_SCALE_PER_SIZE: f32 = 0.15;
pub const PHANTOM_FLAP_DEGREES_PER_TICK: f32 = 7.448451;
pub const PHANTOM_TICKS_PER_FLAP: i32 = 25;
pub const PHANTOM_UNIQUE_FLAP_TICK_OFFSET_MULTIPLIER: i32 = 3;
pub const PHANTOM_FINALIZE_ANCHOR_ABOVE: i32 = 5;
pub const PHANTOM_TARGET_SCAN_INITIAL_TICKS: i32 = 20;
pub const PHANTOM_TARGET_SCAN_RESET_TICKS: i32 = 60;
pub const PHANTOM_TARGET_RANGE: f32 = 64.0;
pub const PHANTOM_TARGET_BOX_INFLATE_XZ: f32 = 16.0;
pub const PHANTOM_TARGET_BOX_INFLATE_Y: f32 = 64.0;
pub const PHANTOM_ATTACK_STRATEGY_START_SWEEP_TICKS: i32 = 10;
pub const PHANTOM_SWEEP_DELAY_BASE_SECONDS: i32 = 8;
pub const PHANTOM_SWEEP_DELAY_RANDOM_SECONDS_BOUND: i32 = 4;
pub const PHANTOM_ANCHOR_ABOVE_TARGET_MIN: i32 = 20;
pub const PHANTOM_ANCHOR_ABOVE_TARGET_RANDOM_BOUND: i32 = 20;
pub const PHANTOM_STOP_ANCHOR_ABOVE_HEIGHTMAP_MIN: i32 = 10;
pub const PHANTOM_STOP_ANCHOR_ABOVE_HEIGHTMAP_RANDOM_BOUND: i32 = 20;
pub const PHANTOM_CIRCLE_DISTANCE_MIN: f32 = 5.0;
pub const PHANTOM_CIRCLE_DISTANCE_RANDOM_SPAN: f32 = 10.0;
pub const PHANTOM_CIRCLE_DISTANCE_MAX: f32 = 15.0;
pub const PHANTOM_CIRCLE_HEIGHT_BASE: f32 = -4.0;
pub const PHANTOM_CIRCLE_HEIGHT_RANDOM_SPAN: f32 = 9.0;
pub const PHANTOM_CIRCLE_ANGLE_STEP_DEGREES: f32 = 15.0;
pub const PHANTOM_TOUCHING_TARGET_DISTANCE_SQUARED: f64 = 4.0;
pub const PHANTOM_SWEEP_CAT_SEARCH_TICK_DELAY: i32 = 20;
pub const PHANTOM_CAT_AVOID_INFLATE: f32 = 16.0;
pub const PHANTOM_SWEEP_HIT_INFLATE: f32 = 0.2;
pub const PHANTOM_SWEEP_HIT_LEVEL_EVENT: i32 = 1039;
pub const PHANTOM_LOOT_ITEM: &str = "minecraft:phantom_membrane";
pub const PHANTOM_USES_NEAREST_PLAYERS_MEMORY: bool = false;
pub const PHANTOM_BURNS_IN_DAYLIGHT: bool = false;

impl PhantomState {
    pub fn new() -> Self {
        Self {
            size: PHANTOM_DEFAULT_SIZE,
            anchor_point: None,
            attack_phase: PhantomAttackPhase::Circle,
        }
    }

    pub fn set_size(&mut self, size: i32) {
        self.size = clamp_phantom_size(size);
    }

    pub fn read_save_data(size: Option<i32>, anchor_point: Option<PhantomBlockPos>) -> Self {
        let mut state = Self::new();
        state.set_size(size.unwrap_or(PHANTOM_DEFAULT_SIZE));
        state.anchor_point = anchor_point;
        state
    }

    pub fn finalize_spawn(block_position: PhantomBlockPos) -> Self {
        Self {
            size: PHANTOM_DEFAULT_SIZE,
            anchor_point: Some(block_position.above(PHANTOM_FINALIZE_ANCHOR_ABOVE)),
            attack_phase: PhantomAttackPhase::Circle,
        }
    }

    pub fn attributes(self) -> PhantomAttributes {
        PhantomAttributes {
            attack_damage: PHANTOM_BASE_ATTACK_DAMAGE + self.size as f32,
            xp_reward: PHANTOM_XP_REWARD,
            dimensions_scale: 1.0 + PHANTOM_DIMENSIONS_SCALE_PER_SIZE * self.size as f32,
        }
    }

    pub fn is_flapping(self, entity_id: i32, tick_count: i32) -> bool {
        (phantom_unique_flap_tick_offset(entity_id) + tick_count).rem_euclid(PHANTOM_TICKS_PER_FLAP)
            == 0
    }
}

impl Default for PhantomState {
    fn default() -> Self {
        Self::new()
    }
}

impl PhantomBlockPos {
    pub fn above(self, amount: i32) -> Self {
        Self {
            y: self.y + amount,
            ..self
        }
    }
}

pub fn clamp_phantom_size(size: i32) -> i32 {
    size.clamp(PHANTOM_MIN_SIZE, PHANTOM_MAX_SIZE)
}

pub fn phantom_unique_flap_tick_offset(entity_id: i32) -> i32 {
    entity_id * PHANTOM_UNIQUE_FLAP_TICK_OFFSET_MULTIPLIER
}

pub fn phantom_target_scan_tick(next_scan_tick: i32) -> Option<i32> {
    (next_scan_tick > 0).then_some(next_scan_tick - 1)
}

pub fn phantom_target_scan_reset_ticks() -> i32 {
    PHANTOM_TARGET_SCAN_RESET_TICKS
}

pub fn phantom_can_continue_swoop(
    has_target: bool,
    target_alive: bool,
    target_player_creative: bool,
    target_player_spectator: bool,
    attack_phase: PhantomAttackPhase,
    cats_nearby: bool,
) -> PhantomSwoopContinuation {
    if !has_target {
        PhantomSwoopContinuation::StopNoTarget
    } else if !target_alive {
        PhantomSwoopContinuation::StopDeadTarget
    } else if target_player_creative || target_player_spectator {
        PhantomSwoopContinuation::StopCreativeOrSpectatorPlayer
    } else if attack_phase != PhantomAttackPhase::Swoop {
        PhantomSwoopContinuation::StopNoLongerSwooping
    } else if cats_nearby {
        PhantomSwoopContinuation::StopScaredOfCat
    } else {
        PhantomSwoopContinuation::Continue
    }
}

pub fn phantom_attack_strategy_start(
    target_pos: PhantomBlockPos,
    target_anchor_random_0_to_19: i32,
    sea_level: i32,
) -> PhantomAttackStrategy {
    PhantomAttackStrategy {
        attack_phase: PhantomAttackPhase::Circle,
        next_sweep_tick: PHANTOM_ATTACK_STRATEGY_START_SWEEP_TICKS,
        anchor_point: Some(phantom_anchor_above_target(
            target_pos,
            target_anchor_random_0_to_19,
            sea_level,
        )),
        played_swoop_sound: false,
    }
}

pub fn phantom_attack_strategy_tick(
    attack_phase: PhantomAttackPhase,
    next_sweep_tick: i32,
    target_pos: PhantomBlockPos,
    target_anchor_random_0_to_19: i32,
    next_sweep_random_0_to_3: i32,
    sea_level: i32,
) -> PhantomAttackStrategy {
    if attack_phase != PhantomAttackPhase::Circle {
        return PhantomAttackStrategy {
            attack_phase,
            next_sweep_tick,
            anchor_point: None,
            played_swoop_sound: false,
        };
    }
    let decremented = next_sweep_tick - 1;
    if decremented > 0 {
        return PhantomAttackStrategy {
            attack_phase,
            next_sweep_tick: decremented,
            anchor_point: None,
            played_swoop_sound: false,
        };
    }
    PhantomAttackStrategy {
        attack_phase: PhantomAttackPhase::Swoop,
        next_sweep_tick: (PHANTOM_SWEEP_DELAY_BASE_SECONDS
            + next_sweep_random_0_to_3.rem_euclid(PHANTOM_SWEEP_DELAY_RANDOM_SECONDS_BOUND))
            * 20,
        anchor_point: Some(phantom_anchor_above_target(
            target_pos,
            target_anchor_random_0_to_19,
            sea_level,
        )),
        played_swoop_sound: true,
    }
}

pub fn phantom_anchor_above_target(
    target_pos: PhantomBlockPos,
    random_0_to_19: i32,
    sea_level: i32,
) -> PhantomBlockPos {
    let mut anchor = target_pos.above(
        PHANTOM_ANCHOR_ABOVE_TARGET_MIN
            + random_0_to_19.rem_euclid(PHANTOM_ANCHOR_ABOVE_TARGET_RANDOM_BOUND),
    );
    if anchor.y < sea_level {
        anchor.y = sea_level + 1;
    }
    anchor
}

pub fn phantom_stop_anchor_after_heightmap(
    anchor_x: i32,
    heightmap_y: i32,
    anchor_z: i32,
    random_0_to_19: i32,
) -> PhantomBlockPos {
    PhantomBlockPos {
        x: anchor_x,
        y: heightmap_y
            + PHANTOM_STOP_ANCHOR_ABOVE_HEIGHTMAP_MIN
            + random_0_to_19.rem_euclid(PHANTOM_STOP_ANCHOR_ABOVE_HEIGHTMAP_RANDOM_BOUND),
        z: anchor_z,
    }
}

pub fn phantom_swoop_tick(
    target_intersects_inflated_box: bool,
    horizontal_collision: bool,
    hurt_time_positive: bool,
    silent: bool,
) -> PhantomSwoopTick {
    if target_intersects_inflated_box {
        PhantomSwoopTick::HitTarget {
            level_event: (!silent).then_some(PHANTOM_SWEEP_HIT_LEVEL_EVENT),
        }
    } else if horizontal_collision || hurt_time_positive {
        PhantomSwoopTick::CancelledToCircle
    } else {
        PhantomSwoopTick::Flying
    }
}

pub fn phantom_burns_in_daylight() -> bool {
    PHANTOM_BURNS_IN_DAYLIGHT
}

pub fn phantom_uses_nearest_players_memory() -> bool {
    PHANTOM_USES_NEAREST_PLAYERS_MEMORY
}

pub fn phantom_membrane_loot_roll(
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

