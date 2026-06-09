const VILLAGER_TRADES_TAGS_PROVIDER_JAVA: &str = include_str!(
    "../../decompiled-server-26.1.2/net/minecraft/data/tags/VillagerTradesTagsProvider.java"
);

const VILLAGER_TRADE_TAG_SENTINELS: &[&str] = &[
    "public class VillagerTradesTagsProvider extends KeyTagProvider<VillagerTrade>",
    "super(output, Registries.VILLAGER_TRADE, lookupProvider);",
    "this.tag(VillagerTradeTags.FARMER_LEVEL_1)",
    "VillagerTrades.FARMER_5_EMERALD_GLISTENING_MELON_SLICE",
    "this.tag(VillagerTradeTags.FISHERMAN_LEVEL_5)",
    "VillagerTrades.FISHERMAN_5_DARK_OAK_BOAT_EMERALD",
    "this.tag(VillagerTradeTags.SHEPHERD_LEVEL_2)",
    "VillagerTrades.SHEPHERD_2_EMERALD_BLACK_CARPET",
    "this.tag(VillagerTradeTags.CARTOGRAPHER_LEVEL_2)",
    "VillagerTrades.CARTOGRAPHER_2_EMERALD_AND_COMPASS_EXPLORER_JUNGLE_MAP",
    "VillagerTrades.CARTOGRAPHER_3_EMERALD_AND_COMPASS_TRIAL_CHAMBER_MAP",
    "this.tag(VillagerTradeTags.COMMON_SMITH_LEVEL_3);",
    "this.tag(VillagerTradeTags.COMMON_SMITH_LEVEL_4);",
    "this.tag(VillagerTradeTags.COMMON_SMITH_LEVEL_5);",
    "this.tag(VillagerTradeTags.ARMORER_LEVEL_1)\n         .addTag(VillagerTradeTags.COMMON_SMITH_LEVEL_1)",
    "this.tag(VillagerTradeTags.WEAPONSMITH_LEVEL_5)\n         .addTag(VillagerTradeTags.COMMON_SMITH_LEVEL_5)",
    "this.tag(VillagerTradeTags.TOOLSMITH_LEVEL_5).addTag(VillagerTradeTags.COMMON_SMITH_LEVEL_5)",
    "VillagerTrades.MASON_3_EMERALD_POLISHED_GRANTITE",
    "VillagerTrades.MASON_4_EMERALD_BROWN_GLAZED_TERRACOTTA",
    "this.tag(VillagerTradeTags.WANDERING_TRADER_BUYING)",
    "this.tag(VillagerTradeTags.WANDERING_TRADER_UNCOMMON)",
    "VillagerTrades.WANDERING_TRADER_EMERALD_PALE_OAK_LOG",
    "this.tag(VillagerTradeTags.WANDERING_TRADER_COMMON)",
    "VillagerTrades.WANDERING_TRADER_EMERALD_OPEN_EYEBLOSSOM",
    "VillagerTrades.WANDERING_TRADER_EMERALD_PALE_OAK_SAPLING",
    "VillagerTrades.WANDERING_TRADER_EMERALD_WILDFLOWERS",
    "VillagerTrades.WANDERING_TRADER_EMERALD_FIREFLY_BUSH",
    "VillagerTrades.WANDERING_TRADER_EMERALD_GOLDEN_DANDELION",
];

const VILLAGER_TRADE_TAG_REFERENCE_COUNTS: &[(&str, usize)] = &[
    ("VillagerTradeTags.COMMON_SMITH_LEVEL_1", 4),
    ("VillagerTradeTags.COMMON_SMITH_LEVEL_2", 4),
    ("VillagerTradeTags.COMMON_SMITH_LEVEL_3", 4),
    ("VillagerTradeTags.COMMON_SMITH_LEVEL_4", 4),
    ("VillagerTradeTags.COMMON_SMITH_LEVEL_5", 4),
    ("VillagerTradeTags.FARMER_LEVEL_1", 1),
    ("VillagerTradeTags.CARTOGRAPHER_LEVEL_2", 1),
    ("VillagerTradeTags.WANDERING_TRADER_BUYING", 1),
    ("VillagerTradeTags.WANDERING_TRADER_UNCOMMON", 1),
    ("VillagerTradeTags.WANDERING_TRADER_COMMON", 1),
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
    fn villager_trade_tags_provider_contract_matches_java() {
        assert_source_contains_all(
            VILLAGER_TRADES_TAGS_PROVIDER_JAVA,
            VILLAGER_TRADE_TAG_SENTINELS,
        );
        assert_eq!(
            VILLAGER_TRADES_TAGS_PROVIDER_JAVA.lines().count(),
            470,
            "VillagerTradesTagsProvider.java line-count drift"
        );
        assert_eq!(
            count_occurrences(VILLAGER_TRADES_TAGS_PROVIDER_JAVA, "this.tag("),
            73,
            "villager trade tag builder count drift"
        );
        assert_eq!(
            count_occurrences(VILLAGER_TRADES_TAGS_PROVIDER_JAVA, ".add("),
            68,
            "villager trade direct-add count drift"
        );
        assert_eq!(
            count_occurrences(VILLAGER_TRADES_TAGS_PROVIDER_JAVA, ".addTag("),
            15,
            "villager trade tag reference count drift"
        );
        assert_eq!(
            count_occurrences(VILLAGER_TRADES_TAGS_PROVIDER_JAVA, "VillagerTradeTags."),
            88,
            "villager trade tag reference token count drift"
        );
        assert_eq!(
            count_occurrences(VILLAGER_TRADES_TAGS_PROVIDER_JAVA, "VillagerTrades."),
            387,
            "villager trade key reference count drift"
        );
    }

    #[test]
    fn villager_trade_tag_reference_shape_matches_java() {
        assert_eq!(
            count_occurrences(VILLAGER_TRADES_TAGS_PROVIDER_JAVA, "COMMON_SMITH_LEVEL_3);"),
            1,
            "empty common smith level 3 tag drift"
        );
        assert_eq!(
            count_occurrences(VILLAGER_TRADES_TAGS_PROVIDER_JAVA, "COMMON_SMITH_LEVEL_4);"),
            1,
            "empty common smith level 4 tag drift"
        );
        assert_eq!(
            count_occurrences(VILLAGER_TRADES_TAGS_PROVIDER_JAVA, "COMMON_SMITH_LEVEL_5);"),
            1,
            "empty common smith level 5 tag drift"
        );

        for (tag, expected_count) in VILLAGER_TRADE_TAG_REFERENCE_COUNTS {
            assert_eq!(
                count_java_identifier_occurrences(VILLAGER_TRADES_TAGS_PROVIDER_JAVA, tag),
                *expected_count,
                "reference-count drift for {tag}"
            );
        }
    }
}
