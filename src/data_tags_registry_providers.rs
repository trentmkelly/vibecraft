const BANNER_PATTERN_TAGS_PROVIDER_JAVA: &str = vibecraft_java_source!("/net/minecraft/data/tags/BannerPatternTagsProvider.java");
const DIALOG_TAGS_PROVIDER_JAVA: &str =
    vibecraft_java_source!("/net/minecraft/data/tags/DialogTagsProvider.java");
const FEATURE_TAGS_PROVIDER_JAVA: &str =
    vibecraft_java_source!("/net/minecraft/data/tags/FeatureTagsProvider.java");
const FLAT_LEVEL_GENERATOR_PRESET_TAGS_PROVIDER_JAVA: &str = vibecraft_java_source!("/net/minecraft/data/tags/FlatLevelGeneratorPresetTagsProvider.java");
const FLUID_TAGS_PROVIDER_JAVA: &str =
    vibecraft_java_source!("/net/minecraft/data/tags/FluidTagsProvider.java");
const INSTRUMENT_TAGS_PROVIDER_JAVA: &str = vibecraft_java_source!("/net/minecraft/data/tags/InstrumentTagsProvider.java");
const PAINTING_VARIANT_TAGS_PROVIDER_JAVA: &str = vibecraft_java_source!("/net/minecraft/data/tags/PaintingVariantTagsProvider.java");
const POI_TYPE_TAGS_PROVIDER_JAVA: &str =
    vibecraft_java_source!("/net/minecraft/data/tags/PoiTypeTagsProvider.java");
const TIMELINE_TAGS_PROVIDER_JAVA: &str = vibecraft_java_source!("/net/minecraft/data/tags/TimelineTagsProvider.java");
const WORLD_PRESET_TAGS_PROVIDER_JAVA: &str = vibecraft_java_source!("/net/minecraft/data/tags/WorldPresetTagsProvider.java");

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
    sentinels: &'static [&'static str],
}

const PROVIDERS: &[ProviderAudit] = &[
    ProviderAudit {
        source: BANNER_PATTERN_TAGS_PROVIDER_JAVA,
        class_name: "BannerPatternTagsProvider",
        extends: "KeyTagProvider<BannerPattern>",
        registry: "Registries.BANNER_PATTERN",
        line_count: 64,
        tag_calls: 11,
        add_calls: 11,
        add_tag_calls: 0,
        sentinels: &[
            "BannerPatternTags.NO_ITEM_REQUIRED",
            "BannerPatterns.SQUARE_BOTTOM_LEFT",
            "BannerPatterns.GRADIENT_UP",
            "BannerPatternTags.PATTERN_ITEM_FLOWER",
            "BannerPatterns.FLOWER",
            "BannerPatternTags.PATTERN_ITEM_GUSTER",
            "BannerPatterns.GUSTER",
            "BannerPatternTags.PATTERN_ITEM_BORDURE_INDENTED",
            "BannerPatterns.CURLY_BORDER",
        ],
    },
    ProviderAudit {
        source: DIALOG_TAGS_PROVIDER_JAVA,
        class_name: "DialogTagsProvider",
        extends: "KeyTagProvider<Dialog>",
        registry: "Registries.DIALOG",
        line_count: 20,
        tag_calls: 2,
        add_calls: 0,
        add_tag_calls: 0,
        sentinels: &[
            "DialogTags.PAUSE_SCREEN_ADDITIONS",
            "DialogTags.QUICK_ACTIONS",
        ],
    },
    ProviderAudit {
        source: FEATURE_TAGS_PROVIDER_JAVA,
        class_name: "FeatureTagsProvider",
        extends: "KeyTagProvider<ConfiguredFeature<?, ?>>",
        registry: "Registries.CONFIGURED_FEATURE",
        line_count: 30,
        tag_calls: 1,
        add_calls: 1,
        add_tag_calls: 0,
        sentinels: &[
            "FeatureTags.CAN_SPAWN_FROM_BONE_MEAL",
            "VegetationFeatures.FLOWER_DEFAULT",
            "VegetationFeatures.FLOWER_CHERRY",
            "VegetationFeatures.WILDFLOWER",
            "VegetationFeatures.FLOWER_PALE_GARDEN",
        ],
    },
    ProviderAudit {
        source: FLAT_LEVEL_GENERATOR_PRESET_TAGS_PROVIDER_JAVA,
        class_name: "FlatLevelGeneratorPresetTagsProvider",
        extends: "KeyTagProvider<FlatLevelGeneratorPreset>",
        registry: "Registries.FLAT_LEVEL_GENERATOR_PRESET",
        line_count: 29,
        tag_calls: 1,
        add_calls: 9,
        add_tag_calls: 0,
        sentinels: &[
            "FlatLevelGeneratorPresetTags.VISIBLE",
            "FlatLevelGeneratorPresets.CLASSIC_FLAT",
            "FlatLevelGeneratorPresets.BOTTOMLESS_PIT",
            "FlatLevelGeneratorPresets.THE_VOID",
        ],
    },
    ProviderAudit {
        source: FLUID_TAGS_PROVIDER_JAVA,
        class_name: "FluidTagsProvider",
        extends: "IntrinsicHolderTagsProvider<Fluid>",
        registry: "Registries.FLUID",
        line_count: 25,
        tag_calls: 6,
        add_calls: 5,
        add_tag_calls: 1,
        sentinels: &[
            "e -> e.builtInRegistryHolder().key()",
            "FluidTags.WATER",
            "Fluids.WATER, Fluids.FLOWING_WATER",
            "FluidTags.LAVA",
            "Fluids.LAVA, Fluids.FLOWING_LAVA",
            "FluidTags.SUPPORTS_SUGAR_CANE_ADJACENTLY",
            "this.tag(FluidTags.SUPPORTS_SUGAR_CANE_ADJACENTLY).addTag(FluidTags.WATER);",
            "FluidTags.BUBBLE_COLUMN_CAN_OCCUPY",
        ],
    },
    ProviderAudit {
        source: INSTRUMENT_TAGS_PROVIDER_JAVA,
        class_name: "InstrumentTagsProvider",
        extends: "KeyTagProvider<Instrument>",
        registry: "Registries.INSTRUMENT",
        line_count: 30,
        tag_calls: 3,
        add_calls: 8,
        add_tag_calls: 2,
        sentinels: &[
            "InstrumentTags.REGULAR_GOAT_HORNS",
            "Instruments.PONDER_GOAT_HORN",
            "InstrumentTags.SCREAMING_GOAT_HORNS",
            "Instruments.DREAM_GOAT_HORN",
            "this.tag(InstrumentTags.GOAT_HORNS).addTag(InstrumentTags.REGULAR_GOAT_HORNS).addTag(InstrumentTags.SCREAMING_GOAT_HORNS);",
        ],
    },
    ProviderAudit {
        source: PAINTING_VARIANT_TAGS_PROVIDER_JAVA,
        class_name: "PaintingVariantTagsProvider",
        extends: "KeyTagProvider<PaintingVariant>",
        registry: "Registries.PAINTING_VARIANT",
        line_count: 69,
        tag_calls: 1,
        add_calls: 1,
        add_tag_calls: 0,
        sentinels: &[
            "PaintingVariantTags.PLACEABLE",
            "PaintingVariants.KEBAB",
            "PaintingVariants.DONKEY_KONG",
            "PaintingVariants.PRAIRIE_RIDE",
            "PaintingVariants.DENNIS",
        ],
    },
    ProviderAudit {
        source: POI_TYPE_TAGS_PROVIDER_JAVA,
        class_name: "PoiTypeTagsProvider",
        extends: "KeyTagProvider<PoiType>",
        registry: "Registries.POINT_OF_INTEREST_TYPE",
        line_count: 37,
        tag_calls: 3,
        add_calls: 3,
        add_tag_calls: 1,
        sentinels: &[
            "PoiTypeTags.ACQUIRABLE_JOB_SITE",
            "PoiTypes.ARMORER",
            "PoiTypes.WEAPONSMITH",
            "this.tag(PoiTypeTags.VILLAGE).addTag(PoiTypeTags.ACQUIRABLE_JOB_SITE).add(PoiTypes.HOME, PoiTypes.MEETING);",
            "this.tag(PoiTypeTags.BEE_HOME).add(PoiTypes.BEEHIVE, PoiTypes.BEE_NEST);",
        ],
    },
    ProviderAudit {
        source: TIMELINE_TAGS_PROVIDER_JAVA,
        class_name: "TimelineTagsProvider",
        extends: "KeyTagProvider<Timeline>",
        registry: "Registries.TIMELINE",
        line_count: 23,
        tag_calls: 4,
        add_calls: 2,
        add_tag_calls: 3,
        sentinels: &[
            "TimelineTags.UNIVERSAL",
            "Timelines.VILLAGER_SCHEDULE",
            "this.tag(TimelineTags.IN_OVERWORLD).addTag(TimelineTags.UNIVERSAL).add(Timelines.OVERWORLD_DAY, Timelines.MOON, Timelines.EARLY_GAME);",
            "this.tag(TimelineTags.IN_NETHER).addTag(TimelineTags.UNIVERSAL);",
            "this.tag(TimelineTags.IN_END).addTag(TimelineTags.UNIVERSAL);",
        ],
    },
    ProviderAudit {
        source: WORLD_PRESET_TAGS_PROVIDER_JAVA,
        class_name: "WorldPresetTagsProvider",
        extends: "KeyTagProvider<WorldPreset>",
        registry: "Registries.WORLD_PRESET",
        line_count: 26,
        tag_calls: 2,
        add_calls: 6,
        add_tag_calls: 1,
        sentinels: &[
            "WorldPresetTags.NORMAL",
            "WorldPresets.NORMAL",
            "WorldPresets.SINGLE_BIOME_SURFACE",
            "this.tag(WorldPresetTags.EXTENDED).addTag(WorldPresetTags.NORMAL).add(WorldPresets.DEBUG);",
        ],
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
    fn data_tags_registry_provider_class_contracts_match_java() {
        for provider in PROVIDERS {
            assert!(
                provider.source.contains(&format!(
                    "public class {} extends {}",
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
            assert!(
                provider
                    .source
                    .contains("protected void addTags(final HolderLookup.Provider registries)"),
                "missing addTags override for {}",
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
    fn data_tags_registry_provider_call_counts_match_java() {
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
                "addTag call count mismatch for {}",
                provider.class_name
            );
        }
    }

    #[test]
    fn data_tags_registry_provider_entries_match_java_sentinels() {
        for provider in PROVIDERS {
            assert_source_contains_all(provider.source, provider.sentinels);
        }
    }

    #[test]
    fn data_tags_registry_provider_group_counts_match_java() {
        assert_eq!(PROVIDERS.len(), 10);
        assert_eq!(
            PROVIDERS
                .iter()
                .map(|provider| provider.tag_calls)
                .sum::<usize>(),
            34
        );
        assert_eq!(
            PROVIDERS
                .iter()
                .map(|provider| provider.add_tag_calls)
                .sum::<usize>(),
            8
        );
        assert_eq!(
            PROVIDERS
                .iter()
                .filter(|provider| provider.extends.starts_with("KeyTagProvider"))
                .count(),
            9
        );
        assert_eq!(
            PROVIDERS
                .iter()
                .filter(|provider| provider.extends.starts_with("IntrinsicHolderTagsProvider"))
                .count(),
            1
        );
    }
}
