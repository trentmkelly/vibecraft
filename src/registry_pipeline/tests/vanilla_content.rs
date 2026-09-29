//! The bundled vanilla `recipe/**` and `loot_table/**` resources load cleanly through
//! the live `DatapackContent` loader (the same path startup and `/reload` use).
//!
//! Java logs and skips any resource that fails to decode, so a clean load with the
//! expected counts proves every vanilla document is understood: an unsupported recipe
//! type, loot entry, function, condition or number provider fails these tests.

use std::path::Path;

use serde_json::Value;

use crate::registry::Identifier;
use crate::registry_pipeline::datapack_content::DatapackContent;
use crate::registry_pipeline::resources::ResourceManager;

/// Number of `recipe/**.json` files in the 26.1.2 server data.
const VANILLA_RECIPE_COUNT: usize = 1515;
/// Number of `loot_table/**.json` files in the 26.1.2 server data.
const VANILLA_LOOT_TABLE_COUNT: usize = 1326;

fn vanilla_content() -> (DatapackContent, Vec<String>) {
    DatapackContent::load_reporting(&ResourceManager::vanilla(), super::registries())
        .unwrap_or_else(|err| panic!("vanilla content failed to load: {err}"))
}

/// Every `*.json` below `dir`.
fn json_files(dir: &Path, found: &mut Vec<std::path::PathBuf>) {
    for entry in std::fs::read_dir(dir).unwrap_or_else(|err| panic!("{dir:?}: {err}")) {
        let path = entry.expect("directory entry").path();
        if path.is_dir() {
            json_files(&path, found);
        } else if path.extension().is_some_and(|ext| ext == "json") {
            found.push(path);
        }
    }
}

#[test]
fn all_vanilla_recipes_load_through_the_live_recipe_manager() {
    let (content, problems) = vanilla_content();
    let recipe_problems: Vec<_> = problems
        .iter()
        .filter(|problem| problem.contains("recipe"))
        .collect();
    assert!(recipe_problems.is_empty(), "{recipe_problems:#?}");
    assert_eq!(
        content.recipes.recipe_map().values().len(),
        VANILLA_RECIPE_COUNT
    );
}

#[test]
fn all_vanilla_loot_tables_decode_with_the_full_loot_codec() {
    let (content, problems) = vanilla_content();
    let loot_problems: Vec<_> = problems
        .iter()
        .filter(|problem| problem.contains("loot_table/"))
        .collect();
    assert!(loot_problems.is_empty(), "{loot_problems:#?}");

    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("vanilla-data/data/minecraft/loot_table");
    let mut files = Vec::new();
    json_files(&root, &mut files);
    assert_eq!(files.len(), VANILLA_LOOT_TABLE_COUNT);
    for file in &files {
        let relative = file
            .strip_prefix(&root)
            .expect("under the loot table root")
            .to_str()
            .expect("utf-8 loot table path");
        let id = format!("minecraft:{}", relative.trim_end_matches(".json"));
        assert!(content.loot_tables.get(&id).is_some(), "{id} did not load");
    }
}

/// Extension-less loot table ids (`minecraft:blocks/stone`) of every vendored table.
fn vendored_loot_table_ids() -> Vec<String> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("vanilla-data/data/minecraft/loot_table");
    let mut files = Vec::new();
    json_files(&root, &mut files);
    let mut ids: Vec<String> = files
        .iter()
        .map(|file| {
            let relative = file
                .strip_prefix(&root)
                .expect("under root")
                .to_string_lossy();
            format!("minecraft:{}", relative.trim_end_matches(".json"))
        })
        .collect();
    ids.sort();
    ids
}

/// Checklist paths in `tools/vanilla_data_verified_recipes_loot_tags.txt` that name a
/// loot table, as table ids.
fn verified_loot_table_ids() -> Vec<String> {
    let list = include_str!("../../../tools/vanilla_data_verified_recipes_loot_tags.txt");
    let prefix = "decompiled-server-26.1.2/data/minecraft/loot_table/";
    let mut ids: Vec<String> = list
        .lines()
        .filter_map(|line| line.strip_prefix(prefix))
        .map(|rest| format!("minecraft:{}", rest.trim_end_matches(".json")))
        .collect();
    ids.sort();
    ids
}

/// A loot table row may only be verified (and so ticked) when the runtime models the
/// whole table: the verified list is exactly the set of fully modeled vanilla tables.
#[test]
fn verified_loot_tables_are_exactly_the_fully_modeled_ones() {
    let (content, _) = vanilla_content();
    let modeled: Vec<String> = vendored_loot_table_ids()
        .into_iter()
        .filter(|id| {
            content
                .loot_tables
                .get(id)
                .is_some_and(|t| t.is_fully_modeled())
        })
        .collect();
    assert_eq!(verified_loot_table_ids(), modeled);
}

/// Every loot table row ticked in the checklist is a fully modeled table.
#[test]
fn ticked_loot_table_rows_are_fully_modeled() {
    let (content, _) = vanilla_content();
    let checklist = include_str!("../../../CHECKLIST_VANILLA_DATA_RESOURCES.md");
    let marker = "resource `decompiled-server-26.1.2/data/minecraft/loot_table/";
    let mut ticked = 0;
    for line in checklist.lines().filter(|line| line.starts_with("- [x]")) {
        let Some(rest) = line.split(marker).nth(1) else {
            continue;
        };
        let path = rest.split('`').next().expect("closing backtick");
        let id = format!("minecraft:{}", path.trim_end_matches(".json"));
        let table = content
            .loot_tables
            .get(&id)
            .unwrap_or_else(|| panic!("{id}"));
        assert!(
            table.is_fully_modeled(),
            "{id} is ticked but not fully modeled"
        );
        ticked += 1;
    }
    assert!(ticked > 1000, "only {ticked} loot table rows ticked");
}

/// Prints (with `--nocapture`) how many vanilla tables each unmodeled part blocks.
#[test]
fn unmodeled_loot_parts_are_reported_per_kind() {
    let (content, _) = vanilla_content();
    let mut counts: std::collections::BTreeMap<String, usize> = Default::default();
    for id in vendored_loot_table_ids() {
        for part in content
            .loot_tables
            .get(&id)
            .expect("table")
            .unmodeled_parts()
        {
            *counts.entry(part).or_default() += 1;
        }
    }
    for (part, tables) in &counts {
        eprintln!("unmodeled: {tables:4} tables  {part}");
    }
    // Ticking depends on this: every remaining part is a known, named kind.
    assert!(counts.keys().all(|part| part.split(' ').count() == 2));
}

/// The item registry contains every item a vanilla loot table can drop, and every
/// nested table reference points at a table the pack ships.
#[test]
fn vanilla_loot_tables_reference_only_existing_items_and_tables() {
    let items = super::registries()
        .lookup(&Identifier::parse("minecraft:item").expect("item registry key"))
        .expect("item registry");
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("vanilla-data/data/minecraft/loot_table");
    let mut files = Vec::new();
    json_files(&root, &mut files);

    let mut checked_items = 0;
    for file in files {
        let text = std::fs::read_to_string(&file).expect("loot table text");
        let table: Value = serde_json::from_str(&text).expect("loot table JSON");
        check_references(&table, items, &root, &mut checked_items);
    }
    assert!(
        checked_items > 2000,
        "only {checked_items} item entries seen"
    );
}

fn check_references(
    value: &Value,
    items: &crate::registry_pipeline::store::MappedRegistry,
    root: &Path,
    checked_items: &mut usize,
) {
    match value {
        Value::Object(object) => {
            match object.get("type").and_then(Value::as_str) {
                Some("minecraft:item") => {
                    let name = object["name"].as_str().expect("item name");
                    let id = Identifier::parse(name).expect("item id");
                    assert!(items.contains(&id), "unknown item {name}");
                    *checked_items += 1;
                }
                Some("minecraft:loot_table") => {
                    if let Some(table) = object["value"].as_str() {
                        let path = table.strip_prefix("minecraft:").unwrap_or(table);
                        assert!(
                            root.join(format!("{path}.json")).is_file(),
                            "unknown nested loot table {table}"
                        );
                    }
                }
                _ => {}
            }
            for child in object.values() {
                check_references(child, items, root, checked_items);
            }
        }
        Value::Array(values) => {
            for child in values {
                check_references(child, items, root, checked_items);
            }
        }
        _ => {}
    }
}

/// `SmeltItemFunction` looks the stack up in the live recipe manager: any smelting
/// recipe applies, the result count multiplies by the input count only when
/// `use_input_count` is set, and items without a recipe are returned unchanged.
#[test]
fn furnace_smelt_uses_the_vanilla_smelting_recipes() {
    use crate::loot_system::{LootContext, LootFunction, LootParamSet, LootStack};

    let (content, _) = vanilla_content();
    let mut context = LootContext::new(LootParamSet::AllParams, 3);
    context.recipes = Some(content.recipes.clone());
    let counted = LootFunction::SmeltItem {
        use_input_count: true,
    };
    let single = LootFunction::SmeltItem {
        use_input_count: false,
    };
    let smelted = counted
        .apply(LootStack::new("minecraft:raw_iron", 3), &mut context)
        .expect("stack");
    assert_eq!(smelted, LootStack::new("minecraft:iron_ingot", 3));
    let smelted = single
        .apply(LootStack::new("minecraft:raw_iron", 3), &mut context)
        .expect("stack");
    assert_eq!(smelted, LootStack::new("minecraft:iron_ingot", 1));
    let cooked = counted
        .apply(LootStack::new("minecraft:chicken", 2), &mut context)
        .expect("stack");
    assert_eq!(cooked, LootStack::new("minecraft:cooked_chicken", 2));
    let unchanged = counted
        .apply(LootStack::new("minecraft:stick", 2), &mut context)
        .expect("stack");
    assert_eq!(unchanged, LootStack::new("minecraft:stick", 2));
    assert_eq!(context.warnings.len(), 1);
}
