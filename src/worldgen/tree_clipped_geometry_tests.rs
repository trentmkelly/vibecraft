use super::*;

fn config(
    foliage_kind: FoliagePlacerKind,
    min_clipped_height: Option<i32>,
) -> TreeConfigurationModel {
    TreeConfigurationModel {
        trunk_provider: BlockStateProviderModel::Simple("minecraft:oak_log"),
        foliage_provider: BlockStateProviderModel::Simple("minecraft:oak_leaves"),
        dirt_provider: BlockStateProviderModel::Simple("minecraft:dirt"),
        trunk_placer: TrunkPlacerModel {
            base_height: 8,
            height_rand_a: 0,
            height_rand_b: 0,
            kind: TrunkPlacerKind::Straight,
        },
        foliage_placer: FoliagePlacerModel {
            radius_min: 2,
            radius_max: 2,
            offset_min: 1,
            offset_max: 1,
            kind: foliage_kind,
        },
        minimum_size: FeatureSizeModel::TwoLayers {
            limit: 1,
            lower_size: 0,
            upper_size: 0,
            min_clipped_height,
        },
        root_placer: None,
        decorators: Vec::new(),
        ignore_vines: false,
    }
}

fn plan(
    origin: BlockPos,
    config: &TreeConfigurationModel,
    blocked_row: Option<usize>,
    build_max_y: i32,
) -> Option<TreePlacementPlan> {
    let mut rows = vec![vec!["minecraft:air"]; 10];
    if let Some(row) = blocked_row {
        rows[row][0] = "minecraft:stone";
    }
    let row_refs: Vec<&[&str]> = rows.iter().map(Vec::as_slice).collect();
    configured_tree_placement_plan(origin, config, -64, build_max_y, &row_refs, 0, 0)
        .expect("valid fixture configuration")
}

fn origin() -> BlockPos {
    BlockPos { x: 4, y: 20, z: -7 }
}

#[test]
fn configured_clipped_tree_places_shortened_trunk_and_translated_crown() {
    // Java getMaxFreeTreeHeight returns obstructionY - 2: row 5 accepts height 3.
    let config = config(FoliagePlacerKind::Blob { height: 2 }, Some(3));
    let clipped = plan(origin(), &config, Some(5), 320).expect("clipping is allowed");
    let full = plan(origin(), &config, None, 320).expect("unblocked tree fits");
    let log_y: Vec<i32> = clipped
        .blocks
        .iter()
        .filter(|block| block.kind == TreePlacementBlockKind::Log)
        .map(|block| block.pos.y)
        .collect();
    assert_eq!(log_y, vec![20, 21, 22]);

    let clipped_leaves: Vec<BlockPos> = clipped
        .blocks
        .iter()
        .filter(|block| block.kind == TreePlacementBlockKind::Leaves)
        .map(|block| block.pos)
        .collect();
    let translated_full_leaves: Vec<BlockPos> = full
        .blocks
        .iter()
        .filter(|block| block.kind == TreePlacementBlockKind::Leaves)
        .map(|block| BlockPos {
            y: block.pos.y - 5,
            ..block.pos
        })
        .collect();
    assert!(!clipped_leaves.is_empty());
    assert_eq!(clipped_leaves, translated_full_leaves);
    assert_eq!(clipped_leaves.iter().map(|pos| pos.y).max(), Some(24));
    assert!(clipped
        .blocks
        .iter()
        .any(|block| block.kind == TreePlacementBlockKind::DirtBelowTrunk && block.pos.y == 19));
}

#[test]
fn configured_clipped_spruce_preserves_original_height_foliage_sampling() {
    let config = config(
        FoliagePlacerKind::Spruce {
            height_min: 2,
            height_max: 2,
        },
        Some(3),
    );
    let clipped = plan(origin(), &config, Some(5), 320).expect("clipping is allowed");
    // Spruce samples max(4, originalHeight - trunkHeight) = 6 before clipping.
    // The attachment is origin + 3, so the bottom row is origin + 3 - 6.
    let leaf_min_y = clipped
        .blocks
        .iter()
        .filter(|block| block.kind == TreePlacementBlockKind::Leaves)
        .map(|block| block.pos.y)
        .min();
    assert_eq!(leaf_min_y, Some(17));
}

#[test]
fn configured_clipped_tree_rejects_missing_or_unmet_minimum() {
    for minimum in [None, Some(4)] {
        let config = config(FoliagePlacerKind::Blob { height: 2 }, minimum);
        assert!(plan(origin(), &config, Some(5), 320).is_none());
    }
}

#[test]
fn configured_clipped_tree_allows_zero_height_when_minimum_is_zero() {
    // The feature-size codec permits min_clipped_height = 0, and TreeFeature
    // accepts a clipped height equal to that minimum. StraightTrunkPlacer still
    // places the below-trunk block and returns a foliage attachment at origin.
    let zero_minimum_config = config(FoliagePlacerKind::Blob { height: 2 }, Some(0));
    let clipped = plan(origin(), &zero_minimum_config, Some(2), 320)
        .expect("zero meets the configured minimum");

    assert!(!clipped
        .blocks
        .iter()
        .any(|block| block.kind == TreePlacementBlockKind::Log));
    assert!(clipped
        .blocks
        .iter()
        .any(|block| block.kind == TreePlacementBlockKind::DirtBelowTrunk && block.pos.y == 19));
    assert!(
        clipped
            .blocks
            .iter()
            .filter(|block| block.kind == TreePlacementBlockKind::Leaves)
            .count()
            > 0
    );

    // The same obstruction is rejected if the feature requires even one trunk
    // block; this distinguishes an actual zero-height acceptance from a failed
    // clearance scan accidentally represented as zero.
    let positive_minimum = config(FoliagePlacerKind::Blob { height: 2 }, Some(1));
    assert!(plan(origin(), &positive_minimum, Some(2), 320).is_none());
}

#[test]
fn configured_clipped_tree_keeps_original_height_build_bounds() {
    let config = config(FoliagePlacerKind::Blob { height: 2 }, Some(3));
    // A height-3 trunk would fit, but Java rejects the original height-8 tree first.
    assert!(plan(BlockPos { y: 60, ..origin() }, &config, Some(5), 64).is_none());
}

#[test]
fn configured_unblocked_tree_matches_simple_tree_geometry() {
    let config = config(FoliagePlacerKind::Blob { height: 2 }, Some(3));
    let configured = plan(origin(), &config, None, 320).expect("unblocked tree fits");
    let simple = simple_tree_placement_plan(SimpleTreePlacementInput {
        origin: origin(),
        trunk: config.trunk_placer,
        foliage: config.foliage_placer,
        trunk_state: "minecraft:oak_log",
        foliage_state: "minecraft:oak_leaves",
        below_trunk_state: "minecraft:dirt",
        rand_a: 0,
        rand_b: 0,
    })
    .expect("valid straight tree");
    assert_eq!(configured, simple);
    assert_eq!(
        configured
            .blocks
            .iter()
            .filter(|block| block.kind == TreePlacementBlockKind::Log)
            .count(),
        8
    );
}
