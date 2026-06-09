const DAMAGE_TYPE_TAGS_PROVIDER_JAVA: &str = include_str!(
    "../../decompiled-server-26.1.2/net/minecraft/data/tags/DamageTypeTagsProvider.java"
);
const ENCHANTMENT_TAGS_PROVIDER_JAVA: &str = include_str!(
    "../../decompiled-server-26.1.2/net/minecraft/data/tags/EnchantmentTagsProvider.java"
);
const GAME_EVENT_TAGS_PROVIDER_JAVA: &str = include_str!(
    "../../decompiled-server-26.1.2/net/minecraft/data/tags/GameEventTagsProvider.java"
);
const POTION_TAGS_PROVIDER_JAVA: &str =
    include_str!("../../decompiled-server-26.1.2/net/minecraft/data/tags/PotionTagsProvider.java");
const STRUCTURE_TAGS_PROVIDER_JAVA: &str = include_str!(
    "../../decompiled-server-26.1.2/net/minecraft/data/tags/StructureTagsProvider.java"
);
const TRADE_REBALANCE_ENCHANTMENT_TAGS_PROVIDER_JAVA: &str = include_str!(
    "../../decompiled-server-26.1.2/net/minecraft/data/tags/TradeRebalanceEnchantmentTagsProvider.java"
);

#[derive(Debug, Clone, Copy)]
struct ProviderAudit {
    source: &'static str,
    class_name: &'static str,
    extends: &'static str,
    registry: &'static str,
    line_count: usize,
    tag_calls: usize,
    add_calls: usize,
    add_tag_calls: usize,
    add_all_calls: usize,
    sentinels: &'static [&'static str],
}

const PROVIDERS: &[ProviderAudit] = &[
    ProviderAudit {
        source: DAMAGE_TYPE_TAGS_PROVIDER_JAVA,
        class_name: "DamageTypeTagsProvider",
        extends: "KeyTagProvider<DamageType>",
        registry: "Registries.DAMAGE_TYPE",
        line_count: 182,
        tag_calls: 33,
        add_calls: 32,
        add_tag_calls: 7,
        add_all_calls: 0,
        sentinels: &[
            "DamageTypeTags.DAMAGES_HELMET",
            "DamageTypes.FALLING_STALACTITE",
            "DamageTypeTags.BYPASSES_ARMOR",
            "DamageTypes.OUTSIDE_BORDER",
            "this.tag(DamageTypeTags.BYPASSES_SHIELD)\n         .addTag(DamageTypeTags.BYPASSES_ARMOR)",
            "DamageTypeTags.AVOIDS_GUARDIAN_THORNS",
            "DamageTypeTags.ALWAYS_HURTS_ENDER_DRAGONS",
            "DamageTypeTags.BYPASSES_WOLF_ARMOR",
            "DamageTypes.MACE_SMASH",
            "DamageTypeTags.PANIC_CAUSES",
            ".addTag(DamageTypeTags.IS_PLAYER_ATTACK);",
            "DamageTypeTags.IS_MACE_SMASH",
        ],
    },
    ProviderAudit {
        source: ENCHANTMENT_TAGS_PROVIDER_JAVA,
        class_name: "EnchantmentTagsProvider",
        extends: "KeyTagProvider<Enchantment>",
        registry: "Registries.ENCHANTMENT",
        line_count: 32,
        tag_calls: 1,
        add_calls: 1,
        add_tag_calls: 0,
        add_all_calls: 0,
        sentinels: &[
            "public abstract class EnchantmentTagsProvider extends KeyTagProvider<Enchantment>",
            "protected void tooltipOrder(final HolderLookup.Provider registries, final ResourceKey<Enchantment>... order)",
            "this.tag(EnchantmentTags.TOOLTIP_ORDER).add(order);",
            "Set<ResourceKey<Enchantment>> set = Set.of(order);",
            ".filter(e -> !set.contains(e.unwrapKey().get()))",
            ".map(Holder::getRegisteredName)",
            "throw new IllegalStateException(\"Not all enchantments were registered for tooltip ordering. Missing: \" + String.join(\", \", unlisted));",
        ],
    },
    ProviderAudit {
        source: GAME_EVENT_TAGS_PROVIDER_JAVA,
        class_name: "GameEventTagsProvider",
        extends: "KeyTagProvider<GameEvent>",
        registry: "Registries.GAME_EVENT",
        line_count: 82,
        tag_calls: 5,
        add_calls: 5,
        add_tag_calls: 1,
        add_all_calls: 4,
        sentinels: &[
            "@VisibleForTesting",
            "static final List<ResourceKey<GameEvent>> VIBRATIONS_EXCEPT_FLAP = List.of(",
            "GameEvent.BLOCK_ATTACH.key()",
            "GameEvent.UNEQUIP.key()",
            "this.tag(GameEventTags.VIBRATIONS).addAll(VIBRATIONS_EXCEPT_FLAP).addAll(VibrationSystem.RESONANCE_EVENTS).add(GameEvent.FLAP.key());",
            "GameEventTags.SHRIEKER_CAN_LISTEN",
            "GameEvent.SCULK_SENSOR_TENDRILS_CLICKING.key()",
            "this.tag(GameEventTags.WARDEN_CAN_LISTEN)",
            ".addTag(GameEventTags.SHRIEKER_CAN_LISTEN);",
            "GameEventTags.IGNORE_VIBRATIONS_SNEAKING",
            "GameEvent.ITEM_INTERACT_START.key()",
            "GameEventTags.ALLAY_CAN_LISTEN",
        ],
    },
    ProviderAudit {
        source: POTION_TAGS_PROVIDER_JAVA,
        class_name: "PotionTagsProvider",
        extends: "HolderTagProvider<Potion>",
        registry: "Registries.POTION",
        line_count: 63,
        tag_calls: 1,
        add_calls: 1,
        add_tag_calls: 0,
        add_all_calls: 0,
        sentinels: &[
            "PotionTags.TRADEABLE",
            "Potions.WIND_CHARGED",
            "Potions.OOZING",
            "Potions.TURTLE_MASTER",
            "Potions.STRONG_TURTLE_MASTER",
            "Potions.LONG_SLOW_FALLING",
        ],
    },
    ProviderAudit {
        source: STRUCTURE_TAGS_PROVIDER_JAVA,
        class_name: "StructureTagsProvider",
        extends: "KeyTagProvider<Structure>",
        registry: "Registries.STRUCTURE",
        line_count: 51,
        tag_calls: 20,
        add_calls: 32,
        add_tag_calls: 2,
        add_all_calls: 0,
        sentinels: &[
            "StructureTags.VILLAGE",
            "BuiltinStructures.VILLAGE_PLAINS",
            "StructureTags.MINESHAFT",
            "StructureTags.RUINED_PORTAL",
            "BuiltinStructures.RUINED_PORTAL_SWAMP",
            "StructureTags.DOLPHIN_LOCATED",
            "this.tag(StructureTags.DOLPHIN_LOCATED).addTag(StructureTags.OCEAN_RUIN).addTag(StructureTags.SHIPWRECK);",
            "StructureTags.ON_TRIAL_CHAMBERS_MAPS",
            "BuiltinStructures.TRIAL_CHAMBERS",
            "StructureTags.ON_JUNGLE_EXPLORER_MAPS",
        ],
    },
    ProviderAudit {
        source: TRADE_REBALANCE_ENCHANTMENT_TAGS_PROVIDER_JAVA,
        class_name: "TradeRebalanceEnchantmentTagsProvider",
        extends: "KeyTagProvider<Enchantment>",
        registry: "Registries.ENCHANTMENT",
        line_count: 26,
        tag_calls: 7,
        add_calls: 7,
        add_tag_calls: 0,
        add_all_calls: 0,
        sentinels: &[
            "EnchantmentTags.TRADES_DESERT_COMMON",
            "Enchantments.FIRE_PROTECTION, Enchantments.THORNS, Enchantments.INFINITY",
            "EnchantmentTags.TRADES_JUNGLE_COMMON",
            "EnchantmentTags.TRADES_PLAINS_COMMON",
            "EnchantmentTags.TRADES_SAVANNA_COMMON",
            "EnchantmentTags.TRADES_SNOW_COMMON",
            "EnchantmentTags.TRADES_SWAMP_COMMON",
            "EnchantmentTags.TRADES_TAIGA_COMMON",
            "Enchantments.BLAST_PROTECTION, Enchantments.FIRE_ASPECT, Enchantments.FLAME",
        ],
    },
];

fn tooltip_order_missing<'a>(registered: &'a [&str], ordered: &[&str]) -> Vec<&'a str> {
    registered
        .iter()
        .copied()
        .filter(|enchantment| !ordered.contains(enchantment))
        .collect()
}

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
    fn data_tags_medium_provider_class_contracts_match_java() {
        for provider in PROVIDERS {
            assert!(
                provider.source.contains(&format!(
                    "public class {} extends {}",
                    provider.class_name, provider.extends
                )) || provider.source.contains(&format!(
                    "public abstract class {} extends {}",
                    provider.class_name, provider.extends
                )),
                "class declaration mismatch for {}",
                provider.class_name
            );
            assert!(
                provider.source.contains(&format!(
                    "super(output, {}, lookupProvider",
                    provider.registry
                )),
                "registry constructor mismatch for {}",
                provider.class_name
            );
            assert_eq!(
                provider.source.lines().count(),
                provider.line_count,
                "line-count drift for {}",
                provider.class_name
            );
        }
    }

    #[test]
    fn data_tags_medium_provider_call_counts_match_java() {
        for provider in PROVIDERS {
            assert_eq!(
                count_occurrences(provider.source, "this.tag("),
                provider.tag_calls,
                "tag call count mismatch for {}",
                provider.class_name
            );
            assert_eq!(
                count_occurrences(provider.source, ".add("),
                provider.add_calls,
                "add call count mismatch for {}",
                provider.class_name
            );
            assert_eq!(
                count_occurrences(provider.source, ".addTag("),
                provider.add_tag_calls,
                "addTag count mismatch for {}",
                provider.class_name
            );
            assert_eq!(
                count_occurrences(provider.source, ".addAll("),
                provider.add_all_calls,
                "addAll count mismatch for {}",
                provider.class_name
            );
        }
    }

    #[test]
    fn data_tags_medium_provider_entries_match_java_sentinels() {
        for provider in PROVIDERS {
            assert_source_contains_all(provider.source, provider.sentinels);
        }
        assert_eq!(
            count_occurrences(GAME_EVENT_TAGS_PROVIDER_JAVA, "GameEvent."),
            49
        );
        assert_eq!(count_occurrences(POTION_TAGS_PROVIDER_JAVA, "Potions."), 41);
        assert_eq!(
            count_occurrences(STRUCTURE_TAGS_PROVIDER_JAVA, "BuiltinStructures."),
            32
        );
    }

    #[test]
    fn data_tags_medium_enchantment_tooltip_order_requires_complete_registry() {
        let registered = ["sharpness", "mending", "flame", "density"];
        assert_eq!(
            tooltip_order_missing(&registered, &["sharpness", "flame", "density"]),
            vec!["mending"]
        );
        assert!(tooltip_order_missing(&registered, &registered).is_empty());
    }

    #[test]
    fn data_tags_medium_provider_group_counts_match_java() {
        assert_eq!(PROVIDERS.len(), 6);
        assert_eq!(
            PROVIDERS
                .iter()
                .map(|provider| provider.tag_calls)
                .sum::<usize>(),
            67
        );
        assert_eq!(
            PROVIDERS
                .iter()
                .map(|provider| provider.add_tag_calls)
                .sum::<usize>(),
            10
        );
    }
}
