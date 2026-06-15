const BIOME_TAGS_PROVIDER_JAVA: &str =
    vibecraft_java_source!("/net/minecraft/data/tags/BiomeTagsProvider.java");

const BIOME_TAG_SENTINELS: &[&str] = &[
    "public class BiomeTagsProvider extends KeyTagProvider<Biome>",
    "super(output, Registries.BIOME, lookupProvider);",
    "this.tag(BiomeTags.IS_DEEP_OCEAN)",
    "Biomes.DEEP_LUKEWARM_OCEAN",
    "this.tag(BiomeTags.IS_OCEAN)\n         .addTag(BiomeTags.IS_DEEP_OCEAN)",
    "this.tag(BiomeTags.IS_MOUNTAIN)",
    "Biomes.CHERRY_GROVE",
    "this.tag(BiomeTags.IS_FOREST)",
    "Biomes.PALE_GARDEN",
    "this.tag(BiomeTags.IS_NETHER).addAll(MultiNoiseBiomeSourceParameterList.Preset.NETHER.usedBiomes());",
    "List<ResourceKey<Biome>> overworldBiomes = MultiNoiseBiomeSourceParameterList.Preset.OVERWORLD.usedBiomes().toList();",
    "this.tag(BiomeTags.IS_OVERWORLD).addAll(overworldBiomes);",
    "this.tag(BiomeTags.HAS_MINESHAFT)",
    ".addTag(BiomeTags.IS_FOREST)",
    "this.tag(BiomeTags.HAS_TRIAL_CHAMBERS).addAll(overworldBiomes.stream().filter(biomeKey -> biomeKey != Biomes.DEEP_DARK));",
    "this.tag(BiomeTags.SPAWNS_COLD_VARIANT_FROGS)",
    ".addTag(BiomeTags.IS_END);",
    "this.tag(BiomeTags.SPAWNS_WARM_VARIANT_FROGS)",
    ".addTag(BiomeTags.IS_NETHER)",
    "this.tag(BiomeTags.SPAWNS_COLD_VARIANT_FARM_ANIMALS)",
    "Biomes.DEEP_COLD_OCEAN",
    "this.tag(BiomeTags.SPAWNS_WARM_VARIANT_FARM_ANIMALS)",
    "Biomes.DEEP_LUKEWARM_OCEAN",
    "this.tag(BiomeTags.SPAWNS_SNOW_FOXES)",
    "this.tag(BiomeTags.SPAWNS_CORAL_VARIANT_ZOMBIE_NAUTILUS).add(Biomes.WARM_OCEAN);",
];

const BIOME_TAG_REFERENCE_COUNTS: &[(&str, usize)] = &[
    ("BiomeTags.IS_OCEAN", 6),
    ("BiomeTags.IS_RIVER", 7),
    ("BiomeTags.IS_BEACH", 5),
    ("BiomeTags.IS_JUNGLE", 5),
    ("BiomeTags.IS_NETHER", 5),
    ("BiomeTags.IS_END", 3),
    ("BiomeTags.IS_OVERWORLD", 2),
    ("BiomeTags.HAS_TRIAL_CHAMBERS", 1),
    ("BiomeTags.SPAWNS_CORAL_VARIANT_ZOMBIE_NAUTILUS", 1),
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
    fn biome_tags_provider_contract_matches_java() {
        assert_source_contains_all(BIOME_TAGS_PROVIDER_JAVA, BIOME_TAG_SENTINELS);
        assert_eq!(
            BIOME_TAGS_PROVIDER_JAVA.lines().count(),
            277,
            "BiomeTagsProvider.java line-count drift"
        );
        assert_eq!(
            count_occurrences(BIOME_TAGS_PROVIDER_JAVA, "this.tag("),
            68,
            "biome tag builder count drift"
        );
        assert_eq!(
            count_occurrences(BIOME_TAGS_PROVIDER_JAVA, ".add("),
            230,
            "biome direct-add count drift"
        );
        assert_eq!(
            count_occurrences(BIOME_TAGS_PROVIDER_JAVA, ".addTag("),
            43,
            "biome tag reference count drift"
        );
        assert_eq!(
            count_occurrences(BIOME_TAGS_PROVIDER_JAVA, ".addAll("),
            3,
            "biome addAll count drift"
        );
        assert_eq!(
            count_occurrences(BIOME_TAGS_PROVIDER_JAVA, "Biomes."),
            232,
            "biome key reference count drift"
        );
    }

    #[test]
    fn biome_tag_reference_shape_matches_java() {
        for (tag, expected_count) in BIOME_TAG_REFERENCE_COUNTS {
            assert_eq!(
                count_java_identifier_occurrences(BIOME_TAGS_PROVIDER_JAVA, tag),
                *expected_count,
                "reference-count drift for {tag}"
            );
        }
    }
}
