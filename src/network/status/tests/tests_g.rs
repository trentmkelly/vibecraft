use super::super::*;
use crate::network::play::{
    BlockHitResultPacketData, CLIENTBOUND_OPEN_SCREEN_PACKET_ID,
};

#[test]
fn use_item_on_crafting_table_opens_live_crafting_menu_with_initial_content() {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let recipe_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("vanilla-data")
        .join("data")
        .join("minecraft")
        .join("recipe");
    let recipe_manager = crate::recipe_system::load_recipe_directory(&recipe_dir).unwrap();
    let world_root = std::env::temp_dir().join(format!(
        "vibecraft-crafting-table-open-path-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&world_root);
    std::fs::create_dir_all(&world_root).unwrap();
    let layout = WorldLayout::new(&world_root);
    let cache = GeneratedChunkCache::default();
    let chunk_pos = crate::storage::region::ChunkPos { x: 0, z: 0 };
    cache.chunks.lock().unwrap().insert(
        chunk_pos,
        Arc::new(crate::storage::chunk::LevelChunk::empty(chunk_pos)),
    );
    let table_pos = crate::block_update::BlockPos { x: 0, y: 64, z: 0 };
    cache.set_block(&world_root, 42, table_pos, "minecraft:crafting_table");
    let world_items = Arc::new(Mutex::new(WorldItemEntities::default()));
    let player_access = Arc::new(Mutex::new(PlayerAccess::default()));
    let mut live_fluid_ticks = LiveFluidTicks::new();
    let mut live_block_ticks = LiveBlockTicks::new();

    let handle = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let mut state = PlaySessionState {
            x: 0.5,
            y: 64.0,
            z: 1.5,
            ..PlaySessionState::default()
        };
        let packet = ServerboundUseItemOnPacket {
            hand: ServerboundSwingHand::MainHand,
            block_hit: BlockHitResultPacketData {
                x: table_pos.x,
                y: table_pos.y,
                z: table_pos.z,
                direction: Direction3d::Up,
                click_x: 0.5,
                click_y: 1.0,
                click_z: 0.5,
                inside: false,
                world_border_hit: false,
            },
            sequence: 11,
        };
        handle_use_item_on(
            &mut stream,
            CompressionState::disabled(),
            &mut state,
            UseItemOnContext {
                world_layout: &layout,
                world_seed: 42,
                chunk_cache: &cache,
                world_items: &world_items,
                recipe_manager: &recipe_manager,
                live_fluid_ticks: &mut live_fluid_ticks,
                live_block_ticks: &mut live_block_ticks,
                game_time: 0,
                max_chained_neighbor_updates: 100_000,
                player_access: &player_access,
                profile_uuid: "00000000-0000-0000-0000-000000000000",
                spawn_protection_radius: 0,
            },
            &packet,
        )
        .unwrap();
        assert_eq!(
            state.active_block_menu.as_ref().map(ActiveBlockMenu::container_id),
            Some(1)
        );
    });

    let mut client = std::net::TcpStream::connect(addr).unwrap();
    let mut packet_ids = Vec::new();
    while let Ok(frame) = read_packet(&mut client) {
        let mut payload = &frame[..];
        packet_ids.push(read_var_i32(&mut payload).unwrap());
    }
    handle.join().unwrap();
    assert_eq!(
        packet_ids,
        vec![
            CLIENTBOUND_BLOCK_CHANGED_ACK_PACKET_ID,
            CLIENTBOUND_OPEN_SCREEN_PACKET_ID,
            CLIENTBOUND_CONTAINER_SET_CONTENT_PACKET_ID,
        ]
    );
    let _ = std::fs::remove_dir_all(world_root);
}
