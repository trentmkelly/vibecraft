use super::*;

#[derive(Debug, Default)]
pub(super) struct TreeDecorationDiagnostics {
    pub(super) context_chunk_build_ms: u128,
    pub(super) context_heightmap_ms: u128,
    pub(super) context_chunks: usize,
    pub(super) source_total_us: u128,
    pub(super) sources_evaluated: usize,
    pub(super) source_context_clone_us: u128,
    pub(super) source_biome_steps_ms: u128,
    pub(super) source_feature_sort_ms: u128,
    pub(super) source_plan_ms: u128,
    pub(super) feature_calls_total: usize,
    pub(super) tree_feature_calls: usize,
    pub(super) tree_attempts: usize,
    pub(super) tree_candidates: usize,
    pub(super) candidate_biome_ms: u128,
    pub(super) validation_ms: u128,
    pub(super) validation_accepts: usize,
    pub(super) validation_rejects: usize,
    pub(super) placement_plan_ms: u128,
    pub(super) placement_plan_blocks: usize,
    pub(super) filter_ms: u128,
    pub(super) filtered_blocks: usize,
    pub(super) source_write_filter_ms: u128,
    pub(super) source_context_map_ms: u128,
    pub(super) output_blocks_seen: usize,
    pub(super) output_blocks_in_target: usize,
    pub(super) output_blocks_written: usize,
}

pub(super) fn local_tree_block_to_world(source_pos: ChunkPos, pos: BlockPos) -> BlockPos {
    BlockPos {
        x: source_pos.x * 16 + pos.x,
        y: pos.y,
        z: source_pos.z * 16 + pos.z,
    }
}

pub(super) fn world_tree_block_to_local(source_pos: ChunkPos, pos: BlockPos) -> BlockPos {
    BlockPos {
        x: pos.x - source_pos.x * 16,
        y: pos.y,
        z: pos.z - source_pos.z * 16,
    }
}

pub(super) type TreeBlockOverlay = HashMap<(i32, i32, i32), &'static str>;

pub(super) fn local_tree_block_to_world_key(source_pos: ChunkPos, pos: BlockPos) -> (i32, i32, i32) {
    (source_pos.x * 16 + pos.x, pos.y, source_pos.z * 16 + pos.z)
}

pub(super) fn live_tree_state_with_previous_blocks(
    source_pos: ChunkPos,
    block_context: &TreeDecorationBlockContext<'_>,
    previous_source_blocks: &[TreePlacementBlock],
    world_pos: BlockPos,
) -> String {
    previous_source_blocks
        .iter()
        .rev()
        .find_map(|block| {
            let block_world_x = source_pos.x * 16 + block.pos.x;
            let block_world_z = source_pos.z * 16 + block.pos.z;
            (block_world_x == world_pos.x
                && block.pos.y == world_pos.y
                && block_world_z == world_pos.z)
                .then_some(block.state.to_string())
        })
        .or_else(|| {
            block_context
                .block_state(world_pos.x, world_pos.y, world_pos.z)
                .map(|state| block_state_id(state).to_string())
        })
        .unwrap_or_else(|| "minecraft:air".to_string())
}

pub(super) fn live_tree_state_with_previous_overlay<'a>(
    block_context: &'a TreeDecorationBlockContext<'_>,
    previous_source_blocks: &TreeBlockOverlay,
    world_pos: BlockPos,
) -> Cow<'a, str> {
    previous_source_blocks
        .get(&(world_pos.x, world_pos.y, world_pos.z))
        .map(|state| Cow::Borrowed(*state))
        .or_else(|| {
            block_context
                .block_state(world_pos.x, world_pos.y, world_pos.z)
                .map(|state| Cow::Borrowed(block_state_id(state)))
        })
        .unwrap_or(Cow::Borrowed("minecraft:air"))
}

pub(super) fn live_straight_blob_tree_placement_plan(
    origin: BlockPos,
    trunk: TrunkPlacerModel,
    clipped_tree_height: i32,
    foliage: FoliagePlacerModel,
    trunk_state: &'static str,
    foliage_state: &'static str,
    below_trunk_state: &'static str,
    rand_a: i32,
    rand_b: i32,
    random: &mut RandomSourceKind,
) -> Result<TreePlacementPlan, String> {
    validate_trunk_placer(trunk)?;
    validate_foliage_placer(foliage)?;
    if trunk.kind != TrunkPlacerKind::Straight {
        return Err("live blob tree placement requires a straight trunk placer".to_string());
    }
    let FoliagePlacerKind::Blob {
        height: foliage_height,
    } = foliage.kind
    else {
        return Err("live straight tree placement currently requires blob foliage".to_string());
    };

    let leaf_radius = sample_inclusive_i32(foliage.radius_min, foliage.radius_max, rand_a);
    let foliage_offset = sample_inclusive_i32(foliage.offset_min, foliage.offset_max, rand_b);
    let foliage_origin = BlockPos {
        x: origin.x,
        y: origin.y + clipped_tree_height + foliage_offset,
        z: origin.z,
    };
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
    for y in 0..clipped_tree_height {
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

    for y_offset in (-foliage_height..=0).rev() {
        let current_radius = (leaf_radius - 1 - y_offset / 2).max(0);
        place_live_blob_leaves_row(
            &mut blocks,
            foliage_origin,
            current_radius,
            y_offset,
            foliage_state,
            random,
        );
    }

    Ok(TreePlacementPlan { blocks })
}

#[derive(Clone, Copy)]
pub(super) enum TreeContextChunkRef<'a> {
    Full(&'a LevelChunk),
    Lightweight(&'a LightweightTreeContextChunk),
}

impl TreeContextChunkRef<'_> {
    pub(super) fn block_state(&self, world_x: i32, world_y: i32, world_z: i32) -> Option<&str> {
        match self {
            Self::Full(chunk) => chunk.get_block_state_name(world_x, world_y, world_z),
            Self::Lightweight(chunk) => {
                Some(chunk.synthetic_block_state(world_x, world_y, world_z))
            }
        }
    }
}

pub(super) struct TreeDecorationBlockContext<'a> {
    pub(super) source_pos: ChunkPos,
    pub(super) source_chunk: TreeContextChunkRef<'a>,
    pub(super) target_pos: ChunkPos,
    pub(super) target_chunk: &'a LevelChunk,
    pub(super) generated_chunks: &'a HashMap<ChunkPos, TreeContextChunkRef<'a>>,
    pub(super) region_overlay: Option<&'a TreeBlockOverlay>,
}

impl TreeDecorationBlockContext<'_> {
    pub(super) fn block_state(&self, world_x: i32, world_y: i32, world_z: i32) -> Option<&str> {
        if let Some(state) = self
            .region_overlay
            .and_then(|overlay| overlay.get(&(world_x, world_y, world_z)).copied())
        {
            return Some(state);
        }
        let chunk_pos = ChunkPos {
            x: world_x.div_euclid(16),
            z: world_z.div_euclid(16),
        };
        if chunk_pos == self.source_pos {
            return self.source_chunk.block_state(world_x, world_y, world_z);
        }
        if chunk_pos == self.target_pos {
            return self
                .target_chunk
                .get_block_state_name(world_x, world_y, world_z);
        }
        self.generated_chunks
            .get(&chunk_pos)
            .and_then(|chunk| chunk.block_state(world_x, world_y, world_z))
    }
}

pub(super) fn live_tree_replaceable_rows(
    block_context: &TreeDecorationBlockContext<'_>,
    origin: BlockPos,
    tree_height: i32,
    min_size: FeatureSizeModel,
    settings: &NoiseGeneratorSettings,
) -> Vec<Vec<String>> {
    (0..=tree_height + 1)
        .map(|y_offset| {
            let radius = feature_size_at_height(min_size, tree_height, y_offset);
            let mut row = Vec::new();
            for dx in -radius..=radius {
                for dz in -radius..=radius {
                    let local_x = origin.x + dx;
                    let local_z = origin.z + dz;
                    let world_y = origin.y + y_offset;
                    let world_x = block_context.source_pos.x * 16 + local_x;
                    let world_z = block_context.source_pos.z * 16 + local_z;
                    let state = if world_y < settings.noise.min_y
                        || world_y >= settings.noise.min_y + settings.noise.height
                    {
                        "minecraft:air".to_string()
                    } else {
                        block_context
                            .block_state(world_x, world_y, world_z)
                            .unwrap_or("minecraft:air")
                            .to_string()
                    };
                    row.push(state);
                }
            }
            row
        })
        .collect()
}

pub(super) fn live_tree_can_place_in_chunk(
    block_context: &TreeDecorationBlockContext<'_>,
    origin: BlockPos,
    config: LiveTreeFeatureConfig,
    rand_a: i32,
    rand_b: i32,
    settings: &NoiseGeneratorSettings,
) -> bool {
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
    let tree_height = trunk_placer_height(trunk, rand_a, rand_b);
    let build_min_y = settings.noise.min_y;
    let build_max_y = settings.noise.min_y + settings.noise.height;
    let min_y = origin.y;
    let max_y = origin.y + tree_height + 1;
    if min_y < build_min_y + 1 || max_y > build_max_y + 1 {
        return false;
    }

    let mut clipped_tree_height = tree_height;
    'height_scan: for y_offset in 0..=tree_height + 1 {
        let radius = feature_size_at_height(config.minimum_size, tree_height, y_offset);
        for dx in -radius..=radius {
            for dz in -radius..=radius {
                let local_x = origin.x + dx;
                let local_z = origin.z + dz;
                let world_y = origin.y + y_offset;
                if world_y < build_min_y || world_y >= build_max_y {
                    continue;
                }
                let world_x = block_context.source_pos.x * 16 + local_x;
                let world_z = block_context.source_pos.z * 16 + local_z;
                if block_context
                    .block_state(world_x, world_y, world_z)
                    .is_some_and(|state| !tree_trunk_free_pos(state))
                {
                    clipped_tree_height = y_offset - 2;
                    break 'height_scan;
                }
            }
        }
    }

    clipped_tree_height >= tree_height
        || config
            .min_clipped_height
            .is_some_and(|min| clipped_tree_height >= min)
}

pub(super) fn live_tree_clipped_height_with_previous_blocks(
    block_context: &TreeDecorationBlockContext<'_>,
    previous_source_blocks: &TreeBlockOverlay,
    origin: BlockPos,
    config: LiveTreeFeatureConfig,
    rand_a: i32,
    rand_b: i32,
    settings: &NoiseGeneratorSettings,
) -> Option<i32> {
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
    let tree_height = trunk_placer_height(trunk, rand_a, rand_b);
    let build_min_y = settings.noise.min_y;
    let build_max_y = settings.noise.min_y + settings.noise.height;
    let min_y = origin.y;
    let max_y = origin.y + tree_height + 1;
    if min_y < build_min_y + 1 || max_y > build_max_y + 1 {
        return None;
    }

    let mut clipped_tree_height = tree_height;
    'height_scan: for y_offset in 0..=tree_height + 1 {
        let radius = feature_size_at_height(config.minimum_size, tree_height, y_offset);
        for dx in -radius..=radius {
            for dz in -radius..=radius {
                let local_x = origin.x + dx;
                let local_z = origin.z + dz;
                let world_y = origin.y + y_offset;
                if world_y < build_min_y || world_y >= build_max_y {
                    continue;
                }
                let world_x = block_context.source_pos.x * 16 + local_x;
                let world_z = block_context.source_pos.z * 16 + local_z;
                let state = live_tree_state_with_previous_overlay(
                    block_context,
                    previous_source_blocks,
                    BlockPos {
                        x: world_x,
                        y: world_y,
                        z: world_z,
                    },
                );
                if !tree_trunk_free_pos(&state) {
                    clipped_tree_height = y_offset - 2;
                    break 'height_scan;
                }
            }
        }
    }

    if clipped_tree_height >= tree_height
        || config
            .min_clipped_height
            .is_some_and(|min| clipped_tree_height >= min)
    {
        Some(clipped_tree_height)
    } else {
        None
    }
}
