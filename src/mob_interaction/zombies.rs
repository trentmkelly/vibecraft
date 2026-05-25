
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ZombieAttributes {
    pub follow_range: f32,
    pub movement_speed: f32,
    pub attack_damage: f32,
    pub armor: f32,
    pub baby_speed_modifier: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ZombieBabyDimensions {
    pub width: f32,
    pub height: f32,
    pub eye_height: f32,
    pub vehicle_attachment_y: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ZombieFinalizeSpawnOutcome {
    pub can_pick_up_loot: Option<bool>,
    pub is_baby: bool,
    pub tried_existing_chicken_jockey: bool,
    pub spawned_new_chicken_jockey: bool,
    pub can_break_doors: bool,
    pub halloween_head: Option<&'static str>,
    pub halloween_head_drop_chance: Option<f32>,
    pub reinforcement_base_chance: f64,
    pub knockback_resistance_bonus: f64,
    pub follow_range_bonus: Option<f64>,
    pub leader_reinforcement_bonus: Option<f64>,
    pub leader_max_health_bonus: Option<f64>,
    pub reset_health_to_max: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ZombieReinforcementOutcome {
    pub attempts: i32,
    pub spawned: bool,
    pub caller_reinforcement_delta: f64,
    pub callee_reinforcement_delta: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ZombieVillagerConversionOutcome {
    NoConversion,
    PerishedNormally,
    Converted {
        preserve_villager_data: bool,
        preserve_gossips: bool,
        preserve_trade_offers: bool,
        preserve_xp: bool,
        level_event: Option<i32>,
    },
}

pub const ZOMBIE_FOLLOW_RANGE: f32 = 35.0;
pub const ZOMBIE_MOVEMENT_SPEED: f32 = 0.23;
pub const ZOMBIE_ATTACK_DAMAGE: f32 = 3.0;
pub const ZOMBIE_ARMOR: f32 = 2.0;
pub const ZOMBIE_BABY_SPEED_MODIFIER: f32 = 0.5;
pub const ZOMBIE_BABY_WIDTH: f32 = 0.49;
pub const ZOMBIE_BABY_HEIGHT: f32 = 0.99;
pub const ZOMBIE_BABY_EYE_HEIGHT: f32 = 0.775;
pub const ZOMBIE_BABY_VEHICLE_ATTACHMENT_Y: f32 = 0.1875;
pub const ZOMBIE_BABY_SPAWN_CHANCE: f32 = 0.05;
pub const ZOMBIE_BABY_XP_MULTIPLIER: f32 = 2.5;
pub const ZOMBIE_WATER_CONVERSION_START_TICKS: i32 = 600;
pub const ZOMBIE_WATER_CONVERSION_DURATION_TICKS: i32 = 300;
pub const ZOMBIE_DROWNED_CONVERSION_TARGET: &str = "minecraft:drowned";
pub const ZOMBIE_DROWNED_CONVERSION_LEVEL_EVENT: i32 = 1040;
pub const ZOMBIE_VILLAGER_CONVERSION_LEVEL_EVENT: i32 = 1026;
pub const ZOMBIE_REINFORCEMENT_ATTEMPTS: i32 = 50;
pub const ZOMBIE_REINFORCEMENT_RANGE_MIN: i32 = 7;
pub const ZOMBIE_REINFORCEMENT_RANGE_MAX: i32 = 40;
pub const ZOMBIE_REINFORCEMENT_CALLER_DELTA: f64 = -0.05;
pub const ZOMBIE_REINFORCEMENT_CALLEE_DELTA: f64 = -0.05;
pub const ZOMBIE_LOOT_PICKUP_CHANCE_SCALE: f32 = 0.55;
pub const ZOMBIE_BREAK_DOOR_CHANCE_SCALE: f32 = 0.1;
pub const ZOMBIE_CHICKEN_JOCKEY_EXISTING_CHANCE: f32 = 0.05;
pub const ZOMBIE_CHICKEN_JOCKEY_NEW_CHANCE: f32 = 0.05;
pub const ZOMBIE_EQUIPMENT_CHANCE_NORMAL: f32 = 0.01;
pub const ZOMBIE_EQUIPMENT_CHANCE_HARD: f32 = 0.05;
pub const ZOMBIE_EQUIPMENT_RANDOM_BOUND: i32 = 6;
pub const ZOMBIE_FIRE_ON_HIT_CHANCE_SCALE: f32 = 0.3;
pub const ZOMBIE_HALLOWEEN_HEAD_CHANCE: f32 = 0.25;
pub const ZOMBIE_HALLOWEEN_JACK_O_LANTERN_CHANCE: f32 = 0.1;
pub const ZOMBIE_RANDOM_REINFORCEMENT_BASE_MAX: f64 = 0.1;
pub const ZOMBIE_RANDOM_KNOCKBACK_RESISTANCE_MAX: f64 = 0.05;
pub const ZOMBIE_FOLLOW_RANGE_BONUS_SCALE: f64 = 1.5;
pub const ZOMBIE_FOLLOW_RANGE_BONUS_THRESHOLD: f64 = 1.0;
pub const ZOMBIE_LEADER_CHANCE_SCALE: f32 = 0.05;
pub const ZOMBIE_LEADER_REINFORCEMENT_BONUS_MIN: f64 = 0.5;
pub const ZOMBIE_LEADER_REINFORCEMENT_BONUS_RANGE: f64 = 0.25;
pub const ZOMBIE_LEADER_MAX_HEALTH_BONUS_MIN: f64 = 1.0;
pub const ZOMBIE_LEADER_MAX_HEALTH_BONUS_RANGE: f64 = 3.0;

pub fn zombie_attributes() -> ZombieAttributes {
    ZombieAttributes {
        follow_range: ZOMBIE_FOLLOW_RANGE,
        movement_speed: ZOMBIE_MOVEMENT_SPEED,
        attack_damage: ZOMBIE_ATTACK_DAMAGE,
        armor: ZOMBIE_ARMOR,
        baby_speed_modifier: ZOMBIE_BABY_SPEED_MODIFIER,
    }
}

pub fn zombie_baby_dimensions() -> ZombieBabyDimensions {
    ZombieBabyDimensions {
        width: ZOMBIE_BABY_WIDTH,
        height: ZOMBIE_BABY_HEIGHT,
        eye_height: ZOMBIE_BABY_EYE_HEIGHT,
        vehicle_attachment_y: ZOMBIE_BABY_VEHICLE_ATTACHMENT_Y,
    }
}

pub fn zombie_is_sun_sensitive() -> bool {
    true
}

pub fn zombie_baby_xp_reward(base_xp_reward: i32, baby: bool) -> i32 {
    if baby {
        (base_xp_reward as f32 * ZOMBIE_BABY_XP_MULTIPLIER) as i32
    } else {
        base_xp_reward
    }
}

pub fn zombie_spawn_as_baby(random_float: f32) -> bool {
    random_float < ZOMBIE_BABY_SPAWN_CHANCE
}

pub fn zombie_water_conversion_tick(
    baby: bool,
    converts_in_water: bool,
    under_water_converting: bool,
    conversion_time: i32,
    in_water_time: i32,
    eye_in_water: bool,
) -> (i32, i32, bool, Option<&'static str>) {
    if baby || !converts_in_water {
        return (in_water_time, conversion_time, under_water_converting, None);
    }
    if under_water_converting {
        let next_conversion_time = conversion_time - 1;
        return (
            in_water_time,
            next_conversion_time,
            true,
            (next_conversion_time < 0).then_some(ZOMBIE_DROWNED_CONVERSION_TARGET),
        );
    }
    if eye_in_water {
        let next_in_water_time = in_water_time + 1;
        if next_in_water_time >= ZOMBIE_WATER_CONVERSION_START_TICKS {
            (
                next_in_water_time,
                ZOMBIE_WATER_CONVERSION_DURATION_TICKS,
                true,
                None,
            )
        } else {
            (next_in_water_time, conversion_time, false, None)
        }
    } else {
        (-1, conversion_time, false, None)
    }
}

pub fn zombie_drowned_conversion_event(silent: bool) -> Option<i32> {
    (!silent).then_some(ZOMBIE_DROWNED_CONVERSION_LEVEL_EVENT)
}

pub fn zombie_fire_on_hit_seconds(
    super_hurt_succeeded: bool,
    main_hand_empty: bool,
    on_fire: bool,
    random_float: f32,
    effective_difficulty: f32,
) -> Option<i32> {
    (super_hurt_succeeded
        && main_hand_empty
        && on_fire
        && random_float < effective_difficulty * ZOMBIE_FIRE_ON_HIT_CHANCE_SCALE)
        .then_some(2 * effective_difficulty as i32)
}

pub fn zombie_reinforcement_attempt(
    super_hurt_succeeded: bool,
    target_present: bool,
    hard_difficulty: bool,
    random_float: f32,
    spawn_reinforcements_chance: f32,
    level_spawning_monsters: bool,
    valid_candidate_found: bool,
) -> ZombieReinforcementOutcome {
    let can_try = super_hurt_succeeded
        && target_present
        && hard_difficulty
        && random_float < spawn_reinforcements_chance
        && level_spawning_monsters;
    ZombieReinforcementOutcome {
        attempts: if can_try {
            ZOMBIE_REINFORCEMENT_ATTEMPTS
        } else {
            0
        },
        spawned: can_try && valid_candidate_found,
        caller_reinforcement_delta: if can_try && valid_candidate_found {
            ZOMBIE_REINFORCEMENT_CALLER_DELTA
        } else {
            0.0
        },
        callee_reinforcement_delta: if can_try && valid_candidate_found {
            ZOMBIE_REINFORCEMENT_CALLEE_DELTA
        } else {
            0.0
        },
    }
}

pub fn zombie_default_main_hand_item(
    hard_difficulty: bool,
    random_float: f32,
    random_0_to_5: i32,
) -> Option<&'static str> {
    let threshold = if hard_difficulty {
        ZOMBIE_EQUIPMENT_CHANCE_HARD
    } else {
        ZOMBIE_EQUIPMENT_CHANCE_NORMAL
    };
    if random_float >= threshold {
        return None;
    }
    match random_0_to_5.rem_euclid(ZOMBIE_EQUIPMENT_RANDOM_BOUND) {
        0 => Some("minecraft:iron_sword"),
        1 => Some("minecraft:iron_spear"),
        _ => Some("minecraft:iron_shovel"),
    }
}

pub fn zombie_can_hold_item(item: &str, baby: bool, passenger: bool) -> bool {
    !(item == "minecraft:egg" && baby && passenger)
}

pub fn zombie_wants_to_pick_up(item: &str) -> bool {
    item != "minecraft:glow_ink_sac"
}

pub fn zombie_killed_villager_outcome(
    difficulty: &str,
    killed_entity_is_villager: bool,
    normal_difficulty_skip_roll: bool,
    conversion_succeeds: bool,
    silent: bool,
) -> ZombieVillagerConversionOutcome {
    if !killed_entity_is_villager || (difficulty != "normal" && difficulty != "hard") {
        return ZombieVillagerConversionOutcome::NoConversion;
    }
    if difficulty == "normal" && normal_difficulty_skip_roll {
        return ZombieVillagerConversionOutcome::PerishedNormally;
    }
    if conversion_succeeds {
        ZombieVillagerConversionOutcome::Converted {
            preserve_villager_data: true,
            preserve_gossips: true,
            preserve_trade_offers: true,
            preserve_xp: true,
            level_event: (!silent).then_some(ZOMBIE_VILLAGER_CONVERSION_LEVEL_EVENT),
        }
    } else {
        ZombieVillagerConversionOutcome::PerishedNormally
    }
}

pub fn zombie_finalize_spawn_outcome(
    spawn_reason_conversion: bool,
    spawn_reason_load_or_dimension_travel: bool,
    group_data_present: bool,
    group_baby: bool,
    group_can_spawn_jockey: bool,
    spawn_baby_random_float: f32,
    loot_random_float: f32,
    difficulty_special_multiplier: f32,
    existing_chicken_random_float: f32,
    existing_chicken_available: bool,
    new_chicken_random_float: f32,
    door_random_float: f32,
    halloween: bool,
    head_empty: bool,
    halloween_head_random_float: f32,
    jack_o_lantern_random_float: f32,
    reinforcement_base_random_double: f64,
    knockback_random_double: f64,
    follow_range_random_double: f64,
    leader_random_float: f32,
    leader_reinforcement_random_double: f64,
    leader_health_random_double: f64,
) -> ZombieFinalizeSpawnOutcome {
    let is_baby = if group_data_present {
        group_baby
    } else {
        zombie_spawn_as_baby(spawn_baby_random_float)
    };
    let can_pick_up_loot = (!spawn_reason_conversion).then_some(
        loot_random_float < ZOMBIE_LOOT_PICKUP_CHANCE_SCALE * difficulty_special_multiplier,
    );

    let mut tried_existing_chicken_jockey = false;
    let mut spawned_new_chicken_jockey = false;
    if is_baby && group_can_spawn_jockey {
        if existing_chicken_random_float < ZOMBIE_CHICKEN_JOCKEY_EXISTING_CHANCE {
            tried_existing_chicken_jockey = true;
        } else if new_chicken_random_float < ZOMBIE_CHICKEN_JOCKEY_NEW_CHANCE {
            spawned_new_chicken_jockey = true;
        }
    }

    let mut can_break_doors =
        door_random_float < difficulty_special_multiplier * ZOMBIE_BREAK_DOOR_CHANCE_SCALE;
    let leader = leader_random_float < difficulty_special_multiplier * ZOMBIE_LEADER_CHANCE_SCALE;
    if leader {
        can_break_doors = true;
    }

    let halloween_head =
        if head_empty && halloween && halloween_head_random_float < ZOMBIE_HALLOWEEN_HEAD_CHANCE {
            Some(
                if jack_o_lantern_random_float < ZOMBIE_HALLOWEEN_JACK_O_LANTERN_CHANCE {
                    "minecraft:jack_o_lantern"
                } else {
                    "minecraft:carved_pumpkin"
                },
            )
        } else {
            None
        };

    let follow_range_bonus = follow_range_random_double
        * ZOMBIE_FOLLOW_RANGE_BONUS_SCALE
        * difficulty_special_multiplier as f64;
    let leader_reinforcement_bonus = leader.then_some(
        leader_reinforcement_random_double * ZOMBIE_LEADER_REINFORCEMENT_BONUS_RANGE
            + ZOMBIE_LEADER_REINFORCEMENT_BONUS_MIN,
    );
    let leader_max_health_bonus = leader.then_some(
        leader_health_random_double * ZOMBIE_LEADER_MAX_HEALTH_BONUS_RANGE
            + ZOMBIE_LEADER_MAX_HEALTH_BONUS_MIN,
    );

    ZombieFinalizeSpawnOutcome {
        can_pick_up_loot,
        is_baby,
        tried_existing_chicken_jockey: tried_existing_chicken_jockey && existing_chicken_available,
        spawned_new_chicken_jockey,
        can_break_doors,
        halloween_head,
        halloween_head_drop_chance: halloween_head.map(|_| 0.0),
        reinforcement_base_chance: reinforcement_base_random_double
            * ZOMBIE_RANDOM_REINFORCEMENT_BASE_MAX,
        knockback_resistance_bonus: knockback_random_double
            * ZOMBIE_RANDOM_KNOCKBACK_RESISTANCE_MAX,
        follow_range_bonus: (follow_range_bonus > ZOMBIE_FOLLOW_RANGE_BONUS_THRESHOLD)
            .then_some(follow_range_bonus),
        leader_reinforcement_bonus,
        leader_max_health_bonus,
        reset_health_to_max: leader
            && !spawn_reason_conversion
            && !spawn_reason_load_or_dimension_travel,
    }
}

