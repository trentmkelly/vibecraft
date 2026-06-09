const JAVA_SOURCE: &str = include_str!(
    "../../decompiled-server-26.1.2/net/minecraft/data/loot/packs/VanillaGiftLoot.java"
);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct GiftTableSummary {
    id: &'static str,
    entry_count: usize,
}

fn vanilla_gift_tables() -> &'static [GiftTableSummary] {
    &[
        GiftTableSummary {
            id: "minecraft:gameplay/cat_morning_gift",
            entry_count: 7,
        },
        GiftTableSummary {
            id: "minecraft:gameplay/hero_of_the_village/armorer_gift",
            entry_count: 4,
        },
        GiftTableSummary {
            id: "minecraft:gameplay/hero_of_the_village/butcher_gift",
            entry_count: 5,
        },
        GiftTableSummary {
            id: "minecraft:gameplay/hero_of_the_village/cartographer_gift",
            entry_count: 2,
        },
        GiftTableSummary {
            id: "minecraft:gameplay/hero_of_the_village/cleric_gift",
            entry_count: 2,
        },
        GiftTableSummary {
            id: "minecraft:gameplay/hero_of_the_village/farmer_gift",
            entry_count: 3,
        },
        GiftTableSummary {
            id: "minecraft:gameplay/hero_of_the_village/fisherman_gift",
            entry_count: 2,
        },
        GiftTableSummary {
            id: "minecraft:gameplay/hero_of_the_village/fletcher_gift",
            entry_count: 14,
        },
        GiftTableSummary {
            id: "minecraft:gameplay/hero_of_the_village/leatherworker_gift",
            entry_count: 1,
        },
        GiftTableSummary {
            id: "minecraft:gameplay/hero_of_the_village/librarian_gift",
            entry_count: 1,
        },
        GiftTableSummary {
            id: "minecraft:gameplay/hero_of_the_village/mason_gift",
            entry_count: 1,
        },
        GiftTableSummary {
            id: "minecraft:gameplay/hero_of_the_village/shepherd_gift",
            entry_count: 16,
        },
        GiftTableSummary {
            id: "minecraft:gameplay/hero_of_the_village/toolsmith_gift",
            entry_count: 4,
        },
        GiftTableSummary {
            id: "minecraft:gameplay/hero_of_the_village/weaponsmith_gift",
            entry_count: 3,
        },
        GiftTableSummary {
            id: "minecraft:gameplay/hero_of_the_village/unemployed_gift",
            entry_count: 1,
        },
        GiftTableSummary {
            id: "minecraft:gameplay/hero_of_the_village/baby_gift",
            entry_count: 1,
        },
        GiftTableSummary {
            id: "minecraft:gameplay/sniffer_digging",
            entry_count: 2,
        },
        GiftTableSummary {
            id: "minecraft:gameplay/panda_sneeze",
            entry_count: 2,
        },
        GiftTableSummary {
            id: "minecraft:gameplay/chicken_lay",
            entry_count: 3,
        },
        GiftTableSummary {
            id: "minecraft:gameplay/armadillo_shed",
            entry_count: 1,
        },
        GiftTableSummary {
            id: "minecraft:gameplay/turtle_grow",
            entry_count: 1,
        },
    ]
}

fn cat_morning_gift_entries() -> &'static [(&'static str, i32)] {
    &[
        ("rabbit_hide", 10),
        ("rabbit_foot", 10),
        ("chicken", 10),
        ("feather", 10),
        ("rotten_flesh", 10),
        ("string", 10),
        ("phantom_membrane", 2),
    ]
}

fn fletcher_potions() -> &'static [&'static str] {
    &[
        "swiftness",
        "slowness",
        "strength",
        "healing",
        "harming",
        "leaping",
        "regeneration",
        "fire_resistance",
        "water_breathing",
        "invisibility",
        "night_vision",
        "weakness",
        "poison",
    ]
}

fn shepherd_wool_order() -> &'static [&'static str] {
    &[
        "white_wool",
        "orange_wool",
        "magenta_wool",
        "light_blue_wool",
        "yellow_wool",
        "lime_wool",
        "pink_wool",
        "gray_wool",
        "light_gray_wool",
        "cyan_wool",
        "purple_wool",
        "blue_wool",
        "brown_wool",
        "green_wool",
        "red_wool",
        "black_wool",
    ]
}

fn chicken_lay_entries() -> &'static [(&'static str, &'static str)] {
    &[
        ("egg", "temperate"),
        ("brown_egg", "warm"),
        ("blue_egg", "cold"),
    ]
}

fn count_occurrences(source: &str, needle: &str) -> usize {
    source.match_indices(needle).count()
}

fn assert_source_contains_all(expected: &[&str]) {
    for sentinel in expected {
        assert!(
            JAVA_SOURCE.contains(sentinel),
            "missing Java sentinel: {sentinel}"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vanilla_gift_loot_emits_all_java_tables_in_order() {
        let tables = vanilla_gift_tables();
        assert_eq!(tables.len(), 21);
        assert_eq!(
            tables.iter().map(|table| table.id).collect::<Vec<_>>(),
            vec![
                "minecraft:gameplay/cat_morning_gift",
                "minecraft:gameplay/hero_of_the_village/armorer_gift",
                "minecraft:gameplay/hero_of_the_village/butcher_gift",
                "minecraft:gameplay/hero_of_the_village/cartographer_gift",
                "minecraft:gameplay/hero_of_the_village/cleric_gift",
                "minecraft:gameplay/hero_of_the_village/farmer_gift",
                "minecraft:gameplay/hero_of_the_village/fisherman_gift",
                "minecraft:gameplay/hero_of_the_village/fletcher_gift",
                "minecraft:gameplay/hero_of_the_village/leatherworker_gift",
                "minecraft:gameplay/hero_of_the_village/librarian_gift",
                "minecraft:gameplay/hero_of_the_village/mason_gift",
                "minecraft:gameplay/hero_of_the_village/shepherd_gift",
                "minecraft:gameplay/hero_of_the_village/toolsmith_gift",
                "minecraft:gameplay/hero_of_the_village/weaponsmith_gift",
                "minecraft:gameplay/hero_of_the_village/unemployed_gift",
                "minecraft:gameplay/hero_of_the_village/baby_gift",
                "minecraft:gameplay/sniffer_digging",
                "minecraft:gameplay/panda_sneeze",
                "minecraft:gameplay/chicken_lay",
                "minecraft:gameplay/armadillo_shed",
                "minecraft:gameplay/turtle_grow",
            ]
        );
        assert_eq!(
            tables
                .iter()
                .map(|table| table.entry_count)
                .collect::<Vec<_>>(),
            vec![7, 4, 5, 2, 2, 3, 2, 14, 1, 1, 1, 16, 4, 3, 1, 1, 2, 2, 3, 1, 1]
        );
    }

    #[test]
    fn cat_morning_gift_matches_java_weighted_items() {
        assert_eq!(
            cat_morning_gift_entries(),
            &[
                ("rabbit_hide", 10),
                ("rabbit_foot", 10),
                ("chicken", 10),
                ("feather", 10),
                ("rotten_flesh", 10),
                ("string", 10),
                ("phantom_membrane", 2),
            ]
        );
    }

    #[test]
    fn fletcher_gift_matches_java_arrow_and_tipped_potion_set() {
        assert_eq!(
            fletcher_potions(),
            &[
                "swiftness",
                "slowness",
                "strength",
                "healing",
                "harming",
                "leaping",
                "regeneration",
                "fire_resistance",
                "water_breathing",
                "invisibility",
                "night_vision",
                "weakness",
                "poison",
            ]
        );
        assert_source_contains_all(&[
            "LootItem.lootTableItem(Items.ARROW).setWeight(26)",
            "SetItemCountFunction.setCount(UniformGenerator.between(0.0F, 1.0F))",
            "SetPotionFunction.setPotion(Potions.SWIFTNESS)",
            "SetPotionFunction.setPotion(Potions.POISON)",
        ]);
    }

    #[test]
    fn shepherd_gift_matches_java_wool_order() {
        assert_eq!(
            shepherd_wool_order(),
            &[
                "white_wool",
                "orange_wool",
                "magenta_wool",
                "light_blue_wool",
                "yellow_wool",
                "lime_wool",
                "pink_wool",
                "gray_wool",
                "light_gray_wool",
                "cyan_wool",
                "purple_wool",
                "blue_wool",
                "brown_wool",
                "green_wool",
                "red_wool",
                "black_wool",
            ]
        );
    }

    #[test]
    fn special_gameplay_gifts_match_java_weights_and_conditions() {
        assert_eq!(
            chicken_lay_entries(),
            &[
                ("egg", "temperate"),
                ("brown_egg", "warm"),
                ("blue_egg", "cold")
            ]
        );
        assert_source_contains_all(&[
            "BuiltInLootTables.SNIFFER_DIGGING",
            "LootItem.lootTableItem(Items.TORCHFLOWER_SEEDS)",
            "LootItem.lootTableItem(Items.PITCHER_POD)",
            "BuiltInLootTables.PANDA_SNEEZE",
            "LootItem.lootTableItem(Items.SLIME_BALL).setWeight(1)",
            "EmptyLootItem.emptyItem().setWeight(699)",
            "AlternativesEntry.alternatives(",
            "DataComponents.CHICKEN_VARIANT, chickenVariants.getOrThrow(ChickenVariants.TEMPERATE)",
            "DataComponents.CHICKEN_VARIANT, chickenVariants.getOrThrow(ChickenVariants.WARM)",
            "DataComponents.CHICKEN_VARIANT, chickenVariants.getOrThrow(ChickenVariants.COLD)",
            "BuiltInLootTables.ARMADILLO_SHED",
            "LootItem.lootTableItem(Items.ARMADILLO_SCUTE)",
            "BuiltInLootTables.TURTLE_GROW",
            "LootItem.lootTableItem(Items.TURTLE_SCUTE)",
        ]);
    }

    #[test]
    fn vanilla_gift_java_source_counts_match_authoritative_file() {
        assert_eq!(count_occurrences(JAVA_SOURCE, "output.accept"), 21);
        assert_eq!(count_occurrences(JAVA_SOURCE, "LootPool.lootPool("), 21);
        assert_eq!(count_occurrences(JAVA_SOURCE, "LootItem.lootTableItem"), 75);
        assert_eq!(
            count_occurrences(JAVA_SOURCE, "SetItemCountFunction.setCount"),
            13
        );
        assert_eq!(
            count_occurrences(JAVA_SOURCE, "SetPotionFunction.setPotion"),
            13
        );
        assert_eq!(count_occurrences(JAVA_SOURCE, "EmptyLootItem.emptyItem"), 1);
        assert_eq!(
            count_occurrences(JAVA_SOURCE, "AlternativesEntry.alternatives"),
            1
        );
        assert_eq!(
            count_occurrences(JAVA_SOURCE, "DataComponentExactPredicate.expect"),
            3
        );
        assert_eq!(
            count_occurrences(JAVA_SOURCE, "LootItemEntityPropertyCondition.hasProperties"),
            3
        );
        assert_eq!(count_occurrences(JAVA_SOURCE, "setWeight"), 10);
        assert_eq!(count_occurrences(JAVA_SOURCE, "setRolls"), 21);
    }

    #[test]
    fn vanilla_gift_java_source_sentinels_match_authoritative_file() {
        assert_source_contains_all(&[
            "public record VanillaGiftLoot(HolderLookup.Provider registries) implements LootTableSubProvider",
            "HolderGetter<ChickenVariant> chickenVariants = this.registries.lookupOrThrow(Registries.CHICKEN_VARIANT)",
            "BuiltInLootTables.CAT_MORNING_GIFT",
            "BuiltInLootTables.ARMORER_GIFT",
            "BuiltInLootTables.BUTCHER_GIFT",
            "BuiltInLootTables.CARTOGRAPHER_GIFT",
            "BuiltInLootTables.CLERIC_GIFT",
            "BuiltInLootTables.FARMER_GIFT",
            "BuiltInLootTables.FISHERMAN_GIFT",
            "BuiltInLootTables.FLETCHER_GIFT",
            "BuiltInLootTables.LEATHERWORKER_GIFT",
            "BuiltInLootTables.LIBRARIAN_GIFT",
            "BuiltInLootTables.MASON_GIFT",
            "BuiltInLootTables.SHEPHERD_GIFT",
            "BuiltInLootTables.TOOLSMITH_GIFT",
            "BuiltInLootTables.WEAPONSMITH_GIFT",
            "BuiltInLootTables.UNEMPLOYED_GIFT",
            "BuiltInLootTables.BABY_VILLAGER_GIFT",
            "BuiltInLootTables.SNIFFER_DIGGING",
            "BuiltInLootTables.PANDA_SNEEZE",
            "BuiltInLootTables.CHICKEN_LAY",
            "BuiltInLootTables.ARMADILLO_SHED",
            "BuiltInLootTables.TURTLE_GROW",
            "LootItem.lootTableItem(Items.CHAINMAIL_HELMET)",
            "LootItem.lootTableItem(Items.COOKED_MUTTON)",
            "LootItem.lootTableItem(Items.MAP)).add(LootItem.lootTableItem(Items.PAPER))",
            "LootItem.lootTableItem(Items.REDSTONE)",
            "LootItem.lootTableItem(Items.COOKIE)",
            "LootItem.lootTableItem(Items.COD)).add(LootItem.lootTableItem(Items.SALMON))",
            "LootItem.lootTableItem(Items.LEATHER)",
            "LootItem.lootTableItem(Items.BOOK)",
            "LootItem.lootTableItem(Items.CLAY)",
            "LootItem.lootTableItem(Items.BLACK_WOOL)",
            "LootItem.lootTableItem(Items.STONE_SHOVEL)",
            "LootItem.lootTableItem(Items.IRON_AXE)",
            "LootItem.lootTableItem(Items.WHEAT_SEEDS)",
            "LootItem.lootTableItem(Items.POPPY)",
        ]);
    }
}
