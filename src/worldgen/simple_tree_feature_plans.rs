use super::*;

#[derive(Clone, Copy)]
pub struct SimpleTreePlacementInput {
    pub origin: BlockPos,
    pub trunk: TrunkPlacerModel,
    pub foliage: FoliagePlacerModel,
    pub trunk_state: &'static str,
    pub foliage_state: &'static str,
    pub below_trunk_state: &'static str,
    pub rand_a: i32,
    pub rand_b: i32,
}

struct SimpleTreeFoliageLayout {
    height: i32,
    leaf_radius: i32,
    offset: i32,
    bush_shape: bool,
}

pub fn simple_tree_placement_plan(
    input: SimpleTreePlacementInput,
) -> Result<TreePlacementPlan, String> {
    validate_trunk_placer(input.trunk)?;
    validate_foliage_placer(input.foliage)?;
    if input.trunk.kind != TrunkPlacerKind::Straight {
        return Err(
            "only straight trunk placement is modeled by simple_tree_placement_plan".to_string(),
        );
    }

    let tree_height = trunk_placer_height(input.trunk, input.rand_a, input.rand_b);
    let foliage = simple_tree_foliage_layout(&input, tree_height);
    let mut blocks = Vec::new();
    place_simple_tree_trunk(&mut blocks, &input, tree_height);
    place_simple_tree_foliage(&mut blocks, &input, tree_height, &foliage);
    Ok(TreePlacementPlan { blocks })
}

fn simple_tree_foliage_layout(
    input: &SimpleTreePlacementInput,
    tree_height: i32,
) -> SimpleTreeFoliageLayout {
    let (height, bush_shape) = match input.foliage.kind {
        FoliagePlacerKind::Blob { height }
        | FoliagePlacerKind::Fancy { height }
        | FoliagePlacerKind::Jungle { height } => (height, false),
        FoliagePlacerKind::Bush { height } => (height, true),
        FoliagePlacerKind::Acacia | FoliagePlacerKind::DarkOak => (0, false),
        FoliagePlacerKind::Pine {
            height_min,
            height_max,
        }
        | FoliagePlacerKind::MegaPine {
            height_min,
            height_max,
        } => (
            sample_inclusive_i32(height_min, height_max, input.rand_a),
            false,
        ),
        FoliagePlacerKind::Spruce {
            height_min,
            height_max,
        } => (
            (tree_height - sample_inclusive_i32(height_min, height_max, input.rand_a)).max(4),
            false,
        ),
        FoliagePlacerKind::RandomSpread {
            foliage_height_min,
            foliage_height_max,
            ..
        } => (
            sample_inclusive_i32(foliage_height_min, foliage_height_max, input.rand_a),
            false,
        ),
        FoliagePlacerKind::Cherry { height, .. } => (height, false),
    };
    SimpleTreeFoliageLayout {
        height,
        leaf_radius: sample_inclusive_i32(
            input.foliage.radius_min,
            input.foliage.radius_max,
            input.rand_a,
        ),
        offset: sample_inclusive_i32(
            input.foliage.offset_min,
            input.foliage.offset_max,
            input.rand_b,
        ),
        bush_shape,
    }
}

fn place_simple_tree_trunk(
    blocks: &mut Vec<TreePlacementBlock>,
    input: &SimpleTreePlacementInput,
    tree_height: i32,
) {
    push_tree_block(
        blocks,
        TreePlacementBlock {
            pos: BlockPos {
                x: input.origin.x,
                y: input.origin.y - 1,
                z: input.origin.z,
            },
            state: input.below_trunk_state,
            kind: TreePlacementBlockKind::DirtBelowTrunk,
        },
    );
    for y in 0..tree_height {
        push_tree_block(
            blocks,
            TreePlacementBlock {
                pos: BlockPos {
                    x: input.origin.x,
                    y: input.origin.y + y,
                    z: input.origin.z,
                },
                state: input.trunk_state,
                kind: TreePlacementBlockKind::Log,
            },
        );
    }
}

fn place_simple_tree_foliage(
    blocks: &mut Vec<TreePlacementBlock>,
    input: &SimpleTreePlacementInput,
    tree_height: i32,
    foliage: &SimpleTreeFoliageLayout,
) {
    let foliage_origin = BlockPos {
        x: input.origin.x,
        y: input.origin.y + tree_height,
        z: input.origin.z,
    };
    match input.foliage.kind {
        FoliagePlacerKind::Acacia => place_acacia_foliage(blocks, input, foliage_origin, foliage),
        FoliagePlacerKind::DarkOak => {
            place_dark_oak_foliage(blocks, input, foliage_origin, foliage);
        }
        FoliagePlacerKind::Pine { .. } => {
            place_pine_foliage(blocks, input, foliage_origin, foliage);
        }
        FoliagePlacerKind::Spruce { .. } => {
            place_spruce_foliage(blocks, input, foliage_origin, foliage);
        }
        FoliagePlacerKind::MegaPine { .. } => {
            place_mega_pine_foliage(blocks, input, foliage_origin, foliage);
        }
        FoliagePlacerKind::Cherry {
            wide_bottom_layer_hole_chance,
            corner_hole_chance,
            ..
        } => place_cherry_foliage(
            blocks,
            input,
            foliage_origin,
            foliage,
            wide_bottom_layer_hole_chance,
            corner_hole_chance,
        ),
        FoliagePlacerKind::RandomSpread {
            leaf_placement_attempts,
            ..
        } => place_random_spread_foliage(
            blocks,
            input,
            foliage_origin,
            foliage,
            leaf_placement_attempts,
        ),
        FoliagePlacerKind::Jungle { .. } => {
            place_jungle_foliage(blocks, input, foliage_origin, foliage);
        }
        FoliagePlacerKind::Fancy { .. } => {
            place_fancy_foliage(blocks, input, foliage_origin, foliage);
        }
        FoliagePlacerKind::Blob { .. } | FoliagePlacerKind::Bush { .. } => {
            place_blob_or_bush_foliage(blocks, input, foliage_origin, foliage);
        }
    }
}

fn offset_foliage_origin(origin: BlockPos, offset: i32) -> BlockPos {
    BlockPos {
        x: origin.x,
        y: origin.y + offset,
        z: origin.z,
    }
}

fn place_acacia_foliage(
    blocks: &mut Vec<TreePlacementBlock>,
    input: &SimpleTreePlacementInput,
    foliage_origin: BlockPos,
    foliage: &SimpleTreeFoliageLayout,
) {
    let acacia_origin = offset_foliage_origin(foliage_origin, foliage.offset);
    place_acacia_leaves_row(
        blocks,
        acacia_origin,
        foliage.leaf_radius,
        -1 - foliage.height,
        input.foliage_state,
    );
    place_acacia_leaves_row(
        blocks,
        acacia_origin,
        foliage.leaf_radius - 1,
        -foliage.height,
        input.foliage_state,
    );
    place_acacia_leaves_row(
        blocks,
        acacia_origin,
        foliage.leaf_radius - 1,
        0,
        input.foliage_state,
    );
}

fn place_dark_oak_foliage(
    blocks: &mut Vec<TreePlacementBlock>,
    input: &SimpleTreePlacementInput,
    foliage_origin: BlockPos,
    foliage: &SimpleTreeFoliageLayout,
) {
    let dark_oak_origin = offset_foliage_origin(foliage_origin, foliage.offset);
    place_dark_oak_single_trunk_leaves_row(
        blocks,
        dark_oak_origin,
        foliage.leaf_radius + 2,
        -1,
        input.foliage_state,
    );
    place_dark_oak_single_trunk_leaves_row(
        blocks,
        dark_oak_origin,
        foliage.leaf_radius + 1,
        0,
        input.foliage_state,
    );
}

fn place_pine_foliage(
    blocks: &mut Vec<TreePlacementBlock>,
    input: &SimpleTreePlacementInput,
    foliage_origin: BlockPos,
    foliage: &SimpleTreeFoliageLayout,
) {
    for (y_offset, current_radius) in
        pine_foliage_rows(foliage.offset, foliage.height, foliage.leaf_radius)
    {
        place_conifer_leaves_row(
            blocks,
            foliage_origin,
            current_radius,
            y_offset,
            input.foliage_state,
        );
    }
}

fn place_spruce_foliage(
    blocks: &mut Vec<TreePlacementBlock>,
    input: &SimpleTreePlacementInput,
    foliage_origin: BlockPos,
    foliage: &SimpleTreeFoliageLayout,
) {
    for (y_offset, current_radius) in spruce_foliage_rows(
        foliage.offset,
        foliage.height,
        foliage.leaf_radius,
        input.rand_b.rem_euclid(2),
    ) {
        place_conifer_leaves_row(
            blocks,
            foliage_origin,
            current_radius,
            y_offset,
            input.foliage_state,
        );
    }
}

fn place_mega_pine_foliage(
    blocks: &mut Vec<TreePlacementBlock>,
    input: &SimpleTreePlacementInput,
    foliage_origin: BlockPos,
    foliage: &SimpleTreeFoliageLayout,
) {
    for (y_offset, current_radius) in mega_pine_foliage_rows(
        foliage_origin.y,
        foliage.offset,
        foliage.height,
        foliage.leaf_radius,
    ) {
        place_mega_pine_leaves_row(
            blocks,
            foliage_origin,
            current_radius,
            y_offset,
            input.foliage_state,
        );
    }
}

fn place_cherry_foliage(
    blocks: &mut Vec<TreePlacementBlock>,
    input: &SimpleTreePlacementInput,
    foliage_origin: BlockPos,
    foliage: &SimpleTreeFoliageLayout,
    wide_bottom_layer_hole_chance: f32,
    corner_hole_chance: f32,
) {
    let cherry_origin = offset_foliage_origin(foliage_origin, foliage.offset);
    for (y_offset, current_radius) in cherry_foliage_rows(foliage.height, foliage.leaf_radius) {
        place_cherry_leaves_row(
            blocks,
            CherryLeavesRowInput {
                origin: cherry_origin,
                radius: current_radius,
                y_offset,
                state: input.foliage_state,
                wide_bottom_layer_hole_chance,
                corner_hole_chance,
                rand_a: input.rand_a,
                rand_b: input.rand_b,
            },
        );
    }
}

fn place_random_spread_foliage(
    blocks: &mut Vec<TreePlacementBlock>,
    input: &SimpleTreePlacementInput,
    foliage_origin: BlockPos,
    foliage: &SimpleTreeFoliageLayout,
    leaf_placement_attempts: i32,
) {
    let rolls = deterministic_random_spread_rolls(
        input.rand_a,
        input.rand_b,
        foliage.leaf_radius,
        foliage.height,
        leaf_placement_attempts,
    );
    for pos in random_spread_foliage_positions(
        foliage_origin,
        foliage.height,
        foliage.leaf_radius,
        leaf_placement_attempts,
        &rolls,
    ) {
        push_tree_block(
            blocks,
            TreePlacementBlock {
                pos,
                state: input.foliage_state,
                kind: TreePlacementBlockKind::Leaves,
            },
        );
    }
}

fn place_jungle_foliage(
    blocks: &mut Vec<TreePlacementBlock>,
    input: &SimpleTreePlacementInput,
    foliage_origin: BlockPos,
    foliage: &SimpleTreeFoliageLayout,
) {
    for (y_offset, current_radius) in mega_jungle_foliage_rows(
        foliage.offset,
        1 + input.rand_b.rem_euclid(2),
        foliage.leaf_radius,
    ) {
        place_mega_pine_leaves_row(
            blocks,
            foliage_origin,
            current_radius,
            y_offset,
            input.foliage_state,
        );
    }
}

fn place_fancy_foliage(
    blocks: &mut Vec<TreePlacementBlock>,
    input: &SimpleTreePlacementInput,
    foliage_origin: BlockPos,
    foliage: &SimpleTreeFoliageLayout,
) {
    for (y_offset, current_radius) in
        fancy_foliage_rows(foliage.offset, foliage.height, foliage.leaf_radius)
    {
        place_fancy_leaves_row(
            blocks,
            foliage_origin,
            current_radius,
            y_offset,
            input.foliage_state,
        );
    }
}

fn place_blob_or_bush_foliage(
    blocks: &mut Vec<TreePlacementBlock>,
    input: &SimpleTreePlacementInput,
    foliage_origin: BlockPos,
    foliage: &SimpleTreeFoliageLayout,
) {
    for y_offset in ((foliage.offset - foliage.height)..=foliage.offset).rev() {
        let current_radius = if foliage.bush_shape {
            foliage.leaf_radius - 1 - y_offset
        } else {
            (foliage.leaf_radius - 1 - y_offset / 2).max(0)
        };
        if current_radius < 0 {
            continue;
        }
        place_simple_leaves_row(
            blocks,
            SimpleLeavesRowInput {
                origin: foliage_origin,
                radius: current_radius,
                y_offset,
                state: input.foliage_state,
                blob_shape: matches!(input.foliage.kind, FoliagePlacerKind::Blob { .. }),
                rand_a: input.rand_a,
                rand_b: input.rand_b,
            },
        );
    }
}
