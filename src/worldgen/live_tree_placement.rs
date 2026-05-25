use super::*;

pub(super) struct LiveTreePlacementInput<'a, 'ctx> {
    pub(super) block_context: &'a TreeDecorationBlockContext<'ctx>,
    pub(super) previous_source_blocks: &'a TreeBlockOverlay,
    pub(super) origin: BlockPos,
    pub(super) config: LiveTreeFeatureConfig,
    pub(super) rand_a: i32,
    pub(super) rand_b: i32,
    pub(super) clipped_tree_height: i32,
}

pub(super) fn live_tree_placement_plan(
    input: LiveTreePlacementInput<'_, '_>,
    random: &mut RandomSourceKind,
) -> Result<TreePlacementPlan, String> {
    let trunk = TrunkPlacerModel {
        base_height: input.config.base_height,
        height_rand_a: input.config.height_rand_a,
        height_rand_b: input.config.height_rand_b,
        kind: if matches!(input.config.foliage.kind, FoliagePlacerKind::Fancy { .. }) {
            TrunkPlacerKind::Fancy
        } else {
            TrunkPlacerKind::Straight
        },
    };
    if !matches!(trunk.kind, TrunkPlacerKind::Fancy) {
        let mut plan = live_straight_blob_tree_placement_plan(
            input.origin,
            trunk,
            input.clipped_tree_height,
            input.config.foliage,
            input.config.trunk_state,
            input.config.leaves_state,
            "minecraft:dirt",
            input.rand_a,
            input.rand_b,
            random,
        )?;
        filter_live_tree_feature_blocks_like_java(
            input.block_context,
            input.previous_source_blocks,
            &mut plan,
        );
        return Ok(plan);
    }

    let cluster_rolls = (0..fancy_trunk_cluster_roll_count(input.clipped_tree_height))
        .map(|_| FancyTrunkClusterRollModel {
            shape_float: feature_random_next_f32(random),
            angle_float: feature_random_next_f32(random),
        })
        .collect::<Vec<_>>();
    let trunk_plan = live_fancy_trunk_placement_plan(
        input.block_context,
        input.previous_source_blocks,
        input.origin,
        input.clipped_tree_height,
        input.config.trunk_state,
        "minecraft:dirt",
        &cluster_rolls,
    );
    let mut blocks = trunk_plan.blocks;
    let (foliage_height, foliage_offset) = match input.config.foliage.kind {
        FoliagePlacerKind::Fancy { height } => (height, input.config.foliage.offset_min),
        _ => (0, 0),
    };
    let leaf_radius =
        sample_inclusive_i32(input.config.foliage.radius_min, input.config.foliage.radius_max, input.rand_a);
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
                input.config.leaves_state,
            );
        }
    }
    let mut plan = TreePlacementPlan { blocks };
    filter_live_tree_feature_blocks_like_java(
        input.block_context,
        input.previous_source_blocks,
        &mut plan,
    );
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

pub(super) struct LiveTreeDecoratorInput<'a, 'ctx, 'heights> {
    pub(super) source_pos: ChunkPos,
    pub(super) block_context: &'a TreeDecorationBlockContext<'ctx>,
    pub(super) source_terrain_heights: SourceTerrainHeights<'heights>,
    pub(super) settings: &'a NoiseGeneratorSettings,
    pub(super) previous_source_blocks: &'a TreeBlockOverlay,
    pub(super) decorators: LiveTreeDecoratorSet,
}

pub(super) fn append_live_tree_decorators(
    input: LiveTreeDecoratorInput<'_, '_, '_>,
    plan: &mut TreePlacementPlan,
    random: &mut RandomSourceKind,
) {
    if input.decorators == LiveTreeDecoratorSet::None {
        return;
    }

    let logs = plan
        .blocks
        .iter()
        .filter(|block| block.kind == TreePlacementBlockKind::Log)
        .map(|block| local_tree_block_to_world(input.source_pos, block.pos))
        .collect::<Vec<_>>();
    if logs.is_empty() {
        return;
    }
    let leaves = plan
        .blocks
        .iter()
        .filter(|block| block.kind == TreePlacementBlockKind::Leaves)
        .map(|block| local_tree_block_to_world(input.source_pos, block.pos))
        .collect::<Vec<_>>();

    let beehive_probability = match input.decorators {
        LiveTreeDecoratorSet::None => 0,
        LiveTreeDecoratorSet::Bees {
            probability_per_tree,
        }
        | LiveTreeDecoratorSet::BeesAndLeafLitter {
            probability_per_tree,
        } => probability_per_tree,
    };
    consume_live_beehive_decorator_random(&logs, &leaves, beehive_probability, random);

    if matches!(input.decorators, LiveTreeDecoratorSet::BeesAndLeafLitter { .. }) {
        append_live_place_on_ground_leaf_litter(
            LiveLeafLitterInput {
                source_pos: input.source_pos,
                block_context: input.block_context,
                source_terrain_heights: input.source_terrain_heights,
                settings: input.settings,
                previous_source_blocks: input.previous_source_blocks,
                tries: 96,
                radius: 4,
                height: 2,
            },
            plan,
            &logs,
            random,
        );
        append_live_place_on_ground_leaf_litter(
            LiveLeafLitterInput {
                source_pos: input.source_pos,
                block_context: input.block_context,
                source_terrain_heights: input.source_terrain_heights,
                settings: input.settings,
                previous_source_blocks: input.previous_source_blocks,
                tries: 150,
                radius: 2,
                height: 2,
            },
            plan,
            &logs,
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

struct LiveLeafLitterInput<'a, 'ctx, 'heights> {
    source_pos: ChunkPos,
    block_context: &'a TreeDecorationBlockContext<'ctx>,
    source_terrain_heights: SourceTerrainHeights<'heights>,
    settings: &'a NoiseGeneratorSettings,
    previous_source_blocks: &'a TreeBlockOverlay,
    tries: i32,
    radius: i32,
    height: i32,
}

fn append_live_place_on_ground_leaf_litter(
    input: LiveLeafLitterInput<'_, '_, '_>,
    plan: &mut TreePlacementPlan,
    logs_world: &[BlockPos],
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

    let mut decorator_blocks: HashMap<(i32, i32, i32), &'static str> = input
        .previous_source_blocks
        .iter()
        .map(|(pos, state)| (*pos, *state))
        .collect();
    for block in &plan.blocks {
        decorator_blocks.insert(
            local_tree_block_to_world_key(input.source_pos, block.pos),
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
    for _ in 0..input.tries {
        let x = min_x - input.radius
            + feature_random_next_i32_bound(random, max_x - min_x + input.radius * 2 + 1);
        let y = min_y - input.height + feature_random_next_i32_bound(random, input.height * 2 + 1);
        let z = min_z - input.radius
            + feature_random_next_i32_bound(random, max_z - min_z + input.radius * 2 + 1);
        let pos = BlockPos { x, y, z };
        let above = BlockPos { x, y: y + 1, z };
        let above_state =
            live_tree_decorator_state(input.block_context, &decorator_blocks, above);
        let pos_state = live_tree_decorator_state(input.block_context, &decorator_blocks, pos);
        if !matches!(
            block_state_id(&above_state),
            "minecraft:air" | "minecraft:cave_air" | "minecraft:void_air" | "minecraft:vine"
        ) || !tree_decorator_solid_render(&pos_state)
            || {
                let motion_height = live_tree_motion_blocking_no_leaves_height(
                    LiveTreeMotionHeightInput {
                        source_pos: input.source_pos,
                        source_terrain_heights: input.source_terrain_heights,
                        block_context: input.block_context,
                        decorator_blocks: &decorator_blocks,
                        motion_height_overlay: &motion_height_overlay,
                        settings: input.settings,
                    },
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
            pos: world_tree_block_to_local(input.source_pos, above),
            state: "minecraft:leaf_litter",
            kind: TreePlacementBlockKind::GroundCover,
        });
        decorator_blocks.insert((above.x, above.y, above.z), "minecraft:leaf_litter");
    }
}

struct LiveTreeMotionHeightInput<'a, 'ctx, 'heights, 'maps> {
    source_pos: ChunkPos,
    source_terrain_heights: SourceTerrainHeights<'heights>,
    block_context: &'a TreeDecorationBlockContext<'ctx>,
    decorator_blocks: &'maps HashMap<(i32, i32, i32), &'static str>,
    motion_height_overlay: &'maps HashMap<(i32, i32), i32>,
    settings: &'a NoiseGeneratorSettings,
}

fn live_tree_motion_blocking_no_leaves_height(
    input: LiveTreeMotionHeightInput<'_, '_, '_, '_>,
    world_x: i32,
    world_z: i32,
) -> i32 {
    let mut height = input.source_terrain_heights.world_height(
        input.source_pos,
        HeightmapKind::MotionBlockingNoLeaves,
        world_x,
        world_z,
        input.settings.noise.min_y,
    );
    if let Some(overlay_height) = input.motion_height_overlay.get(&(world_x, world_z)) {
        height = height.max(*overlay_height);
    }
    if height > input.settings.noise.min_y {
        return height;
    }

    // Rare border case: a tree decorator can probe outside its source chunk. Fall
    // back to the level view there because the source heightmap only covers the
    // chunk that owns the feature.
    for y in (input.settings.noise.min_y..input.settings.noise.min_y + input.settings.noise.height)
        .rev()
    {
        let state = live_tree_decorator_state(
            input.block_context,
            input.decorator_blocks,
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
