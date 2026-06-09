const VANILLA_BLOCK_TAGS_PROVIDER_JAVA: &str = include_str!(
    "../../decompiled-server-26.1.2/net/minecraft/data/tags/VanillaBlockTagsProvider.java"
);

const VANILLA_BLOCK_TAG_SENTINELS: &[&str] = &[
    "public class VanillaBlockTagsProvider extends IntrinsicHolderTagsProvider<Block>",
    "super(output, Registries.BLOCK, lookupProvider, e -> e.builtInRegistryHolder().key());",
    "(new BlockItemTagsProvider() {",
    "protected TagAppender<Block, Block> tag(final TagKey<Block> blockTag, final TagKey<Item> itemTag)",
    "return VanillaBlockTagsProvider.this.tag(blockTag);",
    "this.tag(BlockTags.MOB_INTERACTABLE_DOORS)",
    "Blocks.WAXED_OXIDIZED_COPPER_DOOR",
    "this.tag(BlockTags.OVERWORLD_NATURAL_LOGS)",
    "Blocks.PALE_OAK_LOG",
    "this.tag(BlockTags.ENDERMAN_HOLDABLE)",
    "Blocks.CACTUS_FLOWER",
    "Blocks.POTTED_OPEN_EYEBLOSSOM",
    "Blocks.POTTED_GOLDEN_DANDELION",
    "this.tag(BlockTags.DRAGON_IMMUNE)",
    "Blocks.TEST_INSTANCE_BLOCK",
    "this.tag(BlockTags.GUARDED_BY_PIGLINS)",
    ".addTag(BlockTags.COPPER_CHESTS)",
    "this.tag(BlockTags.EDIBLE_FOR_SHEEP).add(Blocks.SHORT_GRASS).add(Blocks.SHORT_DRY_GRASS).add(Blocks.TALL_DRY_GRASS).add(Blocks.FERN);",
    "this.tag(BlockTags.OVERWORLD_CARVER_REPLACEABLES)",
    ".addTag(BlockTags.SNOW)",
    "this.tag(BlockTags.INSIDE_STEP_SOUND_BLOCKS)",
    "Blocks.LEAF_LITTER",
    "this.tag(BlockTags.COMBINATION_STEP_SOUND_BLOCKS)",
    "Blocks.RESIN_CLUMP",
    "this.tag(BlockTags.MINEABLE_WITH_AXE)",
    "Blocks.CREAKING_HEART",
    "this.tag(BlockTags.MINEABLE_WITH_PICKAXE)",
    "Blocks.RESIN_BRICKS",
    "Blocks.CHISELED_RESIN_BRICKS",
    "this.tag(BlockTags.NEEDS_STONE_TOOL)",
    "Blocks.WAXED_OXIDIZED_COPPER_TRAPDOOR",
    "this.tag(BlockTags.INCORRECT_FOR_NETHERITE_TOOL);",
    "this.tag(BlockTags.INCORRECT_FOR_DIAMOND_TOOL);",
    "this.tag(BlockTags.INCORRECT_FOR_COPPER_TOOL).addTag(BlockTags.NEEDS_DIAMOND_TOOL).addTag(BlockTags.NEEDS_IRON_TOOL);",
    "this.tag(BlockTags.ARMADILLO_SPAWNABLE_ON)",
    "this.tag(BlockTags.REPLACEABLE)\n         .addAll(registries.lookupOrThrow(Registries.BLOCK).listElements().map(Holder.Reference::value).filter(b -> b.defaultBlockState().canBeReplaced()));",
    "this.tag(BlockTags.ENCHANTMENT_POWER_TRANSMITTER).addTag(BlockTags.REPLACEABLE);",
    "this.tag(BlockTags.HAPPY_GHAST_AVOIDS)",
    "this.tag(BlockTags.TRIGGERS_AMBIENT_DESERT_DRY_VEGETATION_BLOCK_SOUNDS).addTag(BlockTags.TERRACOTTA).add(Blocks.SAND, Blocks.RED_SAND);",
    "this.tag(BlockTags.TRIGGERS_AMBIENT_DRIED_GHAST_BLOCK_SOUNDS).add(Blocks.SOUL_SAND, Blocks.SOUL_SOIL);",
    "this.tag(BlockTags.AIR).add(Blocks.AIR, Blocks.VOID_AIR, Blocks.CAVE_AIR);",
];

const BLOCK_TAG_REFERENCE_COUNTS: &[(&str, usize)] = &[
    ("BlockTags.SUBSTRATE_OVERWORLD", 15),
    ("BlockTags.BASE_STONE_OVERWORLD", 9),
    ("BlockTags.SUPPORTS_VEGETATION", 9),
    ("BlockTags.SAND", 8),
    ("BlockTags.NYLIUM", 6),
    ("BlockTags.NEEDS_DIAMOND_TOOL", 6),
    ("BlockTags.NEEDS_IRON_TOOL", 5),
    ("BlockTags.SIGNS", 4),
    ("BlockTags.OVERWORLD_NATURAL_LOGS", 2),
    ("BlockTags.REPLACEABLE", 2),
    ("BlockTags.AIR", 1),
];

fn count_occurrences(source: &str, needle: &str) -> usize {
    source.match_indices(needle).count()
}

fn count_java_identifier_occurrences(source: &str, needle: &str) -> usize {
    source
        .match_indices(needle)
        .filter(|(index, _)| {
            let next_index = index + needle.len();
            !source[next_index..]
                .chars()
                .next()
                .is_some_and(|ch| ch.is_ascii_alphanumeric() || ch == '_')
        })
        .count()
}

fn assert_source_contains_all(source: &str, sentinels: &[&str]) {
    for sentinel in sentinels {
        assert!(
            source.contains(sentinel),
            "missing Java sentinel: {sentinel}"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vanilla_block_tags_provider_contract_matches_java() {
        assert_source_contains_all(
            VANILLA_BLOCK_TAGS_PROVIDER_JAVA,
            VANILLA_BLOCK_TAG_SENTINELS,
        );
        assert_eq!(
            VANILLA_BLOCK_TAGS_PROVIDER_JAVA.lines().count(),
            1310,
            "VanillaBlockTagsProvider.java line-count drift"
        );
        assert_eq!(
            count_occurrences(VANILLA_BLOCK_TAGS_PROVIDER_JAVA, "this.tag("),
            176,
            "vanilla block tag builder count drift"
        );
        assert_eq!(
            count_occurrences(VANILLA_BLOCK_TAGS_PROVIDER_JAVA, ".add("),
            248,
            "vanilla block direct-add count drift"
        );
        assert_eq!(
            count_occurrences(VANILLA_BLOCK_TAGS_PROVIDER_JAVA, ".addTag("),
            175,
            "vanilla block tag reference count drift"
        );
        assert_eq!(
            count_occurrences(VANILLA_BLOCK_TAGS_PROVIDER_JAVA, ".addAll("),
            1,
            "vanilla block addAll count drift"
        );
        assert_eq!(
            count_occurrences(VANILLA_BLOCK_TAGS_PROVIDER_JAVA, "BlockTags."),
            350,
            "vanilla block tag reference token count drift"
        );
        assert_eq!(
            count_occurrences(VANILLA_BLOCK_TAGS_PROVIDER_JAVA, "Blocks."),
            1209,
            "vanilla block reference count drift"
        );
    }

    #[test]
    fn vanilla_block_tags_provider_shape_matches_java() {
        assert_eq!(
            count_occurrences(
                VANILLA_BLOCK_TAGS_PROVIDER_JAVA,
                "new BlockItemTagsProvider()"
            ),
            1,
            "BlockItemTagsProvider delegation count drift"
        );
        assert_eq!(
            count_occurrences(
                VANILLA_BLOCK_TAGS_PROVIDER_JAVA,
                "TagAppender<Block, Block>"
            ),
            1,
            "block tag-appender bridge count drift"
        );
        assert_eq!(
            count_occurrences(
                VANILLA_BLOCK_TAGS_PROVIDER_JAVA,
                "this.tag(BlockTags.INCORRECT_FOR_"
            ),
            7,
            "incorrect-tool tag count drift"
        );
        assert_eq!(
            count_occurrences(
                VANILLA_BLOCK_TAGS_PROVIDER_JAVA,
                "this.tag(BlockTags.SUPPORTS_FROGSPAWN);"
            ),
            1,
            "empty frogspawn support tag drift"
        );

        for (tag, expected_count) in BLOCK_TAG_REFERENCE_COUNTS {
            assert_eq!(
                count_java_identifier_occurrences(VANILLA_BLOCK_TAGS_PROVIDER_JAVA, tag),
                *expected_count,
                "reference-count drift for {tag}"
            );
        }
    }
}
