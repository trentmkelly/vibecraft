use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RabbitVariant {
    Brown,
    White,
    Black,
    WhiteSplotched,
    Gold,
    Salt,
    Evil,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RabbitState {
    pub variant: RabbitVariant,
    pub more_carrot_ticks: i32,
    pub jump_ticks: i32,
    pub jump_duration: i32,
    pub jump_delay_ticks: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RabbitRaidGardenResult {
    DestroyCrop,
    ReduceCarrotAge(i32),
    Noop,
}

pub const RABBIT_MAX_HEALTH: f32 = 3.0;
pub const RABBIT_MOVEMENT_SPEED: f32 = 0.3;
pub const RABBIT_ATTACK_DAMAGE: f32 = 3.0;
pub const RABBIT_EVIL_ATTACK_POWER_INCREMENT: f32 = 5.0;
pub const RABBIT_EVIL_ARMOR_VALUE: f32 = 8.0;
pub const RABBIT_STROLL_SPEED_MOD: f64 = 0.6;
pub const RABBIT_BREED_SPEED_MOD: f64 = 0.8;
pub const RABBIT_FOLLOW_SPEED_MOD: f64 = 1.0;
pub const RABBIT_FLEE_SPEED_MOD: f64 = 2.2;
pub const RABBIT_ATTACK_SPEED_MOD: f64 = 1.4;
pub const RABBIT_BABY_JUMP_HEIGHT: f64 = 0.5;
pub const RABBIT_ADULT_JUMP_HEIGHT: f64 = 1.5;
pub const RABBIT_JUMP_DELAY_TICKS: i32 = 10;
pub const RABBIT_PANIC_JUMP_DELAY_TICKS: i32 = 3;
pub const RABBIT_JUMP_DURATION_TICKS: i32 = 15;
pub const RABBIT_MORE_CARROTS_DELAY: i32 = 40;
pub const RABBIT_IDLE_MINIMAL_DURATION_TICKS: i32 = 180;
pub const RABBIT_BABY_WIDTH: f32 = 0.24;
pub const RABBIT_BABY_HEIGHT: f32 = 0.4;
pub const RABBIT_BABY_EYE_HEIGHT: f32 = 0.39;

impl RabbitVariant {
    pub fn id(self) -> i32 {
        match self {
            Self::Brown => 0,
            Self::White => 1,
            Self::Black => 2,
            Self::WhiteSplotched => 3,
            Self::Gold => 4,
            Self::Salt => 5,
            Self::Evil => 99,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Brown => "brown",
            Self::White => "white",
            Self::Black => "black",
            Self::WhiteSplotched => "white_splotched",
            Self::Gold => "gold",
            Self::Salt => "salt",
            Self::Evil => "evil",
        }
    }

    pub fn by_id(id: i32) -> Self {
        match id {
            1 => Self::White,
            2 => Self::Black,
            3 => Self::WhiteSplotched,
            4 => Self::Gold,
            5 => Self::Salt,
            99 => Self::Evil,
            _ => Self::Brown,
        }
    }

    pub fn is_evil(self) -> bool {
        matches!(self, Self::Evil)
    }
}

impl RabbitState {
    pub fn new() -> Self {
        Self {
            variant: RabbitVariant::Brown,
            more_carrot_ticks: 0,
            jump_ticks: 0,
            jump_duration: 0,
            jump_delay_ticks: 0,
        }
    }

    pub fn wants_more_food(self) -> bool {
        self.more_carrot_ticks <= 0
    }

    pub fn tick_more_carrots(&mut self, random_subtract_0_to_2: i32) {
        if self.more_carrot_ticks > 0 {
            self.more_carrot_ticks -= random_subtract_0_to_2.clamp(0, 2);
            if self.more_carrot_ticks < 0 {
                self.more_carrot_ticks = 0;
            }
        }
    }

    pub fn start_jumping(&mut self) {
        self.jump_duration = RABBIT_JUMP_DURATION_TICKS;
        self.jump_ticks = 0;
    }

    pub fn ai_step_jump(&mut self) {
        if self.jump_ticks != self.jump_duration {
            self.jump_ticks += 1;
        } else if self.jump_duration != 0 {
            self.jump_ticks = 0;
            self.jump_duration = 0;
        }
    }

    pub fn jump_completion(self, partial_tick: f32) -> f32 {
        if self.jump_duration == 0 {
            0.0
        } else {
            (self.jump_ticks as f32 + partial_tick) / self.jump_duration as f32
        }
    }

    pub fn set_landing_delay(&mut self, speed_modifier: f64) {
        self.jump_delay_ticks = if speed_modifier < RABBIT_FLEE_SPEED_MOD {
            RABBIT_JUMP_DELAY_TICKS
        } else {
            RABBIT_PANIC_JUMP_DELAY_TICKS
        };
    }
}

pub fn rabbit_random_variant(
    spawns_white_rabbits: bool,
    spawns_gold_rabbits: bool,
    random_0_to_99: i32,
) -> RabbitVariant {
    let random = random_0_to_99.clamp(0, 99);
    if spawns_white_rabbits {
        if random < 80 {
            RabbitVariant::White
        } else {
            RabbitVariant::WhiteSplotched
        }
    } else if spawns_gold_rabbits {
        RabbitVariant::Gold
    } else if random < 50 {
        RabbitVariant::Brown
    } else if random < 90 {
        RabbitVariant::Salt
    } else {
        RabbitVariant::Black
    }
}

pub fn rabbit_offspring_variant(
    biome_variant: RabbitVariant,
    first_parent_variant: RabbitVariant,
    second_parent_variant: RabbitVariant,
    random_0_to_19: i32,
    choose_partner: bool,
) -> RabbitVariant {
    if random_0_to_19 == 0 {
        biome_variant
    } else if choose_partner {
        second_parent_variant
    } else {
        first_parent_variant
    }
}

pub fn rabbit_should_avoid_entity(variant: RabbitVariant, base_goal_can_use: bool) -> bool {
    !variant.is_evil() && base_goal_can_use
}

pub fn rabbit_raid_garden(
    mob_griefing: bool,
    wants_more_food: bool,
    reached_target: bool,
    carrot_age: Option<i32>,
) -> RabbitRaidGardenResult {
    if !mob_griefing || !wants_more_food || !reached_target {
        return RabbitRaidGardenResult::Noop;
    }
    match carrot_age {
        Some(0) => RabbitRaidGardenResult::DestroyCrop,
        Some(age) if age > 0 => RabbitRaidGardenResult::ReduceCarrotAge(age - 1),
        _ => RabbitRaidGardenResult::Noop,
    }
}

