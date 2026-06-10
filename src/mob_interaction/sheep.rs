#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DyeColorModel {
    White,
    Orange,
    Magenta,
    LightBlue,
    Yellow,
    Lime,
    Pink,
    Gray,
    LightGray,
    Cyan,
    Purple,
    Blue,
    Brown,
    Green,
    Red,
    Black,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SheepState {
    pub wool_data: u8,
    pub eat_animation_tick: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SheepInteraction {
    ShearServer,
    ConsumeClientOrNotReady,
    Delegate,
}

pub const SHEEP_EAT_ANIMATION_TICKS: i32 = 40;
pub const SHEEP_SHEARED_FLAG: u8 = 16;
pub const SHEEP_COLOR_MASK: u8 = 15;
pub const SHEEP_MAX_HEALTH: f32 = 8.0;
pub const SHEEP_MOVEMENT_SPEED: f32 = 0.23;
pub const SHEEP_PANIC_SPEED: f32 = 1.25;
pub const SHEEP_BREED_SPEED: f32 = 1.0;
pub const SHEEP_TEMPT_SPEED: f32 = 1.1;
pub const SHEEP_FOLLOW_PARENT_SPEED: f32 = 1.1;
pub const SHEEP_STROLL_SPEED: f32 = 1.0;
pub const SHEEP_LOOK_AT_PLAYER_DISTANCE: f32 = 6.0;
pub const SHEEP_ATE_AGE_UP_SECONDS: i32 = 60;

impl DyeColorModel {
    pub fn id(self) -> u8 {
        match self {
            Self::White => 0,
            Self::Orange => 1,
            Self::Magenta => 2,
            Self::LightBlue => 3,
            Self::Yellow => 4,
            Self::Lime => 5,
            Self::Pink => 6,
            Self::Gray => 7,
            Self::LightGray => 8,
            Self::Cyan => 9,
            Self::Purple => 10,
            Self::Blue => 11,
            Self::Brown => 12,
            Self::Green => 13,
            Self::Red => 14,
            Self::Black => 15,
        }
    }

    pub fn by_id(id: u8) -> Self {
        match id {
            1 => Self::Orange,
            2 => Self::Magenta,
            3 => Self::LightBlue,
            4 => Self::Yellow,
            5 => Self::Lime,
            6 => Self::Pink,
            7 => Self::Gray,
            8 => Self::LightGray,
            9 => Self::Cyan,
            10 => Self::Purple,
            11 => Self::Blue,
            12 => Self::Brown,
            13 => Self::Green,
            14 => Self::Red,
            15 => Self::Black,
            _ => Self::White,
        }
    }
}

impl SheepState {
    pub fn new() -> Self {
        Self {
            wool_data: 0,
            eat_animation_tick: 0,
        }
    }

    pub fn color(self) -> DyeColorModel {
        DyeColorModel::by_id(self.wool_data & SHEEP_COLOR_MASK)
    }

    pub fn set_color(&mut self, color: DyeColorModel) {
        self.wool_data = (self.wool_data & 0xF0) | (color.id() & SHEEP_COLOR_MASK);
    }

    pub fn is_sheared(self) -> bool {
        self.wool_data & SHEEP_SHEARED_FLAG != 0
    }

    pub fn set_sheared(&mut self, value: bool) {
        if value {
            self.wool_data |= SHEEP_SHEARED_FLAG;
        } else {
            self.wool_data &= !SHEEP_SHEARED_FLAG;
        }
    }

    pub fn ready_for_shearing(self, alive: bool, baby: bool) -> bool {
        alive && !self.is_sheared() && !baby
    }

    pub fn ate(&mut self, can_age_up: bool) -> Option<i32> {
        self.set_sheared(false);
        can_age_up.then_some(SHEEP_ATE_AGE_UP_SECONDS)
    }

    pub fn handle_entity_event(&mut self, event_id: u8) -> bool {
        if event_id == 10 {
            self.eat_animation_tick = SHEEP_EAT_ANIMATION_TICKS;
            true
        } else {
            false
        }
    }

    pub fn client_ai_step(&mut self) {
        self.eat_animation_tick = (self.eat_animation_tick - 1).max(0);
    }

    pub fn head_eat_position_scale(self, partial_tick: f32) -> f32 {
        if self.eat_animation_tick <= 0 {
            0.0
        } else if (4..=36).contains(&self.eat_animation_tick) {
            1.0
        } else if self.eat_animation_tick < 4 {
            (self.eat_animation_tick as f32 - partial_tick) / 4.0
        } else {
            -(self.eat_animation_tick as f32 - SHEEP_EAT_ANIMATION_TICKS as f32 - partial_tick)
                / 4.0
        }
    }

    pub fn head_eat_angle_scale(self, partial_tick: f32, x_rot_degrees: f32) -> f32 {
        if self.eat_animation_tick > 4 && self.eat_animation_tick <= 36 {
            let scale = (self.eat_animation_tick as f32 - 4.0 - partial_tick) / 32.0;
            std::f32::consts::PI / 5.0 + 0.21991149 * (scale * 28.7).sin()
        } else if self.eat_animation_tick > 0 {
            std::f32::consts::PI / 5.0
        } else {
            x_rot_degrees * std::f32::consts::PI / 180.0
        }
    }
}

pub fn sheep_interaction(
    item: &str,
    server_level: bool,
    ready_for_shearing: bool,
) -> SheepInteraction {
    if item == "minecraft:shears" {
        if server_level && ready_for_shearing {
            SheepInteraction::ShearServer
        } else {
            SheepInteraction::ConsumeClientOrNotReady
        }
    } else {
        SheepInteraction::Delegate
    }
}

pub fn sheep_spawn_color(
    warm_variant_biome: bool,
    cold_variant_biome: bool,
    primary_roll_0_to_99: i32,
    common_roll_0_to_499: i32,
) -> DyeColorModel {
    let primary = primary_roll_0_to_99.clamp(0, 99);
    let common = common_roll_0_to_499.clamp(0, 499);
    let common_color = if warm_variant_biome {
        DyeColorModel::Brown
    } else if cold_variant_biome {
        DyeColorModel::Black
    } else {
        DyeColorModel::White
    };
    if warm_variant_biome {
        match primary {
            0..=4 => DyeColorModel::Gray,
            5..=9 => DyeColorModel::LightGray,
            10..=14 => DyeColorModel::White,
            15..=17 => DyeColorModel::Black,
            _ => {
                if common == 499 {
                    DyeColorModel::Pink
                } else {
                    common_color
                }
            }
        }
    } else if cold_variant_biome {
        match primary {
            0..=4 => DyeColorModel::LightGray,
            5..=9 => DyeColorModel::Gray,
            10..=14 => DyeColorModel::White,
            15..=17 => DyeColorModel::Brown,
            _ => {
                if common == 499 {
                    DyeColorModel::Pink
                } else {
                    common_color
                }
            }
        }
    } else {
        match primary {
            0..=4 => DyeColorModel::Black,
            5..=9 => DyeColorModel::Gray,
            10..=14 => DyeColorModel::LightGray,
            15..=17 => DyeColorModel::Brown,
            _ => {
                if common == 499 {
                    DyeColorModel::Pink
                } else {
                    common_color
                }
            }
        }
    }
}

pub fn sheep_offspring_color(
    recipe_mixed_color: Option<DyeColorModel>,
    first_parent_color: DyeColorModel,
    second_parent_color: DyeColorModel,
    choose_first_parent: bool,
) -> DyeColorModel {
    recipe_mixed_color.unwrap_or(if choose_first_parent {
        first_parent_color
    } else {
        second_parent_color
    })
}
