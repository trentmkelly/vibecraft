use super::*;

pub fn simple_tree_placement_plan(
    origin: BlockPos,
    trunk: TrunkPlacerModel,
    foliage: FoliagePlacerModel,
    trunk_state: &'static str,
    foliage_state: &'static str,
    below_trunk_state: &'static str,
    rand_a: i32,
    rand_b: i32,
) -> Result<TreePlacementPlan, String> {
    validate_trunk_placer(trunk)?;
    validate_foliage_placer(foliage)?;
    if trunk.kind != TrunkPlacerKind::Straight {
        return Err(
            "only straight trunk placement is modeled by simple_tree_placement_plan".to_string(),
        );
    }

    let tree_height = trunk_placer_height(trunk, rand_a, rand_b);
    let (foliage_height, bush_shape) = match foliage.kind {
        FoliagePlacerKind::Blob { height }
        | FoliagePlacerKind::Fancy { height }
        | FoliagePlacerKind::Jungle { height } => (height, false),
        FoliagePlacerKind::Bush { height } => (height, true),
        FoliagePlacerKind::Acacia | FoliagePlacerKind::DarkOak => (0, false),
        FoliagePlacerKind::Pine {
            height_min,
            height_max,
        } => (sample_inclusive_i32(height_min, height_max, rand_a), false),
        FoliagePlacerKind::Spruce {
            height_min,
            height_max,
        } => (
            (tree_height - sample_inclusive_i32(height_min, height_max, rand_a)).max(4),
            false,
        ),
        FoliagePlacerKind::MegaPine {
            height_min,
            height_max,
        } => (sample_inclusive_i32(height_min, height_max, rand_a), false),
        FoliagePlacerKind::RandomSpread {
            foliage_height_min,
            foliage_height_max,
            ..
        } => (
            sample_inclusive_i32(foliage_height_min, foliage_height_max, rand_a),
            false,
        ),
        FoliagePlacerKind::Cherry { height, .. } => (height, false),
    };

    let leaf_radius = sample_inclusive_i32(foliage.radius_min, foliage.radius_max, rand_a);
    let foliage_offset = sample_inclusive_i32(foliage.offset_min, foliage.offset_max, rand_b);
    let mut blocks = Vec::new();
    push_tree_block(
        &mut blocks,
        TreePlacementBlock {
            pos: BlockPos {
                x: origin.x,
                y: origin.y - 1,
                z: origin.z,
            },
            state: below_trunk_state,
            kind: TreePlacementBlockKind::DirtBelowTrunk,
        },
    );
    for y in 0..tree_height {
        push_tree_block(
            &mut blocks,
            TreePlacementBlock {
                pos: BlockPos {
                    x: origin.x,
                    y: origin.y + y,
                    z: origin.z,
                },
                state: trunk_state,
                kind: TreePlacementBlockKind::Log,
            },
        );
    }

    let foliage_origin = BlockPos {
        x: origin.x,
        y: origin.y + tree_height,
        z: origin.z,
    };
    if foliage.kind == FoliagePlacerKind::Acacia {
        let acacia_origin = BlockPos {
            x: foliage_origin.x,
            y: foliage_origin.y + foliage_offset,
            z: foliage_origin.z,
        };
        place_acacia_leaves_row(
            &mut blocks,
            acacia_origin,
            leaf_radius,
            -1 - foliage_height,
            foliage_state,
        );
        place_acacia_leaves_row(
            &mut blocks,
            acacia_origin,
            leaf_radius - 1,
            -foliage_height,
            foliage_state,
        );
        place_acacia_leaves_row(
            &mut blocks,
            acacia_origin,
            leaf_radius - 1,
            0,
            foliage_state,
        );
    } else if foliage.kind == FoliagePlacerKind::DarkOak {
        let dark_oak_origin = BlockPos {
            x: foliage_origin.x,
            y: foliage_origin.y + foliage_offset,
            z: foliage_origin.z,
        };
        place_dark_oak_single_trunk_leaves_row(
            &mut blocks,
            dark_oak_origin,
            leaf_radius + 2,
            -1,
            foliage_state,
        );
        place_dark_oak_single_trunk_leaves_row(
            &mut blocks,
            dark_oak_origin,
            leaf_radius + 1,
            0,
            foliage_state,
        );
    } else if matches!(foliage.kind, FoliagePlacerKind::Pine { .. }) {
        for (y_offset, current_radius) in
            pine_foliage_rows(foliage_offset, foliage_height, leaf_radius)
        {
            place_conifer_leaves_row(
                &mut blocks,
                foliage_origin,
                current_radius,
                y_offset,
                foliage_state,
            );
        }
    } else if matches!(foliage.kind, FoliagePlacerKind::Spruce { .. }) {
        for (y_offset, current_radius) in spruce_foliage_rows(
            foliage_offset,
            foliage_height,
            leaf_radius,
            rand_b.rem_euclid(2),
        ) {
            place_conifer_leaves_row(
                &mut blocks,
                foliage_origin,
                current_radius,
                y_offset,
                foliage_state,
            );
        }
    } else if matches!(foliage.kind, FoliagePlacerKind::MegaPine { .. }) {
        for (y_offset, current_radius) in mega_pine_foliage_rows(
            foliage_origin.y,
            foliage_offset,
            foliage_height,
            leaf_radius,
        ) {
            place_mega_pine_leaves_row(
                &mut blocks,
                foliage_origin,
                current_radius,
                y_offset,
                foliage_state,
            );
        }
    } else if let FoliagePlacerKind::Cherry {
        wide_bottom_layer_hole_chance,
        corner_hole_chance,
        ..
    } = foliage.kind
    {
        let cherry_origin = BlockPos {
            x: foliage_origin.x,
            y: foliage_origin.y + foliage_offset,
            z: foliage_origin.z,
        };
        for (y_offset, current_radius) in cherry_foliage_rows(foliage_height, leaf_radius) {
            place_cherry_leaves_row(
                &mut blocks,
                CherryLeavesRowInput {
                    origin: cherry_origin,
                    radius: current_radius,
                    y_offset,
                    state: foliage_state,
                    wide_bottom_layer_hole_chance,
                    corner_hole_chance,
                    rand_a,
                    rand_b,
                },
            );
        }
    } else if let FoliagePlacerKind::RandomSpread {
        leaf_placement_attempts,
        ..
    } = foliage.kind
    {
        let rolls = deterministic_random_spread_rolls(
            rand_a,
            rand_b,
            leaf_radius,
            foliage_height,
            leaf_placement_attempts,
        );
        for pos in random_spread_foliage_positions(
            foliage_origin,
            foliage_height,
            leaf_radius,
            leaf_placement_attempts,
            &rolls,
        ) {
            push_tree_block(
                &mut blocks,
                TreePlacementBlock {
                    pos,
                    state: foliage_state,
                    kind: TreePlacementBlockKind::Leaves,
                },
            );
        }
    } else if matches!(foliage.kind, FoliagePlacerKind::Jungle { .. }) {
        for (y_offset, current_radius) in
            mega_jungle_foliage_rows(foliage_offset, 1 + rand_b.rem_euclid(2), leaf_radius)
        {
            place_mega_pine_leaves_row(
                &mut blocks,
                foliage_origin,
                current_radius,
                y_offset,
                foliage_state,
            );
        }
    } else if matches!(foliage.kind, FoliagePlacerKind::Fancy { .. }) {
        for (y_offset, current_radius) in
            fancy_foliage_rows(foliage_offset, foliage_height, leaf_radius)
        {
            place_fancy_leaves_row(
                &mut blocks,
                foliage_origin,
                current_radius,
                y_offset,
                foliage_state,
            );
        }
    } else {
        for y_offset in ((foliage_offset - foliage_height)..=foliage_offset).rev() {
            let current_radius = if bush_shape {
                leaf_radius - 1 - y_offset
            } else {
                (leaf_radius - 1 - y_offset / 2).max(0)
            };
            if current_radius < 0 {
                continue;
            }
            place_simple_leaves_row(
                &mut blocks,
                SimpleLeavesRowInput {
                    origin: foliage_origin,
                    radius: current_radius,
                    y_offset,
                    state: foliage_state,
                    blob_shape: matches!(foliage.kind, FoliagePlacerKind::Blob { .. }),
                    rand_a,
                    rand_b,
                },
            );
        }
    }

    Ok(TreePlacementPlan { blocks })
}
