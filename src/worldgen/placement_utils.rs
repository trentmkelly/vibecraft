use super::*;

const AIR_BLOCKS: &[&str] = &["minecraft:air"];

pub const PLACEMENT_UTILS_HEIGHTMAP: PlacementModifier = PlacementModifier::Heightmap {
    heightmap: HeightmapKind::MotionBlocking,
};
pub const PLACEMENT_UTILS_HEIGHTMAP_NO_LEAVES: PlacementModifier = PlacementModifier::Heightmap {
    heightmap: HeightmapKind::MotionBlockingNoLeaves,
};
pub const PLACEMENT_UTILS_HEIGHTMAP_TOP_SOLID: PlacementModifier = PlacementModifier::Heightmap {
    heightmap: HeightmapKind::OceanFloorWg,
};
pub const PLACEMENT_UTILS_HEIGHTMAP_WORLD_SURFACE: PlacementModifier =
    PlacementModifier::Heightmap {
        heightmap: HeightmapKind::WorldSurfaceWg,
    };
pub const PLACEMENT_UTILS_HEIGHTMAP_OCEAN_FLOOR: PlacementModifier = PlacementModifier::Heightmap {
    heightmap: HeightmapKind::OceanFloor,
};
pub const PLACEMENT_UTILS_FULL_RANGE: PlacementModifier = PlacementModifier::HeightRange {
    height: HeightProvider::Uniform {
        min_inclusive: VerticalAnchor::AboveBottom(0),
        max_inclusive: VerticalAnchor::BelowTop(0),
    },
};
pub const PLACEMENT_UTILS_RANGE_10_10: PlacementModifier = PlacementModifier::HeightRange {
    height: HeightProvider::Uniform {
        min_inclusive: VerticalAnchor::AboveBottom(10),
        max_inclusive: VerticalAnchor::BelowTop(10),
    },
};
pub const PLACEMENT_UTILS_RANGE_8_8: PlacementModifier = PlacementModifier::HeightRange {
    height: HeightProvider::Uniform {
        min_inclusive: VerticalAnchor::AboveBottom(8),
        max_inclusive: VerticalAnchor::BelowTop(8),
    },
};
pub const PLACEMENT_UTILS_RANGE_4_4: PlacementModifier = PlacementModifier::HeightRange {
    height: HeightProvider::Uniform {
        min_inclusive: VerticalAnchor::AboveBottom(4),
        max_inclusive: VerticalAnchor::BelowTop(4),
    },
};
pub const PLACEMENT_UTILS_RANGE_BOTTOM_TO_MAX_TERRAIN_HEIGHT: PlacementModifier =
    PlacementModifier::HeightRange {
        height: HeightProvider::Uniform {
            min_inclusive: VerticalAnchor::AboveBottom(0),
            max_inclusive: VerticalAnchor::Absolute(256),
        },
    };

pub const PLACEMENT_UTILS_BOOTSTRAP_ORDER: &[PlacedFeatureSource] = &[
    PlacedFeatureSource::Aquatic,
    PlacedFeatureSource::Cave,
    PlacedFeatureSource::End,
    PlacedFeatureSource::MiscOverworld,
    PlacedFeatureSource::Nether,
    PlacedFeatureSource::Ore,
    PlacedFeatureSource::Tree,
    PlacedFeatureSource::Vegetation,
    PlacedFeatureSource::Village,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlacementUtilsCountExtra {
    pub base_count: i32,
    pub base_weight: i32,
    pub extra_count: i32,
    pub extra_weight: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct InlinePlacedFeatureModel {
    pub configured_feature: &'static str,
    pub placement: Vec<PlacementModifier>,
}

pub fn placement_utils_create_key(name: &str) -> String {
    let key = name.strip_prefix("minecraft:").unwrap_or(name);
    format!("minecraft:{key}")
}

pub fn placement_utils_count_extra(
    count: i32,
    chance: f32,
    extra: i32,
) -> Result<PlacementUtilsCountExtra, &'static str> {
    let weight = 1.0 / chance;
    let whole_weight = weight as i32;
    if (weight - whole_weight as f32).abs() > 1.0e-5 {
        return Err("Chance data cannot be represented as list weight");
    }

    Ok(PlacementUtilsCountExtra {
        base_count: count,
        base_weight: whole_weight - 1,
        extra_count: count + extra,
        extra_weight: 1,
    })
}

pub fn placement_utils_is_empty() -> PlacementModifier {
    PlacementModifier::BlockPredicateFilter {
        predicate: BlockPredicate::MatchingBlocks { blocks: AIR_BLOCKS },
    }
}

pub fn placement_utils_filtered_by_block_survival(
    default_state: &'static str,
    survives: bool,
) -> PlacementModifier {
    PlacementModifier::BlockPredicateFilter {
        predicate: BlockPredicate::WouldSurvive {
            offset_y: 0,
            state: default_state,
            survives,
        },
    }
}

pub fn placement_utils_inline_placed(
    configured_feature: &'static str,
    placement: &[PlacementModifier],
) -> InlinePlacedFeatureModel {
    InlinePlacedFeatureModel {
        configured_feature,
        placement: placement.to_vec(),
    }
}

pub fn placement_utils_only_when_empty(
    configured_feature: &'static str,
) -> InlinePlacedFeatureModel {
    placement_utils_filtered(
        configured_feature,
        BlockPredicate::MatchingBlocks { blocks: AIR_BLOCKS },
    )
}

pub fn placement_utils_filtered(
    configured_feature: &'static str,
    predicate: BlockPredicate,
) -> InlinePlacedFeatureModel {
    placement_utils_inline_placed(
        configured_feature,
        &[PlacementModifier::BlockPredicateFilter { predicate }],
    )
}
