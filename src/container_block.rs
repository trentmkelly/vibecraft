#![allow(dead_code)]

use crate::block_update::Direction;
use crate::redstone::MAX_SIGNAL;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContainerKind {
    Chest,
    TrappedChest,
    Barrel,
    ShulkerBox,
    Hopper,
    Dropper,
    Dispenser,
    Furnace,
    Smoker,
    BlastFurnace,
    BrewingStand,
    Crafter,
    DecoratedPot,
    Lectern,
    Jukebox,
    Beacon,
    EnchantingTable,
    Anvil,
    Grindstone,
    SmithingTable,
    Loom,
    Stonecutter,
    CartographyTable,
    CraftingTable,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContainerState {
    pub kind: ContainerKind,
    pub slots: usize,
    pub occupied_slots: usize,
    pub viewers: u32,
    pub locked: bool,
    pub loot_table: Option<String>,
    pub facing: Direction,
    pub lit: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContainerAction {
    OpenMenu {
        menu: &'static str,
        slots: usize,
    },
    DenyLocked,
    UpdateViewerCount {
        viewers: u32,
        open: bool,
    },
    Transfer {
        from_slot: usize,
        to_slot: usize,
        direction: Direction,
    },
    Dispense {
        slot: usize,
        direction: Direction,
    },
    CraftPulse {
        triggered: bool,
    },
    TickProgress {
        lit: bool,
        progress_delta: i32,
    },
    Comparator(u8),
    Noop,
}

pub fn default_container(kind: ContainerKind, facing: Direction) -> ContainerState {
    ContainerState {
        kind,
        slots: slot_count(kind),
        occupied_slots: 0,
        viewers: 0,
        locked: false,
        loot_table: None,
        facing,
        lit: false,
    }
}

pub fn slot_count(kind: ContainerKind) -> usize {
    match kind {
        ContainerKind::Chest
        | ContainerKind::TrappedChest
        | ContainerKind::Barrel
        | ContainerKind::ShulkerBox => 27,
        ContainerKind::Hopper => 5,
        ContainerKind::Dropper | ContainerKind::Dispenser | ContainerKind::Crafter => 9,
        ContainerKind::Furnace | ContainerKind::Smoker | ContainerKind::BlastFurnace => 3,
        ContainerKind::BrewingStand => 5,
        ContainerKind::DecoratedPot | ContainerKind::Lectern | ContainerKind::Jukebox => 1,
        ContainerKind::Beacon => 1,
        ContainerKind::EnchantingTable => 2,
        ContainerKind::Anvil | ContainerKind::Grindstone | ContainerKind::SmithingTable => 3,
        ContainerKind::Loom | ContainerKind::Stonecutter | ContainerKind::CartographyTable => 4,
        ContainerKind::CraftingTable => 10,
    }
}

pub fn menu_name(kind: ContainerKind) -> &'static str {
    match kind {
        ContainerKind::Chest
        | ContainerKind::TrappedChest
        | ContainerKind::Barrel
        | ContainerKind::ShulkerBox => "generic_9x3",
        ContainerKind::Hopper => "hopper",
        ContainerKind::Dropper | ContainerKind::Dispenser => "generic_3x3",
        ContainerKind::Furnace => "furnace",
        ContainerKind::Smoker => "smoker",
        ContainerKind::BlastFurnace => "blast_furnace",
        ContainerKind::BrewingStand => "brewing_stand",
        ContainerKind::Crafter => "crafter_3x3",
        ContainerKind::DecoratedPot => "decorated_pot",
        ContainerKind::Lectern => "lectern",
        ContainerKind::Jukebox => "jukebox",
        ContainerKind::Beacon => "beacon",
        ContainerKind::EnchantingTable => "enchantment",
        ContainerKind::Anvil => "anvil",
        ContainerKind::Grindstone => "grindstone",
        ContainerKind::SmithingTable => "smithing",
        ContainerKind::Loom => "loom",
        ContainerKind::Stonecutter => "stonecutter",
        ContainerKind::CartographyTable => "cartography",
        ContainerKind::CraftingTable => "crafting",
    }
}

pub fn open_container(state: &ContainerState, key_matches_lock: bool) -> ContainerAction {
    if state.locked && !key_matches_lock {
        ContainerAction::DenyLocked
    } else {
        ContainerAction::OpenMenu {
            menu: menu_name(state.kind),
            slots: state.slots,
        }
    }
}

pub fn viewer_change(state: &ContainerState, delta: i32) -> ContainerAction {
    let viewers = if delta.is_negative() {
        state.viewers.saturating_sub(delta.unsigned_abs())
    } else {
        state.viewers.saturating_add(delta as u32)
    };
    ContainerAction::UpdateViewerCount {
        viewers,
        open: viewers > 0,
    }
}

pub fn comparator_output(state: &ContainerState) -> ContainerAction {
    if state.slots == 0 {
        return ContainerAction::Comparator(0);
    }
    let fill = ((state.occupied_slots.min(state.slots) as u32 * u32::from(MAX_SIGNAL))
        / state.slots as u32) as u8;
    ContainerAction::Comparator(fill)
}

pub fn hopper_transfer(
    state: &ContainerState,
    from_slot: usize,
    into_slot: usize,
) -> ContainerAction {
    if state.kind == ContainerKind::Hopper && from_slot < state.slots {
        ContainerAction::Transfer {
            from_slot,
            to_slot: into_slot,
            direction: state.facing,
        }
    } else {
        ContainerAction::Noop
    }
}

pub fn dispenser_or_dropper_fire(state: &ContainerState, slot: Option<usize>) -> ContainerAction {
    match (state.kind, slot) {
        (ContainerKind::Dispenser | ContainerKind::Dropper, Some(slot)) if slot < state.slots => {
            ContainerAction::Dispense {
                slot,
                direction: state.facing,
            }
        }
        _ => ContainerAction::Noop,
    }
}

pub fn furnace_tick(state: &ContainerState, has_fuel: bool, has_recipe: bool) -> ContainerAction {
    match state.kind {
        ContainerKind::Furnace | ContainerKind::Smoker | ContainerKind::BlastFurnace
            if has_fuel && has_recipe =>
        {
            ContainerAction::TickProgress {
                lit: true,
                progress_delta: match state.kind {
                    ContainerKind::Smoker | ContainerKind::BlastFurnace => 2,
                    _ => 1,
                },
            }
        }
        ContainerKind::Furnace | ContainerKind::Smoker | ContainerKind::BlastFurnace => {
            ContainerAction::TickProgress {
                lit: false,
                progress_delta: -1,
            }
        }
        _ => ContainerAction::Noop,
    }
}

pub fn brewing_tick(
    state: &ContainerState,
    has_fuel: bool,
    has_ingredient: bool,
    bottle_count: usize,
) -> ContainerAction {
    if state.kind == ContainerKind::BrewingStand && has_fuel && has_ingredient && bottle_count > 0 {
        ContainerAction::TickProgress {
            lit: true,
            progress_delta: 1,
        }
    } else {
        ContainerAction::Noop
    }
}

pub fn crafter_pulse(state: &ContainerState, powered: bool) -> ContainerAction {
    if state.kind == ContainerKind::Crafter {
        ContainerAction::CraftPulse { triggered: powered }
    } else {
        ContainerAction::Noop
    }
}

pub fn jukebox_signal(state: &ContainerState, playing: bool) -> ContainerAction {
    if state.kind == ContainerKind::Jukebox && playing {
        ContainerAction::Comparator(MAX_SIGNAL)
    } else if state.kind == ContainerKind::Jukebox {
        ContainerAction::Comparator(0)
    } else {
        ContainerAction::Noop
    }
}

#[cfg(test)]
mod tests {
    use super::{
        brewing_tick, comparator_output, crafter_pulse, default_container,
        dispenser_or_dropper_fire, furnace_tick, hopper_transfer, jukebox_signal, menu_name,
        open_container, slot_count, viewer_change, ContainerAction, ContainerKind,
    };
    use crate::block_update::Direction;
    use crate::redstone::MAX_SIGNAL;

    #[test]
    fn container_slot_counts_and_menus_cover_required_blocks() {
        assert_eq!(slot_count(ContainerKind::Chest), 27);
        assert_eq!(slot_count(ContainerKind::Hopper), 5);
        assert_eq!(slot_count(ContainerKind::Crafter), 9);
        assert_eq!(slot_count(ContainerKind::BrewingStand), 5);
        assert_eq!(menu_name(ContainerKind::CraftingTable), "crafting");
        assert_eq!(menu_name(ContainerKind::Anvil), "anvil");
        assert_eq!(menu_name(ContainerKind::Stonecutter), "stonecutter");
    }

    #[test]
    fn locked_containers_deny_open_until_key_matches() {
        let mut chest = default_container(ContainerKind::Chest, Direction::North);
        chest.locked = true;
        assert_eq!(open_container(&chest, false), ContainerAction::DenyLocked);
        assert_eq!(
            open_container(&chest, true),
            ContainerAction::OpenMenu {
                menu: "generic_9x3",
                slots: 27
            }
        );
    }

    #[test]
    fn viewer_count_controls_lid_or_barrel_open_state() {
        let barrel = default_container(ContainerKind::Barrel, Direction::Up);
        assert_eq!(
            viewer_change(&barrel, 1),
            ContainerAction::UpdateViewerCount {
                viewers: 1,
                open: true
            }
        );
        assert_eq!(
            viewer_change(&ContainerKind::Barrel.into_state(Direction::Up, 1), -1),
            ContainerAction::UpdateViewerCount {
                viewers: 0,
                open: false
            }
        );
    }

    #[test]
    fn comparator_output_scales_with_inventory_fullness() {
        let mut chest = default_container(ContainerKind::Chest, Direction::North);
        chest.occupied_slots = 27;
        assert_eq!(
            comparator_output(&chest),
            ContainerAction::Comparator(MAX_SIGNAL)
        );
        chest.occupied_slots = 9;
        assert_eq!(comparator_output(&chest), ContainerAction::Comparator(5));
    }

    #[test]
    fn hopper_dropper_and_dispenser_actions_use_facing_and_slots() {
        let hopper = default_container(ContainerKind::Hopper, Direction::Down);
        assert_eq!(
            hopper_transfer(&hopper, 2, 0),
            ContainerAction::Transfer {
                from_slot: 2,
                to_slot: 0,
                direction: Direction::Down
            }
        );
        let dispenser = default_container(ContainerKind::Dispenser, Direction::East);
        assert_eq!(
            dispenser_or_dropper_fire(&dispenser, Some(4)),
            ContainerAction::Dispense {
                slot: 4,
                direction: Direction::East
            }
        );
    }

    #[test]
    fn furnaces_smokers_and_blast_furnaces_tick_progress() {
        let furnace = default_container(ContainerKind::Furnace, Direction::North);
        assert_eq!(
            furnace_tick(&furnace, true, true),
            ContainerAction::TickProgress {
                lit: true,
                progress_delta: 1
            }
        );
        let smoker = default_container(ContainerKind::Smoker, Direction::North);
        assert_eq!(
            furnace_tick(&smoker, true, true),
            ContainerAction::TickProgress {
                lit: true,
                progress_delta: 2
            }
        );
        assert_eq!(
            furnace_tick(&smoker, false, true),
            ContainerAction::TickProgress {
                lit: false,
                progress_delta: -1
            }
        );
    }

    #[test]
    fn brewing_crafter_and_jukebox_have_server_tick_or_signal_hooks() {
        let brewing = default_container(ContainerKind::BrewingStand, Direction::North);
        assert_eq!(
            brewing_tick(&brewing, true, true, 3),
            ContainerAction::TickProgress {
                lit: true,
                progress_delta: 1
            }
        );
        let crafter = default_container(ContainerKind::Crafter, Direction::South);
        assert_eq!(
            crafter_pulse(&crafter, true),
            ContainerAction::CraftPulse { triggered: true }
        );
        let jukebox = default_container(ContainerKind::Jukebox, Direction::North);
        assert_eq!(
            jukebox_signal(&jukebox, true),
            ContainerAction::Comparator(MAX_SIGNAL)
        );
    }

    trait TestState {
        fn into_state(self, facing: Direction, viewers: u32) -> super::ContainerState;
    }

    impl TestState for ContainerKind {
        fn into_state(self, facing: Direction, viewers: u32) -> super::ContainerState {
            let mut state = default_container(self, facing);
            state.viewers = viewers;
            state
        }
    }
}
