//! The advancement session wiring: join, per-tick triggers/flush, rewards, the seen
//! packet and `/advancement`.

use std::io::Read;
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::time::Duration;

use super::rewards::handle_event;
use super::*;
use crate::command::{execute_builtin_command, LevelBasedPermissionSet, ServerCommandState};
use crate::game_rules::{GameRules, LiveGameRules};
use crate::network::play::{
    CLIENTBOUND_RECIPE_BOOK_ADD_PACKET_ID, CLIENTBOUND_SET_EXPERIENCE_PACKET_ID,
};
use crate::network::varint::read_var_i32;
use crate::player_advancements::AdvancementEvent;

/// A joined player on a loopback connection.
struct Harness {
    root: PathBuf,
    layout: WorldLayout,
    profile: NameAndId,
    bus: WorldPacketBus,
    world_items: Arc<Mutex<WorldItemEntities>>,
    server: ClientStream,
    client: TcpStream,
    play_state: PlaySessionState,
    guard: Option<AdvancementSessionGuard>,
}

impl Harness {
    fn new(name: &str) -> Self {
        let root = std::env::temp_dir().join(format!(
            "vibecraft-advancements-live-{name}-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap_or_else(|e| panic!("{e}"));
        let listener = TcpListener::bind("127.0.0.1:0").unwrap_or_else(|e| panic!("{e}"));
        let client = TcpStream::connect(listener.local_addr().unwrap_or_else(|e| panic!("{e}")))
            .unwrap_or_else(|e| panic!("{e}"));
        let (server, _) = listener.accept().unwrap_or_else(|e| panic!("{e}"));
        client
            .set_read_timeout(Some(Duration::from_millis(200)))
            .unwrap_or_else(|e| panic!("{e}"));
        let layout = WorldLayout::new(&root);
        let rules = LiveGameRules::shared(GameRules::new(false));
        let profile = NameAndId::create_offline(name);
        let guard = begin_session(&profile, &layout, &rules).unwrap_or_else(|e| panic!("{e}"));
        let mut play_state = PlaySessionState::default();
        let resources = active_resources().unwrap_or_else(|e| panic!("{e}"));
        play_state
            .inventory_menu
            .replace_recipes(resources.content.recipes.recipe_map().clone());
        Self {
            root,
            layout,
            profile,
            bus: WorldPacketBus::default(),
            world_items: Arc::new(Mutex::new(WorldItemEntities::new())),
            server: ClientStream::new(server),
            client,
            play_state,
            guard: Some(guard),
        }
    }

    fn tick(&mut self) {
        tick_player_advancements(
            &mut self.server,
            CompressionState::disabled(),
            &mut self.play_state,
            &AdvancementTickContext {
                profile: &self.profile,
                bus: &self.bus,
                world_items: &self.world_items,
            },
        )
        .unwrap_or_else(|e| panic!("{e}"));
    }

    /// Every packet the client received since the last call: `(id, whole payload)`.
    fn received(&mut self) -> Vec<(i32, Vec<u8>)> {
        let mut bytes = Vec::new();
        let mut chunk = [0_u8; 8192];
        while let Ok(read) = self.client.read(&mut chunk) {
            if read == 0 {
                break;
            }
            bytes.extend_from_slice(&chunk[..read]);
        }
        let mut reader = &bytes[..];
        let mut packets = Vec::new();
        while !reader.is_empty() {
            let payload = CompressionState::disabled()
                .decode_packet(&mut reader)
                .unwrap_or_else(|e| panic!("frame: {e}"));
            let id = read_var_i32(&mut &payload[..]).unwrap_or_else(|e| panic!("id: {e}"));
            packets.push((id, payload));
        }
        packets
    }

    fn advancements(&self) -> SharedPlayerAdvancements {
        AdvancementRegistry::global()
            .get(&self.profile.uuid)
            .unwrap_or_else(|| panic!("registered"))
    }

    fn is_done(&self, advancement: &str) -> bool {
        let id = Identifier::parse(advancement).unwrap_or_else(|e| panic!("{e}"));
        lock_advancements(&self.advancements())
            .get_or_start_progress(&id)
            .is_some_and(|progress| progress.is_done())
    }
}

impl Drop for Harness {
    fn drop(&mut self) {
        self.guard = None;
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

fn contains(haystack: &[u8], needle: &str) -> bool {
    haystack.windows(needle.len()).any(|w| w == needle.as_bytes())
}

#[test]
fn the_first_tick_runs_the_tick_trigger_and_unlocks_the_starting_recipe() {
    let mut harness = Harness::new("AdvFirstTick");

    harness.tick();

    // `recipes/decorations/crafting_table` completes on `minecraft:tick` and rewards the
    // crafting table recipe, which the client learns about.
    assert!(harness.is_done("minecraft:recipes/decorations/crafting_table"));
    assert!(harness
        .play_state
        .inventory_menu
        .recipe_book_known_recipes()
        .contains(&"minecraft:crafting_table"));
    let packets = harness.received();
    assert!(packets
        .iter()
        .any(|(id, _)| *id == CLIENTBOUND_RECIPE_BOOK_ADD_PACKET_ID));
    // A finished advancement makes its ancestors visible even without a display, so the
    // first (resetting) flush carries the recipe advancement and its root.
    let update = packets
        .iter()
        .find(|(id, _)| *id == CLIENTBOUND_UPDATE_ADVANCEMENTS_PACKET_ID)
        .unwrap_or_else(|| panic!("an update_advancements packet"));
    assert_eq!(update.1[2], 1, "the first flush resets the client");
    assert!(contains(&update.1, "minecraft:recipes/decorations/crafting_table"));
    assert!(contains(&update.1, "minecraft:recipes/root"));
    assert!(!contains(&update.1, "minecraft:story/root"));
    assert!(!harness.is_done("minecraft:story/root"));

    // Nothing changed: the next tick sends nothing.
    harness.tick();
    assert!(harness.received().is_empty());
}

#[test]
fn inventory_changes_raise_triggers_and_recipe_rewards_reach_the_recipe_book() {
    let mut harness = Harness::new("AdvInventory");
    harness.tick();
    let _ = harness.received();

    harness
        .play_state
        .inventory_menu
        .player_inventory_mut()
        .set(0, ItemStack::new("minecraft:oak_planks", 4));
    harness.tick();

    // `recipes/misc/stick` (`has_planks`, tag #minecraft:planks) rewards the stick recipe.
    assert!(harness.is_done("minecraft:recipes/misc/stick"));
    assert!(harness
        .play_state
        .inventory_menu
        .recipe_book_known_recipes()
        .contains(&"minecraft:stick"));
    assert!(harness
        .received()
        .iter()
        .any(|(id, _)| *id == CLIENTBOUND_RECIPE_BOOK_ADD_PACKET_ID));

    // Picking up a crafting table completes the story root and shows its tab.
    harness
        .play_state
        .inventory_menu
        .player_inventory_mut()
        .set(1, ItemStack::new("minecraft:crafting_table", 1));
    harness.tick();
    assert!(harness.is_done("minecraft:story/root"));
    let packets = harness.received();
    let update = packets
        .iter()
        .find(|(id, _)| *id == CLIENTBOUND_UPDATE_ADVANCEMENTS_PACKET_ID)
        .unwrap_or_else(|| panic!("update"));
    // Two id bytes (130), then `reset`, and `show_advancements` last.
    assert_eq!(update.1[2], 0, "the first flush already happened");
    assert_eq!(*update.1.last().unwrap_or(&0), 1);
    assert!(contains(&update.1, "minecraft:story/root"));
}

#[test]
fn ending_the_session_saves_the_progress_and_unregisters_the_player() {
    let mut harness = Harness::new("AdvSaves");
    harness.tick();
    let uuid = harness.profile.uuid.clone();
    let path = harness.layout.advancements_file(&uuid);
    assert!(!path.exists());

    harness.guard = None;

    let saved = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{e}"));
    assert!(saved.contains("minecraft:recipes/decorations/crafting_table"));
    assert!(saved.contains("DataVersion"));
    assert!(AdvancementRegistry::global().get(&uuid).is_none());
}

#[test]
fn experience_rewards_use_the_live_experience_path() {
    let mut harness = Harness::new("AdvExperience");
    let context = RewardContext {
        profile: &harness.profile,
        bus: &harness.bus,
        world_items: &harness.world_items,
        resources: &active_resources().unwrap_or_else(|e| panic!("{e}")),
    };
    let resources = active_resources().unwrap_or_else(|e| panic!("{e}"));
    let holder = resources
        .content
        .advancements
        .get(&Identifier::parse("minecraft:nether/explore_nether").unwrap_or_else(|e| panic!("{e}")))
        .unwrap_or_else(|| panic!("explore_nether"));
    let experience = holder.value().rewards.experience;
    assert!(experience > 0);

    handle_event(
        &mut harness.server,
        CompressionState::disabled(),
        &mut harness.play_state,
        &context,
        &AdvancementEvent::GrantRewards(holder.value().id.clone()),
    )
    .unwrap_or_else(|e| panic!("{e}"));

    assert_eq!(harness.play_state.xp_total, experience);
    assert_eq!(harness.play_state.score, experience);
    assert!(harness.play_state.xp_level > 0);
    assert!(harness
        .received()
        .iter()
        .any(|(id, _)| *id == CLIENTBOUND_SET_EXPERIENCE_PACKET_ID));
}

#[test]
fn announcements_are_translatable_with_a_hoverable_bracketed_name() {
    let harness = Harness::new("AdvAnnounce");
    let inbox = harness.bus.subscribe(1);
    let resources = active_resources().unwrap_or_else(|e| panic!("{e}"));
    let context = RewardContext {
        profile: &harness.profile,
        bus: &harness.bus,
        world_items: &harness.world_items,
        resources: &resources,
    };
    let mut server = harness.server.try_clone().unwrap_or_else(|e| panic!("{e}"));
    let mut play_state = PlaySessionState::default();

    handle_event(
        &mut server,
        CompressionState::disabled(),
        &mut play_state,
        &context,
        &AdvancementEvent::Announce(
            Identifier::parse("minecraft:story/mine_stone").unwrap_or_else(|e| panic!("{e}")),
        ),
    )
    .unwrap_or_else(|e| panic!("{e}"));

    let mut framed = Vec::new();
    inbox
        .drain_into(&mut framed, CompressionState::disabled())
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(contains(&framed, "chat.type.advancement.task"));
    assert!(contains(&framed, "chat.square_brackets"));
    assert!(contains(&framed, "advancements.story.mine_stone.title"));
    assert!(contains(&framed, "show_text"));
    assert!(contains(&framed, &harness.profile.name));
}

#[test]
fn opening_a_tab_selects_it_and_echoes_the_tab_packet_once() {
    let mut harness = Harness::new("AdvSeenTab");
    harness.tick();
    let _ = harness.received();
    let packet = |tab: &str| {
        let mut bytes = Vec::new();
        write_var_i32(&mut bytes, SERVERBOUND_SEEN_ADVANCEMENTS_PACKET_ID).unwrap_or_else(|e| panic!("{e}"));
        ServerboundSeenAdvancementsPacket {
            action: ServerboundSeenAdvancementsAction::OpenedTab,
            tab: Some(Identifier::parse(tab).unwrap_or_else(|e| panic!("{e}"))),
        }
        .write(&mut bytes)
        .unwrap_or_else(|e| panic!("{e}"));
        bytes
    };
    let handle = |harness: &mut Harness, tab: &str| {
        let mut input = Cursor::new(packet(tab));
        let id = read_var_i32(&mut input).unwrap_or_else(|e| panic!("{e}"));
        assert!(try_handle_seen_advancements(
            &mut harness.server,
            CompressionState::disabled(),
            id,
            &mut input,
            &harness.profile.uuid,
        )
        .unwrap_or_else(|e| panic!("{e}")));
    };

    handle(&mut harness, "minecraft:adventure/root");
    let first = harness.received();
    assert_eq!(first.len(), 1);
    assert_eq!(first[0].0, CLIENTBOUND_SELECT_ADVANCEMENTS_TAB_PACKET_ID);
    assert!(contains(&first[0].1, "minecraft:adventure/root"));

    handle(&mut harness, "minecraft:adventure/root");
    assert!(harness.received().is_empty(), "an unchanged tab is not resent");
    // Unknown advancements and non-root ones are ignored / clear the selection.
    handle(&mut harness, "minecraft:not/an_advancement");
    assert!(harness.received().is_empty());
}

/// Runs `line` as the console through the same seed/execute/apply cycle the live
/// command runner uses.
fn run_command(harness: &Harness, line: &str) -> ServerCommandState {
    let registry = ActiveLoginRegistry::default();
    let (guard, _) = registry
        .register_replacing(&harness.profile.uuid, &harness.profile.name, &harness.server)
        .unwrap_or_else(|e| panic!("{e}"));
    guard.mark_in_play();
    let mut state = ServerCommandState {
        online_players: vec![harness.profile.clone()],
        ..ServerCommandState::default()
    };
    let seed = seed_command_advancements(&mut state, line);
    execute_builtin_command(&mut state, LevelBasedPermissionSet::OWNER, line)
        .unwrap_or_else(|e| panic!("{e:?}"));
    apply_command_advancements(seed, &state, &registry.sessions, &registry.world_bus)
        .unwrap_or_else(|e| panic!("{e}"));
    state
}

#[test]
fn advancement_grant_and_revoke_change_the_live_progress() {
    let harness = Harness::new("AdvCommand");
    assert!(!harness.is_done("minecraft:story/root"));

    run_command(&harness, "advancement grant AdvCommand only minecraft:story/root");
    assert!(harness.is_done("minecraft:story/root"));
    // The reward event was queued for the player's session to run.
    assert!(lock_advancements(&harness.advancements())
        .take_events()
        .contains(&AdvancementEvent::GrantRewards(
            Identifier::parse("minecraft:story/root").unwrap_or_else(|e| panic!("{e}"))
        )));

    run_command(&harness, "advancement revoke AdvCommand only minecraft:story/root");
    assert!(!harness.is_done("minecraft:story/root"));

    run_command(&harness, "advancement grant AdvCommand from minecraft:story/root");
    assert!(harness.is_done("minecraft:story/root"));
    assert!(harness.is_done("minecraft:story/mine_stone"));
    assert!(!harness.is_done("minecraft:nether/root"));

    run_command(&harness, "advancement revoke AdvCommand everything");
    assert!(!harness.is_done("minecraft:story/mine_stone"));
}

#[test]
fn granting_a_single_criterion_only_obtains_that_criterion() {
    let harness = Harness::new("AdvCriterion");

    run_command(
        &harness,
        "advancement grant AdvCriterion only minecraft:story/root crafting_table");

    assert!(harness.is_done("minecraft:story/root"));
}
