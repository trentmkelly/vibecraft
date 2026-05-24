use super::*;

pub fn disk_placement_plan(
    origin: BlockPos,
    config: &DiskConfigurationModel,
    column_contexts: &[(BlockPos, BlockPredicateContext)],
    provider_rolls: &[i32],
) -> Vec<DiskPlacementBlock> {
    let radius = sample_int_provider_from_roll(config.radius, 0).clamp(0, 8);
    let half_height = config.half_height.clamp(0, 4);
    let top = origin.y + half_height;
    let bottom_exclusive = origin.y - half_height - 1;
    let mut placed = Vec::new();
    let mut provider_index = 0;
    for z in origin.z - radius..=origin.z + radius {
        for x in origin.x - radius..=origin.x + radius {
            let dx = x - origin.x;
            let dz = z - origin.z;
            if dx * dx + dz * dz > radius * radius {
                continue;
            }
            let mut placed_above = false;
            for y in (bottom_exclusive + 1..=top).rev() {
                let pos = BlockPos { x, y, z };
                let Some((_, context)) = column_contexts
                    .iter()
                    .find(|(context_pos, _)| *context_pos == pos)
                else {
                    placed_above = false;
                    continue;
                };
                if block_predicate_test(config.target, *context, y) {
                    let provider_roll = provider_rolls.get(provider_index).copied().unwrap_or(0);
                    provider_index += 1;
                    if let Some(state) =
                        block_state_provider_sample(&config.state_provider, provider_roll)
                    {
                        placed.push(DiskPlacementBlock {
                            pos,
                            state,
                            mark_above_for_post_processing: !placed_above,
                        });
                        placed_above = true;
                    }
                } else {
                    placed_above = false;
                }
            }
        }
    }
    placed
}
