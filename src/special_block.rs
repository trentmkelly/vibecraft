#![allow(dead_code)]

use crate::block_update::{BlockPos, Direction};
use crate::redstone::MAX_SIGNAL;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpecialBlockKind {
    Sign,
    HangingSign,
    Book,
    CommandBlock,
    Skull,
    Banner,
    Conduit,
    Bell,
    Campfire,
    Candle,
    Cauldron,
    NoteBlock,
    Spawner,
    Vault,
    TrialSpawner,
    CalibratedSculkSensor,
    ChiseledBookshelf,
    BrushableBlock,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpecialBlockAction {
    OpenEditor,
    DenyEditor,
    ExecuteCommand {
        success_count: i32,
    },
    Animate {
        event: &'static str,
    },
    EmitSignal(u8),
    ToggleLit(bool),
    ChangeLevel(u8),
    SpawnEntity {
        entity: String,
    },
    InsertItem {
        slot: usize,
    },
    RemoveItem {
        slot: usize,
    },
    Brush {
        stage: u8,
        completed: bool,
        turns_into: String,
    },
    Noop,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandBlockMode {
    Redstone,
    Auto,
    Sequence,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignState {
    pub waxed: bool,
    pub editor: Option<String>,
    pub front_text: [String; 4],
    pub back_text: [String; 4],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandBlockState {
    pub command: String,
    pub mode: CommandBlockMode,
    /// Current powered signal state for the block.
    pub powered: bool,
    /// Previous powered value so redstone command blocks can gate on rising edges.
    pub previously_powered: bool,
    pub conditional: bool,
    pub previous_success: bool,
}

pub fn edit_sign(sign: &SignState, player: &str, front: bool) -> SpecialBlockAction {
    let _ = front;
    if sign.waxed
        || sign
            .editor
            .as_deref()
            .is_some_and(|editor| editor != player)
    {
        SpecialBlockAction::DenyEditor
    } else {
        SpecialBlockAction::OpenEditor
    }
}

pub fn command_block_tick(state: &CommandBlockState, has_permission: bool) -> SpecialBlockAction {
    if !has_permission || state.command.is_empty() {
        return SpecialBlockAction::ExecuteCommand { success_count: 0 };
    }

    let should_execute = match state.mode {
        CommandBlockMode::Auto => true,
        CommandBlockMode::Sequence => state.powered || state.previously_powered,
        CommandBlockMode::Redstone => state.powered && !state.previously_powered,
    };

    if !should_execute {
        return SpecialBlockAction::Noop;
    }

    if state.conditional && !state.previous_success {
        return SpecialBlockAction::ExecuteCommand { success_count: 0 };
    }

    SpecialBlockAction::ExecuteCommand { success_count: 1 }
}

pub fn command_block_mode_from_automatic(automatic: bool) -> CommandBlockMode {
    if automatic {
        CommandBlockMode::Auto
    } else {
        CommandBlockMode::Redstone
    }
}

pub fn note_block_signal(note: u8, powered_before: bool, powered_now: bool) -> SpecialBlockAction {
    if !powered_before && powered_now {
        SpecialBlockAction::EmitSignal(note.min(24))
    } else {
        SpecialBlockAction::Noop
    }
}

pub fn bell_ring(hit_direction: Direction) -> SpecialBlockAction {
    match hit_direction {
        Direction::North | Direction::South | Direction::East | Direction::West => {
            SpecialBlockAction::Animate { event: "bell_ring" }
        }
        Direction::Up | Direction::Down => SpecialBlockAction::Noop,
    }
}

pub fn campfire_use(lit: bool, has_food: bool, waterlogged: bool) -> SpecialBlockAction {
    let _ = lit;
    if waterlogged {
        SpecialBlockAction::Noop
    } else if has_food {
        SpecialBlockAction::InsertItem { slot: 0 }
    } else {
        SpecialBlockAction::Noop
    }
}

pub fn candle_use(candles: u8, lit: bool, add_candle: bool) -> SpecialBlockAction {
    if add_candle {
        if candles >= 4 {
            SpecialBlockAction::Noop
        } else {
            SpecialBlockAction::ChangeLevel(candles + 1)
        }
    } else if lit {
        SpecialBlockAction::ToggleLit(false)
    } else {
        SpecialBlockAction::Noop
    }
}

pub fn cauldron_fill_level(current: u8, delta: i8) -> SpecialBlockAction {
    let next = (i16::from(current) + i16::from(delta)).clamp(0, 3) as u8;
    SpecialBlockAction::ChangeLevel(next)
}

pub fn skull_animation(kind: &str, powered: bool) -> SpecialBlockAction {
    if powered && matches!(kind, "dragon" | "piglin") {
        SpecialBlockAction::Animate {
            event: "skull_animation",
        }
    } else {
        SpecialBlockAction::Noop
    }
}

pub fn banner_pattern_count(patterns: usize) -> SpecialBlockAction {
    SpecialBlockAction::EmitSignal(patterns.min(16) as u8)
}

pub fn conduit_effect(active: bool, hostile_nearby: bool) -> SpecialBlockAction {
    if active && hostile_nearby {
        SpecialBlockAction::EmitSignal(MAX_SIGNAL)
    } else if active {
        SpecialBlockAction::EmitSignal(1)
    } else {
        SpecialBlockAction::Noop
    }
}

pub fn spawner_tick(
    kind: SpecialBlockKind,
    player_in_range: bool,
    delay: i32,
) -> SpecialBlockAction {
    if !matches!(
        kind,
        SpecialBlockKind::Spawner | SpecialBlockKind::TrialSpawner | SpecialBlockKind::Vault
    ) || !player_in_range
    {
        return SpecialBlockAction::Noop;
    }

    if delay <= 0 {
        SpecialBlockAction::SpawnEntity {
            entity: match kind {
                SpecialBlockKind::TrialSpawner => "minecraft:trial_mob".to_string(),
                SpecialBlockKind::Vault => "minecraft:vault_reward".to_string(),
                _ => "minecraft:pig".to_string(),
            },
        }
    } else {
        SpecialBlockAction::Animate {
            event: "spawner_wait",
        }
    }
}

pub fn calibrated_sculk_frequency(side_power: u8, vibration_frequency: u8) -> SpecialBlockAction {
    if side_power == 0 || side_power == vibration_frequency {
        SpecialBlockAction::EmitSignal(vibration_frequency.min(MAX_SIGNAL))
    } else {
        SpecialBlockAction::Noop
    }
}

pub fn chiseled_bookshelf_use(slot: usize, occupied: bool) -> SpecialBlockAction {
    if slot >= 6 {
        SpecialBlockAction::Noop
    } else if occupied {
        SpecialBlockAction::RemoveItem { slot }
    } else {
        SpecialBlockAction::InsertItem { slot }
    }
}

fn brushable_stage(brush_count: u8) -> u8 {
    if brush_count == 0 {
        0
    } else if brush_count < 3 {
        1
    } else if brush_count < 6 {
        2
    } else if brush_count < 10 {
        3
    } else {
        4
    }
}

pub fn brushable_progress(progress: u8, turns_into: &str) -> SpecialBlockAction {
    SpecialBlockAction::Brush {
        stage: brushable_stage(progress),
        completed: progress >= 10,
        turns_into: if progress >= 10 {
            turns_into.to_string()
        } else {
            String::new()
        },
    }
}

pub fn block_entity_update_pos(pos: BlockPos, kind: SpecialBlockKind) -> Option<BlockPos> {
    match kind {
        SpecialBlockKind::Sign
        | SpecialBlockKind::HangingSign
        | SpecialBlockKind::CommandBlock
        | SpecialBlockKind::Skull
        | SpecialBlockKind::Banner
        | SpecialBlockKind::Conduit
        | SpecialBlockKind::Bell
        | SpecialBlockKind::Spawner
        | SpecialBlockKind::Campfire
        | SpecialBlockKind::Vault
        | SpecialBlockKind::TrialSpawner
        | SpecialBlockKind::CalibratedSculkSensor
        | SpecialBlockKind::ChiseledBookshelf
        | SpecialBlockKind::BrushableBlock => Some(pos),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sign_editing_respects_wax_and_other_editor() {
        let sign = SignState {
            waxed: false,
            editor: None,
            front_text: Default::default(),
            back_text: Default::default(),
        };
        assert_eq!(
            edit_sign(&sign, "Alex", true),
            SpecialBlockAction::OpenEditor
        );
        assert_eq!(
            edit_sign(
                &SignState {
                    waxed: true,
                    ..sign.clone()
                },
                "Alex",
                true
            ),
            SpecialBlockAction::DenyEditor
        );
        assert_eq!(
            edit_sign(
                &SignState {
                    editor: Some("Steve".to_string()),
                    ..sign
                },
                "Alex",
                true
            ),
            SpecialBlockAction::DenyEditor
        );
    }

    #[test]
    fn command_block_tick_mode_and_edge_detection_match_vanilla_modes() {
        let base = CommandBlockState {
            command: "say hi".to_string(),
            mode: CommandBlockMode::Redstone,
            powered: true,
            previously_powered: false,
            conditional: false,
            previous_success: false,
        };
        assert_eq!(
            command_block_tick(&base, true),
            SpecialBlockAction::ExecuteCommand { success_count: 1 }
        );
        assert_eq!(
            command_block_tick(
                &CommandBlockState {
                    powered: true,
                    previously_powered: true,
                    ..base.clone()
                },
                true
            ),
            SpecialBlockAction::Noop
        );
        assert_eq!(
            command_block_tick(
                &CommandBlockState {
                    mode: CommandBlockMode::Auto,
                    powered: false,
                    previously_powered: false,
                    ..base
                },
                true
            ),
            SpecialBlockAction::ExecuteCommand { success_count: 1 }
        );

        assert_eq!(
            command_block_tick(
                &CommandBlockState {
                    mode: CommandBlockMode::Sequence,
                    powered: true,
                    previously_powered: false,
                    ..CommandBlockState {
                        command: "say hi".to_string(),
                        mode: CommandBlockMode::Sequence,
                        powered: false,
                        previously_powered: false,
                        conditional: true,
                        previous_success: true,
                    }
                },
                true
            ),
            SpecialBlockAction::ExecuteCommand { success_count: 1 }
        );

        assert_eq!(
            command_block_tick(
                &CommandBlockState {
                    mode: CommandBlockMode::Redstone,
                    conditional: true,
                    previous_success: false,
                    ..CommandBlockState {
                        command: "say hi".to_string(),
                        mode: CommandBlockMode::Redstone,
                        powered: true,
                        previously_powered: false,
                        conditional: true,
                        previous_success: false,
                    }
                },
                true
            ),
            SpecialBlockAction::ExecuteCommand { success_count: 0 }
        );
    }

    #[test]
    fn auto_command_block_runs_each_tick_and_redstone_runs_on_leading_edge() {
        let auto = CommandBlockState {
            command: "say auto".to_string(),
            mode: CommandBlockMode::Auto,
            powered: false,
            previously_powered: false,
            conditional: false,
            previous_success: true,
        };
        assert_eq!(
            command_block_tick(&auto, true),
            SpecialBlockAction::ExecuteCommand { success_count: 1 }
        );
        assert_eq!(
            command_block_tick(
                &CommandBlockState {
                    powered: true,
                    previously_powered: true,
                    ..auto
                },
                true,
            ),
            SpecialBlockAction::ExecuteCommand { success_count: 1 }
        );

        let redstone = CommandBlockState {
            command: "say redstone".to_string(),
            mode: CommandBlockMode::Redstone,
            powered: true,
            previously_powered: false,
            conditional: false,
            previous_success: true,
        };
        assert_eq!(
            command_block_tick(&redstone, true),
            SpecialBlockAction::ExecuteCommand { success_count: 1 }
        );
        assert_eq!(
            command_block_tick(
                &CommandBlockState {
                    previously_powered: true,
                    ..redstone
                },
                true,
            ),
            SpecialBlockAction::Noop
        );
    }

    #[test]
    fn note_bell_campfire_candle_and_cauldron_actions_match_deeper_state_transitions() {
        assert_eq!(
            note_block_signal(30, false, true),
            SpecialBlockAction::EmitSignal(24)
        );
        assert_eq!(note_block_signal(2, true, true), SpecialBlockAction::Noop);
        assert_eq!(
            bell_ring(Direction::North),
            SpecialBlockAction::Animate { event: "bell_ring" }
        );
        assert_eq!(bell_ring(Direction::Up), SpecialBlockAction::Noop);
        assert_eq!(
            campfire_use(false, true, false),
            SpecialBlockAction::InsertItem { slot: 0 }
        );
        assert_eq!(campfire_use(true, false, false), SpecialBlockAction::Noop);
        assert_eq!(
            candle_use(2, false, true),
            SpecialBlockAction::ChangeLevel(3)
        );
        assert_eq!(candle_use(4, false, true), SpecialBlockAction::Noop);
        assert_eq!(
            candle_use(2, true, false),
            SpecialBlockAction::ToggleLit(false)
        );
        assert_eq!(candle_use(2, false, false), SpecialBlockAction::Noop);
        assert_eq!(
            cauldron_fill_level(1, 0),
            SpecialBlockAction::ChangeLevel(1)
        );
        assert_eq!(
            cauldron_fill_level(1, 3),
            SpecialBlockAction::ChangeLevel(3)
        );
        assert_eq!(
            cauldron_fill_level(0, -5),
            SpecialBlockAction::ChangeLevel(0)
        );
    }

    #[test]
    fn skull_banner_conduit_and_sculk_emit_expected_signals_or_animations() {
        assert_eq!(
            skull_animation("dragon", true),
            SpecialBlockAction::Animate {
                event: "skull_animation"
            }
        );
        assert_eq!(banner_pattern_count(20), SpecialBlockAction::EmitSignal(16));
        assert_eq!(
            conduit_effect(true, true),
            SpecialBlockAction::EmitSignal(MAX_SIGNAL)
        );
        assert_eq!(conduit_effect(false, false), SpecialBlockAction::Noop);
        assert_eq!(
            calibrated_sculk_frequency(5, 5),
            SpecialBlockAction::EmitSignal(5)
        );
        assert_eq!(calibrated_sculk_frequency(4, 5), SpecialBlockAction::Noop);
        assert_eq!(
            calibrated_sculk_frequency(0, 5),
            SpecialBlockAction::EmitSignal(5)
        );
    }

    #[test]
    fn spawners_vaults_bookshelves_and_brushables_model_interactions() {
        assert_eq!(
            spawner_tick(SpecialBlockKind::Spawner, true, 0),
            SpecialBlockAction::SpawnEntity {
                entity: "minecraft:pig".to_string()
            }
        );
        assert_eq!(
            spawner_tick(SpecialBlockKind::TrialSpawner, true, 0),
            SpecialBlockAction::SpawnEntity {
                entity: "minecraft:trial_mob".to_string()
            }
        );
        assert_eq!(
            chiseled_bookshelf_use(2, false),
            SpecialBlockAction::InsertItem { slot: 2 }
        );
        assert_eq!(chiseled_bookshelf_use(6, false), SpecialBlockAction::Noop);
        assert_eq!(
            chiseled_bookshelf_use(2, true),
            SpecialBlockAction::RemoveItem { slot: 2 }
        );
        assert_eq!(
            brushable_progress(0, "minecraft:sand"),
            SpecialBlockAction::Brush {
                stage: 0,
                completed: false,
                turns_into: String::new(),
            }
        );
        assert_eq!(
            brushable_progress(2, "minecraft:sand"),
            SpecialBlockAction::Brush {
                stage: 1,
                completed: false,
                turns_into: String::new(),
            }
        );
        assert_eq!(
            brushable_progress(9, "minecraft:sand"),
            SpecialBlockAction::Brush {
                stage: 3,
                completed: false,
                turns_into: String::new(),
            }
        );
        assert_eq!(
            brushable_progress(10, "minecraft:sand"),
            SpecialBlockAction::Brush {
                stage: 4,
                completed: true,
                turns_into: "minecraft:sand".to_string()
            }
        );
    }

    #[test]
    fn block_entity_update_positions_cover_special_block_entities() {
        let pos = BlockPos { x: 1, y: 2, z: 3 };
        assert_eq!(
            block_entity_update_pos(pos, SpecialBlockKind::Sign),
            Some(pos)
        );
        assert_eq!(
            block_entity_update_pos(pos, SpecialBlockKind::Campfire),
            Some(pos)
        );
        assert_eq!(block_entity_update_pos(pos, SpecialBlockKind::Book), None);
    }
}
