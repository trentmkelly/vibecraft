#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::collections::BTreeMap;
use std::path::PathBuf;

use crate::worldgen::{
    load_processor_list_registry, ParsedStructureProcessor, STRUCTURE_PROCESSOR_LISTS,
};

const PROCESSOR_LISTS_JAVA: &str =
    vibecraft_java_source!("/net/minecraft/data/worldgen/ProcessorLists.java");

fn count_occurrences(source: &str, needle: &str) -> usize {
    source.match_indices(needle).count()
}

fn assert_source_contains_all(source: &str, sentinels: &[&str]) {
    for sentinel in sentinels {
        assert!(
            source.contains(sentinel),
            "ProcessorLists.java is missing sentinel: {sentinel}"
        );
    }
}

fn vanilla_data_path(parts: &[&str]) -> PathBuf {
    let Some(source_root) = option_env!("VIBECRAFT_DECOMPILED_SOURCE_ROOT") else {
        panic!("VIBECRAFT_DECOMPILED_SOURCE_ROOT is required for Java-source parity tests");
    };
    parts.iter().fold(PathBuf::from(source_root), |path, part| path.join(part))
}

fn load_vanilla_processor_lists() -> crate::worldgen::ParsedProcessorListRegistry {
    load_processor_list_registry(vanilla_data_path(&[
        "data",
        "minecraft",
        "worldgen",
        "processor_list",
    ]))
        .expect("vanilla processor-list registry should load")
}

fn count_processor_shapes(
    processor: &ParsedStructureProcessor,
    processor_types: &mut BTreeMap<String, usize>,
    input_predicates: &mut BTreeMap<String, usize>,
    location_predicates: &mut BTreeMap<String, usize>,
) {
    *processor_types
        .entry(processor.processor_type.clone())
        .or_default() += 1;
    for rule in &processor.rules {
        *input_predicates
            .entry(rule.input_predicate_type.clone())
            .or_default() += 1;
        *location_predicates
            .entry(rule.location_predicate_type.clone())
            .or_default() += 1;
    }
    if let Some(delegate) = &processor.delegate {
        count_processor_shapes(
            delegate,
            processor_types,
            input_predicates,
            location_predicates,
        );
    }
}

fn expected_counts(entries: &[(&str, usize)]) -> BTreeMap<String, usize> {
    entries
        .iter()
        .map(|(key, count)| ((*key).to_string(), *count))
        .collect()
}

#[test]
fn processor_lists_java_bootstrap_shape_matches_decompilation() {
    assert_eq!(PROCESSOR_LISTS_JAVA.lines().count(), 773);
    assert_source_contains_all(
        PROCESSOR_LISTS_JAVA,
        &[
            "private static final ResourceKey<StructureProcessorList> EMPTY = createKey(\"empty\");",
            "public static final ResourceKey<StructureProcessorList> TRIAL_CHAMBERS_COPPER_BULB_DEGRADATION = createKey(\"trial_chambers_copper_bulb_degradation\");",
            "return ResourceKey.create(Registries.PROCESSOR_LIST, Identifier.withDefaultNamespace(name));",
            "context.register(id, new StructureProcessorList(processors));",
            "HolderGetter<Block> blocks = context.lookup(Registries.BLOCK);",
            "ProcessorRule ADD_GILDED_BLACKSTONE = new ProcessorRule(",
            "ProcessorRule REMOVE_GILDED_BLACKSTONE = new ProcessorRule(",
            "register(context, EMPTY, ImmutableList.of());",
            "register(context, OUTPOST_ROT, ImmutableList.of(new BlockRotProcessor(0.05F)));",
            "register(context, FOSSIL_ROT, ImmutableList.of(new BlockRotProcessor(0.9F), new ProtectedBlockProcessor(BlockTags.FEATURES_CANNOT_REPLACE)));",
            "new BlockRotProcessor(blocks.getOrThrow(BlockTags.ANCIENT_CITY_REPLACEABLE), 0.95F)",
            "new AxisAlignedLinearPosTest(0.0F, 0.05F, 0, 100, Direction.Axis.Y)",
            "trailsArchyLootProcessor(BuiltInLootTables.TRAIL_RUINS_ARCHAEOLOGY_COMMON, 6)",
            "trailsArchyLootProcessor(BuiltInLootTables.TRAIL_RUINS_ARCHAEOLOGY_RARE, 3)",
            "new RandomBlockMatchTest(Blocks.WAXED_COPPER_BULB, 0.33333334F)",
            "new AppendLoot(lootTable)",
        ],
    );

    assert_eq!(
        count_occurrences(PROCESSOR_LISTS_JAVA, "ResourceKey<StructureProcessorList>"),
        42
    );
    assert_eq!(
        count_occurrences(PROCESSOR_LISTS_JAVA, " = createKey(\""),
        40
    );
    assert_eq!(
        PROCESSOR_LISTS_JAVA
            .lines()
            .filter(|line| line.trim_start().starts_with("register("))
            .count(),
        40
    );
    assert_eq!(
        count_occurrences(PROCESSOR_LISTS_JAVA, "new RuleProcessor"),
        36
    );
    assert_eq!(
        count_occurrences(PROCESSOR_LISTS_JAVA, "new BlockRotProcessor"),
        6
    );
    assert_eq!(
        count_occurrences(PROCESSOR_LISTS_JAVA, "new ProtectedBlockProcessor"),
        7
    );
    assert_eq!(
        count_occurrences(PROCESSOR_LISTS_JAVA, "new CappedProcessor"),
        1
    );
}

#[test]
fn rust_processor_list_registry_order_matches_java_keys_and_vanilla_json() {
    let registry = load_vanilla_processor_lists();
    assert_eq!(STRUCTURE_PROCESSOR_LISTS.len(), 40);
    assert_eq!(registry.lists.len(), STRUCTURE_PROCESSOR_LISTS.len());

    for id in STRUCTURE_PROCESSOR_LISTS {
        let name = id
            .strip_prefix("minecraft:")
            .expect("processor-list ids should use the default namespace");
        assert!(
            PROCESSOR_LISTS_JAVA.contains(&format!("createKey(\"{name}\")")),
            "ProcessorLists.java is missing key {id}"
        );
        assert!(
            registry.lists.contains_key(*id),
            "vanilla processor-list JSON is missing {id}"
        );
    }
}

#[test]
fn vanilla_processor_list_json_shapes_match_java_bootstrap_families() {
    let registry = load_vanilla_processor_lists();
    let mut processor_types = BTreeMap::new();
    let mut input_predicates = BTreeMap::new();
    let mut location_predicates = BTreeMap::new();
    let mut processor_lengths = BTreeMap::new();

    for list in registry.lists.values() {
        *processor_lengths.entry(list.processors.len()).or_default() += 1;
        for processor in &list.processors {
            count_processor_shapes(
                processor,
                &mut processor_types,
                &mut input_predicates,
                &mut location_predicates,
            );
        }
    }

    assert_eq!(
        processor_lengths,
        BTreeMap::from([(0, 1), (1, 30), (2, 5), (3, 4)])
    );
    assert_eq!(
        processor_types,
        expected_counts(&[
            ("minecraft:block_rot", 6),
            ("minecraft:capped", 4),
            ("minecraft:protected_blocks", 7),
            ("minecraft:rule", 39),
        ])
    );
    assert_eq!(
        input_predicates,
        expected_counts(&[
            ("minecraft:always_true", 1),
            ("minecraft:block_match", 23),
            ("minecraft:blockstate_match", 8),
            ("minecraft:random_block_match", 123),
            ("minecraft:tag_match", 9),
        ])
    );
    assert_eq!(
        location_predicates,
        expected_counts(&[
            ("minecraft:always_true", 154),
            ("minecraft:block_match", 10)
        ])
    );
}

#[test]
fn representative_processor_lists_match_java_special_cases() {
    let registry = load_vanilla_processor_lists();

    let empty = registry.lists.get("minecraft:empty").unwrap();
    assert!(empty.processors.is_empty());

    let outpost_rot = registry.lists.get("minecraft:outpost_rot").unwrap();
    assert_eq!(outpost_rot.processors.len(), 1);
    assert_eq!(
        outpost_rot.processors[0].processor_type,
        "minecraft:block_rot"
    );
    assert_eq!(outpost_rot.processors[0].integrity.as_deref(), Some("0.05"));

    let fossil = registry.lists.get("minecraft:fossil_diamonds").unwrap();
    assert_eq!(
        fossil
            .processors
            .iter()
            .map(|processor| processor.processor_type.as_str())
            .collect::<Vec<_>>(),
        vec![
            "minecraft:block_rot",
            "minecraft:rule",
            "minecraft:protected_blocks"
        ]
    );
    assert_eq!(
        fossil.processors[1].rules[0].output_state_name.as_deref(),
        Some("minecraft:deepslate_diamond_ore")
    );
    assert_eq!(
        fossil.processors[2].cannot_replace.as_deref(),
        Some("#minecraft:features_cannot_replace")
    );

    let ancient_city = registry
        .lists
        .get("minecraft:ancient_city_generic_degradation")
        .unwrap();
    assert_eq!(
        ancient_city.processors[0].rottable_blocks.as_deref(),
        Some("#minecraft:ancient_city_replaceable")
    );
    assert_eq!(
        ancient_city.processors[0].integrity.as_deref(),
        Some("0.95")
    );

    let archaeology = registry
        .lists
        .get("minecraft:trail_ruins_houses_archaeology")
        .unwrap();
    let capped_limits = archaeology
        .processors
        .iter()
        .filter(|processor| processor.processor_type == "minecraft:capped")
        .map(|processor| {
            let delegate = processor.delegate.as_ref().unwrap();
            assert_eq!(delegate.processor_type, "minecraft:rule");
            assert_eq!(
                delegate.rules[0].block_entity_modifier_type.as_deref(),
                Some("minecraft:append_loot")
            );
            processor.limit.unwrap()
        })
        .collect::<Vec<_>>();
    assert_eq!(capped_limits, vec![6, 3]);

    let copper = registry
        .lists
        .get("minecraft:trial_chambers_copper_bulb_degradation")
        .unwrap();
    assert_eq!(copper.processors.len(), 2);
    assert_eq!(copper.processors[0].rules.len(), 3);
    assert_eq!(
        copper.processors[0]
            .rules
            .iter()
            .map(|rule| rule.output_state_name.as_deref().unwrap())
            .collect::<Vec<_>>(),
        vec![
            "minecraft:waxed_oxidized_copper_bulb",
            "minecraft:waxed_weathered_copper_bulb",
            "minecraft:waxed_exposed_copper_bulb",
        ]
    );
    assert_eq!(
        copper.processors[1].cannot_replace.as_deref(),
        Some("#minecraft:features_cannot_replace")
    );
}
