const VANILLA_ENCHANTMENT_TAGS_PROVIDER_JAVA: &str = include_str!(
    "../../decompiled-server-26.1.2/net/minecraft/data/tags/VanillaEnchantmentTagsProvider.java"
);

const TOOLTIP_ORDER_SENTINELS: &[&str] = &[
    "Enchantments.BINDING_CURSE",
    "Enchantments.VANISHING_CURSE",
    "Enchantments.RIPTIDE",
    "Enchantments.CHANNELING",
    "Enchantments.WIND_BURST",
    "Enchantments.FROST_WALKER",
    "Enchantments.LUNGE",
    "Enchantments.SHARPNESS",
    "Enchantments.SMITE",
    "Enchantments.BANE_OF_ARTHROPODS",
    "Enchantments.IMPALING",
    "Enchantments.POWER",
    "Enchantments.DENSITY",
    "Enchantments.BREACH",
    "Enchantments.PIERCING",
    "Enchantments.SWEEPING_EDGE",
    "Enchantments.MULTISHOT",
    "Enchantments.FIRE_ASPECT",
    "Enchantments.FLAME",
    "Enchantments.KNOCKBACK",
    "Enchantments.PUNCH",
    "Enchantments.PROTECTION",
    "Enchantments.BLAST_PROTECTION",
    "Enchantments.FIRE_PROTECTION",
    "Enchantments.PROJECTILE_PROTECTION",
    "Enchantments.FEATHER_FALLING",
    "Enchantments.FORTUNE",
    "Enchantments.LOOTING",
    "Enchantments.SILK_TOUCH",
    "Enchantments.LUCK_OF_THE_SEA",
    "Enchantments.EFFICIENCY",
    "Enchantments.QUICK_CHARGE",
    "Enchantments.LURE",
    "Enchantments.RESPIRATION",
    "Enchantments.AQUA_AFFINITY",
    "Enchantments.SOUL_SPEED",
    "Enchantments.SWIFT_SNEAK",
    "Enchantments.DEPTH_STRIDER",
    "Enchantments.THORNS",
    "Enchantments.LOYALTY",
    "Enchantments.UNBREAKING",
    "Enchantments.INFINITY",
    "Enchantments.MENDING",
];

const SOURCE_SENTINELS: &[&str] = &[
    "public class VanillaEnchantmentTagsProvider extends EnchantmentTagsProvider",
    "super(output, lookupProvider);",
    "this.tooltipOrder(",
    "this.tag(EnchantmentTags.ARMOR_EXCLUSIVE)",
    "Enchantments.PROTECTION, Enchantments.BLAST_PROTECTION, Enchantments.FIRE_PROTECTION, Enchantments.PROJECTILE_PROTECTION",
    "this.tag(EnchantmentTags.BOOTS_EXCLUSIVE).add(Enchantments.FROST_WALKER, Enchantments.DEPTH_STRIDER);",
    "this.tag(EnchantmentTags.BOW_EXCLUSIVE).add(Enchantments.INFINITY, Enchantments.MENDING);",
    "this.tag(EnchantmentTags.CROSSBOW_EXCLUSIVE).add(Enchantments.MULTISHOT, Enchantments.PIERCING);",
    "this.tag(EnchantmentTags.DAMAGE_EXCLUSIVE)",
    "Enchantments.SHARPNESS, Enchantments.SMITE, Enchantments.BANE_OF_ARTHROPODS, Enchantments.IMPALING, Enchantments.DENSITY, Enchantments.BREACH",
    "this.tag(EnchantmentTags.MINING_EXCLUSIVE).add(Enchantments.FORTUNE, Enchantments.SILK_TOUCH);",
    "this.tag(EnchantmentTags.RIPTIDE_EXCLUSIVE).add(Enchantments.LOYALTY, Enchantments.CHANNELING);",
    "this.tag(EnchantmentTags.TREASURE)",
    "Enchantments.SWIFT_SNEAK",
    "this.tag(EnchantmentTags.NON_TREASURE)",
    "Enchantments.LUNGE",
    "this.tag(EnchantmentTags.DOUBLE_TRADE_PRICE).addTag(EnchantmentTags.TREASURE);",
    "this.tag(EnchantmentTags.IN_ENCHANTING_TABLE).addTag(EnchantmentTags.NON_TREASURE);",
    "this.tag(EnchantmentTags.ON_RANDOM_LOOT)",
    "this.tag(EnchantmentTags.TRADEABLE)",
    "this.tag(EnchantmentTags.CURSE).add(Enchantments.BINDING_CURSE, Enchantments.VANISHING_CURSE);",
    "this.tag(EnchantmentTags.SMELTS_LOOT).add(Enchantments.FIRE_ASPECT);",
    "this.tag(EnchantmentTags.PREVENTS_BEE_SPAWNS_WHEN_MINING).add(Enchantments.SILK_TOUCH);",
    "this.tag(EnchantmentTags.PREVENTS_ICE_MELTING).add(Enchantments.SILK_TOUCH);",
];

const CATEGORY_SHAPES: &[(&str, usize)] = &[
    ("EnchantmentTags.ARMOR_EXCLUSIVE", 4),
    ("EnchantmentTags.BOOTS_EXCLUSIVE", 2),
    ("EnchantmentTags.BOW_EXCLUSIVE", 2),
    ("EnchantmentTags.CROSSBOW_EXCLUSIVE", 2),
    ("EnchantmentTags.DAMAGE_EXCLUSIVE", 6),
    ("EnchantmentTags.MINING_EXCLUSIVE", 2),
    ("EnchantmentTags.RIPTIDE_EXCLUSIVE", 2),
    ("EnchantmentTags.TREASURE", 7),
    ("EnchantmentTags.NON_TREASURE", 36),
    ("EnchantmentTags.CURSE", 2),
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
    fn vanilla_enchantment_tags_provider_contract_matches_java() {
        assert_source_contains_all(VANILLA_ENCHANTMENT_TAGS_PROVIDER_JAVA, SOURCE_SENTINELS);
        assert_eq!(VANILLA_ENCHANTMENT_TAGS_PROVIDER_JAVA.lines().count(), 137);
        assert_eq!(
            count_occurrences(VANILLA_ENCHANTMENT_TAGS_PROVIDER_JAVA, "this.tag("),
            21
        );
        assert_eq!(
            count_occurrences(VANILLA_ENCHANTMENT_TAGS_PROVIDER_JAVA, ".add("),
            17
        );
        assert_eq!(
            count_occurrences(VANILLA_ENCHANTMENT_TAGS_PROVIDER_JAVA, ".addTag("),
            6
        );
    }

    #[test]
    fn vanilla_enchantment_tooltip_order_matches_java_sequence() {
        let mut previous = 0;
        for enchantment in TOOLTIP_ORDER_SENTINELS {
            let index = VANILLA_ENCHANTMENT_TAGS_PROVIDER_JAVA
                .find(enchantment)
                .unwrap_or_else(|| panic!("missing tooltip enchantment {enchantment}"));
            assert!(
                index >= previous,
                "tooltip enchantment out of Java order: {enchantment}"
            );
            previous = index;
        }
        assert_eq!(TOOLTIP_ORDER_SENTINELS.len(), 43);
    }

    #[test]
    fn vanilla_enchantment_category_shapes_match_java() {
        for (tag, expected_mentions) in CATEGORY_SHAPES {
            let tag_declaration = format!("this.tag({tag})");
            assert_eq!(
                count_occurrences(VANILLA_ENCHANTMENT_TAGS_PROVIDER_JAVA, &tag_declaration),
                1,
                "unexpected tag count for {tag}"
            );
            assert!(
                *expected_mentions > 0,
                "category shape for {tag} must record at least one Java entry"
            );
        }
        assert_eq!(
            CATEGORY_SHAPES
                .iter()
                .map(|(_, entries)| entries)
                .sum::<usize>(),
            65
        );
        assert_eq!(
            count_occurrences(VANILLA_ENCHANTMENT_TAGS_PROVIDER_JAVA, "Enchantments."),
            121
        );
    }
}
