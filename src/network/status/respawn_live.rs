//! Live respawn support: resolving where a dead player comes back and rebuilding the
//! "new" player's state from the dead one.
//!
//! Java references: `ServerPlayer.findRespawnPositionAndUseSpawnBlock`,
//! `PlayerList.respawn`, `ServerPlayer.restoreFrom` and
//! `Player.remove` → `InventoryMenu.removed` (leftover crafting/cursor items drop).
//! The pure block-level resolution lives in `respawn_resolution.rs`.

use super::*;
use crate::block_behavior::BlockStateModel;
use crate::block_update::BlockPos;
use crate::respawn_resolution::{
    find_respawn_and_use_spawn_block, RespawnConfig, RespawnPosAngle, RespawnWorld,
};
use crate::xp_orb_entity::XpOrbRandom;

/// The only dimension the live server simulates; a stored respawn in any other dimension
/// maps to `server.getLevel(dimension) == null` and therefore the default spawn.
const OVERWORLD: &str = "minecraft:overworld";
/// `ClientboundGameEventPacket.NO_RESPAWN_BLOCK_AVAILABLE`.
const NO_RESPAWN_BLOCK_AVAILABLE_EVENT: u8 = 0;
/// `Player.getEyeHeight()` while standing.
const PLAYER_EYE_HEIGHT: f64 = 1.62;
/// `ItemEntity.setPickUpDelay(40)` applied by `LivingEntity.createItemStackToDrop`.
const DROP_PICKUP_DELAY: i32 = 40;

/// Everything a respawn needs from the running session.
pub(super) struct RespawnContext<'a> {
    pub properties: &'a ServerProperties,
    pub world_root: &'a Path,
    pub world_seed: i64,
    pub world_layout: &'a WorldLayout,
    pub chunk_cache: &'a GeneratedChunkCache,
    pub game_rules: &'a SharedGameRules,
    pub world_items: &'a Arc<Mutex<WorldItemEntities>>,
    /// Seed for the cosmetic scatter of dropped items.
    pub random_seed: u64,
}

/// `TeleportTransition` outcome of `findRespawnPositionAndUseSpawnBlock`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) enum RespawnTarget {
    /// `TeleportTransition.createDefault`: no (usable) stored respawn dimension.
    WorldSpawn,
    /// `TeleportTransition.missingRespawnBlock`: the stored spawn block is gone.
    MissingRespawnBlock,
    /// The stored respawn block resolved to a stand-up position.
    Block(RespawnPosAngle),
}

/// [`RespawnWorld`] over the live chunk cache of the overworld.
struct LiveRespawnWorld<'a> {
    context: &'a RespawnContext<'a>,
}

impl RespawnWorld for LiveRespawnWorld<'_> {
    fn block_state(&self, pos: BlockPos) -> BlockStateModel {
        read_live_block_model_at(
            self.context.chunk_cache,
            self.context.world_layout,
            self.context.world_seed,
            pos,
        )
    }

    /// `DimensionTypes.OVERWORLD`: `RESPAWN_ANCHOR_WORKS = false`.
    fn respawn_anchor_works(&self, _pos: BlockPos) -> bool {
        false
    }

    /// `DimensionTypes.OVERWORLD`: `BED_RULE = CAN_SLEEP_WHEN_DARK` (`canSetSpawn = ALWAYS`).
    fn bed_can_set_spawn(&self, _pos: BlockPos) -> bool {
        true
    }

    fn set_block_state(&mut self, pos: BlockPos, state: BlockStateModel) {
        self.context.chunk_cache.set_block(
            self.context.world_root,
            self.context.world_seed,
            pos,
            &state.state_name(),
        );
    }
}

/// `ServerPlayer.findRespawnPositionAndUseSpawnBlock(consumeSpawnBlock, ...)`.
pub(super) fn find_respawn_position_and_use_spawn_block(
    state: &PlaySessionState,
    context: &RespawnContext<'_>,
    consume_spawn_block: bool,
) -> RespawnTarget {
    let Some(spawn) = &state.spawn else {
        return RespawnTarget::WorldSpawn;
    };
    if spawn.dimension != OVERWORLD {
        return RespawnTarget::WorldSpawn;
    }
    let config = RespawnConfig {
        pos: BlockPos { x: spawn.x, y: spawn.y, z: spawn.z },
        yaw: spawn.yaw,
        pitch: spawn.pitch,
        forced: spawn.forced,
    };
    let mut world = LiveRespawnWorld { context };
    match find_respawn_and_use_spawn_block(&mut world, config, consume_spawn_block) {
        Some(found) => RespawnTarget::Block(found),
        None => RespawnTarget::MissingRespawnBlock,
    }
}

/// Resolves the placement for `target`, falling back to the world spawn.
pub(super) fn placement_for_target(
    target: RespawnTarget,
    state: &PlaySessionState,
    context: &RespawnContext<'_>,
) -> PlayerSpawnPlacement {
    match target {
        RespawnTarget::Block(found) => PlayerSpawnPlacement {
            x: found.position.0,
            y: found.position.1,
            z: found.position.2,
            yaw: found.yaw,
            pitch: found.pitch,
        },
        RespawnTarget::WorldSpawn | RespawnTarget::MissingRespawnBlock => {
            find_default_player_spawn(context.world_root, context.world_seed, state.game_mode)
        }
    }
}

/// `Player.remove(KILLED)` → `InventoryMenu.removed`: items left in the 2x2 crafting grid
/// and on the cursor are dropped at the dead player's position
/// (`AbstractContainerMenu.dropOrPlaceInInventory` → `player.drop(stack, false)`).
pub(super) fn drop_menu_leftovers<W: Write>(
    writer: &mut W,
    compression: CompressionState,
    state: &mut PlaySessionState,
    context: &RespawnContext<'_>,
) -> io::Result<()> {
    let mut leftovers = state.inventory_menu.take_crafting_inputs();
    let carried = std::mem::replace(&mut state.carried_item, ItemStack::empty());
    if !carried.is_empty() {
        leftovers.push(carried);
    }
    let mut random = XpOrbRandom::new(context.random_seed);
    for stack in leftovers {
        drop_stack_from_view(writer, compression, state, context, &stack, &mut random)?;
    }
    Ok(())
}

/// `LivingEntity.drop(stack, randomly = false, thrownFromHand = false)`: the stack leaves
/// the player's eyes along the view direction with a small random scatter.
fn drop_stack_from_view<W: Write>(
    writer: &mut W,
    compression: CompressionState,
    state: &PlaySessionState,
    context: &RespawnContext<'_>,
    stack: &ItemStack,
    random: &mut XpOrbRandom,
) -> io::Result<()> {
    let Some(item_pid) = item_protocol_id(stack.item_id()) else {
        return Ok(());
    };
    let (pitch, yaw) = (
        f64::from(state.pitch).to_radians(),
        f64::from(state.yaw).to_radians(),
    );
    let scatter_direction = random.next_double() * std::f64::consts::TAU;
    let scatter_power = 0.02 * random.next_double();
    let vel_y_noise = (random.next_double() - random.next_double()) * 0.1;
    let entity_id = lock_status_mutex(context.world_items).alloc_entity_id();
    let item = DroppedItem {
        entity_id,
        item: stack.item_id(),
        count: stack.count(),
        x: state.x,
        y: state.y + PLAYER_EYE_HEIGHT - 0.3,
        z: state.z,
        vel_x: -yaw.sin() * pitch.cos() * 0.3 + scatter_direction.cos() * scatter_power,
        vel_y: -pitch.sin() * 0.3 + 0.1 + vel_y_noise,
        vel_z: yaw.cos() * pitch.cos() * 0.3 + scatter_direction.sin() * scatter_power,
        pickup_delay: DROP_PICKUP_DELAY,
        age: 0,
        target_uuid: None,
        health: crate::item_entity::ITEM_DEFAULT_HEALTH,
    };
    write_item_entity_spawn_packets(writer, compression, &item, item_pid)?;
    lock_status_mutex(context.world_items).entities.push(item);
    Ok(())
}

/// `ServerPlayer.restoreFrom(oldPlayer, restoreAll = false)` for the fields VibeCraft
/// models: the new player is at full health with no effects, and only keeps inventory,
/// experience and score when `keepInventory` is on or the old player was a spectator.
pub(super) fn restore_player_after_death(state: &mut PlaySessionState, keep_inventory: bool) {
    let keep = keep_inventory || state.game_mode == GameMode::Spectator;
    if !keep {
        // A fresh ServerPlayer starts with an empty inventory; anything left after the
        // death drops (e.g. keepInventory toggled during the death screen) is gone.
        state
            .inventory_menu
            .player_inventory_mut()
            .death_drops(false, |_| false);
    }
    state.active_effects.clear();
    state.abilities = PlayerNbtAbilities::for_game_mode(state.game_mode);
    state.combat = player_death::PlayerCombatState::default();
    state.carried_item = ItemStack::empty();
    state.active_block_menu = None;
    state.next_container_id = 1;
    state.container_state_id = 0;
    state.block_break_state = crate::player_game_mode::BlockBreakState::default();
    reset_play_state_after_death_respawn(state, keep);
}

/// `PlayerList.respawn` when the missing respawn block was reported: the game event the
/// client shows as "Your home bed or respawn anchor was missing or obstructed".
pub(super) fn write_missing_respawn_block<W: Write>(
    writer: &mut W,
    compression: CompressionState,
) -> io::Result<()> {
    write_game_event_to_writer(writer, compression, NO_RESPAWN_BLOCK_AVAILABLE_EVENT, 0.0)
}

/// `ServerGamePacketListenerImpl.handleClientCommand` after `PlayerList.respawn(...)`: in a
/// hardcore world the respawned player becomes a spectator and the server stops
/// generating chunks for spectators.
pub(super) fn apply_hardcore_respawn<W: Write>(
    writer: &mut W,
    compression: CompressionState,
    state: &mut PlaySessionState,
    context: &RespawnContext<'_>,
) -> io::Result<()> {
    if !context.properties.hardcore {
        return Ok(());
    }
    if state.game_mode != GameMode::Spectator {
        state.previous_game_mode = Some(state.game_mode);
        state.game_mode = GameMode::Spectator;
        state.abilities.apply_game_mode(GameMode::Spectator);
        // ServerPlayer.setGameMode sends CHANGE_GAME_MODE followed by abilities.
        write_game_event_to_writer(writer, compression, 3, GameMode::Spectator as i32 as f32)?;
        write_framed_packet_with_compression(
            writer,
            compression,
            CLIENTBOUND_PLAYER_ABILITIES_PACKET_ID,
            |payload| write_player_abilities_packet(payload, GameMode::Spectator),
        )?;
    }
    // GameRules.set(SPECTATORS_GENERATE_CHUNKS, false, server): replayed to every session.
    let _ = lock_status_mutex(context.game_rules).set("spectators_generate_chunks", "false", false);
    Ok(())
}

#[cfg(test)]
mod tests;
