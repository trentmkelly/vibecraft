use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PandaGene {
    Normal,
    Lazy,
    Worried,
    Playful,
    Brown,
    Weak,
    Aggressive,
}

impl PandaGene {
    pub fn id(self) -> i32 {
        match self {
            PandaGene::Normal => 0,
            PandaGene::Lazy => 1,
            PandaGene::Worried => 2,
            PandaGene::Playful => 3,
            PandaGene::Brown => 4,
            PandaGene::Weak => 5,
            PandaGene::Aggressive => 6,
        }
    }

    pub fn serialized_name(self) -> &'static str {
        match self {
            PandaGene::Normal => "normal",
            PandaGene::Lazy => "lazy",
            PandaGene::Worried => "worried",
            PandaGene::Playful => "playful",
            PandaGene::Brown => "brown",
            PandaGene::Weak => "weak",
            PandaGene::Aggressive => "aggressive",
        }
    }

    pub fn is_recessive(self) -> bool {
        matches!(self, PandaGene::Brown | PandaGene::Weak)
    }
}

pub fn panda_gene_by_id(id: i32) -> PandaGene {
    match id {
        1 => PandaGene::Lazy,
        2 => PandaGene::Worried,
        3 => PandaGene::Playful,
        4 => PandaGene::Brown,
        5 => PandaGene::Weak,
        6 => PandaGene::Aggressive,
        _ => PandaGene::Normal,
    }
}

pub fn panda_random_gene(next_int_16: i32) -> PandaGene {
    match next_int_16.rem_euclid(16) {
        0 => PandaGene::Lazy,
        1 => PandaGene::Worried,
        2 => PandaGene::Playful,
        4 => PandaGene::Aggressive,
        3 | 5 | 6 | 7 | 8 => PandaGene::Weak,
        9 | 10 => PandaGene::Brown,
        _ => PandaGene::Normal,
    }
}

pub fn panda_variant_from_genes(main: PandaGene, hidden: PandaGene) -> PandaGene {
    if main.is_recessive() {
        if main == hidden {
            main
        } else {
            PandaGene::Normal
        }
    } else {
        main
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PandaState {
    pub main_gene: PandaGene,
    pub hidden_gene: PandaGene,
    flags: u8,
    pub unhappy_counter: i32,
    pub sneeze_counter: i32,
    pub eat_counter: i32,
    pub roll_counter: i32,
}

impl PandaState {
    pub fn new() -> Self {
        Self {
            main_gene: PandaGene::Normal,
            hidden_gene: PandaGene::Normal,
            flags: 0,
            unhappy_counter: 0,
            sneeze_counter: 0,
            eat_counter: 0,
            roll_counter: 0,
        }
    }

    pub fn variant(self) -> PandaGene {
        panda_variant_from_genes(self.main_gene, self.hidden_gene)
    }

    pub fn get_flag(self, flag: u8) -> bool {
        (self.flags & flag) != 0
    }

    pub fn set_flag(&mut self, flag: u8, value: bool) {
        if value {
            self.flags |= flag;
        } else {
            self.flags &= !flag;
        }
    }

    pub fn is_sneezing(self) -> bool {
        self.get_flag(PANDA_FLAG_SNEEZE)
    }

    pub fn is_rolling(self) -> bool {
        self.get_flag(PANDA_FLAG_ROLL)
    }

    pub fn is_sitting(self) -> bool {
        self.get_flag(PANDA_FLAG_SIT)
    }

    pub fn is_on_back(self) -> bool {
        self.get_flag(PANDA_FLAG_ON_BACK)
    }

    pub fn is_eating(self) -> bool {
        self.eat_counter > 0
    }

    pub fn can_perform_action(self, scared: bool) -> bool {
        !self.is_on_back()
            && !scared
            && !self.is_eating()
            && !self.is_rolling()
            && !self.is_sitting()
    }

    pub fn set_sneezing(&mut self, value: bool) {
        self.set_flag(PANDA_FLAG_SNEEZE, value);
        if !value {
            self.sneeze_counter = 0;
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PandaAttributes {
    pub movement_speed: f32,
    pub attack_damage: f32,
    pub max_health_override: Option<f32>,
}

pub fn panda_attributes_for_variant(variant: PandaGene) -> PandaAttributes {
    PandaAttributes {
        movement_speed: if variant == PandaGene::Lazy {
            PANDA_LAZY_MOVEMENT_SPEED
        } else {
            PANDA_BASE_MOVEMENT_SPEED
        },
        attack_damage: PANDA_ATTACK_DAMAGE,
        max_health_override: (variant == PandaGene::Weak).then_some(PANDA_WEAK_MAX_HEALTH),
    }
}

pub fn panda_food_item(item: &'static str) -> bool {
    item == "minecraft:bamboo"
}

pub fn panda_eats_from_ground_item(item: &'static str) -> bool {
    matches!(item, "minecraft:bamboo" | "minecraft:cake")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PandaInteractResult {
    Pass,
    SuccessServer {
        consumed: i32,
        sit: bool,
        eat: bool,
        held_item: Option<&'static str>,
    },
    Success {
        on_back: bool,
    },
}

pub fn panda_interact_plan(
    item: &'static str,
    scared: bool,
    on_back: bool,
    target_present: bool,
    can_age_up: bool,
    baby: bool,
    age: i32,
    can_fall_in_love: bool,
    sitting: bool,
    in_water: bool,
    current_held_item: Option<&'static str>,
    player_infinite_materials: bool,
) -> PandaInteractResult {
    if scared {
        return PandaInteractResult::Pass;
    }
    if on_back {
        return PandaInteractResult::Success { on_back: false };
    }
    if !panda_food_item(item) {
        return PandaInteractResult::Pass;
    }
    if target_present {
        return PandaInteractResult::SuccessServer {
            consumed: 0,
            sit: sitting,
            eat: false,
            held_item: current_held_item,
        };
    }
    if can_age_up {
        return PandaInteractResult::SuccessServer {
            consumed: 1,
            sit: sitting,
            eat: false,
            held_item: current_held_item,
        };
    }
    if baby {
        return PandaInteractResult::Pass;
    }
    if age == 0 && can_fall_in_love {
        return PandaInteractResult::SuccessServer {
            consumed: 1,
            sit: sitting,
            eat: false,
            held_item: current_held_item,
        };
    }
    if sitting || in_water {
        return PandaInteractResult::Pass;
    }
    let _drops_previous = current_held_item.is_some() && !player_infinite_materials;
    PandaInteractResult::SuccessServer {
        consumed: 1,
        sit: true,
        eat: true,
        held_item: Some(item),
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PandaRollStep {
    pub rolling_after_step: bool,
    pub counter: i32,
    pub delta: (f32, f32, f32),
}

pub fn panda_roll_step(
    roll_counter: i32,
    baby: bool,
    yaw_degrees: f32,
    current_delta: (f32, f32, f32),
    roll_delta: (f32, f32, f32),
    on_ground: bool,
) -> PandaRollStep {
    let counter = roll_counter + 1;
    if counter > PANDA_TOTAL_ROLL_STEPS {
        return PandaRollStep {
            rolling_after_step: false,
            counter,
            delta: current_delta,
        };
    }
    if counter == 1 {
        let angle = yaw_degrees.to_radians();
        let multiplier = if baby { 0.1 } else { 0.2 };
        let x = current_delta.0 + -angle.sin() * multiplier;
        let z = current_delta.2 + angle.cos() * multiplier;
        PandaRollStep {
            rolling_after_step: true,
            counter,
            delta: (x, 0.27, z),
        }
    } else if matches!(counter, 7 | 15 | 23) {
        PandaRollStep {
            rolling_after_step: true,
            counter,
            delta: (0.0, if on_ground { 0.27 } else { current_delta.1 }, 0.0),
        }
    } else {
        PandaRollStep {
            rolling_after_step: true,
            counter,
            delta: (roll_delta.0, current_delta.1, roll_delta.2),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PandaSneezeTick {
    None,
    PreSneezeSound,
    Finish {
        sound: &'static str,
        particle: &'static str,
        loot_table: &'static str,
    },
}

pub fn panda_sneeze_tick(state: &mut PandaState, mob_drops: bool) -> PandaSneezeTick {
    if !state.is_sneezing() {
        return PandaSneezeTick::None;
    }
    state.sneeze_counter += 1;
    if state.sneeze_counter > 20 {
        state.set_sneezing(false);
        PandaSneezeTick::Finish {
            sound: "minecraft:entity.panda.sneeze",
            particle: "minecraft:sneeze",
            loot_table: if mob_drops {
                "minecraft:gameplay/panda_sneeze"
            } else {
                ""
            },
        }
    } else if state.sneeze_counter == 1 {
        PandaSneezeTick::PreSneezeSound
    } else {
        PandaSneezeTick::None
    }
}

pub fn panda_sneeze_goal_can_use(
    baby: bool,
    can_perform_action: bool,
    weak: bool,
    weak_roll_one_of_500: bool,
    normal_roll_one_of_6000: bool,
) -> bool {
    baby && can_perform_action && ((weak && weak_roll_one_of_500) || normal_roll_one_of_6000)
}

