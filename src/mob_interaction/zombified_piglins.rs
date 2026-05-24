use super::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ZombifiedPiglinAttributes {
    pub follow_range: f32,
    pub movement_speed: f32,
    pub attack_damage: f32,
    pub armor: f32,
    pub spawn_reinforcements_chance: f32,
    pub attacking_speed_modifier: f32,
    pub lava_pathfinding_malus: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ZombifiedPiglinBabyDimensions {
    pub width: f32,
    pub height: f32,
    pub eye_height: f32,
    pub vehicle_attachment_y: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ZombifiedPiglinAiStep {
    pub has_attacking_speed_modifier: bool,
    pub play_first_anger_sound_in: i32,
    pub played_first_anger_sound: bool,
    pub ticks_until_next_alert: i32,
    pub alert_others: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ZombifiedPiglinPortalSpawn {
    pub spawn: bool,
    pub spawn_pos_above_portal_floor: bool,
    pub set_entity_portal_cooldown: bool,
    pub set_vehicle_portal_cooldown: bool,
}

pub const ZOMBIFIED_PIGLIN_ATTACK_DAMAGE: f32 = 5.0;
pub const ZOMBIFIED_PIGLIN_SPAWN_REINFORCEMENTS_CHANCE: f32 = 0.0;
pub const ZOMBIFIED_PIGLIN_ATTACKING_SPEED_MODIFIER: f32 = 0.05;
pub const ZOMBIFIED_PIGLIN_LAVA_PATHFINDING_MALUS: f32 = 8.0;
pub const ZOMBIFIED_PIGLIN_BABY_WIDTH: f32 = 0.49;
pub const ZOMBIFIED_PIGLIN_BABY_HEIGHT: f32 = 0.99;
pub const ZOMBIFIED_PIGLIN_BABY_EYE_HEIGHT: f32 = 0.78;
pub const ZOMBIFIED_PIGLIN_BABY_VEHICLE_ATTACHMENT_Y: f32 = 0.1875;
pub const ZOMBIFIED_PIGLIN_FIRST_ANGER_SOUND_DELAY_MIN_TICKS: i32 = 0;
pub const ZOMBIFIED_PIGLIN_FIRST_ANGER_SOUND_DELAY_MAX_TICKS: i32 = 20;
pub const ZOMBIFIED_PIGLIN_PERSISTENT_ANGER_MIN_TICKS: i32 = 20 * 20;
pub const ZOMBIFIED_PIGLIN_PERSISTENT_ANGER_MAX_TICKS: i32 = 39 * 20;
pub const ZOMBIFIED_PIGLIN_ALERT_RANGE_Y: f32 = 10.0;
pub const ZOMBIFIED_PIGLIN_ALERT_INTERVAL_MIN_TICKS: i32 = 4 * 20;
pub const ZOMBIFIED_PIGLIN_ALERT_INTERVAL_MAX_TICKS: i32 = 6 * 20;
pub const ZOMBIFIED_PIGLIN_PORTAL_SPAWN_RANDOM_BOUND: i32 = 2_000;

pub fn zombified_piglin_attributes() -> ZombifiedPiglinAttributes {
    ZombifiedPiglinAttributes {
        follow_range: ZOMBIE_FOLLOW_RANGE,
        movement_speed: ZOMBIE_MOVEMENT_SPEED,
        attack_damage: ZOMBIFIED_PIGLIN_ATTACK_DAMAGE,
        armor: ZOMBIE_ARMOR,
        spawn_reinforcements_chance: ZOMBIFIED_PIGLIN_SPAWN_REINFORCEMENTS_CHANCE,
        attacking_speed_modifier: ZOMBIFIED_PIGLIN_ATTACKING_SPEED_MODIFIER,
        lava_pathfinding_malus: ZOMBIFIED_PIGLIN_LAVA_PATHFINDING_MALUS,
    }
}

pub fn zombified_piglin_baby_dimensions() -> ZombifiedPiglinBabyDimensions {
    ZombifiedPiglinBabyDimensions {
        width: ZOMBIFIED_PIGLIN_BABY_WIDTH,
        height: ZOMBIFIED_PIGLIN_BABY_HEIGHT,
        eye_height: ZOMBIFIED_PIGLIN_BABY_EYE_HEIGHT,
        vehicle_attachment_y: ZOMBIFIED_PIGLIN_BABY_VEHICLE_ATTACHMENT_Y,
    }
}

pub fn zombified_piglin_start_persistent_anger_time(random_ticks_20_to_39_seconds: i32) -> i32 {
    random_ticks_20_to_39_seconds.clamp(
        ZOMBIFIED_PIGLIN_PERSISTENT_ANGER_MIN_TICKS,
        ZOMBIFIED_PIGLIN_PERSISTENT_ANGER_MAX_TICKS,
    )
}

pub fn zombified_piglin_set_target_delays(
    had_target: bool,
    new_target_present: bool,
    first_anger_sound_delay: i32,
    alert_interval: i32,
) -> Option<(i32, i32)> {
    (!had_target && new_target_present).then_some((
        first_anger_sound_delay.clamp(
            ZOMBIFIED_PIGLIN_FIRST_ANGER_SOUND_DELAY_MIN_TICKS,
            ZOMBIFIED_PIGLIN_FIRST_ANGER_SOUND_DELAY_MAX_TICKS,
        ),
        alert_interval.clamp(
            ZOMBIFIED_PIGLIN_ALERT_INTERVAL_MIN_TICKS,
            ZOMBIFIED_PIGLIN_ALERT_INTERVAL_MAX_TICKS,
        ),
    ))
}

pub fn zombified_piglin_ai_step(
    angry: bool,
    baby: bool,
    has_attacking_speed_modifier: bool,
    play_first_anger_sound_in: i32,
    target_present: bool,
    ticks_until_next_alert: i32,
    has_line_of_sight_to_target: bool,
    sampled_next_alert_interval: i32,
) -> ZombifiedPiglinAiStep {
    let next_modifier = angry && (!baby || has_attacking_speed_modifier);
    let mut next_sound = play_first_anger_sound_in;
    let mut played_first_anger_sound = false;
    if angry && next_sound > 0 {
        next_sound -= 1;
        played_first_anger_sound = next_sound == 0;
    }

    let mut next_alert = ticks_until_next_alert;
    let mut alert_others = false;
    if target_present {
        if next_alert > 0 {
            next_alert -= 1;
        } else {
            alert_others = has_line_of_sight_to_target;
            next_alert = sampled_next_alert_interval.clamp(
                ZOMBIFIED_PIGLIN_ALERT_INTERVAL_MIN_TICKS,
                ZOMBIFIED_PIGLIN_ALERT_INTERVAL_MAX_TICKS,
            );
        }
    }

    ZombifiedPiglinAiStep {
        has_attacking_speed_modifier: next_modifier,
        play_first_anger_sound_in: next_sound,
        played_first_anger_sound,
        ticks_until_next_alert: next_alert,
        alert_others,
    }
}

pub fn zombified_piglin_alerts_other(
    same_entity: bool,
    other_has_target: bool,
    allied_to_target: bool,
    no_spectators_filter: bool,
) -> bool {
    !same_entity && !other_has_target && !allied_to_target && no_spectators_filter
}

pub fn zombified_piglin_spawn_allowed(
    difficulty_peaceful: bool,
    below_is_nether_wart_block: bool,
) -> bool {
    !difficulty_peaceful && !below_is_nether_wart_block
}

pub fn zombified_piglin_spawn_obstruction(unobstructed: bool, contains_liquid: bool) -> bool {
    unobstructed && !contains_liquid
}

pub fn zombified_piglin_default_main_hand_item(random_0_to_19: i32) -> &'static str {
    if random_0_to_19.rem_euclid(20) == 0 {
        "minecraft:golden_spear"
    } else {
        "minecraft:golden_sword"
    }
}

pub fn zombified_piglin_portal_spawn(
    spawning_monsters: bool,
    portal_spawns_piglins: bool,
    random_0_to_1999: i32,
    difficulty_id: i32,
    player_close_enough_for_spawning: bool,
    portal_floor_valid_spawn: bool,
    spawned_entity_has_vehicle: bool,
) -> ZombifiedPiglinPortalSpawn {
    let spawn = spawning_monsters
        && portal_spawns_piglins
        && random_0_to_1999.rem_euclid(ZOMBIFIED_PIGLIN_PORTAL_SPAWN_RANDOM_BOUND) < difficulty_id
        && player_close_enough_for_spawning
        && portal_floor_valid_spawn;
    ZombifiedPiglinPortalSpawn {
        spawn,
        spawn_pos_above_portal_floor: spawn,
        set_entity_portal_cooldown: spawn,
        set_vehicle_portal_cooldown: spawn && spawned_entity_has_vehicle,
    }
}

pub fn zombified_piglin_prevents_player_rest(angry_at_player: bool) -> bool {
    angry_at_player
}

pub fn zombified_piglin_wants_to_pick_up(can_hold_item: bool) -> bool {
    can_hold_item
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DrownedAttributes {
    pub follow_range: f32,
    pub movement_speed: f32,
    pub attack_damage: f32,
    pub armor: f32,
    pub step_height: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DrownedEntityTypeSurface {
    pub width: f32,
    pub height: f32,
    pub eye_height: f32,
    pub passenger_attachment_y: f32,
    pub riding_offset: f32,
    pub client_tracking_range: i32,
    pub not_in_peaceful: bool,
    pub amphibious_navigation: bool,
    pub water_pathfinding_malus: f32,
    pub can_spawn_in_liquids: bool,
}

