//! Item-function parity against the real 26.1.2 server code.
//!
//! `vanilla-data/reports/loot_item_function_runs_26_1_2.json` is produced by
//! `tools/DumpLootRuns.java`, which decodes the vanilla chest tables with the real
//! codecs (registries and tags loaded from the bundled data pack) and runs them with
//! `RandomSource.create(seed)`. Every stack the Rust model produces for the same seed
//! must equal the Java stack, which pins the exact order of random draws made by
//! `enchant_with_levels`, `enchant_randomly`, `set_damage`, `set_enchantments`,
//! `set_stew_effect`, `set_instrument` and `set_ominous_bottle_amplifier`.

use std::path::Path;
use std::sync::{Arc, LazyLock};

use serde_json::Value;

use super::*;
use crate::registry_pipeline::resources::ResourceManager;

/// The registry data a live server attaches to every loot context.
struct VanillaData {
    tags: Arc<RegistryTags>,
    enchantments: Arc<EnchantmentRegistry>,
}

static VANILLA: LazyLock<VanillaData> = LazyLock::new(|| {
    let manager = ResourceManager::vanilla();
    let mut problems = Vec::new();
    let enchantments = EnchantmentRegistry::load(&manager, &mut problems);
    assert!(problems.is_empty(), "{problems:?}");
    VanillaData {
        tags: Arc::new(RegistryTags::load(&manager)),
        enchantments: Arc::new(enchantments),
    }
});

/// A loot context with the vanilla registries attached.
fn vanilla_context(param_set: LootParamSet, seed: u64) -> LootContext {
    let mut context = LootContext::new(param_set, seed);
    context.registry_tags = VANILLA.tags.clone();
    context.enchantments = VANILLA.enchantments.clone();
    context
}

/// The table JSON path for a fixture key (`trade_rebalance:` keys name the bundled
/// datapack's tables).
fn table_path(key: &str) -> std::path::PathBuf {
    let data = Path::new(env!("CARGO_MANIFEST_DIR")).join("vanilla-data/data/minecraft");
    match key.strip_prefix("trade_rebalance:") {
        Some(id) => data
            .join("datapacks/trade_rebalance/data/minecraft/loot_table")
            .join(format!("{id}.json")),
        None => data.join("loot_table").join(format!("{key}.json")),
    }
}

/// `DumpLootRuns.describe`: the item, count and the components the item functions set.
fn describe(stack: &LootStack) -> String {
    let mut text = format!("{} x{}", stack.item, stack.count);
    let component = |key: &str| stack.components.get(key).filter(|value| !value.is_empty());
    for (label, key) in [
        ("enchantments", "minecraft:enchantments"),
        ("stored_enchantments", "minecraft:stored_enchantments"),
    ] {
        if let Some(value) = component(key) {
            text.push_str(&format!("|{label}={value}"));
        }
    }
    if stack.is_damageable_item() {
        text.push_str(&format!("|damage={}", stack.damage_value()));
    }
    for (label, key) in [
        ("ominous_bottle_amplifier", "minecraft:ominous_bottle_amplifier"),
        ("suspicious_stew_effects", "minecraft:suspicious_stew_effects"),
        ("instrument", "minecraft:instrument"),
        ("additional_trade_cost", "minecraft:additional_trade_cost"),
    ] {
        if let Some(value) = component(key) {
            text.push_str(&format!("|{label}={value}"));
        }
    }
    text
}

fn rust_run(key: &str, seed: u64) -> Vec<String> {
    let raw = std::fs::read_to_string(table_path(key)).expect("table JSON");
    let table = decode_loot_table(&raw).expect("table decodes");
    assert!(table.is_fully_modeled(), "{key}: {:?}", table.unmodeled_parts());
    let mut context = vanilla_context(LootParamSet::Chest, seed);
    context.insert_param(LootParamValue::Origin(0.0, 0.0, 0.0));
    table.evaluate(&mut context).iter().map(describe).collect()
}

#[test]
fn enchantment_registry_loads_every_vanilla_enchantment() {
    assert_eq!(VANILLA.enchantments.ids().count(), 43);
    let loyalty = VANILLA.enchantments.get("minecraft:loyalty").expect("loyalty");
    // Values from `enchantment/loyalty.json` (the pack is the authority).
    assert_eq!(loyalty.min_cost.calculate(1), 12);
    assert_eq!(loyalty.max_cost.calculate(3), 50);
}

#[test]
fn item_function_loot_runs_match_the_real_server() {
    let fixture: Value = serde_json::from_str(include_str!(
        "../../../vanilla-data/reports/loot_item_function_runs_26_1_2.json"
    ))
    .expect("fixture JSON");
    let mut compared = 0;
    for (key, by_seed) in fixture.as_object().expect("tables") {
        for (seed, expected) in by_seed.as_object().expect("seeds") {
            let expected: Vec<String> = expected
                .as_array()
                .expect("stacks")
                .iter()
                .map(|stack| stack.as_str().expect("stack text").to_string())
                .collect();
            let seed: u64 = seed.parse().expect("seed");
            assert_eq!(rust_run(key, seed), expected, "{key} seed {seed}");
            compared += 1;
        }
    }
    assert!(compared >= 150, "only {compared} runs compared");
}
