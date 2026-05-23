use super::*;

pub fn placement_modifier_type(id: &str) -> Option<&'static str> {
    let name = id.strip_prefix("minecraft:").unwrap_or(id);
    WORLDGEN_TYPE_REGISTRIES
        .iter()
        .find(|registry| registry.id == "minecraft:placement_modifier_type")?
        .entries
        .iter()
        .copied()
        .find(|entry| entry.strip_prefix("minecraft:").unwrap_or(entry) == name)
}

pub fn placement_modifier_positions(
    modifier: PlacementModifier,
    origin: BlockPos,
    first_roll: i32,
    second_roll: i32,
    third_roll: i32,
) -> Vec<BlockPos> {
    match modifier {
        PlacementModifier::RarityFilter { chance } => {
            if chance > 0 && first_roll.rem_euclid(chance) == 0 {
                vec![origin]
            } else {
                Vec::new()
            }
        }
        PlacementModifier::Count { count } => vec![origin; count.max(0) as usize],
        PlacementModifier::CountProvider {
            sampled_count,
            provider: _,
        } => vec![origin; sampled_count.max(0) as usize],
        PlacementModifier::NoiseBasedCount {
            noise_to_count_ratio,
            noise_factor: _,
            noise_offset,
            sampled_noise,
        } => {
            let count =
                ((sampled_noise + noise_offset) * noise_to_count_ratio as f64).ceil() as i32;
            vec![origin; count.max(0) as usize]
        }
        PlacementModifier::NoiseThresholdCount {
            noise_level,
            below_noise,
            above_noise,
            sampled_noise,
        } => {
            let count = if sampled_noise < noise_level {
                below_noise
            } else {
                above_noise
            };
            vec![origin; count.max(0) as usize]
        }
        PlacementModifier::CountOnEveryLayer { positions } => positions.to_vec(),
        PlacementModifier::InSquare => vec![BlockPos {
            x: origin.x + first_roll.rem_euclid(16),
            y: origin.y,
            z: origin.z + second_roll.rem_euclid(16),
        }],
        PlacementModifier::RandomOffset {
            xz_spread,
            y_spread,
        } => {
            let xz_bound = xz_spread.abs() * 2 + 1;
            let y_bound = y_spread.abs() * 2 + 1;
            vec![BlockPos {
                x: origin.x + first_roll.rem_euclid(xz_bound) - xz_spread.abs(),
                y: origin.y + second_roll.rem_euclid(y_bound) - y_spread.abs(),
                z: origin.z + third_roll.rem_euclid(xz_bound) - xz_spread.abs(),
            }]
        }
        PlacementModifier::Fixed { positions } => {
            let chunk_x = origin.x.div_euclid(16);
            let chunk_z = origin.z.div_euclid(16);
            positions
                .iter()
                .copied()
                .filter(|pos| pos.x.div_euclid(16) == chunk_x && pos.z.div_euclid(16) == chunk_z)
                .collect()
        }
        PlacementModifier::BiomeFilter
        | PlacementModifier::BlockPredicateFilter { .. }
        | PlacementModifier::EnvironmentScan { .. }
        | PlacementModifier::SurfaceWaterDepthFilter { .. }
        | PlacementModifier::SurfaceRelativeThresholdFilter { .. }
        | PlacementModifier::Heightmap { .. }
        | PlacementModifier::HeightRange { .. } => Vec::new(),
    }
}

pub fn placed_feature_positions(
    modifiers: &[PlacementModifier],
    origin: BlockPos,
    context: PlacementContextModel,
    rolls: &[(i32, i32, i32)],
) -> Vec<BlockPos> {
    modifiers
        .iter()
        .enumerate()
        .fold(vec![origin], |positions, (index, modifier)| {
            let (first_roll, second_roll, third_roll) =
                rolls.get(index).copied().unwrap_or((0, 0, 0));
            positions
                .into_iter()
                .flat_map(|position| {
                    placement_modifier_positions_with_context(
                        *modifier,
                        position,
                        context,
                        first_roll,
                        second_roll,
                        third_roll,
                    )
                })
                .collect()
        })
}

pub fn placed_feature_invocations(
    configured_feature_id: &'static str,
    modifiers: &[PlacementModifier],
    origin: BlockPos,
    context: PlacementContextModel,
    rolls: &[(i32, i32, i32)],
) -> Result<Vec<PlacedFeatureInvocation>, String> {
    let feature = configured_feature(configured_feature_id)
        .ok_or_else(|| format!("unknown configured feature {configured_feature_id}"))?;
    Ok(placed_feature_positions(modifiers, origin, context, rolls)
        .into_iter()
        .map(|pos| PlacedFeatureInvocation {
            feature: feature.id,
            source: feature.source,
            pos,
        })
        .collect())
}

pub fn placement_modifier_positions_with_context(
    modifier: PlacementModifier,
    origin: BlockPos,
    context: PlacementContextModel,
    first_roll: i32,
    second_roll: i32,
    third_roll: i32,
) -> Vec<BlockPos> {
    match modifier {
        PlacementModifier::BiomeFilter => {
            if context.biome_allows_feature {
                vec![origin]
            } else {
                Vec::new()
            }
        }
        PlacementModifier::BlockPredicateFilter { predicate } => {
            if block_predicate_test(predicate, context.block_predicate, origin.y) {
                vec![origin]
            } else {
                Vec::new()
            }
        }
        PlacementModifier::SurfaceWaterDepthFilter { max_water_depth } => {
            if context.world_surface_height - context.ocean_floor_height <= max_water_depth {
                vec![origin]
            } else {
                Vec::new()
            }
        }
        PlacementModifier::SurfaceRelativeThresholdFilter {
            heightmap,
            min_inclusive,
            max_inclusive,
        } => {
            let surface = placement_context_height(context, heightmap);
            let min_y = surface + min_inclusive;
            let max_y = surface + max_inclusive;
            if (min_y..=max_y).contains(&origin.y) {
                vec![origin]
            } else {
                Vec::new()
            }
        }
        PlacementModifier::Heightmap { heightmap } => {
            let height = placement_context_height(context, heightmap);
            if height > context.min_y {
                vec![BlockPos {
                    x: origin.x,
                    y: height,
                    z: origin.z,
                }]
            } else {
                Vec::new()
            }
        }
        PlacementModifier::HeightRange { height } => vec![BlockPos {
            x: origin.x,
            y: height_provider_sample_with_rolls(
                height,
                WorldGenerationHeightContext {
                    min_y: context.min_y,
                    height: context.block_predicate.height,
                },
                first_roll,
                second_roll,
                third_roll,
            ),
            z: origin.z,
        }],
        PlacementModifier::EnvironmentScan {
            direction_y,
            target_condition,
            allowed_search_condition,
            max_steps,
            states,
        } => environment_scan_placement_position(
            origin,
            context,
            direction_y,
            target_condition,
            allowed_search_condition,
            max_steps,
            states,
        )
        .into_iter()
        .collect(),
        _ => placement_modifier_positions(modifier, origin, first_roll, second_roll, third_roll),
    }
}

pub fn environment_scan_placement_position(
    origin: BlockPos,
    fallback_context: PlacementContextModel,
    direction_y: i32,
    target_condition: BlockPredicate,
    allowed_search_condition: BlockPredicate,
    max_steps: i32,
    states: &[BlockPredicateContext],
) -> Option<BlockPos> {
    if !(1..=32).contains(&max_steps) || direction_y == 0 {
        return None;
    }

    let step = direction_y.signum();
    for i in 0..=max_steps {
        let pos = BlockPos {
            x: origin.x,
            y: origin.y + i * step,
            z: origin.z,
        };
        if pos.y < fallback_context.min_y
            || pos.y >= fallback_context.min_y + fallback_context.block_predicate.height
        {
            return None;
        }

        let block_context = states
            .get(i as usize)
            .copied()
            .unwrap_or(fallback_context.block_predicate);
        if i == 0 && !block_predicate_test(allowed_search_condition, block_context, pos.y) {
            return None;
        }
        if block_predicate_test(target_condition, block_context, pos.y) {
            return Some(pos);
        }
        if i == max_steps {
            break;
        }
        if !block_predicate_test(allowed_search_condition, block_context, pos.y) {
            return None;
        }
    }
    None
}

fn placement_context_height(context: PlacementContextModel, heightmap: HeightmapKind) -> i32 {
    match heightmap {
        HeightmapKind::WorldSurface | HeightmapKind::WorldSurfaceWg => context.world_surface_height,
        HeightmapKind::OceanFloor
        | HeightmapKind::OceanFloorWg
        | HeightmapKind::MotionBlocking
        | HeightmapKind::MotionBlockingNoLeaves => context.ocean_floor_height,
    }
}

