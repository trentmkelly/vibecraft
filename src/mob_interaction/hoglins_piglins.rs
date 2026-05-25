use super::*;

pub const PIGLIN_BRUTE_HOME_STROLL_AROUND_DISTANCE: i32 = 5;
pub const HOGLIN_MAX_HEALTH: f32 = 40.0;
pub const HOGLIN_MOVEMENT_SPEED: f32 = 0.3;
pub const HOGLIN_KNOCKBACK_RESISTANCE: f32 = 0.6;
pub const HOGLIN_ATTACK_KNOCKBACK: f32 = 1.0;
pub const HOGLIN_ATTACK_DAMAGE: f32 = 6.0;
pub const HOGLIN_BABY_ATTACK_DAMAGE: f32 = 0.5;
pub const HOGLIN_XP_REWARD: i32 = 5;
pub const HOGLIN_BABY_XP_REWARD: i32 = 3;
pub const HOGLIN_WIDTH: f32 = 1.3964844;
pub const HOGLIN_HEIGHT: f32 = 1.4;
pub const HOGLIN_PASSENGER_ATTACHMENT_Y: f32 = 1.49375;
pub const HOGLIN_CLIENT_TRACKING_RANGE: i32 = 8;
pub const HOGLIN_BABY_RANDOM_CHANCE: f32 = 0.2;
pub const HOGLIN_ATTACK_ANIMATION_DURATION_TICKS: i32 = 10;
pub const HOGLIN_ATTACK_EVENT_ID: u8 = 4;
pub const HOGLIN_ATTACK_TARGET_MEMORY_TICKS: i64 = 200;
pub const HOGLIN_ATTACK_INTERVAL_TICKS: i32 = 40;
pub const HOGLIN_BABY_ATTACK_INTERVAL_TICKS: i32 = 15;
pub const HOGLIN_REPELLENT_DETECTION_HORIZONTAL: i32 = 8;
pub const HOGLIN_REPELLENT_DETECTION_VERTICAL: i32 = 4;
pub const HOGLIN_REPELLENT_PACIFY_TIME: i32 = 200;
pub const HOGLIN_RETREAT_MIN_SECONDS: i32 = 5;
pub const HOGLIN_RETREAT_MAX_SECONDS: i32 = 20;
pub const HOGLIN_DESIRED_DISTANCE_FROM_PIGLIN_IDLING: i32 = 8;
pub const HOGLIN_DESIRED_DISTANCE_FROM_PIGLIN_RETREATING: i32 = 15;
pub const HOGLIN_AVOID_REPELLENT_SPEED: f32 = 1.0;
pub const HOGLIN_RETREAT_SPEED: f32 = 1.3;
pub const HOGLIN_BREEDING_SPEED: f32 = 0.6;
pub const HOGLIN_IDLE_SPEED: f32 = 0.4;
pub const HOGLIN_BABY_FOLLOW_ADULT_SPEED: f32 = 0.6;
pub const HOGLIN_ADULT_FOLLOW_RANGE_MIN: i32 = 5;
pub const HOGLIN_ADULT_FOLLOW_RANGE_MAX: i32 = 16;
pub const HOGLIN_LOOK_TARGET_RANGE: f32 = 8.0;
pub const HOGLIN_LOOK_INTERVAL_MIN_TICKS: i32 = 30;
pub const HOGLIN_LOOK_INTERVAL_MAX_TICKS: i32 = 60;
pub const HOGLIN_DO_NOTHING_MIN_TICKS: i32 = 30;
pub const HOGLIN_DO_NOTHING_MAX_TICKS: i32 = 60;
pub const HOGLIN_STEP_SOUND_VOLUME: f32 = 0.15;
pub const HOGLIN_STEP_SOUND_PITCH: f32 = 1.0;

pub fn zoglin_attributes() -> ZoglinAttributes {
    ZoglinAttributes {
        max_health: ZOGLIN_MAX_HEALTH,
        movement_speed: ZOGLIN_MOVEMENT_SPEED,
        knockback_resistance: ZOGLIN_KNOCKBACK_RESISTANCE,
        attack_knockback: ZOGLIN_ATTACK_KNOCKBACK,
        attack_damage: ZOGLIN_ATTACK_DAMAGE,
        xp_reward: ZOGLIN_XP_REWARD,
    }
}

pub fn zoglin_attack_damage(baby: bool) -> f32 {
    if baby {
        ZOGLIN_BABY_ATTACK_DAMAGE
    } else {
        ZOGLIN_ATTACK_DAMAGE
    }
}

pub fn zoglin_attack_interval_ticks(baby: bool) -> i32 {
    if baby {
        ZOGLIN_BABY_ATTACK_INTERVAL_TICKS
    } else {
        ZOGLIN_ATTACK_INTERVAL_TICKS
    }
}

pub fn zoglin_finalize_spawn_is_baby(random_float_0_to_1: f32) -> bool {
    random_float_0_to_1 < ZOGLIN_BABY_RANDOM_CHANCE
}

pub fn zoglin_valid_attack_target(
    target_entity_type: &'static str,
    sensor_attackable: bool,
) -> bool {
    sensor_attackable
        && target_entity_type != "minecraft:zoglin"
        && target_entity_type != "minecraft:creeper"
}

pub fn zoglin_ambient_sound(has_attack_target: bool, client_side: bool) -> Option<&'static str> {
    if client_side {
        None
    } else if has_attack_target {
        Some("minecraft:entity.zoglin.angry")
    } else {
        Some("minecraft:entity.zoglin.ambient")
    }
}

pub fn zoglin_on_hurt_should_retarget(
    was_hurt: bool,
    attacker_is_living: bool,
    can_attack_attacker: bool,
    other_target_much_further_than_current: bool,
) -> bool {
    was_hurt && attacker_is_living && can_attack_attacker && !other_target_much_further_than_current
}

pub fn zoglin_event_attack_animation_ticks(event_id: u8) -> Option<i32> {
    (event_id == ZOGLIN_ATTACK_EVENT_ID).then_some(ZOGLIN_ATTACK_ANIMATION_DURATION_TICKS)
}

pub fn zoglin_next_attack_animation_ticks(current_ticks: i32) -> i32 {
    (current_ticks - 1).max(0)
}

pub fn zoglin_blocked_by_item_throws_target(baby: bool) -> bool {
    !baby
}

pub fn zoglin_save_is_baby_key() -> &'static str {
    "IsBaby"
}

pub fn zoglin_is_immune_to_regular_zombification() -> bool {
    true
}

pub fn hoglin_is_converting(
    immune_to_zombification: bool,
    no_ai: bool,
    piglins_zombify_environment: bool,
) -> bool {
    !immune_to_zombification && !no_ai && piglins_zombify_environment
}

pub fn hoglin_conversion_tick(
    time_in_overworld: i32,
    immune_to_zombification: bool,
    no_ai: bool,
    piglins_zombify_environment: bool,
) -> HoglinConversionTick {
    if hoglin_is_converting(immune_to_zombification, no_ai, piglins_zombify_environment) {
        let next = time_in_overworld + 1;
        HoglinConversionTick {
            time_in_overworld: next,
            convert_to_zoglin: next > HOGLIN_CONVERSION_TIME_TICKS,
            nausea_ticks: if next > HOGLIN_CONVERSION_TIME_TICKS {
                HOGLIN_CONVERSION_NAUSEA_TICKS
            } else {
                0
            },
        }
    } else {
        HoglinConversionTick {
            time_in_overworld: 0,
            convert_to_zoglin: false,
            nausea_ticks: 0,
        }
    }
}

pub fn abstract_piglin_is_converting(
    immune_to_zombification: bool,
    no_ai: bool,
    piglins_zombify_environment: bool,
) -> bool {
    !immune_to_zombification && !no_ai && piglins_zombify_environment
}

pub fn abstract_piglin_conversion_tick(
    time_in_overworld: i32,
    immune_to_zombification: bool,
    no_ai: bool,
    piglins_zombify_environment: bool,
) -> AbstractPiglinConversionTick {
    if abstract_piglin_is_converting(immune_to_zombification, no_ai, piglins_zombify_environment) {
        let next = time_in_overworld + 1;
        AbstractPiglinConversionTick {
            time_in_overworld: next,
            convert_to_zombified_piglin: next > ABSTRACT_PIGLIN_CONVERSION_TIME_TICKS,
            nausea_ticks: if next > ABSTRACT_PIGLIN_CONVERSION_TIME_TICKS {
                ABSTRACT_PIGLIN_CONVERSION_NAUSEA_TICKS
            } else {
                0
            },
            keep_equipment: true,
            preserve_can_pick_up_loot: true,
        }
    } else {
        AbstractPiglinConversionTick {
            time_in_overworld: ABSTRACT_PIGLIN_DEFAULT_TIME_IN_OVERWORLD,
            convert_to_zombified_piglin: false,
            nausea_ticks: 0,
            keep_equipment: true,
            preserve_can_pick_up_loot: true,
        }
    }
}

pub fn abstract_piglin_save_defaults() -> (bool, bool, i32) {
    (
        ABSTRACT_PIGLIN_DEFAULT_IMMUNE_TO_ZOMBIFICATION,
        ABSTRACT_PIGLIN_DEFAULT_PICK_UP_LOOT,
        ABSTRACT_PIGLIN_DEFAULT_TIME_IN_OVERWORLD,
    )
}

pub fn piglin_finish_conversion_plan() -> PiglinFinishConversionPlan {
    PiglinFinishConversionPlan {
        cancel_admiring: true,
        drop_inventory: true,
        target_entity: ABSTRACT_PIGLIN_ZOMBIFIED_TARGET,
        conversion_type: ConversionTypeModel::Single,
        nausea_ticks: ABSTRACT_PIGLIN_CONVERSION_NAUSEA_TICKS,
    }
}

pub fn piglin_brute_attributes() -> PiglinBruteAttributes {
    PiglinBruteAttributes {
        max_health: PIGLIN_BRUTE_MAX_HEALTH,
        movement_speed: PIGLIN_BRUTE_MOVEMENT_SPEED,
        attack_damage: PIGLIN_BRUTE_ATTACK_DAMAGE,
        follow_range: PIGLIN_BRUTE_FOLLOW_RANGE,
        xp_reward: PIGLIN_BRUTE_XP_REWARD,
    }
}

pub fn piglin_brute_ai_constants() -> PiglinBruteAiConstants {
    PiglinBruteAiConstants {
        anger_duration_ticks: PIGLIN_BRUTE_ANGER_DURATION_TICKS,
        melee_attack_cooldown_ticks: PIGLIN_BRUTE_MELEE_ATTACK_COOLDOWN_TICKS,
        activity_sound_likelihood_per_tick: PIGLIN_BRUTE_ACTIVITY_SOUND_LIKELIHOOD_PER_TICK,
        max_look_dist: PIGLIN_BRUTE_MAX_LOOK_DIST,
        interaction_range: PIGLIN_BRUTE_INTERACTION_RANGE,
        idle_speed_multiplier: PIGLIN_BRUTE_IDLE_SPEED_MULTIPLIER,
        home_close_enough_distance: PIGLIN_BRUTE_HOME_CLOSE_ENOUGH_DISTANCE,
        home_too_far_distance: PIGLIN_BRUTE_HOME_TOO_FAR_DISTANCE,
        home_stroll_around_distance: PIGLIN_BRUTE_HOME_STROLL_AROUND_DISTANCE,
    }
}

pub fn piglin_brute_can_hunt() -> bool {
    false
}

pub fn piglin_brute_default_main_hand_item() -> &'static str {
    PIGLIN_BRUTE_DEFAULT_MAIN_HAND
}

pub fn piglin_brute_wants_to_pick_up(item: &str, super_wants_to_pick_up: bool) -> bool {
    item == PIGLIN_BRUTE_DEFAULT_MAIN_HAND && super_wants_to_pick_up
}

pub fn piglin_brute_arm_pose(aggressive: bool, holding_melee_weapon: bool) -> &'static str {
    if aggressive && holding_melee_weapon {
        "attacking_with_melee_weapon"
    } else {
        "default"
    }
}

pub fn piglin_brute_target_choice(
    angry_at_attackable: bool,
    nearest_visible_attackable_player: bool,
    nearest_visible_nemesis: bool,
) -> PiglinBruteTargetChoice {
    if angry_at_attackable {
        PiglinBruteTargetChoice::AngryAt
    } else if nearest_visible_attackable_player {
        PiglinBruteTargetChoice::NearestVisibleAttackablePlayer
    } else if nearest_visible_nemesis {
        PiglinBruteTargetChoice::NearestVisibleNemesis
    } else {
        PiglinBruteTargetChoice::None
    }
}

pub fn piglin_brute_retaliates_against(attacker_is_abstract_piglin: bool) -> bool {
    !attacker_is_abstract_piglin
}

pub fn hoglin_base_attack_damage(
    body_is_baby: bool,
    attack_damage: f32,
    random_0_to_attack_damage_minus_1: i32,
) -> f32 {
    let attack_damage_int = attack_damage as i32;
    if !body_is_baby && attack_damage_int > 0 {
        attack_damage / 2.0 + random_0_to_attack_damage_minus_1.rem_euclid(attack_damage_int) as f32
    } else {
        attack_damage
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HoglinThrowTargetInput {
    pub body_x: f64,
    pub body_z: f64,
    pub target_x: f64,
    pub target_z: f64,
    pub attack_knockback: f64,
    pub target_knockback_resistance: f64,
    pub random_y_rot_minus_10_to_10: f64,
    pub random_float_0_to_1_for_horizontal: f64,
    pub random_float_0_to_1_for_vertical: f64,
}

pub fn hoglin_base_throw_target(input: HoglinThrowTargetInput) -> Option<HoglinBaseThrowVector> {
    let effective_knockback_power = input.attack_knockback - input.target_knockback_resistance;
    if effective_knockback_power <= 0.0 {
        return None;
    }

    let dx = input.target_x - input.body_x;
    let dz = input.target_z - input.body_z;
    let length = (dx * dx + dz * dz).sqrt();
    if length == 0.0 {
        return Some(HoglinBaseThrowVector {
            x: 0.0,
            y: effective_knockback_power * input.random_float_0_to_1_for_vertical * 0.5,
            z: 0.0,
            hurt_marked: true,
        });
    }

    let horizontal_scale =
        effective_knockback_power * (input.random_float_0_to_1_for_horizontal * 0.5 + 0.2);
    let x = dx / length * horizontal_scale;
    let z = dz / length * horizontal_scale;
    let cos = input.random_y_rot_minus_10_to_10.cos();
    let sin = input.random_y_rot_minus_10_to_10.sin();
    Some(HoglinBaseThrowVector {
        x: x * cos + z * sin,
        y: effective_knockback_power * input.random_float_0_to_1_for_vertical * 0.5,
        z: z * cos - x * sin,
        hurt_marked: true,
    })
}

pub fn hoglin_attributes() -> HoglinAttributes {
    HoglinAttributes {
        max_health: HOGLIN_MAX_HEALTH,
        movement_speed: HOGLIN_MOVEMENT_SPEED,
        knockback_resistance: HOGLIN_KNOCKBACK_RESISTANCE,
        attack_knockback: HOGLIN_ATTACK_KNOCKBACK,
        attack_damage: HOGLIN_ATTACK_DAMAGE,
        xp_reward: HOGLIN_XP_REWARD,
    }
}

pub fn hoglin_entity_type_surface() -> HoglinEntityTypeSurface {
    HoglinEntityTypeSurface {
        width: HOGLIN_WIDTH,
        height: HOGLIN_HEIGHT,
        passenger_attachment_y: HOGLIN_PASSENGER_ATTACHMENT_Y,
        client_tracking_range: HOGLIN_CLIENT_TRACKING_RANGE,
    }
}

pub fn hoglin_attack_damage(baby: bool) -> f32 {
    if baby {
        HOGLIN_BABY_ATTACK_DAMAGE
    } else {
        HOGLIN_ATTACK_DAMAGE
    }
}

pub fn hoglin_xp_reward(baby: bool) -> i32 {
    if baby {
        HOGLIN_BABY_XP_REWARD
    } else {
        HOGLIN_XP_REWARD
    }
}

pub fn hoglin_attack_interval_ticks(baby: bool) -> i32 {
    if baby {
        HOGLIN_BABY_ATTACK_INTERVAL_TICKS
    } else {
        HOGLIN_ATTACK_INTERVAL_TICKS
    }
}

pub fn hoglin_finalize_spawn_is_baby(random_float_0_to_1: f32) -> bool {
    random_float_0_to_1 < HOGLIN_BABY_RANDOM_CHANCE
}

pub fn hoglin_spawn_allowed(block_below: &'static str) -> bool {
    block_below != "minecraft:nether_wart_block"
}

pub fn hoglin_walk_target_value(near_repellent: bool, block_below: &'static str) -> f32 {
    if near_repellent {
        -1.0
    } else if block_below == "minecraft:crimson_nylium" {
        10.0
    } else {
        0.0
    }
}

pub fn hoglin_can_be_hunted(adult: bool, cannot_be_hunted: bool) -> bool {
    adult && !cannot_be_hunted
}

pub fn hoglin_can_fall_in_love(pacified: bool, super_can_fall_in_love: bool) -> bool {
    !pacified && super_can_fall_in_love
}

pub fn hoglin_piglins_outnumber_hoglins(
    baby: bool,
    visible_adult_piglins: i32,
    visible_adult_hoglins: i32,
) -> bool {
    !baby && visible_adult_piglins > visible_adult_hoglins + 1
}

pub fn hoglin_on_hit_target_action(
    baby: bool,
    target_entity_type: &'static str,
    piglins_outnumber_hoglins: bool,
) -> HoglinAiAction {
    if baby {
        HoglinAiAction::None
    } else if target_entity_type == "minecraft:piglin" && piglins_outnumber_hoglins {
        HoglinAiAction::BroadcastRetreat
    } else {
        HoglinAiAction::BroadcastAttackTarget
    }
}

pub fn hoglin_was_hurt_action(
    baby: bool,
    attacker_entity_type: &'static str,
    active_activity_avoid: bool,
    other_target_much_further: bool,
    sensor_attackable: bool,
) -> HoglinAiAction {
    if baby {
        HoglinAiAction::SetAvoidTarget
    } else if (active_activity_avoid && attacker_entity_type == "minecraft:piglin")
        || attacker_entity_type == "minecraft:hoglin"
        || other_target_much_further
        || !sensor_attackable
    {
        HoglinAiAction::None
    } else {
        HoglinAiAction::SetAttackTarget
    }
}

pub fn hoglin_find_nearest_valid_attack_target(
    pacified: bool,
    breeding: bool,
    nearest_visible_attackable_player: bool,
) -> bool {
    !pacified && !breeding && nearest_visible_attackable_player
}

pub fn hoglin_activity_sound(
    activity: &'static str,
    converting: bool,
    near_repellent: bool,
    client_side: bool,
) -> Option<&'static str> {
    if client_side {
        None
    } else if activity == "avoid" || converting {
        Some("minecraft:entity.hoglin.retreat")
    } else if activity == "fight" {
        Some("minecraft:entity.hoglin.angry")
    } else if near_repellent {
        Some("minecraft:entity.hoglin.retreat")
    } else {
        Some("minecraft:entity.hoglin.ambient")
    }
}

pub fn hoglin_event_attack_animation_ticks(event_id: u8) -> Option<i32> {
    (event_id == HOGLIN_ATTACK_EVENT_ID).then_some(HOGLIN_ATTACK_ANIMATION_DURATION_TICKS)
}

pub fn hoglin_next_attack_animation_ticks(current_ticks: i32) -> i32 {
    (current_ticks - 1).max(0)
}

pub fn hoglin_blocked_by_item_throws_target(baby: bool) -> bool {
    !baby
}
