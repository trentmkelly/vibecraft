//! Live wiring of behavioral (non-`BlockItem`) `Item.useOn` overrides into the
//! play session: the dispatch Java performs in `ServerPlayerGameMode.useItemOn`
//! via `itemStack.useOn(context)` after the block-interaction pass.
//!
//! `FlintAndSteelItem` is the first member wired here; the remaining family
//! (axe strip, hoe till, shovel path, bone meal, ...) still sits on
//! `TODO(item-use-block-and-entity-behaviors)` in `item_family_behavior.rs`.

use std::io::{self, Write};

use super::block_placement_live::{
    direction3d_to_block, place_block_item_in_world, run_live_shape_cascade, write_block_update,
    LiveBlockWorld, LiveCascade,
};
use super::chunk_b::{
    raw_stack_for_player_inventory, write_block_change_ack, write_player_inventory_slot_update,
    UseItemOnContext,
};
use super::*;
use crate::block_update::BlockPos;
use crate::item_flint_and_steel::FlintAndSteelUse;

/// Java `FlintAndSteelItem.useOn`: light a campfire/candle/candle cake in
/// place, or place a `BaseFireBlock` state against the clicked face, then
/// damage the held item by 1.
pub(super) fn use_flint_and_steel<W: Write>(
    stream: &mut W,
    compression: CompressionState,
    state: &mut PlaySessionState,
    context: &mut UseItemOnContext<'_, '_>,
    packet: &crate::network::play::ServerboundUseItemOnPacket,
    held_slot: usize,
) -> io::Result<()> {
    let clicked_pos = BlockPos {
        x: packet.block_hit.x,
        y: packet.block_hit.y,
        z: packet.block_hit.z,
    };
    let world = LiveBlockWorld {
        layout: context.world_layout,
        seed: context.world_seed,
        cache: context.chunk_cache,
    };
    let outcome = crate::item_flint_and_steel::use_on(
        &world,
        clicked_pos,
        direction3d_to_block(packet.block_hit.direction),
    );
    let (pos, new_state) = match outcome {
        // InteractionResult.FAIL: no world change; only the sequence ack.
        FlintAndSteelUse::Fail => {
            return write_block_change_ack(stream, compression, packet.sequence)
        }
        FlintAndSteelUse::Light { pos, state } | FlintAndSteelUse::PlaceFire { pos, state } => {
            (pos, state)
        }
    };

    // Java plays FLINTANDSTEEL_USE via level.playSound(player, ...), which
    // broadcasts to every player EXCEPT the user (the client predicts its own
    // sound). With no live player registry there is no one else to notify;
    // see project memory "live player registry gap" for the broadcast work.
    // Java also raises GameEvent.BLOCK_PLACE/BLOCK_CHANGE (sculk) and the
    // PLACED_BLOCK criterion on the fire branch; neither system is live yet.

    // Java Level.setBlock(pos, state, 11): write the state, send the client
    // update, then run the updateNeighborShapes cascade (flag 11 has
    // UPDATE_KNOWN_SHAPE clear, so shape updates cascade like placement).
    place_block_item_in_world(context, pos, &new_state.state_name());
    write_block_change_ack(stream, compression, packet.sequence)?;
    let id = crate::block_states::network_id_for_block_state(&new_state.state_name()).unwrap_or(0);
    write_block_update(stream, compression, pos, id)?;

    // TODO(live-fire-tick): Java FireBlock.onPlace schedules the spread/age
    // tick (30 + nextInt(10)); the live scheduled-tick catalog has no fire
    // handler yet, so placed fire neither spreads nor burns out.
    let mut cascade = LiveCascade {
        layout: context.world_layout,
        seed: context.world_seed,
        cache: context.chunk_cache,
        fluid_ticks: context.live_fluid_ticks,
        block_ticks: context.live_block_ticks,
        game_time: context.game_time,
        random_roll: ((context.game_time as i32) ^ pos.x ^ pos.z).rem_euclid(25),
        max_chained_neighbor_updates: context.max_chained_neighbor_updates,
    };
    run_live_shape_cascade(stream, compression, &mut cascade, vec![pos])?;

    hurt_and_break_held_item(stream, compression, state, held_slot)
}

/// Java `ItemStack.hurtAndBreak(1, player, hand.asEquipmentSlot())` for the
/// held stack, followed by the slot resync Java's dirty-slot broadcaster
/// performs.
fn hurt_and_break_held_item<W: Write>(
    stream: &mut W,
    compression: CompressionState,
    state: &mut PlaySessionState,
    held_slot: usize,
) -> io::Result<()> {
    // Java processDurabilityChange returns 0 when hasInfiniteMaterials
    // (creative); the stack is untouched and no resync is needed.
    if state.game_mode == GameMode::Creative {
        return Ok(());
    }
    let mut stack = state
        .inventory_menu
        .player_inventory()
        .get(held_slot)
        .clone();
    stack.hurt_and_break(1);
    state
        .inventory_menu
        .player_inventory_mut()
        .set(held_slot, stack);
    let stack = state.inventory_menu.player_inventory().get(held_slot);
    // TODO(slot-component-sync): raw_stack_for_player_inventory sends an
    // empty component patch, so the client's durability bar is not updated
    // until inventory slot updates carry data components; the break (count 0)
    // does sync.
    write_player_inventory_slot_update(
        stream,
        compression,
        held_slot,
        raw_stack_for_player_inventory(stack),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::block_survival::SurvivalWorld;
    use crate::item_stack::ItemStack;
    use crate::network::play::{
        BlockHitResultPacketData, Direction3d, ServerboundSwingHand, ServerboundUseItemOnPacket,
    };
    use std::sync::{Arc, Mutex};

    fn temp_world_root(test_name: &str) -> std::path::PathBuf {
        let root = std::env::temp_dir().join(format!(
            "vibecraft-flint-and-steel-{test_name}-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        root
    }

    fn use_packet(x: i32, y: i32, z: i32, direction: Direction3d) -> ServerboundUseItemOnPacket {
        ServerboundUseItemOnPacket {
            hand: ServerboundSwingHand::MainHand,
            block_hit: BlockHitResultPacketData {
                x,
                y,
                z,
                direction,
                click_x: 0.5,
                click_y: 1.0,
                click_z: 0.5,
                inside: false,
                world_border_hit: false,
            },
            sequence: 7,
        }
    }

    /// Runs `use_flint_and_steel` against a one-block world and returns the
    /// state written at `fire_pos` plus the held stack afterwards.
    fn run_use(
        test_name: &str,
        ground: &str,
        packet: &ServerboundUseItemOnPacket,
        fire_pos: BlockPos,
    ) -> (crate::block_behavior::BlockStateModel, ItemStack) {
        let root = temp_world_root(test_name);
        let layout = WorldLayout::new(&root);
        let cache = GeneratedChunkCache::default();
        let clicked = BlockPos {
            x: packet.block_hit.x,
            y: packet.block_hit.y,
            z: packet.block_hit.z,
        };
        cache.set_block(layout.root(), 42, clicked, ground);

        let mut session = PlaySessionState::default();
        session
            .inventory_menu
            .player_inventory_mut()
            .set(0, ItemStack::new("minecraft:flint_and_steel", 1));

        let mut fluid_ticks = LiveFluidTicks::new();
        let mut block_ticks = LiveBlockTicks::new();
        let player_access = Arc::new(Mutex::new(crate::player_access::PlayerAccess::default()));
        let recipe_manager = crate::recipe_system::RecipeManagerModel::default();
        let mut context = UseItemOnContext {
            world_layout: &layout,
            world_seed: 42,
            chunk_cache: &cache,
            recipe_manager: &recipe_manager,
            live_fluid_ticks: &mut fluid_ticks,
            live_block_ticks: &mut block_ticks,
            game_time: 0,
            max_chained_neighbor_updates: 512,
            player_access: &player_access,
            profile_uuid: "test-uuid",
            spawn_protection_radius: 0,
        };
        let mut output = Vec::new();
        use_flint_and_steel(
            &mut output,
            CompressionState::disabled(),
            &mut session,
            &mut context,
            packet,
            0,
        )
        .unwrap();

        let placed = LiveBlockWorld {
            layout: &layout,
            seed: 42,
            cache: &cache,
        }
        .state_at(fire_pos);
        let held = session.inventory_menu.player_inventory().get(0).clone();
        (placed, held)
    }

    #[test]
    fn flint_and_steel_places_fire_on_sturdy_ground_and_damages_the_item() {
        let packet = use_packet(0, 64, 0, Direction3d::Up);
        let fire_pos = BlockPos { x: 0, y: 65, z: 0 };
        let (placed, held) = run_use("place", "minecraft:stone", &packet, fire_pos);
        assert_eq!(placed.registry_id, "minecraft:fire");
        assert_eq!(held.damage_value(), 1);
        assert_eq!(held.count(), 1);
    }

    #[test]
    fn flint_and_steel_places_soul_fire_over_soul_sand() {
        let packet = use_packet(0, 64, 0, Direction3d::Up);
        let fire_pos = BlockPos { x: 0, y: 65, z: 0 };
        let (placed, _) = run_use("soul", "minecraft:soul_sand", &packet, fire_pos);
        assert_eq!(placed.registry_id, "minecraft:soul_fire");
    }

    #[test]
    fn flint_and_steel_lights_an_unlit_campfire_in_place() {
        let packet = use_packet(0, 64, 0, Direction3d::Up);
        let campfire = BlockPos { x: 0, y: 64, z: 0 };
        let (placed, held) = run_use(
            "campfire",
            "minecraft:campfire[facing=north,lit=false,signal_fire=false,waterlogged=false]",
            &packet,
            campfire,
        );
        assert_eq!(placed.registry_id, "minecraft:campfire");
        assert_eq!(placed.property("lit"), Some("true"));
        assert_eq!(held.damage_value(), 1);
    }

    #[test]
    fn flint_and_steel_fails_against_unsupported_air_without_damaging_the_item() {
        // Clicking the side of stone: the fire cell floats over air with no
        // flammable neighbour -> canBePlacedAt false -> FAIL, item untouched.
        let packet = use_packet(0, 64, 0, Direction3d::East);
        let fire_pos = BlockPos { x: 1, y: 64, z: 0 };
        let (placed, held) = run_use("fail", "minecraft:stone", &packet, fire_pos);
        assert!(placed.is_air());
        assert_eq!(held.damage_value(), 0);
    }
}
