#![allow(dead_code)]

use crate::block_update::BlockPos;

/// Default survival/adventure block interaction reach.
/// Matches Java `Player.DEFAULT_BLOCK_INTERACTION_RANGE`.
pub const DEFAULT_BLOCK_INTERACTION_RANGE: f32 = 4.5;
/// Default survival/adventure entity interaction reach.
/// Matches Java `Player.DEFAULT_ENTITY_INTERACTION_RANGE`.
pub const DEFAULT_ENTITY_INTERACTION_RANGE: f32 = 3.0;
/// Creative-mode additive modifier applied to entity interaction range.
/// Matches Java `Player.CREATIVE_ENTITY_INTERACTION_RANGE_MODIFIER_VALUE`.
pub const CREATIVE_ENTITY_INTERACTION_RANGE_MODIFIER: f32 = 2.0;
/// Creative-mode additive modifier applied to block interaction range.
/// Matches Java `ServerPlayer.CREATIVE_BLOCK_INTERACTION_RANGE_MODIFIER` (ADD_VALUE 0.5).
pub const CREATIVE_BLOCK_INTERACTION_RANGE_MODIFIER: f32 = 0.5;

/// Server-side entity interaction reach for the given game mode.
pub fn entity_interaction_range(is_creative: bool) -> f32 {
    if is_creative {
        DEFAULT_ENTITY_INTERACTION_RANGE + CREATIVE_ENTITY_INTERACTION_RANGE_MODIFIER
    } else {
        DEFAULT_ENTITY_INTERACTION_RANGE
    }
}

/// Server-side block interaction reach for the given game mode.
pub fn block_interaction_range(is_creative: bool) -> f32 {
    if is_creative {
        DEFAULT_BLOCK_INTERACTION_RANGE + CREATIVE_BLOCK_INTERACTION_RANGE_MODIFIER
    } else {
        DEFAULT_BLOCK_INTERACTION_RANGE
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockBreakAction {
    Start,
    Stop,
    Abort,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerGameMode {
    Survival,
    Creative,
    Adventure,
    Spectator,
}

impl PlayerGameMode {
    pub fn has_instabuild(self) -> bool {
        self == PlayerGameMode::Creative
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BlockBreakState {
    pub is_destroying: bool,
    pub destroy_progress_start: i32,
    pub destroy_pos: BlockPos,
    pub game_ticks: i32,
    pub has_delayed_destroy: bool,
    pub delayed_destroy_pos: BlockPos,
    pub delayed_tick_start: i32,
    pub last_sent_state: i32,
}

impl Default for BlockBreakState {
    fn default() -> Self {
        Self {
            is_destroying: false,
            destroy_progress_start: 0,
            destroy_pos: BlockPos { x: 0, y: 0, z: 0 },
            game_ticks: 0,
            has_delayed_destroy: false,
            delayed_destroy_pos: BlockPos { x: 0, y: 0, z: 0 },
            delayed_tick_start: 0,
            last_sent_state: -1,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BlockBreakInputContext {
    pub pos: BlockPos,
    pub action: BlockBreakAction,
    pub max_y: i32,
    pub sequence: i32,
    pub game_mode: PlayerGameMode,
    /// Whether the player is within reach of the target block
    pub within_reach: bool,
    /// Whether the server spawn protection covers this position
    pub spawn_protected: bool,
    /// Whether the level permits interaction at this position
    pub may_interact: bool,
    /// Whether adventure mode restricts breaking this block (CanDestroy)
    pub block_action_restricted: bool,
    /// Hardness of the block at pos (-1.0 = unbreakable, 0.0 = instant)
    pub block_hardness: f32,
    /// Player's computed tool speed for this block type
    pub tool_speed: f32,
    /// Whether the player's held item is the correct tool for drops
    pub has_correct_tool: bool,
    /// Whether the block at pos is air
    pub block_is_air: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BlockBreakOutcome {
    /// Server rejected the action; send corrective block update to client
    Denied(BlockBreakDenyReason),
    /// Instant break — destroy the block immediately
    InstantBreak,
    /// Ongoing break progress; value 0-9
    Progress(i32),
    /// Accumulated progress reached threshold — destroy the block
    Destroy,
    /// Send -1 progress to client to clear the break overlay
    ProgressReset,
    /// Abort acknowledged; clear break overlay
    Aborted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockBreakDenyReason {
    TooFar,
    TooHigh,
    SpawnProtection,
    MayNotInteract,
    BlockActionRestricted,
}

pub fn handle_block_break_action(
    state: &mut BlockBreakState,
    ctx: &BlockBreakInputContext,
) -> BlockBreakOutcome {
    if !ctx.within_reach {
        return BlockBreakOutcome::Denied(BlockBreakDenyReason::TooFar);
    }
    if ctx.pos.y > ctx.max_y {
        return BlockBreakOutcome::Denied(BlockBreakDenyReason::TooHigh);
    }

    match ctx.action {
        BlockBreakAction::Start => {
            if ctx.spawn_protected {
                return BlockBreakOutcome::Denied(BlockBreakDenyReason::SpawnProtection);
            }
            if !ctx.may_interact {
                return BlockBreakOutcome::Denied(BlockBreakDenyReason::MayNotInteract);
            }
            if ctx.game_mode.has_instabuild() {
                return BlockBreakOutcome::InstantBreak;
            }
            if ctx.block_action_restricted {
                return BlockBreakOutcome::Denied(BlockBreakDenyReason::BlockActionRestricted);
            }

            state.destroy_progress_start = state.game_ticks;

            // Java seeds progress at 1.0 and only recomputes it for non-air
            // blocks (after firing the onHitBlock/attack hooks, which the caller
            // applies as side effects here).
            let progress = if ctx.block_is_air {
                1.0
            } else {
                compute_destroy_progress(ctx.block_hardness, ctx.tool_speed, ctx.has_correct_tool)
            };

            // Insta-mine only applies to a non-air block at full progress; an
            // air block instead "starts destroying" (reporting state 10 without
            // being insta-broken), matching `handleBlockBreakAction`.
            if !ctx.block_is_air && progress >= 1.0 {
                return BlockBreakOutcome::InstantBreak;
            }

            // If a previous break was in progress the caller resyncs that block;
            // server-side we just retarget to the new position.
            state.is_destroying = true;
            state.destroy_pos = ctx.pos;

            let progress_state = (progress * 10.0) as i32;
            state.last_sent_state = progress_state;
            BlockBreakOutcome::Progress(progress_state)
        }

        BlockBreakAction::Stop => {
            if ctx.pos != state.destroy_pos {
                return BlockBreakOutcome::ProgressReset;
            }

            if !ctx.block_is_air {
                let ticks_spent = state.game_ticks - state.destroy_progress_start;
                let progress = compute_destroy_progress(
                    ctx.block_hardness,
                    ctx.tool_speed,
                    ctx.has_correct_tool,
                ) * (ticks_spent + 1) as f32;

                if progress >= 0.7 {
                    state.is_destroying = false;
                    return BlockBreakOutcome::Destroy;
                }

                if !state.has_delayed_destroy {
                    state.is_destroying = false;
                    state.has_delayed_destroy = true;
                    state.delayed_destroy_pos = ctx.pos;
                    state.delayed_tick_start = state.destroy_progress_start;
                }
            }

            BlockBreakOutcome::ProgressReset
        }

        BlockBreakAction::Abort => {
            state.is_destroying = false;
            // If pos mismatch, both positions need their overlay cleared
            BlockBreakOutcome::Aborted
        }
    }
}

/// Compute the destroy progress per tick.
///
/// Returns 0.0 for unbreakable blocks (hardness < 0), otherwise:
/// `tool_speed / block_hardness / modifier`
/// where modifier = 30 when has_correct_tool, 100 otherwise.
pub fn compute_destroy_progress(
    block_hardness: f32,
    tool_speed: f32,
    has_correct_tool: bool,
) -> f32 {
    if block_hardness < 0.0 {
        return 0.0;
    }
    let modifier = if has_correct_tool { 30.0 } else { 100.0 };
    tool_speed / block_hardness / modifier
}

/// Compute the player's effective mining speed for a block, 1:1 with
/// `Player.getDestroySpeed` in 26.1.2.
///
/// # Parameters
/// - `base_tool_speed`: raw speed from the selected item (1.0 = hand/wrong tool, 8.0 = diamond pick on stone)
/// - `mining_efficiency`: value of the `minecraft:mining_efficiency` attribute (adds to speed when > 1.0)
/// - `haste_amplifier`: effective Haste/Conduit Power dig-speed amplifier if active (None = none)
/// - `mining_fatigue_amplifier`: amplifier of the Mining Fatigue effect if active
/// - `block_break_speed`: value of the `minecraft:block_break_speed` attribute (default 1.0)
/// - `eye_in_water`: player's eye is submerged in water
/// - `submerged_mining_speed`: value of the `minecraft:submerged_mining_speed` attribute
///   (default 0.2; Aqua Affinity raises it to 1.0)
/// - `on_ground`: player is standing on solid ground
#[allow(clippy::too_many_arguments)]
pub fn player_tool_speed(
    base_tool_speed: f32,
    mining_efficiency: f32,
    haste_amplifier: Option<u8>,
    mining_fatigue_amplifier: Option<u8>,
    block_break_speed: f32,
    eye_in_water: bool,
    submerged_mining_speed: f32,
    on_ground: bool,
) -> f32 {
    let mut speed = base_tool_speed;
    if speed > 1.0 {
        speed += mining_efficiency;
    }
    if let Some(amplifier) = haste_amplifier {
        speed *= 1.0 + (i32::from(amplifier) + 1) as f32 * 0.2;
    }
    if let Some(amplifier) = mining_fatigue_amplifier {
        speed *= match amplifier {
            0 => 0.3,
            1 => 0.09,
            2 => 0.0027,
            _ => 8.1e-4,
        };
    }
    speed *= block_break_speed;
    if eye_in_water {
        speed *= submerged_mining_speed;
    }
    if !on_ground {
        speed /= 5.0;
    }
    speed
}

/// Ticks required to break a block (rounds up to nearest tick).
pub fn ticks_to_break(destroy_progress_per_tick: f32) -> i32 {
    if destroy_progress_per_tick <= 0.0 {
        return i32::MAX;
    }
    (1.0 / destroy_progress_per_tick).ceil() as i32
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::block_update::BlockPos;

    fn default_ctx(action: BlockBreakAction, pos: BlockPos) -> BlockBreakInputContext {
        BlockBreakInputContext {
            pos,
            action,
            max_y: 319,
            sequence: 1,
            game_mode: PlayerGameMode::Survival,
            within_reach: true,
            spawn_protected: false,
            may_interact: true,
            block_action_restricted: false,
            block_hardness: 1.5,
            tool_speed: 8.0,
            has_correct_tool: true,
            block_is_air: false,
        }
    }

    #[test]
    fn start_destroy_outside_reach_is_denied_too_far() {
        let mut state = BlockBreakState::default();
        let ctx = BlockBreakInputContext {
            within_reach: false,
            ..default_ctx(BlockBreakAction::Start, BlockPos { x: 0, y: 64, z: 0 })
        };
        assert_eq!(
            handle_block_break_action(&mut state, &ctx),
            BlockBreakOutcome::Denied(BlockBreakDenyReason::TooFar)
        );
    }

    #[test]
    fn start_destroy_above_max_y_is_denied_too_high() {
        let mut state = BlockBreakState::default();
        let ctx = BlockBreakInputContext {
            pos: BlockPos { x: 0, y: 400, z: 0 },
            max_y: 319,
            ..default_ctx(BlockBreakAction::Start, BlockPos { x: 0, y: 400, z: 0 })
        };
        assert_eq!(
            handle_block_break_action(&mut state, &ctx),
            BlockBreakOutcome::Denied(BlockBreakDenyReason::TooHigh)
        );
    }

    #[test]
    fn start_destroy_in_spawn_protection_is_denied() {
        let mut state = BlockBreakState::default();
        let ctx = BlockBreakInputContext {
            spawn_protected: true,
            ..default_ctx(BlockBreakAction::Start, BlockPos { x: 0, y: 64, z: 0 })
        };
        assert_eq!(
            handle_block_break_action(&mut state, &ctx),
            BlockBreakOutcome::Denied(BlockBreakDenyReason::SpawnProtection)
        );
    }

    #[test]
    fn start_destroy_creative_instant_breaks() {
        let mut state = BlockBreakState::default();
        let ctx = BlockBreakInputContext {
            game_mode: PlayerGameMode::Creative,
            ..default_ctx(BlockBreakAction::Start, BlockPos { x: 1, y: 64, z: 1 })
        };
        assert_eq!(
            handle_block_break_action(&mut state, &ctx),
            BlockBreakOutcome::InstantBreak
        );
    }

    #[test]
    fn start_destroy_adventure_mode_restricted_is_denied() {
        let mut state = BlockBreakState::default();
        let ctx = BlockBreakInputContext {
            game_mode: PlayerGameMode::Adventure,
            block_action_restricted: true,
            ..default_ctx(BlockBreakAction::Start, BlockPos { x: 1, y: 64, z: 1 })
        };
        assert_eq!(
            handle_block_break_action(&mut state, &ctx),
            BlockBreakOutcome::Denied(BlockBreakDenyReason::BlockActionRestricted)
        );
    }

    #[test]
    fn start_destroy_progress_updates_state_and_returns_progress() {
        let mut state = BlockBreakState::default();
        let pos = BlockPos { x: 5, y: 64, z: 5 };
        let ctx = default_ctx(BlockBreakAction::Start, pos);
        let outcome = handle_block_break_action(&mut state, &ctx);
        // diamond pick on stone (hardness 1.5): 8.0/1.5/30 = 0.1778 → state = 1
        assert!(matches!(outcome, BlockBreakOutcome::Progress(1)));
        assert!(state.is_destroying);
        assert_eq!(state.destroy_pos, pos);
    }

    #[test]
    fn start_destroy_instant_mine_when_progress_gte_1() {
        let mut state = BlockBreakState::default();
        let _ctx = BlockBreakInputContext {
            block_hardness: 0.0, // sand/gravel hardness = 0.5, but 0.0 would be instant
            tool_speed: 1.0,
            ..default_ctx(BlockBreakAction::Start, BlockPos { x: 0, y: 64, z: 0 })
        };
        // progress = 1.0 / 0.0 / 30 → infinite, triggers instant mine
        // Actually let's use hardness that would give progress >= 1.0:
        // With tool_speed=8.0, hardness=0.1, correct tool: 8.0/0.1/30 = 2.67 >= 1.0
        let ctx2 = BlockBreakInputContext {
            block_hardness: 0.1,
            tool_speed: 8.0,
            ..default_ctx(BlockBreakAction::Start, BlockPos { x: 0, y: 64, z: 0 })
        };
        assert_eq!(
            handle_block_break_action(&mut state, &ctx2),
            BlockBreakOutcome::InstantBreak
        );
    }

    #[test]
    fn abort_destroy_clears_destroying_state() {
        let mut state = BlockBreakState {
            is_destroying: true,
            destroy_pos: BlockPos { x: 3, y: 64, z: 3 },
            ..BlockBreakState::default()
        };
        let ctx = BlockBreakInputContext {
            pos: BlockPos { x: 3, y: 64, z: 3 },
            ..default_ctx(BlockBreakAction::Abort, BlockPos { x: 3, y: 64, z: 3 })
        };
        let outcome = handle_block_break_action(&mut state, &ctx);
        assert_eq!(outcome, BlockBreakOutcome::Aborted);
        assert!(!state.is_destroying);
    }

    #[test]
    fn stop_destroy_with_sufficient_ticks_destroys_block() {
        let mut state = BlockBreakState {
            is_destroying: true,
            destroy_progress_start: 0,
            destroy_pos: BlockPos { x: 2, y: 64, z: 2 },
            game_ticks: 100,
            ..BlockBreakState::default()
        };
        // At 100 ticks with 0.1778/tick, accumulated = 17.78 >> 0.7
        let ctx = BlockBreakInputContext {
            pos: BlockPos { x: 2, y: 64, z: 2 },
            ..default_ctx(BlockBreakAction::Stop, BlockPos { x: 2, y: 64, z: 2 })
        };
        assert_eq!(
            handle_block_break_action(&mut state, &ctx),
            BlockBreakOutcome::Destroy
        );
        assert!(!state.is_destroying);
    }

    #[test]
    fn break_speed_parity_diamond_pickaxe_on_stone_dirt_obsidian() {
        // Diamond pickaxe on stone: hardness=1.5, speed=8.0, correct tool
        let stone_progress = compute_destroy_progress(1.5, 8.0, true);
        let stone_ticks = ticks_to_break(stone_progress);
        assert_eq!(stone_ticks, 6, "diamond pick on stone should take 6 ticks");

        // Diamond pickaxe on dirt: hardness=0.5, tool_speed=1.0 (no bonus), correct tool (dirt doesn't require correct tool for drops)
        let dirt_progress = compute_destroy_progress(0.5, 1.0, true);
        let dirt_ticks = ticks_to_break(dirt_progress);
        assert_eq!(dirt_ticks, 15, "diamond pick on dirt should take 15 ticks");

        // Diamond pickaxe on obsidian: hardness=50.0, speed=8.0, correct tool
        let obsidian_progress = compute_destroy_progress(50.0, 8.0, true);
        let obsidian_ticks = ticks_to_break(obsidian_progress);
        assert_eq!(
            obsidian_ticks, 188,
            "diamond pick on obsidian should take 188 ticks"
        );
    }

    #[test]
    fn player_tool_speed_haste_fatigue_water_ground_modifiers() {
        // Default attribute values: block_break_speed = 1.0, submerged_mining_speed = 0.2.
        const BREAK: f32 = 1.0;
        const SUBMERGED: f32 = 0.2;

        // Base tool speed 8.0, no effects, on ground, not in water.
        assert_eq!(
            player_tool_speed(8.0, 0.0, None, None, BREAK, false, SUBMERGED, true),
            8.0
        );

        // Haste II (amplifier=1): speed *= 1.0 + (1+1)*0.2 = 1.4
        let haste2 = player_tool_speed(8.0, 0.0, Some(1), None, BREAK, false, SUBMERGED, true);
        assert!((haste2 - 11.2).abs() < 0.001);

        // Mining Fatigue I (amplifier=0): speed *= 0.3
        let fatigue = player_tool_speed(8.0, 0.0, None, Some(0), BREAK, false, SUBMERGED, true);
        assert!((fatigue - 2.4).abs() < 0.001);

        // Not on ground: speed / 5
        let airborne = player_tool_speed(8.0, 0.0, None, None, BREAK, false, SUBMERGED, false);
        assert_eq!(airborne, 1.6);

        // Eye in water, default submerged_mining_speed 0.2: speed *= 0.2 (the
        // old "/5 without aqua affinity").
        let underwater = player_tool_speed(8.0, 0.0, None, None, BREAK, true, SUBMERGED, true);
        assert!((underwater - 1.6).abs() < 0.001);

        // Aqua Affinity raises submerged_mining_speed to 1.0 → no water penalty.
        let aqua = player_tool_speed(8.0, 0.0, None, None, BREAK, true, 1.0, true);
        assert_eq!(aqua, 8.0);

        // The block_break_speed attribute multiplies the final speed.
        let buffed = player_tool_speed(8.0, 0.0, None, None, 1.5, false, SUBMERGED, true);
        assert_eq!(buffed, 12.0);

        // Mining efficiency adds only when speed > 1.0
        let with_efficiency = player_tool_speed(8.0, 2.0, None, None, BREAK, false, SUBMERGED, true);
        assert_eq!(with_efficiency, 10.0);

        // Below 1.0 base speed: efficiency not added
        let hand_speed = player_tool_speed(1.0, 2.0, None, None, BREAK, false, SUBMERGED, true);
        assert_eq!(hand_speed, 1.0);
    }

    #[test]
    fn interaction_range_constants_match_java_defaults() {
        assert_eq!(DEFAULT_BLOCK_INTERACTION_RANGE, 4.5);
        assert_eq!(DEFAULT_ENTITY_INTERACTION_RANGE, 3.0);
        assert_eq!(CREATIVE_ENTITY_INTERACTION_RANGE_MODIFIER, 2.0);
        assert_eq!(CREATIVE_BLOCK_INTERACTION_RANGE_MODIFIER, 0.5);

        assert_eq!(entity_interaction_range(false), 3.0);
        assert_eq!(entity_interaction_range(true), 5.0);
        assert_eq!(block_interaction_range(false), 4.5);
        assert_eq!(block_interaction_range(true), 5.0);
    }
}
