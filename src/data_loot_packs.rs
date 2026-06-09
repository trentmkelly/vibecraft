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

#[derive(Debug, Clone, PartialEq, Eq)]
struct BlockInteractTableSummary {
    id: &'static str,
    pool_count: usize,
    sentinel_entries: Vec<&'static str>,
}

fn vanilla_block_interact_tables() -> Vec<BlockInteractTableSummary> {
    vec![
        BlockInteractTableSummary {
            id: "minecraft:gameplay/harvest_beehive",
            pool_count: 1,
            sentinel_entries: vec!["honeycomb:count=3"],
        },
        BlockInteractTableSummary {
            id: "minecraft:gameplay/harvest_cave_vine",
            pool_count: 1,
            sentinel_entries: vec!["glow_berries"],
        },
        BlockInteractTableSummary {
            id: "minecraft:gameplay/harvest_sweet_berry_bush",
            pool_count: 2,
            sentinel_entries: vec!["sweet_berries:age=3:count=1", "sweet_berries:count=1..2"],
        },
        BlockInteractTableSummary {
            id: "minecraft:gameplay/carve_pumpkin",
            pool_count: 1,
            sentinel_entries: vec!["pumpkin_seeds:count=4"],
        },
    ]
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ChargedCreeperEntrySummary {
    table_id: &'static str,
    entity_type: &'static str,
    dropped_item: &'static str,
}

fn vanilla_charged_creeper_entries() -> Vec<ChargedCreeperEntrySummary> {
    vec![
        ChargedCreeperEntrySummary {
            table_id: "minecraft:entities/charged_creeper/piglin",
            entity_type: "minecraft:piglin",
            dropped_item: "minecraft:piglin_head",
        },
        ChargedCreeperEntrySummary {
            table_id: "minecraft:entities/charged_creeper/creeper",
            entity_type: "minecraft:creeper",
            dropped_item: "minecraft:creeper_head",
        },
        ChargedCreeperEntrySummary {
            table_id: "minecraft:entities/charged_creeper/skeleton",
            entity_type: "minecraft:skeleton",
            dropped_item: "minecraft:skeleton_skull",
        },
        ChargedCreeperEntrySummary {
            table_id: "minecraft:entities/charged_creeper/wither_skeleton",
            entity_type: "minecraft:wither_skeleton",
            dropped_item: "minecraft:wither_skeleton_skull",
        },
        ChargedCreeperEntrySummary {
            table_id: "minecraft:entities/charged_creeper/zombie",
            entity_type: "minecraft:zombie",
            dropped_item: "minecraft:zombie_head",
        },
    ]
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ChargedCreeperLootSummary {
    dispatcher_table_id: &'static str,
    entry_count: usize,
    per_entry_pool_count: usize,
    per_entry_roll_shape: &'static str,
    dispatcher_roll_shape: &'static str,
}

fn vanilla_charged_creeper_loot_summary() -> ChargedCreeperLootSummary {
    ChargedCreeperLootSummary {
        dispatcher_table_id: "minecraft:entities/charged_creeper",
        entry_count: vanilla_charged_creeper_entries().len(),
        per_entry_pool_count: 1,
        per_entry_roll_shape: "constant:1",
        dispatcher_roll_shape: "constant:1",
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct EntityInteractTableSummary {
    id: &'static str,
    pool_count: usize,
    roll_shape: &'static str,
    item: &'static str,
}

fn vanilla_entity_interact_tables() -> Vec<EntityInteractTableSummary> {
    vec![EntityInteractTableSummary {
        id: "minecraft:gameplay/armadillo_brush",
        pool_count: 1,
        roll_shape: "constant:1",
        item: "minecraft:armadillo_scute",
    }]
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct EquipmentTableSummary {
    id: &'static str,
    pool_count: usize,
    nested_reference: Option<&'static str>,
    weighted_entries: Vec<&'static str>,
    enchanted_entries: Vec<&'static str>,
}

fn vanilla_equipment_tables() -> Vec<EquipmentTableSummary> {
    vec![
        EquipmentTableSummary {
            id: "minecraft:equipment/trial_chamber",
            pool_count: 1,
            nested_reference: None,
            weighted_entries: vec![
                "chainmail_helmet+chainmail_chestplate:copper_bolt_trim@4",
                "iron_helmet+iron_chestplate:copper_flow_trim@2",
                "diamond_helmet+diamond_chestplate:copper_flow_trim@1",
            ],
            enchanted_entries: vec![],
        },
        EquipmentTableSummary {
            id: "minecraft:equipment/trial_chamber_melee",
            pool_count: 2,
            nested_reference: Some("minecraft:equipment/trial_chamber"),
            weighted_entries: vec!["iron_sword@4", "diamond_sword@1"],
            enchanted_entries: vec!["iron_sword:sharpness=1", "iron_sword:knockback=1"],
        },
        EquipmentTableSummary {
            id: "minecraft:equipment/trial_chamber_ranged",
            pool_count: 2,
            nested_reference: Some("minecraft:equipment/trial_chamber"),
            weighted_entries: vec!["bow@2"],
            enchanted_entries: vec!["bow:power=1", "bow:punch=1"],
        },
    ]
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TrialChamberEquipmentHelperSummary {
    pool_count: usize,
    roll_shape: &'static str,
    chance_per_piece: &'static str,
    trim_component: &'static str,
    enchantments: Vec<&'static str>,
}

fn trial_chamber_equipment_helper() -> TrialChamberEquipmentHelperSummary {
    TrialChamberEquipmentHelperSummary {
        pool_count: 2,
        roll_shape: "constant:1",
        chance_per_piece: "random_chance:0.5",
        trim_component: "minecraft:trim",
        enchantments: vec![
            "protection=4",
            "projectile_protection=4",
            "fire_protection=4",
        ],
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FishingDispatcherEntrySummary {
    table_id: &'static str,
    weight: i32,
    quality: i32,
    condition: Option<&'static str>,
}

fn vanilla_fishing_dispatcher_entries() -> Vec<FishingDispatcherEntrySummary> {
    vec![
        FishingDispatcherEntrySummary {
            table_id: "minecraft:gameplay/fishing/junk",
            weight: 10,
            quality: -2,
            condition: None,
        },
        FishingDispatcherEntrySummary {
            table_id: "minecraft:gameplay/fishing/treasure",
            weight: 5,
            quality: 2,
            condition: Some("fishing_hook_in_open_water"),
        },
        FishingDispatcherEntrySummary {
            table_id: "minecraft:gameplay/fishing/fish",
            weight: 85,
            quality: -1,
            condition: None,
        },
    ]
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FishingEntrySummary {
    item: &'static str,
    weight: i32,
    functions: Vec<&'static str>,
    condition: Option<&'static str>,
}

fn vanilla_fishing_fish_entries() -> Vec<FishingEntrySummary> {
    vec![
        FishingEntrySummary {
            item: "minecraft:cod",
            weight: 60,
            functions: vec![],
            condition: None,
        },
        FishingEntrySummary {
            item: "minecraft:salmon",
            weight: 25,
            functions: vec![],
            condition: None,
        },
        FishingEntrySummary {
            item: "minecraft:tropical_fish",
            weight: 2,
            functions: vec![],
            condition: None,
        },
        FishingEntrySummary {
            item: "minecraft:pufferfish",
            weight: 13,
            functions: vec![],
            condition: None,
        },
    ]
}

fn vanilla_fishing_junk_entries() -> Vec<FishingEntrySummary> {
    vec![
        FishingEntrySummary {
            item: "minecraft:lily_pad",
            weight: 17,
            functions: vec![],
            condition: None,
        },
        FishingEntrySummary {
            item: "minecraft:leather_boots",
            weight: 10,
            functions: vec!["damage=0.0..0.9"],
            condition: None,
        },
        FishingEntrySummary {
            item: "minecraft:leather",
            weight: 10,
            functions: vec![],
            condition: None,
        },
        FishingEntrySummary {
            item: "minecraft:bone",
            weight: 10,
            functions: vec![],
            condition: None,
        },
        FishingEntrySummary {
            item: "minecraft:potion",
            weight: 10,
            functions: vec!["potion=water"],
            condition: None,
        },
        FishingEntrySummary {
            item: "minecraft:string",
            weight: 5,
            functions: vec![],
            condition: None,
        },
        FishingEntrySummary {
            item: "minecraft:fishing_rod",
            weight: 2,
            functions: vec!["damage=0.0..0.9"],
            condition: None,
        },
        FishingEntrySummary {
            item: "minecraft:bowl",
            weight: 10,
            functions: vec![],
            condition: None,
        },
        FishingEntrySummary {
            item: "minecraft:stick",
            weight: 5,
            functions: vec![],
            condition: None,
        },
        FishingEntrySummary {
            item: "minecraft:ink_sac",
            weight: 1,
            functions: vec!["count=10"],
            condition: None,
        },
        FishingEntrySummary {
            item: "minecraft:tripwire_hook",
            weight: 10,
            functions: vec![],
            condition: None,
        },
        FishingEntrySummary {
            item: "minecraft:rotten_flesh",
            weight: 10,
            functions: vec![],
            condition: None,
        },
        FishingEntrySummary {
            item: "minecraft:bamboo",
            weight: 10,
            functions: vec![],
            condition: Some("biome in jungle,sparse_jungle,bamboo_jungle"),
        },
    ]
}

fn vanilla_fishing_treasure_entries() -> Vec<FishingEntrySummary> {
    vec![
        FishingEntrySummary {
            item: "minecraft:name_tag",
            weight: 1,
            functions: vec![],
            condition: None,
        },
        FishingEntrySummary {
            item: "minecraft:saddle",
            weight: 1,
            functions: vec![],
            condition: None,
        },
        FishingEntrySummary {
            item: "minecraft:bow",
            weight: 1,
            functions: vec!["damage=0.0..0.25", "enchant_with_levels=30"],
            condition: None,
        },
        FishingEntrySummary {
            item: "minecraft:fishing_rod",
            weight: 1,
            functions: vec!["damage=0.0..0.25", "enchant_with_levels=30"],
            condition: None,
        },
        FishingEntrySummary {
            item: "minecraft:book",
            weight: 1,
            functions: vec!["enchant_with_levels=30"],
            condition: None,
        },
        FishingEntrySummary {
            item: "minecraft:nautilus_shell",
            weight: 1,
            functions: vec![],
            condition: None,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn count_occurrences(source: &str, needle: &str) -> usize {
        source.match_indices(needle).count()
    }

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
        assert_eq!(
            provider.subproviders,
            vec![("TradeRebalanceChestLoot", "chest")]
        );
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
        assert!(tables[0]
            .sentinel_entries
            .iter()
            .any(|entry| entry.contains("suspicious_stew")));
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
            assert!(
                source.contains(expected),
                "missing Java sentinel: {expected}"
            );
        }
    }

    #[test]
    fn vanilla_block_interact_loot_matches_java_table_set_and_counts() {
        let tables = vanilla_block_interact_tables();
        assert_eq!(
            tables.iter().map(|table| table.id).collect::<Vec<_>>(),
            vec![
                "minecraft:gameplay/harvest_beehive",
                "minecraft:gameplay/harvest_cave_vine",
                "minecraft:gameplay/harvest_sweet_berry_bush",
                "minecraft:gameplay/carve_pumpkin",
            ]
        );
        assert_eq!(
            tables
                .iter()
                .map(|table| table.pool_count)
                .collect::<Vec<_>>(),
            vec![1, 1, 2, 1]
        );
        assert!(tables[0].sentinel_entries.contains(&"honeycomb:count=3"));
        assert!(tables[2]
            .sentinel_entries
            .contains(&"sweet_berries:age=3:count=1"));
        assert!(tables[3]
            .sentinel_entries
            .contains(&"pumpkin_seeds:count=4"));
    }

    #[test]
    fn vanilla_block_interact_java_source_sentinels_match_authoritative_file() {
        let source = include_str!(
            "../../decompiled-server-26.1.2/net/minecraft/data/loot/packs/VanillaBlockInteractLoot.java"
        );
        for expected in [
            "BuiltInLootTables.HARVEST_BEEHIVE",
            "LootItem.lootTableItem(Items.HONEYCOMB).apply(SetItemCountFunction.setCount(ConstantValue.exactly(3.0F)))",
            "BuiltInLootTables.HARVEST_CAVE_VINE",
            "BuiltInLootTables.HARVEST_SWEET_BERRY_BUSH",
            "hasProperty(SweetBerryBushBlock.AGE, 3)",
            "UniformGenerator.between(1.0F, 2.0F)",
            "BuiltInLootTables.CARVE_PUMPKIN",
            "LootItem.lootTableItem(Items.PUMPKIN_SEEDS).apply(SetItemCountFunction.setCount(ConstantValue.exactly(4.0F)))",
        ] {
            assert!(source.contains(expected), "missing Java sentinel: {expected}");
        }
    }

    #[test]
    fn vanilla_charged_creeper_loot_matches_java_entry_order_and_heads() {
        let entries = vanilla_charged_creeper_entries();
        assert_eq!(
            entries
                .iter()
                .map(|entry| (entry.table_id, entry.entity_type, entry.dropped_item))
                .collect::<Vec<_>>(),
            vec![
                (
                    "minecraft:entities/charged_creeper/piglin",
                    "minecraft:piglin",
                    "minecraft:piglin_head"
                ),
                (
                    "minecraft:entities/charged_creeper/creeper",
                    "minecraft:creeper",
                    "minecraft:creeper_head"
                ),
                (
                    "minecraft:entities/charged_creeper/skeleton",
                    "minecraft:skeleton",
                    "minecraft:skeleton_skull"
                ),
                (
                    "minecraft:entities/charged_creeper/wither_skeleton",
                    "minecraft:wither_skeleton",
                    "minecraft:wither_skeleton_skull"
                ),
                (
                    "minecraft:entities/charged_creeper/zombie",
                    "minecraft:zombie",
                    "minecraft:zombie_head"
                ),
            ]
        );

        let summary = vanilla_charged_creeper_loot_summary();
        assert_eq!(
            summary.dispatcher_table_id,
            "minecraft:entities/charged_creeper"
        );
        assert_eq!(summary.entry_count, 5);
        assert_eq!(summary.per_entry_pool_count, 1);
        assert_eq!(summary.per_entry_roll_shape, "constant:1");
        assert_eq!(summary.dispatcher_roll_shape, "constant:1");
    }

    #[test]
    fn vanilla_charged_creeper_java_source_sentinels_match_authoritative_file() {
        let source = include_str!(
            "../../decompiled-server-26.1.2/net/minecraft/data/loot/packs/VanillaChargedCreeperExplosionLoot.java"
        );
        for expected in [
            "private static final List<VanillaChargedCreeperExplosionLoot.Entry> ENTRIES = List.of(",
            "BuiltInLootTables.CHARGED_CREEPER_PIGLIN, EntityType.PIGLIN, Items.PIGLIN_HEAD",
            "BuiltInLootTables.CHARGED_CREEPER_CREEPER, EntityType.CREEPER, Items.CREEPER_HEAD",
            "BuiltInLootTables.CHARGED_CREEPER_SKELETON, EntityType.SKELETON, Items.SKELETON_SKULL",
            "BuiltInLootTables.CHARGED_CREEPER_WITHER_SKELETON, EntityType.WITHER_SKELETON, Items.WITHER_SKELETON_SKULL",
            "BuiltInLootTables.CHARGED_CREEPER_ZOMBIE, EntityType.ZOMBIE, Items.ZOMBIE_HEAD",
            "HolderGetter<EntityType<?>> entityTypes = this.registries.lookupOrThrow(Registries.ENTITY_TYPE)",
            "List<LootPoolEntryContainer.Builder<?>> alternatives = new ArrayList<>(ENTRIES.size())",
            "LootPool.lootPool().setRolls(ConstantValue.exactly(1.0F)).add(LootItem.lootTableItem(entry.item))",
            "EntityPredicate.Builder.entity().entityType(EntityTypePredicate.of(entityTypes, entry.entityType))",
            "alternatives.add(NestedLootTable.lootTableReference(entry.lootTable).when(predicate))",
            "BuiltInLootTables.CHARGED_CREEPER",
            "AlternativesEntry.alternatives(alternatives.toArray(LootPoolEntryContainer.Builder[]::new))",
            "private record Entry(ResourceKey<LootTable> lootTable, EntityType<?> entityType, Item item)",
        ] {
            assert!(source.contains(expected), "missing Java sentinel: {expected}");
        }
    }

    #[test]
    fn vanilla_entity_interact_loot_matches_java_armadillo_brush_table() {
        let tables = vanilla_entity_interact_tables();
        assert_eq!(
            tables,
            vec![EntityInteractTableSummary {
                id: "minecraft:gameplay/armadillo_brush",
                pool_count: 1,
                roll_shape: "constant:1",
                item: "minecraft:armadillo_scute",
            }]
        );
    }

    #[test]
    fn vanilla_entity_interact_java_source_sentinels_match_authoritative_file() {
        let source = include_str!(
            "../../decompiled-server-26.1.2/net/minecraft/data/loot/packs/VanillaEntityInteractLoot.java"
        );
        for expected in [
            "public record VanillaEntityInteractLoot(HolderLookup.Provider registries) implements LootTableSubProvider",
            "BuiltInLootTables.ARMADILLO_BRUSH",
            "LootTable.lootTable().withPool(LootPool.lootPool().setRolls(ConstantValue.exactly(1.0F)).add(LootItem.lootTableItem(Items.ARMADILLO_SCUTE)))",
        ] {
            assert!(source.contains(expected), "missing Java sentinel: {expected}");
        }
    }

    #[test]
    fn vanilla_equipment_loot_matches_java_table_shapes() {
        let tables = vanilla_equipment_tables();
        assert_eq!(
            tables.iter().map(|table| table.id).collect::<Vec<_>>(),
            vec![
                "minecraft:equipment/trial_chamber",
                "minecraft:equipment/trial_chamber_melee",
                "minecraft:equipment/trial_chamber_ranged",
            ]
        );
        assert_eq!(
            tables
                .iter()
                .map(|table| (table.pool_count, table.nested_reference))
                .collect::<Vec<_>>(),
            vec![
                (1, None),
                (2, Some("minecraft:equipment/trial_chamber")),
                (2, Some("minecraft:equipment/trial_chamber")),
            ]
        );
        assert_eq!(
            tables[0].weighted_entries,
            vec![
                "chainmail_helmet+chainmail_chestplate:copper_bolt_trim@4",
                "iron_helmet+iron_chestplate:copper_flow_trim@2",
                "diamond_helmet+diamond_chestplate:copper_flow_trim@1",
            ]
        );
        assert_eq!(
            tables[1].enchanted_entries,
            vec!["iron_sword:sharpness=1", "iron_sword:knockback=1"]
        );
        assert_eq!(
            tables[2].enchanted_entries,
            vec!["bow:power=1", "bow:punch=1"]
        );
    }

    #[test]
    fn vanilla_equipment_helper_matches_java_trim_and_enchantment_shape() {
        let helper = trial_chamber_equipment_helper();
        assert_eq!(helper.pool_count, 2);
        assert_eq!(helper.roll_shape, "constant:1");
        assert_eq!(helper.chance_per_piece, "random_chance:0.5");
        assert_eq!(helper.trim_component, "minecraft:trim");
        assert_eq!(
            helper.enchantments,
            vec![
                "protection=4",
                "projectile_protection=4",
                "fire_protection=4"
            ]
        );
    }

    #[test]
    fn vanilla_equipment_java_source_counts_match_authoritative_file() {
        let source = include_str!(
            "../../decompiled-server-26.1.2/net/minecraft/data/loot/packs/VanillaEquipmentLoot.java"
        );
        assert_eq!(count_occurrences(source, "output.accept"), 3);
        assert_eq!(count_occurrences(source, "LootPool.lootPool("), 7);
        assert_eq!(count_occurrences(source, "LootItem.lootTableItem"), 9);
        assert_eq!(
            count_occurrences(source, "NestedLootTable.inlineLootTable"),
            3
        );
        assert_eq!(
            count_occurrences(source, "NestedLootTable.lootTableReference"),
            2
        );
        assert_eq!(count_occurrences(source, "withEnchantment"), 10);
        assert_eq!(
            count_occurrences(source, "SetComponentsFunction.setComponent"),
            2
        );
        assert_eq!(
            count_occurrences(source, "LootItemRandomChanceCondition.randomChance"),
            2
        );
    }

    #[test]
    fn vanilla_equipment_java_source_sentinels_match_authoritative_file() {
        let source = include_str!(
            "../../decompiled-server-26.1.2/net/minecraft/data/loot/packs/VanillaEquipmentLoot.java"
        );
        for expected in [
            "public record VanillaEquipmentLoot(HolderLookup.Provider registries) implements LootTableSubProvider",
            "BuiltInLootTables.EQUIPMENT_TRIAL_CHAMBER",
            "BuiltInLootTables.EQUIPMENT_TRIAL_CHAMBER_MELEE",
            "BuiltInLootTables.EQUIPMENT_TRIAL_CHAMBER_RANGED",
            "ArmorTrim flowTrim = new ArmorTrim(trimMaterials.getOrThrow(TrimMaterials.COPPER), trimPatterns.getOrThrow(TrimPatterns.FLOW))",
            "ArmorTrim boltTrim = new ArmorTrim(trimMaterials.getOrThrow(TrimMaterials.COPPER), trimPatterns.getOrThrow(TrimPatterns.BOLT))",
            "trialChamberEquipment(Items.CHAINMAIL_HELMET, Items.CHAINMAIL_CHESTPLATE, boltTrim, enchantments)",
            "trialChamberEquipment(Items.IRON_HELMET, Items.IRON_CHESTPLATE, flowTrim, enchantments)",
            "trialChamberEquipment(Items.DIAMOND_HELMET, Items.DIAMOND_CHESTPLATE, flowTrim, enchantments)",
            "LootPool.lootPool().setRolls(ConstantValue.exactly(1.0F)).add(NestedLootTable.lootTableReference(BuiltInLootTables.EQUIPMENT_TRIAL_CHAMBER))",
            "LootItem.lootTableItem(Items.IRON_SWORD).setWeight(4)",
            "withEnchantment(enchantments.getOrThrow(Enchantments.SHARPNESS), ConstantValue.exactly(1.0F))",
            "withEnchantment(enchantments.getOrThrow(Enchantments.KNOCKBACK), ConstantValue.exactly(1.0F))",
            "LootItem.lootTableItem(Items.DIAMOND_SWORD)",
            "LootItem.lootTableItem(Items.BOW).setWeight(2)",
            "withEnchantment(enchantments.getOrThrow(Enchantments.POWER), ConstantValue.exactly(1.0F))",
            "withEnchantment(enchantments.getOrThrow(Enchantments.PUNCH), ConstantValue.exactly(1.0F))",
            "public static LootTable.Builder trialChamberEquipment(",
            "LootItemRandomChanceCondition.randomChance(0.5F)",
            "SetComponentsFunction.setComponent(DataComponents.TRIM, trim)",
            "withEnchantment(enchantments.getOrThrow(Enchantments.PROTECTION), ConstantValue.exactly(4.0F))",
            "withEnchantment(enchantments.getOrThrow(Enchantments.PROJECTILE_PROTECTION), ConstantValue.exactly(4.0F))",
            "withEnchantment(enchantments.getOrThrow(Enchantments.FIRE_PROTECTION), ConstantValue.exactly(4.0F))",
        ] {
            assert!(source.contains(expected), "missing Java sentinel: {expected}");
        }
    }

    #[test]
    fn vanilla_fishing_dispatcher_matches_java_category_weights() {
        let entries = vanilla_fishing_dispatcher_entries();
        assert_eq!(
            entries,
            vec![
                FishingDispatcherEntrySummary {
                    table_id: "minecraft:gameplay/fishing/junk",
                    weight: 10,
                    quality: -2,
                    condition: None,
                },
                FishingDispatcherEntrySummary {
                    table_id: "minecraft:gameplay/fishing/treasure",
                    weight: 5,
                    quality: 2,
                    condition: Some("fishing_hook_in_open_water"),
                },
                FishingDispatcherEntrySummary {
                    table_id: "minecraft:gameplay/fishing/fish",
                    weight: 85,
                    quality: -1,
                    condition: None,
                },
            ]
        );
    }

    #[test]
    fn vanilla_fishing_fish_loot_matches_java_weights() {
        let entries = vanilla_fishing_fish_entries();
        assert_eq!(
            entries
                .iter()
                .map(|entry| (entry.item, entry.weight))
                .collect::<Vec<_>>(),
            vec![
                ("minecraft:cod", 60),
                ("minecraft:salmon", 25),
                ("minecraft:tropical_fish", 2),
                ("minecraft:pufferfish", 13),
            ]
        );
        assert!(entries
            .iter()
            .all(|entry| entry.functions.is_empty() && entry.condition.is_none()));
    }

    #[test]
    fn vanilla_fishing_junk_loot_matches_java_weights_functions_and_biome_gate() {
        let entries = vanilla_fishing_junk_entries();
        assert_eq!(entries.len(), 13);
        assert_eq!(
            entries
                .iter()
                .map(|entry| (entry.item, entry.weight))
                .collect::<Vec<_>>(),
            vec![
                ("minecraft:lily_pad", 17),
                ("minecraft:leather_boots", 10),
                ("minecraft:leather", 10),
                ("minecraft:bone", 10),
                ("minecraft:potion", 10),
                ("minecraft:string", 5),
                ("minecraft:fishing_rod", 2),
                ("minecraft:bowl", 10),
                ("minecraft:stick", 5),
                ("minecraft:ink_sac", 1),
                ("minecraft:tripwire_hook", 10),
                ("minecraft:rotten_flesh", 10),
                ("minecraft:bamboo", 10),
            ]
        );
        assert_eq!(entries[1].functions, vec!["damage=0.0..0.9"]);
        assert_eq!(entries[4].functions, vec!["potion=water"]);
        assert_eq!(entries[6].functions, vec!["damage=0.0..0.9"]);
        assert_eq!(entries[9].functions, vec!["count=10"]);
        assert_eq!(
            entries[12].condition,
            Some("biome in jungle,sparse_jungle,bamboo_jungle")
        );
    }

    #[test]
    fn vanilla_fishing_treasure_loot_matches_java_functions() {
        let entries = vanilla_fishing_treasure_entries();
        assert_eq!(entries.len(), 6);
        assert_eq!(
            entries
                .iter()
                .map(|entry| (entry.item, entry.functions.clone()))
                .collect::<Vec<_>>(),
            vec![
                ("minecraft:name_tag", vec![]),
                ("minecraft:saddle", vec![]),
                (
                    "minecraft:bow",
                    vec!["damage=0.0..0.25", "enchant_with_levels=30"]
                ),
                (
                    "minecraft:fishing_rod",
                    vec!["damage=0.0..0.25", "enchant_with_levels=30"]
                ),
                ("minecraft:book", vec!["enchant_with_levels=30"]),
                ("minecraft:nautilus_shell", vec![]),
            ]
        );
        assert!(entries
            .iter()
            .all(|entry| entry.weight == 1 && entry.condition.is_none()));
    }

    #[test]
    fn vanilla_fishing_java_source_counts_match_authoritative_file() {
        let source = include_str!(
            "../../decompiled-server-26.1.2/net/minecraft/data/loot/packs/VanillaFishingLoot.java"
        );
        assert_eq!(count_occurrences(source, "output.accept"), 4);
        assert_eq!(count_occurrences(source, "LootPool.lootPool("), 4);
        assert_eq!(count_occurrences(source, "LootItem.lootTableItem"), 23);
        assert_eq!(
            count_occurrences(source, "NestedLootTable.lootTableReference"),
            3
        );
        assert_eq!(
            count_occurrences(source, "SetItemDamageFunction.setDamage"),
            4
        );
        assert_eq!(
            count_occurrences(source, "EnchantWithLevelsFunction.enchantWithLevels"),
            3
        );
        assert_eq!(count_occurrences(source, "SetPotionFunction.setPotion"), 1);
        assert_eq!(
            count_occurrences(source, "SetItemCountFunction.setCount"),
            1
        );
        assert_eq!(count_occurrences(source, "setWeight"), 20);
        assert_eq!(count_occurrences(source, "setQuality"), 3);
        assert_eq!(count_occurrences(source, "LocationCheck.checkLocation"), 1);
        assert_eq!(
            count_occurrences(source, "FishingHookPredicate.inOpenWater"),
            1
        );
    }

    #[test]
    fn vanilla_fishing_java_source_sentinels_match_authoritative_file() {
        let source = include_str!(
            "../../decompiled-server-26.1.2/net/minecraft/data/loot/packs/VanillaFishingLoot.java"
        );
        for expected in [
            "public record VanillaFishingLoot(HolderLookup.Provider registries) implements LootTableSubProvider",
            "BuiltInLootTables.FISHING",
            "BuiltInLootTables.FISHING_FISH",
            "BuiltInLootTables.FISHING_JUNK",
            "BuiltInLootTables.FISHING_TREASURE",
            "NestedLootTable.lootTableReference(BuiltInLootTables.FISHING_JUNK).setWeight(10).setQuality(-2)",
            "NestedLootTable.lootTableReference(BuiltInLootTables.FISHING_TREASURE)",
            "FishingHookPredicate.inOpenWater(true)",
            "NestedLootTable.lootTableReference(BuiltInLootTables.FISHING_FISH).setWeight(85).setQuality(-1)",
            "LootItem.lootTableItem(Items.COD).setWeight(60)",
            "LootItem.lootTableItem(Items.SALMON).setWeight(25)",
            "LootItem.lootTableItem(Items.TROPICAL_FISH).setWeight(2)",
            "LootItem.lootTableItem(Items.PUFFERFISH).setWeight(13)",
            "LootItem.lootTableItem(Items.LEATHER_BOOTS).setWeight(10).apply(SetItemDamageFunction.setDamage(UniformGenerator.between(0.0F, 0.9F)))",
            "LootItem.lootTableItem(Items.POTION).setWeight(10).apply(SetPotionFunction.setPotion(Potions.WATER))",
            "LootItem.lootTableItem(Items.FISHING_ROD).setWeight(2).apply(SetItemDamageFunction.setDamage(UniformGenerator.between(0.0F, 0.9F)))",
            "LootItem.lootTableItem(Items.INK_SAC).setWeight(1).apply(SetItemCountFunction.setCount(ConstantValue.exactly(10.0F)))",
            "HolderSet.direct(\n                                       biomes.getOrThrow(Biomes.JUNGLE), biomes.getOrThrow(Biomes.SPARSE_JUNGLE), biomes.getOrThrow(Biomes.BAMBOO_JUNGLE)",
            "LootItem.lootTableItem(Items.BOW)\n                        .apply(SetItemDamageFunction.setDamage(UniformGenerator.between(0.0F, 0.25F)))",
            "EnchantWithLevelsFunction.enchantWithLevels(this.registries, ConstantValue.exactly(30.0F))",
            "LootItem.lootTableItem(Items.NAUTILUS_SHELL)",
            "public static LootTable.Builder fishingFishLootTable()",
        ] {
            assert!(source.contains(expected), "missing Java sentinel: {expected}");
        }
    }
}
