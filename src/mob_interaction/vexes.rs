#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VexAttributes {
    pub max_health: f32,
    pub attack_damage: f32,
    pub xp_reward: i32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VexClassSurface {
    pub data_flags_default: u8,
    pub ambient_sound: &'static str,
    pub death_sound: &'static str,
    pub hurt_sound: &'static str,
    pub light_level_magic_value: f32,
    pub default_mainhand_item: &'static str,
    pub mainhand_drop_chance: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VexGoalSurface {
    pub float_goal_priority: i32,
    pub charge_attack_priority: i32,
    pub random_move_priority: i32,
    pub look_at_player_priority: i32,
    pub look_at_player_range: f32,
    pub look_at_player_probability: f32,
    pub look_at_mob_priority: i32,
    pub look_at_mob_range: f32,
    pub hurt_by_target_priority: i32,
    pub hurt_by_excluded_class: &'static str,
    pub hurt_by_alerts_others: bool,
    pub copy_owner_target_priority: i32,
    pub nearest_player_target_priority: i32,
    pub nearest_player_must_see: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VexTickOutcome {
    pub no_physics_during_tick: bool,
    pub no_gravity_after_tick: bool,
    pub limited_life_ticks: i32,
    pub starve_damage: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VexSummonPlan {
    pub can_summon: bool,
    pub count: i32,
    pub limited_life_ticks: i32,
    pub y_offset: i32,
    pub horizontal_random_bound: i32,
    pub copy_evoker_team: bool,
    pub game_event: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VexSaveState {
    pub bound_origin_present: bool,
    pub life_ticks_written: Option<i32>,
    pub owner_present: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VexVec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VexMoveControlInput {
    pub move_to_operation: bool,
    pub wanted: VexVec3,
    pub position: VexVec3,
    pub delta_movement: VexVec3,
    pub speed_modifier: f64,
    pub bounding_box_size: f64,
    pub target_position: Option<VexVec3>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VexMoveControlTick {
    pub wait_operation: bool,
    pub delta_movement: VexVec3,
    pub y_rot: Option<f32>,
    pub y_body_rot: Option<f32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VexBlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VexRandomMoveCandidate {
    pub wanted_position: Option<VexVec3>,
    pub look_at: Option<VexVec3>,
    pub speed: f32,
}

pub const VEX_MAX_HEALTH: f32 = 14.0;
pub const VEX_ATTACK_DAMAGE: f32 = 4.0;
pub const VEX_XP_REWARD: i32 = 3;
pub const VEX_FLAP_DEGREES_PER_TICK: f32 = 45.836624;
pub const VEX_TICKS_PER_FLAP: i32 = 4;
pub const VEX_CHARGING_FLAG: u8 = 1;
pub const VEX_DATA_FLAGS_DEFAULT: u8 = 0;
pub const VEX_DEFAULT_MAINHAND_ITEM: &str = "minecraft:iron_sword";
pub const VEX_MAINHAND_DROP_CHANCE: f32 = 0.0;
pub const VEX_LIGHT_LEVEL_MAGIC_VALUE: f32 = 1.0;
pub const VEX_AMBIENT_SOUND: &str = "minecraft:entity.vex.ambient";
pub const VEX_DEATH_SOUND: &str = "minecraft:entity.vex.death";
pub const VEX_HURT_SOUND: &str = "minecraft:entity.vex.hurt";
pub const VEX_CHARGE_SOUND: &str = "minecraft:entity.vex.charge";
pub const VEX_FLOAT_GOAL_PRIORITY: i32 = 0;
pub const VEX_CHARGE_ATTACK_GOAL_PRIORITY: i32 = 4;
pub const VEX_RANDOM_MOVE_GOAL_PRIORITY: i32 = 8;
pub const VEX_LOOK_AT_PLAYER_GOAL_PRIORITY: i32 = 9;
pub const VEX_LOOK_AT_PLAYER_RANGE: f32 = 3.0;
pub const VEX_LOOK_AT_PLAYER_PROBABILITY: f32 = 1.0;
pub const VEX_LOOK_AT_MOB_GOAL_PRIORITY: i32 = 10;
pub const VEX_LOOK_AT_MOB_RANGE: f32 = 8.0;
pub const VEX_HURT_BY_TARGET_GOAL_PRIORITY: i32 = 1;
pub const VEX_HURT_BY_EXCLUDED_CLASS: &str = "Raider";
pub const VEX_COPY_OWNER_TARGET_GOAL_PRIORITY: i32 = 2;
pub const VEX_NEAREST_PLAYER_TARGET_GOAL_PRIORITY: i32 = 3;
pub const VEX_CHARGE_RANDOM_BOUND: i32 = 7;
pub const VEX_CHARGE_MIN_DISTANCE_SQR: f32 = 4.0;
pub const VEX_RETARGET_DISTANCE_SQR: f32 = 9.0;
pub const VEX_RANDOM_MOVE_ATTEMPTS: i32 = 3;
pub const VEX_RANDOM_MOVE_XZ_RANDOM_BOUND: i32 = 15;
pub const VEX_RANDOM_MOVE_Y_RANDOM_BOUND: i32 = 11;
pub const VEX_RANDOM_MOVE_XZ_OFFSET: i32 = 7;
pub const VEX_RANDOM_MOVE_Y_OFFSET: i32 = 5;
pub const VEX_RANDOM_MOVE_SPEED: f32 = 0.25;
pub const VEX_MOVE_ACCELERATION: f32 = 0.05;
pub const VEX_MOVE_CLOSE_DAMPING: f32 = 0.5;
pub const VEX_OWNER_TARGET_RANGE: f32 = 16.0;
pub const EVOKER_VEX_SUMMON_COUNT: i32 = 3;
pub const EVOKER_VEX_SUMMON_CASTING_TIME: i32 = 100;
pub const EVOKER_VEX_SUMMON_INTERVAL: i32 = 340;
pub const EVOKER_VEX_LIMITED_LIFE_MIN_TICKS: i32 = 20 * 30;
pub const EVOKER_VEX_LIMITED_LIFE_RANDOM_BOUND_SECONDS: i32 = 90;

pub fn vex_attributes() -> VexAttributes {
    VexAttributes {
        max_health: VEX_MAX_HEALTH,
        attack_damage: VEX_ATTACK_DAMAGE,
        xp_reward: VEX_XP_REWARD,
    }
}

pub fn vex_class_surface() -> VexClassSurface {
    VexClassSurface {
        data_flags_default: VEX_DATA_FLAGS_DEFAULT,
        ambient_sound: VEX_AMBIENT_SOUND,
        death_sound: VEX_DEATH_SOUND,
        hurt_sound: VEX_HURT_SOUND,
        light_level_magic_value: VEX_LIGHT_LEVEL_MAGIC_VALUE,
        default_mainhand_item: VEX_DEFAULT_MAINHAND_ITEM,
        mainhand_drop_chance: VEX_MAINHAND_DROP_CHANCE,
    }
}

pub fn vex_goal_surface() -> VexGoalSurface {
    VexGoalSurface {
        float_goal_priority: VEX_FLOAT_GOAL_PRIORITY,
        charge_attack_priority: VEX_CHARGE_ATTACK_GOAL_PRIORITY,
        random_move_priority: VEX_RANDOM_MOVE_GOAL_PRIORITY,
        look_at_player_priority: VEX_LOOK_AT_PLAYER_GOAL_PRIORITY,
        look_at_player_range: VEX_LOOK_AT_PLAYER_RANGE,
        look_at_player_probability: VEX_LOOK_AT_PLAYER_PROBABILITY,
        look_at_mob_priority: VEX_LOOK_AT_MOB_GOAL_PRIORITY,
        look_at_mob_range: VEX_LOOK_AT_MOB_RANGE,
        hurt_by_target_priority: VEX_HURT_BY_TARGET_GOAL_PRIORITY,
        hurt_by_excluded_class: VEX_HURT_BY_EXCLUDED_CLASS,
        hurt_by_alerts_others: true,
        copy_owner_target_priority: VEX_COPY_OWNER_TARGET_GOAL_PRIORITY,
        nearest_player_target_priority: VEX_NEAREST_PLAYER_TARGET_GOAL_PRIORITY,
        nearest_player_must_see: true,
    }
}

pub fn vex_is_flapping(tick_count: i32) -> bool {
    tick_count.rem_euclid(VEX_TICKS_PER_FLAP) == 0
}

pub fn vex_set_charging(flags: u8, charging: bool) -> u8 {
    if charging {
        flags | VEX_CHARGING_FLAG
    } else {
        flags & !VEX_CHARGING_FLAG
    }
}

pub fn vex_is_affected_by_blocks(removed: bool) -> bool {
    !removed
}

pub fn vex_save_state(
    bound_origin_present: bool,
    has_limited_life: bool,
    limited_life_ticks: i32,
    owner_present: bool,
) -> VexSaveState {
    VexSaveState {
        bound_origin_present,
        life_ticks_written: has_limited_life.then_some(limited_life_ticks),
        owner_present,
    }
}

pub fn vex_restore_owner(old_entity_is_vex: bool, old_owner_present: bool) -> bool {
    old_entity_is_vex && old_owner_present
}

pub fn vex_is_charging(flags: u8) -> bool {
    flags & VEX_CHARGING_FLAG != 0
}

pub fn vex_tick(has_limited_life: bool, limited_life_ticks: i32) -> VexTickOutcome {
    if has_limited_life {
        let next_life = limited_life_ticks - 1;
        if next_life <= 0 {
            return VexTickOutcome {
                no_physics_during_tick: true,
                no_gravity_after_tick: true,
                limited_life_ticks: 20,
                starve_damage: true,
            };
        }
        VexTickOutcome {
            no_physics_during_tick: true,
            no_gravity_after_tick: true,
            limited_life_ticks: next_life,
            starve_damage: false,
        }
    } else {
        VexTickOutcome {
            no_physics_during_tick: true,
            no_gravity_after_tick: true,
            limited_life_ticks,
            starve_damage: false,
        }
    }
}

pub fn vex_charge_attack_can_use(
    target_present: bool,
    target_alive: bool,
    move_control_has_wanted: bool,
    random_0_to_6: i32,
    distance_sqr: f32,
) -> bool {
    target_present
        && target_alive
        && !move_control_has_wanted
        && random_0_to_6.rem_euclid(VEX_CHARGE_RANDOM_BOUND) == 0
        && distance_sqr > VEX_CHARGE_MIN_DISTANCE_SQR
}

pub fn vex_charge_attack_can_continue(
    move_control_has_wanted: bool,
    charging: bool,
    target_present: bool,
    target_alive: bool,
) -> bool {
    move_control_has_wanted && charging && target_present && target_alive
}

pub fn vex_charge_attack_tick(intersects_target: bool, distance_sqr: f32) -> (bool, bool) {
    if intersects_target {
        (true, false)
    } else if distance_sqr < VEX_RETARGET_DISTANCE_SQR {
        (false, true)
    } else {
        (false, false)
    }
}

pub fn vex_charge_attack_start(target_eye_position: Option<VexVec3>) -> (Option<VexVec3>, bool, &'static str) {
    (target_eye_position, true, VEX_CHARGE_SOUND)
}

pub fn vex_move_control_tick(input: VexMoveControlInput) -> VexMoveControlTick {
    if !input.move_to_operation {
        return VexMoveControlTick {
            wait_operation: false,
            delta_movement: input.delta_movement,
            y_rot: None,
            y_body_rot: None,
        };
    }

    let delta = VexVec3 {
        x: input.wanted.x - input.position.x,
        y: input.wanted.y - input.position.y,
        z: input.wanted.z - input.position.z,
    };
    let delta_length = vec3_length(delta);
    if delta_length < input.bounding_box_size {
        return VexMoveControlTick {
            wait_operation: true,
            delta_movement: vec3_scale(input.delta_movement, VEX_MOVE_CLOSE_DAMPING as f64),
            y_rot: None,
            y_body_rot: None,
        };
    }

    let next_delta = vec3_add(
        input.delta_movement,
        vec3_scale(delta, input.speed_modifier * 0.05 / delta_length),
    );
    let look_delta = if let Some(target) = input.target_position {
        VexVec3 {
            x: target.x - input.position.x,
            y: 0.0,
            z: target.z - input.position.z,
        }
    } else {
        next_delta
    };
    let y_rot = -look_delta.x.atan2(look_delta.z).to_degrees() as f32;

    VexMoveControlTick {
        wait_operation: false,
        delta_movement: next_delta,
        y_rot: Some(y_rot),
        y_body_rot: Some(y_rot),
    }
}

pub fn vex_copy_owner_target_can_use(
    owner_present: bool,
    owner_target_present: bool,
    can_attack_owner_target: bool,
) -> bool {
    owner_present && owner_target_present && can_attack_owner_target
}

pub fn vex_random_move_can_use(move_control_has_wanted: bool, random_0_to_6: i32) -> bool {
    !move_control_has_wanted && random_0_to_6.rem_euclid(VEX_CHARGE_RANDOM_BOUND) == 0
}

pub fn vex_random_move_candidate(
    bound_origin: Option<VexBlockPos>,
    block_position: VexBlockPos,
    random_x_0_to_14: i32,
    random_y_0_to_10: i32,
    random_z_0_to_14: i32,
    empty_block: bool,
    target_present: bool,
) -> VexRandomMoveCandidate {
    if !empty_block {
        return VexRandomMoveCandidate {
            wanted_position: None,
            look_at: None,
            speed: VEX_RANDOM_MOVE_SPEED,
        };
    }

    let origin = bound_origin.unwrap_or(block_position);
    let test_pos = VexBlockPos {
        x: origin.x + random_x_0_to_14 - VEX_RANDOM_MOVE_XZ_OFFSET,
        y: origin.y + random_y_0_to_10 - VEX_RANDOM_MOVE_Y_OFFSET,
        z: origin.z + random_z_0_to_14 - VEX_RANDOM_MOVE_XZ_OFFSET,
    };
    let center = VexVec3 {
        x: test_pos.x as f64 + 0.5,
        y: test_pos.y as f64 + 0.5,
        z: test_pos.z as f64 + 0.5,
    };
    VexRandomMoveCandidate {
        wanted_position: Some(center),
        look_at: (!target_present).then_some(center),
        speed: VEX_RANDOM_MOVE_SPEED,
    }
}

pub fn evoker_vex_summon_can_use(
    super_can_use: bool,
    nearby_vex_count: i32,
    random_1_to_8: i32,
) -> bool {
    super_can_use && random_1_to_8 > nearby_vex_count
}

pub fn evoker_vex_limited_life_ticks(random_0_to_89: i32) -> i32 {
    20 * (30 + random_0_to_89.rem_euclid(EVOKER_VEX_LIMITED_LIFE_RANDOM_BOUND_SECONDS))
}

pub fn evoker_vex_summon_plan(
    super_can_use: bool,
    nearby_vex_count: i32,
    random_1_to_8: i32,
    random_life_0_to_89: i32,
    evoker_has_team: bool,
) -> VexSummonPlan {
    VexSummonPlan {
        can_summon: evoker_vex_summon_can_use(super_can_use, nearby_vex_count, random_1_to_8),
        count: EVOKER_VEX_SUMMON_COUNT,
        limited_life_ticks: evoker_vex_limited_life_ticks(random_life_0_to_89),
        y_offset: 1,
        horizontal_random_bound: 5,
        copy_evoker_team: evoker_has_team,
        game_event: "minecraft:entity_place",
    }
}

fn vec3_add(a: VexVec3, b: VexVec3) -> VexVec3 {
    VexVec3 {
        x: a.x + b.x,
        y: a.y + b.y,
        z: a.z + b.z,
    }
}

fn vec3_scale(v: VexVec3, scale: f64) -> VexVec3 {
    VexVec3 {
        x: v.x * scale,
        y: v.y * scale,
        z: v.z * scale,
    }
}

fn vec3_length(v: VexVec3) -> f64 {
    (v.x * v.x + v.y * v.y + v.z * v.z).sqrt()
}
