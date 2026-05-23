use super::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HuskEntityTypeSurface {
    pub width: f32,
    pub height: f32,
    pub eye_height: f32,
    pub passenger_attachment_y: f32,
    pub riding_offset: f32,
    pub client_tracking_range: i32,
    pub not_in_peaceful: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HuskBabyDimensions {
    pub width: f32,
    pub height: f32,
    pub eye_height: f32,
    pub vehicle_attachment_y: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HuskFinalizeSpawnOutcome {
    pub can_pick_up_loot: Option<bool>,
    pub tried_to_spawn_camel_husk: bool,
    pub spawned_camel_husk: bool,
    pub spawned_parched_passenger: bool,
    pub equipped_iron_spear: bool,
}

pub const HUSK_WIDTH: f32 = 0.6;
pub const HUSK_HEIGHT: f32 = 1.95;
pub const HUSK_EYE_HEIGHT: f32 = 1.74;
pub const HUSK_PASSENGER_ATTACHMENT_Y: f32 = 2.075;
pub const HUSK_RIDING_OFFSET: f32 = -0.7;
pub const HUSK_CLIENT_TRACKING_RANGE: i32 = 8;
pub const HUSK_NOT_IN_PEACEFUL: bool = true;
pub const HUSK_BABY_WIDTH: f32 = 0.49;
pub const HUSK_BABY_HEIGHT: f32 = 0.99;
pub const HUSK_BABY_EYE_HEIGHT: f32 = 0.825;
pub const HUSK_BABY_VEHICLE_ATTACHMENT_Y: f32 = 0.1875;
pub const HUSK_HUNGER_EFFECT_ID: &str = "minecraft:hunger";
pub const HUSK_HUNGER_DURATION_SCALE_TICKS: i32 = 140;
pub const HUSK_HUNGER_AMPLIFIER: u8 = 0;
pub const HUSK_CONVERTS_IN_WATER: bool = true;
pub const HUSK_UNDERWATER_CONVERSION_TARGET: &str = "minecraft:zombie";
pub const HUSK_UNDERWATER_CONVERSION_LEVEL_EVENT: i32 = 1041;
pub const HUSK_LOOT_PICKUP_CHANCE_SCALE: f32 = 0.55;
pub const HUSK_CAMEL_HUSK_SPAWN_CHANCE: f32 = 0.1;
pub const HUSK_CAMEL_HUSK_RIDER_ITEM: &str = "minecraft:iron_spear";

pub fn husk_entity_type_surface() -> HuskEntityTypeSurface {
    HuskEntityTypeSurface {
        width: HUSK_WIDTH,
        height: HUSK_HEIGHT,
        eye_height: HUSK_EYE_HEIGHT,
        passenger_attachment_y: HUSK_PASSENGER_ATTACHMENT_Y,
        riding_offset: HUSK_RIDING_OFFSET,
        client_tracking_range: HUSK_CLIENT_TRACKING_RANGE,
        not_in_peaceful: HUSK_NOT_IN_PEACEFUL,
    }
}

pub fn husk_baby_dimensions() -> HuskBabyDimensions {
    HuskBabyDimensions {
        width: HUSK_BABY_WIDTH,
        height: HUSK_BABY_HEIGHT,
        eye_height: HUSK_BABY_EYE_HEIGHT,
        vehicle_attachment_y: HUSK_BABY_VEHICLE_ATTACHMENT_Y,
    }
}

pub fn husk_is_sun_sensitive() -> bool {
    false
}

pub fn husk_hunger_duration_ticks(
    effective_difficulty: f32,
    super_hurt_succeeded: bool,
    main_hand_empty: bool,
    target_is_living: bool,
) -> Option<i32> {
    (super_hurt_succeeded && main_hand_empty && target_is_living)
        .then_some(HUSK_HUNGER_DURATION_SCALE_TICKS * effective_difficulty as i32)
}

pub fn husk_should_pick_up_loot(
    spawn_reason_conversion: bool,
    random_float: f32,
    difficulty_special_multiplier: f32,
) -> Option<bool> {
    (!spawn_reason_conversion)
        .then_some(random_float < HUSK_LOOT_PICKUP_CHANCE_SCALE * difficulty_special_multiplier)
}

pub fn husk_underwater_conversion_event(silent: bool) -> Option<i32> {
    (!silent).then_some(HUSK_UNDERWATER_CONVERSION_LEVEL_EVENT)
}

pub fn husk_finalize_spawn_outcome(
    spawn_reason_natural: bool,
    spawn_reason_conversion: bool,
    random_loot_float: f32,
    difficulty_special_multiplier: f32,
    camel_husk_collision_free: bool,
    camel_husk_random_float: f32,
) -> HuskFinalizeSpawnOutcome {
    let mut tried_to_spawn_camel_husk = !spawn_reason_natural;
    let mut spawned_camel_husk = false;

    if !tried_to_spawn_camel_husk && camel_husk_collision_free {
        tried_to_spawn_camel_husk = true;
        spawned_camel_husk = camel_husk_random_float < HUSK_CAMEL_HUSK_SPAWN_CHANCE;
    }

    HuskFinalizeSpawnOutcome {
        can_pick_up_loot: husk_should_pick_up_loot(
            spawn_reason_conversion,
            random_loot_float,
            difficulty_special_multiplier,
        ),
        tried_to_spawn_camel_husk,
        spawned_camel_husk,
        spawned_parched_passenger: spawned_camel_husk,
        equipped_iron_spear: spawned_camel_husk,
    }
}

