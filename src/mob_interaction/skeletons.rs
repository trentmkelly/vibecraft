#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SkeletonEntityTypeSurface {
    pub width: f32,
    pub height: f32,
    pub eye_height: f32,
    pub riding_offset: f32,
    pub client_tracking_range: i32,
    pub not_in_peaceful: bool,
    pub fire_immune: bool,
    pub immune_to_powder_snow: bool,
    pub immune_to_wither_rose: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AbstractSkeletonRangedShot {
    pub speed: f32,
    pub inaccuracy: i32,
    pub vertical_lead_multiplier: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkeletonWeaponGoal {
    Bow { min_attack_interval: i32 },
    Melee,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SkeletonFreezeTick {
    pub in_powder_snow_time: i32,
    pub conversion_time: i32,
    pub freeze_converting: bool,
    pub convert_to_stray: bool,
}

pub const ABSTRACT_SKELETON_MOVEMENT_SPEED: f32 = 0.25;
pub const ABSTRACT_SKELETON_BOW_SPEED: f32 = 1.0;
pub const ABSTRACT_SKELETON_BOW_RANGE: f32 = 15.0;
pub const ABSTRACT_SKELETON_HARD_ATTACK_INTERVAL: i32 = 20;
pub const ABSTRACT_SKELETON_NORMAL_ATTACK_INTERVAL: i32 = 40;
pub const ABSTRACT_SKELETON_INCREASED_HARD_ATTACK_INTERVAL: i32 = 50;
pub const ABSTRACT_SKELETON_INCREASED_NORMAL_ATTACK_INTERVAL: i32 = 70;
pub const ABSTRACT_SKELETON_MELEE_SPEED: f32 = 1.2;
pub const ABSTRACT_SKELETON_FLEE_SUN_SPEED: f32 = 1.0;
pub const ABSTRACT_SKELETON_WOLF_AVOID_DISTANCE: f32 = 6.0;
pub const ABSTRACT_SKELETON_LOOK_AT_PLAYER_RANGE: f32 = 8.0;
pub const ABSTRACT_SKELETON_TURTLE_TARGET_INTERVAL: i32 = 10;
pub const ABSTRACT_SKELETON_DEFAULT_MAINHAND: &str = "minecraft:bow";
pub const ABSTRACT_SKELETON_PICKUP_LOOT_MULTIPLIER: f32 = 0.55;
pub const ABSTRACT_SKELETON_HALLOWEEN_HEAD_CHANCE: f32 = 0.25;
pub const ABSTRACT_SKELETON_HALLOWEEN_JACK_O_LANTERN_CHANCE: f32 = 0.1;
pub const ABSTRACT_SKELETON_ARROW_SPEED: f32 = 1.6;
pub const ABSTRACT_SKELETON_ARROW_VERTICAL_LEAD: f32 = 0.2;
pub const ABSTRACT_SKELETON_STEP_SOUND_VOLUME: f32 = 0.15;
pub const ABSTRACT_SKELETON_STEP_SOUND_PITCH: f32 = 1.0;
pub const SKELETON_STRAY_CONVERSION_THRESHOLD: i32 = 140;
pub const SKELETON_TOTAL_STRAY_CONVERSION_TIME: i32 = 300;
pub const SKELETON_NOT_CONVERTING: i32 = -1;
pub const SKELETON_STRAY_CONVERSION_EVENT: i32 = 1048;
pub const STRAY_SLOWNESS_ARROW_TICKS: i32 = 600;
pub const WITHER_SKELETON_ATTACK_DAMAGE: f32 = 4.0;
pub const WITHER_SKELETON_WITHER_TICKS: i32 = 200;
pub const WITHER_SKELETON_ARROW_FIRE_SECONDS: f32 = 100.0;
pub const WITHER_SKELETON_LAVA_PATHFINDING_MALUS: f32 = 8.0;
pub const BOGGED_MAX_HEALTH: f32 = 16.0;
pub const BOGGED_POISON_ARROW_TICKS: i32 = 100;
pub const BOGGED_DEFAULT_SHEARED: bool = false;
pub const PARCHED_MAX_HEALTH: f32 = 16.0;
pub const PARCHED_WEAKNESS_ARROW_TICKS: i32 = 600;

pub fn skeleton_entity_type_surface(entity_type: &'static str) -> SkeletonEntityTypeSurface {
    if entity_type == "minecraft:wither_skeleton" {
        SkeletonEntityTypeSurface {
            width: 0.7,
            height: 2.4,
            eye_height: 2.1,
            riding_offset: -0.875,
            client_tracking_range: 8,
            not_in_peaceful: true,
            fire_immune: true,
            immune_to_powder_snow: false,
            immune_to_wither_rose: true,
        }
    } else {
        SkeletonEntityTypeSurface {
            width: 0.6,
            height: 1.99,
            eye_height: 1.74,
            riding_offset: -0.7,
            client_tracking_range: 8,
            not_in_peaceful: true,
            fire_immune: false,
            immune_to_powder_snow: entity_type == "minecraft:stray",
            immune_to_wither_rose: false,
        }
    }
}

pub fn abstract_skeleton_weapon_goal(
    holding_bow: bool,
    hard_difficulty: bool,
    increased_interval: bool,
) -> SkeletonWeaponGoal {
    if holding_bow {
        let min_attack_interval = match (hard_difficulty, increased_interval) {
            (true, false) => ABSTRACT_SKELETON_HARD_ATTACK_INTERVAL,
            (false, false) => ABSTRACT_SKELETON_NORMAL_ATTACK_INTERVAL,
            (true, true) => ABSTRACT_SKELETON_INCREASED_HARD_ATTACK_INTERVAL,
            (false, true) => ABSTRACT_SKELETON_INCREASED_NORMAL_ATTACK_INTERVAL,
        };
        SkeletonWeaponGoal::Bow {
            min_attack_interval,
        }
    } else {
        SkeletonWeaponGoal::Melee
    }
}

pub fn abstract_skeleton_ranged_shot(difficulty_id: i32) -> AbstractSkeletonRangedShot {
    AbstractSkeletonRangedShot {
        speed: ABSTRACT_SKELETON_ARROW_SPEED,
        inaccuracy: 14 - difficulty_id * 4,
        vertical_lead_multiplier: ABSTRACT_SKELETON_ARROW_VERTICAL_LEAD,
    }
}

pub fn abstract_skeleton_can_pick_up_loot(
    random_float_0_to_1: f32,
    special_multiplier: f32,
) -> bool {
    random_float_0_to_1 < ABSTRACT_SKELETON_PICKUP_LOOT_MULTIPLIER * special_multiplier
}

pub fn abstract_skeleton_halloween_head(
    random_head_roll: f32,
    random_pumpkin_roll: f32,
    head_empty: bool,
    halloween: bool,
) -> Option<&'static str> {
    if head_empty && halloween && random_head_roll < ABSTRACT_SKELETON_HALLOWEEN_HEAD_CHANCE {
        if random_pumpkin_roll < ABSTRACT_SKELETON_HALLOWEEN_JACK_O_LANTERN_CHANCE {
            Some("minecraft:jack_o_lantern")
        } else {
            Some("minecraft:carved_pumpkin")
        }
    } else {
        None
    }
}

pub fn skeleton_freeze_tick(
    in_powder_snow: bool,
    freeze_converting: bool,
    in_powder_snow_time: i32,
    conversion_time: i32,
    alive: bool,
    no_ai: bool,
) -> SkeletonFreezeTick {
    if !alive || no_ai {
        return SkeletonFreezeTick {
            in_powder_snow_time,
            conversion_time,
            freeze_converting,
            convert_to_stray: false,
        };
    }
    if in_powder_snow {
        if freeze_converting {
            let next_conversion = conversion_time - 1;
            SkeletonFreezeTick {
                in_powder_snow_time,
                conversion_time: next_conversion,
                freeze_converting: true,
                convert_to_stray: next_conversion < 0,
            }
        } else {
            let next_powder = in_powder_snow_time + 1;
            SkeletonFreezeTick {
                in_powder_snow_time: next_powder,
                conversion_time: if next_powder >= SKELETON_STRAY_CONVERSION_THRESHOLD {
                    SKELETON_TOTAL_STRAY_CONVERSION_TIME
                } else {
                    conversion_time
                },
                freeze_converting: next_powder >= SKELETON_STRAY_CONVERSION_THRESHOLD,
                convert_to_stray: false,
            }
        }
    } else {
        SkeletonFreezeTick {
            in_powder_snow_time: SKELETON_NOT_CONVERTING,
            conversion_time,
            freeze_converting: false,
            convert_to_stray: false,
        }
    }
}

pub fn skeleton_save_stray_conversion_time(freeze_converting: bool, conversion_time: i32) -> i32 {
    if freeze_converting {
        conversion_time
    } else {
        SKELETON_NOT_CONVERTING
    }
}

pub fn stray_spawn_allowed(
    monster_spawn_rules_pass: bool,
    spawner_reason: bool,
    sky_visible_above_powder_snow: bool,
) -> bool {
    monster_spawn_rules_pass && (spawner_reason || sky_visible_above_powder_snow)
}

pub fn skeleton_arrow_effect(entity_type: &'static str) -> Option<(&'static str, i32)> {
    match entity_type {
        "minecraft:stray" => Some(("minecraft:slowness", STRAY_SLOWNESS_ARROW_TICKS)),
        "minecraft:bogged" => Some(("minecraft:poison", BOGGED_POISON_ARROW_TICKS)),
        "minecraft:parched" => Some(("minecraft:weakness", PARCHED_WEAKNESS_ARROW_TICKS)),
        _ => None,
    }
}

pub fn wither_skeleton_melee_effect(target_is_living: bool) -> Option<(&'static str, i32)> {
    target_is_living.then_some(("minecraft:wither", WITHER_SKELETON_WITHER_TICKS))
}

pub fn wither_skeleton_can_be_affected(effect: &'static str) -> bool {
    effect != "minecraft:wither"
}

pub fn parched_can_be_affected(effect: &'static str) -> bool {
    effect != "minecraft:weakness"
}

pub fn bogged_ready_for_shearing(sheared: bool, alive: bool) -> bool {
    !sheared && alive
}

pub fn bogged_shear_sets_sheared(ready_for_shearing: bool) -> bool {
    ready_for_shearing
}
