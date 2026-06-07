use super::container_decorative::{
    ContainerOpenersCounterEffect, ContainerOpenersCounterModel, ContainerUserOpenState,
};
use super::*;

#[derive(Debug, Clone, PartialEq)]
pub struct EnderChestBlockEntityModel {
    pub world_position: BlockPos,
    pub remove: bool,
    pub chest_lid: ChestLidController,
    pub openers_counter: ContainerOpenersCounterModel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EnderChestBlockEvent {
    pub pos: BlockPos,
    pub block: &'static str,
    pub action: i32,
    pub param: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EnderChestOpenEffect {
    pub counter_effect: ContainerOpenersCounterEffect,
    pub sound_event: Option<&'static str>,
    pub block_event: EnderChestBlockEvent,
}

impl EnderChestBlockEntityModel {
    pub const BLOCK_ID: &'static str = "minecraft:ender_chest";
    pub const OPEN_SOUND: &'static str = "minecraft:block.ender_chest.open";
    pub const CLOSE_SOUND: &'static str = "minecraft:block.ender_chest.close";
    pub const MENU_TYPE: &'static str = "generic_9x3";
    pub const CONTAINER_TITLE: &'static str = "container.enderchest";
    pub const BLOCK_EVENT_ACTION: i32 = 1;
    pub const OPEN_STAT: &'static str = "minecraft:open_enderchest";

    pub const fn new(world_position: BlockPos) -> Self {
        Self {
            world_position,
            remove: false,
            chest_lid: ChestLidController::new(),
            openers_counter: ContainerOpenersCounterModel {
                open_count: 0,
                max_interaction_range: 0.0,
            },
        }
    }

    pub fn lid_animate_tick(&mut self) {
        self.chest_lid.tick_lid();
    }

    pub fn trigger_event(&mut self, action: i32, param: i32) -> bool {
        if action != Self::BLOCK_EVENT_ACTION {
            return false;
        }
        self.chest_lid.should_be_open(param > 0);
        true
    }

    pub fn start_open(
        &mut self,
        spectator: bool,
        interaction_range: f64,
    ) -> Option<EnderChestOpenEffect> {
        if self.remove || spectator {
            return None;
        }
        let effect = self.openers_counter.increment_openers(interaction_range);
        Some(self.effect_for_counter(effect))
    }

    pub fn stop_open(&mut self, spectator: bool) -> Option<EnderChestOpenEffect> {
        if self.remove || spectator {
            return None;
        }
        let effect = self.openers_counter.decrement_openers();
        Some(self.effect_for_counter(effect))
    }

    pub fn recheck_open(
        &mut self,
        users: &[ContainerUserOpenState],
    ) -> Option<EnderChestOpenEffect> {
        if self.remove {
            return None;
        }
        let effect = self.openers_counter.recheck_openers(users);
        Some(self.effect_for_counter(effect))
    }

    pub fn still_valid(&self, same_block_entity: bool, player_distance_sqr: f64) -> bool {
        same_block_entity && player_distance_sqr <= 64.0
    }

    pub fn get_openness(&self, partial_tick: f32) -> f32 {
        self.chest_lid.get_openness(partial_tick)
    }

    pub fn open_menu(
        &mut self,
        container_id: i32,
        ender_chest_slots: Vec<Option<PotItemStack>>,
        blocked_above: bool,
    ) -> Option<BlockEntityMenuOpen> {
        if blocked_above {
            return None;
        }
        Some(open_ender_chest_menu(container_id, ender_chest_slots))
    }

    fn effect_for_counter(
        &self,
        counter_effect: ContainerOpenersCounterEffect,
    ) -> EnderChestOpenEffect {
        let sound_event = if counter_effect.on_open {
            Some(Self::OPEN_SOUND)
        } else if counter_effect.on_close {
            Some(Self::CLOSE_SOUND)
        } else {
            None
        };
        EnderChestOpenEffect {
            block_event: EnderChestBlockEvent {
                pos: self.world_position,
                block: Self::BLOCK_ID,
                action: Self::BLOCK_EVENT_ACTION,
                param: counter_effect.opener_count_changed.1,
            },
            counter_effect,
            sound_event,
        }
    }
}
