#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PufferfishState {
    pub puff_state: u8,
    pub inflate_counter: i32,
    pub deflate_timer: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PufferfishContactEffect {
    pub damage: i32,
    pub poison_effect: &'static str,
    pub poison_duration_ticks: i32,
    pub poison_amplifier: u8,
}

impl PufferfishState {
    pub const SMALL: u8 = 0;
    pub const MID: u8 = 1;
    pub const FULL: u8 = 2;

    pub fn new() -> Self {
        Self {
            puff_state: Self::SMALL,
            inflate_counter: 0,
            deflate_timer: 0,
        }
    }

    pub fn start_inflating(&mut self) {
        self.inflate_counter = 1;
        self.deflate_timer = 0;
    }

    pub fn stop_inflating(&mut self) {
        self.inflate_counter = 0;
    }

    pub fn tick(&mut self, alive: bool, effective_ai: bool) {
        if !alive || !effective_ai {
            return;
        }

        if self.inflate_counter > 0 {
            if self.puff_state == Self::SMALL {
                self.puff_state = Self::MID;
            } else if self.inflate_counter > 40 && self.puff_state == Self::MID {
                self.puff_state = Self::FULL;
            }
            self.inflate_counter += 1;
        } else if self.puff_state != Self::SMALL {
            if self.deflate_timer > 60 && self.puff_state == Self::FULL {
                self.puff_state = Self::MID;
            } else if self.deflate_timer > 100 && self.puff_state == Self::MID {
                self.puff_state = Self::SMALL;
            }
            self.deflate_timer += 1;
        }
    }

    pub fn contact_effect(
        self,
        target_alive: bool,
        target_scary: bool,
    ) -> Option<PufferfishContactEffect> {
        if !target_alive || !target_scary || self.puff_state == Self::SMALL {
            return None;
        }

        Some(PufferfishContactEffect {
            damage: 1 + i32::from(self.puff_state),
            poison_effect: "minecraft:poison",
            poison_duration_ticks: 60 * i32::from(self.puff_state),
            poison_amplifier: 0,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SalmonVariantModel {
    pub name: &'static str,
    pub id: i32,
    pub bounding_box_scale: f32,
    pub spawn_weight: i32,
}

pub const SALMON_VARIANTS: &[SalmonVariantModel] = &[
    SalmonVariantModel {
        name: "small",
        id: 0,
        bounding_box_scale: 0.5,
        spawn_weight: 30,
    },
    SalmonVariantModel {
        name: "medium",
        id: 1,
        bounding_box_scale: 1.0,
        spawn_weight: 50,
    },
    SalmonVariantModel {
        name: "large",
        id: 2,
        bounding_box_scale: 1.5,
        spawn_weight: 15,
    },
];

pub const DEFAULT_SALMON_VARIANT_ID: i32 = 1;

pub fn salmon_variant_by_id(id: i32) -> SalmonVariantModel {
    let clamped = id.clamp(0, (SALMON_VARIANTS.len() - 1) as i32);
    SALMON_VARIANTS[clamped as usize]
}

pub fn salmon_variant_by_name(name: &str) -> Option<SalmonVariantModel> {
    SALMON_VARIANTS
        .iter()
        .copied()
        .find(|variant| variant.name == name)
}

pub fn salmon_spawn_weight_total() -> i32 {
    SALMON_VARIANTS
        .iter()
        .map(|variant| variant.spawn_weight)
        .sum()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TropicalFishBase {
    Small = 0,
    Large = 1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TropicalFishPatternModel {
    pub name: &'static str,
    pub base: TropicalFishBase,
    pub index: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TropicalFishVariantModel {
    pub pattern: TropicalFishPatternModel,
    pub base_color_id: i32,
    pub pattern_color_id: i32,
}

pub const TROPICAL_FISH_PATTERNS: &[TropicalFishPatternModel] = &[
    TropicalFishPatternModel {
        name: "kob",
        base: TropicalFishBase::Small,
        index: 0,
    },
    TropicalFishPatternModel {
        name: "sunstreak",
        base: TropicalFishBase::Small,
        index: 1,
    },
    TropicalFishPatternModel {
        name: "snooper",
        base: TropicalFishBase::Small,
        index: 2,
    },
    TropicalFishPatternModel {
        name: "dasher",
        base: TropicalFishBase::Small,
        index: 3,
    },
    TropicalFishPatternModel {
        name: "brinely",
        base: TropicalFishBase::Small,
        index: 4,
    },
    TropicalFishPatternModel {
        name: "spotty",
        base: TropicalFishBase::Small,
        index: 5,
    },
    TropicalFishPatternModel {
        name: "flopper",
        base: TropicalFishBase::Large,
        index: 0,
    },
    TropicalFishPatternModel {
        name: "stripey",
        base: TropicalFishBase::Large,
        index: 1,
    },
    TropicalFishPatternModel {
        name: "glitter",
        base: TropicalFishBase::Large,
        index: 2,
    },
    TropicalFishPatternModel {
        name: "blockfish",
        base: TropicalFishBase::Large,
        index: 3,
    },
    TropicalFishPatternModel {
        name: "betty",
        base: TropicalFishBase::Large,
        index: 4,
    },
    TropicalFishPatternModel {
        name: "clayfish",
        base: TropicalFishBase::Large,
        index: 5,
    },
];

pub const DEFAULT_TROPICAL_FISH_VARIANT_PACKED_ID: i32 = 0;

pub fn tropical_fish_pattern_packed_id(pattern: TropicalFishPatternModel) -> i32 {
    pattern.base as i32 | pattern.index << 8
}

pub fn tropical_fish_pack_variant(
    pattern: TropicalFishPatternModel,
    base_color_id: i32,
    pattern_color_id: i32,
) -> i32 {
    tropical_fish_pattern_packed_id(pattern) & 65_535
        | (base_color_id & 0xff) << 16
        | (pattern_color_id & 0xff) << 24
}

pub fn tropical_fish_base_color_id(packed_variant: i32) -> i32 {
    packed_variant >> 16 & 0xff
}

pub fn tropical_fish_pattern_color_id(packed_variant: i32) -> i32 {
    packed_variant >> 24 & 0xff
}

pub fn tropical_fish_pattern_by_packed_id(packed_id: i32) -> TropicalFishPatternModel {
    TROPICAL_FISH_PATTERNS
        .iter()
        .copied()
        .find(|pattern| tropical_fish_pattern_packed_id(*pattern) == packed_id)
        .unwrap_or(TROPICAL_FISH_PATTERNS[0])
}

pub fn tropical_fish_pattern_from_variant(packed_variant: i32) -> TropicalFishPatternModel {
    tropical_fish_pattern_by_packed_id(packed_variant & 65_535)
}

pub fn tropical_fish_common_variants() -> Vec<TropicalFishVariantModel> {
    const ORANGE: i32 = 1;
    const LIGHT_BLUE: i32 = 3;
    const YELLOW: i32 = 4;
    const LIME: i32 = 5;
    const PINK: i32 = 6;
    const GRAY: i32 = 7;
    const CYAN: i32 = 9;
    const PURPLE: i32 = 10;
    const BLUE: i32 = 11;
    const RED: i32 = 14;
    const WHITE: i32 = 0;

    [
        ("stripey", ORANGE, GRAY),
        ("flopper", GRAY, GRAY),
        ("flopper", GRAY, BLUE),
        ("clayfish", WHITE, GRAY),
        ("sunstreak", BLUE, GRAY),
        ("kob", ORANGE, WHITE),
        ("spotty", PINK, LIGHT_BLUE),
        ("blockfish", PURPLE, YELLOW),
        ("clayfish", WHITE, RED),
        ("spotty", WHITE, YELLOW),
        ("glitter", WHITE, GRAY),
        ("clayfish", WHITE, ORANGE),
        ("dasher", CYAN, PINK),
        ("brinely", LIME, LIGHT_BLUE),
        ("betty", RED, WHITE),
        ("snooper", GRAY, RED),
        ("blockfish", RED, WHITE),
        ("flopper", WHITE, YELLOW),
        ("kob", RED, WHITE),
        ("sunstreak", GRAY, WHITE),
        ("dasher", CYAN, YELLOW),
        ("flopper", YELLOW, YELLOW),
    ]
    .into_iter()
    .filter_map(|(name, base_color_id, pattern_color_id)| {
        TROPICAL_FISH_PATTERNS
            .iter()
            .copied()
            .find(|pattern| pattern.name == name)
            .map(|pattern| TropicalFishVariantModel {
                pattern,
                base_color_id,
                pattern_color_id,
            })
    })
    .collect()
}
