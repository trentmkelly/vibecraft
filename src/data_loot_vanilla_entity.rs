const VANILLA_ENTITY_LOOT_JAVA: &str = vibecraft_java_source!("/net/minecraft/data/loot/packs/VanillaEntityLoot.java");

const EMPTY_ENTITY_TABLES: &[&str] = &[
    "ALLAY",
    "ARMADILLO",
    "ARMOR_STAND",
    "AXOLOTL",
    "BAT",
    "BEE",
    "CAMEL",
    "ENDER_DRAGON",
    "ENDERMITE",
    "FOX",
    "FROG",
    "HAPPY_GHAST",
    "GIANT",
    "GOAT",
    "ILLUSIONER",
    "MANNEQUIN",
    "OCELOT",
    "PLAYER",
    "SILVERFISH",
    "SNIFFER",
    "TADPOLE",
    "VEX",
    "VILLAGER",
    "WANDERING_TRADER",
    "WITHER",
    "WOLF",
    "CREAKING",
    "PIGLIN",
    "PIGLIN_BRUTE",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SourceCount {
    needle: &'static str,
    count: usize,
}

const VANILLA_ENTITY_SOURCE_COUNTS: &[SourceCount] = &[
    SourceCount {
        needle: "this.add(",
        count: 93,
    },
    SourceCount {
        needle: ".withPool(",
        count: 108,
    },
    SourceCount {
        needle: ".setRolls(",
        count: 104,
    },
    SourceCount {
        needle: "LootItem.lootTableItem(",
        count: 122,
    },
    SourceCount {
        needle: "EmptyLootItem.emptyItem(",
        count: 3,
    },
    SourceCount {
        needle: "NestedLootTable.lootTableReference(",
        count: 2,
    },
    SourceCount {
        needle: "TagEntry.expandTag(",
        count: 1,
    },
    SourceCount {
        needle: "SetItemCountFunction",
        count: 90,
    },
    SourceCount {
        needle: "EnchantedCountIncreaseFunction",
        count: 81,
    },
    SourceCount {
        needle: "SmeltItemFunction",
        count: 20,
    },
    SourceCount {
        needle: "LootItemKilledByPlayerCondition",
        count: 25,
    },
    SourceCount {
        needle: "LootItemRandomChanceCondition",
        count: 5,
    },
    SourceCount {
        needle: "LootItemRandomChanceWithEnchantedBonusCondition",
        count: 12,
    },
    SourceCount {
        needle: "LootItemEntityPropertyCondition",
        count: 8,
    },
    SourceCount {
        needle: "DamageSourceCondition",
        count: 3,
    },
    SourceCount {
        needle: "SetPotionFunction",
        count: 4,
    },
    SourceCount {
        needle: "SetOminousBottleAmplifierFunction",
        count: 2,
    },
];

const CORE_ENTITY_SENTINELS: &[&str] = &[
    "super(FeatureFlags.REGISTRY.allFlags(), registries)",
    "HolderGetter<EntityType<?>> entityTypes = this.registries.lookupOrThrow(Registries.ENTITY_TYPE)",
    "HolderGetter<FrogVariant> frogVariants = this.registries.lookupOrThrow(Registries.FROG_VARIANT)",
    "EntityType.BLAZE",
    "LootItem.lootTableItem(Items.BLAZE_ROD)",
    "EntityType.BOGGED",
    "SetPotionFunction.setPotion(Potions.POISON)",
    "EntityType.CAMEL_HUSK",
    "EntityType.COPPER_GOLEM",
    "LootItem.lootTableItem(Items.COPPER_INGOT)",
    "EntityType.NAUTILUS",
    "LootItem.lootTableItem(Items.NAUTILUS_SHELL)",
    "EntityType.PARCHED",
    "SetPotionFunction.setPotion(Potions.WEAKNESS)",
    "EntityType.ZOMBIE_NAUTILUS",
];

const SMELTING_AND_COMMON_DROP_SENTINELS: &[&str] = &[
    "LootItem.lootTableItem(Items.CHICKEN)",
    "LootItem.lootTableItem(Items.BEEF)",
    "LootItem.lootTableItem(Items.COD).apply(SmeltItemFunction.smelted().when(this.shouldSmeltLoot()))",
    "LootItem.lootTableItem(Items.SALMON).apply(SmeltItemFunction.smelted().when(this.shouldSmeltLoot()))",
    "LootItem.lootTableItem(Items.PORKCHOP)",
    "LootItem.lootTableItem(Items.RABBIT)",
    "LootItem.lootTableItem(Items.MUTTON)",
    "LootItem.lootTableItem(Items.POTATO).apply(SmeltItemFunction.smelted().when(this.shouldSmeltLoot()))",
    "LootItem.lootTableItem(Items.TROPICAL_FISH).apply(SetItemCountFunction.setCount(ConstantValue.exactly(1.0F)))",
    "LootItemRandomChanceCondition.randomChance(0.05F)",
];

const SPECIAL_CONDITION_SENTINELS: &[&str] = &[
    "TagEntry.expandTag(ItemTags.CREEPER_DROP_MUSIC_DISCS)",
    "EntityPredicate.Builder.entity().of(entityTypes, EntityTypeTags.SKELETONS)",
    "DamageTypeTags.IS_PROJECTILE",
    "direct(EntityPredicate.Builder.entity().of(entityTypes, EntityType.FIREBALL))",
    "LootItem.lootTableItem(Items.MUSIC_DISC_TEARS)",
    "DamageTypeTags.IS_LIGHTNING",
    "LootItem.lootTableItem(Items.BOWL)",
    "RaiderPredicate.CAPTAIN_WITHOUT_RAID",
    "SetOminousBottleAmplifierFunction.setAmplifier(UniformGenerator.between(0.0F, 4.0F))",
    "LootItem.lootTableItem(Items.MUSIC_DISC_LAVA_CHICKEN)",
    "flags(EntityFlagsPredicate.Builder.flags().setIsBaby(true))",
    "vehicle(EntityPredicate.Builder.entity().of(entityTypes, EntityType.CHICKEN))",
    "vehicle(EntityPredicate.Builder.entity().entityType(EntityTypePredicate.of(entityTypes, EntityType.CAMEL_HUSK)))",
    "vehicle(EntityPredicate.Builder.entity().entityType(EntityTypePredicate.of(entityTypes, EntityType.ZOMBIE_HORSE)))",
];

const FROG_SLIME_SHEEP_SENTINELS: &[&str] = &[
    "this.killedByFrog(entityTypes).invert()",
    "this.killedByFrog(entityTypes)",
    "this.killedByFrogVariant(entityTypes, frogVariants, FrogVariants.WARM)",
    "this.killedByFrogVariant(entityTypes, frogVariants, FrogVariants.COLD)",
    "this.killedByFrogVariant(entityTypes, frogVariants, FrogVariants.TEMPERATE)",
    "LootItem.lootTableItem(Items.PEARLESCENT_FROGLIGHT)",
    "LootItem.lootTableItem(Items.VERDANT_FROGLIGHT)",
    "LootItem.lootTableItem(Items.OCHRE_FROGLIGHT)",
    "SlimePredicate.sized(MinMaxBounds.Ints.atLeast(2))",
    "SlimePredicate.sized(MinMaxBounds.Ints.exactly(1))",
    "withPool(createSheepDispatchPool(BuiltInLootTables.SHEEP_BY_DYE))",
    "LootData.WOOL_ITEM_BY_DYE",
    "BuiltInLootTables.SHEEP_BY_DYE.get(dye)",
];

const ELDER_GUARDIAN_SENTINELS: &[&str] = &[
    "public LootTable.Builder elderGuardianLootTable()",
    "LootItem.lootTableItem(Blocks.WET_SPONGE)",
    "NestedLootTable.lootTableReference(BuiltInLootTables.FISHING_FISH)",
    "LootItem.lootTableItem(Items.TIDE_ARMOR_TRIM_SMITHING_TEMPLATE).setWeight(1)",
    "LootItemRandomChanceWithEnchantedBonusCondition.randomChanceAndLootingBoost(this.registries, 0.025F, 0.01F)",
];

fn count_occurrences(source: &str, needle: &str) -> usize {
    source.match_indices(needle).count()
}

fn assert_source_contains_all(sentinels: &[&str]) {
    for sentinel in sentinels {
        assert!(
            VANILLA_ENTITY_LOOT_JAVA.contains(sentinel),
            "missing Java sentinel: {sentinel}"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vanilla_entity_loot_source_counts_match_java_generator_shape() {
        for expected in VANILLA_ENTITY_SOURCE_COUNTS {
            assert_eq!(
                count_occurrences(VANILLA_ENTITY_LOOT_JAVA, expected.needle),
                expected.count,
                "source count changed for {}",
                expected.needle
            );
        }
    }

    #[test]
    fn vanilla_entity_loot_empty_tables_match_java_set() {
        assert_eq!(EMPTY_ENTITY_TABLES.len(), 29);
        for entity in EMPTY_ENTITY_TABLES {
            assert!(
                VANILLA_ENTITY_LOOT_JAVA.contains(&format!(
                    "this.add(EntityType.{entity}, LootTable.lootTable())"
                )),
                "missing empty entity loot table for {entity}"
            );
        }
    }

    #[test]
    fn vanilla_entity_loot_core_entities_match_java_sentinels() {
        assert_source_contains_all(CORE_ENTITY_SENTINELS);
        assert_source_contains_all(SMELTING_AND_COMMON_DROP_SENTINELS);
    }

    #[test]
    fn vanilla_entity_loot_special_conditions_match_java_sentinels() {
        assert_source_contains_all(SPECIAL_CONDITION_SENTINELS);
    }

    #[test]
    fn vanilla_entity_loot_frog_slime_sheep_logic_matches_java_sentinels() {
        assert_source_contains_all(FROG_SLIME_SHEEP_SENTINELS);
    }

    #[test]
    fn vanilla_entity_loot_elder_guardian_helper_matches_java_sentinels() {
        assert_source_contains_all(ELDER_GUARDIAN_SENTINELS);
    }
}
