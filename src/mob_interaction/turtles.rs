use super::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TurtleAttributes {
    pub max_health: f32,
    pub movement_speed: f32,
    pub step_height: f32,
}

pub fn turtle_attributes() -> TurtleAttributes {
    TurtleAttributes {
        max_health: TURTLE_MAX_HEALTH,
        movement_speed: TURTLE_MOVEMENT_SPEED,
        step_height: TURTLE_STEP_HEIGHT,
    }
}

pub fn turtle_food_item(item: &str) -> bool {
    item == "minecraft:seagrass"
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TurtleStateModel {
    pub home_pos: (i32, i32, i32),
    pub has_egg: bool,
    pub laying_egg: bool,
    pub lay_egg_counter: i32,
    pub going_home: bool,
}

impl TurtleStateModel {
    pub fn new(home_pos: (i32, i32, i32)) -> Self {
        Self {
            home_pos,
            has_egg: false,
            laying_egg: false,
            lay_egg_counter: 0,
            going_home: false,
        }
    }

    pub fn set_laying_egg(&mut self, value: bool) {
        self.lay_egg_counter = if value { 1 } else { 0 };
        self.laying_egg = value;
    }
}

pub fn turtle_can_fall_in_love(super_can_fall_in_love: bool, has_egg: bool) -> bool {
    super_can_fall_in_love && !has_egg
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TurtleBreedPlan {
    pub has_egg_after: bool,
    pub parent_age: i32,
    pub reset_love: bool,
    pub bred_animals_stat: bool,
    pub xp_min: i32,
    pub xp_max: i32,
}

pub fn turtle_breed_plan(has_love_cause: bool, mob_drops: bool) -> TurtleBreedPlan {
    TurtleBreedPlan {
        has_egg_after: true,
        parent_age: TURTLE_BREED_PARENT_AGE,
        reset_love: true,
        bred_animals_stat: has_love_cause,
        xp_min: if mob_drops { 1 } else { 0 },
        xp_max: if mob_drops { 7 } else { 0 },
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TurtleLayEggStep {
    None,
    StartLaying,
    DigParticles {
        level_event: i32,
        game_event: &'static str,
    },
    PlaceEggs {
        egg_count: i32,
        sound: &'static str,
        game_event: &'static str,
        has_egg_after: bool,
        laying_after: bool,
        in_love_time: i32,
    },
}

pub fn turtle_lay_egg_step(
    has_egg: bool,
    laying_egg: bool,
    lay_egg_counter: i32,
    in_water: bool,
    reached_target: bool,
    on_sand: bool,
    random_egg_count_zero_to_three: i32,
) -> TurtleLayEggStep {
    if !has_egg || in_water || !reached_target {
        return TurtleLayEggStep::None;
    }
    if !laying_egg || lay_egg_counter < 1 {
        return TurtleLayEggStep::StartLaying;
    }
    if lay_egg_counter > TURTLE_LAY_EGG_DELAY_TICKS {
        return TurtleLayEggStep::PlaceEggs {
            egg_count: random_egg_count_zero_to_three.clamp(0, 3) + 1,
            sound: "minecraft:entity.turtle.lay_egg",
            game_event: "minecraft:block_place",
            has_egg_after: false,
            laying_after: false,
            in_love_time: 600,
        };
    }
    if on_sand && lay_egg_counter % TURTLE_LAY_EGG_PARTICLE_INTERVAL_TICKS == 0 {
        TurtleLayEggStep::DigParticles {
            level_event: 2001,
            game_event: "minecraft:entity_action",
        }
    } else {
        TurtleLayEggStep::None
    }
}

pub fn turtle_grow_drop_loot(adult_after_growth: bool, mob_drops: bool) -> Option<&'static str> {
    (adult_after_growth && mob_drops).then_some("minecraft:gameplay/turtle_grow")
}

pub fn turtle_spawn_allowed(
    pos_y: i32,
    sea_level: i32,
    on_sand: bool,
    bright_enough: bool,
) -> bool {
    pos_y < sea_level + 4 && on_sand && bright_enough
}

pub fn turtle_go_home_can_use(
    baby: bool,
    has_egg: bool,
    home_distance: f32,
    random_roll_zero: bool,
) -> bool {
    if baby {
        false
    } else if has_egg {
        true
    } else {
        random_roll_zero && home_distance >= TURTLE_GO_HOME_DISTANCE
    }
}

pub fn turtle_lay_egg_goal_can_use(has_egg: bool, home_distance: f32, super_can_use: bool) -> bool {
    has_egg && home_distance < TURTLE_LAY_EGG_HOME_DISTANCE && super_can_use
}

pub fn turtle_go_to_water_can_use(
    baby: bool,
    in_water: bool,
    going_home: bool,
    has_egg: bool,
    super_can_use: bool,
) -> bool {
    if baby && !in_water {
        super_can_use
    } else {
        !going_home && !in_water && !has_egg && super_can_use
    }
}

pub fn turtle_travel_can_use(going_home: bool, has_egg: bool, in_water: bool) -> bool {
    !going_home && !has_egg && in_water
}

pub fn turtle_water_travel_sinking(
    has_target: bool,
    going_home: bool,
    home_distance: f32,
) -> Option<(f32, f32, f32)> {
    if !has_target && (!going_home || home_distance >= 20.0) {
        Some((0.0, -0.005, 0.0))
    } else {
        None
    }
}

pub fn turtle_can_be_leashed() -> bool {
    false
}

pub fn turtle_lightning_damage() -> f32 {
    f32::MAX
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TurtleEggDestroyPlan {
    None,
    DecreaseEggs {
        eggs_after: i32,
        sound: &'static str,
        destroy_block: bool,
        game_event: Option<&'static str>,
    },
}

pub fn turtle_egg_destroy_plan(
    eggs: i32,
    entity_is_turtle: bool,
    entity_is_bat: bool,
    entity_is_living: bool,
    entity_is_player: bool,
    mob_griefing: bool,
    random_roll_zero: bool,
) -> TurtleEggDestroyPlan {
    let can_destroy = !entity_is_turtle
        && !entity_is_bat
        && entity_is_living
        && (entity_is_player || mob_griefing);
    if !can_destroy || !random_roll_zero {
        return TurtleEggDestroyPlan::None;
    }
    if eggs <= 1 {
        TurtleEggDestroyPlan::DecreaseEggs {
            eggs_after: 0,
            sound: "minecraft:entity.turtle.egg_break",
            destroy_block: true,
            game_event: None,
        }
    } else {
        TurtleEggDestroyPlan::DecreaseEggs {
            eggs_after: eggs - 1,
            sound: "minecraft:entity.turtle.egg_break",
            destroy_block: false,
            game_event: Some("minecraft:block_destroy"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TurtleEggRandomTickPlan {
    None,
    Crack {
        hatch: i32,
        sound: &'static str,
        game_event: &'static str,
    },
    Hatch {
        hatchlings: i32,
        hatchling_age: i32,
        sound: &'static str,
        game_event: &'static str,
    },
}

pub fn turtle_egg_random_tick_plan(
    on_sand: bool,
    should_update_hatch_level: bool,
    hatch: i32,
    eggs: i32,
) -> TurtleEggRandomTickPlan {
    if !on_sand || !should_update_hatch_level {
        return TurtleEggRandomTickPlan::None;
    }
    if hatch < TURTLE_EGG_MAX_HATCH_LEVEL {
        TurtleEggRandomTickPlan::Crack {
            hatch: hatch + 1,
            sound: "minecraft:entity.turtle.egg_crack",
            game_event: "minecraft:block_change",
        }
    } else {
        TurtleEggRandomTickPlan::Hatch {
            hatchlings: eggs.clamp(TURTLE_EGG_MIN_EGGS, TURTLE_EGG_MAX_EGGS),
            hatchling_age: TURTLE_HATCHLING_AGE,
            sound: "minecraft:entity.turtle.egg_hatch",
            game_event: "minecraft:block_destroy",
        }
    }
}

pub fn turtle_egg_can_be_replaced(
    secondary_use_active: bool,
    item_is_turtle_egg: bool,
    eggs: i32,
) -> bool {
    !secondary_use_active && item_is_turtle_egg && eggs < TURTLE_EGG_MAX_EGGS
}

pub fn turtle_egg_placement_eggs(existing_eggs: Option<i32>) -> i32 {
    existing_eggs
        .map(|eggs| (eggs + 1).min(TURTLE_EGG_MAX_EGGS))
        .unwrap_or(TURTLE_EGG_MIN_EGGS)
}

