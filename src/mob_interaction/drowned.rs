use super::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DrownedBabyDimensions {
    pub width: f32,
    pub height: f32,
    pub eye_height: f32,
    pub vehicle_attachment_y: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DrownedFinalizeSpawnOutcome {
    pub offhand_nautilus_shell: bool,
    pub guaranteed_offhand_drop: bool,
    pub spawned_zombie_nautilus_jockey: bool,
    pub zombie_nautilus_persistent: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DrownedTridentShot {
    pub item: &'static str,
    pub power: f32,
    pub inaccuracy: i32,
    pub y_lead_scale: f32,
    pub sound: &'static str,
}

pub const DROWNED_FOLLOW_RANGE: f32 = 35.0;
pub const DROWNED_MOVEMENT_SPEED: f32 = 0.23;
pub const DROWNED_ATTACK_DAMAGE: f32 = 3.0;
pub const DROWNED_ARMOR: f32 = 2.0;
pub const DROWNED_STEP_HEIGHT: f32 = 1.0;
pub const DROWNED_WIDTH: f32 = 0.6;
pub const DROWNED_HEIGHT: f32 = 1.95;
pub const DROWNED_EYE_HEIGHT: f32 = 1.74;
pub const DROWNED_PASSENGER_ATTACHMENT_Y: f32 = 2.0125;
pub const DROWNED_RIDING_OFFSET: f32 = -0.7;
pub const DROWNED_CLIENT_TRACKING_RANGE: i32 = 8;
pub const DROWNED_NOT_IN_PEACEFUL: bool = true;
pub const DROWNED_BABY_WIDTH: f32 = 0.49;
pub const DROWNED_BABY_HEIGHT: f32 = 0.99;
pub const DROWNED_BABY_EYE_HEIGHT: f32 = 0.775;
pub const DROWNED_BABY_VEHICLE_ATTACHMENT_Y: f32 = 0.1875;
pub const DROWNED_NAUTILUS_SHELL_CHANCE: f32 = 0.03;
pub const DROWNED_ZOMBIE_NAUTILUS_JOCKEY_CHANCE: f32 = 0.5;
pub const DROWNED_DEFAULT_WATER_PATHFINDING_MALUS: f32 = 0.0;
pub const DROWNED_TRIDENT_ATTACK_INTERVAL_TICKS: i32 = 40;
pub const DROWNED_TRIDENT_ATTACK_RADIUS: f32 = 10.0;
pub const DROWNED_TRIDENT_POWER: f32 = 1.6;
pub const DROWNED_TRIDENT_Y_LEAD_SCALE: f32 = 0.2;
pub const DROWNED_EQUIPMENT_ROLL_THRESHOLD: f32 = 0.9;
pub const DROWNED_EQUIPMENT_RANDOM_BOUND: i32 = 16;
pub const DROWNED_TRIDENT_RANDOM_CUTOFF: i32 = 10;
pub const DROWNED_MORE_FREQUENT_SPAWN_RANDOM_BOUND: i32 = 15;
pub const DROWNED_DEFAULT_SPAWN_RANDOM_BOUND: i32 = 40;
pub const DROWNED_DEEP_SPAWN_SEA_LEVEL_OFFSET: i32 = 5;
pub const DROWNED_SWIM_UP_SEA_LEVEL_OFFSET: i32 = 2;
pub const DROWNED_GO_TO_BEACH_SEA_LEVEL_OFFSET: i32 = 3;
pub const DROWNED_WATER_SEARCH_ATTEMPTS: i32 = 10;
pub const DROWNED_WATER_SEARCH_XZ_RANGE: i32 = 10;
pub const DROWNED_WATER_SEARCH_Y_UP: i32 = 2;
pub const DROWNED_WATER_SEARCH_Y_DOWN: i32 = 5;

pub fn drowned_attributes() -> DrownedAttributes {
    DrownedAttributes {
        follow_range: DROWNED_FOLLOW_RANGE,
        movement_speed: DROWNED_MOVEMENT_SPEED,
        attack_damage: DROWNED_ATTACK_DAMAGE,
        armor: DROWNED_ARMOR,
        step_height: DROWNED_STEP_HEIGHT,
    }
}

pub fn drowned_entity_type_surface() -> DrownedEntityTypeSurface {
    DrownedEntityTypeSurface {
        width: DROWNED_WIDTH,
        height: DROWNED_HEIGHT,
        eye_height: DROWNED_EYE_HEIGHT,
        passenger_attachment_y: DROWNED_PASSENGER_ATTACHMENT_Y,
        riding_offset: DROWNED_RIDING_OFFSET,
        client_tracking_range: DROWNED_CLIENT_TRACKING_RANGE,
        not_in_peaceful: DROWNED_NOT_IN_PEACEFUL,
        amphibious_navigation: true,
        water_pathfinding_malus: DROWNED_DEFAULT_WATER_PATHFINDING_MALUS,
        can_spawn_in_liquids: true,
    }
}

pub fn drowned_baby_dimensions() -> DrownedBabyDimensions {
    DrownedBabyDimensions {
        width: DROWNED_BABY_WIDTH,
        height: DROWNED_BABY_HEIGHT,
        eye_height: DROWNED_BABY_EYE_HEIGHT,
        vehicle_attachment_y: DROWNED_BABY_VEHICLE_ATTACHMENT_Y,
    }
}

pub fn drowned_spawn_allowed(
    below_is_water: bool,
    pos_is_water: bool,
    spawner_reason: bool,
    reinforcement_reason: bool,
    ignores_light_requirements: bool,
    difficulty_peaceful: bool,
    dark_enough_to_spawn: bool,
    more_frequent_drowned_biome: bool,
    random_roll: i32,
    y: i32,
    sea_level: i32,
) -> bool {
    if !below_is_water && !spawner_reason {
        return false;
    }

    let can_monster_spawn = !difficulty_peaceful
        && (ignores_light_requirements || dark_enough_to_spawn)
        && (spawner_reason || pos_is_water);
    if !can_monster_spawn {
        return false;
    }

    if spawner_reason || reinforcement_reason {
        return true;
    }

    if more_frequent_drowned_biome {
        random_roll.rem_euclid(DROWNED_MORE_FREQUENT_SPAWN_RANDOM_BOUND) == 0
    } else {
        random_roll.rem_euclid(DROWNED_DEFAULT_SPAWN_RANDOM_BOUND) == 0
            && y < sea_level - DROWNED_DEEP_SPAWN_SEA_LEVEL_OFFSET
    }
}

pub fn drowned_default_main_hand_item(
    random_float: f32,
    random_0_to_15: i32,
) -> Option<&'static str> {
    if random_float <= DROWNED_EQUIPMENT_ROLL_THRESHOLD {
        return None;
    }
    if random_0_to_15.rem_euclid(DROWNED_EQUIPMENT_RANDOM_BOUND) < DROWNED_TRIDENT_RANDOM_CUTOFF {
        Some("minecraft:trident")
    } else {
        Some("minecraft:fishing_rod")
    }
}

pub fn drowned_finalize_spawn_outcome(
    offhand_empty: bool,
    nautilus_random_float: f32,
    natural_or_structure_spawn: bool,
    structure_spawn: bool,
    main_hand_trident: bool,
    zombie_nautilus_random_float: f32,
    baby: bool,
    more_frequent_drowned_biome: bool,
) -> DrownedFinalizeSpawnOutcome {
    let offhand_nautilus_shell =
        offhand_empty && nautilus_random_float < DROWNED_NAUTILUS_SHELL_CHANCE;
    let spawned_zombie_nautilus_jockey = natural_or_structure_spawn
        && main_hand_trident
        && zombie_nautilus_random_float < DROWNED_ZOMBIE_NAUTILUS_JOCKEY_CHANCE
        && !baby
        && !more_frequent_drowned_biome;
    DrownedFinalizeSpawnOutcome {
        offhand_nautilus_shell,
        guaranteed_offhand_drop: offhand_nautilus_shell,
        spawned_zombie_nautilus_jockey,
        zombie_nautilus_persistent: spawned_zombie_nautilus_jockey && structure_spawn,
    }
}

pub fn drowned_can_replace_current_item(current_item: &str) -> bool {
    current_item != "minecraft:nautilus_shell"
}

pub fn drowned_wants_to_pick_up(item: &str) -> bool {
    item != "minecraft:trident" && item != "minecraft:iron_spear"
}

pub fn drowned_ok_target(
    target_present: bool,
    bright_outside: bool,
    target_in_water: bool,
) -> bool {
    target_present && (!bright_outside || target_in_water)
}

pub fn drowned_wants_to_swim(
    searching_for_land: bool,
    target_present: bool,
    target_in_water: bool,
) -> bool {
    searching_for_land || (target_present && target_in_water)
}

pub fn drowned_should_update_swimming(
    effective_ai: bool,
    underwater: bool,
    searching_for_land: bool,
    target_present: bool,
    target_in_water: bool,
) -> bool {
    effective_ai
        && underwater
        && drowned_wants_to_swim(searching_for_land, target_present, target_in_water)
}

pub fn drowned_trident_attack_can_use(ranged_goal_can_use: bool, main_hand_trident: bool) -> bool {
    ranged_goal_can_use && main_hand_trident
}

pub fn drowned_trident_shot(difficulty_id: i32, main_hand_trident: bool) -> DrownedTridentShot {
    let _uses_existing_trident_stack = main_hand_trident;
    DrownedTridentShot {
        item: "minecraft:trident",
        power: DROWNED_TRIDENT_POWER,
        inaccuracy: 14 - difficulty_id * 4,
        y_lead_scale: DROWNED_TRIDENT_Y_LEAD_SCALE,
        sound: "minecraft:entity.drowned.shoot",
    }
}

pub fn drowned_go_to_water_goal_can_use(
    bright_outside: bool,
    in_water: bool,
    found_water_pos: bool,
) -> bool {
    bright_outside && !in_water && found_water_pos
}

pub fn drowned_go_to_beach_goal_can_use(
    move_to_block_can_use: bool,
    bright_outside: bool,
    in_water: bool,
    y: i32,
    sea_level: i32,
) -> bool {
    move_to_block_can_use
        && !bright_outside
        && in_water
        && y >= sea_level - DROWNED_GO_TO_BEACH_SEA_LEVEL_OFFSET
}

pub fn drowned_swim_up_goal_can_use(
    bright_outside: bool,
    in_water: bool,
    y: i32,
    sea_level: i32,
) -> bool {
    !bright_outside && in_water && y < sea_level - DROWNED_SWIM_UP_SEA_LEVEL_OFFSET
}

