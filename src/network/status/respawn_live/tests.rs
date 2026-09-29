// `PlaySessionState` has ~60 fields; tests set only the few they care about.
#![allow(clippy::field_reassign_with_default)]

use std::io::{Cursor, Read};

use super::*;
use crate::game_rules::{GameRules, LiveGameRules};
use crate::storage::chunk::LevelChunk;

/// A temp world with a stone floor at y = 63 across the four chunks around the origin, so
/// no test touches worldgen for block reads.
struct World {
    root: PathBuf,
    layout: WorldLayout,
    cache: GeneratedChunkCache,
    properties: ServerProperties,
    rules: SharedGameRules,
    items: Arc<Mutex<WorldItemEntities>>,
}

impl World {
    fn new(name: &str) -> Self {
        let root = std::env::temp_dir().join(format!("vibecraft-respawn-{name}-{}", std::process::id()));
        let cache = GeneratedChunkCache::default();
        for cx in -1..=0 {
            for cz in -1..=0 {
                let pos = ChunkPos { x: cx, z: cz };
                let mut chunk = LevelChunk::empty(pos);
                for x in 0..16 {
                    for z in 0..16 {
                        chunk.set_block_state(cx * 16 + x, 63, cz * 16 + z, "minecraft:stone");
                    }
                }
                cache.chunks.lock().unwrap().insert(pos, Arc::new(chunk));
            }
        }
        Self {
            layout: WorldLayout::new(&root),
            root,
            cache,
            properties: ServerProperties::load_or_default(Path::new(
                "/tmp/vibecraft-missing-server.properties",
            ))
            .unwrap(),
            rules: LiveGameRules::shared(GameRules::new(false)),
            items: Arc::new(Mutex::new(WorldItemEntities::new())),
        }
    }

    fn context(&self) -> RespawnContext<'_> {
        RespawnContext {
            properties: &self.properties,
            world_root: &self.root,
            world_seed: 0,
            world_layout: &self.layout,
            chunk_cache: &self.cache,
            game_rules: &self.rules,
            world_items: &self.items,
            random_seed: 42,
        }
    }

    fn place(&self, x: i32, y: i32, z: i32, state: &str) {
        // set_block returns the previous non-air block, which the tests do not need.
        let _ = self.cache.set_block(&self.root, 0, BlockPos { x, y, z }, state);
    }
}

fn spawn_data(x: i32, y: i32, z: i32, forced: bool) -> PlayerSpawnData {
    PlayerSpawnData {
        dimension: "minecraft:overworld".to_string(),
        x,
        y,
        z,
        yaw: 0.0,
        pitch: 0.0,
        forced,
    }
}

fn frames(bytes: &[u8]) -> Vec<(i32, Vec<u8>)> {
    let mut cursor = Cursor::new(bytes);
    let mut out = Vec::new();
    while (cursor.position() as usize) < bytes.len() {
        let length = read_var_i32(&mut cursor).unwrap() as usize;
        let start = cursor.position() as usize;
        let mut payload = &bytes[start..start + length];
        let id = read_var_i32(&mut payload).unwrap();
        out.push((id, payload.to_vec()));
        cursor.set_position((start + length) as u64);
    }
    out
}

#[test]
fn no_stored_respawn_uses_the_world_spawn() {
    let world = World::new("none");
    let state = PlaySessionState::default();
    assert_eq!(
        find_respawn_position_and_use_spawn_block(&state, &world.context(), true),
        RespawnTarget::WorldSpawn
    );
}

#[test]
fn respawn_in_an_unloaded_dimension_uses_the_world_spawn() {
    let world = World::new("nether");
    let mut state = PlaySessionState::default();
    let mut spawn = spawn_data(0, 64, 0, false);
    spawn.dimension = "minecraft:the_nether".to_string();
    state.spawn = Some(spawn);
    // server.getLevel(dimension) == null -> createDefault (not "missing respawn block").
    assert_eq!(
        find_respawn_position_and_use_spawn_block(&state, &world.context(), true),
        RespawnTarget::WorldSpawn
    );
}

#[test]
fn bed_spawn_resolves_beside_the_bed() {
    let world = World::new("bed");
    world.place(0, 64, 0, "minecraft:red_bed[facing=south,occupied=false,part=foot]");
    world.place(0, 64, 1, "minecraft:red_bed[facing=south,occupied=false,part=head]");
    let mut state = PlaySessionState::default();
    state.spawn = Some(spawn_data(0, 64, 0, false));
    let RespawnTarget::Block(found) =
        find_respawn_position_and_use_spawn_block(&state, &world.context(), true)
    else {
        panic!("bed should resolve");
    };
    assert_eq!(found.position.1, 64.0);
    assert_ne!((found.position.0, found.position.2), (0.5, 0.5));
}

#[test]
fn removed_bed_is_a_missing_respawn_block_but_forced_spawn_still_works() {
    let world = World::new("missing");
    let mut state = PlaySessionState::default();
    state.spawn = Some(spawn_data(0, 64, 0, false));
    assert_eq!(
        find_respawn_position_and_use_spawn_block(&state, &world.context(), true),
        RespawnTarget::MissingRespawnBlock
    );
    state.spawn = Some(spawn_data(0, 64, 0, true));
    let RespawnTarget::Block(found) =
        find_respawn_position_and_use_spawn_block(&state, &world.context(), true)
    else {
        panic!("forced spawn in free space resolves");
    };
    assert_eq!(found.position, (0.5, 64.1, 0.5));
}

fn loopback() -> (TcpStream, TcpStream) {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let client = TcpStream::connect(listener.local_addr().unwrap()).unwrap();
    let (server, _) = listener.accept().unwrap();
    (server, client)
}

#[test]
fn respawn_with_missing_bed_sends_event_first_and_forgets_the_spawn() {
    let world = World::new("e2e");
    let (mut server, mut client) = loopback();
    let mut state = PlaySessionState::default();
    state.health = 0.0;
    state.xp_level = 5;
    state.spawn = Some(spawn_data(0, 64, 0, false));
    state
        .inventory_menu
        .player_inventory_mut()
        .set(0, ItemStack::new("minecraft:cobblestone", 3));
    super::super::handle_play_respawn_request(
        &mut server,
        CompressionState::disabled(),
        &mut state,
        &world.context(),
    )
    .unwrap();
    drop(server);
    let mut wire = Vec::new();
    client.read_to_end(&mut wire).unwrap();
    let sent = frames(&wire);

    // Java PlayerList.respawn: NO_RESPAWN_BLOCK_AVAILABLE (game event 0) precedes the respawn.
    assert_eq!(sent[0].0, CLIENTBOUND_GAME_EVENT_PACKET_ID);
    assert_eq!(sent[0].1[0], NO_RESPAWN_BLOCK_AVAILABLE_EVENT);
    assert_eq!(sent[1].0, CLIENTBOUND_RESPAWN_PACKET_ID);
    // initInventoryMenu: the emptied inventory is resent.
    assert!(sent.iter().any(|(id, _)| *id == CLIENTBOUND_CONTAINER_SET_CONTENT_PACKET_ID));

    assert_eq!(state.spawn, None, "missing block clears the respawn config");
    assert_eq!(state.health, 20.0);
    assert_eq!((state.xp_level, state.xp_total), (0, 0));
    assert!(state.inventory_menu.player_inventory().get(0).is_empty());
    assert!(!state.combat.dead);
}

#[test]
fn respawn_with_working_bed_keeps_config_and_looks_at_the_bed() {
    let world = World::new("e2e-bed");
    world.place(0, 64, 0, "minecraft:red_bed[facing=south,occupied=false,part=foot]");
    world.place(0, 64, 1, "minecraft:red_bed[facing=south,occupied=false,part=head]");
    let (mut server, mut client) = loopback();
    let mut state = PlaySessionState::default();
    state.health = 0.0;
    state.spawn = Some(spawn_data(0, 64, 0, false));
    super::super::handle_play_respawn_request(
        &mut server,
        CompressionState::disabled(),
        &mut state,
        &world.context(),
    )
    .unwrap();
    drop(server);
    let mut wire = Vec::new();
    client.read_to_end(&mut wire).unwrap();
    assert_eq!(frames(&wire)[0].0, CLIENTBOUND_RESPAWN_PACKET_ID, "no missing-block event");
    assert!(state.spawn.is_some(), "copyRespawnPosition keeps the config");
    assert_eq!(state.y, 64.0);
    assert_eq!(state.health, 20.0);
}

#[test]
fn keep_inventory_respawn_keeps_items_experience_and_score() {
    let mut state = PlaySessionState::default();
    state.xp_level = 12;
    state.xp_total = 300;
    state.score = 300;
    state
        .inventory_menu
        .player_inventory_mut()
        .set(0, ItemStack::new("minecraft:apple", 4));
    state.active_effects = vec![Tag::Compound(vec![])];
    state.combat.dead = true;
    restore_player_after_death(&mut state, true);
    assert_eq!(state.inventory_menu.player_inventory().get(0).count(), 4);
    assert_eq!((state.xp_level, state.xp_total, state.score), (12, 300, 300));
    assert!(state.active_effects.is_empty(), "effects are not carried over");
    assert!(!state.combat.dead);
    assert_eq!(state.health, 20.0);
}

#[test]
fn normal_respawn_wipes_inventory_and_experience() {
    let mut state = PlaySessionState::default();
    state.xp_level = 12;
    state.xp_total = 300;
    state.score = 300;
    state
        .inventory_menu
        .player_inventory_mut()
        .set(0, ItemStack::new("minecraft:apple", 4));
    restore_player_after_death(&mut state, false);
    assert!(state.inventory_menu.player_inventory().get(0).is_empty());
    assert_eq!((state.xp_level, state.xp_total, state.score), (0, 0, 0));
}

#[test]
fn spectator_respawn_keeps_everything_like_restore_from() {
    let mut state = PlaySessionState::default();
    state.game_mode = GameMode::Spectator;
    state.xp_level = 3;
    state
        .inventory_menu
        .player_inventory_mut()
        .set(0, ItemStack::new("minecraft:apple", 4));
    restore_player_after_death(&mut state, false);
    assert_eq!(state.inventory_menu.player_inventory().get(0).count(), 4);
    assert_eq!(state.xp_level, 3);
}

#[test]
fn cursor_and_crafting_items_drop_at_the_dead_players_position() {
    let world = World::new("leftovers");
    let mut state = PlaySessionState::default();
    state.x = 4.0;
    state.y = 70.0;
    state.z = 5.0;
    state.carried_item = ItemStack::new("minecraft:dirt", 7);
    let mut wire = Vec::new();
    drop_menu_leftovers(&mut wire, CompressionState::disabled(), &mut state, &world.context())
        .unwrap();
    assert!(state.carried_item.is_empty());
    let items = world.items.lock().unwrap();
    assert_eq!(items.entities.len(), 1);
    let dropped = &items.entities[0];
    assert_eq!((dropped.item, dropped.count), ("minecraft:dirt", 7));
    assert!((dropped.y - (70.0 + 1.62 - 0.3)).abs() < 1e-9);
    // yaw 0 / pitch 0 faces +z: the throw is 0.3 along +z plus <= 0.02 of scatter.
    assert!((dropped.vel_z - 0.3).abs() <= 0.02 + 1e-9);
    assert_eq!(dropped.pickup_delay, 40);
    assert!(!wire.is_empty(), "the spawn is announced to the respawning client");
}

#[test]
fn hardcore_respawn_becomes_spectator_and_disables_chunk_generation() {
    let mut world = World::new("hardcore");
    world.properties.hardcore = true;
    let (mut server, mut client) = loopback();
    let mut state = PlaySessionState::default();
    apply_hardcore_respawn(&mut server, CompressionState::disabled(), &mut state, &world.context())
        .unwrap();
    drop(server);
    let mut wire = Vec::new();
    client.read_to_end(&mut wire).unwrap();
    let sent = frames(&wire);
    assert_eq!(state.game_mode, GameMode::Spectator);
    assert_eq!(state.previous_game_mode, Some(GameMode::Survival));
    assert_eq!(sent[0].0, CLIENTBOUND_GAME_EVENT_PACKET_ID);
    assert_eq!(sent[0].1[0], 3, "CHANGE_GAME_MODE");
    assert_eq!(sent[1].0, CLIENTBOUND_PLAYER_ABILITIES_PACKET_ID);
    assert!(!lock_status_mutex(&world.rules).bool("spectators_generate_chunks"));
}

#[test]
fn non_hardcore_respawn_leaves_game_mode_alone() {
    let world = World::new("softcore");
    let mut state = PlaySessionState::default();
    let mut wire = Vec::new();
    apply_hardcore_respawn(&mut wire, CompressionState::disabled(), &mut state, &world.context())
        .unwrap();
    assert!(wire.is_empty());
    assert_eq!(state.game_mode, GameMode::Survival);
    assert!(lock_status_mutex(&world.rules).bool("spectators_generate_chunks"));
}

#[test]
fn respawn_config_persists_as_the_26_1_2_respawn_compound() {
    let mut state = PlaySessionState::default();
    state.spawn = Some(PlayerSpawnData {
        dimension: "minecraft:overworld".to_string(),
        x: 11,
        y: 72,
        z: -13,
        yaw: 90.0,
        pitch: -15.0,
        forced: true,
    });
    let tag = play_session_state_to_nbt(&state);
    let Tag::Compound(root) = &tag else { panic!("compound") };
    let get = |fields: &[(String, Tag)], key: &str| fields.iter().find(|(k, _)| k == key).map(|(_, v)| v.clone());
    assert!(get(root, "SpawnX").is_none(), "legacy keys are no longer written");
    let Some(Tag::Compound(respawn)) = get(root, "respawn") else { panic!("respawn compound") };
    assert_eq!(get(&respawn, "dimension"), Some(Tag::String("minecraft:overworld".into())));
    assert_eq!(get(&respawn, "pos"), Some(Tag::IntArray(vec![11, 72, -13])));
    assert_eq!(get(&respawn, "yaw"), Some(Tag::Float(90.0)));
    assert_eq!(get(&respawn, "pitch"), Some(Tag::Float(-15.0)));
    assert_eq!(get(&respawn, "forced"), Some(Tag::Byte(1)));

    let restored = play_session_state_from_nbt(&tag, GameMode::Survival, &RecipeMap::default()).unwrap();
    assert_eq!(restored.spawn, state.spawn);
}

#[test]
fn unforced_respawn_omits_the_forced_field_like_optional_field_of() {
    let mut state = PlaySessionState::default();
    state.spawn = Some(spawn_data(1, 2, 3, false));
    let tag = play_session_state_to_nbt(&state);
    let Tag::Compound(root) = &tag else { panic!("compound") };
    let Some((_, Tag::Compound(respawn))) = root.iter().find(|(k, _)| k == "respawn") else {
        panic!("respawn compound")
    };
    assert!(respawn.iter().all(|(key, _)| key != "forced"));
    let restored = play_session_state_from_nbt(&tag, GameMode::Survival, &RecipeMap::default()).unwrap();
    assert_eq!(restored.spawn, state.spawn);
}

#[test]
fn legacy_spawn_keys_still_load() {
    let mut state = PlaySessionState::default();
    state.spawn = Some(spawn_data(1, 2, 3, true));
    let Tag::Compound(mut root) = play_session_state_to_nbt(&state) else { panic!("compound") };
    root.retain(|(key, _)| key != "respawn");
    root.extend([
        ("SpawnX".to_string(), Tag::Int(1)),
        ("SpawnY".to_string(), Tag::Int(2)),
        ("SpawnZ".to_string(), Tag::Int(3)),
        ("SpawnForced".to_string(), Tag::Byte(1)),
    ]);
    let restored =
        play_session_state_from_nbt(&Tag::Compound(root), GameMode::Survival, &RecipeMap::default())
            .unwrap();
    assert_eq!(restored.spawn, state.spawn);
}

#[test]
fn last_death_location_persists_as_an_int_array_and_reads_old_lists() {
    let mut state = PlaySessionState::default();
    state.last_death_location = Some(PlayerGlobalPosData {
        dimension: "minecraft:overworld".to_string(),
        x: 3,
        y: 65,
        z: 4,
    });
    let tag = play_session_state_to_nbt(&state);
    let Tag::Compound(root) = &tag else { panic!("compound") };
    let Some((_, Tag::Compound(death))) = root.iter().find(|(k, _)| k == "LastDeathLocation") else {
        panic!("LastDeathLocation")
    };
    assert!(death.contains(&("pos".to_string(), Tag::IntArray(vec![3, 65, 4]))));
    let restored = play_session_state_from_nbt(&tag, GameMode::Survival, &RecipeMap::default()).unwrap();
    assert_eq!(restored.last_death_location, state.last_death_location);

    // Saves written before this fix stored the position as a list.
    let Tag::Compound(mut old_root) = tag else { panic!("compound") };
    for (key, value) in &mut old_root {
        if key == "LastDeathLocation" {
            *value = Tag::Compound(vec![
                ("dimension".to_string(), Tag::String("minecraft:overworld".to_string())),
                ("pos".to_string(), Tag::List(vec![Tag::Int(3), Tag::Int(65), Tag::Int(4)])),
            ]);
        }
    }
    let restored =
        play_session_state_from_nbt(&Tag::Compound(old_root), GameMode::Survival, &RecipeMap::default())
            .unwrap();
    assert_eq!(restored.last_death_location, state.last_death_location);
}
