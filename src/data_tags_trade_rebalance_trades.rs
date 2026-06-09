const TRADE_REBALANCE_TRADE_TAGS_PROVIDER_JAVA: &str = include_str!(
    "../../decompiled-server-26.1.2/net/minecraft/data/tags/TradeRebalanceTradeTagsProvider.java"
);

const LEVEL_TAGS: &[&str] = &[
    "VillagerTradeTags.LIBRARIAN_LEVEL_1",
    "VillagerTradeTags.LIBRARIAN_LEVEL_2",
    "VillagerTradeTags.LIBRARIAN_LEVEL_3",
    "VillagerTradeTags.LIBRARIAN_LEVEL_4",
    "VillagerTradeTags.LIBRARIAN_LEVEL_5",
    "VillagerTradeTags.ARMORER_LEVEL_1",
    "VillagerTradeTags.ARMORER_LEVEL_2",
    "VillagerTradeTags.ARMORER_LEVEL_3",
    "VillagerTradeTags.ARMORER_LEVEL_4",
    "VillagerTradeTags.ARMORER_LEVEL_5",
];

const SOURCE_SENTINELS: &[&str] = &[
    "public class TradeRebalanceTradeTagsProvider extends KeyTagProvider<VillagerTrade>",
    "super(output, Registries.VILLAGER_TRADE, lookupProvider);",
    "this.tag(VillagerTradeTags.LIBRARIAN_LEVEL_1, true)",
    "VillagerTrades.LIBRARIAN_1_PAPER_EMERALD",
    "TradeRebalanceVillagerTrades.LIBRARIAN_1_EMERALD_AND_BOOK_DESERT_ENCHANTED_BOOK",
    "TradeRebalanceVillagerTrades.LIBRARIAN_3_EMERALD_AND_BOOK_TAIGA_ENCHANTED_BOOK",
    "this.tag(VillagerTradeTags.LIBRARIAN_LEVEL_4, true)\n         .add(VillagerTrades.LIBRARIAN_4_WRITABLE_BOOK_EMERALD, VillagerTrades.LIBRARIAN_4_EMERALD_CLOCK, VillagerTrades.LIBRARIAN_4_EMERALD_COMPASS);",
    "TradeRebalanceVillagerTrades.LIBRARIAN_5_EMERALD_AND_BOOK_SWAMP_ENCHANTED_BOOK",
    "this.tag(VillagerTradeTags.ARMORER_LEVEL_1, true)",
    "VillagerTrades.COMMON_SMITH_1_COAL_EMERALD",
    "TradeRebalanceVillagerTrades.ARMORER_2_EMERALD_CHAINMAIL_LEGGINGS_GROUP_2",
    "TradeRebalanceVillagerTrades.ARMORER_4_EMERALD_ENCHANTED_CHAINMAIL_CHESTPLATE_SWAMP",
    "TradeRebalanceVillagerTrades.ARMORER_4_EMERALD_AND_DIAMOND_CHESTPLATE_DIAMOND_HELMET_TAIGA",
    "TradeRebalanceVillagerTrades.ARMORER_5_EMERALD_CHAINMAIL_BOOTS_SWAMP",
    "TradeRebalanceVillagerTrades.ARMORER_5_IRON_BLOCK_EMERALD_NON_TAIGA",
];

const TRADE_REBALANCE_PREFIXES: &[&str] = &[
    "TradeRebalanceVillagerTrades.LIBRARIAN_1_",
    "TradeRebalanceVillagerTrades.LIBRARIAN_2_",
    "TradeRebalanceVillagerTrades.LIBRARIAN_3_",
    "TradeRebalanceVillagerTrades.LIBRARIAN_5_",
    "TradeRebalanceVillagerTrades.ARMORER_1_",
    "TradeRebalanceVillagerTrades.ARMORER_2_",
    "TradeRebalanceVillagerTrades.ARMORER_3_",
    "TradeRebalanceVillagerTrades.ARMORER_4_",
    "TradeRebalanceVillagerTrades.ARMORER_5_",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct LevelShape {
    tag: &'static str,
    vanilla_entries: usize,
    rebalance_entries: usize,
}

const LEVEL_SHAPES: &[LevelShape] = &[
    LevelShape {
        tag: "VillagerTradeTags.LIBRARIAN_LEVEL_1",
        vanilla_entries: 2,
        rebalance_entries: 7,
    },
    LevelShape {
        tag: "VillagerTradeTags.LIBRARIAN_LEVEL_2",
        vanilla_entries: 2,
        rebalance_entries: 7,
    },
    LevelShape {
        tag: "VillagerTradeTags.LIBRARIAN_LEVEL_3",
        vanilla_entries: 2,
        rebalance_entries: 7,
    },
    LevelShape {
        tag: "VillagerTradeTags.LIBRARIAN_LEVEL_4",
        vanilla_entries: 3,
        rebalance_entries: 0,
    },
    LevelShape {
        tag: "VillagerTradeTags.LIBRARIAN_LEVEL_5",
        vanilla_entries: 2,
        rebalance_entries: 7,
    },
    LevelShape {
        tag: "VillagerTradeTags.ARMORER_LEVEL_1",
        vanilla_entries: 1,
        rebalance_entries: 1,
    },
    LevelShape {
        tag: "VillagerTradeTags.ARMORER_LEVEL_2",
        vanilla_entries: 0,
        rebalance_entries: 8,
    },
    LevelShape {
        tag: "VillagerTradeTags.ARMORER_LEVEL_3",
        vanilla_entries: 2,
        rebalance_entries: 1,
    },
    LevelShape {
        tag: "VillagerTradeTags.ARMORER_LEVEL_4",
        vanilla_entries: 0,
        rebalance_entries: 26,
    },
    LevelShape {
        tag: "VillagerTradeTags.ARMORER_LEVEL_5",
        vanilla_entries: 0,
        rebalance_entries: 16,
    },
];

fn count_occurrences(source: &str, needle: &str) -> usize {
    source.match_indices(needle).count()
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
    fn trade_rebalance_trade_tags_provider_contract_matches_java() {
        assert_source_contains_all(TRADE_REBALANCE_TRADE_TAGS_PROVIDER_JAVA, SOURCE_SENTINELS);
        assert_eq!(
            TRADE_REBALANCE_TRADE_TAGS_PROVIDER_JAVA.lines().count(),
            133
        );
        assert_eq!(
            count_occurrences(TRADE_REBALANCE_TRADE_TAGS_PROVIDER_JAVA, "this.tag("),
            10
        );
        assert_eq!(
            count_occurrences(TRADE_REBALANCE_TRADE_TAGS_PROVIDER_JAVA, ", true)"),
            10
        );
        assert_eq!(
            count_occurrences(TRADE_REBALANCE_TRADE_TAGS_PROVIDER_JAVA, ".add("),
            10
        );
    }

    #[test]
    fn trade_rebalance_trade_tags_provider_level_tags_match_java() {
        for tag in LEVEL_TAGS {
            assert_eq!(
                count_occurrences(TRADE_REBALANCE_TRADE_TAGS_PROVIDER_JAVA, tag),
                1,
                "unexpected level tag count for {tag}"
            );
        }

        assert_eq!(
            LEVEL_SHAPES
                .iter()
                .map(|shape| shape.vanilla_entries)
                .sum::<usize>(),
            14
        );
        assert_eq!(
            LEVEL_SHAPES
                .iter()
                .map(|shape| shape.rebalance_entries)
                .sum::<usize>(),
            80
        );
    }

    #[test]
    fn trade_rebalance_trade_tags_provider_rebalance_prefixes_match_java() {
        for prefix in TRADE_REBALANCE_PREFIXES {
            assert!(
                TRADE_REBALANCE_TRADE_TAGS_PROVIDER_JAVA.contains(prefix),
                "missing rebalance prefix {prefix}"
            );
        }
        assert_eq!(
            count_occurrences(
                TRADE_REBALANCE_TRADE_TAGS_PROVIDER_JAVA,
                "TradeRebalanceVillagerTrades."
            ),
            80
        );
        assert_eq!(
            count_occurrences(TRADE_REBALANCE_TRADE_TAGS_PROVIDER_JAVA, "VillagerTrades.")
                - count_occurrences(
                    TRADE_REBALANCE_TRADE_TAGS_PROVIDER_JAVA,
                    "TradeRebalanceVillagerTrades."
                ),
            14
        );
    }
}
