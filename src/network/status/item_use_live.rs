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
use super::chunk_d_2::{pseudo_rand_f32, write_item_entity_spawn_packets};
use super::*;
use crate::block_update::BlockPos;
use crate::item_flint_and_steel::FlintAndSteelUse;
use crate::item_tool_use::{ToolDrop, ToolUse};

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

/// Java `AxeItem` / `ShovelItem` / `HoeItem` use-on block mutations.
pub(super) fn use_block_mutation_tool<W: Write>(
    stream: &mut W,
    compression: CompressionState,
    state: &mut PlaySessionState,
    context: &mut UseItemOnContext<'_, '_>,
    packet: &crate::network::play::ServerboundUseItemOnPacket,
    held_slot: usize,
    item_name: &str,
) -> io::Result<()> {
    let clicked_pos = BlockPos {
        x: packet.block_hit.x,
        y: packet.block_hit.y,
        z: packet.block_hit.z,
    };
    let clicked_face = direction3d_to_block(packet.block_hit.direction);
    let world = LiveBlockWorld {
        layout: context.world_layout,
        seed: context.world_seed,
        cache: context.chunk_cache,
    };
    // TODO(live-sneak-tracking): Java `AxeItem.playerHasBlockingItemUseIntent`
    // returns PASS for main-hand axe use when the offhand has BLOCKS_ATTACKS
    // and the player is not using the secondary/sneak override. The live
    // server does not yet track `ServerboundPlayerCommand` shift state, so
    // that shield/offhand-specific gate is deferred with the placement-path
    // secondary-use TODO.
    let outcome = if crate::item_tool_use::is_axe(item_name) {
        crate::item_tool_use::axe_use_on(&world, clicked_pos)
    } else if crate::item_tool_use::is_shovel(item_name) {
        crate::item_tool_use::shovel_use_on(&world, clicked_pos, clicked_face)
    } else if crate::item_tool_use::is_hoe(item_name) {
        crate::item_tool_use::hoe_use_on(&world, clicked_pos, clicked_face)
    } else {
        ToolUse::Pass
    };

    let ToolUse::ChangeBlock {
        pos,
        state: new_state,
        drop,
    } = outcome
    else {
        return write_block_change_ack(stream, compression, packet.sequence);
    };

    place_block_item_in_world(context, pos, &new_state.state_name());
    write_block_change_ack(stream, compression, packet.sequence)?;
    let id = crate::block_states::network_id_for_block_state(&new_state.state_name()).unwrap_or(0);
    write_block_update(stream, compression, pos, id)?;
    if let Some(drop) = drop {
        spawn_tool_drop(stream, compression, context, pos, drop)?;
    }

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

fn spawn_tool_drop<W: Write>(
    stream: &mut W,
    compression: CompressionState,
    context: &UseItemOnContext<'_, '_>,
    pos: BlockPos,
    drop: ToolDrop,
) -> io::Result<()> {
    let Some(item_pid) = item_protocol_id(drop.item) else {
        return Ok(());
    };
    let eid = lock_status_mutex(context.world_items).alloc_entity_id();
    let (x, y, z) = drop_position_from_face(pos, drop.face);
    let item = DroppedItem {
        entity_id: eid,
        item: drop.item,
        count: 1,
        x,
        y,
        z,
        vel_x: pseudo_rand_f32(eid, 0) as f64 * 0.2 - 0.1,
        vel_y: 0.2,
        vel_z: pseudo_rand_f32(eid, 1) as f64 * 0.2 - 0.1,
        pickup_delay: DEFAULT_PICKUP_DELAY,
        age: 0,
        target_uuid: None,
    };
    write_item_entity_spawn_packets(stream, compression, &item, item_pid)?;
    lock_status_mutex(context.world_items).entities.push(item);
    Ok(())
}

fn drop_position_from_face(pos: BlockPos, face: crate::block_update::Direction) -> (f64, f64, f64) {
    let base_x = f64::from(pos.x) + 0.5;
    let base_y = f64::from(pos.y) + 0.5;
    let base_z = f64::from(pos.z) + 0.5;
    match face {
        crate::block_update::Direction::Down => (base_x, f64::from(pos.y) - 0.125, base_z),
        crate::block_update::Direction::Up => (base_x, f64::from(pos.y) + 1.125, base_z),
        crate::block_update::Direction::North => (base_x, base_y, f64::from(pos.z) - 0.125),
        crate::block_update::Direction::South => (base_x, base_y, f64::from(pos.z) + 1.125),
        crate::block_update::Direction::West => (f64::from(pos.x) - 0.125, base_y, base_z),
        crate::block_update::Direction::East => (f64::from(pos.x) + 1.125, base_y, base_z),
    }
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
        let world_items = Arc::new(Mutex::new(crate::item_entity::WorldItemEntities::new()));
        let player_access = Arc::new(Mutex::new(crate::player_access::PlayerAccess::default()));
        let recipe_manager = crate::recipe_system::RecipeManagerModel::default();
        let mut context = UseItemOnContext {
            world_layout: &layout,
            world_seed: 42,
            chunk_cache: &cache,
            world_items: &world_items,
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

    fn run_tool_use(
        test_name: &str,
        tool: &'static str,
        clicked_state: &str,
        above_state: Option<&str>,
        direction: Direction3d,
    ) -> (
        crate::block_behavior::BlockStateModel,
        ItemStack,
        Vec<crate::item_entity::DroppedItem>,
    ) {
        let root = temp_world_root(test_name);
        let layout = WorldLayout::new(&root);
        let cache = GeneratedChunkCache::default();
        let packet = use_packet(0, 64, 0, direction);
        let clicked = BlockPos { x: 0, y: 64, z: 0 };
        cache.set_block(layout.root(), 42, clicked, clicked_state);
        if let Some(above) = above_state {
            cache.set_block(layout.root(), 42, BlockPos { x: 0, y: 65, z: 0 }, above);
        }

        let mut session = PlaySessionState::default();
        session
            .inventory_menu
            .player_inventory_mut()
            .set(0, ItemStack::new(tool, 1));

        let mut fluid_ticks = LiveFluidTicks::new();
        let mut block_ticks = LiveBlockTicks::new();
        let world_items = Arc::new(Mutex::new(crate::item_entity::WorldItemEntities::new()));
        let player_access = Arc::new(Mutex::new(crate::player_access::PlayerAccess::default()));
        let recipe_manager = crate::recipe_system::RecipeManagerModel::default();
        let mut context = UseItemOnContext {
            world_layout: &layout,
            world_seed: 42,
            chunk_cache: &cache,
            world_items: &world_items,
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
        use_block_mutation_tool(
            &mut output,
            CompressionState::disabled(),
            &mut session,
            &mut context,
            &packet,
            0,
            tool,
        )
        .unwrap();

        let placed = LiveBlockWorld {
            layout: &layout,
            seed: 42,
            cache: &cache,
        }
        .state_at(clicked);
        let held = session.inventory_menu.player_inventory().get(0).clone();
        let drops = lock_status_mutex(&world_items).entities.clone();
        (placed, held, drops)
    }

    #[test]
    fn axe_tool_strips_and_damages_the_held_item_live() {
        let (placed, held, drops) = run_tool_use(
            "axe-strip",
            "minecraft:diamond_axe",
            "minecraft:oak_log[axis=x]",
            None,
            Direction3d::Up,
        );
        assert_eq!(placed.registry_id, "minecraft:stripped_oak_log");
        assert_eq!(placed.property("axis"), Some("x"));
        assert_eq!(held.damage_value(), 1);
        assert!(drops.is_empty());
    }

    #[test]
    fn shovel_tool_flattens_dirt_and_dowses_campfires_live() {
        let (path, held, _) = run_tool_use(
            "shovel-path",
            "minecraft:iron_shovel",
            "minecraft:dirt",
            None,
            Direction3d::Up,
        );
        assert_eq!(path.registry_id, "minecraft:dirt_path");
        assert_eq!(held.damage_value(), 1);

        let (campfire, held, _) = run_tool_use(
            "shovel-campfire",
            "minecraft:iron_shovel",
            "minecraft:campfire[facing=north,lit=true,signal_fire=false,waterlogged=false]",
            None,
            Direction3d::Down,
        );
        assert_eq!(campfire.registry_id, "minecraft:campfire");
        assert_eq!(campfire.property("lit"), Some("false"));
        assert_eq!(held.damage_value(), 1);
    }

    #[test]
    fn hoe_tool_tills_and_spawns_rooted_dirt_drop_live() {
        let (farmland, held, drops) = run_tool_use(
            "hoe-farmland",
            "minecraft:stone_hoe",
            "minecraft:grass_block",
            None,
            Direction3d::North,
        );
        assert_eq!(farmland.registry_id, "minecraft:farmland");
        assert_eq!(held.damage_value(), 1);
        assert!(drops.is_empty());

        let (dirt, held, drops) = run_tool_use(
            "hoe-rooted",
            "minecraft:stone_hoe",
            "minecraft:rooted_dirt",
            None,
            Direction3d::East,
        );
        assert_eq!(dirt.registry_id, "minecraft:dirt");
        assert_eq!(held.damage_value(), 1);
        assert_eq!(drops.len(), 1);
        assert_eq!(drops[0].item, "minecraft:hanging_roots");
        assert_eq!(drops[0].count, 1);
        assert!(drops[0].x > 1.0);
    }

    #[test]
    fn tool_use_passes_without_damaging_when_java_predicate_fails_live() {
        let (placed, held, drops) = run_tool_use(
            "hoe-blocked",
            "minecraft:stone_hoe",
            "minecraft:dirt",
            Some("minecraft:stone"),
            Direction3d::North,
        );
        assert_eq!(placed.registry_id, "minecraft:dirt");
        assert_eq!(held.damage_value(), 0);
        assert!(drops.is_empty());
    }
}
