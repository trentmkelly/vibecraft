#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StriderAttributes {
    pub movement_speed: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StriderEntityTypeSurface {
    pub width: f32,
    pub height: f32,
    pub client_tracking_range: i32,
    pub fire_immune: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StriderClassSurface {
    pub blocks_building: bool,
    pub boost_time_default: i32,
    pub suffocating_default: bool,
    pub can_dispenser_equip_saddle: bool,
    pub saddle_equip_sound: &'static str,
    pub goal_priorities: &'static [(i32, &'static str)],
    pub ambient_sound: &'static str,
    pub hurt_sound: &'static str,
    pub death_sound: &'static str,
    pub happy_sound: &'static str,
    pub retreat_sound: &'static str,
    pub eat_sound: &'static str,
    pub step_sound: &'static str,
    pub lava_step_sound: &'static str,
    pub leash_offset_y_eye_height_multiplier: f32,
    pub leash_offset_z_width_multiplier: f32,
    pub ridden_input: (f64, f64, f64),
    pub ridden_pitch_multiplier: f32,
    pub passengers_inherit_malus: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StriderSuffocationState {
    pub suffocating: bool,
    pub movement_speed_modifier: Option<f32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StriderFinalizeSpawn {
    None,
    ZombifiedPiglinJockey {
        jockey_holds: &'static str,
        strider_saddle: &'static str,
        guaranteed_saddle_drop: bool,
    },
    BabyStriderJockey {
        baby_age: i32,
    },
    AgeableGroup {
        baby_chance: i32,
    },
}

pub const STRIDER_MOVEMENT_SPEED: f32 = 0.175;
pub const STRIDER_WIDTH: f32 = 0.9;
pub const STRIDER_HEIGHT: f32 = 1.7;
pub const STRIDER_CLIENT_TRACKING_RANGE: i32 = 10;
pub const STRIDER_BLOCKS_BUILDING: bool = true;
pub const STRIDER_BOOST_TIME_DEFAULT: i32 = 0;
pub const STRIDER_SUFFOCATING_DEFAULT: bool = false;
pub const STRIDER_WATER_PATHFINDING_MALUS: f32 = -1.0;
pub const STRIDER_LAVA_PATHFINDING_MALUS: f32 = 0.0;
pub const STRIDER_FIRE_PATHFINDING_MALUS: f32 = 0.0;
pub const STRIDER_SUFFOCATING_MODIFIER: f32 = -0.34;
pub const STRIDER_SUFFOCATE_STEERING_MODIFIER: f32 = 0.35;
pub const STRIDER_STEERING_MODIFIER: f32 = 0.55;
pub const STRIDER_PANIC_SPEED: f32 = 1.65;
pub const STRIDER_BREED_SPEED: f32 = 1.0;
pub const STRIDER_TEMPT_SPEED: f32 = 1.4;
pub const STRIDER_GO_TO_LAVA_SPEED: f32 = 1.0;
pub const STRIDER_GO_TO_LAVA_SEARCH_RANGE: i32 = 8;
pub const STRIDER_GO_TO_LAVA_VERTICAL_SEARCH_RANGE: i32 = 2;
pub const STRIDER_RANDOM_STROLL_SPEED: f32 = 1.0;
pub const STRIDER_RANDOM_STROLL_INTERVAL: i32 = 60;
pub const STRIDER_LOOK_RANGE: f32 = 8.0;
pub const STRIDER_HAPPY_SOUND_RANDOM_BOUND: i32 = 140;
pub const STRIDER_RETREAT_SOUND_RANDOM_BOUND: i32 = 60;
pub const STRIDER_FLOAT_LAVA_DAMPING: f64 = 0.5;
pub const STRIDER_FLOAT_LAVA_UPWARD_PUSH: f64 = 0.05;
pub const STRIDER_LIQUID_COLLISION_HEIGHT: f64 = 8.0;
pub const STRIDER_STEP_DISTANCE_INCREMENT: f32 = 0.6;
pub const STRIDER_STEP_SOUND_VOLUME: f32 = 1.0;
pub const STRIDER_STEP_SOUND_PITCH: f32 = 1.0;
pub const STRIDER_ZOMBIFIED_PIGLIN_JOCKEY_RANDOM_BOUND: i32 = 30;
pub const STRIDER_BABY_JOCKEY_RANDOM_BOUND: i32 = 10;
pub const STRIDER_BABY_JOCKEY_AGE: i32 = -24000;
pub const STRIDER_GROUP_BABY_CHANCE_PERCENT: i32 = 50;
pub const STRIDER_SADDLE_EQUIP_SOUND: &str = "minecraft:entity.strider.saddle";
pub const STRIDER_AMBIENT_SOUND: &str = "minecraft:entity.strider.ambient";
pub const STRIDER_HURT_SOUND: &str = "minecraft:entity.strider.hurt";
pub const STRIDER_DEATH_SOUND: &str = "minecraft:entity.strider.death";
pub const STRIDER_HAPPY_SOUND: &str = "minecraft:entity.strider.happy";
pub const STRIDER_RETREAT_SOUND: &str = "minecraft:entity.strider.retreat";
pub const STRIDER_EAT_SOUND: &str = "minecraft:entity.strider.eat";
pub const STRIDER_STEP_SOUND: &str = "minecraft:entity.strider.step";
pub const STRIDER_STEP_LAVA_SOUND: &str = "minecraft:entity.strider.step_lava";
pub const STRIDER_GOAL_PRIORITIES: &[(i32, &str)] = &[
    (1, "PanicGoal"),
    (2, "BreedGoal"),
    (3, "TemptGoal"),
    (4, "StriderGoToLavaGoal"),
    (5, "FollowParentGoal"),
    (7, "RandomStrollGoal"),
    (8, "LookAtPlayerGoal<Player>"),
    (8, "RandomLookAroundGoal"),
    (9, "LookAtPlayerGoal<Strider>"),
];
pub const STRIDER_LEASH_OFFSET_Y_EYE_HEIGHT_MULTIPLIER: f32 = 0.6;
pub const STRIDER_LEASH_OFFSET_Z_WIDTH_MULTIPLIER: f32 = 0.4;
pub const STRIDER_RIDDEN_INPUT: (f64, f64, f64) = (0.0, 0.0, 1.0);
pub const STRIDER_RIDDEN_PITCH_MULTIPLIER: f32 = 0.5;
pub const STRIDER_PASSENGERS_INHERIT_MALUS: bool = true;

pub fn strider_attributes() -> StriderAttributes {
    StriderAttributes {
        movement_speed: STRIDER_MOVEMENT_SPEED,
    }
}

pub fn strider_entity_type_surface() -> StriderEntityTypeSurface {
    StriderEntityTypeSurface {
        width: STRIDER_WIDTH,
        height: STRIDER_HEIGHT,
        client_tracking_range: STRIDER_CLIENT_TRACKING_RANGE,
        fire_immune: true,
    }
}

pub fn strider_class_surface() -> StriderClassSurface {
    StriderClassSurface {
        blocks_building: STRIDER_BLOCKS_BUILDING,
        boost_time_default: STRIDER_BOOST_TIME_DEFAULT,
        suffocating_default: STRIDER_SUFFOCATING_DEFAULT,
        can_dispenser_equip_saddle: true,
        saddle_equip_sound: STRIDER_SADDLE_EQUIP_SOUND,
        goal_priorities: STRIDER_GOAL_PRIORITIES,
        ambient_sound: STRIDER_AMBIENT_SOUND,
        hurt_sound: STRIDER_HURT_SOUND,
        death_sound: STRIDER_DEATH_SOUND,
        happy_sound: STRIDER_HAPPY_SOUND,
        retreat_sound: STRIDER_RETREAT_SOUND,
        eat_sound: STRIDER_EAT_SOUND,
        step_sound: STRIDER_STEP_SOUND,
        lava_step_sound: STRIDER_STEP_LAVA_SOUND,
        leash_offset_y_eye_height_multiplier: STRIDER_LEASH_OFFSET_Y_EYE_HEIGHT_MULTIPLIER,
        leash_offset_z_width_multiplier: STRIDER_LEASH_OFFSET_Z_WIDTH_MULTIPLIER,
        ridden_input: STRIDER_RIDDEN_INPUT,
        ridden_pitch_multiplier: STRIDER_RIDDEN_PITCH_MULTIPLIER,
        passengers_inherit_malus: STRIDER_PASSENGERS_INHERIT_MALUS,
    }
}

pub fn strider_can_stand_on_fluid(fluid: &'static str) -> bool {
    fluid == "minecraft:lava"
}

pub fn strider_spawn_allowed(first_non_lava_block_above_is_air: bool) -> bool {
    first_non_lava_block_above_is_air
}

pub fn strider_can_use_saddle_slot(alive: bool, baby: bool) -> bool {
    alive && !baby
}

pub fn strider_controlling_passenger(
    saddled: bool,
    first_passenger_is_player: bool,
    player_holds_warped_fungus_on_a_stick: bool,
) -> bool {
    saddled && first_passenger_is_player && player_holds_warped_fungus_on_a_stick
}

pub fn strider_ridden_speed(movement_speed: f32, suffocating: bool, boost_factor: f32) -> f32 {
    movement_speed
        * if suffocating {
            STRIDER_SUFFOCATE_STEERING_MODIFIER
        } else {
            STRIDER_STEERING_MODIFIER
        }
        * boost_factor
}

pub fn strider_suffocation_state(
    no_ai: bool,
    inside_warm_block: bool,
    on_warm_block: bool,
    lava_fluid_height_positive: bool,
    vehicle_is_warm_strider: bool,
) -> Option<StriderSuffocationState> {
    if no_ai {
        return None;
    }
    let warm =
        inside_warm_block || on_warm_block || lava_fluid_height_positive || vehicle_is_warm_strider;
    Some(StriderSuffocationState {
        suffocating: !warm,
        movement_speed_modifier: (!warm).then_some(STRIDER_SUFFOCATING_MODIFIER),
    })
}

pub fn strider_walk_target_value(target_has_lava_fluid: bool, currently_in_lava: bool) -> f32 {
    if target_has_lava_fluid {
        10.0
    } else if currently_in_lava {
        f32::NEG_INFINITY
    } else {
        0.0
    }
}

pub fn strider_can_add_passenger(already_vehicle: bool, eye_in_lava: bool) -> bool {
    !already_vehicle && !eye_in_lava
}

pub fn strider_interaction_starts_riding(
    has_food_in_hand: bool,
    saddled: bool,
    already_vehicle: bool,
    player_secondary_use_active: bool,
) -> bool {
    !has_food_in_hand && saddled && !already_vehicle && !player_secondary_use_active
}

pub fn strider_ambient_sound(panicking: bool, being_tempted: bool) -> Option<&'static str> {
    (!panicking && !being_tempted).then_some(STRIDER_AMBIENT_SOUND)
}

pub fn strider_step_sound(in_lava: bool) -> &'static str {
    if in_lava {
        STRIDER_STEP_LAVA_SOUND
    } else {
        STRIDER_STEP_SOUND
    }
}

pub fn strider_eat_sound_on_food_interaction(has_food: bool, silent: bool) -> Option<&'static str> {
    (has_food && !silent).then_some(STRIDER_EAT_SOUND)
}

pub fn strider_fall_damage_resets_in_lava(in_lava: bool) -> bool {
    in_lava
}

pub fn strider_float_in_lava(
    in_lava: bool,
    above_liquid_collision_shape: bool,
    block_above_is_lava: bool,
    current_delta: (f64, f64, f64),
) -> (bool, (f64, f64, f64)) {
    if !in_lava {
        return (false, current_delta);
    }
    if above_liquid_collision_shape && !block_above_is_lava {
        (true, current_delta)
    } else {
        (
            false,
            (
                current_delta.0 * STRIDER_FLOAT_LAVA_DAMPING,
                current_delta.1 * STRIDER_FLOAT_LAVA_DAMPING + STRIDER_FLOAT_LAVA_UPWARD_PUSH,
                current_delta.2 * STRIDER_FLOAT_LAVA_DAMPING,
            ),
        )
    }
}

pub fn strider_go_to_lava_can_use(in_lava: bool, super_can_use: bool) -> bool {
    !in_lava && super_can_use
}

pub fn strider_go_to_lava_can_continue(in_lava: bool, target_valid: bool) -> bool {
    !in_lava && target_valid
}

pub fn strider_go_to_lava_valid_target(
    block: &'static str,
    block_above_pathfindable_land: bool,
) -> bool {
    block == "minecraft:lava" && block_above_pathfindable_land
}

pub fn strider_navigation_valid_path_type(path_type: &'static str, super_valid: bool) -> bool {
    matches!(path_type, "lava" | "fire" | "fire_in_neighbor") || super_valid
}

pub fn strider_finalize_spawn(
    baby: bool,
    random_0_to_29: i32,
    random_0_to_9_after_failed_zombie_jockey_roll: i32,
) -> StriderFinalizeSpawn {
    if baby {
        return StriderFinalizeSpawn::None;
    }
    if random_0_to_29.rem_euclid(STRIDER_ZOMBIFIED_PIGLIN_JOCKEY_RANDOM_BOUND) == 0 {
        StriderFinalizeSpawn::ZombifiedPiglinJockey {
            jockey_holds: "minecraft:warped_fungus_on_a_stick",
            strider_saddle: "minecraft:saddle",
            guaranteed_saddle_drop: true,
        }
    } else if random_0_to_9_after_failed_zombie_jockey_roll
        .rem_euclid(STRIDER_BABY_JOCKEY_RANDOM_BOUND)
        == 0
    {
        StriderFinalizeSpawn::BabyStriderJockey {
            baby_age: STRIDER_BABY_JOCKEY_AGE,
        }
    } else {
        StriderFinalizeSpawn::AgeableGroup {
            baby_chance: STRIDER_GROUP_BABY_CHANCE_PERCENT,
        }
    }
}

pub fn strider_is_sensitive_to_water() -> bool {
    true
}

pub fn strider_is_on_fire() -> bool {
    false
}
