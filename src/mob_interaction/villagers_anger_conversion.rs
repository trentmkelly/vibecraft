use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VillagerTradeResult {
    LazyLoadOffers,
    IncreaseUsesRewardXpTriggerAdvancement,
    StopTrading,
}

pub fn villager_slot_index(raw_slot: i32) -> Option<usize> {
    let index = raw_slot - VILLAGER_INVENTORY_SLOT_OFFSET;
    (index >= 0 && (index as usize) < VILLAGER_INVENTORY_SIZE).then_some(index as usize)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AngerState {
    pub anger_end_time: i64,
    pub target_present: bool,
}

impl AngerState {
    pub fn is_angry(self, game_time: i64) -> bool {
        self.anger_end_time > 0 && self.anger_end_time - game_time > 0
    }

    pub fn set_time_to_remain_angry(mut self, game_time: i64, remaining_time: i64) -> Self {
        self.anger_end_time = game_time + remaining_time;
        self
    }

    pub fn stop_being_angry(mut self) -> Self {
        self.anger_end_time = NO_ANGER_END_TIME;
        self.target_present = false;
        self
    }
}

pub fn should_stop_anger_for_player(creative: bool, spectator: bool, peaceful: bool) -> bool {
    creative || spectator || peaceful
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConversionTypeModel {
    Single,
    SplitOnDeath,
}

impl ConversionTypeModel {
    pub fn discard_after_conversion(self) -> bool {
        matches!(self, Self::Single)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConversionParamsModel {
    pub conversion_type: ConversionTypeModel,
    pub keep_equipment: bool,
    pub preserve_can_pick_up_loot: bool,
    pub team_present: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConversionCopyPlan {
    pub copy_position_motion_passenger_vehicle: bool,
    pub keep_equipment: bool,
    pub copy_effects_absorption_age_anger_and_flags: bool,
    pub preserve_can_pick_up_loot: bool,
    pub move_scoreboard_team: bool,
    pub discard_original: bool,
}

pub fn conversion_copy_plan(params: ConversionParamsModel) -> ConversionCopyPlan {
    ConversionCopyPlan {
        copy_position_motion_passenger_vehicle: params.conversion_type
            == ConversionTypeModel::Single,
        keep_equipment: params.keep_equipment,
        copy_effects_absorption_age_anger_and_flags: true,
        preserve_can_pick_up_loot: params.preserve_can_pick_up_loot,
        move_scoreboard_team: params.team_present,
        discard_original: params.conversion_type.discard_after_conversion(),
    }
}

fn set_flag(flags: &mut u8, flag: u8, value: bool) {
    if value {
        *flags |= flag;
    } else {
        *flags &= !flag;
    }
}
