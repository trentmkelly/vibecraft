use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GoatState {
    pub is_screaming: bool,
    pub has_left_horn: bool,
    pub has_right_horn: bool,
    pub lower_head_tick: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GoatHornDrop {
    None,
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GoatInteraction {
    Milk,
    Delegate,
}

pub const GOAT_SCREAMING_CHANCE_DENOMINATOR: i32 = 50;
pub const GOAT_INITIAL_MISSING_HORN_CHANCE_DENOMINATOR: i32 = 10;
pub const GOAT_RAM_PREPARE_TIME: i32 = 20;
pub const GOAT_RAM_MIN_DISTANCE: i32 = 4;
pub const GOAT_RAM_MAX_DISTANCE: i32 = 7;
pub const GOAT_TIME_BETWEEN_RAMS_MIN: i32 = 600;
pub const GOAT_TIME_BETWEEN_RAMS_MAX: i32 = 6000;
pub const GOAT_TIME_BETWEEN_RAMS_SCREAMER_MIN: i32 = 100;
pub const GOAT_TIME_BETWEEN_RAMS_SCREAMER_MAX: i32 = 300;
pub const GOAT_TIME_BETWEEN_LONG_JUMPS_MIN: i32 = 600;
pub const GOAT_TIME_BETWEEN_LONG_JUMPS_MAX: i32 = 1200;
pub const GOAT_MAX_LONG_JUMP_HEIGHT: i32 = 5;
pub const GOAT_MAX_LONG_JUMP_WIDTH: i32 = 5;
pub const GOAT_MAX_JUMP_VELOCITY_MULTIPLIER: f32 = 3.5714288;
pub const GOAT_PREPARE_RAM_SPEED_MULTIPLIER: f32 = 1.25;
pub const GOAT_RAMMING_SPEED_MULTIPLIER: f32 = 3.0;
pub const GOAT_ADULT_RAM_KNOCKBACK_FORCE: f32 = 2.5;
pub const GOAT_BABY_RAM_KNOCKBACK_FORCE: f32 = 1.0;
pub const GOAT_MAX_ADULT_RAMMING_X_HEAD_ROT_DEGREES: f32 = 30.0;
pub const GOAT_MAX_BABY_RAMMING_X_HEAD_ROT_DEGREES: f32 = 52.5;
pub const GOAT_MAX_LOWER_HEAD_TICK: i32 = 20;
pub const GOAT_LONG_JUMPING_WIDTH: f32 = 0.9 * 0.7;
pub const GOAT_LONG_JUMPING_HEIGHT: f32 = 1.3 * 0.7;

impl GoatState {
    pub fn new() -> Self {
        Self {
            is_screaming: false,
            has_left_horn: true,
            has_right_horn: true,
            lower_head_tick: 0,
        }
    }

    pub fn finalize_spawn(
        screaming_roll_zero_of_50: bool,
        adult: bool,
        missing_horn_roll_zero_of_10: bool,
        remove_left_horn: bool,
    ) -> Self {
        let mut state = Self::new();
        state.is_screaming = screaming_roll_zero_of_50;
        if adult && missing_horn_roll_zero_of_10 {
            if remove_left_horn {
                state.has_left_horn = false;
            } else {
                state.has_right_horn = false;
            }
        }
        state
    }

    pub fn drop_horn(&mut self, baby: bool, choose_left_when_both_present: bool) -> GoatHornDrop {
        if baby {
            return GoatHornDrop::None;
        }

        let horn_to_drop = match (self.has_left_horn, self.has_right_horn) {
            (false, false) => GoatHornDrop::None,
            (true, false) => GoatHornDrop::Left,
            (false, true) => GoatHornDrop::Right,
            (true, true) => {
                if choose_left_when_both_present {
                    GoatHornDrop::Left
                } else {
                    GoatHornDrop::Right
                }
            }
        };

        match horn_to_drop {
            GoatHornDrop::Left => self.has_left_horn = false,
            GoatHornDrop::Right => self.has_right_horn = false,
            GoatHornDrop::None => {}
        }
        horn_to_drop
    }

    pub fn tick_lower_head(&mut self, lowering_head: bool) {
        if lowering_head {
            self.lower_head_tick += 1;
        } else {
            self.lower_head_tick -= 2;
        }
        self.lower_head_tick = self.lower_head_tick.clamp(0, GOAT_MAX_LOWER_HEAD_TICK);
    }

    pub fn ramming_x_head_rot_radians(self, baby: bool) -> f32 {
        let max_rotation = if baby {
            GOAT_MAX_BABY_RAMMING_X_HEAD_ROT_DEGREES
        } else {
            GOAT_MAX_ADULT_RAMMING_X_HEAD_ROT_DEGREES
        };
        self.lower_head_tick as f32 / GOAT_MAX_LOWER_HEAD_TICK as f32
            * max_rotation
            * std::f32::consts::PI
            / 180.0
    }

    pub fn ram_cooldown_range(self) -> (i32, i32) {
        if self.is_screaming {
            (
                GOAT_TIME_BETWEEN_RAMS_SCREAMER_MIN,
                GOAT_TIME_BETWEEN_RAMS_SCREAMER_MAX,
            )
        } else {
            (GOAT_TIME_BETWEEN_RAMS_MIN, GOAT_TIME_BETWEEN_RAMS_MAX)
        }
    }
}

pub fn goat_interaction(item: &str, baby: bool) -> GoatInteraction {
    if item == "minecraft:bucket" && !baby {
        GoatInteraction::Milk
    } else {
        GoatInteraction::Delegate
    }
}

pub fn goat_ram_knockback_force(baby: bool) -> f32 {
    if baby {
        GOAT_BABY_RAM_KNOCKBACK_FORCE
    } else {
        GOAT_ADULT_RAM_KNOCKBACK_FORCE
    }
}

