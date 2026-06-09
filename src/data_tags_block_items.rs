const BLOCK_ITEM_TAGS_PROVIDER_JAVA: &str = include_str!(
    "../../decompiled-server-26.1.2/net/minecraft/data/tags/BlockItemTagsProvider.java"
);

const BLOCK_ITEM_TAG_SENTINELS: &[&str] = &[
    "public abstract class BlockItemTagsProvider",
    "protected void run()",
    "Block[] smallFlowersInteractingWithBees = new Block[]{",
    "Blocks.OPEN_EYEBLOSSOM",
    "Blocks.TORCHFLOWER",
    "Block[] otherFlowers = new Block[]{",
    "Blocks.WILDFLOWERS",
    "Blocks.CACTUS_FLOWER",
    "this.tag(BlockTags.WOOL, ItemTags.WOOL)",
    "Blocks.PALE_OAK_PLANKS",
    "this.tag(BlockTags.BUTTONS, ItemTags.BUTTONS).addTag(BlockTags.WOODEN_BUTTONS).addTag(BlockTags.STONE_BUTTONS);",
    "Blocks.WAXED_OXIDIZED_COPPER_DOOR",
    "this.tag(BlockTags.PALE_OAK_LOGS, ItemTags.PALE_OAK_LOGS)",
    "this.tag(BlockTags.LOGS_THAT_BURN, ItemTags.LOGS_THAT_BURN)",
    ".addTag(BlockTags.CHERRY_LOGS);",
    "this.tag(BlockTags.SLABS, ItemTags.SLABS)",
    "Blocks.RESIN_BRICK_SLAB",
    "this.tag(BlockTags.WALLS, ItemTags.WALLS)",
    "Blocks.RESIN_BRICK_WALL",
    "this.tag(BlockTags.SMALL_FLOWERS, ItemTags.SMALL_FLOWERS).add(smallFlowersInteractingWithBees).add(Blocks.CLOSED_EYEBLOSSOM, Blocks.GOLDEN_DANDELION);",
    "this.tag(BlockTags.FLOWERS, ItemTags.FLOWERS).addTag(BlockTags.SMALL_FLOWERS).add(otherFlowers);",
    "this.tag(BlockTags.COPPER_CHESTS, ItemTags.COPPER_CHESTS)",
    "Blocks.WAXED_OXIDIZED_COPPER_CHEST",
    "this.tag(BlockTags.COPPER_GOLEM_STATUES, ItemTags.COPPER_GOLEM_STATUES)",
    "Blocks.WAXED_OXIDIZED_COPPER_GOLEM_STATUE",
    "this.tag(BlockTags.CHAINS, ItemTags.CHAINS).add(Blocks.IRON_CHAIN).addAll(Blocks.COPPER_CHAIN.asList());",
    "this.tag(BlockTags.WOODEN_SHELVES, ItemTags.WOODEN_SHELVES)",
    "Blocks.PALE_OAK_SHELF",
    "this.tag(BlockTags.LANTERNS, ItemTags.LANTERNS).add(Blocks.LANTERN, Blocks.SOUL_LANTERN).addAll(Blocks.COPPER_LANTERN.asList());",
    "this.tag(BlockTags.BARS, ItemTags.BARS).add(Blocks.IRON_BARS).addAll(Blocks.COPPER_BARS.asList());",
    "this.tag(BlockTags.BEE_ATTRACTIVE, ItemTags.BEE_FOOD).add(smallFlowersInteractingWithBees).add(otherFlowers);",
    "protected abstract TagAppender<Block, Block> tag(TagKey<Block> blockTag, TagKey<Item> itemTag);",
];

const BLOCK_TAG_REFERENCE_COUNTS: &[(&str, usize)] = &[
    ("BlockTags.WOOL", 2),
    ("BlockTags.LOGS", 2),
    ("BlockTags.COPPER", 1),
    ("BlockTags.LOGS_THAT_BURN", 2),
    ("BlockTags.SMALL_FLOWERS", 2),
    ("BlockTags.WOODEN_BUTTONS", 2),
    ("BlockTags.PALE_OAK_LOGS", 2),
    ("BlockTags.BEE_ATTRACTIVE", 1),
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
    fn block_item_tags_provider_contract_matches_java() {
        assert_source_contains_all(BLOCK_ITEM_TAGS_PROVIDER_JAVA, BLOCK_ITEM_TAG_SENTINELS);
        assert_eq!(
            BLOCK_ITEM_TAGS_PROVIDER_JAVA.lines().count(),
            630,
            "BlockItemTagsProvider.java line-count drift"
        );
        assert_eq!(
            count_occurrences(BLOCK_ITEM_TAGS_PROVIDER_JAVA, "this.tag("),
            73,
            "paired block/item tag builder count drift"
        );
        assert_eq!(
            count_occurrences(BLOCK_ITEM_TAGS_PROVIDER_JAVA, ".add("),
            100,
            "block item direct-add count drift"
        );
        assert_eq!(
            count_occurrences(BLOCK_ITEM_TAGS_PROVIDER_JAVA, ".addTag("),
            25,
            "block item tag reference count drift"
        );
        assert_eq!(
            count_occurrences(BLOCK_ITEM_TAGS_PROVIDER_JAVA, ".addAll("),
            3,
            "block item addAll count drift"
        );
        assert_eq!(
            count_occurrences(BLOCK_ITEM_TAGS_PROVIDER_JAVA, "BlockTags."),
            98,
            "block tag reference count drift"
        );
        assert_eq!(
            count_occurrences(BLOCK_ITEM_TAGS_PROVIDER_JAVA, "ItemTags."),
            73,
            "item tag reference count drift"
        );
        assert_eq!(
            count_occurrences(BLOCK_ITEM_TAGS_PROVIDER_JAVA, "Blocks."),
            571,
            "block reference count drift"
        );
    }

    #[test]
    fn block_item_tag_reference_shape_matches_java() {
        assert_eq!(
            count_occurrences(BLOCK_ITEM_TAGS_PROVIDER_JAVA, "new Block[]{"),
            2,
            "flower array count drift"
        );
        assert_eq!(
            count_occurrences(
                BLOCK_ITEM_TAGS_PROVIDER_JAVA,
                "smallFlowersInteractingWithBees"
            ),
            3,
            "small flower reuse count drift"
        );
        assert_eq!(
            count_occurrences(BLOCK_ITEM_TAGS_PROVIDER_JAVA, "otherFlowers"),
            3,
            "other flower reuse count drift"
        );

        for (tag, expected_count) in BLOCK_TAG_REFERENCE_COUNTS {
            assert_eq!(
                count_java_identifier_occurrences(BLOCK_ITEM_TAGS_PROVIDER_JAVA, tag),
                *expected_count,
                "reference-count drift for {tag}"
            );
        }
    }
}
