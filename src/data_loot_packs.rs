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

#[derive(Debug, Clone, PartialEq, Eq)]
struct ArchaeologyTableSummary {
    id: &'static str,
    entry_count: usize,
    weighted_entries: usize,
    sentinel_entries: Vec<&'static str>,
}

fn vanilla_archaeology_tables() -> Vec<ArchaeologyTableSummary> {
    vec![
        ArchaeologyTableSummary {
            id: "minecraft:archaeology/desert_well",
            entry_count: 6,
            weighted_entries: 2,
            sentinel_entries: vec![
                "arms_up_pottery_sherd@2",
                "brewer_pottery_sherd@2",
                "suspicious_stew=night_vision,jump_boost,weakness,blindness,poison,saturation",
            ],
        },
        ArchaeologyTableSummary {
            id: "minecraft:archaeology/desert_pyramid",
            entry_count: 8,
            weighted_entries: 0,
            sentinel_entries: vec![
                "archer_pottery_sherd",
                "miner_pottery_sherd",
                "prize_pottery_sherd",
                "skull_pottery_sherd",
            ],
        },
        ArchaeologyTableSummary {
            id: "minecraft:archaeology/trail_ruins_common",
            entry_count: 31,
            weighted_entries: 14,
            sentinel_entries: vec![
                "emerald@2",
                "purple_candle@2",
                "spruce_hanging_sign",
                "lead",
            ],
        },
        ArchaeologyTableSummary {
            id: "minecraft:archaeology/trail_ruins_rare",
            entry_count: 12,
            weighted_entries: 0,
            sentinel_entries: vec![
                "burn_pottery_sherd",
                "wayfinder_armor_trim_smithing_template",
                "host_armor_trim_smithing_template",
                "music_disc_relic",
            ],
        },
        ArchaeologyTableSummary {
            id: "minecraft:archaeology/ocean_ruin_warm",
            entry_count: 10,
            weighted_entries: 5,
            sentinel_entries: vec![
                "angler_pottery_sherd",
                "sniffer_egg",
                "iron_axe",
                "gold_nugget@2",
            ],
        },
        ArchaeologyTableSummary {
            id: "minecraft:archaeology/ocean_ruin_cold",
            entry_count: 10,
            weighted_entries: 5,
            sentinel_entries: vec![
                "blade_pottery_sherd",
                "explorer_pottery_sherd",
                "plenty_pottery_sherd",
                "gold_nugget@2",
            ],
        },
    ]
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

    #[test]
    fn vanilla_archaeology_loot_emits_all_java_tables_with_entry_counts() {
        let tables = vanilla_archaeology_tables();
        assert_eq!(
            tables.iter().map(|table| table.id).collect::<Vec<_>>(),
            vec![
                "minecraft:archaeology/desert_well",
                "minecraft:archaeology/desert_pyramid",
                "minecraft:archaeology/trail_ruins_common",
                "minecraft:archaeology/trail_ruins_rare",
                "minecraft:archaeology/ocean_ruin_warm",
                "minecraft:archaeology/ocean_ruin_cold",
            ]
        );
        assert_eq!(
            tables
                .iter()
                .map(|table| (table.entry_count, table.weighted_entries))
                .collect::<Vec<_>>(),
            vec![(6, 2), (8, 0), (31, 14), (12, 0), (10, 5), (10, 5)]
        );
        assert!(tables[0].sentinel_entries.iter().any(|entry| entry.contains("suspicious_stew")));
        assert!(tables[3].sentinel_entries.contains(&"music_disc_relic"));
        assert!(tables[4].sentinel_entries.contains(&"sniffer_egg"));
    }

    #[test]
    fn vanilla_archaeology_java_source_sentinels_match_authoritative_file() {
        let source = include_str!(
            "../../decompiled-server-26.1.2/net/minecraft/data/loot/packs/VanillaArchaeologyLoot.java"
        );
        for expected in [
            "BuiltInLootTables.DESERT_WELL_ARCHAEOLOGY",
            "BuiltInLootTables.DESERT_PYRAMID_ARCHAEOLOGY",
            "BuiltInLootTables.TRAIL_RUINS_ARCHAEOLOGY_COMMON",
            "BuiltInLootTables.TRAIL_RUINS_ARCHAEOLOGY_RARE",
            "BuiltInLootTables.OCEAN_RUIN_WARM_ARCHAEOLOGY",
            "BuiltInLootTables.OCEAN_RUIN_COLD_ARCHAEOLOGY",
            "withEffect(MobEffects.NIGHT_VISION, UniformGenerator.between(7.0F, 10.0F))",
            "LootItem.lootTableItem(Items.SNIFFER_EGG)",
            "LootItem.lootTableItem(Items.MUSIC_DISC_RELIC)",
        ] {
            assert!(source.contains(expected), "missing Java sentinel: {expected}");
        }
    }
}
