use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParrotVariantModel {
    RedBlue,
    Blue,
    Green,
    YellowBlue,
    Gray,
}

impl ParrotVariantModel {
    pub fn id(self) -> i32 {
        match self {
            Self::RedBlue => 0,
            Self::Blue => 1,
            Self::Green => 2,
            Self::YellowBlue => 3,
            Self::Gray => 4,
        }
    }

    pub fn serialized_name(self) -> &'static str {
        match self {
            Self::RedBlue => "red_blue",
            Self::Blue => "blue",
            Self::Green => "green",
            Self::YellowBlue => "yellow_blue",
            Self::Gray => "gray",
        }
    }
}

pub fn parrot_variant_by_id(id: i32) -> ParrotVariantModel {
    match id.clamp(0, 4) {
        0 => ParrotVariantModel::RedBlue,
        1 => ParrotVariantModel::Blue,
        2 => ParrotVariantModel::Green,
        3 => ParrotVariantModel::YellowBlue,
        _ => ParrotVariantModel::Gray,
    }
}

pub fn parrot_food_item(item: &str) -> bool {
    matches!(
        item,
        "minecraft:wheat_seeds"
            | "minecraft:melon_seeds"
            | "minecraft:pumpkin_seeds"
            | "minecraft:beetroot_seeds"
            | "minecraft:torchflower_seeds"
            | "minecraft:pitcher_pod"
    )
}

pub fn parrot_poisonous_item(item: &str) -> bool {
    item == "minecraft:cookie"
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParrotInteractResult {
    Pass,
    TameFood {
        consumed: i32,
        eat_sound: &'static str,
        tame_event: u8,
        tamed: bool,
    },
    ToggleSitting,
    Poisoned {
        consumed: i32,
        effect: &'static str,
        duration_ticks: i32,
        lethal_damage: bool,
    },
}

pub fn parrot_interact_plan(
    item: &str,
    tame: bool,
    flying: bool,
    owned_by_player: bool,
    tame_roll_zero: bool,
    invulnerable: bool,
) -> ParrotInteractResult {
    if !tame && parrot_food_item(item) {
        return ParrotInteractResult::TameFood {
            consumed: 1,
            eat_sound: "minecraft:entity.parrot.eat",
            tame_event: if tame_roll_zero { 7 } else { 6 },
            tamed: tame_roll_zero,
        };
    }
    if parrot_poisonous_item(item) {
        return ParrotInteractResult::Poisoned {
            consumed: 1,
            effect: "minecraft:poison",
            duration_ticks: PARROT_POISON_TICKS,
            lethal_damage: !invulnerable,
        };
    }
    if !flying && tame && owned_by_player {
        ParrotInteractResult::ToggleSitting
    } else {
        ParrotInteractResult::Pass
    }
}

pub fn parrot_party_state_after_ai_step(
    jukebox_present: bool,
    jukebox_is_still_jukebox: bool,
    distance_to_jukebox_center: f32,
    currently_partying: bool,
) -> bool {
    jukebox_present
        && jukebox_is_still_jukebox
        && distance_to_jukebox_center < PARROT_JUKEBOX_PARTY_DISTANCE
        && currently_partying
}

pub fn parrot_should_attempt_mimic(alive: bool, silent: bool, ai_step_roll: i32) -> bool {
    alive && !silent && ai_step_roll == 0
}

pub fn parrot_mimic_sound(entity_type: &str) -> Option<&'static str> {
    match entity_type {
        "minecraft:blaze" => Some("minecraft:entity.parrot.imitate.blaze"),
        "minecraft:bogged" => Some("minecraft:entity.parrot.imitate.bogged"),
        "minecraft:breeze" => Some("minecraft:entity.parrot.imitate.breeze"),
        "minecraft:camel_husk" => Some("minecraft:entity.parrot.imitate.camel_husk"),
        "minecraft:cave_spider" | "minecraft:spider" => {
            Some("minecraft:entity.parrot.imitate.spider")
        }
        "minecraft:creaking" => Some("minecraft:entity.parrot.imitate.creaking"),
        "minecraft:creeper" => Some("minecraft:entity.parrot.imitate.creeper"),
        "minecraft:drowned" => Some("minecraft:entity.parrot.imitate.drowned"),
        "minecraft:elder_guardian" => Some("minecraft:entity.parrot.imitate.elder_guardian"),
        "minecraft:ender_dragon" => Some("minecraft:entity.parrot.imitate.ender_dragon"),
        "minecraft:endermite" => Some("minecraft:entity.parrot.imitate.endermite"),
        "minecraft:evoker" => Some("minecraft:entity.parrot.imitate.evoker"),
        "minecraft:ghast" => Some("minecraft:entity.parrot.imitate.ghast"),
        "minecraft:happy_ghast" => Some("minecraft:empty"),
        "minecraft:guardian" => Some("minecraft:entity.parrot.imitate.guardian"),
        "minecraft:hoglin" => Some("minecraft:entity.parrot.imitate.hoglin"),
        "minecraft:husk" => Some("minecraft:entity.parrot.imitate.husk"),
        "minecraft:illusioner" => Some("minecraft:entity.parrot.imitate.illusioner"),
        "minecraft:magma_cube" => Some("minecraft:entity.parrot.imitate.magma_cube"),
        "minecraft:parched" => Some("minecraft:entity.parrot.imitate.parched"),
        "minecraft:phantom" => Some("minecraft:entity.parrot.imitate.phantom"),
        "minecraft:piglin" => Some("minecraft:entity.parrot.imitate.piglin"),
        "minecraft:piglin_brute" => Some("minecraft:entity.parrot.imitate.piglin_brute"),
        "minecraft:pillager" => Some("minecraft:entity.parrot.imitate.pillager"),
        "minecraft:ravager" => Some("minecraft:entity.parrot.imitate.ravager"),
        "minecraft:shulker" => Some("minecraft:entity.parrot.imitate.shulker"),
        "minecraft:silverfish" => Some("minecraft:entity.parrot.imitate.silverfish"),
        "minecraft:skeleton" => Some("minecraft:entity.parrot.imitate.skeleton"),
        "minecraft:slime" => Some("minecraft:entity.parrot.imitate.slime"),
        "minecraft:stray" => Some("minecraft:entity.parrot.imitate.stray"),
        "minecraft:vex" => Some("minecraft:entity.parrot.imitate.vex"),
        "minecraft:vindicator" => Some("minecraft:entity.parrot.imitate.vindicator"),
        "minecraft:warden" => Some("minecraft:entity.parrot.imitate.warden"),
        "minecraft:witch" => Some("minecraft:entity.parrot.imitate.witch"),
        "minecraft:wither" => Some("minecraft:entity.parrot.imitate.wither"),
        "minecraft:wither_skeleton" => Some("minecraft:entity.parrot.imitate.wither_skeleton"),
        "minecraft:zoglin" => Some("minecraft:entity.parrot.imitate.zoglin"),
        "minecraft:zombie" => Some("minecraft:entity.parrot.imitate.zombie"),
        "minecraft:zombie_horse" => Some("minecraft:entity.parrot.imitate.zombie_horse"),
        "minecraft:zombie_nautilus" => Some("minecraft:entity.parrot.imitate.zombie_nautilus"),
        "minecraft:zombie_villager" => Some("minecraft:entity.parrot.imitate.zombie_villager"),
        _ => None,
    }
}

pub fn parrot_mimic_nearby_plan(
    alive: bool,
    silent: bool,
    sound_roll_zero: bool,
    nearby_entity_type: Option<&str>,
    nearby_entity_silent: bool,
) -> Option<&'static str> {
    if !alive || silent || !sound_roll_zero || nearby_entity_silent {
        return None;
    }
    nearby_entity_type.and_then(parrot_mimic_sound)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParrotShoulderPlan {
    pub can_use_goal: bool,
    pub can_mount_now: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParrotShoulderInput {
    pub has_server_player_owner: bool,
    pub ordered_to_sit: bool,
    pub owner_spectator: bool,
    pub owner_flying: bool,
    pub owner_in_water: bool,
    pub owner_in_powder_snow: bool,
    pub ride_cooldown_counter: i32,
    pub in_sitting_pose: bool,
    pub leashed: bool,
    pub bounding_boxes_intersect: bool,
    pub owner_is_passenger: bool,
    pub owner_on_ground: bool,
}

pub fn parrot_shoulder_plan(input: ParrotShoulderInput) -> ParrotShoulderPlan {
    let owner_can_be_sat_on = !input.owner_spectator
        && !input.owner_flying
        && !input.owner_in_water
        && !input.owner_in_powder_snow;
    let can_use_goal = input.has_server_player_owner
        && !input.ordered_to_sit
        && owner_can_be_sat_on
        && input.ride_cooldown_counter > SHOULDER_RIDING_COOLDOWN_TICKS;
    let player_accepts_shoulder_entity = !input.owner_is_passenger
        && input.owner_on_ground
        && !input.owner_in_water
        && !input.owner_in_powder_snow;
    ParrotShoulderPlan {
        can_use_goal,
        can_mount_now: can_use_goal
            && !input.in_sitting_pose
            && !input.leashed
            && input.bounding_boxes_intersect
            && player_accepts_shoulder_entity,
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HappyGhastAttributes {
    pub max_health: f32,
    pub tempt_range: f32,
    pub flying_speed: f32,
    pub movement_speed: f32,
    pub follow_range: f32,
    pub camera_distance: f32,
}

pub fn happy_ghast_attributes() -> HappyGhastAttributes {
    HappyGhastAttributes {
        max_health: HAPPY_GHAST_MAX_HEALTH,
        tempt_range: HAPPY_GHAST_TEMPT_RANGE,
        flying_speed: HAPPY_GHAST_FLYING_SPEED,
        movement_speed: HAPPY_GHAST_MOVEMENT_SPEED,
        follow_range: HAPPY_GHAST_FOLLOW_RANGE,
        camera_distance: HAPPY_GHAST_CAMERA_DISTANCE,
    }
}

pub fn happy_ghast_food_item(item: &str) -> bool {
    item == "minecraft:snowball"
}

pub fn happy_ghast_harness_item(item: &str) -> bool {
    matches!(
        item,
        "minecraft:white_harness"
            | "minecraft:orange_harness"
            | "minecraft:magenta_harness"
            | "minecraft:light_blue_harness"
            | "minecraft:yellow_harness"
            | "minecraft:lime_harness"
            | "minecraft:pink_harness"
            | "minecraft:gray_harness"
            | "minecraft:light_gray_harness"
            | "minecraft:cyan_harness"
            | "minecraft:purple_harness"
            | "minecraft:blue_harness"
            | "minecraft:brown_harness"
            | "minecraft:green_harness"
            | "minecraft:red_harness"
            | "minecraft:black_harness"
    )
}

pub fn happy_ghast_tempt_item(item: &str, baby: bool, wearing_harness: bool) -> bool {
    if !baby && !wearing_harness {
        happy_ghast_food_item(item) || happy_ghast_harness_item(item)
    } else {
        happy_ghast_food_item(item)
    }
}

pub fn happy_ghast_can_use_body_slot(alive: bool, baby: bool) -> bool {
    alive && !baby
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HappyGhastInteractPlan {
    Pass,
    EquipHarness,
    StartRide,
}

pub fn happy_ghast_interact_plan(
    baby: bool,
    item: &str,
    wearing_harness: bool,
    secondary_use_active: bool,
    item_interaction_consumed: bool,
) -> HappyGhastInteractPlan {
    if baby {
        return HappyGhastInteractPlan::Pass;
    }
    if !item.is_empty() && item_interaction_consumed {
        return if happy_ghast_harness_item(item) && !wearing_harness {
            HappyGhastInteractPlan::EquipHarness
        } else {
            HappyGhastInteractPlan::Pass
        };
    }
    if wearing_harness && !secondary_use_active {
        HappyGhastInteractPlan::StartRide
    } else {
        HappyGhastInteractPlan::Pass
    }
}

pub fn happy_ghast_can_add_passenger(current_passengers: usize) -> bool {
    current_passengers < HAPPY_GHAST_MAX_PASSENGERS
}

pub fn happy_ghast_controlling_passenger(
    wearing_harness: bool,
    still_timeout: bool,
    first_passenger_is_player: bool,
) -> bool {
    wearing_harness && !still_timeout && first_passenger_is_player
}

pub fn happy_ghast_restriction_radius(baby: bool, wearing_harness: bool) -> i32 {
    if !baby && !wearing_harness {
        HAPPY_GHAST_LARGE_RESTRICTION_RADIUS
    } else {
        HAPPY_GHAST_SMALL_RESTRICTION_RADIUS
    }
}

pub fn happy_ghast_heal_interval_ticks(in_clouds_or_precipitation: bool) -> i32 {
    if in_clouds_or_precipitation {
        HAPPY_GHAST_FAST_HEALING_TICKS
    } else {
        HAPPY_GHAST_SLOW_HEALING_TICKS
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HappyGhastStillTimeoutStep {
    pub timeout: i32,
    pub stays_still: bool,
}

pub fn happy_ghast_still_timeout_tick(
    current_timeout: i32,
    tick_count: i32,
    player_above: bool,
) -> HappyGhastStillTimeoutStep {
    let mut timeout = current_timeout;
    if timeout > 0 && tick_count > HAPPY_GHAST_STILL_TIMEOUT_ON_LOAD_GRACE_PERIOD {
        timeout -= 1;
    }
    if player_above {
        timeout = HAPPY_GHAST_MAX_STILL_TIMEOUT;
    }
    HappyGhastStillTimeoutStep {
        timeout,
        stays_still: timeout > 0,
    }
}

pub fn happy_ghast_still_timeout_after_add_passenger(
    current_timeout: i32,
    player_above: bool,
) -> i32 {
    if !player_above {
        0
    } else if current_timeout > HAPPY_GHAST_MAX_STILL_TIMEOUT {
        HAPPY_GHAST_MAX_STILL_TIMEOUT
    } else {
        current_timeout
    }
}

pub fn happy_ghast_still_timeout_after_remove_passenger() -> i32 {
    HAPPY_GHAST_MAX_STILL_TIMEOUT
}

pub fn happy_ghast_can_be_collided_with(
    baby: bool,
    alive: bool,
    client_side: bool,
    other_is_player_above: bool,
    is_vehicle: bool,
    other_is_happy_ghast: bool,
    still_timeout: bool,
) -> bool {
    if baby || !alive {
        return false;
    }
    if (client_side && other_is_player_above) || (is_vehicle && other_is_happy_ghast) {
        true
    } else {
        still_timeout
    }
}

pub fn happy_ghast_leash_holder_offsets() -> [(f32, f32, f32); 4] {
    [
        (-0.03125, 0.4375, 0.46875),
        (0.03125, 0.4375, 0.46875),
        (-0.03125, 0.4375, -0.46875),
        (0.03125, 0.4375, -0.46875),
    ]
}

pub fn happy_ghast_notify_leash_holder_time(holder_supports_quad_leash: bool) -> i32 {
    if holder_supports_quad_leash {
        5
    } else {
        0
    }
}
