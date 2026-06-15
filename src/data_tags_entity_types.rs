const ENTITY_TYPE_TAGS_PROVIDER_JAVA: &str = vibecraft_java_source!("/net/minecraft/data/tags/EntityTypeTagsProvider.java");

const ENTITY_TYPE_TAG_SENTINELS: &[&str] = &[
    "public class EntityTypeTagsProvider extends IntrinsicHolderTagsProvider<EntityType<?>>",
    "super(output, Registries.ENTITY_TYPE, lookupProvider, e -> e.builtInRegistryHolder().key());",
    "EntityTypeTags.SKELETONS",
    "EntityType.PARCHED",
    "EntityTypeTags.ZOMBIES",
    "EntityType.ZOMBIE_NAUTILUS",
    "this.tag(EntityTypeTags.UNDEAD).addTag(EntityTypeTags.SKELETONS).addTag(EntityTypeTags.ZOMBIES)",
    "EntityTypeTags.IMPACT_PROJECTILES",
    "EntityType.BREEZE_WIND_CHARGE",
    "EntityTypeTags.CAN_BREATHE_UNDER_WATER",
    "EntityType.COPPER_GOLEM",
    "EntityType.NAUTILUS",
    "EntityTypeTags.FALL_DAMAGE_IMMUNE",
    "EntityType.HAPPY_GHAST",
    "EntityTypeTags.AQUATIC",
    "EntityTypeTags.NOT_SCARY_FOR_PUFFERFISH",
    "EntityTypeTags.NO_ANGER_FROM_WIND_CHARGE",
    "EntityTypeTags.BOAT",
    "EntityType.BAMBOO_RAFT",
    "EntityTypeTags.CAN_EQUIP_SADDLE",
    "EntityType.CAMEL_HUSK",
    "EntityTypeTags.CAN_WEAR_NAUTILUS_ARMOR",
    "EntityTypeTags.FOLLOWABLE_FRIENDLY_MOBS",
    "EntityType.SNIFFER",
    "EntityTypeTags.CANNOT_BE_PUSHED_ONTO_BOATS",
    "EntityType.CREAKING",
    "EntityTypeTags.CANDIDATE_FOR_IRON_GOLEM_GIFT",
    ".addTag(EntityTypeTags.ACCEPTS_IRON_GOLEM_GIFT)",
    "EntityTypeTags.CAN_FLOAT_WHILE_RIDDEN",
    "EntityTypeTags.CANNOT_BE_AGE_LOCKED",
];

const ENTITY_TYPE_TAG_REFERENCE_COUNTS: &[(&str, usize)] = &[
    ("EntityTypeTags.ACCEPTS_IRON_GOLEM_GIFT", 2),
    ("EntityTypeTags.AQUATIC", 2),
    ("EntityTypeTags.ARROWS", 2),
    ("EntityTypeTags.ARTHROPOD", 2),
    ("EntityTypeTags.ILLAGER", 2),
    ("EntityTypeTags.SKELETONS", 2),
    ("EntityTypeTags.UNDEAD", 6),
    ("EntityTypeTags.ZOMBIES", 2),
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
    fn entity_type_tags_provider_contract_matches_java() {
        assert_source_contains_all(ENTITY_TYPE_TAGS_PROVIDER_JAVA, ENTITY_TYPE_TAG_SENTINELS);
        assert_eq!(
            ENTITY_TYPE_TAGS_PROVIDER_JAVA.lines().count(),
            257,
            "EntityTypeTagsProvider.java line-count drift"
        );
        assert_eq!(
            count_occurrences(ENTITY_TYPE_TAGS_PROVIDER_JAVA, "this.tag("),
            47,
            "entity tag builder count drift"
        );
        assert_eq!(
            count_occurrences(ENTITY_TYPE_TAGS_PROVIDER_JAVA, ".add("),
            89,
            "entity tag direct-add count drift"
        );
        assert_eq!(
            count_occurrences(ENTITY_TYPE_TAGS_PROVIDER_JAVA, ".addTag("),
            12,
            "entity tag reference count drift"
        );
        assert_eq!(
            count_occurrences(ENTITY_TYPE_TAGS_PROVIDER_JAVA, "EntityType."),
            245,
            "entity type reference count drift"
        );
    }

    #[test]
    fn entity_type_tag_reference_shape_matches_java() {
        for (tag, expected_count) in ENTITY_TYPE_TAG_REFERENCE_COUNTS {
            assert_eq!(
                count_java_identifier_occurrences(ENTITY_TYPE_TAGS_PROVIDER_JAVA, tag),
                *expected_count,
                "reference-count drift for {tag}"
            );
        }
    }
}
