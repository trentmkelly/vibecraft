//! `PlayerAdvancements` against small hand-written advancement trees.

use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

use super::triggers::{
    inventory_changed_matches, player_trigger_matches, recipe_unlocked_matches, TriggerEnvironment,
};
use super::*;
use crate::game_rules::{GameRules, LiveGameRules};
use crate::item_stack::ItemStack;
use crate::player_inventory::PlayerInventory;

/// Item tags for the tests: `#minecraft:logs` is every `*_log`.
struct TestTags;

impl TriggerEnvironment for TestTags {
    fn item_in_tag(&self, item: &str, tag: &str) -> bool {
        tag == "minecraft:logs" && item.ends_with("_log")
    }
}

fn id(value: &str) -> Identifier {
    Identifier::parse(value).unwrap_or_else(|err| panic!("{err}"))
}

/// A temporary world root removed on drop.
struct TempWorld(PathBuf);

impl TempWorld {
    fn new(name: &str) -> Self {
        let root = std::env::temp_dir().join(format!(
            "vibecraft-player-advancements-{name}-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap_or_else(|err| panic!("{err}"));
        Self(root)
    }

    fn layout(&self) -> WorldLayout {
        WorldLayout::new(&self.0)
    }
}

impl Drop for TempWorld {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

const ROOT: &str = r#"{
    "display": {
        "icon": {"id": "minecraft:stone"},
        "title": {"translate": "test.root.title"},
        "description": {"translate": "test.root.description"},
        "background": "minecraft:gui/advancements/backgrounds/stone",
        "announce_to_chat": false
    },
    "criteria": {"tick": {"trigger": "minecraft:tick"}}
}"#;

const CHILD: &str = r##"{
    "parent": "test:root",
    "display": {
        "icon": {"id": "minecraft:diamond"},
        "title": "Child",
        "description": "The child",
        "frame": "challenge"
    },
    "rewards": {"experience": 7, "recipes": ["minecraft:stick"]},
    "criteria": {
        "has_stick": {
            "trigger": "minecraft:inventory_changed",
            "conditions": {"items": [{"items": "minecraft:stick"}]}
        },
        "has_log": {
            "trigger": "minecraft:inventory_changed",
            "conditions": {"items": [{"items": "#minecraft:logs"}]}
        }
    },
    "requirements": [["has_stick", "has_log"]]
}"##;

const HIDDEN_GRANDCHILD: &str = r#"{
    "parent": "test:child",
    "display": {
        "icon": {"id": "minecraft:emerald"},
        "title": "Grandchild",
        "description": "Hidden",
        "hidden": true
    },
    "criteria": {
        "unlocked": {
            "trigger": "minecraft:recipe_unlocked",
            "conditions": {"recipe": "minecraft:stick"}
        }
    }
}"#;

fn manager(entries: &[(&str, &str)]) -> Arc<ServerAdvancementManagerModel> {
    let mut definitions = BTreeMap::new();
    for (name, json) in entries {
        let definition = AdvancementDefinition::from_json(name, json)
            .unwrap_or_else(|err| panic!("{name}: {err}"));
        definitions.insert(id(name), definition);
    }
    let mut manager = ServerAdvancementManagerModel::default();
    manager.apply(definitions);
    Arc::new(manager)
}

fn test_tree() -> Arc<ServerAdvancementManagerModel> {
    manager(&[
        ("test:root", ROOT),
        ("test:child", CHILD),
        ("test:grandchild", HIDDEN_GRANDCHILD),
    ])
}

fn player(world: &TempWorld, manager: &Arc<ServerAdvancementManagerModel>) -> PlayerAdvancements {
    PlayerAdvancements::new(PlayerAdvancementsConfig {
        manager: Arc::clone(manager),
        layout: world.layout(),
        uuid: "00000000-0000-0000-0000-000000000001".to_string(),
        game_rules: LiveGameRules::shared(GameRules::new(false)),
    })
}

#[test]
fn first_flush_is_a_reset_with_the_visible_tree_and_its_progress() {
    let world = TempWorld::new("first-flush");
    let mut advancements = player(&world, &test_tree());
    // A fresh player has nothing visible yet (Java only reveals a tab once its root is
    // obtained, e.g. by the `tick` trigger): the first flush sends no packet...
    assert!(advancements.flush_dirty(true).is_none());
    // ...and is not repeated as a reset.
    advancements.award(&id("test:root"), "tick");
    let packet = advancements
        .flush_dirty(true)
        .unwrap_or_else(|| panic!("completing the root reveals its tree"));

    assert!(!packet.reset);
    assert!(packet.show_advancements);
    // The root shows; the unfinished child is one step below a done root so it shows too;
    // the `hidden` grandchild stays out until it is obtained itself.
    let added: Vec<String> = packet.added.iter().map(|h| h.id.to_string()).collect();
    assert_eq!(added, ["test:child", "test:root"]);
    assert!(packet.removed.is_empty());
    let progressed: Vec<String> = packet.progress.iter().map(|(id, _)| id.to_string()).collect();
    assert_eq!(progressed, ["test:child", "test:root"]);
    assert!(advancements.flush_dirty(true).is_none());
}

#[test]
fn the_first_flush_after_loading_progress_is_a_reset() {
    let world = TempWorld::new("first-flush-reset");
    let manager = test_tree();
    let mut advancements = player(&world, &manager);
    advancements.award(&id("test:root"), "tick");
    advancements.save();

    let mut joined = player(&world, &manager);
    let packet = joined
        .flush_dirty(true)
        .unwrap_or_else(|| panic!("saved progress is sent on join"));
    assert!(packet.reset);
    assert_eq!(packet.added.len(), 2);
}

#[test]
fn awarding_the_last_requirement_completes_grants_rewards_and_reveals_children() {
    let world = TempWorld::new("award");
    let mut advancements = player(&world, &test_tree());
    advancements.award(&id("test:root"), "tick");
    advancements.take_events();
    let _ = advancements.flush_dirty(true);

    assert!(advancements.award(&id("test:child"), "has_stick"));
    // Already obtained: not newly awarded.
    assert!(!advancements.award(&id("test:child"), "has_stick"));
    // The requirement is an OR of both criteria, so one completes it; `announce_to_chat`
    // defaults to true and `show_advancement_messages` to true.
    assert_eq!(
        advancements.take_events(),
        [
            AdvancementEvent::GrantRewards(id("test:child")),
            AdvancementEvent::Announce(id("test:child")),
        ]
    );
    assert!(advancements
        .get_or_start_progress(&id("test:child"))
        .is_some_and(|p| p.is_done()));
    let packet = advancements
        .flush_dirty(true)
        .unwrap_or_else(|| panic!("progress changed"));
    assert!(packet.added.is_empty());
    assert_eq!(packet.progress.len(), 1);
    assert_eq!(packet.progress[0].0, id("test:child"));

    // A `hidden` advancement appears once it is obtained.
    advancements.award(&id("test:grandchild"), "unlocked");
    let packet = advancements
        .flush_dirty(true)
        .unwrap_or_else(|| panic!("the grandchild is revealed"));
    let added: Vec<String> = packet.added.iter().map(|h| h.id.to_string()).collect();
    assert_eq!(added, ["test:grandchild"]);
}

#[test]
fn completion_queues_the_announcement_only_when_the_game_rule_allows_it() {
    let world = TempWorld::new("announce");
    let manager = test_tree();
    let rules = LiveGameRules::shared(GameRules::new(false));
    let mut advancements = PlayerAdvancements::new(PlayerAdvancementsConfig {
        manager: Arc::clone(&manager),
        layout: world.layout(),
        uuid: "00000000-0000-0000-0000-000000000002".to_string(),
        game_rules: Arc::clone(&rules),
    });

    advancements.award(&id("test:child"), "has_log");
    assert_eq!(
        advancements.take_events(),
        [
            AdvancementEvent::GrantRewards(id("test:child")),
            AdvancementEvent::Announce(id("test:child")),
        ]
    );

    rules
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .set("show_advancement_messages", "false", false)
        .unwrap_or_else(|err| panic!("{err:?}"));
    advancements.revoke(&id("test:child"), "has_log");
    advancements.award(&id("test:child"), "has_log");
    assert_eq!(
        advancements.take_events(),
        [AdvancementEvent::GrantRewards(id("test:child"))]
    );

    // The root has `announce_to_chat: false`.
    advancements.award(&id("test:root"), "tick");
    assert_eq!(
        advancements.take_events(),
        [AdvancementEvent::GrantRewards(id("test:root"))]
    );
}

#[test]
fn revoking_reopens_the_criterion_and_hides_dependent_children_again() {
    let world = TempWorld::new("revoke");
    let mut advancements = player(&world, &test_tree());
    advancements.award(&id("test:root"), "tick");
    advancements.award(&id("test:child"), "has_stick");
    advancements.award(&id("test:grandchild"), "unlocked");
    let _ = advancements.flush_dirty(true);
    assert!(advancements.is_visible(&id("test:grandchild")));
    assert!(!advancements.is_listening(&id("minecraft:inventory_changed"), &id("test:child"), "has_log"));

    assert!(advancements.revoke(&id("test:child"), "has_stick"));
    assert!(!advancements.revoke(&id("test:child"), "has_stick"));
    assert!(advancements.is_listening(&id("minecraft:inventory_changed"), &id("test:child"), "has_stick"));
    assert!(advancements.revoke(&id("test:grandchild"), "unlocked"));

    let packet = advancements
        .flush_dirty(true)
        .unwrap_or_else(|| panic!("the grandchild is hidden again"));
    assert_eq!(
        packet.removed.iter().map(ToString::to_string).collect::<Vec<_>>(),
        ["test:grandchild"]
    );
}

#[test]
fn unknown_advancements_and_criteria_are_ignored() {
    let world = TempWorld::new("unknown");
    let mut advancements = player(&world, &test_tree());
    assert!(!advancements.award(&id("test:missing"), "x"));
    assert!(!advancements.award(&id("test:child"), "not_a_criterion"));
    assert!(!advancements.revoke(&id("test:missing"), "x"));
    assert!(advancements.take_events().is_empty());
}

#[test]
fn progress_round_trips_through_the_players_advancements_file() {
    let world = TempWorld::new("round-trip");
    let manager = test_tree();
    let mut advancements = player(&world, &manager);
    advancements.award(&id("test:child"), "has_stick");
    advancements.award(&id("test:root"), "tick");
    advancements.save();

    let path = world
        .layout()
        .advancements_file("00000000-0000-0000-0000-000000000001");
    let saved = fs::read_to_string(&path).unwrap_or_else(|err| panic!("{err}"));
    let json: serde_json::Value = serde_json::from_str(&saved).unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(json["DataVersion"], crate::storage::datafix::TARGET_DATA_VERSION);
    assert_eq!(json["test:child"]["done"], true);
    let obtained = json["test:child"]["criteria"]["has_stick"]
        .as_str()
        .unwrap_or_else(|| panic!("criterion time is a string"));
    assert_eq!(obtained.len(), "2024-01-02 03:04:05 +0000".len());
    // A never-progressed advancement is not written.
    assert!(json.get("test:grandchild").is_none());

    let mut reloaded = player(&world, &manager);
    assert!(reloaded
        .get_or_start_progress(&id("test:child"))
        .is_some_and(|p| p.is_done()));
    // Loaded progress is sent, and its listeners are not registered.
    assert!(!reloaded.is_listening(&id("minecraft:inventory_changed"), &id("test:child"), "has_stick"));
    let packet = reloaded.flush_dirty(true).unwrap_or_else(|| panic!("first flush"));
    assert!(packet.reset);
}

#[test]
fn unknown_saved_advancements_are_dropped_and_a_bad_file_is_not_overwritten() {
    let world = TempWorld::new("bad-file");
    let path = world
        .layout()
        .advancements_file("00000000-0000-0000-0000-000000000001");
    fs::create_dir_all(path.parent().unwrap_or_else(|| panic!("parent")))
        .unwrap_or_else(|err| panic!("{err}"));
    fs::write(
        &path,
        format!(
            r#"{{"DataVersion": {v}, "test:gone": {{"criteria": {{"x": "2024-01-01 00:00:00 +0000"}}, "done": true}},
                "test:root": {{"criteria": {{"tick": "2024-01-01 00:00:00 +0000"}}, "done": true}}}}"#,
            v = crate::storage::datafix::TARGET_DATA_VERSION
        ),
    )
    .unwrap_or_else(|err| panic!("{err}"));
    let mut advancements = player(&world, &test_tree());
    assert!(advancements
        .get_or_start_progress(&id("test:root"))
        .is_some_and(|p| p.is_done()));
    assert!(advancements.get_or_start_progress(&id("test:gone")).is_none());

    // An older DataVersion needs a datafix this server lacks: leave the file untouched.
    fs::write(&path, r#"{"DataVersion": 100, "test:root": {"criteria": {}, "done": false}}"#)
        .unwrap_or_else(|err| panic!("{err}"));
    let old = player(&world, &test_tree());
    old.save();
    assert!(fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("{err}"))
        .contains("\"DataVersion\": 100"));

    // Unparseable content loads nothing but the next save replaces it.
    fs::write(&path, "not json").unwrap_or_else(|err| panic!("{err}"));
    let broken = player(&world, &test_tree());
    broken.save();
    assert!(fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("{err}"))
        .contains("DataVersion"));
}

#[test]
fn triggers_award_only_matching_criteria_and_stop_listening_once_obtained() {
    let world = TempWorld::new("trigger");
    let mut advancements = player(&world, &test_tree());
    let inventory_changed = id("minecraft:inventory_changed");
    let mut inventory = PlayerInventory::new();
    inventory.set(0, ItemStack::new("minecraft:stone", 1));
    let stone = inventory.get(0).clone();

    advancements.trigger(&inventory_changed, |spec| {
        inventory_changed_matches(spec, &inventory, &stone, &TestTags)
    });
    assert!(advancements.take_events().is_empty());

    inventory.set(1, ItemStack::new("minecraft:stick", 1));
    let stick = inventory.get(1).clone();
    advancements.trigger(&inventory_changed, |spec| {
        inventory_changed_matches(spec, &inventory, &stick, &TestTags)
    });
    assert_eq!(
        advancements.take_events(),
        [
            AdvancementEvent::GrantRewards(id("test:child")),
            AdvancementEvent::Announce(id("test:child"))
        ]
    );
    // Done advancement: no listener left for its other criterion.
    assert!(!advancements.is_listening(&inventory_changed, &id("test:child"), "has_log"));

    advancements.trigger(&id("minecraft:tick"), player_trigger_matches);
    let stick_recipe = id("minecraft:stick");
    advancements.trigger(&id("minecraft:recipe_unlocked"), |spec| {
        recipe_unlocked_matches(spec, &stick_recipe)
    });
    assert_eq!(
        advancements.take_events(),
        [
            AdvancementEvent::GrantRewards(id("test:root")),
            AdvancementEvent::GrantRewards(id("test:grandchild")),
            AdvancementEvent::Announce(id("test:grandchild")),
        ]
    );
}

#[test]
fn item_conditions_support_ids_lists_tags_counts_and_slots() {
    let spec = |conditions: serde_json::Value| CriterionSpec {
        trigger: id("minecraft:inventory_changed"),
        conditions,
    };
    let mut inventory = PlayerInventory::new();
    inventory.set(0, ItemStack::new("minecraft:oak_log", 3));
    let log = inventory.get(0).clone();
    let matches = |inventory: &PlayerInventory, conditions| {
        inventory_changed_matches(&spec(conditions), inventory, &log, &TestTags)
    };

    assert!(matches(&inventory, serde_json::json!({})));
    assert!(matches(&inventory, serde_json::json!({"items": [{"items": ["minecraft:stone", "minecraft:oak_log"]}]})));
    assert!(matches(&inventory, serde_json::json!({"items": [{"items": "#minecraft:logs"}]})));
    assert!(matches(&inventory, serde_json::json!({"items": [{"items": "minecraft:oak_log", "count": {"min": 3}}]})));
    assert!(!matches(&inventory, serde_json::json!({"items": [{"items": "minecraft:oak_log", "count": 4}]})));
    assert!(!matches(&inventory, serde_json::json!({"items": [{"items": "minecraft:dirt"}]})));
    assert!(matches(&inventory, serde_json::json!({"slots": {"occupied": {"min": 1}}})));
    assert!(!matches(&inventory, serde_json::json!({"slots": {"occupied": {"min": 2}}})));
    // Several predicates must each be found somewhere in the inventory.
    inventory.set(5, ItemStack::new("minecraft:dirt", 1));
    assert!(matches(&inventory, serde_json::json!({"items": [{"items": "minecraft:oak_log"}, {"items": "minecraft:dirt"}]})));
    assert!(!matches(&inventory, serde_json::json!({"items": [{"items": "minecraft:oak_log"}, {"items": "minecraft:gold_ingot"}]})));
    // Conditions this evaluator cannot decode never match rather than matching wrongly.
    assert!(!matches(&inventory, serde_json::json!({"player": [{"condition": "minecraft:entity_properties"}]})));
    assert!(!matches(&inventory, serde_json::json!({"items": [{"items": "minecraft:oak_log", "components": {"minecraft:damage": 1}}]})));
}

#[test]
fn reload_forgets_listeners_and_resends_everything_against_the_new_manager() {
    let world = TempWorld::new("reload");
    let mut advancements = player(&world, &test_tree());
    advancements.award(&id("test:root"), "tick");
    let _ = advancements.flush_dirty(true);
    assert!(advancements.set_selected_tab(Some(&id("test:root"))).is_some());
    assert!(advancements.flush_dirty(true).is_none());
    advancements.save();

    advancements.reload(manager(&[("test:root", ROOT)]));

    assert!(advancements.selected_tab().is_none());
    assert!(advancements.get_or_start_progress(&id("test:child")).is_none());
    let packet = advancements
        .flush_dirty(true)
        .unwrap_or_else(|| panic!("reload resets the client"));
    assert!(packet.reset);
    assert_eq!(packet.added.len(), 1);
}

#[test]
fn only_a_displayed_root_can_be_the_selected_tab() {
    let world = TempWorld::new("tab");
    let mut advancements = player(&world, &test_tree());

    assert!(advancements.set_selected_tab(Some(&id("test:child"))).is_none());
    let selected = advancements
        .set_selected_tab(Some(&id("test:root")))
        .unwrap_or_else(|| panic!("selecting a root sends the tab"));
    assert_eq!(selected.tab, Some(id("test:root")));
    // Re-selecting the same tab is not resent; selecting a non-root clears it.
    assert!(advancements.set_selected_tab(Some(&id("test:root"))).is_none());
    let cleared = advancements
        .set_selected_tab(Some(&id("test:child")))
        .unwrap_or_else(|| panic!("clearing sends an empty tab"));
    assert_eq!(cleared.tab, None);
}

#[test]
fn the_wire_form_of_a_definition_matches_display_info_and_advancement_codecs() {
    let tree = test_tree();
    let holder = tree.get(&id("test:root")).unwrap_or_else(|| panic!("root"));
    let data = wire::holder_data(holder.value()).unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(data.id, id("test:root"));
    assert_eq!(data.value.parent, None);
    assert_eq!(data.value.requirements, vec![vec!["tick".to_string()]]);
    let payload = data.value.display_payload.unwrap_or_else(|| panic!("display"));
    // title + description (network NBT compounds), icon, frame, flags, background, x, y.
    let flags_offset = payload.len() - (4 + 2 + "minecraft:gui/advancements/backgrounds/stone".len() + 4 + 4);
    assert_eq!(&payload[flags_offset..flags_offset + 4], &[0, 0, 0, 3]);
    assert_eq!(&payload[payload.len() - 8..], &[0; 8]);
}

#[test]
fn vanilla_advancements_load_with_their_triggers_and_positions() {
    let resources = crate::registry_pipeline::server_resources::active_resources()
        .unwrap_or_else(|err| panic!("{err}"));
    let manager = &resources.content.advancements;
    assert_eq!(manager.get_all_advancements().count(), 1617);
    let stone = manager
        .get(&id("minecraft:story/mine_stone"))
        .unwrap_or_else(|| panic!("story/mine_stone"))
        .value();
    let spec = stone
        .criterion_specs
        .get("get_stone")
        .unwrap_or_else(|| panic!("criterion spec"));
    assert_eq!(spec.trigger, id("minecraft:inventory_changed"));
    let display = stone.display.as_ref().unwrap_or_else(|| panic!("display"));
    assert_eq!(display.x, 1.0);
    let root = manager.get(&id("minecraft:story/root")).unwrap_or_else(|| panic!("root"));
    assert!(root.value().display.is_some());
    // Every advancement encodes for the wire.
    for holder in manager.get_all_advancements() {
        wire::holder_data(holder.value()).unwrap_or_else(|err| panic!("{}: {err}", holder.value().id));
    }
}
