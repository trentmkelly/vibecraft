//! The bundled feature packs under `vanilla-data/data/minecraft/datapacks/`: their
//! `pack.mcmeta` parse, and the `trade_rebalance` pack loads over the vanilla data
//! through the live server-resources reload (registries, tags and loot tables).

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use serde_json::Value;

use super::world_packs::TestWorld;
use crate::registry::Identifier;
use crate::registry_pipeline::server_resources::{LoadedResources, ServerResources};
use crate::resources::{PackCompatibility, WorldDataConfiguration};

/// `decompiled-server-26.1.2/data/minecraft/datapacks`, as written in the checklist.
const CHECKLIST_PACKS: &str = "decompiled-server-26.1.2/data/minecraft/datapacks";

fn packs_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("vanilla-data/data/minecraft/datapacks")
}

/// Extension-less paths of every `.json` below `dir`, relative to `dir`.
fn json_paths(dir: &Path) -> Vec<String> {
    fn walk(dir: &Path, root: &Path, out: &mut Vec<String>) {
        for entry in fs::read_dir(dir).unwrap_or_else(|err| panic!("{}: {err}", dir.display())) {
            let path = entry.unwrap_or_else(|err| panic!("{err}")).path();
            if path.is_dir() {
                walk(&path, root, out);
            } else if path.extension().is_some_and(|ext| ext == "json") {
                let relative = path.strip_prefix(root).unwrap_or(&path);
                let text = relative.to_string_lossy().replace('\\', "/");
                out.push(text.trim_end_matches(".json").to_string());
            }
        }
    }
    let mut out = Vec::new();
    walk(dir, dir, &mut out);
    out.sort();
    out
}

fn rebalance_data(kind: &str) -> PathBuf {
    packs_root()
        .join("trade_rebalance/data/minecraft")
        .join(kind)
}

fn id(text: &str) -> Identifier {
    Identifier::parse(text).unwrap_or_else(|err| panic!("{text}: {err}"))
}

/// The resources of a world started with `vanilla` + `trade_rebalance` selected
/// (registries are only built at world load; `/reload` never adds elements).
fn rebalance_resources() -> Arc<LoadedResources> {
    let world = TestWorld::new();
    let mut repository = world.repository();
    repository.set_selected(["vanilla", "trade_rebalance"]);
    ServerResources::new(
        repository,
        WorldDataConfiguration::default_26_1_2(),
        &world.root,
    )
    .unwrap_or_else(|err| panic!("load with trade_rebalance: {err}"))
    .current()
}

#[test]
fn bundled_pack_metadata_parses_and_requests_the_feature_flag() {
    for (pack, feature) in [
        ("minecart_improvements", "minecraft:minecart_improvements"),
        ("redstone_experiments", "minecraft:redstone_experiments"),
        ("trade_rebalance", "minecraft:trade_rebalance"),
    ] {
        let text = fs::read_to_string(packs_root().join(pack).join("pack.mcmeta"))
            .unwrap_or_else(|err| panic!("{pack}: {err}"));
        let metadata = crate::resources::parse_pack_metadata(&text)
            .unwrap_or_else(|err| panic!("{pack}: {err}"));
        assert_eq!(
            metadata.compatibility,
            PackCompatibility::Compatible,
            "{pack}"
        );
        let raw: Value = serde_json::from_str(&text).expect("pack.mcmeta JSON");
        assert_eq!(raw["features"]["enabled"][0], feature, "{pack}");
    }
}

#[test]
fn trade_rebalance_villager_trades_replace_the_vanilla_registry_entries() {
    let loaded = rebalance_resources();
    let trades = loaded
        .registries
        .lookup(&id("minecraft:villager_trade"))
        .unwrap_or_else(|| panic!("villager_trade"));
    let files = json_paths(&rebalance_data("villager_trade"));
    assert_eq!(files.len(), 81);
    for file in &files {
        let element = trades
            .elements()
            .iter()
            .find(|element| element.key == id(&format!("minecraft:{file}")))
            .unwrap_or_else(|| panic!("{file} is not registered"));
        let text =
            fs::read_to_string(rebalance_data("villager_trade").join(format!("{file}.json")))
                .expect("rebalance trade");
        let source: Value = serde_json::from_str(&text).expect("trade JSON");
        assert_eq!(element.json, source, "{file} was not loaded from the pack");
    }
}

#[test]
fn trade_rebalance_tags_bind_in_their_registries() {
    let loaded = rebalance_resources();
    let mut seen = 0;
    for (registry, directory) in [
        ("minecraft:villager_trade", "tags/villager_trade"),
        ("minecraft:enchantment", "tags/enchantment"),
    ] {
        let tags = loaded
            .registries
            .lookup(&id(registry))
            .unwrap_or_else(|| panic!("{registry}"))
            .tags();
        for file in json_paths(&rebalance_data(directory)) {
            let name = file.as_str();
            let key = id(&format!("minecraft:{name}"));
            let members = tags
                .get(&key)
                .unwrap_or_else(|| panic!("{registry} tag {key} is not bound"));
            assert!(!members.is_empty(), "{registry} tag {key} is empty");
            seen += 1;
        }
    }
    assert_eq!(seen, 17);
}

/// The rebalance loot tables that replace vanilla chest tables and that the loot
/// runtime models completely.
fn modeled_loot_tables(loaded: &LoadedResources) -> BTreeSet<String> {
    json_paths(&rebalance_data("loot_table"))
        .into_iter()
        .filter(|file| {
            loaded
                .content
                .loot_tables
                .get(&format!("minecraft:{file}"))
                .is_some_and(|table| table.is_fully_modeled())
        })
        .collect()
}

#[test]
fn trade_rebalance_loot_tables_load_over_the_vanilla_chests() {
    let loaded = rebalance_resources();
    for file in json_paths(&rebalance_data("loot_table")) {
        assert!(
            loaded
                .content
                .loot_tables
                .get(&format!("minecraft:{file}"))
                .is_some(),
            "{file} did not load"
        );
    }
}

/// Checklist paths of the bundled pack files verified by the tests above.
pub(super) fn verified_paths() -> BTreeSet<String> {
    let loaded = rebalance_resources();
    let mut paths = BTreeSet::new();
    for pack in [
        "minecart_improvements",
        "redstone_experiments",
        "trade_rebalance",
    ] {
        paths.insert(format!("{CHECKLIST_PACKS}/{pack}/pack.mcmeta"));
    }
    let data = format!("{CHECKLIST_PACKS}/trade_rebalance/data/minecraft");
    for kind in ["villager_trade", "tags/villager_trade", "tags/enchantment"] {
        for file in json_paths(&rebalance_data(kind)) {
            paths.insert(format!("{data}/{kind}/{file}.json"));
        }
    }
    for file in modeled_loot_tables(&loaded) {
        paths.insert(format!("{data}/loot_table/{file}.json"));
    }
    paths
}
