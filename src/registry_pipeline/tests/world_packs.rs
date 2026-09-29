//! World data packs: discovery of folders and archives in `datapacks/`, pack
//! metadata, overlays and filters, registry/tag overrides through the real
//! `PackRepository` -> `ResourceManager` path, and `/reload` semantics.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

use zip::write::SimpleFileOptions;
use zip::ZipWriter;

use crate::registry::Identifier;
use crate::registry_pipeline::resources::{FileToIdConverter, ResourceManager};
use crate::registry_pipeline::server_resources::ServerResources;
use crate::resources::{
    configure_pack_repository, discover_pack_folder, DataPackRepository, PackCompatibility,
    PackConfigureOptions, PackSource, WorldDataConfiguration,
};

/// A well-formed `pack.mcmeta` for the current data format.
const MCMETA: &str = r#"{"pack":{"description":"test","min_format":[101,1],"max_format":101}}"#;

/// A scratch world directory removed on drop.
pub(super) struct TestWorld {
    pub(super) root: PathBuf,
}

impl TestWorld {
    pub(super) fn new() -> Self {
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        let root = std::env::temp_dir().join(format!(
            "vibecraft-world-packs-{}-{}",
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(root.join("datapacks")).unwrap_or_else(|err| panic!("mkdir: {err}"));
        Self { root }
    }

    fn datapacks(&self) -> PathBuf {
        self.root.join("datapacks")
    }

    /// Writes `datapacks/<relative>`.
    fn write(&self, relative: &str, content: &str) -> &Self {
        let path = self.datapacks().join(relative);
        fs::create_dir_all(path.parent().unwrap_or(Path::new(".")))
            .unwrap_or_else(|e| panic!("{e}"));
        fs::write(path, content).unwrap_or_else(|err| panic!("write: {err}"));
        self
    }

    /// Writes `datapacks/<name>.zip` holding `entries`.
    fn write_zip(&self, name: &str, entries: &[(&str, &str)]) -> &Self {
        let file = fs::File::create(self.datapacks().join(name)).unwrap_or_else(|e| panic!("{e}"));
        let mut zip = ZipWriter::new(file);
        for (entry, content) in entries {
            zip.start_file(*entry, SimpleFileOptions::default())
                .unwrap_or_else(|e| panic!("{e}"));
            zip.write_all(content.as_bytes())
                .unwrap_or_else(|e| panic!("{e}"));
        }
        zip.finish().unwrap_or_else(|e| panic!("{e}"));
        self
    }

    /// A repository over the bundled and world packs with the configuration a fresh
    /// server start computes (new world packs are selected automatically).
    pub(super) fn repository(&self) -> DataPackRepository {
        let mut repository = DataPackRepository::server_repository(&self.datapacks())
            .unwrap_or_else(|err| panic!("repository: {err}"));
        let configured = configure_pack_repository(
            &mut repository,
            &WorldDataConfiguration::default_26_1_2(),
            PackConfigureOptions {
                init_mode: false,
                safe_mode: false,
            },
        );
        assert!(configured
            .data_packs
            .enabled
            .contains(&"vanilla".to_string()));
        repository
    }
}

impl Drop for TestWorld {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn id(text: &str) -> Identifier {
    Identifier::parse(text).unwrap_or_else(|err| panic!("{text}: {err}"))
}

fn ids(packs: &[crate::resources::DataPack]) -> Vec<&str> {
    packs.iter().map(|pack| pack.id.as_str()).collect()
}

#[test]
fn folder_discovery_lists_directories_and_archives_like_folder_repository_source() {
    let world = TestWorld::new();
    world.write("dir_pack/pack.mcmeta", MCMETA);
    world.write(
        "dir_pack/data/example/tags/thing/a.json",
        r#"{"values":[]}"#,
    );
    world.write_zip("archive.zip", &[("pack.mcmeta", MCMETA)]);
    world.write("no_meta/data/example/x.json", "{}");
    world.write("broken/pack.mcmeta", "{ not json");
    world.write("notes.txt", "hello");
    world.write_zip("no_meta_archive.zip", &[("data/example/x.json", "{}")]);
    world.write(
        "old_pack/pack.mcmeta",
        r#"{"pack":{"description":"old","pack_format":48,"supported_formats":[45,48]}}"#,
    );

    let packs = discover_pack_folder(&world.datapacks(), PackSource::World);
    // Ids are `file/<file name>` (archives keep their `.zip`); packs without valid
    // metadata and non-pack entries are skipped, incompatible packs are kept.
    assert_eq!(
        ids(&packs),
        vec!["file/archive.zip", "file/dir_pack", "file/old_pack"]
    );
    let old = packs.iter().find(|p| p.id == "file/old_pack");
    assert_eq!(
        old.map(|pack| pack.metadata.compatibility),
        Some(PackCompatibility::TooOld)
    );
    assert!(packs.iter().all(|pack| pack.source == PackSource::World));
}

#[test]
fn folder_discovery_creates_a_missing_datapacks_folder() {
    let world = TestWorld::new();
    let missing = world.root.join("fresh").join("datapacks");
    assert!(discover_pack_folder(&missing, PackSource::World).is_empty());
    assert!(missing.is_dir());
}

#[test]
fn archive_and_folder_packs_stack_by_selection_order_with_replace() {
    let world = TestWorld::new();
    world.write("a_folder/pack.mcmeta", MCMETA);
    world.write(
        "a_folder/data/example/tags/thing/t.json",
        r#"{"values":["example:folder"]}"#,
    );
    world.write_zip(
        "z_archive.zip",
        &[
            ("pack.mcmeta", MCMETA),
            (
                "data/example/tags/thing/t.json",
                r#"{"values":["example:zip"]}"#,
            ),
        ],
    );
    let repository = world.repository();
    // Discovery order is alphabetical, so the archive is selected after the folder
    // and has the higher priority.
    assert_eq!(
        repository.selected_ids(),
        vec!["vanilla", "file/a_folder", "file/z_archive.zip"]
    );
    let resources = ServerResources::new(
        repository,
        WorldDataConfiguration::default_26_1_2(),
        &world.root,
    )
    .unwrap_or_else(|err| panic!("load: {err}"));
    assert_eq!(resources.current().known_packs().len(), 1); // only vanilla/core
    let manager = resources_manager(&world, &["file/a_folder", "file/z_archive.zip"]);
    let stacks = manager.list_matching_resource_stacks(&FileToIdConverter::json("tags/thing"));
    let stack = &stacks[&id("example:tags/thing/t.json")];
    let sources: Vec<&str> = stack.iter().map(|r| r.source_pack_id()).collect();
    assert_eq!(sources, vec!["file/a_folder", "file/z_archive.zip"]);
}

/// A manager over the named repository packs, in the given order.
fn resources_manager(world: &TestWorld, pack_ids: &[&str]) -> ResourceManager {
    let repository = world.repository();
    let packs = pack_ids
        .iter()
        .map(|id| {
            let pack = repository
                .pack(id)
                .unwrap_or_else(|| panic!("missing {id}"));
            pack.content
                .open(&pack.id, &pack.metadata)
                .unwrap_or_else(|err| panic!("open {id}: {err}"))
        })
        .collect();
    ResourceManager::new(packs)
}

#[test]
fn world_pack_overrides_registry_elements_and_drops_the_known_pack() {
    let world = TestWorld::new();
    world.write("override/pack.mcmeta", MCMETA);
    world.write(
        "override/data/minecraft/chat_type/say_command.json",
        r#"{"chat":{"parameters":["content"],"translation_key":"custom.say"},
            "narration":{"parameters":["content"],"translation_key":"custom.narrate"}}"#,
    );
    world.write(
        "override/data/example/chat_type/brand_new.json",
        r#"{"chat":{"parameters":["sender","content"],"translation_key":"custom.new"},
            "narration":{"parameters":["content"],"translation_key":"custom.n"}}"#,
    );
    let resources = ServerResources::new(
        world.repository(),
        WorldDataConfiguration::default_26_1_2(),
        &world.root,
    )
    .unwrap_or_else(|err| panic!("load: {err}"));
    let current = resources.current();
    let chat_types = current
        .registries
        .lookup(&id("minecraft:chat_type"))
        .unwrap_or_else(|| panic!("chat_type registry"));
    let say = chat_types
        .elements()
        .iter()
        .find(|element| element.key == id("minecraft:say_command"))
        .unwrap_or_else(|| panic!("say_command"));
    assert_eq!(say.json["chat"]["translation_key"], "custom.say");
    assert!(
        say.info.known_pack.is_none(),
        "override is not vanilla core"
    );
    assert!(chat_types.contains(&id("example:brand_new")));
    // Untouched elements still come from the vanilla pack.
    let chat = chat_types
        .elements()
        .iter()
        .find(|element| element.key == id("minecraft:chat"))
        .unwrap_or_else(|| panic!("chat"));
    assert!(chat.info.known_pack.is_some());
}

#[test]
fn invalid_world_pack_registry_data_fails_the_load_with_the_pack_named() {
    let world = TestWorld::new();
    world.write("bad/pack.mcmeta", MCMETA);
    world.write(
        "bad/data/minecraft/chat_type/say_command.json",
        r#"{"chat":{}}"#,
    );
    let error = ServerResources::new(
        world.repository(),
        WorldDataConfiguration::default_26_1_2(),
        &world.root,
    )
    .err()
    .unwrap_or_else(|| panic!("load unexpectedly succeeded"));
    assert!(
        error.contains("Failed to parse minecraft:say_command from pack file/bad"),
        "{error}"
    );
}

#[test]
fn reload_swaps_tags_from_the_new_selection_and_keeps_elements() {
    let world = TestWorld::new();
    world.write("tags_pack/pack.mcmeta", MCMETA);
    world.write(
        "tags_pack/data/minecraft/tags/damage_type/is_fire.json",
        r#"{"replace":true,"values":["minecraft:lava"]}"#,
    );
    let mut config = WorldDataConfiguration::default_26_1_2();
    let mut repository = world.repository();
    // Start without the pack, like a server that had it disabled.
    repository.set_selected(["vanilla"]);
    config.data_packs = crate::resources::DataPackConfig::new(["vanilla"], ["file/tags_pack"]);
    let resources = ServerResources::new(repository, config, &world.root)
        .unwrap_or_else(|err| panic!("load: {err}"));
    let fire_tag = |resources: &ServerResources| {
        let current = resources.current();
        let registry = current
            .registries
            .lookup(&id("minecraft:damage_type"))
            .unwrap_or_else(|| panic!("damage_type"));
        (
            registry.len(),
            registry.tags()[&id("minecraft:is_fire")].len(),
        )
    };
    let (elements, vanilla_fire) = fire_tag(&resources);
    assert!(vanilla_fire > 1);

    let loaded = resources
        .reload(&["vanilla".to_string(), "file/tags_pack".to_string()])
        .unwrap_or_else(|err| panic!("reload: {err}"));
    assert_eq!(loaded.known_packs().len(), 1);
    assert_eq!(fire_tag(&resources), (elements, 1));
    let listing = resources.list_packs();
    assert_eq!(listing.selected, vec!["vanilla", "file/tags_pack"]);
    // `getSelectedPacks(repository, true)` disables every available unselected pack.
    assert_eq!(
        listing.disabled,
        vec![
            "minecart_improvements",
            "redstone_experiments",
            "trade_rebalance"
        ]
    );

    // Java rolls nothing back on success but keeps the old state on failure: an
    // unopenable selection (archive deleted after discovery) changes nothing.
    world.write_zip("gone.zip", &[("pack.mcmeta", MCMETA)]);
    let listing = resources.list_packs();
    assert!(listing.available.contains(&"file/gone.zip".to_string()));
    fs::remove_file(world.datapacks().join("gone.zip")).unwrap_or_else(|e| panic!("{e}"));
    let failure = resources.reload(&["vanilla".to_string(), "file/gone.zip".to_string()]);
    assert!(failure.is_err());
    assert_eq!(
        resources.list_packs().selected,
        vec!["vanilla", "file/tags_pack"]
    );
    assert_eq!(fire_tag(&resources), (elements, 1));
}

#[test]
fn reload_keeps_the_previous_tags_of_a_registry_whose_reloaded_tag_set_is_empty() {
    // `TagLoader.loadPendingTags` returns no PendingTags for an empty result, so a
    // selection without any vanilla tags leaves every registry's tags untouched.
    let world = TestWorld::new();
    let resources = ServerResources::new(
        world.repository(),
        WorldDataConfiguration::default_26_1_2(),
        &world.root,
    )
    .unwrap_or_else(|err| panic!("load: {err}"));
    let before = resources
        .current()
        .registries
        .lookup(&id("minecraft:damage_type"))
        .map(|registry| registry.tags().len());
    resources
        .reload(&[])
        .unwrap_or_else(|err| panic!("reload: {err}"));
    let after = resources
        .current()
        .registries
        .lookup(&id("minecraft:damage_type"))
        .map(|registry| registry.tags().len());
    assert_eq!(before, after);
    assert!(before.is_some_and(|count| count > 0));
}

#[test]
fn reload_persists_the_selection_into_level_dat_data_fields_only() {
    use crate::storage::nbt::Tag;
    use crate::storage::world::WorldLayout;

    let world = TestWorld::new();
    world.write("kept/pack.mcmeta", MCMETA);
    let layout = WorldLayout::new(&world.root);
    layout
        .save_level_dat(&Tag::Compound(vec![(
            "Data".to_string(),
            Tag::Compound(vec![
                ("LevelName".to_string(), Tag::String("w".to_string())),
                ("DataPacks".to_string(), Tag::Compound(vec![])),
            ]),
        )]))
        .unwrap_or_else(|err| panic!("save: {err}"));
    let resources = ServerResources::new(
        world.repository(),
        WorldDataConfiguration::default_26_1_2(),
        &world.root,
    )
    .unwrap_or_else(|err| panic!("load: {err}"));
    resources
        .reload(&["vanilla".to_string()])
        .unwrap_or_else(|err| panic!("reload: {err}"));

    let Tag::Compound(root) = layout.load_level_dat().unwrap_or_else(|e| panic!("{e}")) else {
        panic!("root is not a compound");
    };
    let Some((_, Tag::Compound(data))) = root.iter().find(|(key, _)| key == "Data") else {
        panic!("no Data");
    };
    let field = |name: &str| data.iter().find(|(key, _)| key == name).map(|(_, v)| v);
    assert_eq!(field("LevelName"), Some(&Tag::String("w".to_string())));
    let Some(Tag::Compound(packs)) = field("DataPacks") else {
        panic!("DataPacks missing: {data:?}");
    };
    let strings = |name: &str| match packs.iter().find(|(key, _)| key == name) {
        Some((_, Tag::List(items))) => items
            .iter()
            .filter_map(|item| match item {
                Tag::String(text) => Some(text.clone()),
                _ => None,
            })
            .collect::<Vec<_>>(),
        other => panic!("{name}: {other:?}"),
    };
    assert_eq!(strings("Enabled"), vec!["vanilla"]);
    assert_eq!(
        strings("Disabled"),
        vec![
            "file/kept",
            "minecart_improvements",
            "redstone_experiments",
            "trade_rebalance"
        ]
    );
}

#[test]
fn overlays_stack_above_the_pack_and_later_overlays_win() {
    let world = TestWorld::new();
    world.write(
        "layered/pack.mcmeta",
        r#"{"pack":{"description":"o","min_format":[101,1],"max_format":101},
            "overlays":{"entries":[
              {"directory":"first","min_format":101,"max_format":101},
              {"directory":"second","min_format":101,"max_format":101},
              {"directory":"other_version","min_format":90,"max_format":95}]}}"#,
    );
    world.write(
        "layered/data/example/tags/thing/t.json",
        r#"{"values":["example:base"]}"#,
    );
    world.write(
        "layered/first/data/example/tags/thing/t.json",
        r#"{"values":["example:first"]}"#,
    );
    world.write(
        "layered/second/data/example/tags/thing/t.json",
        r#"{"values":["example:second"]}"#,
    );
    world.write(
        "layered/other_version/data/example/tags/thing/u.json",
        r#"{"values":[]}"#,
    );
    let manager = resources_manager(&world, &["file/layered"]);
    let converter = FileToIdConverter::json("tags/thing");
    let listed = manager.list_matching_resources(&converter);
    assert_eq!(listed.len(), 1, "inapplicable overlays contribute nothing");
    let text = listed[&id("example:tags/thing/t.json")]
        .read_to_string()
        .unwrap_or_else(|err| panic!("read: {err}"));
    assert!(text.contains("example:second"), "{text}");
}

#[test]
fn filter_sections_hide_lower_priority_resources_in_covered_namespaces() {
    let world = TestWorld::new();
    world.write(
        "hider/pack.mcmeta",
        r#"{"pack":{"description":"f","min_format":[101,1],"max_format":101},
            "filter":{"block":[{"namespace":"minecraft","path":"tags/damage_type/is_fire\\.json"}]}}"#,
    );
    world.write(
        "hider/data/example/tags/damage_type/own.json",
        r#"{"values":[]}"#,
    );
    let manager = resources_manager(&world, &["vanilla", "file/hider"]);
    let listed = manager.list_matching_resources(&FileToIdConverter::json("tags/damage_type"));
    assert!(!listed.contains_key(&id("minecraft:tags/damage_type/is_fire.json")));
    assert!(listed.contains_key(&id("minecraft:tags/damage_type/is_projectile.json")));
    assert!(listed.contains_key(&id("example:tags/damage_type/own.json")));
}

#[test]
fn bundled_feature_packs_are_available_but_not_selected_by_default() {
    let world = TestWorld::new();
    let repository = world.repository();
    assert_eq!(repository.selected_ids(), vec!["vanilla"]);
    let trade = repository
        .pack("trade_rebalance")
        .unwrap_or_else(|| panic!("trade_rebalance"));
    assert_eq!(trade.source, PackSource::Feature);
    assert!(trade
        .requested_features
        .contains(crate::registry::feature_flags::TRADE_REBALANCE));
    // Selecting it offers the pack to clients as `minecraft:trade_rebalance`.
    let resources = ServerResources::new(
        repository,
        WorldDataConfiguration::default_26_1_2(),
        &world.root,
    )
    .unwrap_or_else(|err| panic!("load: {err}"));
    let loaded = resources
        .reload(&["vanilla".to_string(), "trade_rebalance".to_string()])
        .unwrap_or_else(|err| panic!("reload: {err}"));
    let packs: Vec<String> = loaded.known_packs().iter().map(|p| p.id.clone()).collect();
    assert_eq!(packs, vec!["core", "trade_rebalance"]);
}

/// The vanilla `plains` biome JSON with `features` replaced.
fn plains_with_features(features: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("vanilla-data/data/minecraft/worldgen/biome/plains.json");
    let mut biome: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(&path).unwrap_or_else(|err| panic!("{}: {err}", path.display())),
    )
    .unwrap_or_else(|err| panic!("plains: {err}"));
    biome["features"] = serde_json::from_str(features).unwrap_or_else(|err| panic!("{err}"));
    biome.to_string()
}

#[test]
fn biome_feature_references_resolve_into_the_placed_feature_registry() {
    // A datapack biome may use a datapack placed feature...
    let good = TestWorld::new();
    good.write("good/pack.mcmeta", MCMETA);
    good.write(
        "good/data/example/worldgen/placed_feature/mine.json",
        r#"{"feature":"minecraft:ore_coal","placement":[]}"#,
    );
    good.write(
        "good/data/example/worldgen/biome/reffing.json",
        &plains_with_features(r#"[["example:mine","minecraft:ore_coal_upper"]]"#),
    );
    let resources = ServerResources::new(
        good.repository(),
        WorldDataConfiguration::default_26_1_2(),
        &good.root,
    )
    .unwrap_or_else(|err| panic!("load: {err}"));
    let current = resources.current();
    let features = current
        .registries
        .lookup(&id("minecraft:worldgen/placed_feature"))
        .unwrap_or_else(|| panic!("placed_feature registry"));
    assert!(features.contains(&id("example:mine")));
    assert!(features.contains(&id("minecraft:ore_coal_upper")));

    // ...but an unknown reference fails the load like Java's `Unbound values`.
    let bad = TestWorld::new();
    bad.write("bad/pack.mcmeta", MCMETA);
    bad.write(
        "bad/data/example/worldgen/biome/dangling.json",
        &plains_with_features(r#"[["example:missing"]]"#),
    );
    let error = ServerResources::new(
        bad.repository(),
        WorldDataConfiguration::default_26_1_2(),
        &bad.root,
    )
    .err()
    .unwrap_or_else(|| panic!("load unexpectedly succeeded"));
    assert!(
        error.contains("Unbound values in registry ResourceKey[minecraft:root / minecraft:worldgen/placed_feature]: [example:missing]"),
        "{error}"
    );
}

#[test]
fn datapack_create_writes_a_pack_the_repository_then_discovers() {
    use crate::command::{execute_builtin_command, LevelBasedPermissionSet, ServerCommandState};

    let world = TestWorld::new();
    let mut state = ServerCommandState {
        datapack_directory: Some(world.datapacks()),
        ..ServerCommandState::default()
    };
    let created = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::OWNER,
        "datapack create made A \"quoted\" pack",
    )
    .unwrap_or_else(|err| panic!("create: {err:?}"));
    assert_eq!(created.feedback_key, "commands.datapack.create.success");
    let mcmeta = fs::read_to_string(world.datapacks().join("made/pack.mcmeta"))
        .unwrap_or_else(|err| panic!("mcmeta: {err}"));
    // Gson pretty printing with `PackFormat.minorRange()` encoded as min/max_format.
    assert!(
        mcmeta.starts_with("{\n  \"pack\": {\n    \"description\": "),
        "{mcmeta}"
    );
    assert!(
        mcmeta
            .contains("\"min_format\": [\n      101,\n      1\n    ],\n    \"max_format\": 101\n"),
        "{mcmeta}"
    );
    assert!(world.datapacks().join("made/data").is_dir());

    // The new pack parses, is compatible, and is picked up by the repository.
    let repository = world.repository();
    let made = repository
        .pack("file/made")
        .unwrap_or_else(|| panic!("file/made"));
    assert_eq!(made.metadata.compatibility, PackCompatibility::Compatible);
    // Creating it again is refused because the folder now exists.
    let again = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::OWNER,
        "datapack create made x",
    );
    assert_eq!(
        again,
        Err(crate::command::CommandError::DataPackAlreadyExists)
    );
}
