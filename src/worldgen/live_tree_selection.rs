use super::*;
use super::live_tree_placement::tree_decorator_solid_render;

#[derive(Debug, Clone, Copy)]
pub(super) struct LiveTreeFeatureConfig {
    pub(super) trunk_state: &'static str,
    pub(super) leaves_state: &'static str,
    pub(super) base_height: i32,
    pub(super) height_rand_a: i32,
    pub(super) height_rand_b: i32,
    pub(super) rand_a_bound: i32,
    pub(super) rand_b_bound: i32,
    pub(super) foliage: FoliagePlacerModel,
    pub(super) minimum_size: FeatureSizeModel,
    pub(super) min_clipped_height: Option<i32>,
    pub(super) decorators: LiveTreeDecoratorSet,
}

#[derive(Debug, Clone)]
pub(super) enum LiveTreeFeatureSelection {
    Tree(LiveTreeFeatureConfig),
    Fallen(FallenTreeConfigurationModel),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum LiveTreeDecoratorSet {
    None,
    Bees { probability_per_tree: u32 },
    BeesAndLeafLitter { probability_per_tree: u32 },
}

pub(super) fn live_tree_feature_selection(
    feature: &str,
    random: &mut RandomSourceKind,
) -> Option<LiveTreeFeatureSelection> {
    let feature = feature.strip_prefix("minecraft:").unwrap_or(feature);
    match feature {
        "trees_birch_and_oak_leaf_litter" => {
            if feature_random_next_f32(random) < 0.0025 {
                return Some(LiveTreeFeatureSelection::Fallen(
                    live_fallen_birch_tree_config(),
                ));
            }
            if feature_random_next_f32(random) < 0.2 {
                return Some(LiveTreeFeatureSelection::Tree(
                    live_birch_leaf_litter_tree_config(),
                ));
            }
            if feature_random_next_f32(random) < 0.1 {
                return Some(LiveTreeFeatureSelection::Tree(
                    live_fancy_oak_leaf_litter_tree_config(),
                ));
            }
            if feature_random_next_f32(random) < 0.0125 {
                return Some(LiveTreeFeatureSelection::Fallen(
                    live_fallen_oak_tree_config(),
                ));
            }
            Some(LiveTreeFeatureSelection::Tree(
                live_oak_leaf_litter_tree_config(),
            ))
        }
        "trees_plains" => {
            if feature_random_next_f32(random) < 0.33333334 {
                return Some(LiveTreeFeatureSelection::Tree(
                    live_fancy_oak_bees_005_tree_config(),
                ));
            }
            if feature_random_next_f32(random) < 0.0125 {
                return Some(LiveTreeFeatureSelection::Fallen(
                    live_fallen_oak_tree_config(),
                ));
            }
            Some(LiveTreeFeatureSelection::Tree(
                live_oak_bees_005_tree_config(),
            ))
        }
        "trees_birch" => {
            if feature_random_next_f32(random) < 0.0125 {
                Some(LiveTreeFeatureSelection::Fallen(
                    live_fallen_birch_tree_config(),
                ))
            } else {
                Some(LiveTreeFeatureSelection::Tree(live_birch_tree_config()))
            }
        }
        _ => Some(LiveTreeFeatureSelection::Tree(live_oak_tree_config())),
    }
}

fn live_oak_tree_config() -> LiveTreeFeatureConfig {
    LiveTreeFeatureConfig {
        trunk_state: "minecraft:oak_log",
        leaves_state: "minecraft:oak_leaves",
        base_height: 4,
        height_rand_a: 2,
        height_rand_b: 0,
        rand_a_bound: 3,
        rand_b_bound: 1,
        foliage: FoliagePlacerModel {
            radius_min: 2,
            radius_max: 2,
            offset_min: 0,
            offset_max: 0,
            kind: FoliagePlacerKind::Blob { height: 3 },
        },
        minimum_size: FeatureSizeModel::TwoLayers {
            limit: 1,
            lower_size: 0,
            upper_size: 1,
            min_clipped_height: None,
        },
        min_clipped_height: None,
        decorators: LiveTreeDecoratorSet::None,
    }
}

pub(super) fn live_oak_bees_005_tree_config() -> LiveTreeFeatureConfig {
    LiveTreeFeatureConfig {
        decorators: LiveTreeDecoratorSet::Bees {
            probability_per_tree: 50_000,
        },
        ..live_oak_tree_config()
    }
}

pub(super) fn live_oak_leaf_litter_tree_config() -> LiveTreeFeatureConfig {
    LiveTreeFeatureConfig {
        decorators: LiveTreeDecoratorSet::BeesAndLeafLitter {
            probability_per_tree: 2_000,
        },
        ..live_oak_tree_config()
    }
}

pub(super) fn live_birch_tree_config() -> LiveTreeFeatureConfig {
    LiveTreeFeatureConfig {
        trunk_state: "minecraft:birch_log",
        leaves_state: "minecraft:birch_leaves",
        base_height: 5,
        decorators: LiveTreeDecoratorSet::Bees {
            probability_per_tree: 2_000,
        },
        ..live_oak_tree_config()
    }
}

fn live_birch_leaf_litter_tree_config() -> LiveTreeFeatureConfig {
    LiveTreeFeatureConfig {
        decorators: LiveTreeDecoratorSet::BeesAndLeafLitter {
            probability_per_tree: 2_000,
        },
        ..live_birch_tree_config()
    }
}

fn live_fancy_oak_tree_config() -> LiveTreeFeatureConfig {
    LiveTreeFeatureConfig {
        trunk_state: "minecraft:oak_log",
        leaves_state: "minecraft:oak_leaves",
        base_height: 3,
        height_rand_a: 11,
        height_rand_b: 0,
        rand_a_bound: 12,
        rand_b_bound: 1,
        foliage: FoliagePlacerModel {
            radius_min: 2,
            radius_max: 2,
            offset_min: 4,
            offset_max: 4,
            kind: FoliagePlacerKind::Fancy { height: 4 },
        },
        minimum_size: FeatureSizeModel::TwoLayers {
            limit: 0,
            lower_size: 0,
            upper_size: 0,
            min_clipped_height: Some(4),
        },
        min_clipped_height: Some(4),
        decorators: LiveTreeDecoratorSet::None,
    }
}

fn live_fancy_oak_bees_005_tree_config() -> LiveTreeFeatureConfig {
    LiveTreeFeatureConfig {
        decorators: LiveTreeDecoratorSet::Bees {
            probability_per_tree: 50_000,
        },
        ..live_fancy_oak_tree_config()
    }
}

fn live_fancy_oak_leaf_litter_tree_config() -> LiveTreeFeatureConfig {
    LiveTreeFeatureConfig {
        decorators: LiveTreeDecoratorSet::BeesAndLeafLitter {
            probability_per_tree: 2_000,
        },
        ..live_fancy_oak_tree_config()
    }
}

pub(super) fn live_tree_selector_trace(feature: &str, mut random: RandomSourceKind) -> Option<String> {
    let feature = feature.strip_prefix("minecraft:").unwrap_or(feature);
    match feature {
        "trees_birch_and_oak_leaf_litter" => {
            let fallen_birch = feature_random_next_f32(&mut random);
            let birch = feature_random_next_f32(&mut random);
            let fancy = feature_random_next_f32(&mut random);
            let fallen_oak = feature_random_next_f32(&mut random);
            Some(format!(
                "fallen_birch={fallen_birch:.6},birch={birch:.6},fancy={fancy:.6},fallen_oak={fallen_oak:.6}"
            ))
        }
        "trees_plains" => {
            let fancy = feature_random_next_f32(&mut random);
            let fallen_oak = feature_random_next_f32(&mut random);
            Some(format!("fancy={fancy:.6},fallen_oak={fallen_oak:.6}"))
        }
        "trees_birch" => {
            let fallen_birch = feature_random_next_f32(&mut random);
            Some(format!("fallen_birch={fallen_birch:.6}"))
        }
        _ => None,
    }
}

pub(super) fn live_tree_sapling_for_tree_config(config: LiveTreeFeatureConfig) -> &'static str {
    match config.trunk_state {
        "minecraft:birch_log" => "minecraft:birch_sapling",
        "minecraft:spruce_log" => "minecraft:spruce_sapling",
        "minecraft:jungle_log" => "minecraft:jungle_sapling",
        "minecraft:acacia_log" => "minecraft:acacia_sapling",
        "minecraft:dark_oak_log" => "minecraft:dark_oak_sapling",
        _ => "minecraft:oak_sapling",
    }
}

pub(super) fn tree_placement_filter_sapling(feature: &str) -> Option<&'static str> {
    match feature.strip_prefix("minecraft:").unwrap_or(feature) {
        "trees_birch" => Some("minecraft:birch_sapling"),
        "trees_cherry" => Some("minecraft:cherry_sapling"),
        "trees_badlands" | "trees_swamp" => Some("minecraft:oak_sapling"),
        "trees_snowy" => Some("minecraft:spruce_sapling"),
        _ => None,
    }
}

pub(super) fn live_tree_sapling_for_trunk_provider(provider: &BlockStateProviderModel) -> &'static str {
    match block_state_provider_sample(provider, 0).unwrap_or("minecraft:oak_log") {
        "minecraft:birch_log" => "minecraft:birch_sapling",
        "minecraft:spruce_log" => "minecraft:spruce_sapling",
        "minecraft:jungle_log" => "minecraft:jungle_sapling",
        "minecraft:acacia_log" => "minecraft:acacia_sapling",
        "minecraft:dark_oak_log" => "minecraft:dark_oak_sapling",
        _ => "minecraft:oak_sapling",
    }
}

pub(super) fn live_tree_sapling_survives_at(
    source_pos: ChunkPos,
    block_context: &TreeDecorationBlockContext<'_>,
    previous_source_blocks: &TreeBlockOverlay,
    origin: BlockPos,
    _sapling_state: &'static str,
) -> bool {
    let below = local_tree_block_to_world(
        source_pos,
        BlockPos {
            x: origin.x,
            y: origin.y - 1,
            z: origin.z,
        },
    );
    let below_state =
        live_tree_state_with_previous_overlay(block_context, previous_source_blocks, below);
    block_matches_tag(&below_state, "minecraft:supports_vegetation")
}

fn live_fallen_oak_tree_config() -> FallenTreeConfigurationModel {
    FallenTreeConfigurationModel {
        trunk_provider: BlockStateProviderModel::Simple("minecraft:oak_log"),
        min_log_length: 4,
        max_log_length: 7,
        stump_decorators: vec![TreeDecoratorModel::TrunkVine],
        log_decorators: vec![TreeDecoratorModel::AttachedToLogs { probability: 0.1 }],
    }
}

fn live_fallen_birch_tree_config() -> FallenTreeConfigurationModel {
    FallenTreeConfigurationModel {
        trunk_provider: BlockStateProviderModel::Simple("minecraft:birch_log"),
        min_log_length: 5,
        max_log_length: 8,
        stump_decorators: Vec::new(),
        log_decorators: vec![TreeDecoratorModel::AttachedToLogs { probability: 0.1 }],
    }
}

fn live_random_horizontal_direction(random: &mut RandomSourceKind) -> HorizontalDirection {
    match feature_random_next_i32_bound(random, 4) {
        0 => HorizontalDirection::North,
        1 => HorizontalDirection::East,
        2 => HorizontalDirection::South,
        _ => HorizontalDirection::West,
    }
}

pub(super) fn live_fallen_tree_placement_plan(
    source_pos: ChunkPos,
    block_context: &TreeDecorationBlockContext<'_>,
    previous_source_blocks: &TreeBlockOverlay,
    origin: BlockPos,
    config: &FallenTreeConfigurationModel,
    random: &mut RandomSourceKind,
) -> Option<TreePlacementPlan> {
    let trunk_state = block_state_provider_sample(&config.trunk_provider, 0)?;
    let direction = live_random_horizontal_direction(random);
    let log_length = fallen_tree_log_length(
        config.min_log_length,
        config.max_log_length,
        feature_random_next_i32_bound(
            random,
            (config.max_log_length - config.min_log_length + 1).max(1),
        ),
    );
    let distance_roll = feature_random_next_i32_bound(random, 2);
    let mut blocks = vec![TreePlacementBlock {
        pos: origin,
        state: trunk_state,
        kind: TreePlacementBlockKind::Log,
    }];

    let Some(start) = live_fallen_tree_start_pos(
        source_pos,
        block_context,
        previous_source_blocks,
        &blocks,
        origin,
        direction,
        distance_roll,
    ) else {
        return Some(TreePlacementPlan { blocks });
    };

    if !live_fallen_tree_can_place_log(
        source_pos,
        block_context,
        previous_source_blocks,
        &blocks,
        start,
        direction,
        log_length,
    ) {
        return Some(TreePlacementPlan { blocks });
    }

    for i in 0..log_length.max(0) {
        blocks.push(TreePlacementBlock {
            pos: offset_horizontal(start, direction, i),
            state: rotated_log_state(trunk_state, direction),
            kind: TreePlacementBlockKind::Log,
        });
    }
    Some(TreePlacementPlan { blocks })
}

fn live_fallen_tree_start_pos(
    source_pos: ChunkPos,
    block_context: &TreeDecorationBlockContext<'_>,
    previous_source_blocks: &TreeBlockOverlay,
    planned_blocks: &[TreePlacementBlock],
    origin: BlockPos,
    direction: HorizontalDirection,
    distance_roll: i32,
) -> Option<BlockPos> {
    let mut pos = offset_horizontal(origin, direction, 2 + distance_roll.rem_euclid(2));
    pos.y += 1;
    for _ in 0..6 {
        if live_fallen_tree_may_place_on(
            source_pos,
            block_context,
            previous_source_blocks,
            planned_blocks,
            pos,
        ) {
            return Some(pos);
        }
        pos.y -= 1;
    }
    None
}

fn live_fallen_tree_can_place_log(
    source_pos: ChunkPos,
    block_context: &TreeDecorationBlockContext<'_>,
    previous_source_blocks: &TreeBlockOverlay,
    planned_blocks: &[TreePlacementBlock],
    start: BlockPos,
    direction: HorizontalDirection,
    log_length: i32,
) -> bool {
    let mut ground_gap = 0;
    for i in 0..log_length.max(0) {
        let pos = offset_horizontal(start, direction, i);
        let state = live_tree_state_with_planned_blocks(
            source_pos,
            block_context,
            previous_source_blocks,
            planned_blocks,
            pos,
        );
        if !tree_valid_pos(&state) {
            return false;
        }
        if !live_fallen_tree_is_over_solid_ground(
            source_pos,
            block_context,
            previous_source_blocks,
            planned_blocks,
            pos,
        ) {
            ground_gap += 1;
            if ground_gap > 2 {
                return false;
            }
        } else {
            ground_gap = 0;
        }
    }
    true
}

fn live_fallen_tree_may_place_on(
    source_pos: ChunkPos,
    block_context: &TreeDecorationBlockContext<'_>,
    previous_source_blocks: &TreeBlockOverlay,
    planned_blocks: &[TreePlacementBlock],
    pos: BlockPos,
) -> bool {
    let state = live_tree_state_with_planned_blocks(
        source_pos,
        block_context,
        previous_source_blocks,
        planned_blocks,
        pos,
    );
    tree_valid_pos(&state)
        && live_fallen_tree_is_over_solid_ground(
            source_pos,
            block_context,
            previous_source_blocks,
            planned_blocks,
            pos,
        )
}

fn live_fallen_tree_is_over_solid_ground(
    source_pos: ChunkPos,
    block_context: &TreeDecorationBlockContext<'_>,
    previous_source_blocks: &TreeBlockOverlay,
    planned_blocks: &[TreePlacementBlock],
    pos: BlockPos,
) -> bool {
    let below = BlockPos {
        x: pos.x,
        y: pos.y - 1,
        z: pos.z,
    };
    let state = live_tree_state_with_planned_blocks(
        source_pos,
        block_context,
        previous_source_blocks,
        planned_blocks,
        below,
    );
    tree_decorator_solid_render(&state)
}

pub(super) fn live_tree_state_with_planned_blocks<'a>(
    source_pos: ChunkPos,
    block_context: &'a TreeDecorationBlockContext<'a>,
    previous_source_blocks: &TreeBlockOverlay,
    planned_blocks: &[TreePlacementBlock],
    local_pos: BlockPos,
) -> Cow<'a, str> {
    planned_blocks
        .iter()
        .rev()
        .find_map(|block| {
            (block.pos.x == local_pos.x && block.pos.y == local_pos.y && block.pos.z == local_pos.z)
                .then_some(Cow::Borrowed(block.state))
        })
        .unwrap_or_else(|| {
            live_tree_state_with_previous_overlay(
                block_context,
                previous_source_blocks,
                local_tree_block_to_world(source_pos, local_pos),
            )
        })
}
