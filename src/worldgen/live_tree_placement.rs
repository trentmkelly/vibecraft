use super::*;

pub(super) fn live_tree_placement_plan(
    block_context: &TreeDecorationBlockContext<'_>,
    previous_source_blocks: &TreeBlockOverlay,
    origin: BlockPos,
    config: LiveTreeFeatureConfig,
    rand_a: i32,
    rand_b: i32,
    clipped_tree_height: i32,
    random: &mut RandomSourceKind,
) -> Result<TreePlacementPlan, String> {
    let trunk = TrunkPlacerModel {
        base_height: config.base_height,
        height_rand_a: config.height_rand_a,
        height_rand_b: config.height_rand_b,
        kind: if matches!(config.foliage.kind, FoliagePlacerKind::Fancy { .. }) {
            TrunkPlacerKind::Fancy
        } else {
            TrunkPlacerKind::Straight
        },
    };
    if !matches!(trunk.kind, TrunkPlacerKind::Fancy) {
        let mut plan = live_straight_blob_tree_placement_plan(
            origin,
            trunk,
            clipped_tree_height,
            config.foliage,
            config.trunk_state,
            config.leaves_state,
            "minecraft:dirt",
            rand_a,
            rand_b,
            random,
        )?;
        filter_live_tree_feature_blocks_like_java(block_context, previous_source_blocks, &mut plan);
        return Ok(plan);
    }

    let cluster_rolls = (0..fancy_trunk_cluster_roll_count(clipped_tree_height))
        .map(|_| FancyTrunkClusterRollModel {
            shape_float: feature_random_next_f32(random),
            angle_float: feature_random_next_f32(random),
        })
        .collect::<Vec<_>>();
    let trunk_plan = live_fancy_trunk_placement_plan(
        block_context,
        previous_source_blocks,
        origin,
        clipped_tree_height,
        config.trunk_state,
        "minecraft:dirt",
        &cluster_rolls,
    );
    let mut blocks = trunk_plan.blocks;
    let (foliage_height, foliage_offset) = match config.foliage.kind {
        FoliagePlacerKind::Fancy { height } => (height, config.foliage.offset_min),
        _ => (0, 0),
    };
    let leaf_radius =
        sample_inclusive_i32(config.foliage.radius_min, config.foliage.radius_max, rand_a);
    for attachment in trunk_plan.attachments {
        for (y_offset, current_radius) in fancy_foliage_rows(
            foliage_offset,
            foliage_height,
            leaf_radius + attachment.radius_offset,
        ) {
            place_fancy_leaves_row(
                &mut blocks,
                attachment.pos,
                current_radius,
                y_offset,
                config.leaves_state,
            );
        }
    }
    let mut plan = TreePlacementPlan { blocks };
    filter_live_tree_feature_blocks_like_java(block_context, previous_source_blocks, &mut plan);
    Ok(plan)
}

fn filter_live_tree_feature_blocks_like_java(
    block_context: &TreeDecorationBlockContext<'_>,
    previous_source_blocks: &TreeBlockOverlay,
    plan: &mut TreePlacementPlan,
) {
    let mut accepted = Vec::with_capacity(plan.blocks.len());
    for block in plan.blocks.iter().copied() {
        let can_place = match block.kind {
            TreePlacementBlockKind::DirtBelowTrunk | TreePlacementBlockKind::GroundCover => true,
            TreePlacementBlockKind::Log | TreePlacementBlockKind::Leaves => {
                let state = live_tree_state_with_planned_blocks(
                    block_context.source_pos,
                    block_context,
                    previous_source_blocks,
                    &accepted,
                    block.pos,
                );
                tree_valid_pos(&state)
            }
        };
        if can_place {
            accepted.push(block);
        }
    }
    plan.blocks = accepted;
}

pub(super) fn append_live_tree_decorators(
    source_pos: ChunkPos,
    block_context: &TreeDecorationBlockContext<'_>,
    source_terrain_heights: SourceTerrainHeights<'_>,
    settings: &NoiseGeneratorSettings,
    previous_source_blocks: &TreeBlockOverlay,
    plan: &mut TreePlacementPlan,
    decorators: LiveTreeDecoratorSet,
    random: &mut RandomSourceKind,
) {
    if decorators == LiveTreeDecoratorSet::None {
        return;
    }

    let logs = plan
        .blocks
        .iter()
        .filter(|block| block.kind == TreePlacementBlockKind::Log)
        .map(|block| local_tree_block_to_world(source_pos, block.pos))
        .collect::<Vec<_>>();
    if logs.is_empty() {
        return;
    }
    let leaves = plan
        .blocks
        .iter()
        .filter(|block| block.kind == TreePlacementBlockKind::Leaves)
        .map(|block| local_tree_block_to_world(source_pos, block.pos))
        .collect::<Vec<_>>();

    let beehive_probability = match decorators {
        LiveTreeDecoratorSet::None => 0,
        LiveTreeDecoratorSet::Bees {
            probability_per_tree,
        }
        | LiveTreeDecoratorSet::BeesAndLeafLitter {
            probability_per_tree,
        } => probability_per_tree,
    };
    consume_live_beehive_decorator_random(&logs, &leaves, beehive_probability, random);

    if matches!(decorators, LiveTreeDecoratorSet::BeesAndLeafLitter { .. }) {
        append_live_place_on_ground_leaf_litter(
            source_pos,
            block_context,
            source_terrain_heights,
            settings,
            previous_source_blocks,
            plan,
            &logs,
            96,
            4,
            2,
            random,
        );
        append_live_place_on_ground_leaf_litter(
            source_pos,
            block_context,
            source_terrain_heights,
            settings,
            previous_source_blocks,
            plan,
            &logs,
            150,
            2,
            2,
            random,
        );
    }
}

fn consume_live_beehive_decorator_random(
    logs: &[BlockPos],
    leaves: &[BlockPos],
    probability_per_tree: u32,
    random: &mut RandomSourceKind,
) {
    if logs.is_empty() {
        return;
    }
    let roll = feature_random_next_f32(random);
    if roll >= probability_per_tree as f32 / 1_000_000.0 {
        return;
    }

    let mut sorted_logs = logs.to_vec();
    sorted_logs.sort_by_key(|pos| pos.y);
    let mut sorted_leaves = leaves.to_vec();
    sorted_leaves.sort_by_key(|pos| pos.y);
    if sorted_leaves.is_empty() {
        let _leafless_height_roll = feature_random_next_i32_bound(random, 3);
    }
    let hive_y = sorted_leaves
        .first()
        .zip(sorted_logs.first())
        .map(|(leaf, log)| (leaf.y - 1).max(log.y + 1))
        .unwrap_or_else(|| {
            let first_log_y = sorted_logs.first().map(|pos| pos.y).unwrap_or(0);
            let last_log_y = sorted_logs.last().map(|pos| pos.y).unwrap_or(first_log_y);
            (first_log_y + 1).min(last_log_y)
        });
    let candidate_count = sorted_logs.iter().filter(|pos| pos.y == hive_y).count() * 3;
    for bound in (2..=candidate_count).rev() {
        let _shuffle_roll = feature_random_next_i32_bound(random, bound as i32);
    }
    let _bee_count_roll = feature_random_next_i32_bound(random, 2);
}

#[allow(clippy::too_many_arguments)]
fn append_live_place_on_ground_leaf_litter(
    source_pos: ChunkPos,
    block_context: &TreeDecorationBlockContext<'_>,
    source_terrain_heights: SourceTerrainHeights<'_>,
    settings: &NoiseGeneratorSettings,
    previous_source_blocks: &TreeBlockOverlay,
    plan: &mut TreePlacementPlan,
    logs_world: &[BlockPos],
    tries: i32,
    radius: i32,
    height: i32,
    random: &mut RandomSourceKind,
) {
    let mut lowest = logs_world.to_vec();
    lowest.sort_by_key(|pos| pos.y);
    let Some(origin) = lowest.first().copied() else {
        return;
    };
    let min_y = origin.y;
    let mut min_x = origin.x;
    let mut max_x = origin.x;
    let mut min_z = origin.z;
    let mut max_z = origin.z;
    for position in lowest.iter().filter(|pos| pos.y == min_y) {
        min_x = min_x.min(position.x);
        max_x = max_x.max(position.x);
        min_z = min_z.min(position.z);
        max_z = max_z.max(position.z);
    }

    let mut decorator_blocks: HashMap<(i32, i32, i32), &'static str> = previous_source_blocks
        .iter()
        .map(|(pos, state)| (*pos, *state))
        .collect();
    for block in &plan.blocks {
        decorator_blocks.insert(
            local_tree_block_to_world_key(source_pos, block.pos),
            block.state,
        );
    }
    let mut motion_height_overlay: HashMap<(i32, i32), i32> = HashMap::new();
    for ((x, y, z), state) in &decorator_blocks {
        if tree_decorator_motion_blocking_no_leaves_opaque(state) {
            let key = (*x, *z);
            let height = *y + 1;
            motion_height_overlay
                .entry(key)
                .and_modify(|current| *current = (*current).max(height))
                .or_insert(height);
        }
    }
    for _ in 0..tries {
        let x =
            min_x - radius + feature_random_next_i32_bound(random, max_x - min_x + radius * 2 + 1);
        let y = min_y - height + feature_random_next_i32_bound(random, height * 2 + 1);
        let z =
            min_z - radius + feature_random_next_i32_bound(random, max_z - min_z + radius * 2 + 1);
        let pos = BlockPos { x, y, z };
        let above = BlockPos { x, y: y + 1, z };
        let above_state = live_tree_decorator_state(block_context, &decorator_blocks, above);
        let pos_state = live_tree_decorator_state(block_context, &decorator_blocks, pos);
        if !matches!(
            block_state_id(&above_state),
            "minecraft:air" | "minecraft:cave_air" | "minecraft:void_air" | "minecraft:vine"
        ) || !tree_decorator_solid_render(&pos_state)
            || {
                let motion_height = live_tree_motion_blocking_no_leaves_height(
                    source_pos,
                    source_terrain_heights,
                    block_context,
                    &decorator_blocks,
                    &motion_height_overlay,
                    settings,
                    x,
                    z,
                );
                motion_height > above.y
            }
        {
            continue;
        }

        let _state_roll = feature_random_next_i32_bound(random, 12);
        plan.blocks.push(TreePlacementBlock {
            pos: world_tree_block_to_local(source_pos, above),
            state: "minecraft:leaf_litter",
            kind: TreePlacementBlockKind::GroundCover,
        });
        decorator_blocks.insert((above.x, above.y, above.z), "minecraft:leaf_litter");
    }
}

fn live_tree_motion_blocking_no_leaves_height(
    source_pos: ChunkPos,
    source_terrain_heights: SourceTerrainHeights<'_>,
    block_context: &TreeDecorationBlockContext<'_>,
    decorator_blocks: &HashMap<(i32, i32, i32), &'static str>,
    motion_height_overlay: &HashMap<(i32, i32), i32>,
    settings: &NoiseGeneratorSettings,
    world_x: i32,
    world_z: i32,
) -> i32 {
    let mut height = source_terrain_heights.world_height(
        source_pos,
        HeightmapKind::MotionBlockingNoLeaves,
        world_x,
        world_z,
        settings.noise.min_y,
    );
    if let Some(overlay_height) = motion_height_overlay.get(&(world_x, world_z)) {
        height = height.max(*overlay_height);
    }
    if height > settings.noise.min_y {
        return height;
    }

    // Rare border case: a tree decorator can probe outside its source chunk. Fall
    // back to the level view there because the source heightmap only covers the
    // chunk that owns the feature.
    for y in (settings.noise.min_y..settings.noise.min_y + settings.noise.height).rev() {
        let state = live_tree_decorator_state(
            block_context,
            decorator_blocks,
            BlockPos {
                x: world_x,
                y,
                z: world_z,
            },
        );
        if tree_decorator_motion_blocking_no_leaves_opaque(&state) {
            return y + 1;
        }
    }
    height
}

fn tree_decorator_motion_blocking_no_leaves_opaque(block: &str) -> bool {
    (tree_decorator_solid_render(block) || block_has_fluid(block)) && !block_is_leaves(block)
}

pub(super) fn tree_decorator_solid_render(block: &str) -> bool {
    let id = block_state_id(block);
    if matches!(
        id,
        "minecraft:air"
            | "minecraft:cave_air"
            | "minecraft:void_air"
            | "minecraft:water"
            | "minecraft:lava"
            | "minecraft:vine"
            | "minecraft:leaf_litter"
            | "minecraft:short_grass"
            | "minecraft:tall_grass"
            | "minecraft:fern"
            | "minecraft:large_fern"
            | "minecraft:bush"
            | "minecraft:dead_bush"
            | "minecraft:brown_mushroom"
            | "minecraft:red_mushroom"
            | "minecraft:sugar_cane"
            | "minecraft:oak_sapling"
            | "minecraft:birch_sapling"
            | "minecraft:spruce_sapling"
            | "minecraft:jungle_sapling"
            | "minecraft:acacia_sapling"
            | "minecraft:dark_oak_sapling"
    ) {
        return false;
    }
    !id.ends_with("_leaves") && !id.ends_with("_flower") && !id.ends_with("_tulip")
}

fn live_tree_decorator_state(
    block_context: &TreeDecorationBlockContext<'_>,
    decorator_blocks: &HashMap<(i32, i32, i32), &'static str>,
    world_pos: BlockPos,
) -> String {
    decorator_blocks
        .get(&(world_pos.x, world_pos.y, world_pos.z))
        .map(|state| (*state).to_string())
        .unwrap_or_else(|| {
            block_context
                .block_state(world_pos.x, world_pos.y, world_pos.z)
                .map(|state| block_state_id(state).to_string())
                .unwrap_or_else(|| "minecraft:air".to_string())
        })
}
