//! Player-side `TntBlock` behaviour: igniting with flint and steel or a fire
//! charge (`useItemOn`), lighting on placement next to power (`onPlace`) and
//! the unstable-TNT trigger when a survival player breaks it
//! (`playerWillDestroy`).
//!
//! The lit entity itself is created by [`super::tnt_live::prime_tnt`] and
//! simulated by the shared world tick.
//!
//! TODO(live-redstone): `TntBlock.neighborChanged` (ignite when a neighbour
//! becomes powered) needs the live neighbour-update dispatch and signal
//! graph; `onPlace` below already asks `hasNeighborSignal` and starts working
//! once that exists.
//! TODO(projectiles-live): `TntBlock.onProjectileHit` (flaming arrows) needs
//! live projectile entities.

use std::io::{self, Write};
use std::sync::{Arc, Mutex};

use super::block_placement_live::{
    consume_placed_block_item, run_live_shape_cascade, write_block_update, LiveBlockWorld,
    LiveCascade,
};
use super::chunk_b::{write_block_change_ack, UseItemOnContext};
use super::game_rule_live::translatable_component_tag;
use super::item_use_live::hurt_and_break_held_item;
use super::tnt_live::prime_tnt;
use super::*;
use crate::block_behavior::BlockStateModel;
use crate::block_placement::PlacementWorld;
use crate::block_update::BlockPos;
use crate::damage_type::DamageEntityRef;
use crate::network::play::ServerboundUseItemOnPacket;

/// The `InteractionResult` of `TntBlock.useItemOn`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum TntItemUse {
    /// The item is not flint and steel / a fire charge, or the block is not TNT
    /// (`super.useItemOn` -> `PASS_TO_DEFAULT_BLOCK_INTERACTION`).
    NotApplicable,
    /// `InteractionResult.SUCCESS`: the TNT was lit.
    Success,
    /// `InteractionResult.PASS` after the "TNT explosions are disabled" overlay.
    Pass,
}

/// The player as a `DamageEntityRef` (the `owner` of TNT they light).
fn player_ref(state: &PlaySessionState) -> DamageEntityRef {
    DamageEntityRef::player(PLAYER_ENTITY_ID, state.game_mode == GameMode::Creative)
}

/// `Level.setBlock(pos, AIR, flags)` for a lit TNT block: clears the block,
/// tells the client and cascades the neighbour shape updates.
fn remove_tnt_block<W: Write>(
    writer: &mut W,
    compression: CompressionState,
    context: &mut UseItemOnContext<'_, '_>,
    pos: BlockPos,
) -> io::Result<()> {
    context
        .chunk_cache
        .set_block(context.world_layout.root(), context.world_seed, pos, "minecraft:air");
    write_block_update(writer, compression, pos, 0)?;
    let mut cascade = LiveCascade {
        layout: context.world_layout,
        seed: context.world_seed,
        cache: context.chunk_cache,
        fluid_ticks: context.live_fluid_ticks,
        block_ticks: context.live_block_ticks,
        game_time: context.game_time,
        random_roll: ((context.game_time as i32) ^ pos.x ^ pos.z).rem_euclid(40),
        max_chained_neighbor_updates: context.max_chained_neighbor_updates,
    };
    run_live_shape_cascade(writer, compression, &mut cascade, vec![pos])
}

/// `TntBlock.useItemOn(itemStack, state, level, pos, player, hand, hit)`.
pub(super) fn use_item_on_tnt(
    stream: &mut ClientStream,
    compression: CompressionState,
    state: &mut PlaySessionState,
    context: &mut UseItemOnContext<'_, '_>,
    packet: &ServerboundUseItemOnPacket,
    held_slot: usize,
    item_name: &str,
) -> io::Result<TntItemUse> {
    let is_flint = item_name == "minecraft:flint_and_steel";
    if !is_flint && item_name != "minecraft:fire_charge" {
        return Ok(TntItemUse::NotApplicable);
    }
    let clicked_pos = BlockPos {
        x: packet.block_hit.x,
        y: packet.block_hit.y,
        z: packet.block_hit.z,
    };
    let clicked = read_live_block_model_at(
        context.chunk_cache,
        context.world_layout,
        context.world_seed,
        clicked_pos,
    );
    if clicked.registry_id != "minecraft:tnt" {
        return Ok(TntItemUse::NotApplicable);
    }
    if prime_tnt(
        context.world_items,
        &context.chunk_cache.level_random,
        clicked_pos,
        Some(player_ref(state))) {
        // `level.setBlock(pos, AIR, 11)`.
        remove_tnt_block(stream, compression, context, clicked_pos)?;
        write_block_change_ack(stream, compression, packet.sequence)?;
        if is_flint {
            hurt_and_break_held_item(stream, compression, state, held_slot)?;
        } else {
            consume_placed_block_item(stream, compression, state, held_slot)?;
        }
        // TODO(live-stats): `player.awardStat(Stats.ITEM_USED.get(item))`.
        return Ok(TntItemUse::Success);
    }
    // `prime` only fails when `tnt_explodes` is off.
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_SYSTEM_CHAT_PACKET_ID,
        |payload| {
            ClientboundSystemChatPacket {
                content: translatable_component_tag("block.minecraft.tnt.disabled", &[]),
                overlay: true,
            }
            .write(payload)
        },
    )?;
    Ok(TntItemUse::Pass)
}

/// `TntBlock.onPlace`: a TNT block placed where a neighbour is powered is
/// lit at once (and removed).
pub(super) fn tnt_on_place<W: Write>(
    stream: &mut W,
    compression: CompressionState,
    context: &mut UseItemOnContext<'_, '_>,
    pos: BlockPos,
) -> io::Result<()> {
    let world = LiveBlockWorld {
        layout: context.world_layout,
        seed: context.world_seed,
        cache: context.chunk_cache,
    };
    if world.has_neighbor_signal(pos) && prime_tnt(context.world_items, &context.chunk_cache.level_random, pos, None) {
        // `level.removeBlock(pos, false)`.
        remove_tnt_block(stream, compression, context, pos)?;
    }
    Ok(())
}

/// `TntBlock.playerWillDestroy`: a survival player breaking an `unstable`
/// TNT block lights it (`prime(level, pos)`, result ignored; the caller has
/// already cleared the block).
pub(super) fn tnt_will_destroy(
    world_items: &Arc<Mutex<WorldItemEntities>>,
    level_random: &LevelRandomSource,
    game_mode: GameMode,
    broken: &BlockStateModel,
    pos: BlockPos,
) {
    if broken.registry_id == "minecraft:tnt"
        && broken.property("unstable") == Some("true")
        && game_mode != GameMode::Creative
    {
        prime_tnt(world_items, level_random, pos, None);
    }
}
