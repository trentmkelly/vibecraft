#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum DyeColorModel {
    White,
    Orange,
    Magenta,
    LightBlue,
    Yellow,
    Lime,
    Pink,
    Gray,
    LightGray,
    Cyan,
    Purple,
    Blue,
    Brown,
    Green,
    Red,
    Black,
}

fn wool_item_by_dye() -> Vec<(DyeColorModel, &'static str)> {
    vec![
        (DyeColorModel::White, "minecraft:white_wool"),
        (DyeColorModel::Orange, "minecraft:orange_wool"),
        (DyeColorModel::Magenta, "minecraft:magenta_wool"),
        (DyeColorModel::LightBlue, "minecraft:light_blue_wool"),
        (DyeColorModel::Yellow, "minecraft:yellow_wool"),
        (DyeColorModel::Lime, "minecraft:lime_wool"),
        (DyeColorModel::Pink, "minecraft:pink_wool"),
        (DyeColorModel::Gray, "minecraft:gray_wool"),
        (DyeColorModel::LightGray, "minecraft:light_gray_wool"),
        (DyeColorModel::Cyan, "minecraft:cyan_wool"),
        (DyeColorModel::Purple, "minecraft:purple_wool"),
        (DyeColorModel::Blue, "minecraft:blue_wool"),
        (DyeColorModel::Brown, "minecraft:brown_wool"),
        (DyeColorModel::Green, "minecraft:green_wool"),
        (DyeColorModel::Red, "minecraft:red_wool"),
        (DyeColorModel::Black, "minecraft:black_wool"),
    ]
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct LootTableSummary {
    id: &'static str,
    pool_count: usize,
    roll_shapes: Vec<&'static str>,
    sentinel_entries: Vec<&'static str>,
}

fn trade_rebalance_chest_tables() -> Vec<LootTableSummary> {
    vec![
        LootTableSummary {
            id: "minecraft:chests/abandoned_mineshaft",
            pool_count: 4,
            roll_shapes: vec!["constant:1", "uniform:2..4", "constant:3", "constant:1"],
            sentinel_entries: vec![
                "golden_apple@20",
                "book=random_applicable@10",
                "rail:4..8@20",
                "book=efficiency@1",
            ],
        },
        LootTableSummary {
            id: "minecraft:chests/ancient_city",
            pool_count: 2,
            roll_shapes: vec!["uniform:5..10", "constant:1"],
            sentinel_entries: vec![
                "diamond_hoe:damage=0.8..1.0:levels=30..50@2",
                "book=swift_sneak@3",
                "potion=strong_regeneration@5",
                "book=mending@4",
            ],
        },
        LootTableSummary {
            id: "minecraft:chests/desert_pyramid",
            pool_count: 3,
            roll_shapes: vec!["uniform:2..4", "constant:4", "constant:1"],
            sentinel_entries: vec![
                "diamond:1..3@5",
                "enchanted_golden_apple@2",
                "dune_template:count=2@1",
                "book=unbreaking@2",
            ],
        },
        LootTableSummary {
            id: "minecraft:chests/jungle_temple",
            pool_count: 3,
            roll_shapes: vec!["uniform:2..6", "constant:1", "constant:1"],
            sentinel_entries: vec![
                "bamboo:1..3@15",
                "book:levels=30@1",
                "wild_template:count=2@1",
                "book=unbreaking@1",
            ],
        },
        LootTableSummary {
            id: "minecraft:chests/pillager_outpost",
            pool_count: 7,
            roll_shapes: vec![
                "uniform:0..1",
                "uniform:2..3",
                "uniform:1..3",
                "uniform:2..3",
                "uniform:0..1",
                "constant:1",
                "constant:1",
            ],
            sentinel_entries: vec![
                "crossbow@1",
                "goat_horn=regular_goat_horns@1",
                "sentry_template:count=2@1",
                "book=quick_charge@2",
            ],
        },
    ]
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct LootProviderSummary {
    required_tables: usize,
    subproviders: Vec<(&'static str, &'static str)>,
}

fn trade_rebalance_loot_table_provider() -> LootProviderSummary {
    LootProviderSummary {
        required_tables: 0,
        subproviders: vec![("TradeRebalanceChestLoot", "chest")],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loot_data_wool_item_by_dye_matches_java_enum_map() {
        let mapping = wool_item_by_dye();
        assert_eq!(mapping.len(), 16);
        assert_eq!(
            mapping,
            vec![
                (DyeColorModel::White, "minecraft:white_wool"),
                (DyeColorModel::Orange, "minecraft:orange_wool"),
                (DyeColorModel::Magenta, "minecraft:magenta_wool"),
                (DyeColorModel::LightBlue, "minecraft:light_blue_wool"),
                (DyeColorModel::Yellow, "minecraft:yellow_wool"),
                (DyeColorModel::Lime, "minecraft:lime_wool"),
                (DyeColorModel::Pink, "minecraft:pink_wool"),
                (DyeColorModel::Gray, "minecraft:gray_wool"),
                (DyeColorModel::LightGray, "minecraft:light_gray_wool"),
                (DyeColorModel::Cyan, "minecraft:cyan_wool"),
                (DyeColorModel::Purple, "minecraft:purple_wool"),
                (DyeColorModel::Blue, "minecraft:blue_wool"),
                (DyeColorModel::Brown, "minecraft:brown_wool"),
                (DyeColorModel::Green, "minecraft:green_wool"),
                (DyeColorModel::Red, "minecraft:red_wool"),
                (DyeColorModel::Black, "minecraft:black_wool"),
            ]
        );
    }

    #[test]
    fn trade_rebalance_chest_loot_emits_the_java_table_set_and_pool_shapes() {
        let tables = trade_rebalance_chest_tables();
        assert_eq!(
            tables.iter().map(|table| table.id).collect::<Vec<_>>(),
            vec![
                "minecraft:chests/abandoned_mineshaft",
                "minecraft:chests/ancient_city",
                "minecraft:chests/desert_pyramid",
                "minecraft:chests/jungle_temple",
                "minecraft:chests/pillager_outpost",
            ]
        );
        assert_eq!(
            tables
                .iter()
                .map(|table| (table.id, table.pool_count))
                .collect::<Vec<_>>(),
            vec![
                ("minecraft:chests/abandoned_mineshaft", 4),
                ("minecraft:chests/ancient_city", 2),
                ("minecraft:chests/desert_pyramid", 3),
                ("minecraft:chests/jungle_temple", 3),
                ("minecraft:chests/pillager_outpost", 7),
            ]
        );
        assert!(tables[0].sentinel_entries.contains(&"book=efficiency@1"));
        assert!(tables[1]
            .sentinel_entries
            .contains(&"potion=strong_regeneration@5"));
        assert!(tables[4]
            .sentinel_entries
            .contains(&"goat_horn=regular_goat_horns@1"));
    }

    #[test]
    fn trade_rebalance_provider_registers_only_chest_subprovider() {
        let provider = trade_rebalance_loot_table_provider();
        assert_eq!(provider.required_tables, 0);
        assert_eq!(provider.subproviders, vec![("TradeRebalanceChestLoot", "chest")]);
    }

    #[test]
    fn trade_rebalance_chest_java_source_sentinels_match_authoritative_file() {
        let source = include_str!(
            "../../decompiled-server-26.1.2/net/minecraft/data/loot/packs/TradeRebalanceChestLoot.java"
        );
        for expected in [
            "output.accept(\n         BuiltInLootTables.ABANDONED_MINESHAFT",
            "output.accept(BuiltInLootTables.ANCIENT_CITY, this.ancientCityLootTable());",
            "output.accept(BuiltInLootTables.DESERT_PYRAMID, this.desertPyramidLootTable());",
            "output.accept(BuiltInLootTables.JUNGLE_TEMPLE, this.jungleTempleLootTable());",
            "output.accept(BuiltInLootTables.PILLAGER_OUTPOST, this.pillagerOutpostLootTable());",
            "withEnchantment(enchantments.getOrThrow(Enchantments.EFFICIENCY))",
            "SetPotionFunction.setPotion(Potions.STRONG_REGENERATION)",
            "SetInstrumentFunction.setInstrumentOptions(instruments.getOrThrow(InstrumentTags.REGULAR_GOAT_HORNS))",
            "withEnchantment(enchantments.getOrThrow(Enchantments.QUICK_CHARGE))",
        ] {
            assert!(source.contains(expected), "missing Java sentinel: {expected}");
        }
    }

    #[test]
    fn trade_rebalance_provider_java_source_matches_wrapper_shape() {
        let source = include_str!(
            "../../decompiled-server-26.1.2/net/minecraft/data/loot/packs/TradeRebalanceLootTableProvider.java"
        );
        assert!(source.contains("new LootTableProvider("));
        assert!(source.contains("output, Set.of(), List.of("));
        assert!(source.contains(
            "new LootTableProvider.SubProviderEntry(TradeRebalanceChestLoot::new, LootContextParamSets.CHEST)"
        ));
    }
}
