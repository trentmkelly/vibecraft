use super::*;

pub fn chunk_pyramid_direct_dependencies(
    kind: ChunkPyramidKind,
    target: &str,
) -> Option<Vec<&'static str>> {
    chunk_pyramid_dependencies(kind, target, false)
}

pub fn chunk_pyramid_accumulated_dependencies(
    kind: ChunkPyramidKind,
    target: &str,
) -> Option<Vec<&'static str>> {
    chunk_pyramid_dependencies(kind, target, true)
}

pub fn chunk_generation_task_worst_case_radius(target: &str) -> Option<i32> {
    chunk_pyramid_accumulated_radius_of(ChunkPyramidKind::Generation, target, "minecraft:empty")
}

pub fn worldgen_region_access_plan(
    kind: ChunkPyramidKind,
    target: &str,
    center: ChunkPos,
) -> Option<WorldGenRegionAccessPlan> {
    let target = chunk_status(target)?;
    let direct_dependencies = chunk_pyramid_direct_dependencies(kind, target.id)?;
    let read_radius = direct_dependencies.len().saturating_sub(1) as i32;
    let mut chunks = Vec::with_capacity(((read_radius * 2 + 1) * (read_radius * 2 + 1)) as usize);
    for x in center.x - read_radius..=center.x + read_radius {
        for z in center.z - read_radius..=center.z + read_radius {
            let distance = (center.x - x).abs().max((center.z - z).abs());
            let max_read_status = direct_dependencies
                .get(distance as usize)
                .copied()
                .unwrap_or(target.id);
            chunks.push(WorldGenRegionAccessChunk {
                chunk: ChunkPos { x, z },
                distance,
                max_read_status,
                can_write_blocks: distance <= target.block_state_write_radius,
            });
        }
    }
    Some(WorldGenRegionAccessPlan {
        center,
        target_status: target.id,
        block_state_write_radius: target.block_state_write_radius,
        chunks,
    })
}

pub fn worldgen_region_can_read_status(
    kind: ChunkPyramidKind,
    target: &str,
    center: ChunkPos,
    chunk: ChunkPos,
    requested_status: &str,
) -> Option<bool> {
    let plan = worldgen_region_access_plan(kind, target, center)?;
    let entry = plan.chunks.iter().find(|entry| entry.chunk == chunk)?;
    chunk_status_is_or_before(requested_status, entry.max_read_status)
}

pub fn worldgen_region_can_write_block(
    kind: ChunkPyramidKind,
    target: &str,
    center: ChunkPos,
    chunk: ChunkPos,
) -> Option<bool> {
    let plan = worldgen_region_access_plan(kind, target, center)?;
    plan.chunks
        .iter()
        .find(|entry| entry.chunk == chunk)
        .map(|entry| entry.can_write_blocks)
        .or(Some(false))
}

pub fn chunk_generation_task_layer_radius(
    target: &str,
    status: &str,
    needs_generation: bool,
) -> Option<i32> {
    let kind = if needs_generation {
        ChunkPyramidKind::Generation
    } else {
        ChunkPyramidKind::Loading
    };
    chunk_pyramid_accumulated_radius_of(kind, target, status)
}

pub fn chunk_generation_task_can_load_without_generation<F>(
    target: &str,
    center_x: i32,
    center_z: i32,
    mut persisted_status_at: F,
) -> Option<bool>
where
    F: FnMut(i32, i32) -> Option<&'static str>,
{
    let target = chunk_status(target)?;
    if target.id == "minecraft:empty" {
        return Some(true);
    }

    let center_status = chunk_status(persisted_status_at(center_x, center_z)?)?;
    if chunk_status_is_before(center_status.id, target.id)? {
        return Some(false);
    }

    let dependencies =
        chunk_pyramid_accumulated_dependencies(ChunkPyramidKind::Loading, target.id)?;
    let range = dependencies.len().saturating_sub(1) as i32;

    for x in center_x - range..=center_x + range {
        for z in center_z - range..=center_z + range {
            let distance = (center_x - x).abs().max((center_z - z).abs()) as usize;
            let required_status = dependencies.get(distance).copied()?;
            let persisted_status = chunk_status(persisted_status_at(x, z)?)?;
            if chunk_status_is_before(persisted_status.id, required_status)? {
                return Some(false);
            }
        }
    }

    Some(true)
}

pub fn chunk_generation_task_next_layer(
    scheduled_status: Option<&str>,
    needs_generation: bool,
    can_load_without_generation: bool,
) -> Option<ChunkGenerationLayerPlan> {
    match scheduled_status {
        None => Some(ChunkGenerationLayerPlan {
            status: "minecraft:empty",
            needs_generation,
        }),
        Some(status) => {
            let status = chunk_status(status)?;
            if !needs_generation && status.id == "minecraft:empty" && !can_load_without_generation {
                return Some(ChunkGenerationLayerPlan {
                    status: "minecraft:empty",
                    needs_generation: true,
                });
            }

            let next_status = CHUNK_STATUS_PIPELINE.get(status.index + 1)?;
            Some(ChunkGenerationLayerPlan {
                status: next_status.id,
                needs_generation,
            })
        }
    }
}

pub fn chunk_generation_task_chunk_step(
    status: &str,
    persisted_status: Option<&str>,
    needs_generation: bool,
) -> Option<ChunkGenerationChunkStepPlan> {
    let status = chunk_status(status)?;
    let generate = persisted_status
        .and_then(|persisted_status| chunk_status_is_after(status.id, persisted_status))
        .unwrap_or(false);

    if generate && !needs_generation {
        return Some(ChunkGenerationChunkStepPlan::UnexpectedGeneration);
    }

    let pyramid = if generate {
        ChunkPyramidKind::Generation
    } else {
        ChunkPyramidKind::Loading
    };

    Some(ChunkGenerationChunkStepPlan::Apply { pyramid, generate })
}

pub fn chunk_generation_task_wait_for_scheduled_layer(
    scheduled_layer: &[ChunkGenerationFutureState],
) -> ChunkGenerationWaitPlan {
    let mut remaining_layer = scheduled_layer.to_vec();
    let mut marked_for_cancellation = false;

    while let Some(result_now) = remaining_layer.last().copied() {
        match result_now {
            ChunkGenerationFutureState::Pending => {
                return ChunkGenerationWaitPlan {
                    waiting_for_index: Some(remaining_layer.len() - 1),
                    remaining_layer,
                    marked_for_cancellation,
                };
            }
            ChunkGenerationFutureState::Success => {
                remaining_layer.pop();
            }
            ChunkGenerationFutureState::Failure => {
                remaining_layer.pop();
                marked_for_cancellation = true;
            }
        }
    }

    ChunkGenerationWaitPlan {
        waiting_for_index: None,
        remaining_layer,
        marked_for_cancellation,
    }
}

pub fn chunk_generation_task_schedule_layer_positions<I>(
    target: &str,
    status: &str,
    needs_generation: bool,
    center_x: i32,
    center_z: i32,
    mut outcomes: I,
) -> Option<ChunkGenerationScheduleLayerPlan>
where
    I: Iterator<Item = bool>,
{
    let radius = chunk_generation_task_layer_radius(target, status, needs_generation)?;
    let mut visited_positions = Vec::new();
    let mut stopped_early = false;

    for x in center_x - radius..=center_x + radius {
        for z in center_z - radius..=center_z + radius {
            match outcomes.next() {
                Some(true) => visited_positions.push((x, z)),
                Some(false) => {
                    visited_positions.push((x, z));
                    stopped_early = true;
                    return Some(ChunkGenerationScheduleLayerPlan {
                        radius,
                        visited_positions,
                        stopped_early,
                    });
                }
                None => {
                    stopped_early = true;
                    return Some(ChunkGenerationScheduleLayerPlan {
                        radius,
                        visited_positions,
                        stopped_early,
                    });
                }
            }
        }
    }

    Some(ChunkGenerationScheduleLayerPlan {
        radius,
        visited_positions,
        stopped_early,
    })
}

pub fn chunk_generation_task_run_until_wait_decision(
    target: &str,
    scheduled_status: Option<&str>,
    needs_generation: bool,
    marked_for_cancellation: bool,
    can_load_without_generation: bool,
    scheduled_layer: &[ChunkGenerationFutureState],
) -> Option<ChunkGenerationRunPlan> {
    let target = chunk_status(target)?;
    let wait_plan = chunk_generation_task_wait_for_scheduled_layer(scheduled_layer);
    if let Some(waiting_for_index) = wait_plan.waiting_for_index {
        return Some(ChunkGenerationRunPlan::Waiting {
            waiting_for_index,
            remaining_layer: wait_plan.remaining_layer,
            marked_for_cancellation: wait_plan.marked_for_cancellation,
        });
    }

    let marked_for_cancellation = marked_for_cancellation || wait_plan.marked_for_cancellation;
    let target_reached = scheduled_status
        .and_then(chunk_status)
        .is_some_and(|scheduled| scheduled.id == target.id);
    if marked_for_cancellation || target_reached {
        return Some(ChunkGenerationRunPlan::Released);
    }

    chunk_generation_task_next_layer(
        scheduled_status,
        needs_generation,
        can_load_without_generation,
    )
    .map(ChunkGenerationRunPlan::Schedule)
}

pub(super) fn chunk_pyramid_dependencies(
    kind: ChunkPyramidKind,
    target: &str,
    accumulated: bool,
) -> Option<Vec<&'static str>> {
    let target = chunk_status(target)?;
    let mut accumulated_by_status: Vec<Vec<&'static str>> = Vec::new();

    for status in CHUNK_STATUS_PIPELINE.iter().take(target.index + 1) {
        let direct = chunk_pyramid_direct_dependencies_for_status(kind, status)?;
        if !accumulated {
            accumulated_by_status.push(direct);
            continue;
        }

        if status.index == 0 {
            accumulated_by_status.push(direct);
            continue;
        }

        let parent_accumulated = accumulated_by_status.get(status.index - 1)?;
        let parent_id = CHUNK_STATUS_PIPELINE.get(status.index - 1)?.id;
        let parent_radius = direct
            .iter()
            .rposition(|dependency| chunk_status_is_or_after(dependency, parent_id) == Some(true))
            .unwrap_or(0);
        let len = direct.len().max(parent_radius + parent_accumulated.len());
        let mut combined = Vec::with_capacity(len);

        for distance in 0..len {
            let distance_in_parent = distance as isize - parent_radius as isize;
            let dependency = if distance_in_parent < 0
                || distance_in_parent as usize >= parent_accumulated.len()
            {
                direct[distance]
            } else if distance >= direct.len() {
                parent_accumulated[distance_in_parent as usize]
            } else {
                chunk_status_max(
                    direct[distance],
                    parent_accumulated[distance_in_parent as usize],
                )?
            };
            combined.push(dependency);
        }

        accumulated_by_status.push(combined);
    }

    accumulated_by_status.get(target.index).cloned()
}

pub(super) fn chunk_pyramid_accumulated_radius_of(
    kind: ChunkPyramidKind,
    target: &str,
    dependency: &str,
) -> Option<i32> {
    if chunk_status(target)?.id == chunk_status(dependency)?.id {
        return Some(0);
    }

    let dependencies = chunk_pyramid_accumulated_dependencies(kind, target)?;
    chunk_dependencies_radius_of(&dependencies, dependency)
}

pub(super) fn chunk_dependencies_radius_of(dependencies: &[&'static str], dependency: &str) -> Option<i32> {
    let dependency = chunk_status(dependency)?;
    if dependencies.is_empty() {
        return None;
    }

    let first = chunk_status(dependencies[0])?;
    if dependency.index > first.index {
        return None;
    }

    let mut radius_by_dependency = vec![0_i32; first.index + 1];
    for (radius, status_id) in dependencies.iter().enumerate() {
        let status = chunk_status(status_id)?;
        for status_index in 0..=status.index {
            radius_by_dependency[status_index] = radius as i32;
        }
    }

    radius_by_dependency.get(dependency.index).copied()
}

pub(super) fn chunk_pyramid_direct_dependencies_for_status(
    kind: ChunkPyramidKind,
    status: &ChunkStatusEntry,
) -> Option<Vec<&'static str>> {
    if status.index == 0 {
        return Some(Vec::new());
    }

    let mut dependencies = vec![status.parent];
    for requirement in chunk_pyramid_direct_requirements(kind, status) {
        let required = chunk_status(requirement.status)?;
        if required.index >= status.index {
            return None;
        }

        let new_len = requirement.radius as usize + 1;
        if new_len > dependencies.len() {
            dependencies.resize(new_len, required.id);
        }

        let update_len = new_len.min(dependencies.len());
        for dependency in dependencies.iter_mut().take(update_len) {
            *dependency = chunk_status_max(dependency, required.id)?;
        }
    }

    Some(dependencies)
}

pub(super) fn chunk_pyramid_direct_requirements(
    kind: ChunkPyramidKind,
    status: &ChunkStatusEntry,
) -> &'static [ChunkStatusRequirement] {
    match kind {
        ChunkPyramidKind::Generation => status.requirements,
        ChunkPyramidKind::Loading => match status.id {
            "minecraft:light" => INITIALIZE_LIGHT_DISTANCE_1_REQUIREMENT,
            _ => NO_REQUIREMENTS,
        },
    }
}
