const VANILLA_CHEST_LOOT_JAVA: &str = vibecraft_java_source!("/net/minecraft/data/loot/packs/VanillaChestLoot.java");

const VANILLA_CHEST_OUTPUT_TABLES: &[&str] = &[
    "ABANDONED_MINESHAFT",
    "BASTION_BRIDGE",
    "BASTION_HOGLIN_STABLE",
    "BASTION_OTHER",
    "BASTION_TREASURE",
    "BURIED_TREASURE",
    "ANCIENT_CITY",
    "ANCIENT_CITY_ICE_BOX",
    "DESERT_PYRAMID",
    "END_CITY_TREASURE",
    "IGLOO_CHEST",
    "JUNGLE_TEMPLE",
    "JUNGLE_TEMPLE_DISPENSER",
    "NETHER_BRIDGE",
    "PILLAGER_OUTPOST",
    "SHIPWRECK_MAP",
    "SHIPWRECK_SUPPLY",
    "SHIPWRECK_TREASURE",
    "SIMPLE_DUNGEON",
    "SPAWN_BONUS_CHEST",
    "STRONGHOLD_CORRIDOR",
    "STRONGHOLD_CROSSING",
    "STRONGHOLD_LIBRARY",
    "UNDERWATER_RUIN_BIG",
    "UNDERWATER_RUIN_SMALL",
    "VILLAGE_WEAPONSMITH",
    "VILLAGE_TOOLSMITH",
    "VILLAGE_CARTOGRAPHER",
    "VILLAGE_MASON",
    "VILLAGE_ARMORER",
    "VILLAGE_SHEPHERD",
    "VILLAGE_BUTCHER",
    "VILLAGE_FLETCHER",
    "VILLAGE_FISHER",
    "VILLAGE_TANNERY",
    "VILLAGE_TEMPLE",
    "VILLAGE_PLAINS_HOUSE",
    "VILLAGE_TAIGA_HOUSE",
    "VILLAGE_SAVANNA_HOUSE",
    "VILLAGE_SNOWY_HOUSE",
    "VILLAGE_DESERT_HOUSE",
    "WOODLAND_MANSION",
    "RUINED_PORTAL",
    "TRIAL_CHAMBERS_CORRIDOR_DISPENSER",
    "TRIAL_CHAMBERS_WATER_DISPENSER",
    "TRIAL_CHAMBERS_CHAMBER_DISPENSER",
    "TRIAL_CHAMBERS_CORRIDOR_POT",
    "TRIAL_CHAMBERS_SUPPLY",
    "TRIAL_CHAMBERS_ENTRANCE",
    "TRIAL_CHAMBERS_INTERSECTION",
    "TRIAL_CHAMBERS_INTERSECTION_BARREL",
    "TRIAL_CHAMBERS_CORRIDOR",
    "TRIAL_CHAMBERS_REWARD_RARE",
    "TRIAL_CHAMBERS_REWARD_COMMON",
    "TRIAL_CHAMBERS_REWARD_UNIQUE",
    "TRIAL_CHAMBERS_REWARD",
    "TRIAL_CHAMBERS_REWARD_OMINOUS_RARE",
    "TRIAL_CHAMBERS_REWARD_OMINOUS_COMMON",
    "TRIAL_CHAMBERS_REWARD_OMINOUS_UNIQUE",
    "TRIAL_CHAMBERS_REWARD_OMINOUS",
    "SPAWNER_TRIAL_CHAMBER_KEY",
    "SPAWNER_TRIAL_CHAMBER_CONSUMABLES",
    "SPAWNER_OMINOUS_TRIAL_CHAMBER_KEY",
    "SPAWNER_OMINOUS_TRIAL_CHAMBER_CONSUMABLES",
    "SPAWNER_TRIAL_ITEMS_TO_DROP_WHEN_OMINOUS",
];

const VANILLA_CHEST_HELPER_METHODS: &[&str] = &[
    "shipwreckSupplyLootTable",
    "shipwreckMapLootTable",
    "bastionHoglinStableLootTable",
    "bastionBridgeLootTable",
    "endCityTreasureLootTable",
    "netherBridgeLootTable",
    "bastionTreasureLootTable",
    "bastionOtherLootTable",
    "woodlandMansionLootTable",
    "strongholdLibraryLootTable",
    "strongholdCorridorLootTable",
    "ancientCityLootTable",
    "jungleTempleLootTable",
    "shipwreckTreasureLootTable",
    "pillagerOutpostLootTable",
    "desertPyramidLootTable",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SourceCount {
    needle: &'static str,
    count: usize,
}

const VANILLA_CHEST_SOURCE_COUNTS: &[SourceCount] = &[
    SourceCount {
        needle: "output.accept(",
        count: 65,
    },
    SourceCount {
        needle: ".withPool(",
        count: 135,
    },
    SourceCount {
        needle: ".setRolls(",
        count: 135,
    },
    SourceCount {
        needle: "LootItem.lootTableItem(",
        count: 767,
    },
    SourceCount {
        needle: "EmptyLootItem.emptyItem(",
        count: 35,
    },
    SourceCount {
        needle: "NestedLootTable.lootTableReference(",
        count: 8,
    },
    SourceCount {
        needle: "SetItemCountFunction",
        count: 496,
    },
    SourceCount {
        needle: "SetPotionFunction",
        count: 29,
    },
    SourceCount {
        needle: "SetItemDamageFunction",
        count: 25,
    },
    SourceCount {
        needle: "EnchantRandomlyFunction",
        count: 51,
    },
    SourceCount {
        needle: "EnchantWithLevelsFunction",
        count: 32,
    },
    SourceCount {
        needle: "ExplorationMapFunction",
        count: 4,
    },
    SourceCount {
        needle: "SetStewEffectFunction",
        count: 3,
    },
    SourceCount {
        needle: "SetOminousBottleAmplifierFunction",
        count: 3,
    },
    SourceCount {
        needle: "SetInstrumentFunction",
        count: 2,
    },
    SourceCount {
        needle: "SetEnchantmentsFunction",
        count: 2,
    },
    SourceCount {
        needle: "LootItemRandomChanceCondition",
        count: 3,
    },
];

const CORE_STRUCTURE_SENTINELS: &[&str] = &[
    "HolderLookup.RegistryLookup<Enchantment> enchantments = this.registries.lookupOrThrow(Registries.ENCHANTMENT)",
    "BuiltInLootTables.ABANDONED_MINESHAFT",
    "LootItem.lootTableItem(Items.GLOW_BERRIES).setWeight(15)",
    "BuiltInLootTables.BURIED_TREASURE",
    "LootItem.lootTableItem(Items.HEART_OF_THE_SEA)",
    "LootItem.lootTableItem(Items.COPPER_NAUTILUS_ARMOR).setWeight(20)",
    "SetPotionFunction.setPotion(Potions.WATER_BREATHING)",
    "BuiltInLootTables.ANCIENT_CITY_ICE_BOX",
    "withEffect(MobEffects.NIGHT_VISION, UniformGenerator.between(7.0F, 10.0F))",
    "BuiltInLootTables.IGLOO_CHEST",
    "BuiltInLootTables.JUNGLE_TEMPLE_DISPENSER",
    "BuiltInLootTables.SIMPLE_DUNGEON",
    "LootItem.lootTableItem(Items.MUSIC_DISC_OTHERSIDE).setWeight(2)",
    "BuiltInLootTables.SPAWN_BONUS_CHEST",
    "LootItem.lootTableItem(Blocks.MANGROVE_LOG).setWeight(3)",
    "BuiltInLootTables.RUINED_PORTAL",
    "LootItem.lootTableItem(Items.LODESTONE).setWeight(2)",
];

const MAP_AND_OCEAN_SENTINELS: &[&str] = &[
    "BuiltInLootTables.SHIPWRECK_MAP",
    "BuiltInLootTables.SHIPWRECK_SUPPLY",
    "BuiltInLootTables.SHIPWRECK_TREASURE",
    "BuiltInLootTables.UNDERWATER_RUIN_BIG",
    "BuiltInLootTables.UNDERWATER_RUIN_SMALL",
    "ExplorationMapFunction.makeExplorationMap()",
    "setDestination(StructureTags.ON_TREASURE_MAPS)",
    "setMapDecoration(MapDecorationTypes.RED_X)",
    "setName(Component.translatable(\"filled_map.buried_treasure\"), SetNameFunction.Target.ITEM_NAME)",
    "LootItem.lootTableItem(Items.COAST_ARMOR_TRIM_SMITHING_TEMPLATE)",
    "LootItem.lootTableItem(Items.DIAMOND_NAUTILUS_ARMOR).setWeight(2)",
];

const BASTION_AND_STRONGHOLD_SENTINELS: &[&str] = &[
    "BuiltInLootTables.BASTION_BRIDGE",
    "BuiltInLootTables.BASTION_HOGLIN_STABLE",
    "BuiltInLootTables.BASTION_OTHER",
    "BuiltInLootTables.BASTION_TREASURE",
    "LootItem.lootTableItem(Items.SNOUT_ARMOR_TRIM_SMITHING_TEMPLATE).setWeight(1)",
    "LootItem.lootTableItem(Items.NETHERITE_UPGRADE_SMITHING_TEMPLATE).setWeight(1)",
    "withEnchantment(enchantments.getOrThrow(Enchantments.SOUL_SPEED))",
    "LootItem.lootTableItem(Items.MUSIC_DISC_PIGSTEP).setWeight(5)",
    "BuiltInLootTables.STRONGHOLD_CORRIDOR",
    "BuiltInLootTables.STRONGHOLD_CROSSING",
    "BuiltInLootTables.STRONGHOLD_LIBRARY",
    "LootItem.lootTableItem(Items.EYE_ARMOR_TRIM_SMITHING_TEMPLATE).setWeight(1)",
    "EnchantWithLevelsFunction.enchantWithLevels(this.registries, ConstantValue.exactly(30.0F))",
];

const VILLAGE_SENTINELS: &[&str] = &[
    "BuiltInLootTables.VILLAGE_WEAPONSMITH",
    "BuiltInLootTables.VILLAGE_TOOLSMITH",
    "BuiltInLootTables.VILLAGE_CARTOGRAPHER",
    "BuiltInLootTables.VILLAGE_MASON",
    "BuiltInLootTables.VILLAGE_ARMORER",
    "BuiltInLootTables.VILLAGE_SHEPHERD",
    "BuiltInLootTables.VILLAGE_BUTCHER",
    "BuiltInLootTables.VILLAGE_FLETCHER",
    "BuiltInLootTables.VILLAGE_FISHER",
    "BuiltInLootTables.VILLAGE_TANNERY",
    "BuiltInLootTables.VILLAGE_TEMPLE",
    "BuiltInLootTables.VILLAGE_PLAINS_HOUSE",
    "BuiltInLootTables.VILLAGE_TAIGA_HOUSE",
    "BuiltInLootTables.VILLAGE_SAVANNA_HOUSE",
    "BuiltInLootTables.VILLAGE_SNOWY_HOUSE",
    "BuiltInLootTables.VILLAGE_DESERT_HOUSE",
    "LootItem.lootTableItem(Items.BUNDLE).setWeight(1)",
    "LootItem.lootTableItem(Items.COPPER_SPEAR).setWeight(7)",
    "LootItem.lootTableItem(Items.SPRUCE_SIGN).setWeight(1)",
    "LootItem.lootTableItem(Blocks.BLUE_ICE).setWeight(1)",
];

const TRIAL_CHAMBER_SENTINELS: &[&str] = &[
    "BuiltInLootTables.TRIAL_CHAMBERS_CORRIDOR_DISPENSER",
    "BuiltInLootTables.TRIAL_CHAMBERS_WATER_DISPENSER",
    "BuiltInLootTables.TRIAL_CHAMBERS_CHAMBER_DISPENSER",
    "BuiltInLootTables.TRIAL_CHAMBERS_CORRIDOR_POT",
    "BuiltInLootTables.TRIAL_CHAMBERS_SUPPLY",
    "BuiltInLootTables.TRIAL_CHAMBERS_ENTRANCE",
    "BuiltInLootTables.TRIAL_CHAMBERS_INTERSECTION",
    "BuiltInLootTables.TRIAL_CHAMBERS_INTERSECTION_BARREL",
    "BuiltInLootTables.TRIAL_CHAMBERS_CORRIDOR",
    "LootItem.lootTableItem(Items.TRIAL_KEY).apply(SetItemCountFunction.setCount(ConstantValue.exactly(1.0F))).setWeight(10)",
    "LootItem.lootTableItem(Items.MUSIC_DISC_CREATOR_MUSIC_BOX).setWeight(5)",
    "SetPotionFunction.setPotion(Potions.POISON)",
    "SetPotionFunction.setPotion(Potions.WEAKNESS)",
    "SetPotionFunction.setPotion(Potions.HEALING)",
];

const TRIAL_REWARD_AND_SPAWNER_SENTINELS: &[&str] = &[
    "BuiltInLootTables.TRIAL_CHAMBERS_REWARD_RARE",
    "BuiltInLootTables.TRIAL_CHAMBERS_REWARD_COMMON",
    "BuiltInLootTables.TRIAL_CHAMBERS_REWARD_UNIQUE",
    "BuiltInLootTables.TRIAL_CHAMBERS_REWARD",
    "BuiltInLootTables.TRIAL_CHAMBERS_REWARD_OMINOUS_RARE",
    "BuiltInLootTables.TRIAL_CHAMBERS_REWARD_OMINOUS_COMMON",
    "BuiltInLootTables.TRIAL_CHAMBERS_REWARD_OMINOUS_UNIQUE",
    "BuiltInLootTables.TRIAL_CHAMBERS_REWARD_OMINOUS",
    "NestedLootTable.lootTableReference(BuiltInLootTables.TRIAL_CHAMBERS_REWARD_RARE).setWeight(8)",
    "LootItemRandomChanceCondition.randomChance(0.25F)",
    "LootItemRandomChanceCondition.randomChance(0.75F)",
    "SetOminousBottleAmplifierFunction.setAmplifier(UniformGenerator.between(2.0F, 4.0F))",
    "new SetEnchantmentsFunction.Builder().withEnchantment(enchantments.getOrThrow(Enchantments.WIND_BURST), ConstantValue.exactly(1.0F))",
    "BuiltInLootTables.SPAWNER_TRIAL_CHAMBER_KEY",
    "BuiltInLootTables.SPAWNER_TRIAL_CHAMBER_CONSUMABLES",
    "BuiltInLootTables.SPAWNER_OMINOUS_TRIAL_CHAMBER_KEY",
    "BuiltInLootTables.SPAWNER_OMINOUS_TRIAL_CHAMBER_CONSUMABLES",
    "BuiltInLootTables.SPAWNER_TRIAL_ITEMS_TO_DROP_WHEN_OMINOUS",
    "LootItem.lootTableItem(Items.OMINOUS_TRIAL_KEY)",
    "SetPotionFunction.setPotion(Potions.WIND_CHARGED)",
    "SetPotionFunction.setPotion(Potions.OOZING)",
    "SetPotionFunction.setPotion(Potions.WEAVING)",
    "SetPotionFunction.setPotion(Potions.INFESTED)",
];

const HELPER_FEATURE_SENTINELS: &[&str] = &[
    "LootItem.lootTableItem(Items.SPIRE_ARMOR_TRIM_SMITHING_TEMPLATE).setWeight(1)",
    "LootItem.lootTableItem(Items.RIB_ARMOR_TRIM_SMITHING_TEMPLATE).setWeight(1)",
    "LootItem.lootTableItem(Items.DUNE_ARMOR_TRIM_SMITHING_TEMPLATE)",
    "LootItem.lootTableItem(Items.WILD_ARMOR_TRIM_SMITHING_TEMPLATE)",
    "LootItem.lootTableItem(Items.SENTRY_ARMOR_TRIM_SMITHING_TEMPLATE)",
    "LootItem.lootTableItem(Items.WARD_ARMOR_TRIM_SMITHING_TEMPLATE).setWeight(4)",
    "LootItem.lootTableItem(Items.SILENCE_ARMOR_TRIM_SMITHING_TEMPLATE).setWeight(1)",
    "LootItem.lootTableItem(Items.VEX_ARMOR_TRIM_SMITHING_TEMPLATE).setWeight(1)",
    "SetInstrumentFunction.setInstrumentOptions(instruments.getOrThrow(InstrumentTags.REGULAR_GOAT_HORNS))",
    "LootItem.lootTableItem(Items.DIAMOND_SPEAR)",
    "LootItem.lootTableItem(Items.IRON_SPEAR)",
];

fn count_occurrences(source: &str, needle: &str) -> usize {
    source.match_indices(needle).count()
}

fn assert_source_contains_all(sentinels: &[&str]) {
    for sentinel in sentinels {
        assert!(
            VANILLA_CHEST_LOOT_JAVA.contains(sentinel),
            "missing Java sentinel: {sentinel}"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vanilla_chest_loot_emits_all_java_table_ids() {
        assert_eq!(VANILLA_CHEST_OUTPUT_TABLES.len(), 65);
        assert_eq!(
            count_occurrences(VANILLA_CHEST_LOOT_JAVA, "output.accept("),
            65
        );
        for table in VANILLA_CHEST_OUTPUT_TABLES {
            assert!(
                VANILLA_CHEST_LOOT_JAVA.contains(&format!("BuiltInLootTables.{table}")),
                "missing emitted chest table {table}"
            );
        }
    }

    #[test]
    fn vanilla_chest_loot_helper_methods_match_java_surface() {
        assert_eq!(VANILLA_CHEST_HELPER_METHODS.len(), 16);
        for method in VANILLA_CHEST_HELPER_METHODS {
            assert!(
                VANILLA_CHEST_LOOT_JAVA.contains(&format!("public LootTable.Builder {method}(")),
                "missing helper method {method}"
            );
        }
        assert!(VANILLA_CHEST_LOOT_JAVA.contains(
            "public void spawnerLootTables(final BiConsumer<ResourceKey<LootTable>, LootTable.Builder> output)"
        ));
    }

    #[test]
    fn vanilla_chest_loot_source_counts_match_java_generator_shape() {
        for expected in VANILLA_CHEST_SOURCE_COUNTS {
            assert_eq!(
                count_occurrences(VANILLA_CHEST_LOOT_JAVA, expected.needle),
                expected.count,
                "source count changed for {}",
                expected.needle
            );
        }
    }

    #[test]
    fn vanilla_chest_core_structure_tables_match_java_sentinels() {
        assert_source_contains_all(CORE_STRUCTURE_SENTINELS);
        assert_source_contains_all(MAP_AND_OCEAN_SENTINELS);
        assert_source_contains_all(BASTION_AND_STRONGHOLD_SENTINELS);
    }

    #[test]
    fn vanilla_chest_village_tables_match_java_sentinels() {
        assert_source_contains_all(VILLAGE_SENTINELS);
    }

    #[test]
    fn vanilla_chest_trial_chamber_tables_match_java_sentinels() {
        assert_source_contains_all(TRIAL_CHAMBER_SENTINELS);
        assert_source_contains_all(TRIAL_REWARD_AND_SPAWNER_SENTINELS);
    }

    #[test]
    fn vanilla_chest_helper_feature_sentinels_match_java() {
        assert_source_contains_all(HELPER_FEATURE_SENTINELS);
    }
}
