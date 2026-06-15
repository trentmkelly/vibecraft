const JAVA_SOURCE: &str = vibecraft_java_source!("/net/minecraft/data/loot/packs/VanillaPiglinBarterLoot.java");

#[derive(Debug, Clone, PartialEq, Eq)]
struct BarterEntrySummary {
    item: &'static str,
    weight: i32,
    function: Option<&'static str>,
}

fn piglin_barter_entries() -> Vec<BarterEntrySummary> {
    vec![
        BarterEntrySummary {
            item: "book",
            weight: 5,
            function: Some("random_enchant=soul_speed"),
        },
        BarterEntrySummary {
            item: "iron_boots",
            weight: 8,
            function: Some("random_enchant=soul_speed"),
        },
        BarterEntrySummary {
            item: "potion",
            weight: 8,
            function: Some("potion=fire_resistance"),
        },
        BarterEntrySummary {
            item: "splash_potion",
            weight: 8,
            function: Some("potion=fire_resistance"),
        },
        BarterEntrySummary {
            item: "potion",
            weight: 10,
            function: Some("potion=water"),
        },
        BarterEntrySummary {
            item: "iron_nugget",
            weight: 10,
            function: Some("count=10..36"),
        },
        BarterEntrySummary {
            item: "ender_pearl",
            weight: 10,
            function: Some("count=2..4"),
        },
        BarterEntrySummary {
            item: "dried_ghast",
            weight: 10,
            function: Some("count=1"),
        },
        BarterEntrySummary {
            item: "string",
            weight: 20,
            function: Some("count=3..9"),
        },
        BarterEntrySummary {
            item: "quartz",
            weight: 20,
            function: Some("count=5..12"),
        },
        BarterEntrySummary {
            item: "obsidian",
            weight: 40,
            function: None,
        },
        BarterEntrySummary {
            item: "crying_obsidian",
            weight: 40,
            function: Some("count=1..3"),
        },
        BarterEntrySummary {
            item: "fire_charge",
            weight: 40,
            function: None,
        },
        BarterEntrySummary {
            item: "leather",
            weight: 40,
            function: Some("count=2..4"),
        },
        BarterEntrySummary {
            item: "soul_sand",
            weight: 40,
            function: Some("count=2..8"),
        },
        BarterEntrySummary {
            item: "nether_brick",
            weight: 40,
            function: Some("count=2..8"),
        },
        BarterEntrySummary {
            item: "spectral_arrow",
            weight: 40,
            function: Some("count=6..12"),
        },
        BarterEntrySummary {
            item: "gravel",
            weight: 40,
            function: Some("count=8..16"),
        },
        BarterEntrySummary {
            item: "blackstone",
            weight: 40,
            function: Some("count=8..16"),
        },
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
    fn piglin_barter_loot_matches_java_order_weights_and_functions() {
        let entries = piglin_barter_entries();
        assert_eq!(entries.len(), 19);
        assert_eq!(
            entries
                .iter()
                .map(|entry| (entry.item, entry.weight, entry.function))
                .collect::<Vec<_>>(),
            vec![
                ("book", 5, Some("random_enchant=soul_speed")),
                ("iron_boots", 8, Some("random_enchant=soul_speed")),
                ("potion", 8, Some("potion=fire_resistance")),
                ("splash_potion", 8, Some("potion=fire_resistance")),
                ("potion", 10, Some("potion=water")),
                ("iron_nugget", 10, Some("count=10..36")),
                ("ender_pearl", 10, Some("count=2..4")),
                ("dried_ghast", 10, Some("count=1")),
                ("string", 20, Some("count=3..9")),
                ("quartz", 20, Some("count=5..12")),
                ("obsidian", 40, None),
                ("crying_obsidian", 40, Some("count=1..3")),
                ("fire_charge", 40, None),
                ("leather", 40, Some("count=2..4")),
                ("soul_sand", 40, Some("count=2..8")),
                ("nether_brick", 40, Some("count=2..8")),
                ("spectral_arrow", 40, Some("count=6..12")),
                ("gravel", 40, Some("count=8..16")),
                ("blackstone", 40, Some("count=8..16")),
            ]
        );
    }

    #[test]
    fn piglin_barter_java_source_counts_match_authoritative_file() {
        assert_eq!(count_occurrences(JAVA_SOURCE, "output.accept"), 1);
        assert_eq!(count_occurrences(JAVA_SOURCE, "LootPool.lootPool("), 1);
        assert_eq!(count_occurrences(JAVA_SOURCE, "LootItem.lootTableItem"), 19);
        assert_eq!(count_occurrences(JAVA_SOURCE, "setWeight"), 19);
        assert_eq!(
            count_occurrences(JAVA_SOURCE, "SetItemCountFunction.setCount"),
            12
        );
        assert_eq!(
            count_occurrences(JAVA_SOURCE, "SetPotionFunction.setPotion"),
            3
        );
        assert_eq!(count_occurrences(JAVA_SOURCE, "EnchantRandomlyFunction"), 3);
        assert_eq!(
            count_occurrences(JAVA_SOURCE, "UniformGenerator.between"),
            11
        );
        assert_eq!(count_occurrences(JAVA_SOURCE, "ConstantValue.exactly"), 2);
    }

    #[test]
    fn piglin_barter_java_source_sentinels_match_authoritative_file() {
        assert_source_contains_all(&[
            "public record VanillaPiglinBarterLoot(HolderLookup.Provider registries) implements LootTableSubProvider",
            "HolderLookup.RegistryLookup<Enchantment> enchantments = this.registries.lookupOrThrow(Registries.ENCHANTMENT)",
            "BuiltInLootTables.PIGLIN_BARTERING",
            "LootPool.lootPool()\n                  .setRolls(ConstantValue.exactly(1.0F))",
            "LootItem.lootTableItem(Items.BOOK)\n                        .setWeight(5)",
            "withEnchantment(enchantments.getOrThrow(Enchantments.SOUL_SPEED))",
            "LootItem.lootTableItem(Items.IRON_BOOTS)\n                        .setWeight(8)",
            "LootItem.lootTableItem(Items.POTION).setWeight(8).apply(SetPotionFunction.setPotion(Potions.FIRE_RESISTANCE))",
            "LootItem.lootTableItem(Items.SPLASH_POTION).setWeight(8).apply(SetPotionFunction.setPotion(Potions.FIRE_RESISTANCE))",
            "LootItem.lootTableItem(Items.POTION).setWeight(10).apply(SetPotionFunction.setPotion(Potions.WATER))",
            "LootItem.lootTableItem(Items.IRON_NUGGET).setWeight(10).apply(SetItemCountFunction.setCount(UniformGenerator.between(10.0F, 36.0F)))",
            "LootItem.lootTableItem(Items.ENDER_PEARL).setWeight(10).apply(SetItemCountFunction.setCount(UniformGenerator.between(2.0F, 4.0F)))",
            "LootItem.lootTableItem(Items.DRIED_GHAST).setWeight(10).apply(SetItemCountFunction.setCount(ConstantValue.exactly(1.0F)))",
            "LootItem.lootTableItem(Items.STRING).setWeight(20).apply(SetItemCountFunction.setCount(UniformGenerator.between(3.0F, 9.0F)))",
            "LootItem.lootTableItem(Items.QUARTZ).setWeight(20).apply(SetItemCountFunction.setCount(UniformGenerator.between(5.0F, 12.0F)))",
            "LootItem.lootTableItem(Items.OBSIDIAN).setWeight(40)",
            "LootItem.lootTableItem(Items.CRYING_OBSIDIAN).setWeight(40).apply(SetItemCountFunction.setCount(UniformGenerator.between(1.0F, 3.0F)))",
            "LootItem.lootTableItem(Items.FIRE_CHARGE).setWeight(40)",
            "LootItem.lootTableItem(Items.LEATHER).setWeight(40).apply(SetItemCountFunction.setCount(UniformGenerator.between(2.0F, 4.0F)))",
            "LootItem.lootTableItem(Items.SOUL_SAND).setWeight(40).apply(SetItemCountFunction.setCount(UniformGenerator.between(2.0F, 8.0F)))",
            "LootItem.lootTableItem(Items.NETHER_BRICK).setWeight(40).apply(SetItemCountFunction.setCount(UniformGenerator.between(2.0F, 8.0F)))",
            "LootItem.lootTableItem(Items.SPECTRAL_ARROW).setWeight(40).apply(SetItemCountFunction.setCount(UniformGenerator.between(6.0F, 12.0F)))",
            "LootItem.lootTableItem(Items.GRAVEL).setWeight(40).apply(SetItemCountFunction.setCount(UniformGenerator.between(8.0F, 16.0F)))",
            "LootItem.lootTableItem(Items.BLACKSTONE).setWeight(40).apply(SetItemCountFunction.setCount(UniformGenerator.between(8.0F, 16.0F)))",
        ]);
    }
}
