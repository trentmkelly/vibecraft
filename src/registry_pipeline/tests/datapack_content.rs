//! Reloadable data-pack content (`ReloadableServerResources`): recipes, loot tables,
//! advancements and functions read from every enabled pack through the real
//! `PackRepository` -> `ResourceManager` path, including `/reload` swaps and rollback.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::loot_system::{LootEntry, LootParamSet};
use crate::recipe_system::{load_recipe_directory, RecipeKind};
use crate::registry::Identifier;
use crate::registry_pipeline::server_resources::ServerResources;
use crate::resources::{
    configure_pack_repository, DataPackConfig, DataPackRepository, PackConfigureOptions,
    WorldDataConfiguration,
};

const MCMETA: &str = r#"{"pack":{"description":"test","min_format":[101,1],"max_format":101}}"#;

/// A scratch world whose `datapacks/content` pack overrides and adds data.
struct ContentWorld {
    root: PathBuf,
}

impl ContentWorld {
    fn new() -> Self {
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        let root = std::env::temp_dir().join(format!(
            "vibecraft-datapack-content-{}-{}",
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::Relaxed)
        ));
        let world = Self { root };
        world.write("pack.mcmeta", MCMETA);
        world
    }

    /// Writes `datapacks/content/<relative>`.
    fn write(&self, relative: &str, content: &str) -> &Self {
        let path = self.root.join("datapacks/content").join(relative);
        fs::create_dir_all(path.parent().unwrap_or(Path::new("."))).unwrap_or_else(|e| panic!("{e}"));
        fs::write(path, content).unwrap_or_else(|e| panic!("{e}"));
        self
    }

    /// A server with only the vanilla pack selected; `file/content` is available
    /// but disabled, so enabling it is a reload.
    fn server(&self) -> ServerResources {
        let mut repository = DataPackRepository::server_repository(&self.root.join("datapacks"))
            .unwrap_or_else(|e| panic!("{e}"));
        let mut config = WorldDataConfiguration::default_26_1_2();
        config.data_packs = DataPackConfig::new(["vanilla"], ["file/content"]);
        let configured = configure_pack_repository(
            &mut repository,
            &config,
            PackConfigureOptions {
                init_mode: false,
                safe_mode: false,
            },
        );
        ServerResources::new(repository, configured, &self.root).unwrap_or_else(|e| panic!("{e}"))
    }

    fn enabled() -> Vec<String> {
        vec!["vanilla".to_string(), "file/content".to_string()]
    }
}

impl Drop for ContentWorld {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn id(text: &str) -> Identifier {
    Identifier::parse(text).unwrap_or_else(|e| panic!("{text}: {e}"))
}

const STONE_LOOT: &str = r#"{
  "type": "minecraft:block",
  "random_sequence": "minecraft:blocks/stone",
  "pools": [{"rolls": 1, "entries": [{"type": "minecraft:item", "name": "minecraft:diamond"}]}]
}"#;

fn world_with_content() -> ContentWorld {
    let world = ContentWorld::new();
    world
        .write(
            "data/content/recipe/gems.json",
            r##"{"type":"minecraft:crafting_shapeless","ingredients":["#minecraft:planks"],
                "result":{"id":"minecraft:diamond","count":2}}"##,
        )
        .write(
            "data/minecraft/recipe/stick.json",
            r##"{"type":"minecraft:crafting_shaped","key":{"#":"minecraft:diamond"},
                "pattern":["#","#"],"result":{"id":"minecraft:stick","count":8}}"##,
        )
        .write("data/minecraft/recipe/acacia_boat.json", "{}")
        .write("data/minecraft/loot_table/blocks/stone.json", STONE_LOOT)
        .write(
            "data/minecraft/loot_table/blocks/dirt.json",
            r#"{"pools":[{"rolls":1,"entries":[{"type":"minecraft:item","name":"minecraft:stick",
                "functions":[{"function":"minecraft:enchanted_count_increase"}]}]}]}"#,
        )
        .write("data/content/function/hello.mcfunction", "say hello\nsay again")
        .write(
            "data/minecraft/tags/function/tick.json",
            r#"{"values":["content:hello"]}"#,
        )
        .write(
            "data/content/advancement/root.json",
            r#"{"criteria":{"tick":{"trigger":"minecraft:tick"}}}"#,
        );
    world
}

fn stick_count(resources: &ServerResources) -> Option<u32> {
    let current = resources.current();
    let holder = current.content.recipes.recipe_map().by_key("minecraft:stick")?;
    match &holder.recipe {
        RecipeKind::Shaped { result, .. } => Some(result.count),
        _ => None,
    }
}

#[test]
fn startup_recipes_come_from_the_enabled_packs_and_match_the_bundled_directory() {
    let world = ContentWorld::new();
    let resources = world.server();
    let from_packs = &resources.current().content.recipes;
    let recipe_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("vanilla-data/data/minecraft/recipe");
    let from_directory = load_recipe_directory(&recipe_dir).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(**from_packs, from_directory);
}

#[test]
fn reload_applies_recipe_overrides_additions_and_removals_from_packs() {
    let world = world_with_content();
    let resources = world.server();
    let before = resources.current();
    assert_eq!(stick_count(&resources), Some(4));
    assert!(before.content.recipes.recipe_map().by_key("content:gems").is_none());
    assert!(before.content.recipes.recipe_map().by_key("minecraft:acacia_boat").is_some());

    resources
        .reload(&ContentWorld::enabled())
        .unwrap_or_else(|e| panic!("{e}"));

    let after = resources.current();
    let recipes = after.content.recipes.recipe_map();
    // Override: the pack copy shadows the vanilla one.
    assert_eq!(stick_count(&resources), Some(8));
    // Addition, with an item tag from the tag registry in its ingredient.
    assert!(recipes.by_key("content:gems").is_some());
    // Removal: an override that does not decode is skipped, hiding the vanilla copy.
    assert!(recipes.by_key("minecraft:acacia_boat").is_none());
    assert_eq!(
        recipes.values().len(),
        before.content.recipes.recipe_map().values().len(),
        "one recipe added, one removed"
    );
    // The previous content is untouched (readers holding it keep a consistent view).
    assert_eq!(
        before.content.recipes.recipe_map().by_key("minecraft:stick").map(|h| &h.recipe)
            .and_then(|r| match r { RecipeKind::Shaped { result, .. } => Some(result.count), _ => None }),
        Some(4)
    );
}

#[test]
fn reload_loads_loot_tables_and_skips_undecodable_ones() {
    let world = world_with_content();
    let resources = world.server();
    assert!(resources.current().content.loot_tables.get("minecraft:blocks/stone").is_none());

    resources
        .reload(&ContentWorld::enabled())
        .unwrap_or_else(|e| panic!("{e}"));
    let current = resources.current();
    let stone = current
        .content
        .loot_tables
        .get("minecraft:blocks/stone")
        .unwrap_or_else(|| panic!("stone table"));
    assert_eq!(stone.param_set, LootParamSet::Block);
    assert_eq!(stone.random_sequence.as_deref(), Some("minecraft:blocks/stone"));
    assert_eq!(stone.pools.len(), 1);
    assert!(matches!(
        &stone.pools[0].entries[0],
        LootEntry::Item { item, .. } if item == "minecraft:diamond"
    ));
    // `enchanted_count_increase` is outside the modelled function set: rejected, not
    // silently stripped.
    assert!(current.content.loot_tables.get("minecraft:blocks/dirt").is_none());
}

#[test]
fn reload_loads_functions_function_tags_and_advancements() {
    let world = world_with_content();
    let resources = world.server();
    assert!(resources.current().content.functions.get_functions().is_empty());

    resources
        .reload(&ContentWorld::enabled())
        .unwrap_or_else(|e| panic!("{e}"));
    let current = resources.current();
    let functions = &current.content.functions;
    let hello = functions
        .get_function(&id("content:hello"))
        .unwrap_or_else(|| panic!("content:hello"));
    assert_eq!(hello.commands, vec!["say hello", "say again"]);
    let tick: Vec<_> = functions
        .get_tag(&id("minecraft:tick"))
        .iter()
        .map(|function| function.id.as_str())
        .collect();
    assert_eq!(tick, vec!["content:hello"]);
    assert!(current
        .content
        .advancements
        .get(&id("content:root"))
        .is_some());
}

#[test]
fn a_failed_reload_keeps_the_previous_content() {
    let world = world_with_content();
    let resources = world.server();
    resources
        .reload(&ContentWorld::enabled())
        .unwrap_or_else(|e| panic!("{e}"));
    let good = resources.current();

    // A pack discovered as an archive whose file then disappears cannot be opened:
    // `Pack::open` throws, the reload future fails and the old resources stay.
    let archive = world.root.join("datapacks/vanishing.zip");
    let mut zip = zip::ZipWriter::new(fs::File::create(&archive).unwrap_or_else(|e| panic!("{e}")));
    zip.start_file("pack.mcmeta", zip::write::SimpleFileOptions::default())
        .unwrap_or_else(|e| panic!("{e}"));
    std::io::Write::write_all(&mut zip, MCMETA.as_bytes()).unwrap_or_else(|e| panic!("{e}"));
    zip.finish().unwrap_or_else(|e| panic!("{e}"));
    assert!(resources.list_packs().available.contains(&"file/vanishing.zip".to_string()));
    fs::remove_file(&archive).unwrap_or_else(|e| panic!("{e}"));

    let mut selection = ContentWorld::enabled();
    selection.push("file/vanishing.zip".to_string());
    assert!(resources.reload(&selection).is_err());
    assert!(std::sync::Arc::ptr_eq(&good, &resources.current()));
    assert_eq!(stick_count(&resources), Some(8));
    assert_eq!(resources.list_packs().selected, ContentWorld::enabled());
}
