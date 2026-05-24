use super::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WitchAttributes {
    pub max_health: f32,
    pub movement_speed: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WitchRangedAttack {
    pub potion: &'static str,
    pub clear_target: bool,
    pub velocity: f32,
    pub inaccuracy: f32,
    pub throw_sound: Option<&'static str>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WitchDrinkStart {
    pub potion: &'static str,
    pub using_item: bool,
    pub speed_modifier: f32,
    pub drink_sound: Option<&'static str>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WitchDrinkFinish {
    pub using_item: bool,
    pub clear_main_hand: bool,
    pub apply_potion_effects: bool,
    pub game_event: &'static str,
    pub remove_speed_modifier: bool,
}

pub const WITCH_MAX_HEALTH: f32 = 26.0;
pub const WITCH_MOVEMENT_SPEED: f32 = 0.25;
pub const WITCH_DRINKING_SPEED_MODIFIER: f32 = -0.25;
pub const WITCH_RANGED_ATTACK_SPEED: f32 = 1.0;
pub const WITCH_RANGED_ATTACK_INTERVAL_TICKS: i32 = 60;
pub const WITCH_RANGED_ATTACK_RADIUS: f32 = 10.0;
pub const WITCH_RANDOM_STROLL_SPEED: f32 = 1.0;
pub const WITCH_LOOK_AT_PLAYER_RANGE: f32 = 8.0;
pub const WITCH_ATTACK_PLAYER_RANDOM_INTERVAL: i32 = 10;
pub const WITCH_WATER_BREATHING_CHANCE: f32 = 0.15;
pub const WITCH_FIRE_RESISTANCE_CHANCE: f32 = 0.15;
pub const WITCH_HEALING_CHANCE: f32 = 0.05;
pub const WITCH_SWIFTNESS_CHANCE: f32 = 0.5;
pub const WITCH_SWIFTNESS_DISTANCE_SQR: f32 = 121.0;
pub const WITCH_THROW_SLOWNESS_DISTANCE: f64 = 8.0;
pub const WITCH_THROW_POISON_MIN_HEALTH: f32 = 8.0;
pub const WITCH_THROW_WEAKNESS_DISTANCE: f64 = 3.0;
pub const WITCH_THROW_WEAKNESS_CHANCE: f32 = 0.25;
pub const WITCH_RAIDER_HEALING_HEALTH: f32 = 4.0;
pub const WITCH_CLOSE_THROW_VELOCITY: f32 = 0.45;
pub const WITCH_FAR_THROW_VELOCITY: f32 = 0.75;
pub const WITCH_THROW_CLOSE_DISTANCE: f64 = 2.0;
pub const WITCH_THROW_INACCURACY: f32 = 8.0;
pub const WITCH_PARTICLE_EVENT_ID: u8 = 15;
pub const WITCH_PARTICLE_CHANCE: f32 = 7.5E-4;
pub const WITCH_PARTICLE_MIN_COUNT: i32 = 10;
pub const WITCH_PARTICLE_RANDOM_BOUND: i32 = 35;
pub const WITCH_RESISTANT_DAMAGE_SCALE: f32 = 0.15;
pub const WITCH_CAN_BE_RAID_LEADER: bool = false;
pub const WITCH_RAID_BUFFS_APPLIED: bool = false;

pub fn witch_attributes() -> WitchAttributes {
    WitchAttributes {
        max_health: WITCH_MAX_HEALTH,
        movement_speed: WITCH_MOVEMENT_SPEED,
    }
}

pub fn witch_heal_raiders_goal_enabled(
    has_active_raid: bool,
    target_entity_type: &'static str,
) -> bool {
    has_active_raid && target_entity_type != "minecraft:witch"
}

pub fn witch_attack_players_enabled(heal_raiders_cooldown: i32) -> bool {
    heal_raiders_cooldown <= 0
}

pub fn witch_select_drink_potion(
    water_roll: f32,
    fire_roll: f32,
    heal_roll: f32,
    speed_roll: f32,
    eye_in_water: bool,
    has_water_breathing: bool,
    on_fire_or_fire_damage: bool,
    has_fire_resistance: bool,
    health: f32,
    max_health: f32,
    target_present: bool,
    has_speed: bool,
    target_distance_sqr: f32,
) -> Option<&'static str> {
    if water_roll < WITCH_WATER_BREATHING_CHANCE && eye_in_water && !has_water_breathing {
        Some("minecraft:water_breathing")
    } else if fire_roll < WITCH_FIRE_RESISTANCE_CHANCE
        && on_fire_or_fire_damage
        && !has_fire_resistance
    {
        Some("minecraft:fire_resistance")
    } else if heal_roll < WITCH_HEALING_CHANCE && health < max_health {
        Some("minecraft:healing")
    } else if speed_roll < WITCH_SWIFTNESS_CHANCE
        && target_present
        && !has_speed
        && target_distance_sqr > WITCH_SWIFTNESS_DISTANCE_SQR
    {
        Some("minecraft:swiftness")
    } else {
        None
    }
}

pub fn witch_start_drinking(potion: Option<&'static str>, silent: bool) -> Option<WitchDrinkStart> {
    potion.map(|potion| WitchDrinkStart {
        potion,
        using_item: true,
        speed_modifier: WITCH_DRINKING_SPEED_MODIFIER,
        drink_sound: (!silent).then_some("minecraft:entity.witch.drink"),
    })
}

pub fn witch_finish_drinking(
    item_is_potion: bool,
    potion_contents_present: bool,
) -> WitchDrinkFinish {
    WitchDrinkFinish {
        using_item: false,
        clear_main_hand: true,
        apply_potion_effects: item_is_potion && potion_contents_present,
        game_event: "minecraft:drink",
        remove_speed_modifier: true,
    }
}

pub fn witch_ranged_attack(
    drinking_potion: bool,
    target_is_raider: bool,
    target_health: f32,
    target_has_slowness: bool,
    target_has_poison: bool,
    target_has_weakness: bool,
    horizontal_distance: f64,
    weakness_roll: f32,
    silent: bool,
) -> Option<WitchRangedAttack> {
    if drinking_potion {
        return None;
    }

    let (potion, clear_target) = if target_is_raider {
        (
            if target_health <= WITCH_RAIDER_HEALING_HEALTH {
                "minecraft:healing"
            } else {
                "minecraft:regeneration"
            },
            true,
        )
    } else if horizontal_distance >= WITCH_THROW_SLOWNESS_DISTANCE && !target_has_slowness {
        ("minecraft:slowness", false)
    } else if target_health >= WITCH_THROW_POISON_MIN_HEALTH && !target_has_poison {
        ("minecraft:poison", false)
    } else if horizontal_distance <= WITCH_THROW_WEAKNESS_DISTANCE
        && !target_has_weakness
        && weakness_roll < WITCH_THROW_WEAKNESS_CHANCE
    {
        ("minecraft:weakness", false)
    } else {
        ("minecraft:harming", false)
    };

    Some(WitchRangedAttack {
        potion,
        clear_target,
        velocity: if horizontal_distance <= WITCH_THROW_CLOSE_DISTANCE {
            WITCH_CLOSE_THROW_VELOCITY
        } else {
            WITCH_FAR_THROW_VELOCITY
        },
        inaccuracy: WITCH_THROW_INACCURACY,
        throw_sound: (!silent).then_some("minecraft:entity.witch.throw"),
    })
}

pub fn witch_projectile_y_adjustment(horizontal_distance: f64) -> f64 {
    horizontal_distance * 0.2
}

pub fn witch_damage_after_magic_absorb(
    source_is_self: bool,
    witch_resistant_damage_type: bool,
    damage_after_super: f32,
) -> f32 {
    if source_is_self {
        0.0
    } else if witch_resistant_damage_type {
        damage_after_super * WITCH_RESISTANT_DAMAGE_SCALE
    } else {
        damage_after_super
    }
}

pub fn witch_particle_count(random_0_to_34: i32) -> i32 {
    random_0_to_34.rem_euclid(WITCH_PARTICLE_RANDOM_BOUND) + WITCH_PARTICLE_MIN_COUNT
}

pub fn witch_finalize_can_join_raid(spawn_reason_natural: bool) -> bool {
    !spawn_reason_natural
}

pub fn witch_ravager_rider_in_java_26_1_2() -> bool {
    false
}

