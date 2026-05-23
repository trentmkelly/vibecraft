use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CatVariantModel {
    Tabby,
    Black,
    Red,
    Siamese,
    BritishShorthair,
    Calico,
    Persian,
    Ragdoll,
    White,
    Jellie,
    AllBlack,
}

impl CatVariantModel {
    pub fn serialized_name(self) -> &'static str {
        match self {
            CatVariantModel::Tabby => "tabby",
            CatVariantModel::Black => "black",
            CatVariantModel::Red => "red",
            CatVariantModel::Siamese => "siamese",
            CatVariantModel::BritishShorthair => "british_shorthair",
            CatVariantModel::Calico => "calico",
            CatVariantModel::Persian => "persian",
            CatVariantModel::Ragdoll => "ragdoll",
            CatVariantModel::White => "white",
            CatVariantModel::Jellie => "jellie",
            CatVariantModel::AllBlack => "all_black",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FelineAttributes {
    pub max_health: f32,
    pub movement_speed: f32,
    pub attack_damage: f32,
}

pub fn feline_attributes() -> FelineAttributes {
    FelineAttributes {
        max_health: FELINE_MAX_HEALTH,
        movement_speed: FELINE_MOVEMENT_SPEED,
        attack_damage: FELINE_ATTACK_DAMAGE,
    }
}

pub fn cat_default_variant() -> CatVariantModel {
    CatVariantModel::Black
}

pub fn cat_spawn_variant(
    in_black_cat_structure: bool,
    moon_brightness_at_least_point_nine: bool,
    fallback: CatVariantModel,
) -> CatVariantModel {
    if in_black_cat_structure || moon_brightness_at_least_point_nine {
        CatVariantModel::AllBlack
    } else {
        fallback
    }
}

pub fn cat_food_item(item: &str) -> bool {
    matches!(item, "minecraft:cod" | "minecraft:salmon")
}

pub fn ocelot_food_item(item: &str) -> bool {
    matches!(item, "minecraft:cod" | "minecraft:salmon")
}

pub fn cat_collar_dye_color_id(item: &str) -> Option<i32> {
    match item {
        "minecraft:white_dye" => Some(0),
        "minecraft:orange_dye" => Some(1),
        "minecraft:magenta_dye" => Some(2),
        "minecraft:light_blue_dye" => Some(3),
        "minecraft:yellow_dye" => Some(4),
        "minecraft:lime_dye" => Some(5),
        "minecraft:pink_dye" => Some(6),
        "minecraft:gray_dye" => Some(7),
        "minecraft:light_gray_dye" => Some(8),
        "minecraft:cyan_dye" => Some(9),
        "minecraft:purple_dye" => Some(10),
        "minecraft:blue_dye" => Some(11),
        "minecraft:brown_dye" => Some(12),
        "minecraft:green_dye" => Some(13),
        "minecraft:red_dye" => Some(14),
        "minecraft:black_dye" => Some(15),
        _ => None,
    }
}

pub fn cat_default_collar_color_id() -> i32 {
    14
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FelineMovePose {
    Standing,
    Crouching,
}

pub fn feline_pose_for_move(has_wanted_move: bool, speed_modifier: f32) -> (FelineMovePose, bool) {
    if has_wanted_move && (speed_modifier - FELINE_CROUCH_SPEED_MOD).abs() < f32::EPSILON {
        (FelineMovePose::Crouching, false)
    } else if has_wanted_move && (speed_modifier - FELINE_SPRINT_SPEED_MOD).abs() < f32::EPSILON {
        (FelineMovePose::Standing, true)
    } else {
        (FelineMovePose::Standing, false)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CatInteractPlan {
    DyeCollar {
        color_id: i32,
        consumed: i32,
        persist: bool,
    },
    Heal {
        consumed: i32,
        heal_min: i32,
        heal_max: i32,
    },
    ToggleSit,
    TameFood {
        consumed: i32,
        tame_event: u8,
        tamed: bool,
        ordered_to_sit: bool,
        persist: bool,
        eat_sound: &'static str,
    },
    Delegate,
}

pub fn cat_interact_plan(
    item: &str,
    tame: bool,
    owned_by_player: bool,
    current_collar_color_id: i32,
    health_below_max: bool,
    parent_interaction_consumes_action: bool,
    tame_roll_zero: bool,
) -> CatInteractPlan {
    if tame && owned_by_player {
        if let Some(color_id) = cat_collar_dye_color_id(item) {
            if color_id != current_collar_color_id {
                return CatInteractPlan::DyeCollar {
                    color_id,
                    consumed: 1,
                    persist: true,
                };
            }
        } else if cat_food_item(item) && health_below_max {
            return CatInteractPlan::Heal {
                consumed: 1,
                heal_min: 1,
                heal_max: 1,
            };
        }

        if !parent_interaction_consumes_action {
            return CatInteractPlan::ToggleSit;
        }
        return CatInteractPlan::Delegate;
    }

    if !tame && cat_food_item(item) {
        return CatInteractPlan::TameFood {
            consumed: 1,
            tame_event: if tame_roll_zero { 7 } else { 6 },
            tamed: tame_roll_zero,
            ordered_to_sit: tame_roll_zero,
            persist: true,
            eat_sound: "minecraft:entity.cat.eat",
        };
    }

    CatInteractPlan::Delegate
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OcelotInteractPlan {
    TrustFood {
        consumed: i32,
        event: u8,
        trusting: bool,
    },
    Delegate,
}

pub fn ocelot_interact_plan(
    item: &str,
    tempt_goal_running_or_absent: bool,
    trusting: bool,
    player_distance_sqr: f32,
    trust_roll_zero: bool,
) -> OcelotInteractPlan {
    if tempt_goal_running_or_absent
        && !trusting
        && ocelot_food_item(item)
        && player_distance_sqr < 9.0
    {
        OcelotInteractPlan::TrustFood {
            consumed: 1,
            event: if trust_roll_zero { 41 } else { 40 },
            trusting: trust_roll_zero,
        }
    } else {
        OcelotInteractPlan::Delegate
    }
}

pub fn feline_should_avoid_player(
    tame_or_trusting: bool,
    player_creative_or_spectator: bool,
) -> bool {
    !tame_or_trusting && !player_creative_or_spectator
}

pub fn cat_tempt_can_use(super_can_use: bool, tame: bool) -> bool {
    super_can_use && !tame
}

pub fn cat_tempt_can_scare(selected_player_matches_current: bool, super_can_scare: bool) -> bool {
    !selected_player_matches_current && super_can_scare
}

pub fn cat_should_play_beg_sound(tempt_running: bool, tame: bool, tick_count: i32) -> bool {
    tempt_running && !tame && tick_count % CAT_BEG_SOUND_INTERVAL_TICKS == 0
}

pub fn cat_can_mate(
    tame: bool,
    partner_is_cat: bool,
    partner_tame: bool,
    super_can_mate: bool,
) -> bool {
    tame && partner_is_cat && partner_tame && super_can_mate
}

pub fn cat_remove_when_far_away(tame: bool, tick_count: i32) -> bool {
    !tame && tick_count > FELINE_REMOVE_WHEN_FAR_TICKS
}

pub fn ocelot_remove_when_far_away(trusting: bool, tick_count: i32) -> bool {
    !trusting && tick_count > FELINE_REMOVE_WHEN_FAR_TICKS
}

pub fn cat_relax_on_owner_can_use(
    tame: bool,
    ordered_to_sit: bool,
    owner_is_player: bool,
    owner_sleeping: bool,
    distance_to_owner_sqr: f32,
    owner_on_bed: bool,
    space_occupied_by_relaxing_cat: bool,
) -> bool {
    tame && !ordered_to_sit
        && owner_is_player
        && owner_sleeping
        && distance_to_owner_sqr <= CAT_OWNER_RELAX_DISTANCE_SQR
        && owner_on_bed
        && !space_occupied_by_relaxing_cat
}

pub fn cat_relax_on_owner_tick(distance_to_owner_sqr: f32, on_bed_ticks: i32) -> (bool, bool) {
    if distance_to_owner_sqr < CAT_LIE_ON_OWNER_DISTANCE_SQR {
        if on_bed_ticks > CAT_ON_BED_RELAX_TICKS {
            (true, false)
        } else {
            (false, true)
        }
    } else {
        (false, false)
    }
}

pub fn cat_morning_gift_plan(
    owner_sleep_timer: i32,
    gift_chance_roll_passed: bool,
    leashed: bool,
) -> Option<(&'static str, bool)> {
    (owner_sleep_timer >= 100 && gift_chance_roll_passed)
        .then_some(("minecraft:gameplay/cat_morning_gift", leashed))
}

pub fn cat_lie_on_bed_can_use(
    tame: bool,
    ordered_to_sit: bool,
    lying: bool,
    super_can_use: bool,
) -> bool {
    tame && !ordered_to_sit && !lying && super_can_use
}

pub fn cat_sit_on_block_can_use(tame: bool, ordered_to_sit: bool, super_can_use: bool) -> bool {
    tame && !ordered_to_sit && super_can_use
}

pub fn cat_sit_on_block_target_valid(
    block: &str,
    above_empty: bool,
    chest_open_count: i32,
    furnace_lit: bool,
    bed_is_head: bool,
) -> bool {
    above_empty
        && match block {
            "minecraft:chest" => chest_open_count < 1,
            "minecraft:furnace" => furnace_lit,
            "minecraft:white_bed"
            | "minecraft:orange_bed"
            | "minecraft:magenta_bed"
            | "minecraft:light_blue_bed"
            | "minecraft:yellow_bed"
            | "minecraft:lime_bed"
            | "minecraft:pink_bed"
            | "minecraft:gray_bed"
            | "minecraft:light_gray_bed"
            | "minecraft:cyan_bed"
            | "minecraft:purple_bed"
            | "minecraft:blue_bed"
            | "minecraft:brown_bed"
            | "minecraft:green_bed"
            | "minecraft:red_bed"
            | "minecraft:black_bed" => !bed_is_head,
            _ => false,
        }
}

pub fn cat_lie_on_bed_target_valid(block: &str, above_empty: bool) -> bool {
    above_empty && block.ends_with("_bed") && block.starts_with("minecraft:")
}

pub fn cat_spawner_should_try_spawn(next_tick_after_decrement: i32, player_present: bool) -> bool {
    next_tick_after_decrement <= 0 && player_present
}

pub fn cat_spawner_offset(random_0_to_23: i32, negative: bool) -> i32 {
    (8 + random_0_to_23.clamp(0, 23)) * if negative { -1 } else { 1 }
}

pub fn cat_village_spawn_allowed(occupied_home_count: i64, cats_in_radius: usize) -> bool {
    occupied_home_count > 4 && cats_in_radius < CAT_VILLAGE_MAX_CATS
}

pub fn cat_hut_spawn_allowed(cats_in_radius: usize) -> bool {
    cats_in_radius == 0
}

pub fn ocelot_spawn_rules(random_roll: i32) -> bool {
    random_roll != 0
}

pub fn ocelot_spawn_obstruction(
    unobstructed: bool,
    contains_liquid: bool,
    y: i32,
    sea_level: i32,
    block_below: &str,
) -> bool {
    unobstructed
        && !contains_liquid
        && y >= sea_level
        && (block_below == "minecraft:grass_block" || block_below.ends_with("_leaves"))
}

pub fn ocelot_leash_offset(eye_height: f32, width: f32) -> (f32, f32, f32) {
    (0.0, 0.5 * eye_height, 0.4 * width)
}

