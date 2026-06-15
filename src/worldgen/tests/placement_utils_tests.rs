use super::*;

const PLACEMENT_UTILS_JAVA: &str = vibecraft_java_source!("/net/minecraft/data/worldgen/placement/PlacementUtils.java");

fn count_occurrences(source: &str, needle: &str) -> usize {
    source.match_indices(needle).count()
}

#[test]
fn placement_utils_java_source_shape_matches_rust_helper_contract() {
    assert_eq!(PLACEMENT_UTILS_JAVA.lines().count(), 117);
    assert_eq!(
        count_occurrences(
            PLACEMENT_UTILS_JAVA,
            "public static final PlacementModifier"
        ),
        10
    );
    assert_eq!(
        count_occurrences(PLACEMENT_UTILS_JAVA, "HeightmapPlacement.onHeightmap"),
        5
    );
    assert_eq!(
        count_occurrences(PLACEMENT_UTILS_JAVA, "HeightRangePlacement.uniform"),
        5
    );
    assert_eq!(
        count_occurrences(PLACEMENT_UTILS_JAVA, ".bootstrap(context);"),
        9
    );
    assert_eq!(
        count_occurrences(PLACEMENT_UTILS_JAVA, "public static void register"),
        2
    );
    assert_eq!(
        count_occurrences(PLACEMENT_UTILS_JAVA, "Holder<PlacedFeature> inlinePlaced"),
        2
    );
    assert_eq!(
        count_occurrences(PLACEMENT_UTILS_JAVA, "BlockPredicateFilter.forPredicate"),
        3
    );

    for sentinel in [
        "AquaticPlacements.bootstrap(context);",
        "VillagePlacements.bootstrap(context);",
        "ResourceKey.create(Registries.PLACED_FEATURE, Identifier.withDefaultNamespace(name))",
        "throw new IllegalStateException(\"Chance data cannot be represented as list weight\");",
        "BlockPredicate.ONLY_IN_AIR_PREDICATE",
        "BlockPredicate.wouldSurvive(block.defaultBlockState(), BlockPos.ZERO)",
        "Holder.direct(new PlacedFeature(configuredFeature, List.of(placedFeatures)))",
    ] {
        assert!(
            PLACEMENT_UTILS_JAVA.contains(sentinel),
            "missing PlacementUtils sentinel {sentinel}"
        );
    }
}

#[test]
fn placement_utils_bootstrap_order_matches_java_and_registry_sources() {
    assert_eq!(
        PLACEMENT_UTILS_BOOTSTRAP_ORDER,
        &[
            PlacedFeatureSource::Aquatic,
            PlacedFeatureSource::Cave,
            PlacedFeatureSource::End,
            PlacedFeatureSource::MiscOverworld,
            PlacedFeatureSource::Nether,
            PlacedFeatureSource::Ore,
            PlacedFeatureSource::Tree,
            PlacedFeatureSource::Vegetation,
            PlacedFeatureSource::Village,
        ]
    );
    assert_eq!(PLACED_FEATURE_BOOTSTRAP_SOURCES.len(), 9);
    assert_eq!(
        PLACED_FEATURE_BOOTSTRAP_SOURCES
            .iter()
            .map(|entry| entry.source)
            .collect::<Vec<_>>(),
        PLACEMENT_UTILS_BOOTSTRAP_ORDER
    );
}

#[test]
fn placement_utils_shared_constants_match_java_anchors_and_heightmaps() {
    assert_eq!(
        PLACEMENT_UTILS_HEIGHTMAP,
        PlacementModifier::Heightmap {
            heightmap: HeightmapKind::MotionBlocking,
        }
    );
    assert_eq!(
        PLACEMENT_UTILS_HEIGHTMAP_NO_LEAVES,
        PlacementModifier::Heightmap {
            heightmap: HeightmapKind::MotionBlockingNoLeaves,
        }
    );
    assert_eq!(
        PLACEMENT_UTILS_HEIGHTMAP_TOP_SOLID,
        PlacementModifier::Heightmap {
            heightmap: HeightmapKind::OceanFloorWg,
        }
    );
    assert_eq!(
        PLACEMENT_UTILS_HEIGHTMAP_WORLD_SURFACE,
        PlacementModifier::Heightmap {
            heightmap: HeightmapKind::WorldSurfaceWg,
        }
    );
    assert_eq!(
        PLACEMENT_UTILS_HEIGHTMAP_OCEAN_FLOOR,
        PlacementModifier::Heightmap {
            heightmap: HeightmapKind::OceanFloor,
        }
    );

    for (actual, min_inclusive, max_inclusive) in [
        (
            PLACEMENT_UTILS_FULL_RANGE,
            VerticalAnchor::AboveBottom(0),
            VerticalAnchor::BelowTop(0),
        ),
        (
            PLACEMENT_UTILS_RANGE_10_10,
            VerticalAnchor::AboveBottom(10),
            VerticalAnchor::BelowTop(10),
        ),
        (
            PLACEMENT_UTILS_RANGE_8_8,
            VerticalAnchor::AboveBottom(8),
            VerticalAnchor::BelowTop(8),
        ),
        (
            PLACEMENT_UTILS_RANGE_4_4,
            VerticalAnchor::AboveBottom(4),
            VerticalAnchor::BelowTop(4),
        ),
        (
            PLACEMENT_UTILS_RANGE_BOTTOM_TO_MAX_TERRAIN_HEIGHT,
            VerticalAnchor::AboveBottom(0),
            VerticalAnchor::Absolute(256),
        ),
    ] {
        assert_eq!(
            actual,
            PlacementModifier::HeightRange {
                height: HeightProvider::Uniform {
                    min_inclusive,
                    max_inclusive,
                },
            }
        );
    }
}

#[test]
fn placement_utils_count_extra_matches_java_weight_validation() {
    assert_eq!(
        placement_utils_count_extra(1, 0.25, 1),
        Ok(PlacementUtilsCountExtra {
            base_count: 1,
            base_weight: 3,
            extra_count: 2,
            extra_weight: 1,
        })
    );
    assert_eq!(
        placement_utils_count_extra(4, 0.5, 2),
        Ok(PlacementUtilsCountExtra {
            base_count: 4,
            base_weight: 1,
            extra_count: 6,
            extra_weight: 1,
        })
    );
    assert_eq!(
        placement_utils_count_extra(1, 0.3, 1),
        Err("Chance data cannot be represented as list weight")
    );
}

#[test]
fn placement_utils_filter_and_inline_helpers_match_java_wrappers() {
    assert_eq!(
        placement_utils_create_key("ore_diamond"),
        "minecraft:ore_diamond"
    );
    assert_eq!(
        placement_utils_create_key("minecraft:ore_diamond"),
        "minecraft:ore_diamond"
    );

    assert_eq!(
        placement_utils_is_empty(),
        PlacementModifier::BlockPredicateFilter {
            predicate: BlockPredicate::MatchingBlocks {
                blocks: &["minecraft:air"],
            },
        }
    );
    assert_eq!(
        placement_utils_filtered_by_block_survival("minecraft:oak_sapling", true),
        PlacementModifier::BlockPredicateFilter {
            predicate: BlockPredicate::WouldSurvive {
                offset_y: 0,
                state: "minecraft:oak_sapling",
                survives: true,
            },
        }
    );

    assert_eq!(
        placement_utils_inline_placed(
            "minecraft:oak",
            &[PlacementModifier::BiomeFilter, PLACEMENT_UTILS_HEIGHTMAP]
        ),
        InlinePlacedFeatureModel {
            configured_feature: "minecraft:oak",
            placement: vec![PlacementModifier::BiomeFilter, PLACEMENT_UTILS_HEIGHTMAP],
        }
    );
    assert_eq!(
        placement_utils_only_when_empty("minecraft:oak"),
        InlinePlacedFeatureModel {
            configured_feature: "minecraft:oak",
            placement: vec![placement_utils_is_empty()],
        }
    );
}
