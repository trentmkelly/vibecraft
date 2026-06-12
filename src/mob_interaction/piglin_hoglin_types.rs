use super::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ZoglinAttributes {
    pub max_health: f32,
    pub movement_speed: f32,
    pub knockback_resistance: f32,
    pub attack_knockback: f32,
    pub attack_damage: f32,
    pub xp_reward: i32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HoglinBaseThrowVector {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub hurt_marked: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HoglinConversionTick {
    pub time_in_overworld: i32,
    pub convert_to_zoglin: bool,
    pub nausea_ticks: i32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AbstractPiglinConversionTick {
    pub time_in_overworld: i32,
    pub convert_to_zombified_piglin: bool,
    pub nausea_ticks: i32,
    pub keep_equipment: bool,
    pub preserve_can_pick_up_loot: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PiglinFinishConversionPlan {
    pub cancel_admiring: bool,
    pub drop_inventory: bool,
    pub target_entity: &'static str,
    pub conversion_type: ConversionTypeModel,
    pub nausea_ticks: i32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PiglinBruteAttributes {
    pub max_health: f32,
    pub movement_speed: f32,
    pub attack_damage: f32,
    pub follow_range: f32,
    pub xp_reward: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PiglinBruteTargetChoice {
    AngryAt,
    NearestVisibleAttackablePlayer,
    NearestVisibleNemesis,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PiglinBruteAiConstants {
    pub anger_duration_ticks: i64,
    pub melee_attack_cooldown_ticks: i32,
    pub activity_sound_likelihood_per_tick: f32,
    pub max_look_dist: f32,
    pub interaction_range: i32,
    pub idle_speed_multiplier: f32,
    pub home_close_enough_distance: i32,
    pub home_too_far_distance: i32,
    pub home_stroll_around_distance: i32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HoglinAttributes {
    pub max_health: f32,
    pub movement_speed: f32,
    pub knockback_resistance: f32,
    pub attack_knockback: f32,
    pub attack_damage: f32,
    pub xp_reward: i32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HoglinEntityTypeSurface {
    pub width: f32,
    pub height: f32,
    pub passenger_attachment_y: f32,
    pub client_tracking_range: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HoglinAiAction {
    SetAttackTarget,
    SetAvoidTarget,
    BroadcastAttackTarget,
    BroadcastRetreat,
    None,
}

pub const ZOGLIN_MAX_HEALTH: f32 = 40.0;
pub const ZOGLIN_MOVEMENT_SPEED: f32 = 0.3;
pub const ZOGLIN_KNOCKBACK_RESISTANCE: f32 = 0.6;
pub const ZOGLIN_ATTACK_KNOCKBACK: f32 = 1.0;
pub const ZOGLIN_ATTACK_DAMAGE: f32 = 6.0;
pub const ZOGLIN_BABY_ATTACK_DAMAGE: f32 = 0.5;
pub const ZOGLIN_XP_REWARD: i32 = 5;
pub const ZOGLIN_DEFAULT_BABY: bool = false;
pub const ZOGLIN_BABY_RANDOM_CHANCE: f32 = 0.2;
pub const ZOGLIN_ATTACK_INTERVAL_TICKS: i32 = 40;
pub const ZOGLIN_BABY_ATTACK_INTERVAL_TICKS: i32 = 15;
pub const ZOGLIN_ATTACK_TARGET_MEMORY_TICKS: i64 = 200;
pub const ZOGLIN_ATTACK_ANIMATION_DURATION_TICKS: i32 = 10;
pub const ZOGLIN_IDLE_SPEED_MULTIPLIER: f32 = 0.4;
pub const ZOGLIN_FIGHTING_MOVEMENT_SPEED: f32 = 0.3;
pub const ZOGLIN_LOOK_TARGET_RANGE: f32 = 8.0;
pub const ZOGLIN_LOOK_INTERVAL_MIN_TICKS: i32 = 30;
pub const ZOGLIN_LOOK_INTERVAL_MAX_TICKS: i32 = 60;
pub const ZOGLIN_DO_NOTHING_MIN_TICKS: i32 = 30;
pub const ZOGLIN_DO_NOTHING_MAX_TICKS: i32 = 60;
pub const ZOGLIN_CORE_ACTIVITY_PRIORITY: i32 = 0;
pub const ZOGLIN_IDLE_ACTIVITY_PRIORITY: i32 = 10;
pub const ZOGLIN_FIGHT_ACTIVITY_PRIORITY: i32 = 10;
pub const ZOGLIN_HURT_RETARGET_DISTANCE_MARGIN: f32 = 4.0;
pub const ZOGLIN_ATTACK_EVENT_ID: u8 = 4;
pub const ZOGLIN_ATTACK_SOUND: &str = "minecraft:entity.zoglin.attack";
pub const ZOGLIN_HURT_SOUND: &str = "minecraft:entity.zoglin.hurt";
pub const ZOGLIN_DEATH_SOUND: &str = "minecraft:entity.zoglin.death";
pub const ZOGLIN_STEP_SOUND: &str = "minecraft:entity.zoglin.step";
pub const ZOGLIN_STEP_SOUND_VOLUME: f32 = 0.15;
pub const ZOGLIN_STEP_SOUND_PITCH: f32 = 1.0;
pub const HOGLIN_CONVERSION_TIME_TICKS: i32 = 300;
pub const HOGLIN_CONVERSION_NAUSEA_TICKS: i32 = 200;
pub const ABSTRACT_PIGLIN_CONVERSION_TIME_TICKS: i32 = 300;
pub const ABSTRACT_PIGLIN_CONVERSION_NAUSEA_TICKS: i32 = 200;
pub const ABSTRACT_PIGLIN_DEFAULT_IMMUNE_TO_ZOMBIFICATION: bool = false;
pub const ABSTRACT_PIGLIN_DEFAULT_PICK_UP_LOOT: bool = true;
pub const ABSTRACT_PIGLIN_DEFAULT_TIME_IN_OVERWORLD: i32 = 0;
pub const ABSTRACT_PIGLIN_ZOMBIFIED_TARGET: &str = "minecraft:zombified_piglin";
pub const PIGLIN_BRUTE_MAX_HEALTH: f32 = 50.0;
pub const PIGLIN_BRUTE_MOVEMENT_SPEED: f32 = 0.35;
pub const PIGLIN_BRUTE_ATTACK_DAMAGE: f32 = 7.0;
pub const PIGLIN_BRUTE_FOLLOW_RANGE: f32 = 12.0;
pub const PIGLIN_BRUTE_XP_REWARD: i32 = 20;
pub const PIGLIN_BRUTE_DEFAULT_MAIN_HAND: &str = "minecraft:golden_axe";
pub const PIGLIN_BRUTE_ANGER_DURATION_TICKS: i64 = 600;
pub const PIGLIN_BRUTE_MELEE_ATTACK_COOLDOWN_TICKS: i32 = 20;
pub const PIGLIN_BRUTE_ACTIVITY_SOUND_LIKELIHOOD_PER_TICK: f32 = 0.0125;
pub const PIGLIN_BRUTE_MAX_LOOK_DIST: f32 = 8.0;
pub const PIGLIN_BRUTE_INTERACTION_RANGE: i32 = 8;
pub const PIGLIN_BRUTE_IDLE_SPEED_MULTIPLIER: f32 = 0.6;
pub const PIGLIN_BRUTE_HOME_CLOSE_ENOUGH_DISTANCE: i32 = 2;
pub const PIGLIN_BRUTE_HOME_TOO_FAR_DISTANCE: i32 = 100;
