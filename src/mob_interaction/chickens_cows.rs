
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ChickenState {
    pub egg_time: i32,
    pub is_chicken_jockey: bool,
    pub flap: f32,
    pub flap_speed: f32,
    pub flapping: f32,
    pub delta_y: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChickenTickEvent {
    None,
    LayEgg,
}

pub const CHICKEN_EGG_TIME_MIN: i32 = 6000;
pub const CHICKEN_EGG_TIME_RANDOM_BOUND: i32 = 6000;
pub const CHICKEN_ADULT_WIDTH: f32 = 0.4;
pub const CHICKEN_ADULT_HEIGHT: f32 = 0.7;
pub const CHICKEN_BABY_WIDTH: f32 = 0.3;
pub const CHICKEN_BABY_HEIGHT: f32 = 0.4;
pub const CHICKEN_BABY_EYE_HEIGHT: f32 = 0.28;
pub const CHICKEN_JOCKEY_BASE_EXPERIENCE: i32 = 10;

impl ChickenState {
    pub fn new(random_egg_offset: i32) -> Self {
        Self {
            egg_time: chicken_next_egg_time(random_egg_offset),
            is_chicken_jockey: false,
            flap: 0.0,
            flap_speed: 0.0,
            flapping: 1.0,
            delta_y: 0.0,
        }
    }

    pub fn tick(
        &mut self,
        on_ground: bool,
        alive: bool,
        baby: bool,
        server_level: bool,
        next_random_egg_offset: i32,
    ) -> ChickenTickEvent {
        self.flap_speed += if on_ground { -0.3 } else { 1.2 };
        self.flap_speed = self.flap_speed.clamp(0.0, 1.0);
        if !on_ground && self.flapping < 1.0 {
            self.flapping = 1.0;
        }
        self.flapping *= 0.9;
        if !on_ground && self.delta_y < 0.0 {
            self.delta_y *= 0.6;
        }
        self.flap += self.flapping * 2.0;

        if server_level && alive && !baby && !self.is_chicken_jockey {
            self.egg_time -= 1;
            if self.egg_time <= 0 {
                self.egg_time = chicken_next_egg_time(next_random_egg_offset);
                return ChickenTickEvent::LayEgg;
            }
        }

        ChickenTickEvent::None
    }

    pub fn remove_when_far_away(self) -> bool {
        self.is_chicken_jockey
    }

    pub fn base_experience_reward(self, super_reward: i32) -> i32 {
        if self.is_chicken_jockey {
            CHICKEN_JOCKEY_BASE_EXPERIENCE
        } else {
            super_reward
        }
    }
}

pub fn chicken_next_egg_time(random_offset: i32) -> i32 {
    CHICKEN_EGG_TIME_MIN + random_offset.clamp(0, CHICKEN_EGG_TIME_RANDOM_BOUND - 1)
}

pub fn chicken_dimensions(baby: bool) -> (f32, f32, Option<f32>) {
    if baby {
        (
            CHICKEN_BABY_WIDTH,
            CHICKEN_BABY_HEIGHT,
            Some(CHICKEN_BABY_EYE_HEIGHT),
        )
    } else {
        (CHICKEN_ADULT_WIDTH, CHICKEN_ADULT_HEIGHT, None)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CowInteraction {
    FillMilkBucket,
    Delegate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MooshroomVariant {
    Red,
    Brown,
}

pub const COW_FOOD_ITEM: &str = "minecraft:wheat";
pub const COW_ADULT_WIDTH: f32 = 0.9;
pub const COW_ADULT_HEIGHT: f32 = 1.4;
pub const COW_BABY_WIDTH: f32 = 0.45;
pub const COW_BABY_HEIGHT: f32 = 0.7;
pub const COW_BABY_EYE_HEIGHT: f32 = 0.665;
pub const MOOSHROOM_MUTATE_CHANCE: i32 = 1024;

pub fn cow_interaction(item: &str, baby: bool) -> CowInteraction {
    if item == "minecraft:bucket" && !baby {
        CowInteraction::FillMilkBucket
    } else {
        CowInteraction::Delegate
    }
}

pub fn cow_is_food(item: &str) -> bool {
    item == COW_FOOD_ITEM
}

pub fn cow_dimensions(baby: bool) -> (f32, f32, Option<f32>) {
    if baby {
        (COW_BABY_WIDTH, COW_BABY_HEIGHT, Some(COW_BABY_EYE_HEIGHT))
    } else {
        (COW_ADULT_WIDTH, COW_ADULT_HEIGHT, None)
    }
}

pub fn cow_breed_variant(
    parent_variant: &'static str,
    partner_variant: &'static str,
    choose_parent: bool,
) -> &'static str {
    if choose_parent {
        parent_variant
    } else {
        partner_variant
    }
}

pub fn mooshroom_thunder_variant(
    current: MooshroomVariant,
    last_lightning_uuid: Option<&str>,
    lightning_uuid: &str,
) -> MooshroomVariant {
    if last_lightning_uuid == Some(lightning_uuid) {
        current
    } else {
        match current {
            MooshroomVariant::Red => MooshroomVariant::Brown,
            MooshroomVariant::Brown => MooshroomVariant::Red,
        }
    }
}

pub fn mooshroom_offspring_variant(
    parent: MooshroomVariant,
    partner: MooshroomVariant,
    mutate_same_variant: bool,
    choose_parent: bool,
) -> MooshroomVariant {
    if parent == partner && mutate_same_variant {
        match parent {
            MooshroomVariant::Red => MooshroomVariant::Brown,
            MooshroomVariant::Brown => MooshroomVariant::Red,
        }
    } else if choose_parent {
        parent
    } else {
        partner
    }
}

