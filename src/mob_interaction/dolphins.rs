use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DolphinState {
    pub got_fish: bool,
    pub moistness: i32,
    pub air_supply: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DolphinFeedResult {
    AgeUp { seconds: i32 },
    SetGotFish,
    NotFish,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DolphinLandTickOutcome {
    pub dry_out_damage: Option<f32>,
    pub jump_delta_y: Option<f64>,
    pub needs_sync: bool,
}

pub const DOLPHIN_TOTAL_AIR_SUPPLY: i32 = 4800;
pub const DOLPHIN_TOTAL_MOISTNESS_LEVEL: i32 = 2400;
pub const DOLPHIN_TREASURE_MIN_AIR_SUPPLY: i32 = 100;
pub const DOLPHIN_TREASURE_SEARCH_RADIUS: i32 = 50;
pub const DOLPHIN_TREASURE_STOP_DISTANCE: f64 = 4.0;
pub const DOLPHIN_SWIM_WITH_PLAYER_RANGE: f64 = 10.0;
pub const DOLPHIN_SWIM_WITH_PLAYER_CONTINUE_DISTANCE_SQUARED: f64 = 256.0;
pub const DOLPHIN_GRACE_DURATION_TICKS: i32 = 100;
pub const DOLPHIN_GRACE_REFRESH_RANDOM_BOUND: i32 = 6;
pub const DOLPHIN_BABY_SCALE: f32 = 0.65;
pub const DOLPHIN_DRY_OUT_DAMAGE: f32 = 1.0;

impl DolphinState {
    pub fn new() -> Self {
        Self {
            got_fish: false,
            moistness: DOLPHIN_TOTAL_MOISTNESS_LEVEL,
            air_supply: DOLPHIN_TOTAL_AIR_SUPPLY,
        }
    }

    pub fn tick_moistness(
        &mut self,
        no_ai: bool,
        in_water_or_rain: bool,
        on_ground: bool,
    ) -> DolphinLandTickOutcome {
        if no_ai {
            self.air_supply = DOLPHIN_TOTAL_AIR_SUPPLY;
            return DolphinLandTickOutcome {
                dry_out_damage: None,
                jump_delta_y: None,
                needs_sync: false,
            };
        }

        if in_water_or_rain {
            self.moistness = DOLPHIN_TOTAL_MOISTNESS_LEVEL;
            return DolphinLandTickOutcome {
                dry_out_damage: None,
                jump_delta_y: None,
                needs_sync: false,
            };
        }

        self.moistness -= 1;
        DolphinLandTickOutcome {
            dry_out_damage: (self.moistness <= 0).then_some(DOLPHIN_DRY_OUT_DAMAGE),
            jump_delta_y: on_ground.then_some(0.5),
            needs_sync: on_ground,
        }
    }

    pub fn can_start_treasure_goal(self) -> bool {
        self.got_fish && self.air_supply >= DOLPHIN_TREASURE_MIN_AIR_SUPPLY
    }

    pub fn should_clear_got_fish_on_treasure_stop(
        self,
        treasure_missing: bool,
        within_stop_distance: bool,
        stuck: bool,
    ) -> bool {
        treasure_missing || within_stop_distance || stuck
    }
}

pub fn dolphin_feed_result(
    item_is_fish: bool,
    can_age_up: bool,
    ticks_until_adult: i32,
) -> DolphinFeedResult {
    if !item_is_fish {
        DolphinFeedResult::NotFish
    } else if can_age_up {
        DolphinFeedResult::AgeUp {
            seconds: speed_up_seconds_when_feeding(ticks_until_adult),
        }
    } else {
        DolphinFeedResult::SetGotFish
    }
}

pub fn dolphin_grace_refresh_ticks(player_swimming: bool, random_roll: i32) -> Option<i32> {
    (player_swimming && random_roll.rem_euclid(DOLPHIN_GRACE_REFRESH_RANDOM_BOUND) == 0)
        .then_some(DOLPHIN_GRACE_DURATION_TICKS)
}
