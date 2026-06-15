const VANILLA_ITEM_TAGS_PROVIDER_JAVA: &str = vibecraft_java_source!("/net/minecraft/data/tags/VanillaItemTagsProvider.java");

const VANILLA_ITEM_TAG_SENTINELS: &[&str] = &[
    "public class VanillaItemTagsProvider extends IntrinsicHolderTagsProvider<Item>",
    "super(output, Registries.ITEM, lookupProvider, e -> e.builtInRegistryHolder().key());",
    "(new BlockItemTagsProvider() {",
    "return new VanillaItemTagsProvider.BlockToItemConverter(VanillaItemTagsProvider.this.tag(itemTag));",
    "this.tag(ItemTags.BOATS)",
    "Items.PALE_OAK_BOAT",
    "Items.BAMBOO_CHEST_RAFT",
    "this.tag(ItemTags.BUNDLES)",
    "Items.WHITE_BUNDLE",
    "this.tag(ItemTags.EGGS).add(Items.EGG, Items.BLUE_EGG, Items.BROWN_EGG);",
    "this.tag(ItemTags.PIGLIN_LOVED)",
    "Items.GOLDEN_NAUTILUS_ARMOR",
    "Items.GOLDEN_DANDELION",
    "this.tag(ItemTags.COPPER_TOOL_MATERIALS).add(Items.COPPER_INGOT);",
    "this.tag(ItemTags.REPAIRS_COPPER_ARMOR).add(Items.COPPER_INGOT);",
    "Items.COPPER_PICKAXE",
    "this.tag(ItemTags.SPEARS)\n         .add(Items.DIAMOND_SPEAR, Items.STONE_SPEAR, Items.GOLDEN_SPEAR, Items.NETHERITE_SPEAR, Items.WOODEN_SPEAR, Items.IRON_SPEAR, Items.COPPER_SPEAR);",
    "this.tag(ItemTags.BREAKS_DECORATED_POTS)",
    "Items.GUSTER_POTTERY_SHERD",
    "this.tag(ItemTags.TRIM_MATERIALS)",
    "Items.RESIN_BRICK",
    "this.tag(ItemTags.LUNGE_ENCHANTABLE).addTag(ItemTags.SPEARS);",
    "this.tag(ItemTags.HARNESSES)",
    "Items.BLACK_HARNESS",
    "this.tag(ItemTags.HAPPY_GHAST_TEMPT_ITEMS).addTag(ItemTags.HAPPY_GHAST_FOOD).addTag(ItemTags.HARNESSES);",
    "this.tag(ItemTags.CAMEL_HUSK_FOOD).add(Items.RABBIT_FOOT);",
    "this.tag(ItemTags.NAUTILUS_TAMING_ITEMS).add(Items.PUFFERFISH_BUCKET, Items.PUFFERFISH);",
    "this.tag(ItemTags.NAUTILUS_FOOD).addTag(ItemTags.FISHES).addTag(ItemTags.NAUTILUS_BUCKET_FOOD);",
    "this.tag(ItemTags.SHEARABLE_FROM_COPPER_GOLEM).add(Items.POPPY);",
    "this.tag(ItemTags.METAL_NUGGETS).add(Items.COPPER_NUGGET, Items.IRON_NUGGET, Items.GOLD_NUGGET);",
    "this.tag(ItemTags.LOOM_PATTERNS)",
    "Items.BORDURE_INDENTED_BANNER_PATTERN",
    "private static class BlockToItemConverter implements TagAppender<Block, Block>",
    "private final TagAppender<Item, Item> itemAppender;",
    "this.itemAppender.add(Objects.requireNonNull(element.asItem()));",
    "this.itemAppender.addOptional(Objects.requireNonNull(element.asItem()));",
    "return TagKey.create(Registries.ITEM, blockTag.location());",
    "this.itemAppender.addTag(blockTagToItemTag(tag));",
    "this.itemAppender.addOptionalTag(blockTagToItemTag(tag));",
];

const ITEM_TAG_REFERENCE_COUNTS: &[(&str, usize)] = &[
    ("ItemTags.AXES", 6),
    ("ItemTags.SWORDS", 5),
    ("ItemTags.PICKAXES", 5),
    ("ItemTags.SHOVELS", 5),
    ("ItemTags.HOES", 5),
    ("ItemTags.FOOT_ARMOR", 5),
    ("ItemTags.LEG_ARMOR", 5),
    ("ItemTags.CHEST_ARMOR", 5),
    ("ItemTags.HEAD_ARMOR", 5),
    ("ItemTags.SPEARS", 4),
    ("ItemTags.DYES", 4),
    ("ItemTags.HARNESSES", 2),
    ("ItemTags.NAUTILUS_BUCKET_FOOD", 2),
    ("ItemTags.SHEARABLE_FROM_COPPER_GOLEM", 1),
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
    fn vanilla_item_tags_provider_contract_matches_java() {
        assert_source_contains_all(VANILLA_ITEM_TAGS_PROVIDER_JAVA, VANILLA_ITEM_TAG_SENTINELS);
        assert_eq!(
            VANILLA_ITEM_TAGS_PROVIDER_JAVA.lines().count(),
            556,
            "VanillaItemTagsProvider.java line-count drift"
        );
        assert_eq!(
            count_occurrences(VANILLA_ITEM_TAGS_PROVIDER_JAVA, "this.tag("),
            135,
            "vanilla item tag builder count drift"
        );
        assert_eq!(
            count_occurrences(VANILLA_ITEM_TAGS_PROVIDER_JAVA, ".add("),
            166,
            "vanilla item direct-add count drift"
        );
        assert_eq!(
            count_occurrences(VANILLA_ITEM_TAGS_PROVIDER_JAVA, ".addTag("),
            66,
            "vanilla item tag reference count drift"
        );
        assert_eq!(
            count_occurrences(VANILLA_ITEM_TAGS_PROVIDER_JAVA, ".addOptional("),
            1,
            "vanilla item optional element forwarding count drift"
        );
        assert_eq!(
            count_occurrences(VANILLA_ITEM_TAGS_PROVIDER_JAVA, ".addOptionalTag("),
            1,
            "vanilla item optional tag forwarding count drift"
        );
        assert_eq!(
            count_occurrences(VANILLA_ITEM_TAGS_PROVIDER_JAVA, "ItemTags."),
            199,
            "item tag reference token count drift"
        );
        assert_eq!(
            count_occurrences(VANILLA_ITEM_TAGS_PROVIDER_JAVA, "Items."),
            495,
            "item reference count drift"
        );
    }

    #[test]
    fn vanilla_item_block_bridge_and_reference_shape_match_java() {
        assert_eq!(
            count_occurrences(
                VANILLA_ITEM_TAGS_PROVIDER_JAVA,
                "new BlockItemTagsProvider()"
            ),
            1,
            "BlockItemTagsProvider delegation count drift"
        );
        assert_eq!(
            count_occurrences(VANILLA_ITEM_TAGS_PROVIDER_JAVA, "BlockToItemConverter"),
            3,
            "BlockToItemConverter reference count drift"
        );
        assert_eq!(
            count_occurrences(VANILLA_ITEM_TAGS_PROVIDER_JAVA, "TagAppender<Block, Block>"),
            6,
            "block tag-appender bridge count drift"
        );
        assert_eq!(
            count_occurrences(VANILLA_ITEM_TAGS_PROVIDER_JAVA, "TagAppender<Item, Item>"),
            2,
            "item tag-appender storage count drift"
        );
        assert_eq!(
            count_occurrences(VANILLA_ITEM_TAGS_PROVIDER_JAVA, "Objects.requireNonNull"),
            2,
            "block-to-item null-check count drift"
        );

        for (tag, expected_count) in ITEM_TAG_REFERENCE_COUNTS {
            assert_eq!(
                count_java_identifier_occurrences(VANILLA_ITEM_TAGS_PROVIDER_JAVA, tag),
                *expected_count,
                "reference-count drift for {tag}"
            );
        }
    }
}
