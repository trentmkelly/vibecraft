#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CrossbowAttackHand {
    MainHand,
    OffHand,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CrossbowAttackInput {
    pub main_hand_crossbow: bool,
    pub off_hand_crossbow: bool,
    pub crossbow_power: f32,
    pub difficulty_id: i32,
    pub target_present: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CrossbowAttackPlan {
    pub selected_hand: CrossbowAttackHand,
    pub perform_shooting: Option<CrossbowShootingPlan>,
    pub on_attack_performed: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CrossbowShootingPlan {
    pub hand: CrossbowAttackHand,
    pub power: f32,
    pub inaccuracy: i32,
    pub target_present: bool,
}

pub const CROSSBOW_ATTACK_BASE_INACCURACY: i32 = 14;
pub const CROSSBOW_ATTACK_DIFFICULTY_INACCURACY_STEP: i32 = 4;

pub fn crossbow_attack_selected_hand(main_hand_crossbow: bool) -> CrossbowAttackHand {
    if main_hand_crossbow {
        CrossbowAttackHand::MainHand
    } else {
        CrossbowAttackHand::OffHand
    }
}

pub fn crossbow_attack_inaccuracy(difficulty_id: i32) -> i32 {
    CROSSBOW_ATTACK_BASE_INACCURACY - difficulty_id * CROSSBOW_ATTACK_DIFFICULTY_INACCURACY_STEP
}

pub fn crossbow_attack_mob_perform_crossbow_attack(input: CrossbowAttackInput) -> CrossbowAttackPlan {
    let selected_hand = crossbow_attack_selected_hand(input.main_hand_crossbow);
    let selected_item_is_crossbow = match selected_hand {
        CrossbowAttackHand::MainHand => input.main_hand_crossbow,
        CrossbowAttackHand::OffHand => input.off_hand_crossbow,
    };

    CrossbowAttackPlan {
        selected_hand,
        perform_shooting: selected_item_is_crossbow.then_some(CrossbowShootingPlan {
            hand: selected_hand,
            power: input.crossbow_power,
            inaccuracy: crossbow_attack_inaccuracy(input.difficulty_id),
            target_present: input.target_present,
        }),
        on_attack_performed: true,
    }
}
