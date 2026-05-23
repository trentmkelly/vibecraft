use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AxolotlVariantModel {
    pub name: &'static str,
    pub id: i32,
    pub common_spawn: bool,
}

pub const AXOLOTL_VARIANTS: &[AxolotlVariantModel] = &[
    AxolotlVariantModel {
        name: "lucy",
        id: 0,
        common_spawn: true,
    },
    AxolotlVariantModel {
        name: "wild",
        id: 1,
        common_spawn: true,
    },
    AxolotlVariantModel {
        name: "gold",
        id: 2,
        common_spawn: true,
    },
    AxolotlVariantModel {
        name: "cyan",
        id: 3,
        common_spawn: true,
    },
    AxolotlVariantModel {
        name: "blue",
        id: 4,
        common_spawn: false,
    },
];

pub const DEFAULT_AXOLOTL_VARIANT_ID: i32 = 0;
pub const AXOLOTL_RARE_VARIANT_CHANCE: i32 = 1200;

pub fn axolotl_variant_by_id(id: i32) -> AxolotlVariantModel {
    AXOLOTL_VARIANTS
        .get(id as usize)
        .copied()
        .unwrap_or(AXOLOTL_VARIANTS[DEFAULT_AXOLOTL_VARIANT_ID as usize])
}

pub fn axolotl_variant_by_name(name: &str) -> Option<AxolotlVariantModel> {
    AXOLOTL_VARIANTS
        .iter()
        .copied()
        .find(|variant| variant.name == name)
}

pub fn axolotl_spawn_variants(common_spawn: bool) -> Vec<AxolotlVariantModel> {
    AXOLOTL_VARIANTS
        .iter()
        .copied()
        .filter(|variant| variant.common_spawn == common_spawn)
        .collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AxolotlState {
    pub playing_dead: bool,
    pub from_bucket: bool,
    pub air_supply: i32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AxolotlHurtContext {
    pub no_ai: bool,
    pub random_one_in_three: bool,
    pub random_damage_gate: i32,
    pub damage: f32,
    pub current_health: f32,
    pub max_health: f32,
    pub in_water: bool,
    pub source_entity_present: bool,
    pub direct_entity_present: bool,
}

pub const AXOLOTL_PLAY_DEAD_TICKS: i32 = 200;
pub const AXOLOTL_MAX_AIR_SUPPLY: i32 = 6000;
pub const AXOLOTL_REHYDRATE_AIR_TICKS: i32 = 1800;
pub const AXOLOTL_DRY_OUT_DAMAGE: f32 = 2.0;
pub const AXOLOTL_MAX_HEALTH: f32 = 14.0;
pub const AXOLOTL_MOVEMENT_SPEED: f32 = 1.0;
pub const AXOLOTL_ATTACK_DAMAGE: f32 = 2.0;
pub const AXOLOTL_STEP_HEIGHT: f32 = 1.0;
pub const AXOLOTL_ADULT_WIDTH: f32 = 0.75;
pub const AXOLOTL_ADULT_HEIGHT: f32 = 0.42;
pub const AXOLOTL_ADULT_EYE_HEIGHT: f32 = 0.2751;
pub const AXOLOTL_BABY_WIDTH: f32 = 0.5;
pub const AXOLOTL_BABY_HEIGHT: f32 = 0.25;
pub const AXOLOTL_BABY_EYE_HEIGHT: f32 = 0.2;
pub const AXOLOTL_TARGET_DETECTION_DISTANCE_SQUARED: f64 = 64.0;
pub const AXOLOTL_HUNTING_COOLDOWN_TICKS: i32 = 2400;
pub const AXOLOTL_PLAYER_REGEN_DETECTION_RANGE: f64 = 20.0;
pub const AXOLOTL_REGEN_BUFF_BASE_DURATION: i32 = 100;
pub const AXOLOTL_REGEN_BUFF_MAX_DURATION: i32 = 2400;

impl AxolotlState {
    pub fn new() -> Self {
        Self {
            playing_dead: false,
            from_bucket: false,
            air_supply: AXOLOTL_MAX_AIR_SUPPLY,
        }
    }

    pub fn should_play_ambient_sound(self) -> bool {
        !self.playing_dead
    }

    pub fn can_be_seen_as_enemy(self, super_can_be_seen_as_enemy: bool) -> bool {
        !self.playing_dead && super_can_be_seen_as_enemy
    }

    pub fn update_playing_dead_from_memory(&mut self, play_dead_ticks: Option<i32>, no_ai: bool) {
        if !no_ai {
            self.playing_dead = play_dead_ticks.is_some_and(|ticks| ticks > 0);
        }
    }

    pub fn rehydrate(&mut self) {
        self.air_supply =
            (self.air_supply + AXOLOTL_REHYDRATE_AIR_TICKS).min(AXOLOTL_MAX_AIR_SUPPLY);
    }
}

pub fn axolotl_dimensions(baby: bool) -> (f32, f32, f32) {
    if baby {
        (
            AXOLOTL_BABY_WIDTH,
            AXOLOTL_BABY_HEIGHT,
            AXOLOTL_BABY_EYE_HEIGHT,
        )
    } else {
        (
            AXOLOTL_ADULT_WIDTH,
            AXOLOTL_ADULT_HEIGHT,
            AXOLOTL_ADULT_EYE_HEIGHT,
        )
    }
}

pub fn axolotl_bucket_pickup_result(held_item: &str, entity_alive: bool) -> BucketPickupResult {
    bucket_pickup_result(held_item, entity_alive)
}

pub fn axolotl_requires_custom_persistence(super_requires: bool, from_bucket: bool) -> bool {
    super_requires || from_bucket
}

pub fn axolotl_remove_when_far_away(from_bucket: bool, has_custom_name: bool) -> bool {
    !from_bucket && !has_custom_name
}

pub fn axolotl_bucket_saved_keys(has_hunting_cooldown: bool) -> Vec<&'static str> {
    let mut keys = vec!["Variant", "Age", "AgeLocked"];
    if has_hunting_cooldown {
        keys.push("HuntingCooldown");
    }
    keys
}

pub fn axolotl_can_attack_target(
    target_entity_type: &str,
    target_in_water: bool,
    distance_squared: f64,
    body_has_hunting_cooldown: bool,
    sensor_attackable: bool,
) -> bool {
    let hostile = matches!(
        target_entity_type,
        "minecraft:drowned" | "minecraft:guardian" | "minecraft:elder_guardian"
    );
    let hunt_target = matches!(
        target_entity_type,
        "minecraft:tropical_fish"
            | "minecraft:pufferfish"
            | "minecraft:salmon"
            | "minecraft:cod"
            | "minecraft:squid"
            | "minecraft:glow_squid"
            | "minecraft:tadpole"
    ) && !body_has_hunting_cooldown;
    distance_squared <= AXOLOTL_TARGET_DETECTION_DISTANCE_SQUARED
        && target_in_water
        && (hostile || hunt_target)
        && sensor_attackable
}

pub fn axolotl_find_attack_target(
    breeding: bool,
    nearest_attackable: Option<&'static str>,
) -> Option<&'static str> {
    if breeding {
        None
    } else {
        nearest_attackable
    }
}

pub fn axolotl_on_stop_attacking_effects(
    target_dead_or_dying: bool,
    killed_by_player: bool,
    player_within_range: bool,
    player_regen_duration: Option<i32>,
    player_has_mining_fatigue: bool,
) -> (Option<i32>, bool) {
    if !(target_dead_or_dying && killed_by_player && player_within_range) {
        return (player_regen_duration, player_has_mining_fatigue);
    }
    let next_regen = match player_regen_duration {
        Some(duration) if duration > AXOLOTL_REGEN_BUFF_MAX_DURATION - 1 => Some(duration),
        Some(duration) => {
            Some((duration + AXOLOTL_REGEN_BUFF_BASE_DURATION).min(AXOLOTL_REGEN_BUFF_MAX_DURATION))
        }
        None => Some(AXOLOTL_REGEN_BUFF_BASE_DURATION),
    };
    (next_regen, false)
}

pub fn axolotl_play_dead_memory_on_hurt(context: AxolotlHurtContext) -> Option<i32> {
    let low_health = context.current_health / context.max_health < 0.5;
    let damaging_entity_present = context.source_entity_present || context.direct_entity_present;
    if !context.no_ai
        && context.random_one_in_three
        && ((context.random_damage_gate as f32) < context.damage || low_health)
        && context.damage < context.current_health
        && context.in_water
        && damaging_entity_present
    {
        Some(AXOLOTL_PLAY_DEAD_TICKS)
    } else {
        None
    }
}

