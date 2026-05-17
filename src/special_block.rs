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
    ExecuteCommand { success_count: i32 },
    Animate { event: &'static str },
    EmitSignal(u8),
    ToggleLit(bool),
    ChangeLevel(u8),
    SpawnEntity { entity: String },
    InsertItem { slot: usize },
    RemoveItem { slot: usize },
    Brush { completed: bool, turns_into: String },
    Noop,
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
    pub automatic: bool,
    pub powered: bool,
    pub conditional: bool,
    pub previous_success: bool,
}

pub fn edit_sign(sign: &SignState, player: &str, front: bool) -> SpecialBlockAction {
    if sign.waxed
        || sign
            .editor
            .as_deref()
            .is_some_and(|editor| editor != player)
    {
        SpecialBlockAction::DenyEditor
    } else {
        let _ = front;
        SpecialBlockAction::OpenEditor
    }
}

pub fn command_block_tick(state: &CommandBlockState, has_permission: bool) -> SpecialBlockAction {
    if !has_permission || state.command.is_empty() {
        return SpecialBlockAction::ExecuteCommand { success_count: 0 };
    }
    if state.conditional && !state.previous_success {
        return SpecialBlockAction::ExecuteCommand { success_count: 0 };
    }
    if state.automatic || state.powered {
        SpecialBlockAction::ExecuteCommand { success_count: 1 }
    } else {
        SpecialBlockAction::Noop
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
    if waterlogged {
        SpecialBlockAction::ToggleLit(false)
    } else if has_food {
        SpecialBlockAction::InsertItem { slot: 0 }
    } else {
        SpecialBlockAction::ToggleLit(!lit)
    }
}

pub fn candle_use(candles: u8, lit: bool, add_candle: bool) -> SpecialBlockAction {
    if add_candle && candles < 4 {
        SpecialBlockAction::ChangeLevel(candles + 1)
    } else {
        SpecialBlockAction::ToggleLit(!lit)
    }
}

pub fn cauldron_fill_level(current: u8, delta: i8) -> SpecialBlockAction {
    SpecialBlockAction::ChangeLevel((i16::from(current) + i16::from(delta)).clamp(0, 3) as u8)
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

pub fn brushable_progress(progress: u8, turns_into: &str) -> SpecialBlockAction {
    SpecialBlockAction::Brush {
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
    fn command_blocks_execute_only_when_allowed_and_triggered() {
        let command = CommandBlockState {
            command: "say hi".to_string(),
            automatic: false,
            powered: true,
            conditional: false,
            previous_success: false,
        };
        assert_eq!(
            command_block_tick(&command, true),
            SpecialBlockAction::ExecuteCommand { success_count: 1 }
        );
        assert_eq!(
            command_block_tick(
                &CommandBlockState {
                    conditional: true,
                    ..command
                },
                true
            ),
            SpecialBlockAction::ExecuteCommand { success_count: 0 }
        );
    }

    #[test]
    fn note_bell_campfire_candle_and_cauldron_actions_match_core_state_changes() {
        assert_eq!(
            note_block_signal(30, false, true),
            SpecialBlockAction::EmitSignal(24)
        );
        assert_eq!(
            bell_ring(Direction::North),
            SpecialBlockAction::Animate { event: "bell_ring" }
        );
        assert_eq!(
            campfire_use(true, false, false),
            SpecialBlockAction::ToggleLit(false)
        );
        assert_eq!(
            campfire_use(true, true, false),
            SpecialBlockAction::InsertItem { slot: 0 }
        );
        assert_eq!(
            candle_use(2, false, true),
            SpecialBlockAction::ChangeLevel(3)
        );
        assert_eq!(
            cauldron_fill_level(1, 3),
            SpecialBlockAction::ChangeLevel(3)
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
        assert_eq!(
            calibrated_sculk_frequency(5, 5),
            SpecialBlockAction::EmitSignal(5)
        );
        assert_eq!(calibrated_sculk_frequency(4, 5), SpecialBlockAction::Noop);
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
        assert_eq!(
            chiseled_bookshelf_use(2, true),
            SpecialBlockAction::RemoveItem { slot: 2 }
        );
        assert_eq!(
            brushable_progress(10, "minecraft:sand"),
            SpecialBlockAction::Brush {
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
            None
        );
    }
}
