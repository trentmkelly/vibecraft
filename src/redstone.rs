#![allow(dead_code)]

use crate::block_update::{BlockPos, Direction};
use crate::scheduled_tick::{ScheduledTick, TickPriority};

pub const MAX_SIGNAL: u8 = 15;
pub const REPEATER_DELAY_TICKS: [i32; 4] = [2, 4, 6, 8];
pub const OBSERVER_PULSE_TICKS: i32 = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComparatorMode {
    Compare,
    Subtract,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputKind {
    Lever,
    StoneButton,
    WoodenButton,
    PressurePlate,
    Tripwire,
    TargetBlock,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PistonKind {
    Normal,
    Sticky,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PistonDecision {
    Stay,
    Extend { sticky: bool },
    Retract { sticky: bool, pull_head: bool },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RedstoneDust {
    pub power: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RepeaterState {
    pub facing: Direction,
    pub delay_index: u8,
    pub powered: bool,
    pub locked: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ComparatorState {
    pub facing: Direction,
    pub mode: ComparatorMode,
    pub powered: bool,
    pub output: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ObserverState {
    pub facing: Direction,
    pub powered: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OpenableState {
    pub open: bool,
    pub powered: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PistonState {
    pub kind: PistonKind,
    pub facing: Direction,
    pub extended: bool,
}

pub fn clamp_signal(signal: u8) -> u8 {
    signal.min(MAX_SIGNAL)
}

pub fn dust_output(power: u8) -> u8 {
    clamp_signal(power)
}

pub fn dust_propagated_power(source_power: u8) -> u8 {
    clamp_signal(source_power).saturating_sub(1)
}

pub fn strongest_signal(signals: impl IntoIterator<Item = u8>) -> u8 {
    signals.into_iter().map(clamp_signal).max().unwrap_or(0)
}

pub fn repeater_delay_ticks(delay_index: u8) -> i32 {
    REPEATER_DELAY_TICKS[usize::from(delay_index.min(3))]
}

pub fn repeater_output(state: RepeaterState, rear_input: u8, side_inputs: [u8; 2]) -> u8 {
    if side_inputs.into_iter().any(|side| side > rear_input) || state.locked {
        if state.powered {
            MAX_SIGNAL
        } else {
            0
        }
    } else if rear_input > 0 {
        MAX_SIGNAL
    } else {
        0
    }
}

pub fn comparator_output(mode: ComparatorMode, rear_input: u8, side_input: u8) -> u8 {
    let rear = clamp_signal(rear_input);
    let side = clamp_signal(side_input);
    match mode {
        ComparatorMode::Compare if rear >= side => rear,
        ComparatorMode::Compare => 0,
        ComparatorMode::Subtract => rear.saturating_sub(side),
    }
}

pub fn input_signal(kind: InputKind, active: bool, target_strength: u8) -> u8 {
    if !active {
        return 0;
    }

    match kind {
        InputKind::Lever
        | InputKind::StoneButton
        | InputKind::WoodenButton
        | InputKind::PressurePlate
        | InputKind::Tripwire => MAX_SIGNAL,
        InputKind::TargetBlock => clamp_signal(target_strength),
    }
}

pub fn input_reset_delay(kind: InputKind) -> Option<i32> {
    match kind {
        InputKind::StoneButton => Some(20),
        InputKind::WoodenButton => Some(30),
        InputKind::PressurePlate | InputKind::Tripwire => Some(10),
        InputKind::Lever | InputKind::TargetBlock => None,
    }
}

pub fn observer_on_neighbor_changed(
    pos: BlockPos,
    state: ObserverState,
    changed_pos: BlockPos,
    game_time: i64,
) -> Option<ScheduledTick> {
    let watched_pos = pos.relative(state.facing);
    if changed_pos == watched_pos && !state.powered {
        Some(ScheduledTick::create(
            game_time,
            0,
            pos,
            "minecraft:observer",
            OBSERVER_PULSE_TICKS,
            TickPriority::Normal,
        ))
    } else {
        None
    }
}

pub fn openable_from_power(_state: OpenableState, powered: bool) -> OpenableState {
    OpenableState {
        open: powered,
        powered,
    }
}

pub fn piston_decision(
    state: PistonState,
    direct_power: bool,
    quasi_power: bool,
    movable_blocks: usize,
) -> PistonDecision {
    let should_extend = direct_power || quasi_power;
    if should_extend && !state.extended && movable_blocks <= 12 {
        return PistonDecision::Extend {
            sticky: state.kind == PistonKind::Sticky,
        };
    }
    if !should_extend && state.extended {
        return PistonDecision::Retract {
            sticky: state.kind == PistonKind::Sticky,
            pull_head: state.kind == PistonKind::Sticky,
        };
    }
    PistonDecision::Stay
}

pub fn quasi_connectivity_positions(pos: BlockPos) -> [BlockPos; 5] {
    let above = pos.relative(Direction::Up);
    [
        above,
        above.relative(Direction::North),
        above.relative(Direction::South),
        above.relative(Direction::West),
        above.relative(Direction::East),
    ]
}

#[cfg(test)]
mod tests {
    use super::{
        clamp_signal, comparator_output, dust_output, dust_propagated_power, input_reset_delay,
        input_signal, observer_on_neighbor_changed, openable_from_power, piston_decision,
        quasi_connectivity_positions, repeater_delay_ticks, repeater_output, strongest_signal,
        ComparatorMode, InputKind, ObserverState, OpenableState, PistonDecision, PistonKind,
        PistonState, RepeaterState, MAX_SIGNAL, OBSERVER_PULSE_TICKS,
    };
    use crate::block_update::{BlockPos, Direction};

    #[test]
    fn dust_power_clamps_and_decays_one_level_per_step() {
        assert_eq!(clamp_signal(99), MAX_SIGNAL);
        assert_eq!(dust_output(12), 12);
        assert_eq!(dust_output(99), MAX_SIGNAL);
        assert_eq!(dust_propagated_power(15), 14);
        assert_eq!(dust_propagated_power(1), 0);
        assert_eq!(strongest_signal([0, 7, 99, 3]), MAX_SIGNAL);
    }

    #[test]
    fn repeater_outputs_full_power_after_delay_unless_locked_by_sides() {
        let repeater = RepeaterState {
            facing: Direction::North,
            delay_index: 2,
            powered: false,
            locked: false,
        };
        assert_eq!(repeater_delay_ticks(0), 2);
        assert_eq!(repeater_delay_ticks(2), 6);
        assert_eq!(repeater_delay_ticks(99), 8);
        assert_eq!(repeater_output(repeater, 3, [0, 0]), MAX_SIGNAL);
        assert_eq!(repeater_output(repeater, 3, [4, 0]), 0);
        assert_eq!(
            repeater_output(
                RepeaterState {
                    powered: true,
                    locked: true,
                    ..repeater
                },
                0,
                [0, 0],
            ),
            MAX_SIGNAL
        );
    }

    #[test]
    fn comparator_compare_and_subtract_modes_match_core_rules() {
        assert_eq!(comparator_output(ComparatorMode::Compare, 10, 3), 10);
        assert_eq!(comparator_output(ComparatorMode::Compare, 3, 10), 0);
        assert_eq!(comparator_output(ComparatorMode::Subtract, 10, 3), 7);
        assert_eq!(comparator_output(ComparatorMode::Subtract, 3, 10), 0);
    }

    #[test]
    fn inputs_emit_vanilla_strengths_and_reset_delays() {
        assert_eq!(input_signal(InputKind::Lever, true, 0), MAX_SIGNAL);
        assert_eq!(input_signal(InputKind::StoneButton, true, 0), MAX_SIGNAL);
        assert_eq!(input_signal(InputKind::WoodenButton, true, 0), MAX_SIGNAL);
        assert_eq!(input_signal(InputKind::PressurePlate, true, 0), MAX_SIGNAL);
        assert_eq!(input_signal(InputKind::Tripwire, true, 0), MAX_SIGNAL);
        assert_eq!(input_signal(InputKind::TargetBlock, true, 9), 9);
        assert_eq!(input_signal(InputKind::TargetBlock, false, 15), 0);
        assert_eq!(input_reset_delay(InputKind::StoneButton), Some(20));
        assert_eq!(input_reset_delay(InputKind::WoodenButton), Some(30));
        assert_eq!(input_reset_delay(InputKind::PressurePlate), Some(10));
        assert_eq!(input_reset_delay(InputKind::Lever), None);
    }

    #[test]
    fn observer_schedules_two_tick_pulse_only_for_watched_face() {
        let pos = BlockPos { x: 0, y: 64, z: 0 };
        let observer = ObserverState {
            facing: Direction::South,
            powered: false,
        };
        let tick = observer_on_neighbor_changed(pos, observer, pos.relative(Direction::South), 40)
            .expect("watched block should pulse observer");
        assert_eq!(tick.ty, "minecraft:observer");
        assert_eq!(tick.trigger_tick, 40 + i64::from(OBSERVER_PULSE_TICKS));

        assert_eq!(
            observer_on_neighbor_changed(pos, observer, pos.relative(Direction::North), 40),
            None
        );
    }

    #[test]
    fn doors_and_trapdoors_follow_powered_open_state() {
        let closed = OpenableState {
            open: false,
            powered: false,
        };
        assert_eq!(
            openable_from_power(closed, true),
            OpenableState {
                open: true,
                powered: true
            }
        );
        assert_eq!(
            openable_from_power(closed, false),
            OpenableState {
                open: false,
                powered: false
            }
        );
    }

    #[test]
    fn piston_decisions_cover_extension_retraction_push_limit_and_sticky_pull() {
        let piston = PistonState {
            kind: PistonKind::Normal,
            facing: Direction::East,
            extended: false,
        };
        assert_eq!(
            piston_decision(piston, true, false, 12),
            PistonDecision::Extend { sticky: false }
        );
        assert_eq!(
            piston_decision(piston, true, false, 13),
            PistonDecision::Stay
        );

        let sticky_extended = PistonState {
            kind: PistonKind::Sticky,
            extended: true,
            ..piston
        };
        assert_eq!(
            piston_decision(sticky_extended, false, false, 0),
            PistonDecision::Retract {
                sticky: true,
                pull_head: true
            }
        );
    }

    #[test]
    fn quasi_connectivity_checks_block_above_and_horizontal_neighbors_above() {
        let pos = BlockPos { x: 4, y: 70, z: -8 };
        assert_eq!(
            quasi_connectivity_positions(pos),
            [
                BlockPos { x: 4, y: 71, z: -8 },
                BlockPos { x: 4, y: 71, z: -9 },
                BlockPos { x: 4, y: 71, z: -7 },
                BlockPos { x: 3, y: 71, z: -8 },
                BlockPos { x: 5, y: 71, z: -8 },
            ]
        );
        assert_eq!(
            piston_decision(
                PistonState {
                    kind: PistonKind::Normal,
                    facing: Direction::Up,
                    extended: false
                },
                false,
                true,
                0,
            ),
            PistonDecision::Extend { sticky: false }
        );
    }
}
