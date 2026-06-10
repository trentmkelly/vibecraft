use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrogVariantModel {
    Temperate,
    Warm,
    Cold,
}

impl FrogVariantModel {
    pub fn registry_id(self) -> &'static str {
        match self {
            FrogVariantModel::Temperate => "minecraft:temperate",
            FrogVariantModel::Warm => "minecraft:warm",
            FrogVariantModel::Cold => "minecraft:cold",
        }
    }

    pub fn texture(self) -> &'static str {
        match self {
            FrogVariantModel::Temperate => "minecraft:entity/frog/frog_temperate",
            FrogVariantModel::Warm => "minecraft:entity/frog/frog_warm",
            FrogVariantModel::Cold => "minecraft:entity/frog/frog_cold",
        }
    }
}

pub fn frog_variant_for_spawn_biome(biome: &'static str) -> FrogVariantModel {
    if frog_warm_variant_biome(biome) {
        FrogVariantModel::Warm
    } else if frog_cold_variant_biome(biome) {
        FrogVariantModel::Cold
    } else {
        FrogVariantModel::Temperate
    }
}

pub fn frog_warm_variant_biome(biome: &'static str) -> bool {
    matches!(
        biome,
        "minecraft:desert"
            | "minecraft:warm_ocean"
            | "minecraft:jungle"
            | "minecraft:sparse_jungle"
            | "minecraft:bamboo_jungle"
            | "minecraft:savanna"
            | "minecraft:savanna_plateau"
            | "minecraft:windswept_savanna"
            | "minecraft:nether_wastes"
            | "minecraft:soul_sand_valley"
            | "minecraft:crimson_forest"
            | "minecraft:warped_forest"
            | "minecraft:basalt_deltas"
            | "minecraft:badlands"
            | "minecraft:eroded_badlands"
            | "minecraft:wooded_badlands"
            | "minecraft:mangrove_swamp"
    )
}

pub fn frog_cold_variant_biome(biome: &'static str) -> bool {
    matches!(
        biome,
        "minecraft:snowy_plains"
            | "minecraft:ice_spikes"
            | "minecraft:frozen_peaks"
            | "minecraft:jagged_peaks"
            | "minecraft:snowy_slopes"
            | "minecraft:frozen_ocean"
            | "minecraft:deep_frozen_ocean"
            | "minecraft:grove"
            | "minecraft:deep_dark"
            | "minecraft:frozen_river"
            | "minecraft:snowy_taiga"
            | "minecraft:snowy_beach"
            | "minecraft:the_end"
            | "minecraft:end_highlands"
            | "minecraft:end_midlands"
            | "minecraft:small_end_islands"
            | "minecraft:end_barrens"
    )
}

pub fn frog_can_eat(entity_type: &'static str, slime_size: Option<i32>) -> bool {
    match entity_type {
        "minecraft:slime" => slime_size == Some(1),
        "minecraft:magma_cube" => true,
        _ => false,
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FrogTongueCatchPlan {
    pub tongue_sound: &'static str,
    pub eat_sound: &'static str,
    pub pose: &'static str,
    pub target_velocity_scale: f32,
    pub catch_animation_ticks: i32,
    pub eat_animation_ticks: i32,
    pub max_eating_distance: f32,
}

pub fn frog_tongue_catch_plan(
    entity_type: &'static str,
    slime_size: Option<i32>,
    path_distance: f32,
    panicking: bool,
    pose: &'static str,
) -> Option<FrogTongueCatchPlan> {
    if panicking
        || pose == "croaking"
        || path_distance >= 1.75
        || !frog_can_eat(entity_type, slime_size)
    {
        return None;
    }
    Some(FrogTongueCatchPlan {
        tongue_sound: "minecraft:entity.frog.tongue",
        eat_sound: "minecraft:entity.frog.eat",
        pose: "using_tongue",
        target_velocity_scale: 0.75,
        catch_animation_ticks: 6,
        eat_animation_ticks: 10,
        max_eating_distance: 1.75,
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FrogBreedingPlan {
    pub memory: &'static str,
    pub activity: &'static str,
    pub placed_block: &'static str,
    pub land_search_radius: i32,
}

pub fn frog_breeding_lay_spawn_plan() -> FrogBreedingPlan {
    FrogBreedingPlan {
        memory: "minecraft:is_pregnant",
        activity: "minecraft:lay_spawn",
        placed_block: "minecraft:frogspawn",
        land_search_radius: 8,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FoxVariantModel {
    Red,
    Snow,
}

impl FoxVariantModel {
    pub fn id(self) -> i32 {
        match self {
            FoxVariantModel::Red => 0,
            FoxVariantModel::Snow => 1,
        }
    }

    pub fn serialized_name(self) -> &'static str {
        match self {
            FoxVariantModel::Red => "red",
            FoxVariantModel::Snow => "snow",
        }
    }
}

pub fn fox_variant_by_id(id: i32) -> FoxVariantModel {
    match id {
        1 => FoxVariantModel::Snow,
        _ => FoxVariantModel::Red,
    }
}

pub fn fox_variant_for_spawn_biome(biome: &'static str) -> FoxVariantModel {
    if matches!(
        biome,
        "minecraft:snowy_plains"
            | "minecraft:ice_spikes"
            | "minecraft:frozen_ocean"
            | "minecraft:snowy_taiga"
            | "minecraft:frozen_river"
            | "minecraft:snowy_beach"
    ) {
        FoxVariantModel::Snow
    } else {
        FoxVariantModel::Red
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FoxState {
    pub variant: FoxVariantModel,
    flags: u8,
    pub trusted: [Option<&'static str>; 2],
    pub ticks_since_eaten: i32,
}

impl FoxState {
    pub fn new() -> Self {
        Self {
            variant: FoxVariantModel::Red,
            flags: 0,
            trusted: [None, None],
            ticks_since_eaten: 0,
        }
    }

    pub fn get_flag(self, flag: u8) -> bool {
        (self.flags & flag) != 0
    }

    pub fn set_flag(&mut self, flag: u8, value: bool) {
        if value {
            self.flags |= flag;
        } else {
            self.flags &= !flag;
        }
    }

    pub fn is_sitting(self) -> bool {
        self.get_flag(FOX_FLAG_SITTING)
    }

    pub fn is_crouching(self) -> bool {
        self.get_flag(FOX_FLAG_CROUCHING)
    }

    pub fn is_interested(self) -> bool {
        self.get_flag(FOX_FLAG_INTERESTED)
    }

    pub fn is_pouncing(self) -> bool {
        self.get_flag(FOX_FLAG_POUNCING)
    }

    pub fn is_sleeping(self) -> bool {
        self.get_flag(FOX_FLAG_SLEEPING)
    }

    pub fn is_faceplanted(self) -> bool {
        self.get_flag(FOX_FLAG_FACEPLANTED)
    }

    pub fn is_defending(self) -> bool {
        self.get_flag(FOX_FLAG_DEFENDING)
    }

    pub fn clear_states(&mut self) {
        self.set_flag(FOX_FLAG_INTERESTED, false);
        self.set_flag(FOX_FLAG_CROUCHING, false);
        self.set_flag(FOX_FLAG_SITTING, false);
        self.set_flag(FOX_FLAG_SLEEPING, false);
        self.set_flag(FOX_FLAG_DEFENDING, false);
        self.set_flag(FOX_FLAG_FACEPLANTED, false);
    }

    pub fn can_move(self) -> bool {
        !self.is_sleeping() && !self.is_sitting() && !self.is_faceplanted()
    }

    pub fn add_trusted(&mut self, entity_id: &'static str) {
        if self.trusted[0].is_some() {
            self.trusted[1] = Some(entity_id);
        } else {
            self.trusted[0] = Some(entity_id);
        }
    }

    pub fn trusts(self, entity_id: &'static str) -> bool {
        self.trusted.contains(&Some(entity_id))
    }
}

pub fn fox_food_item(item: &'static str) -> bool {
    matches!(item, "minecraft:sweet_berries" | "minecraft:glow_berries")
}

pub fn fox_can_eat_held_item(
    held_item: &'static str,
    has_target: bool,
    on_ground: bool,
    sleeping: bool,
) -> bool {
    fox_food_item(held_item) && !has_target && on_ground && !sleeping
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FoxHeldItemDecision {
    Reject,
    Hold,
    ReplaceAndSpitOld,
}

pub fn fox_can_hold_item(
    held_item: Option<&'static str>,
    candidate_item: &'static str,
    ticks_since_eaten: i32,
) -> FoxHeldItemDecision {
    match held_item {
        None => FoxHeldItemDecision::Hold,
        Some(current)
            if ticks_since_eaten > 0
                && fox_food_item(candidate_item)
                && !fox_food_item(current) =>
        {
            FoxHeldItemDecision::ReplaceAndSpitOld
        }
        _ => FoxHeldItemDecision::Reject,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FoxBreedPlan {
    pub baby_variant: FoxVariantModel,
    pub trusted: [Option<&'static str>; 2],
    pub parent_age: i32,
    pub baby_age: i32,
    pub event_id: u8,
    pub xp_min: i32,
    pub xp_max_inclusive: i32,
}

pub fn fox_breed_plan(
    parent_variant: FoxVariantModel,
    partner_variant: FoxVariantModel,
    choose_parent_variant: bool,
    parent_love_cause: Option<&'static str>,
    partner_love_cause: Option<&'static str>,
) -> FoxBreedPlan {
    let mut trusted = [None, None];
    if let Some(parent) = parent_love_cause {
        trusted[0] = Some(parent);
    }
    if let Some(partner) = partner_love_cause {
        if parent_love_cause == Some(partner) {
            trusted[0] = Some(partner);
        } else if trusted[0].is_some() {
            trusted[1] = Some(partner);
        } else {
            trusted[0] = Some(partner);
        }
    }
    FoxBreedPlan {
        baby_variant: if choose_parent_variant {
            parent_variant
        } else {
            partner_variant
        },
        trusted,
        parent_age: 6000,
        baby_age: -24000,
        event_id: 18,
        xp_min: 1,
        xp_max_inclusive: 7,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FoxBerryHarvestPlan {
    None,
    PickSweetBerries {
        held_item: Option<&'static str>,
        dropped_count: i32,
        new_age: i32,
        sound: &'static str,
        game_event: &'static str,
    },
    PickGlowBerry,
}

pub fn fox_berry_harvest_plan(
    block: &'static str,
    age: i32,
    held_item: Option<&'static str>,
    random_next_int_2: i32,
    mob_griefing: bool,
    sleeping: bool,
    ticks_waited: i32,
) -> FoxBerryHarvestPlan {
    if sleeping || !mob_griefing || ticks_waited < FOX_BERRY_WAIT_TICKS {
        return FoxBerryHarvestPlan::None;
    }
    match block {
        "minecraft:sweet_berry_bush" if age >= 2 => {
            let mut count = 1 + random_next_int_2 + i32::from(age == 3);
            let mut new_held_item = held_item;
            if held_item.is_none() {
                new_held_item = Some("minecraft:sweet_berries");
                count -= 1;
            }
            FoxBerryHarvestPlan::PickSweetBerries {
                held_item: new_held_item,
                dropped_count: count.max(0),
                new_age: 1,
                sound: "minecraft:block.sweet_berry_bush.pick_berries",
                game_event: "minecraft:block_change",
            }
        }
        "minecraft:cave_vines" if age > 0 => FoxBerryHarvestPlan::PickGlowBerry,
        _ => FoxBerryHarvestPlan::None,
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FoxStalkPlan {
    pub interested: bool,
    pub crouching: bool,
    pub move_speed: Option<f32>,
}

pub fn fox_stalk_prey_plan(
    target_entity_type: &'static str,
    distance_sqr: f32,
    sleeping: bool,
    crouching: bool,
    interested: bool,
    jumping: bool,
    path_clear: bool,
) -> Option<FoxStalkPlan> {
    let stalkable = matches!(target_entity_type, "minecraft:chicken" | "minecraft:rabbit");
    if sleeping
        || !stalkable
        || distance_sqr <= FOX_STALK_DISTANCE_SQR
        || crouching
        || interested
        || jumping
    {
        return None;
    }
    if path_clear {
        Some(FoxStalkPlan {
            interested: true,
            crouching: true,
            move_speed: None,
        })
    } else {
        Some(FoxStalkPlan {
            interested: false,
            crouching: false,
            move_speed: Some(1.5),
        })
    }
}
