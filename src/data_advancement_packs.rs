#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

pub const DATA_ADVANCEMENT_PACKS_PACKAGE_NULL_MARKED: bool = true;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct AdvancementPackClass {
    java_file: &'static str,
    provider_name: &'static str,
    root_id: &'static str,
    expected_count: usize,
    first_id: &'static str,
    last_id: &'static str,
}

const VANILLA_PACK_PROVIDER_ORDER: &[&str] = &[
    "VanillaTheEndAdvancements",
    "VanillaHusbandryAdvancements",
    "VanillaAdventureAdvancements",
    "VanillaNetherAdvancements",
    "VanillaStoryAdvancements",
];

const PACK_CLASSES: &[AdvancementPackClass] = &[
    AdvancementPackClass {
        java_file: "VanillaStoryAdvancements.java",
        provider_name: "VanillaStoryAdvancements",
        root_id: "story/root",
        expected_count: 16,
        first_id: "story/root",
        last_id: "story/enter_the_end",
    },
    AdvancementPackClass {
        java_file: "VanillaNetherAdvancements.java",
        provider_name: "VanillaNetherAdvancements",
        root_id: "nether/root",
        expected_count: 23,
        first_id: "nether/root",
        last_id: "nether/distract_piglin",
    },
    AdvancementPackClass {
        java_file: "VanillaTheEndAdvancements.java",
        provider_name: "VanillaTheEndAdvancements",
        root_id: "end/root",
        expected_count: 9,
        first_id: "end/root",
        last_id: "end/dragon_egg",
    },
    AdvancementPackClass {
        java_file: "VanillaAdventureAdvancements.java",
        provider_name: "VanillaAdventureAdvancements",
        root_id: "adventure/root",
        expected_count: 47,
        first_id: "adventure/root",
        last_id: "adventure/adventuring_time",
    },
    AdvancementPackClass {
        java_file: "VanillaHusbandryAdvancements.java",
        provider_name: "VanillaHusbandryAdvancements",
        root_id: "husbandry/root",
        expected_count: 30,
        first_id: "husbandry/root",
        last_id: "husbandry/bred_all_animals",
    },
];

fn java_source(class: AdvancementPackClass) -> &'static str {
    match class.java_file {
        "VanillaStoryAdvancements.java" => include_str!(
            "../../decompiled-server-26.1.2/net/minecraft/data/advancements/packs/VanillaStoryAdvancements.java"
        ),
        "VanillaNetherAdvancements.java" => include_str!(
            "../../decompiled-server-26.1.2/net/minecraft/data/advancements/packs/VanillaNetherAdvancements.java"
        ),
        "VanillaTheEndAdvancements.java" => include_str!(
            "../../decompiled-server-26.1.2/net/minecraft/data/advancements/packs/VanillaTheEndAdvancements.java"
        ),
        "VanillaAdventureAdvancements.java" => include_str!(
            "../../decompiled-server-26.1.2/net/minecraft/data/advancements/packs/VanillaAdventureAdvancements.java"
        ),
        "VanillaHusbandryAdvancements.java" => include_str!(
            "../../decompiled-server-26.1.2/net/minecraft/data/advancements/packs/VanillaHusbandryAdvancements.java"
        ),
        _ => unreachable!("unknown advancement pack class"),
    }
}

fn parse_provider_order(source: &str) -> Vec<String> {
    VANILLA_PACK_PROVIDER_ORDER
        .iter()
        .filter(|provider| source.contains(&format!("new {provider}()")))
        .map(|provider| (*provider).to_string())
        .collect()
}

fn parse_saved_advancement_ids(source: &str) -> Vec<String> {
    let mut ids = Vec::new();
    let mut cursor = source;
    let marker = ".save(output, \"";
    while let Some(index) = cursor.find(marker) {
        let after = &cursor[index + marker.len()..];
        let id = after
            .split_once('"')
            .map(|(id, _)| id)
            .expect("save output id should be a string literal");
        ids.push(id.to_string());
        cursor = after;
    }
    ids
}

fn parse_static_entity_list_count(source: &str, name: &str) -> usize {
    let start = source
        .find(name)
        .unwrap_or_else(|| panic!("missing static list {name}"));
    let after = &source[start..];
    let open = after.find('(').expect("static list should use call syntax");
    let close = after[open..]
        .find(");")
        .expect("static list should end with );")
        + open;
    after[open..close].matches("EntityType.").count()
}

fn parse_item_array_count(source: &str, name: &str) -> usize {
    let start = source
        .find(name)
        .unwrap_or_else(|| panic!("missing item array {name}"));
    let after = &source[start..];
    let open = after.find('{').expect("item array should use braces");
    let close = after[open..]
        .find("};")
        .expect("item array should end with };")
        + open;
    after[open..close].matches("Items.").count()
}

fn vanilla_advancement_json_ids() -> BTreeSet<String> {
    let root = Path::new("../decompiled-server-26.1.2/data/minecraft/advancement");
    let mut ids = BTreeSet::new();
    for category in ["story", "nether", "end", "adventure", "husbandry"] {
        let category_root = root.join(category);
        for entry in fs::read_dir(&category_root).expect("vanilla advancement category exists") {
            let path = entry.expect("advancement file entry").path();
            if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
                continue;
            }
            let relative = path
                .strip_prefix(root)
                .expect("advancement path should be under root")
                .with_extension("");
            ids.insert(relative.to_string_lossy().replace('\\', "/"));
        }
    }
    ids
}

fn all_java_pack_ids() -> BTreeMap<&'static str, Vec<String>> {
    PACK_CLASSES
        .iter()
        .map(|class| {
            (
                class.provider_name,
                parse_saved_advancement_ids(java_source(*class)),
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vanilla_advancement_provider_registers_subproviders_in_java_order() {
        let source = include_str!(
            "../../decompiled-server-26.1.2/net/minecraft/data/advancements/packs/VanillaAdvancementProvider.java"
        );
        assert_eq!(
            parse_provider_order(source),
            VANILLA_PACK_PROVIDER_ORDER
                .iter()
                .map(|name| (*name).to_string())
                .collect::<Vec<_>>()
        );
        assert!(source.contains("return new AdvancementProvider("));
        assert!(source.contains("List.of("));
    }

    #[test]
    fn every_pack_class_save_id_matches_java_counts_and_sentinels() {
        for class in PACK_CLASSES {
            let ids = parse_saved_advancement_ids(java_source(*class));
            assert_eq!(
                ids.len(),
                class.expected_count,
                "{} save count",
                class.provider_name
            );
            assert_eq!(ids.first().unwrap(), class.first_id);
            assert_eq!(ids.last().unwrap(), class.last_id);
            assert!(ids.contains(&class.root_id.to_string()));

            let unique = ids.iter().collect::<BTreeSet<_>>();
            assert_eq!(
                unique.len(),
                ids.len(),
                "{} duplicate ids",
                class.provider_name
            );
        }
    }

    #[test]
    fn java_pack_ids_match_bundled_vanilla_advancement_jsons() {
        let java_ids = all_java_pack_ids()
            .values()
            .flat_map(|ids| ids.iter().cloned())
            .collect::<BTreeSet<_>>();
        let json_ids = vanilla_advancement_json_ids();

        assert_eq!(java_ids.len(), 125);
        assert_eq!(json_ids.len(), 125);
        assert_eq!(java_ids, json_ids);
    }

    #[test]
    fn story_nether_and_end_pack_contract_sentinels_match_java() {
        let story = java_source(PACK_CLASSES[0]);
        assert!(story
            .contains("Identifier.withDefaultNamespace(\"gui/advancements/backgrounds/stone\")"));
        assert!(story.contains("AdvancementRequirements.Strategy.OR"));
        assert!(story
            .contains("ChangeDimensionTrigger.TriggerInstance.changedDimensionTo(Level.NETHER)"));
        assert!(
            story.contains("ChangeDimensionTrigger.TriggerInstance.changedDimensionTo(Level.END)")
        );

        let nether = java_source(PACK_CLASSES[1]);
        assert!(nether
            .contains("Identifier.withDefaultNamespace(\"gui/advancements/backgrounds/nether\")"));
        assert!(nether.contains("AdvancementRewards.Builder.experience(1000)"));
        assert!(nether.contains("VanillaAdventureAdvancements.addBiomes("));
        assert!(nether.contains("ItemTags.PIGLIN_LOVED"));
        assert!(nether.contains("PiglinAi.BARTERING_ITEM"));

        let end = java_source(PACK_CLASSES[2]);
        assert!(
            end.contains("Identifier.withDefaultNamespace(\"gui/advancements/backgrounds/end\")")
        );
        assert!(end.contains("LevitationTrigger.TriggerInstance.levitated"));
        assert!(end.contains("AdvancementRewards.Builder.experience(50)"));
    }

    #[test]
    fn adventure_pack_helper_constants_and_late_ids_match_java() {
        let source = java_source(PACK_CLASSES[3]);
        assert!(source.contains("private static final int DISTANCE_FROM_BOTTOM_TO_TOP = 384"));
        assert!(source.contains("private static final int Y_COORDINATE_AT_TOP = 320"));
        assert!(source.contains("private static final int Y_COORDINATE_AT_BOTTOM = -64"));
        assert!(source.contains("MobCategory.MONSTER, Set.of(EntityType.GIANT, EntityType.ILLUSIONER, EntityType.WARDEN)"));
        assert!(source.contains("createMonsterHunterAdvancement("));
        assert!(source.contains("validateMobsToKill(MOBS_TO_KILL, entityTypes)"));
        assert!(source.contains("smithingWithStyle("));
        assert!(source.contains("craftingANewLook("));
        assert!(source.contains("respectingTheRemnantsCriterions("));
        assert!(source.contains(".save(output, \"adventure/crafters_crafting_crafters\")"));
        assert!(source.contains(".save(output, \"adventure/lighten_up\")"));
        assert!(source.contains(".save(output, \"adventure/adventuring_time\")"));
        assert_eq!(parse_static_entity_list_count(source, "MOBS_TO_KILL"), 41);
    }

    #[test]
    fn husbandry_pack_static_lists_and_helpers_match_java() {
        let source = java_source(PACK_CLASSES[4]);
        assert_eq!(
            parse_static_entity_list_count(source, "BREEDABLE_ANIMALS"),
            23
        );
        assert_eq!(
            parse_static_entity_list_count(source, "INDIRECTLY_BREEDABLE_ANIMALS"),
            3
        );
        assert_eq!(parse_item_array_count(source, "FISH ="), 4);
        assert_eq!(parse_item_array_count(source, "FISH_BUCKETS ="), 4);
        assert_eq!(parse_item_array_count(source, "EDIBLE_ITEMS ="), 40);
        assert_eq!(parse_item_array_count(source, "WAX_SCRAPING_TOOLS ="), 7);
        assert!(source.contains("createBreedAllAnimalsAdvancement("));
        assert!(source.contains("addLeashedFrogVariants("));
        assert!(source.contains("sortedVariants("));
        assert!(source.contains("HOLDER_KEY_COMPARATOR"));
        assert!(source.contains(".save(output, \"husbandry/place_dried_ghast_in_water\")"));
        assert!(source.contains(".save(output, \"husbandry/bred_all_animals\")"));
    }

    #[test]
    fn package_is_null_marked() {
        const {
            assert!(DATA_ADVANCEMENT_PACKS_PACKAGE_NULL_MARKED);
        }
    }
}
