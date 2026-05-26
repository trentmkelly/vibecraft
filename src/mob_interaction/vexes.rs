
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VexAttributes {
    pub max_health: f32,
    pub attack_damage: f32,
    pub xp_reward: i32,
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

pub const VEX_MAX_HEALTH: f32 = 14.0;
pub const VEX_ATTACK_DAMAGE: f32 = 4.0;
pub const VEX_XP_REWARD: i32 = 3;
pub const VEX_FLAP_DEGREES_PER_TICK: f32 = 45.836624;
pub const VEX_TICKS_PER_FLAP: i32 = 4;
pub const VEX_CHARGING_FLAG: u8 = 1;
pub const VEX_DEFAULT_MAINHAND_ITEM: &str = "minecraft:iron_sword";
pub const VEX_MAINHAND_DROP_CHANCE: f32 = 0.0;
pub const VEX_LIGHT_LEVEL_MAGIC_VALUE: f32 = 1.0;
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

