
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PigState {
    pub saddled: bool,
    pub boost_time_total: i32,
    pub boosting: bool,
    pub boost_time: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PigInteraction {
    StartRide,
    DelegateToAnimal,
    EquipSaddle,
    Pass,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PigBoostUseResult {
    BoostStarted {
        damage: i32,
        converts_to_fishing_rod: bool,
    },
    Pass,
}

pub const PIG_MAX_HEALTH: f32 = 10.0;
pub const PIG_MOVEMENT_SPEED: f64 = 0.25;
pub const PIG_RIDDEN_SPEED_FACTOR: f64 = 0.225;
pub const PIG_BOOST_MIN_TIME: i32 = 140;
pub const PIG_BOOST_MAX_TIME: i32 = 980;
pub const PIG_BOOST_RANDOM_BOUND: i32 = 841;
pub const PIG_BOOST_SPEED_AMPLIFIER: f32 = 1.15;
pub const PIG_CARROT_ON_A_STICK_DURABILITY: i32 = 25;
pub const PIG_CARROT_ON_A_STICK_DAMAGE_PER_BOOST: i32 = 7;
pub const PIG_TEMPT_SPEED: f32 = 1.2;
pub const PIG_PANIC_SPEED: f32 = 1.25;
pub const PIG_FOLLOW_PARENT_SPEED: f32 = 1.1;
pub const PIG_LEASH_EYE_HEIGHT_FACTOR: f32 = 0.6;
pub const PIG_LEASH_WIDTH_FACTOR: f32 = 0.4;

impl PigState {
    pub fn new() -> Self {
        Self {
            saddled: false,
            boost_time_total: 0,
            boosting: false,
            boost_time: 0,
        }
    }

    pub fn controlling_passenger(
        self,
        first_passenger_is_player: bool,
        player_holds_carrot_on_a_stick: bool,
    ) -> bool {
        self.saddled && first_passenger_is_player && player_holds_carrot_on_a_stick
    }

    pub fn can_use_saddle_slot(alive: bool, baby: bool) -> bool {
        alive && !baby
    }

    pub fn boost(&mut self, random_offset_0_to_840: i32) -> bool {
        if self.boosting {
            return false;
        }
        self.boosting = true;
        self.boost_time = 0;
        self.boost_time_total =
            PIG_BOOST_MIN_TIME + random_offset_0_to_840.clamp(0, PIG_BOOST_RANDOM_BOUND - 1);
        true
    }

    pub fn tick_boost(&mut self) {
        if self.boosting {
            let previous_boost_time = self.boost_time;
            self.boost_time += 1;
            if previous_boost_time > self.boost_time_total {
                self.boosting = false;
            }
        }
    }

    pub fn boost_factor(self) -> f32 {
        if self.boosting {
            1.0 + PIG_BOOST_SPEED_AMPLIFIER
                * ((self.boost_time as f32 / self.boost_time_total as f32) * std::f32::consts::PI)
                    .sin()
        } else {
            1.0
        }
    }

    pub fn ridden_speed(self, movement_speed: f64) -> f32 {
        (movement_speed * PIG_RIDDEN_SPEED_FACTOR * self.boost_factor() as f64) as f32
    }
}

pub fn pig_interaction(
    has_food: bool,
    saddled: bool,
    is_vehicle: bool,
    player_secondary_use_active: bool,
    super_interaction_consumes: bool,
    item_equippable_saddle: bool,
) -> PigInteraction {
    if !has_food && saddled && !is_vehicle && !player_secondary_use_active {
        PigInteraction::StartRide
    } else if super_interaction_consumes {
        PigInteraction::DelegateToAnimal
    } else if item_equippable_saddle {
        PigInteraction::EquipSaddle
    } else {
        PigInteraction::Pass
    }
}

pub fn pig_thunder_converts_to_zombified_piglin(difficulty: &str) -> bool {
    difficulty != "peaceful"
}

pub fn pig_food_on_a_stick_use(
    server_side: bool,
    player_is_passenger: bool,
    controlled_vehicle_is_pig: bool,
    boost_started: bool,
    current_damage: i32,
) -> PigBoostUseResult {
    if server_side && player_is_passenger && controlled_vehicle_is_pig && boost_started {
        PigBoostUseResult::BoostStarted {
            damage: PIG_CARROT_ON_A_STICK_DAMAGE_PER_BOOST,
            converts_to_fishing_rod: current_damage + PIG_CARROT_ON_A_STICK_DAMAGE_PER_BOOST
                >= PIG_CARROT_ON_A_STICK_DURABILITY,
        }
    } else {
        PigBoostUseResult::Pass
    }
}

pub fn pig_offspring_variant<'a>(
    first_parent_variant: &'a str,
    second_parent_variant: &'a str,
    choose_first_parent: bool,
) -> &'a str {
    if choose_first_parent {
        first_parent_variant
    } else {
        second_parent_variant
    }
}

