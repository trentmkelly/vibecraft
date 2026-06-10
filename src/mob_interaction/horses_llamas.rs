use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HorseState {
    pub flags: u8,
    pub temper: i32,
    pub max_temper: i32,
    pub baby: bool,
    pub alive: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HorseVariant {
    White,
    Creamy,
    Chestnut,
    Brown,
    Black,
    Gray,
    DarkBrown,
}

impl HorseVariant {
    pub fn by_id(id: i32) -> Self {
        match id.rem_euclid(HORSE_VARIANT_COUNT) {
            0 => Self::White,
            1 => Self::Creamy,
            2 => Self::Chestnut,
            3 => Self::Brown,
            4 => Self::Black,
            5 => Self::Gray,
            _ => Self::DarkBrown,
        }
    }

    pub fn id(self) -> i32 {
        match self {
            Self::White => 0,
            Self::Creamy => 1,
            Self::Chestnut => 2,
            Self::Brown => 3,
            Self::Black => 4,
            Self::Gray => 5,
            Self::DarkBrown => 6,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HorseMarkings {
    None,
    White,
    WhiteField,
    WhiteDots,
    BlackDots,
}

impl HorseMarkings {
    pub fn by_id(id: i32) -> Self {
        match id.rem_euclid(HORSE_MARKINGS_COUNT) {
            0 => Self::None,
            1 => Self::White,
            2 => Self::WhiteField,
            3 => Self::WhiteDots,
            _ => Self::BlackDots,
        }
    }

    pub fn id(self) -> i32 {
        match self {
            Self::None => 0,
            Self::White => 1,
            Self::WhiteField => 2,
            Self::WhiteDots => 3,
            Self::BlackDots => 4,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LlamaVariant {
    Creamy,
    White,
    Brown,
    Gray,
}

impl LlamaVariant {
    pub fn by_id(id: i32) -> Self {
        match id.clamp(0, 3) {
            0 => Self::Creamy,
            1 => Self::White,
            2 => Self::Brown,
            _ => Self::Gray,
        }
    }

    pub fn id(self) -> i32 {
        match self {
            Self::Creamy => 0,
            Self::White => 1,
            Self::Brown => 2,
            Self::Gray => 3,
        }
    }
}

impl HorseState {
    pub fn is_tamed(self) -> bool {
        self.flags & HORSE_FLAG_TAME != 0
    }

    pub fn can_use_saddle_slot(self) -> bool {
        self.alive && !self.baby && self.is_tamed()
    }

    pub fn modify_temper(mut self, amount: i32) -> Self {
        self.temper = (self.temper + amount).clamp(0, self.max_temper);
        self
    }
}

pub fn horse_type_variant(variant: HorseVariant, markings: HorseMarkings) -> i32 {
    variant.id() & 0xFF | (markings.id() << 8) & 0xFF00
}

pub fn horse_variant_from_type(type_variant: i32) -> HorseVariant {
    HorseVariant::by_id(type_variant & 0xFF)
}

pub fn horse_markings_from_type(type_variant: i32) -> HorseMarkings {
    HorseMarkings::by_id((type_variant & 0xFF00) >> 8)
}

pub fn horse_offspring_variant(
    first_parent: HorseVariant,
    second_parent: HorseVariant,
    select_skin_roll: i32,
    random_variant_roll: i32,
) -> HorseVariant {
    let roll = select_skin_roll.rem_euclid(9);
    if roll < 4 {
        first_parent
    } else if roll < 8 {
        second_parent
    } else {
        HorseVariant::by_id(random_variant_roll)
    }
}

pub fn horse_offspring_markings(
    first_parent: HorseMarkings,
    second_parent: HorseMarkings,
    select_marking_roll: i32,
    random_marking_roll: i32,
) -> HorseMarkings {
    let roll = select_marking_roll.rem_euclid(5);
    if roll < 2 {
        first_parent
    } else if roll < 4 {
        second_parent
    } else {
        HorseMarkings::by_id(random_marking_roll)
    }
}

pub fn chested_horse_inventory_columns(has_chest: bool) -> i32 {
    if has_chest {
        CHESTED_HORSE_INVENTORY_COLUMNS
    } else {
        0
    }
}

pub fn chested_horse_can_equip_chest(has_chest: bool, tamed: bool, baby: bool, item: &str) -> bool {
    !has_chest && tamed && !baby && item == "minecraft:chest"
}

pub fn llama_strength_from_spawn(max_strength_roll_hits: bool, strength_roll: i32) -> i32 {
    let max_strength = if max_strength_roll_hits {
        LLAMA_MAX_STRENGTH
    } else {
        LLAMA_COMMON_MAX_STRENGTH
    };
    1 + strength_roll.rem_euclid(max_strength)
}

pub fn llama_inventory_columns(has_chest: bool, strength: i32) -> i32 {
    if has_chest {
        strength.clamp(1, LLAMA_MAX_STRENGTH)
    } else {
        0
    }
}

pub fn llama_offspring_strength(
    first_parent_strength: i32,
    second_parent_strength: i32,
    strength_roll: i32,
    bonus_roll_hits: bool,
) -> i32 {
    let max_parent = first_parent_strength
        .max(second_parent_strength)
        .clamp(1, LLAMA_MAX_STRENGTH);
    let strength = 1 + strength_roll.rem_euclid(max_parent);
    if bonus_roll_hits {
        (strength + 1).clamp(1, LLAMA_MAX_STRENGTH)
    } else {
        strength
    }
}

pub fn llama_offspring_variant(
    first_parent: LlamaVariant,
    second_parent: LlamaVariant,
    choose_first_parent: bool,
) -> LlamaVariant {
    if choose_first_parent {
        first_parent
    } else {
        second_parent
    }
}
