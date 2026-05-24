use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BeeState {
    pub flags: u8,
    pub time_since_sting: i32,
    pub ticks_without_nectar_since_exiting_hive: i32,
    pub stay_out_of_hive_countdown: i32,
    pub crops_grown_since_pollination: i32,
    pub under_water_ticks: i32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BeeServerStepOutcome {
    pub drown_damage: Option<f32>,
    pub sting_death_damage: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BeeHiveUseResult {
    ShearHoneycombAndReset,
    FillHoneyBottleAndReset,
    Delegate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BeeHiveReleaseResult {
    StayInHive,
    FrontBlocked,
    Release {
        honey_level: i32,
        bee_has_nectar: bool,
        bee_crops_grown: i32,
        copied_flower_pos: bool,
    },
}

pub const BEE_FLAG_ROLL: u8 = 2;
pub const BEE_FLAG_HAS_STUNG: u8 = 4;
pub const BEE_FLAG_HAS_NECTAR: u8 = 8;
pub const BEE_STING_DEATH_COUNTDOWN: i32 = 1200;
pub const BEE_TICKS_BEFORE_GOING_TO_KNOWN_FLOWER: i32 = 600;
pub const BEE_TICKS_WITHOUT_NECTAR_BEFORE_GOING_HOME: i32 = 3600;
pub const BEE_MAX_CROPS_GROWABLE: i32 = 10;
pub const BEE_GROW_CROP_CHANCE_BOUND: i32 = 30;
pub const BEE_POISON_SECONDS_NORMAL: i32 = 10;
pub const BEE_POISON_SECONDS_HARD: i32 = 18;
pub const BEE_TOO_FAR_DISTANCE: i32 = 48;
pub const BEE_HIVE_CLOSE_ENOUGH_DISTANCE: i32 = 2;
pub const BEE_HIVE_SEARCH_DISTANCE: i32 = 20;
pub const BEE_HIVE_MAX_OCCUPANTS: i32 = 3;
pub const BEE_MIN_TICKS_IN_HIVE_WITH_NECTAR: i32 = 2400;
pub const BEE_MIN_TICKS_IN_HIVE_WITHOUT_NECTAR: i32 = 600;
pub const BEE_HONEY_LEVEL_MAX: i32 = 5;
pub const BEE_COOLDOWN_BEFORE_LOCATING_NEW_HIVE: i32 = 200;
pub const BEE_MIN_FIND_FLOWER_RETRY_COOLDOWN: i32 = 20;
pub const BEE_MAX_FIND_FLOWER_RETRY_COOLDOWN: i32 = 60;
pub const BEE_PERSISTENT_ANGER_MIN_TICKS: i32 = 20 * 20;
pub const BEE_PERSISTENT_ANGER_MAX_TICKS: i32 = 39 * 20;
pub const BEE_DROWN_DAMAGE: f32 = 1.0;
pub const BEE_MAX_HEALTH: f32 = 10.0;
pub const BEE_FLYING_SPEED: f32 = 0.6;
pub const BEE_MOVEMENT_SPEED: f32 = 0.3;
pub const BEE_ATTACK_DAMAGE: f32 = 2.0;

impl BeeState {
    pub fn new() -> Self {
        Self {
            flags: 0,
            time_since_sting: 0,
            ticks_without_nectar_since_exiting_hive: 0,
            stay_out_of_hive_countdown: 0,
            crops_grown_since_pollination: 0,
            under_water_ticks: 0,
        }
    }

    pub fn has_nectar(self) -> bool {
        self.get_flag(BEE_FLAG_HAS_NECTAR)
    }

    pub fn has_stung(self) -> bool {
        self.get_flag(BEE_FLAG_HAS_STUNG)
    }

    pub fn is_rolling(self) -> bool {
        self.get_flag(BEE_FLAG_ROLL)
    }

    pub fn set_has_nectar(&mut self, has_nectar: bool) {
        if has_nectar {
            self.ticks_without_nectar_since_exiting_hive = 0;
        }
        self.set_flag(BEE_FLAG_HAS_NECTAR, has_nectar);
    }

    pub fn set_has_stung(&mut self, has_stung: bool) {
        self.set_flag(BEE_FLAG_HAS_STUNG, has_stung);
    }

    pub fn set_rolling(&mut self, rolling: bool) {
        self.set_flag(BEE_FLAG_ROLL, rolling);
    }

    pub fn wants_to_enter_hive(
        self,
        pollinating: bool,
        has_target: bool,
        bees_stay_in_hive_environment: bool,
        hive_near_fire: bool,
    ) -> bool {
        if self.stay_out_of_hive_countdown > 0 || pollinating || self.has_stung() || has_target {
            return false;
        }
        (self.has_nectar()
            || self.ticks_without_nectar_since_exiting_hive
                > BEE_TICKS_WITHOUT_NECTAR_BEFORE_GOING_HOME
            || bees_stay_in_hive_environment)
            && !hive_near_fire
    }

    pub fn tick_server_ai(
        &mut self,
        in_water: bool,
        sting_death_roll_hits: bool,
    ) -> BeeServerStepOutcome {
        if in_water {
            self.under_water_ticks += 1;
        } else {
            self.under_water_ticks = 0;
        }

        let drown_damage = (self.under_water_ticks > 20).then_some(BEE_DROWN_DAMAGE);
        let mut sting_death_damage = false;
        if self.has_stung() {
            self.time_since_sting += 1;
            sting_death_damage = self.time_since_sting % 5 == 0 && sting_death_roll_hits;
        }

        if !self.has_nectar() {
            self.ticks_without_nectar_since_exiting_hive += 1;
        }

        BeeServerStepOutcome {
            drown_damage,
            sting_death_damage,
        }
    }

    pub fn tick_ai_step(&mut self, angry: bool, has_target: bool, target_distance_squared: f64) {
        if self.stay_out_of_hive_countdown > 0 {
            self.stay_out_of_hive_countdown -= 1;
        }
        self.set_rolling(angry && !self.has_stung() && has_target && target_distance_squared < 4.0);
    }

    fn set_flag(&mut self, flag: u8, value: bool) {
        if value {
            self.flags |= flag;
        } else {
            self.flags &= !flag;
        }
    }

    fn get_flag(self, flag: u8) -> bool {
        self.flags & flag != 0
    }

    pub fn drop_off_nectar(&mut self) {
        self.set_has_nectar(false);
        self.crops_grown_since_pollination = 0;
    }

    pub fn can_grow_crop(self, random_float: f32, hive_valid: bool) -> bool {
        self.crops_grown_since_pollination < BEE_MAX_CROPS_GROWABLE
            && random_float >= 0.3
            && self.has_nectar()
            && hive_valid
    }

    pub fn grow_crop(
        &mut self,
        random_roll: i32,
        growable_below_one: bool,
        growable_below_two: bool,
    ) -> bool {
        if random_roll.rem_euclid(BEE_GROW_CROP_CHANCE_BOUND) != 0 {
            return false;
        }
        if growable_below_one || growable_below_two {
            self.crops_grown_since_pollination += 1;
            true
        } else {
            false
        }
    }
}

pub fn bee_poison_duration_ticks(difficulty: &str) -> Option<i32> {
    match difficulty {
        "normal" => Some(BEE_POISON_SECONDS_NORMAL * 20),
        "hard" => Some(BEE_POISON_SECONDS_HARD * 20),
        _ => None,
    }
}

pub fn bee_hive_min_ticks_in_hive(has_nectar: bool) -> i32 {
    if has_nectar {
        BEE_MIN_TICKS_IN_HIVE_WITH_NECTAR
    } else {
        BEE_MIN_TICKS_IN_HIVE_WITHOUT_NECTAR
    }
}

pub fn bee_hive_can_add_occupant(current_occupants: i32) -> bool {
    current_occupants < BEE_HIVE_MAX_OCCUPANTS
}

pub fn bee_hive_use(
    honey_level: i32,
    item: &str,
    server_level: bool,
    _sedated: bool,
    _contains_bees: bool,
) -> BeeHiveUseResult {
    if honey_level < BEE_HONEY_LEVEL_MAX {
        return BeeHiveUseResult::Delegate;
    }
    match item {
        "minecraft:shears" if server_level => BeeHiveUseResult::ShearHoneycombAndReset,
        "minecraft:glass_bottle" => BeeHiveUseResult::FillHoneyBottleAndReset,
        _ => BeeHiveUseResult::Delegate,
    }
}

pub fn bee_hive_should_anger_nearby_bees(
    hive_emptied: bool,
    sedated: bool,
    contains_bees: bool,
) -> bool {
    hive_emptied && !sedated && contains_bees
}

pub fn bee_hive_release_occupant(
    bees_stay_in_hive_environment: bool,
    emergency: bool,
    front_blocked: bool,
    honey_delivered: bool,
    honey_level: i32,
    honey_level_bonus_roll_hits: bool,
    bee_has_saved_flower: bool,
    hive_has_saved_flower: bool,
    copy_flower_roll_hits: bool,
) -> BeeHiveReleaseResult {
    if bees_stay_in_hive_environment && !emergency {
        return BeeHiveReleaseResult::StayInHive;
    }
    if front_blocked && !emergency {
        return BeeHiveReleaseResult::FrontBlocked;
    }

    let mut next_honey_level = honey_level;
    let mut bee_has_nectar = honey_delivered;
    let bee_crops_grown = 0;
    if honey_delivered {
        bee_has_nectar = false;
        if next_honey_level < BEE_HONEY_LEVEL_MAX {
            let mut increase = if honey_level_bonus_roll_hits { 2 } else { 1 };
            if next_honey_level + increase > BEE_HONEY_LEVEL_MAX {
                increase -= 1;
            }
            next_honey_level += increase;
        }
    }

    BeeHiveReleaseResult::Release {
        honey_level: next_honey_level,
        bee_has_nectar,
        bee_crops_grown,
        copied_flower_pos: hive_has_saved_flower && !bee_has_saved_flower && copy_flower_roll_hits,
    }
}

