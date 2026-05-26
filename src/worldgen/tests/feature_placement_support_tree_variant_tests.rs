use super::super::*;

macro_rules! simple_tree_plan {
    (
        $origin:expr,
        $trunk:expr,
        $foliage:expr,
        $trunk_state:expr,
        $foliage_state:expr,
        $below_trunk_state:expr,
        $rand_a:expr,
        $rand_b:expr $(,)?
    ) => {
        super::super::simple_tree_placement_plan(SimpleTreePlacementInput {
            origin: $origin,
            trunk: $trunk,
            foliage: $foliage,
            trunk_state: $trunk_state,
            foliage_state: $foliage_state,
            below_trunk_state: $below_trunk_state,
            rand_a: $rand_a,
            rand_b: $rand_b,
        })
    };
}

pub(super) fn assert_tree_variant_and_trunk_support() {
    assert_bush_foliage_support();
    assert_acacia_foliage_support();
    assert_dark_oak_foliage_support();
    assert_fancy_foliage_support();
    assert_mega_jungle_foliage_support();
    assert_random_spread_foliage_support();
    assert_cherry_foliage_support();
    assert_pine_foliage_support();
    assert_spruce_foliage_support();
    assert_tree_variant_large_tree_support();
}

fn assert_bush_foliage_support() {
    let bush_plan = simple_tree_plan!(
        BlockPos {
            x: 20,
            y: 64,
            z: 20,
        },
        TrunkPlacerModel {
            base_height: 3,
            height_rand_a: 0,
            height_rand_b: 0,
            kind: TrunkPlacerKind::Straight,
        },
        FoliagePlacerModel {
            radius_min: 2,
            radius_max: 2,
            offset_min: 1,
            offset_max: 1,
            kind: FoliagePlacerKind::Bush { height: 2 },
        },
        "minecraft:oak_log",
        "minecraft:oak_leaves",
        "minecraft:dirt",
        2,
        4,
    )
    .unwrap();
    assert!(bush_plan.blocks.iter().any(|block| {
        block.kind == TreePlacementBlockKind::Leaves
            && block.pos
                == BlockPos {
                    x: 20,
                    y: 68,
                    z: 20,
                }
    }));
    assert!(!bush_plan.blocks.iter().any(|block| {
        block.kind == TreePlacementBlockKind::Leaves
            && block.pos
                == BlockPos {
                    x: 21,
                    y: 68,
                    z: 20,
                }
    }));
    assert!(bush_plan.blocks.iter().any(|block| {
        block.kind == TreePlacementBlockKind::Leaves
            && block.pos
                == BlockPos {
                    x: 21,
                    y: 67,
                    z: 20,
                }
    }));
    assert!(!bush_plan.blocks.iter().any(|block| {
        block.kind == TreePlacementBlockKind::Leaves
            && block.pos
                == BlockPos {
                    x: 22,
                    y: 67,
                    z: 20,
                }
    }));
    assert!(bush_plan.blocks.iter().any(|block| {
        block.kind == TreePlacementBlockKind::Leaves
            && block.pos
                == BlockPos {
                    x: 22,
                    y: 66,
                    z: 20,
                }
    }));
}

fn assert_acacia_foliage_support() {
    let acacia_plan = simple_tree_plan!(
        BlockPos {
            x: 40,
            y: 64,
            z: 40,
        },
        TrunkPlacerModel {
            base_height: 4,
            height_rand_a: 0,
            height_rand_b: 0,
            kind: TrunkPlacerKind::Straight,
        },
        FoliagePlacerModel {
            radius_min: 2,
            radius_max: 2,
            offset_min: 0,
            offset_max: 0,
            kind: FoliagePlacerKind::Acacia,
        },
        "minecraft:acacia_log",
        "minecraft:acacia_leaves",
        "minecraft:dirt",
        0,
        0,
    )
    .unwrap();
    assert!(acacia_plan.blocks.iter().any(|block| {
        block.kind == TreePlacementBlockKind::Leaves
            && block.pos
                == BlockPos {
                    x: 42,
                    y: 67,
                    z: 40,
                }
    }));
    assert!(!acacia_plan.blocks.iter().any(|block| {
        block.kind == TreePlacementBlockKind::Leaves
            && block.pos
                == BlockPos {
                    x: 42,
                    y: 67,
                    z: 42,
                }
    }));
    assert!(acacia_plan.blocks.iter().any(|block| {
        block.kind == TreePlacementBlockKind::Leaves
            && block.pos
                == BlockPos {
                    x: 41,
                    y: 68,
                    z: 41,
                }
    }));
    assert!(acacia_plan.blocks.iter().any(|block| {
        block.kind == TreePlacementBlockKind::Leaves
            && block.pos
                == BlockPos {
                    x: 41,
                    y: 68,
                    z: 40,
                }
    }));
}

fn assert_dark_oak_foliage_support() {
    let dark_oak_plan = simple_tree_plan!(
        BlockPos {
            x: 60,
            y: 64,
            z: 60,
        },
        TrunkPlacerModel {
            base_height: 4,
            height_rand_a: 0,
            height_rand_b: 0,
            kind: TrunkPlacerKind::Straight,
        },
        FoliagePlacerModel {
            radius_min: 2,
            radius_max: 2,
            offset_min: 0,
            offset_max: 0,
            kind: FoliagePlacerKind::DarkOak,
        },
        "minecraft:dark_oak_log",
        "minecraft:dark_oak_leaves",
        "minecraft:dirt",
        0,
        0,
    )
    .unwrap();
    assert!(dark_oak_plan.blocks.iter().any(|block| {
        block.kind == TreePlacementBlockKind::Leaves
            && block.pos
                == BlockPos {
                    x: 64,
                    y: 67,
                    z: 60,
                }
    }));
    assert!(!dark_oak_plan.blocks.iter().any(|block| {
        block.kind == TreePlacementBlockKind::Leaves
            && block.pos
                == BlockPos {
                    x: 64,
                    y: 67,
                    z: 64,
                }
    }));
    assert!(dark_oak_plan.blocks.iter().any(|block| {
        block.kind == TreePlacementBlockKind::Leaves
            && block.pos
                == BlockPos {
                    x: 63,
                    y: 68,
                    z: 63,
                }
    }));
}

fn assert_fancy_foliage_support() {
    let fancy_plan = simple_tree_plan!(
        BlockPos {
            x: 70,
            y: 64,
            z: 70,
        },
        TrunkPlacerModel {
            base_height: 4,
            height_rand_a: 0,
            height_rand_b: 0,
            kind: TrunkPlacerKind::Straight,
        },
        FoliagePlacerModel {
            radius_min: 2,
            radius_max: 2,
            offset_min: 0,
            offset_max: 0,
            kind: FoliagePlacerKind::Fancy { height: 2 },
        },
        "minecraft:oak_log",
        "minecraft:oak_leaves",
        "minecraft:dirt",
        0,
        0,
    )
    .unwrap();
    assert_eq!(
        super::super::fancy_foliage_rows(0, 2, 2),
        vec![(0, 2), (-1, 3), (-2, 2)]
    );
    assert!(fancy_plan.blocks.iter().any(|block| {
        block.kind == TreePlacementBlockKind::Leaves
            && block.pos
                == BlockPos {
                    x: 72,
                    y: 67,
                    z: 70,
                }
    }));
    assert!(!fancy_plan.blocks.iter().any(|block| {
        block.kind == TreePlacementBlockKind::Leaves
            && block.pos
                == BlockPos {
                    x: 73,
                    y: 67,
                    z: 70,
                }
    }));
    assert!(!fancy_plan.blocks.iter().any(|block| {
        block.kind == TreePlacementBlockKind::Leaves
            && block.pos
                == BlockPos {
                    x: 68,
                    y: 68,
                    z: 70,
                }
    }));
    assert!(!fancy_plan.blocks.iter().any(|block| {
        block.kind == TreePlacementBlockKind::Leaves
            && block.pos
                == BlockPos {
                    x: 72,
                    y: 68,
                    z: 70,
                }
    }));
    assert!(super::super::fancy_leaves_row_should_skip(-2, 0, 2));
    assert!(super::super::fancy_leaves_row_should_skip(2, 0, 2));
}

fn assert_mega_jungle_foliage_support() {
    let mega_jungle_plan = simple_tree_plan!(
        BlockPos {
            x: 140,
            y: 64,
            z: 140,
        },
        TrunkPlacerModel {
            base_height: 4,
            height_rand_a: 0,
            height_rand_b: 0,
            kind: TrunkPlacerKind::Straight,
        },
        FoliagePlacerModel {
            radius_min: 2,
            radius_max: 2,
            offset_min: 0,
            offset_max: 0,
            kind: FoliagePlacerKind::Jungle { height: 4 },
        },
        "minecraft:jungle_log",
        "minecraft:jungle_leaves",
        "minecraft:dirt",
        0,
        1,
    )
    .unwrap();
    assert_eq!(
        super::super::mega_jungle_foliage_rows(0, 2, 2),
        vec![(0, 3), (-1, 4), (-2, 5)]
    );
    assert!(mega_jungle_plan.blocks.iter().any(|block| {
        block.kind == TreePlacementBlockKind::Leaves
            && block.pos
                == BlockPos {
                    x: 145,
                    y: 66,
                    z: 140,
                }
    }));
    assert!(!mega_jungle_plan.blocks.iter().any(|block| {
        block.kind == TreePlacementBlockKind::Leaves
            && block.pos
                == BlockPos {
                    x: 145,
                    y: 66,
                    z: 145,
                }
    }));
}

fn assert_random_spread_foliage_support() {
    let random_spread_plan = simple_tree_plan!(
        BlockPos {
            x: 160,
            y: 64,
            z: 160,
        },
        TrunkPlacerModel {
            base_height: 4,
            height_rand_a: 0,
            height_rand_b: 0,
            kind: TrunkPlacerKind::Straight,
        },
        FoliagePlacerModel {
            radius_min: 3,
            radius_max: 3,
            offset_min: 0,
            offset_max: 0,
            kind: FoliagePlacerKind::RandomSpread {
                foliage_height_min: 4,
                foliage_height_max: 4,
                leaf_placement_attempts: 4,
            },
        },
        "minecraft:azalea_log",
        "minecraft:azalea_leaves",
        "minecraft:dirt",
        0,
        0,
    )
    .unwrap();
    assert_eq!(
        super::super::random_spread_foliage_positions(
            BlockPos {
                x: 10,
                y: 20,
                z: 30
            },
            4,
            3,
            2,
            &[2, 0, 3, 1, 1, 0, 0, 2, 0, 3, 0, 1],
        ),
        vec![
            BlockPos {
                x: 12,
                y: 22,
                z: 31
            },
            BlockPos { x: 8, y: 17, z: 29 },
        ]
    );
    assert_eq!(
        random_spread_plan
            .blocks
            .iter()
            .filter(|block| block.kind == TreePlacementBlockKind::Leaves)
            .count(),
        4
    );
}

fn assert_cherry_foliage_support() {
    let cherry_plan = simple_tree_plan!(
        BlockPos {
            x: 180,
            y: 64,
            z: 180,
        },
        TrunkPlacerModel {
            base_height: 4,
            height_rand_a: 0,
            height_rand_b: 0,
            kind: TrunkPlacerKind::Straight,
        },
        FoliagePlacerModel {
            radius_min: 4,
            radius_max: 4,
            offset_min: 0,
            offset_max: 0,
            kind: FoliagePlacerKind::Cherry {
                height: 5,
                wide_bottom_layer_hole_chance: 0.0,
                corner_hole_chance: 0.0,
                hanging_leaves_chance: 0.0,
                hanging_leaves_extension_chance: 0.0,
            },
        },
        "minecraft:cherry_log",
        "minecraft:cherry_leaves",
        "minecraft:dirt",
        0,
        0,
    )
    .unwrap();
    assert_eq!(
        super::super::cherry_foliage_rows(5, 4),
        vec![(2, 1), (1, 2), (0, 3), (-1, 3), (-2, 2)]
    );
    assert!(cherry_plan.blocks.iter().any(|block| {
        block.kind == TreePlacementBlockKind::Leaves
            && block.pos
                == BlockPos {
                    x: 183,
                    y: 68,
                    z: 182,
                }
    }));
    assert!(!cherry_plan.blocks.iter().any(|block| {
        block.kind == TreePlacementBlockKind::Leaves
            && block.pos
                == BlockPos {
                    x: 183,
                    y: 68,
                    z: 183,
                }
    }));
    assert!(!super::super::cherry_leaves_row_should_skip(
        super::super::CherryLeavesSkipInput {
            dx: 3,
            y_offset: -1,
            dz: 0,
            radius: 3,
            wide_bottom_layer_hole_chance: 0.0,
            corner_hole_chance: 0.0,
            rand_a: 0,
            rand_b: 0,
        }
    ));
    assert!(super::super::cherry_leaves_row_should_skip(
        super::super::CherryLeavesSkipInput {
            dx: 3,
            y_offset: -1,
            dz: 0,
            radius: 3,
            wide_bottom_layer_hole_chance: 1.0,
            corner_hole_chance: 0.0,
            rand_a: 0,
            rand_b: 0,
        }
    ));
}

fn assert_pine_foliage_support() {
    let pine_plan = simple_tree_plan!(
        BlockPos {
            x: 80,
            y: 64,
            z: 80,
        },
        TrunkPlacerModel {
            base_height: 4,
            height_rand_a: 0,
            height_rand_b: 0,
            kind: TrunkPlacerKind::Straight,
        },
        FoliagePlacerModel {
            radius_min: 2,
            radius_max: 2,
            offset_min: 0,
            offset_max: 0,
            kind: FoliagePlacerKind::Pine {
                height_min: 3,
                height_max: 3,
            },
        },
        "minecraft:spruce_log",
        "minecraft:spruce_leaves",
        "minecraft:dirt",
        0,
        0,
    )
    .unwrap();
    assert_eq!(
        super::super::pine_foliage_rows(0, 3, 2),
        vec![(0, 0), (-1, 1), (-2, 2), (-3, 1)]
    );
    assert!(pine_plan.blocks.iter().any(|block| {
        block.kind == TreePlacementBlockKind::Leaves
            && block.pos
                == BlockPos {
                    x: 82,
                    y: 66,
                    z: 80,
                }
    }));
    assert!(!pine_plan.blocks.iter().any(|block| {
        block.kind == TreePlacementBlockKind::Leaves
            && block.pos
                == BlockPos {
                    x: 82,
                    y: 66,
                    z: 82,
                }
    }));
    assert!(pine_plan.blocks.iter().any(|block| {
        block.kind == TreePlacementBlockKind::Leaves
            && block.pos
                == BlockPos {
                    x: 81,
                    y: 65,
                    z: 80,
                }
    }));
}

fn assert_spruce_foliage_support() {
    let spruce_plan = simple_tree_plan!(
        BlockPos {
            x: 100,
            y: 64,
            z: 100,
        },
        TrunkPlacerModel {
            base_height: 6,
            height_rand_a: 0,
            height_rand_b: 0,
            kind: TrunkPlacerKind::Straight,
        },
        FoliagePlacerModel {
            radius_min: 2,
            radius_max: 2,
            offset_min: 0,
            offset_max: 0,
            kind: FoliagePlacerKind::Spruce {
                height_min: 2,
                height_max: 2,
            },
        },
        "minecraft:spruce_log",
        "minecraft:spruce_leaves",
        "minecraft:dirt",
        0,
        1,
    )
    .unwrap();
    assert_eq!(
        super::super::spruce_foliage_rows(0, 4, 2, 1),
        vec![(0, 1), (-1, 0), (-2, 1), (-3, 2), (-4, 1)]
    );
    assert!(spruce_plan.blocks.iter().any(|block| {
        block.kind == TreePlacementBlockKind::Leaves
            && block.pos
                == BlockPos {
                    x: 102,
                    y: 67,
                    z: 100,
                }
    }));
    assert!(!spruce_plan.blocks.iter().any(|block| {
        block.kind == TreePlacementBlockKind::Leaves
            && block.pos
                == BlockPos {
                    x: 102,
                    y: 67,
                    z: 102,
                }
    }));
    assert!(spruce_plan.blocks.iter().any(|block| {
        block.kind == TreePlacementBlockKind::Leaves
            && block.pos
                == BlockPos {
                    x: 101,
                    y: 66,
                    z: 100,
                }
    }));
}

include!("feature_placement_support_tree_variant_large_tests.rs");
