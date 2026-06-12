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
pub struct PhantomGoalSurface {
    pub goal_priorities: &'static [(i32, &'static str)],
    pub target_priorities: &'static [(i32, &'static str)],
    pub move_control: &'static str,
    pub look_control: &'static str,
    pub body_control: &'static str,
    pub sound_source: &'static str,
    pub ambient_sound: &'static str,
    pub hurt_sound: &'static str,
    pub death_sound: &'static str,
    pub should_render_at_any_distance: bool,
    pub on_climbable: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PhantomClientTickSurface {
    pub flap_sound: &'static str,
    pub flap_sound_volume_min: f32,
    pub flap_sound_volume_random_span: f32,
    pub flap_sound_pitch_min: f32,
    pub flap_sound_pitch_random_span: f32,
    pub particle: &'static str,
    pub particle_count: i32,
    pub particle_width_multiplier: f32,
    pub particle_height_base: f32,
    pub particle_height_anim_multiplier: f32,
    pub particle_height_scale: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PhantomCircleStart {
    pub distance: f32,
    pub height: f32,
    pub clockwise: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PhantomBodyRotation {
    pub y_head_rot: f32,
    pub y_body_rot: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PhantomMoveControlSurface {
    pub initial_speed: f32,
    pub horizontal_collision_yaw_flip: f32,
    pub turn_step_degrees: f32,
    pub fast_speed: f32,
    pub slow_speed: f32,
    pub fast_turn_threshold_degrees: f32,
    pub fast_approach_base: f32,
    pub slow_approach: f32,
    pub delta_movement_approach: f32,
    pub y_relative_scale: f32,
    pub horizontal_distance_epsilon: f64,
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
pub const PHANTOM_TRAVEL_FLYING_FRICTION: f32 = 0.2;
pub const PHANTOM_GOAL_PRIORITIES: &[(i32, &str)] = &[
    (1, "PhantomAttackStrategyGoal"),
    (2, "PhantomSweepAttackGoal"),
    (3, "PhantomCircleAroundAnchorGoal"),
];
pub const PHANTOM_TARGET_PRIORITIES: &[(i32, &str)] = &[(1, "PhantomAttackPlayerTargetGoal")];
pub const PHANTOM_FLAP_SOUND: &str = "minecraft:entity.phantom.flap";
pub const PHANTOM_FLAP_SOUND_VOLUME_MIN: f32 = 0.95;
pub const PHANTOM_FLAP_SOUND_VOLUME_RANDOM_SPAN: f32 = 0.05;
pub const PHANTOM_FLAP_SOUND_PITCH_MIN: f32 = 0.95;
pub const PHANTOM_FLAP_SOUND_PITCH_RANDOM_SPAN: f32 = 0.05;
pub const PHANTOM_PARTICLE: &str = "minecraft:mycelium";
pub const PHANTOM_PARTICLE_COUNT: i32 = 2;
pub const PHANTOM_PARTICLE_WIDTH_MULTIPLIER: f32 = 1.48;
pub const PHANTOM_PARTICLE_HEIGHT_BASE: f32 = 0.3;
pub const PHANTOM_PARTICLE_HEIGHT_ANIM_MULTIPLIER: f32 = 0.45;
pub const PHANTOM_PARTICLE_HEIGHT_SCALE: f32 = 2.5;
pub const PHANTOM_SOUND_SOURCE: &str = "hostile";
pub const PHANTOM_AMBIENT_SOUND: &str = "minecraft:entity.phantom.ambient";
pub const PHANTOM_HURT_SOUND: &str = "minecraft:entity.phantom.hurt";
pub const PHANTOM_DEATH_SOUND: &str = "minecraft:entity.phantom.death";
pub const PHANTOM_SWOOP_SOUND: &str = "minecraft:entity.phantom.swoop";
pub const PHANTOM_SWOOP_SOUND_VOLUME: f32 = 10.0;
pub const PHANTOM_SWOOP_SOUND_PITCH_MIN: f32 = 0.95;
pub const PHANTOM_SWOOP_SOUND_PITCH_RANDOM_SPAN: f32 = 0.1;
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
pub const PHANTOM_CIRCLE_HEIGHT_RESELECT_TICKS: i32 = 350;
pub const PHANTOM_CIRCLE_DISTANCE_RESELECT_TICKS: i32 = 250;
pub const PHANTOM_CIRCLE_ANGLE_RESELECT_TICKS: i32 = 450;
pub const PHANTOM_CIRCLE_BLOCKED_BELOW_MIN_HEIGHT: f32 = 1.0;
pub const PHANTOM_CIRCLE_BLOCKED_ABOVE_MAX_HEIGHT: f32 = -1.0;
pub const PHANTOM_TOUCHING_TARGET_DISTANCE_SQUARED: f64 = 4.0;
pub const PHANTOM_MOVE_CONTROL_INITIAL_SPEED: f32 = 0.1;
pub const PHANTOM_MOVE_CONTROL_COLLISION_YAW_FLIP: f32 = 180.0;
pub const PHANTOM_MOVE_CONTROL_TURN_STEP_DEGREES: f32 = 4.0;
pub const PHANTOM_MOVE_CONTROL_FAST_SPEED: f32 = 1.8;
pub const PHANTOM_MOVE_CONTROL_SLOW_SPEED: f32 = 0.2;
pub const PHANTOM_MOVE_CONTROL_FAST_TURN_THRESHOLD_DEGREES: f32 = 3.0;
pub const PHANTOM_MOVE_CONTROL_FAST_APPROACH_BASE: f32 = 0.005;
pub const PHANTOM_MOVE_CONTROL_SLOW_APPROACH: f32 = 0.025;
pub const PHANTOM_MOVE_CONTROL_DELTA_MOVEMENT_APPROACH: f32 = 0.2;
pub const PHANTOM_MOVE_CONTROL_Y_RELATIVE_SCALE: f32 = 0.7;
pub const PHANTOM_MOVE_CONTROL_HORIZONTAL_DISTANCE_EPSILON: f64 = 1.0E-5;
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

pub fn phantom_goal_surface() -> PhantomGoalSurface {
    PhantomGoalSurface {
        goal_priorities: PHANTOM_GOAL_PRIORITIES,
        target_priorities: PHANTOM_TARGET_PRIORITIES,
        move_control: "PhantomMoveControl",
        look_control: "PhantomLookControl",
        body_control: "PhantomBodyRotationControl",
        sound_source: PHANTOM_SOUND_SOURCE,
        ambient_sound: PHANTOM_AMBIENT_SOUND,
        hurt_sound: PHANTOM_HURT_SOUND,
        death_sound: PHANTOM_DEATH_SOUND,
        should_render_at_any_distance: true,
        on_climbable: false,
    }
}

pub fn phantom_client_tick_surface() -> PhantomClientTickSurface {
    PhantomClientTickSurface {
        flap_sound: PHANTOM_FLAP_SOUND,
        flap_sound_volume_min: PHANTOM_FLAP_SOUND_VOLUME_MIN,
        flap_sound_volume_random_span: PHANTOM_FLAP_SOUND_VOLUME_RANDOM_SPAN,
        flap_sound_pitch_min: PHANTOM_FLAP_SOUND_PITCH_MIN,
        flap_sound_pitch_random_span: PHANTOM_FLAP_SOUND_PITCH_RANDOM_SPAN,
        particle: PHANTOM_PARTICLE,
        particle_count: PHANTOM_PARTICLE_COUNT,
        particle_width_multiplier: PHANTOM_PARTICLE_WIDTH_MULTIPLIER,
        particle_height_base: PHANTOM_PARTICLE_HEIGHT_BASE,
        particle_height_anim_multiplier: PHANTOM_PARTICLE_HEIGHT_ANIM_MULTIPLIER,
        particle_height_scale: PHANTOM_PARTICLE_HEIGHT_SCALE,
    }
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

pub fn phantom_circle_start(
    distance_random_0_to_1: f32,
    height_random_0_to_1: f32,
    clockwise_random_bool: bool,
) -> PhantomCircleStart {
    PhantomCircleStart {
        distance: PHANTOM_CIRCLE_DISTANCE_MIN
            + distance_random_0_to_1.clamp(0.0, 1.0) * PHANTOM_CIRCLE_DISTANCE_RANDOM_SPAN,
        height: PHANTOM_CIRCLE_HEIGHT_BASE
            + height_random_0_to_1.clamp(0.0, 1.0) * PHANTOM_CIRCLE_HEIGHT_RANDOM_SPAN,
        clockwise: if clockwise_random_bool { 1.0 } else { -1.0 },
    }
}

pub fn phantom_circle_distance_tick(distance: f32, clockwise: f32) -> (f32, f32) {
    let next_distance = distance + 1.0;
    if next_distance > PHANTOM_CIRCLE_DISTANCE_MAX {
        (PHANTOM_CIRCLE_DISTANCE_MIN, -clockwise)
    } else {
        (next_distance, clockwise)
    }
}

pub fn phantom_circle_height_after_block_check(
    height: f32,
    target_below_phantom: bool,
    blocked_below: bool,
    target_above_phantom: bool,
    blocked_above: bool,
) -> f32 {
    let mut height = height;
    if target_below_phantom && blocked_below {
        height = height.max(PHANTOM_CIRCLE_BLOCKED_BELOW_MIN_HEIGHT);
    }
    if target_above_phantom && blocked_above {
        height = height.min(PHANTOM_CIRCLE_BLOCKED_ABOVE_MAX_HEIGHT);
    }
    height
}

pub fn phantom_circle_touching_target(distance_squared: f64) -> bool {
    distance_squared < PHANTOM_TOUCHING_TARGET_DISTANCE_SQUARED
}

pub fn phantom_body_rotation_client_tick(y_body_rot: f32, y_rot: f32) -> PhantomBodyRotation {
    PhantomBodyRotation {
        y_head_rot: y_body_rot,
        y_body_rot: y_rot,
    }
}

pub fn phantom_move_control_surface() -> PhantomMoveControlSurface {
    PhantomMoveControlSurface {
        initial_speed: PHANTOM_MOVE_CONTROL_INITIAL_SPEED,
        horizontal_collision_yaw_flip: PHANTOM_MOVE_CONTROL_COLLISION_YAW_FLIP,
        turn_step_degrees: PHANTOM_MOVE_CONTROL_TURN_STEP_DEGREES,
        fast_speed: PHANTOM_MOVE_CONTROL_FAST_SPEED,
        slow_speed: PHANTOM_MOVE_CONTROL_SLOW_SPEED,
        fast_turn_threshold_degrees: PHANTOM_MOVE_CONTROL_FAST_TURN_THRESHOLD_DEGREES,
        fast_approach_base: PHANTOM_MOVE_CONTROL_FAST_APPROACH_BASE,
        slow_approach: PHANTOM_MOVE_CONTROL_SLOW_APPROACH,
        delta_movement_approach: PHANTOM_MOVE_CONTROL_DELTA_MOVEMENT_APPROACH,
        y_relative_scale: PHANTOM_MOVE_CONTROL_Y_RELATIVE_SCALE,
        horizontal_distance_epsilon: PHANTOM_MOVE_CONTROL_HORIZONTAL_DISTANCE_EPSILON,
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
