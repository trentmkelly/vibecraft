const VANILLA_BLOCK_LOOT_JAVA: &str = vibecraft_java_source!("/net/minecraft/data/loot/packs/VanillaBlockLoot.java");

#[derive(Debug, Clone, PartialEq)]
struct VanillaBlockLootConstants {
    jungle_leaves_sapling_chances: Vec<f32>,
    explosion_resistant_blocks: Vec<&'static str>,
    enabled_feature_source: &'static str,
}

fn vanilla_block_loot_constants() -> VanillaBlockLootConstants {
    VanillaBlockLootConstants {
        jungle_leaves_sapling_chances: vec![0.025, 0.027777778, 0.03125, 0.041666668, 0.1],
        explosion_resistant_blocks: vec![
            "DRAGON_EGG",
            "BEACON",
            "CONDUIT",
            "SKELETON_SKULL",
            "WITHER_SKELETON_SKULL",
            "PLAYER_HEAD",
            "ZOMBIE_HEAD",
            "CREEPER_HEAD",
            "DRAGON_HEAD",
            "PIGLIN_HEAD",
            "SHULKER_BOX",
            "BLACK_SHULKER_BOX",
            "BLUE_SHULKER_BOX",
            "BROWN_SHULKER_BOX",
            "CYAN_SHULKER_BOX",
            "GRAY_SHULKER_BOX",
            "GREEN_SHULKER_BOX",
            "LIGHT_BLUE_SHULKER_BOX",
            "LIGHT_GRAY_SHULKER_BOX",
            "LIME_SHULKER_BOX",
            "MAGENTA_SHULKER_BOX",
            "ORANGE_SHULKER_BOX",
            "PINK_SHULKER_BOX",
            "PURPLE_SHULKER_BOX",
            "RED_SHULKER_BOX",
            "WHITE_SHULKER_BOX",
            "YELLOW_SHULKER_BOX",
        ],
        enabled_feature_source: "FeatureFlags.REGISTRY.allFlags()",
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct RegistrationCounter {
    source_call: &'static str,
    count: usize,
}

const VANILLA_BLOCK_LOOT_REGISTRATION_COUNTERS: &[RegistrationCounter] = &[
    RegistrationCounter {
        source_call: "this.dropSelf(",
        count: 611,
    },
    RegistrationCounter {
        source_call: "this.add(",
        count: 330,
    },
    RegistrationCounter {
        source_call: "this.dropOther(",
        count: 9,
    },
    RegistrationCounter {
        source_call: "this.dropPottedContents(",
        count: 38,
    },
    RegistrationCounter {
        source_call: "this.dropWhenSilkTouch(",
        count: 68,
    },
    RegistrationCounter {
        source_call: "this.otherWhenSilkTouch(",
        count: 7,
    },
    RegistrationCounter {
        source_call: "this.addNetherVinesDropTable(",
        count: 2,
    },
];

const VANILLA_BLOCK_LOOT_HELPER_COUNTERS: &[RegistrationCounter] = &[
    RegistrationCounter {
        source_call: "createSlabItemTable",
        count: 62,
    },
    RegistrationCounter {
        source_call: "createDoorTable",
        count: 21,
    },
    RegistrationCounter {
        source_call: "createSinglePropConditionTable",
        count: 20,
    },
    RegistrationCounter {
        source_call: "createNameableBlockEntityTable",
        count: 20,
    },
    RegistrationCounter {
        source_call: "createShulkerBoxDrop",
        count: 17,
    },
    RegistrationCounter {
        source_call: "createCandleDrops",
        count: 17,
    },
    RegistrationCounter {
        source_call: "createBannerDrop",
        count: 16,
    },
    RegistrationCounter {
        source_call: "createSingleItemTableWithSilkTouch",
        count: 16,
    },
    RegistrationCounter {
        source_call: "createOreDrop",
        count: 11,
    },
    RegistrationCounter {
        source_call: "createLeavesDrops",
        count: 8,
    },
    RegistrationCounter {
        source_call: "createCopperGolemStatueBlock",
        count: 8,
    },
    RegistrationCounter {
        source_call: "createShearsOnlyDrop",
        count: 5,
    },
    RegistrationCounter {
        source_call: "createShearsOrSilkTouchOnlyDrop",
        count: 4,
    },
    RegistrationCounter {
        source_call: "createSegmentedBlockDrops",
        count: 3,
    },
    RegistrationCounter {
        source_call: "createMultifaceBlockDrops",
        count: 3,
    },
    RegistrationCounter {
        source_call: "createOakLeavesDrops",
        count: 2,
    },
    RegistrationCounter {
        source_call: "createCropDrops",
        count: 2,
    },
    RegistrationCounter {
        source_call: "createCopperOreDrops",
        count: 2,
    },
    RegistrationCounter {
        source_call: "createLapisOreDrops",
        count: 2,
    },
    RegistrationCounter {
        source_call: "createRedstoneOreDrops",
        count: 2,
    },
    RegistrationCounter {
        source_call: "createAttachedStemDrops",
        count: 2,
    },
    RegistrationCounter {
        source_call: "createStemDrops",
        count: 2,
    },
    RegistrationCounter {
        source_call: "createGrassDrops",
        count: 2,
    },
    RegistrationCounter {
        source_call: "createDoublePlantWithSeedDrops",
        count: 2,
    },
    RegistrationCounter {
        source_call: "createMushroomBlockDrop",
        count: 2,
    },
    RegistrationCounter {
        source_call: "createCaveVinesDrop",
        count: 2,
    },
];

fn count_occurrences(source: &str, needle: &str) -> usize {
    source.match_indices(needle).count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constants_and_constructor_match_java() {
        let constants = vanilla_block_loot_constants();
        assert_eq!(
            constants.jungle_leaves_sapling_chances,
            vec![0.025, 0.027777778, 0.03125, 0.041666668, 0.1]
        );
        assert_eq!(constants.explosion_resistant_blocks.len(), 27);
        for block in &constants.explosion_resistant_blocks {
            assert!(
                VANILLA_BLOCK_LOOT_JAVA.contains(&format!("Blocks.{block}")),
                "missing explosion-resistant block sentinel {block}"
            );
        }
        assert!(VANILLA_BLOCK_LOOT_JAVA.contains("map(ItemLike::asItem)"));
        assert!(VANILLA_BLOCK_LOOT_JAVA.contains(constants.enabled_feature_source));
        assert!(VANILLA_BLOCK_LOOT_JAVA
            .contains("super(EXPLOSION_RESISTANT, FeatureFlags.REGISTRY.allFlags(), registries)"));
    }

    #[test]
    fn generate_registration_shape_matches_java() {
        assert!(VANILLA_BLOCK_LOOT_JAVA.contains("protected void generate()"));
        assert!(VANILLA_BLOCK_LOOT_JAVA.contains("Registries.ENCHANTMENT"));
        assert!(VANILLA_BLOCK_LOOT_JAVA.contains("Registries.ITEM"));
        for counter in VANILLA_BLOCK_LOOT_REGISTRATION_COUNTERS {
            assert_eq!(
                count_occurrences(VANILLA_BLOCK_LOOT_JAVA, counter.source_call),
                counter.count,
                "registration call count changed for {}",
                counter.source_call
            );
        }
    }

    #[test]
    fn helper_family_usage_counts_match_java() {
        for counter in VANILLA_BLOCK_LOOT_HELPER_COUNTERS {
            assert_eq!(
                count_occurrences(VANILLA_BLOCK_LOOT_JAVA, counter.source_call),
                counter.count,
                "helper call count changed for {}",
                counter.source_call
            );
        }
    }

    #[test]
    fn modern_block_families_have_expected_drop_strategies() {
        let sentinels = [
            "this.dropSelf(Blocks.PALE_OAK_PLANKS)",
            "this.dropSelf(Blocks.PALE_OAK_LOG)",
            "this.dropSelf(Blocks.PALE_OAK_HANGING_SIGN)",
            "this.dropSelf(Blocks.RESIN_BLOCK)",
            "this.dropSelf(Blocks.RESIN_BRICK_WALL)",
            "this.dropSelf(Blocks.RESIN_BRICK_STAIRS)",
            "this.add(Blocks.PALE_MOSS_CARPET, x$0 -> this.createMossyCarpetBlockDrops(x$0))",
            "this.add(Blocks.PALE_HANGING_MOSS, x$0 -> this.createShearsOrSilkTouchOnlyDrop(x$0))",
            "this.add(Blocks.RESIN_CLUMP, x$0 -> this.createMultifaceBlockDrops(x$0))",
            "this.dropSelf(Blocks.CRAFTER)",
            "this.dropSelf(Blocks.HEAVY_CORE)",
            "this.dropSelf(Blocks.FIREFLY_BUSH)",
            "this.dropSelf(Blocks.CACTUS_FLOWER)",
            "Blocks.COPPER_BARS.forEach(x$0 -> this.dropSelf(x$0))",
            "Blocks.COPPER_CHAIN.forEach(x$0 -> this.dropSelf(x$0))",
            "Blocks.COPPER_LANTERN.forEach(block -> this.add(block, this::createSingleItemTable))",
            "this.add(Blocks.COPPER_GOLEM_STATUE, x$0 -> this.createCopperGolemStatueBlock(x$0))",
            "this.add(Blocks.WAXED_OXIDIZED_COPPER_GOLEM_STATUE, x$0 -> this.createCopperGolemStatueBlock(x$0))",
        ];
        for sentinel in sentinels {
            assert!(
                VANILLA_BLOCK_LOOT_JAVA.contains(sentinel),
                "missing modern block loot sentinel: {sentinel}"
            );
        }
    }

    #[test]
    fn special_stateful_tables_match_java_sentinels() {
        let sentinels = [
            "this.add(Blocks.DECORATED_POT, this::createDecoratedPotTable)",
            "DynamicLoot.dynamicEntry(DecoratedPotBlock.SHERDS_DYNAMIC_DROP_ID)",
            "hasProperty(DecoratedPotBlock.CRACKED, true)",
            "include(DataComponents.POT_DECORATIONS)",
            "Blocks.TNT",
            "hasProperty(TntBlock.UNSTABLE, false)",
            "hasProperty(CocoaBlock.AGE, 2)",
            "List.of(2, 3, 4)",
            "hasProperty(SeaPickleBlock.PICKLES, count.intValue())",
            "hasProperty(ComposterBlock.LEVEL, 8)",
            "this.add(Blocks.CAVE_VINES, x$0 -> this.createCaveVinesDrop(x$0))",
            "this.add(Blocks.CAVE_VINES_PLANT, x$0 -> this.createCaveVinesDrop(x$0))",
            "Blocks.SNOW",
            "SnowLayerBlock.LAYERS.getPossibleValues()",
            "? LootItem.lootTableItem(Blocks.SNOW_BLOCK)",
        ];
        for sentinel in sentinels {
            assert!(
                VANILLA_BLOCK_LOOT_JAVA.contains(sentinel),
                "missing stateful loot sentinel: {sentinel}"
            );
        }
    }

    #[test]
    fn crop_and_plant_tables_match_java_sentinels() {
        let sentinels = [
            "hasProperty(BeetrootBlock.AGE, 3)",
            "this.createCropDrops(Blocks.BEETROOTS, Items.BEETROOT, Items.BEETROOT_SEEDS, isBeetrootMaxAge)",
            "hasProperty(CropBlock.AGE, 7)",
            "this.createCropDrops(Blocks.WHEAT, Items.WHEAT, Items.WHEAT_SEEDS, isWheatMaxAge)",
            "hasProperty(CarrotBlock.AGE, 7)",
            "hasProperty(PotatoBlock.AGE, 7)",
            "LootItem.lootTableItem(Items.POISONOUS_POTATO).when(LootItemRandomChanceCondition.randomChance(0.02F))",
            "hasProperty(MangrovePropaguleBlock.AGE, 4)",
            "Blocks.TORCHFLOWER_CROP",
            "this.add(Blocks.PITCHER_CROP, block -> this.createPitcherCropLoot())",
            "PitcherCropBlock.AGE.getPossibleValues()",
            "age == 4",
            "LootItem.lootTableItem(Items.PITCHER_PLANT)",
            "LootItem.lootTableItem(Items.PITCHER_POD)",
            "hasProperty(SweetBerryBushBlock.AGE, 3)",
            "UniformGenerator.between(2.0F, 3.0F)",
            "hasProperty(SweetBerryBushBlock.AGE, 2)",
            "UniformGenerator.between(1.0F, 2.0F)",
            "this.add(Blocks.MELON_STEM, block -> this.createStemDrops(block, Items.MELON_SEEDS))",
            "this.add(Blocks.ATTACHED_PUMPKIN_STEM, block -> this.createAttachedStemDrops(block, Items.PUMPKIN_SEEDS))",
        ];
        for sentinel in sentinels {
            assert!(
                VANILLA_BLOCK_LOOT_JAVA.contains(sentinel),
                "missing crop or plant loot sentinel: {sentinel}"
            );
        }
    }

    #[test]
    fn ore_and_fortune_tables_match_java_sentinels() {
        let sentinels = [
            "this.add(Blocks.COAL_ORE, block -> this.createOreDrop(block, Items.COAL))",
            "this.add(Blocks.DEEPSLATE_DIAMOND_ORE, block -> this.createOreDrop(block, Items.DIAMOND))",
            "this.add(Blocks.COPPER_ORE, x$0 -> this.createCopperOreDrops(x$0))",
            "this.add(Blocks.DEEPSLATE_COPPER_ORE, x$0 -> this.createCopperOreDrops(x$0))",
            "this.add(Blocks.IRON_ORE, block -> this.createOreDrop(block, Items.RAW_IRON))",
            "this.add(Blocks.GOLD_ORE, block -> this.createOreDrop(block, Items.RAW_GOLD))",
            "LootItem.lootTableItem(Items.GOLD_NUGGET)",
            "UniformGenerator.between(2.0F, 6.0F)",
            "this.add(Blocks.LAPIS_ORE, x$0 -> this.createLapisOreDrops(x$0))",
            "this.add(Blocks.REDSTONE_ORE, x$0 -> this.createRedstoneOreDrops(x$0))",
            "LootItem.lootTableItem(Items.GLOWSTONE_DUST)",
            "LimitCount.limitCount(IntRange.range(1, 4))",
            "LootItem.lootTableItem(Items.PRISMARINE_CRYSTALS)",
            "LimitCount.limitCount(IntRange.range(1, 5))",
            "LootItem.lootTableItem(Items.RESIN_CLUMP)",
            "LimitCount.limitCount(IntRange.upperBound(9))",
            "MatchTool.toolMatches(ItemPredicate.Builder.item().of(items, ItemTags.CLUSTER_MAX_HARVESTABLES))",
        ];
        for sentinel in sentinels {
            assert!(
                VANILLA_BLOCK_LOOT_JAVA.contains(sentinel),
                "missing ore or fortune loot sentinel: {sentinel}"
            );
        }
    }

    #[test]
    fn component_and_block_entity_tables_match_java_sentinels() {
        let sentinels = [
            "this.add(Blocks.SHULKER_BOX, x$0 -> this.createShulkerBoxDrop(x$0))",
            "this.add(Blocks.YELLOW_SHULKER_BOX, x$0 -> this.createShulkerBoxDrop(x$0))",
            "this.add(Blocks.BLACK_BANNER, x$0 -> this.createBannerDrop(x$0))",
            "this.add(Blocks.YELLOW_BANNER, x$0 -> this.createBannerDrop(x$0))",
            "CopyComponentsFunction.copyComponentsFromBlockEntity(LootContextParams.BLOCK_ENTITY)",
            "include(DataComponents.PROFILE)",
            "include(DataComponents.NOTE_BLOCK_SOUND)",
            "include(DataComponents.CUSTOM_NAME)",
            "this.add(Blocks.SKELETON_SKULL, this::createMobSkullDrop)",
            "this.add(Blocks.DRAGON_HEAD, this::createMobSkullDrop)",
            "this.add(Blocks.BEE_NEST, x$0 -> this.createBeeNestDrop(x$0))",
            "this.add(Blocks.BEEHIVE, x$0 -> this.createBeeHiveDrop(x$0))",
            "this.add(Blocks.CHEST, x$0 -> this.createNameableBlockEntityTable(x$0))",
            "this.add(Blocks.WAXED_OXIDIZED_COPPER_CHEST, x$0 -> this.createNameableBlockEntityTable(x$0))",
        ];
        for sentinel in sentinels {
            assert!(
                VANILLA_BLOCK_LOOT_JAVA.contains(sentinel),
                "missing component/block-entity loot sentinel: {sentinel}"
            );
        }
    }

    #[test]
    fn foliage_shears_and_silk_touch_tables_match_java_sentinels() {
        let sentinels = [
            "this.add(Blocks.OAK_LEAVES, block -> this.createOakLeavesDrops(block, Blocks.OAK_SAPLING, NORMAL_LEAVES_SAPLING_CHANCES))",
            "this.add(Blocks.JUNGLE_LEAVES, block -> this.createLeavesDrops(block, Blocks.JUNGLE_SAPLING, JUNGLE_LEAVES_SAPLING_CHANGES))",
            "this.add(Blocks.MANGROVE_LEAVES, x$0 -> this.createMangroveLeavesDrops(x$0))",
            "Blocks.COBWEB",
            "this.createSilkTouchOrShearsDispatchTable(",
            "Blocks.DEAD_BUSH",
            "this.createShearsDispatchTable(",
            "this.add(Blocks.NETHER_SPROUTS, x$0 -> this.createShearsOnlyDrop(x$0))",
            "this.add(Blocks.GLOW_LICHEN, block -> this.createMultifaceBlockDrops(block, this.hasShears()))",
            "this.add(Blocks.TALL_SEAGRASS, this.createDoublePlantShearsDrop(Blocks.SEAGRASS))",
            "this.dropWhenSilkTouch(Blocks.SCULK_SENSOR)",
            "this.dropWhenSilkTouch(Blocks.CHISELED_BOOKSHELF)",
            "this.dropWhenSilkTouch(Blocks.GLASS)",
            "this.dropWhenSilkTouch(Blocks.BLACK_STAINED_GLASS_PANE)",
            "this.dropWhenSilkTouch(Blocks.HORN_CORAL_FAN)",
            "this.otherWhenSilkTouch(Blocks.INFESTED_DEEPSLATE, Blocks.DEEPSLATE)",
            "this.addNetherVinesDropTable(Blocks.WEEPING_VINES, Blocks.WEEPING_VINES_PLANT)",
            "this.addNetherVinesDropTable(Blocks.TWISTING_VINES, Blocks.TWISTING_VINES_PLANT)",
        ];
        for sentinel in sentinels {
            assert!(
                VANILLA_BLOCK_LOOT_JAVA.contains(sentinel),
                "missing foliage/shears/silk-touch sentinel: {sentinel}"
            );
        }
    }

    #[test]
    fn no_drop_and_transformed_drop_tables_match_java_sentinels() {
        let sentinels = [
            "this.add(Blocks.SUSPICIOUS_SAND, noDrop())",
            "this.add(Blocks.SUSPICIOUS_GRAVEL, noDrop())",
            "this.add(Blocks.CAKE, noDrop())",
            "this.add(Blocks.SPAWNER, noDrop())",
            "this.add(Blocks.TRIAL_SPAWNER, noDrop())",
            "this.add(Blocks.VAULT, noDrop())",
            "this.add(Blocks.REINFORCED_DEEPSLATE, noDrop())",
            "this.dropOther(Blocks.FARMLAND, Blocks.DIRT)",
            "this.dropOther(Blocks.TRIPWIRE, Items.STRING)",
            "this.dropOther(Blocks.POWDER_SNOW_CAULDRON, Blocks.CAULDRON)",
            "this.dropOther(Blocks.BIG_DRIPLEAF_STEM, Blocks.BIG_DRIPLEAF)",
            "this.add(Blocks.STONE, block -> this.createSingleItemTableWithSilkTouch(block, Blocks.COBBLESTONE))",
            "this.add(Blocks.ENDER_CHEST, block -> this.createSingleItemTableWithSilkTouch(block, Blocks.OBSIDIAN, ConstantValue.exactly(8.0F)))",
        ];
        for sentinel in sentinels {
            assert!(
                VANILLA_BLOCK_LOOT_JAVA.contains(sentinel),
                "missing no-drop or transformed-drop sentinel: {sentinel}"
            );
        }
    }
}
