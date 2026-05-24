use super::*;

pub fn feature_type_by_id(id: &str) -> Option<&'static FeatureType> {
    let name = id.strip_prefix("minecraft:").unwrap_or(id);
    FEATURE_TYPES.iter().find(|entry| {
        entry
            .id
            .strip_prefix("minecraft:")
            .is_some_and(|entry_name| entry_name == name)
    })
}

pub fn configured_feature(id: &str) -> Option<&'static ConfiguredFeatureEntry> {
    let name = id.strip_prefix("minecraft:").unwrap_or(id);
    CONFIGURED_FEATURES.iter().find(|entry| {
        entry
            .id
            .strip_prefix("minecraft:")
            .is_some_and(|entry_name| entry_name == name)
    })
}

pub fn placed_feature_source(id: &str) -> Option<PlacedFeatureSource> {
    let name = id.strip_prefix("minecraft:").unwrap_or(id);
    PLACED_FEATURE_BOOTSTRAP_SOURCES
        .iter()
        .find(|entry| {
            entry.keys.iter().any(|key| {
                key.strip_prefix("minecraft:")
                    .is_some_and(|entry_name| entry_name == name)
            })
        })
        .map(|entry| entry.source)
}

fn ore_count(count: i32) -> PlacementModifier {
    PlacementModifier::Count { count }
}

fn ore_uniform_count(
    min_inclusive: i32,
    max_inclusive: i32,
    sampled_count: i32,
) -> PlacementModifier {
    PlacementModifier::CountProvider {
        provider: IntProviderModel::Uniform {
            min_inclusive,
            max_inclusive,
        },
        sampled_count,
    }
}

fn ore_rarity(chance: i32) -> PlacementModifier {
    PlacementModifier::RarityFilter { chance }
}

fn ore_uniform(min_inclusive: VerticalAnchor, max_inclusive: VerticalAnchor) -> PlacementModifier {
    PlacementModifier::HeightRange {
        height: HeightProvider::Uniform {
            min_inclusive,
            max_inclusive,
        },
    }
}

fn ore_triangle(min_inclusive: VerticalAnchor, max_inclusive: VerticalAnchor) -> PlacementModifier {
    PlacementModifier::HeightRange {
        height: HeightProvider::Trapezoid {
            min_inclusive,
            max_inclusive,
            plateau: 0,
        },
    }
}

pub fn placed_ore_feature(id: &str) -> Option<PlacedOreFeatureModel> {
    let name = id.strip_prefix("minecraft:").unwrap_or(id);
    let model = match name {
        "ore_magma" => PlacedOreFeatureModel {
            configured_feature: "minecraft:ore_magma",
            placement: vec![
                ore_count(4),
                PlacementModifier::InSquare,
                ore_uniform(VerticalAnchor::Absolute(27), VerticalAnchor::Absolute(36)),
                PlacementModifier::BiomeFilter,
            ],
        },
        "ore_soul_sand" => PlacedOreFeatureModel {
            configured_feature: "minecraft:ore_soul_sand",
            placement: vec![
                ore_count(12),
                PlacementModifier::InSquare,
                ore_uniform(VerticalAnchor::AboveBottom(0), VerticalAnchor::Absolute(31)),
                PlacementModifier::BiomeFilter,
            ],
        },
        "ore_gold_deltas" => PlacedOreFeatureModel {
            configured_feature: "minecraft:ore_nether_gold",
            placement: vec![
                ore_count(20),
                PlacementModifier::InSquare,
                ore_uniform(
                    VerticalAnchor::AboveBottom(10),
                    VerticalAnchor::BelowTop(10),
                ),
                PlacementModifier::BiomeFilter,
            ],
        },
        "ore_quartz_deltas" => PlacedOreFeatureModel {
            configured_feature: "minecraft:ore_quartz",
            placement: vec![
                ore_count(32),
                PlacementModifier::InSquare,
                ore_uniform(
                    VerticalAnchor::AboveBottom(10),
                    VerticalAnchor::BelowTop(10),
                ),
                PlacementModifier::BiomeFilter,
            ],
        },
        "ore_gold_nether" => PlacedOreFeatureModel {
            configured_feature: "minecraft:ore_nether_gold",
            placement: vec![
                ore_count(10),
                PlacementModifier::InSquare,
                ore_uniform(
                    VerticalAnchor::AboveBottom(10),
                    VerticalAnchor::BelowTop(10),
                ),
                PlacementModifier::BiomeFilter,
            ],
        },
        "ore_quartz_nether" => PlacedOreFeatureModel {
            configured_feature: "minecraft:ore_quartz",
            placement: vec![
                ore_count(16),
                PlacementModifier::InSquare,
                ore_uniform(
                    VerticalAnchor::AboveBottom(10),
                    VerticalAnchor::BelowTop(10),
                ),
                PlacementModifier::BiomeFilter,
            ],
        },
        "ore_gravel_nether" => PlacedOreFeatureModel {
            configured_feature: "minecraft:ore_gravel_nether",
            placement: vec![
                ore_count(2),
                PlacementModifier::InSquare,
                ore_uniform(VerticalAnchor::Absolute(5), VerticalAnchor::Absolute(41)),
                PlacementModifier::BiomeFilter,
            ],
        },
        "ore_blackstone" => PlacedOreFeatureModel {
            configured_feature: "minecraft:ore_blackstone",
            placement: vec![
                ore_count(2),
                PlacementModifier::InSquare,
                ore_uniform(VerticalAnchor::Absolute(5), VerticalAnchor::Absolute(31)),
                PlacementModifier::BiomeFilter,
            ],
        },
        "ore_dirt" => PlacedOreFeatureModel {
            configured_feature: "minecraft:ore_dirt",
            placement: vec![
                ore_count(7),
                PlacementModifier::InSquare,
                ore_uniform(VerticalAnchor::Absolute(0), VerticalAnchor::Absolute(160)),
                PlacementModifier::BiomeFilter,
            ],
        },
        "ore_gravel" => PlacedOreFeatureModel {
            configured_feature: "minecraft:ore_gravel",
            placement: vec![
                ore_count(14),
                PlacementModifier::InSquare,
                ore_uniform(VerticalAnchor::AboveBottom(0), VerticalAnchor::BelowTop(0)),
                PlacementModifier::BiomeFilter,
            ],
        },
        "ore_granite_upper" | "ore_diorite_upper" | "ore_andesite_upper" => {
            let configured_feature = match name {
                "ore_granite_upper" => "minecraft:ore_granite",
                "ore_diorite_upper" => "minecraft:ore_diorite",
                _ => "minecraft:ore_andesite",
            };
            PlacedOreFeatureModel {
                configured_feature,
                placement: vec![
                    ore_rarity(6),
                    PlacementModifier::InSquare,
                    ore_uniform(VerticalAnchor::Absolute(64), VerticalAnchor::Absolute(128)),
                    PlacementModifier::BiomeFilter,
                ],
            }
        }
        "ore_granite_lower" | "ore_diorite_lower" | "ore_andesite_lower" => {
            let configured_feature = match name {
                "ore_granite_lower" => "minecraft:ore_granite",
                "ore_diorite_lower" => "minecraft:ore_diorite",
                _ => "minecraft:ore_andesite",
            };
            PlacedOreFeatureModel {
                configured_feature,
                placement: vec![
                    ore_count(2),
                    PlacementModifier::InSquare,
                    ore_uniform(VerticalAnchor::Absolute(0), VerticalAnchor::Absolute(60)),
                    PlacementModifier::BiomeFilter,
                ],
            }
        }
        "ore_tuff" => PlacedOreFeatureModel {
            configured_feature: "minecraft:ore_tuff",
            placement: vec![
                ore_count(2),
                PlacementModifier::InSquare,
                ore_uniform(VerticalAnchor::AboveBottom(0), VerticalAnchor::Absolute(0)),
                PlacementModifier::BiomeFilter,
            ],
        },
        "ore_coal_upper" => PlacedOreFeatureModel {
            configured_feature: "minecraft:ore_coal",
            placement: vec![
                ore_count(30),
                PlacementModifier::InSquare,
                ore_uniform(VerticalAnchor::Absolute(136), VerticalAnchor::BelowTop(0)),
                PlacementModifier::BiomeFilter,
            ],
        },
        "ore_coal_lower" => PlacedOreFeatureModel {
            configured_feature: "minecraft:ore_coal_buried",
            placement: vec![
                ore_count(20),
                PlacementModifier::InSquare,
                ore_triangle(VerticalAnchor::Absolute(0), VerticalAnchor::Absolute(192)),
                PlacementModifier::BiomeFilter,
            ],
        },
        "ore_iron_upper" => PlacedOreFeatureModel {
            configured_feature: "minecraft:ore_iron",
            placement: vec![
                ore_count(90),
                PlacementModifier::InSquare,
                ore_triangle(VerticalAnchor::Absolute(80), VerticalAnchor::Absolute(384)),
                PlacementModifier::BiomeFilter,
            ],
        },
        "ore_iron_middle" => PlacedOreFeatureModel {
            configured_feature: "minecraft:ore_iron",
            placement: vec![
                ore_count(10),
                PlacementModifier::InSquare,
                ore_triangle(VerticalAnchor::Absolute(-24), VerticalAnchor::Absolute(56)),
                PlacementModifier::BiomeFilter,
            ],
        },
        "ore_iron_small" => PlacedOreFeatureModel {
            configured_feature: "minecraft:ore_iron_small",
            placement: vec![
                ore_count(10),
                PlacementModifier::InSquare,
                ore_uniform(VerticalAnchor::AboveBottom(0), VerticalAnchor::Absolute(72)),
                PlacementModifier::BiomeFilter,
            ],
        },
        "ore_gold_extra" => PlacedOreFeatureModel {
            configured_feature: "minecraft:ore_gold",
            placement: vec![
                ore_count(50),
                PlacementModifier::InSquare,
                ore_uniform(VerticalAnchor::Absolute(32), VerticalAnchor::Absolute(256)),
                PlacementModifier::BiomeFilter,
            ],
        },
        "ore_gold" => PlacedOreFeatureModel {
            configured_feature: "minecraft:ore_gold_buried",
            placement: vec![
                ore_count(4),
                PlacementModifier::InSquare,
                ore_triangle(VerticalAnchor::Absolute(-64), VerticalAnchor::Absolute(32)),
                PlacementModifier::BiomeFilter,
            ],
        },
        "ore_gold_lower" => PlacedOreFeatureModel {
            configured_feature: "minecraft:ore_gold_buried",
            placement: vec![
                ore_uniform_count(0, 1, 0),
                PlacementModifier::InSquare,
                ore_uniform(VerticalAnchor::Absolute(-64), VerticalAnchor::Absolute(-48)),
                PlacementModifier::BiomeFilter,
            ],
        },
        "ore_redstone" => PlacedOreFeatureModel {
            configured_feature: "minecraft:ore_redstone",
            placement: vec![
                ore_count(4),
                PlacementModifier::InSquare,
                ore_uniform(VerticalAnchor::AboveBottom(0), VerticalAnchor::Absolute(15)),
                PlacementModifier::BiomeFilter,
            ],
        },
        "ore_redstone_lower" => PlacedOreFeatureModel {
            configured_feature: "minecraft:ore_redstone",
            placement: vec![
                ore_count(8),
                PlacementModifier::InSquare,
                ore_triangle(
                    VerticalAnchor::AboveBottom(-32),
                    VerticalAnchor::AboveBottom(32),
                ),
                PlacementModifier::BiomeFilter,
            ],
        },
        "ore_diamond" | "ore_diamond_large" | "ore_diamond_buried" => {
            let (configured_feature, frequency) = match name {
                "ore_diamond" => ("minecraft:ore_diamond_small", ore_count(7)),
                "ore_diamond_large" => ("minecraft:ore_diamond_large", ore_rarity(9)),
                _ => ("minecraft:ore_diamond_buried", ore_count(4)),
            };
            PlacedOreFeatureModel {
                configured_feature,
                placement: vec![
                    frequency,
                    PlacementModifier::InSquare,
                    ore_triangle(
                        VerticalAnchor::AboveBottom(-80),
                        VerticalAnchor::AboveBottom(80),
                    ),
                    PlacementModifier::BiomeFilter,
                ],
            }
        }
        "ore_diamond_medium" => PlacedOreFeatureModel {
            configured_feature: "minecraft:ore_diamond_medium",
            placement: vec![
                ore_count(2),
                PlacementModifier::InSquare,
                ore_uniform(VerticalAnchor::Absolute(-64), VerticalAnchor::Absolute(-4)),
                PlacementModifier::BiomeFilter,
            ],
        },
        "ore_lapis" => PlacedOreFeatureModel {
            configured_feature: "minecraft:ore_lapis",
            placement: vec![
                ore_count(2),
                PlacementModifier::InSquare,
                ore_triangle(VerticalAnchor::Absolute(-32), VerticalAnchor::Absolute(32)),
                PlacementModifier::BiomeFilter,
            ],
        },
        "ore_lapis_buried" => PlacedOreFeatureModel {
            configured_feature: "minecraft:ore_lapis_buried",
            placement: vec![
                ore_count(4),
                PlacementModifier::InSquare,
                ore_uniform(VerticalAnchor::AboveBottom(0), VerticalAnchor::Absolute(64)),
                PlacementModifier::BiomeFilter,
            ],
        },
        "ore_infested" => PlacedOreFeatureModel {
            configured_feature: "minecraft:ore_infested",
            placement: vec![
                ore_count(14),
                PlacementModifier::InSquare,
                ore_uniform(VerticalAnchor::AboveBottom(0), VerticalAnchor::Absolute(63)),
                PlacementModifier::BiomeFilter,
            ],
        },
        "ore_emerald" => PlacedOreFeatureModel {
            configured_feature: "minecraft:ore_emerald",
            placement: vec![
                ore_count(100),
                PlacementModifier::InSquare,
                ore_triangle(VerticalAnchor::Absolute(-16), VerticalAnchor::Absolute(480)),
                PlacementModifier::BiomeFilter,
            ],
        },
        "ore_ancient_debris_large" => PlacedOreFeatureModel {
            configured_feature: "minecraft:ore_ancient_debris_large",
            placement: vec![
                PlacementModifier::InSquare,
                ore_triangle(VerticalAnchor::Absolute(8), VerticalAnchor::Absolute(24)),
                PlacementModifier::BiomeFilter,
            ],
        },
        "ore_debris_small" => PlacedOreFeatureModel {
            configured_feature: "minecraft:ore_ancient_debris_small",
            placement: vec![
                PlacementModifier::InSquare,
                ore_uniform(VerticalAnchor::AboveBottom(8), VerticalAnchor::BelowTop(8)),
                PlacementModifier::BiomeFilter,
            ],
        },
        "ore_copper" => PlacedOreFeatureModel {
            configured_feature: "minecraft:ore_copper_small",
            placement: vec![
                ore_count(16),
                PlacementModifier::InSquare,
                ore_triangle(VerticalAnchor::Absolute(-16), VerticalAnchor::Absolute(112)),
                PlacementModifier::BiomeFilter,
            ],
        },
        "ore_copper_large" => PlacedOreFeatureModel {
            configured_feature: "minecraft:ore_copper_large",
            placement: vec![
                ore_count(16),
                PlacementModifier::InSquare,
                ore_triangle(VerticalAnchor::Absolute(-16), VerticalAnchor::Absolute(112)),
                PlacementModifier::BiomeFilter,
            ],
        },
        "ore_clay" => PlacedOreFeatureModel {
            configured_feature: "minecraft:ore_clay",
            placement: vec![
                ore_count(46),
                PlacementModifier::InSquare,
                ore_uniform(
                    VerticalAnchor::AboveBottom(0),
                    VerticalAnchor::Absolute(256),
                ),
                PlacementModifier::BiomeFilter,
            ],
        },
        _ => return None,
    };
    Some(model)
}

pub fn placed_disk_feature(id: &str) -> Option<PlacedDiskFeatureModel> {
    let name = id.strip_prefix("minecraft:").unwrap_or(id);
    let model = match name {
        "disk_clay" => PlacedDiskFeatureModel {
            configured_feature: "minecraft:disk_clay",
            placement: vec![
                PlacementModifier::InSquare,
                PlacementModifier::Heightmap {
                    heightmap: HeightmapKind::OceanFloorWg,
                },
                PlacementModifier::BlockPredicateFilter {
                    predicate: BlockPredicate::MatchingFluids {
                        fluids: &["minecraft:water"],
                    },
                },
                PlacementModifier::BiomeFilter,
            ],
        },
        "disk_gravel" => PlacedDiskFeatureModel {
            configured_feature: "minecraft:disk_gravel",
            placement: vec![
                PlacementModifier::InSquare,
                PlacementModifier::Heightmap {
                    heightmap: HeightmapKind::OceanFloorWg,
                },
                PlacementModifier::BlockPredicateFilter {
                    predicate: BlockPredicate::MatchingFluids {
                        fluids: &["minecraft:water"],
                    },
                },
                PlacementModifier::BiomeFilter,
            ],
        },
        "disk_sand" => PlacedDiskFeatureModel {
            configured_feature: "minecraft:disk_sand",
            placement: vec![
                PlacementModifier::Count { count: 3 },
                PlacementModifier::InSquare,
                PlacementModifier::Heightmap {
                    heightmap: HeightmapKind::OceanFloorWg,
                },
                PlacementModifier::BlockPredicateFilter {
                    predicate: BlockPredicate::MatchingFluids {
                        fluids: &["minecraft:water"],
                    },
                },
                PlacementModifier::BiomeFilter,
            ],
        },
        _ => return None,
    };
    Some(model)
}

pub fn configured_disk_configuration(id: &str) -> Option<DiskConfigurationModel> {
    let name = id.strip_prefix("minecraft:").unwrap_or(id);
    match name {
        "disk_clay" => Some(DiskConfigurationModel {
            state_provider: BlockStateProviderModel::Simple("minecraft:clay"),
            target: BlockPredicate::MatchingBlocks {
                blocks: &["minecraft:dirt", "minecraft:clay"],
            },
            radius: IntProviderModel::Uniform {
                min_inclusive: 2,
                max_inclusive: 3,
            },
            half_height: 1,
        }),
        "disk_gravel" => Some(DiskConfigurationModel {
            state_provider: BlockStateProviderModel::Simple("minecraft:gravel"),
            target: BlockPredicate::MatchingBlocks {
                blocks: &["minecraft:dirt", "minecraft:grass_block"],
            },
            radius: IntProviderModel::Uniform {
                min_inclusive: 2,
                max_inclusive: 5,
            },
            half_height: 2,
        }),
        "disk_sand" => Some(DiskConfigurationModel {
            state_provider: BlockStateProviderModel::RuleBased {
                fallback: Some(Box::new(BlockStateProviderModel::Simple("minecraft:sand"))),
                rules: vec![RuleBasedBlockStateProviderRule {
                    if_true: BlockPredicate::MatchingBlocksAt {
                        offset_y: -1,
                        blocks: &["minecraft:air"],
                    },
                    then: Box::new(BlockStateProviderModel::Simple("minecraft:sandstone")),
                }],
            },
            target: BlockPredicate::MatchingBlocks {
                blocks: &["minecraft:dirt", "minecraft:grass_block"],
            },
            radius: IntProviderModel::Uniform {
                min_inclusive: 2,
                max_inclusive: 6,
            },
            half_height: 2,
        }),
        _ => None,
    }
}
