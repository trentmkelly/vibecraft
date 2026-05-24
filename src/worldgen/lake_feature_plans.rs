use super::*;

pub fn lake_grid_index(x: i32, y: i32, z: i32) -> usize {
    ((x * 16 + z) * 8 + y) as usize
}

pub fn lake_is_boundary(grid: &[bool], x: i32, y: i32, z: i32) -> bool {
    if *grid.get(lake_grid_index(x, y, z)).unwrap_or(&false) {
        return false;
    }
    (x < 15 && *grid.get(lake_grid_index(x + 1, y, z)).unwrap_or(&false))
        || (x > 0 && *grid.get(lake_grid_index(x - 1, y, z)).unwrap_or(&false))
        || (z < 15 && *grid.get(lake_grid_index(x, y, z + 1)).unwrap_or(&false))
        || (z > 0 && *grid.get(lake_grid_index(x, y, z - 1)).unwrap_or(&false))
        || (y < 7 && *grid.get(lake_grid_index(x, y + 1, z)).unwrap_or(&false))
        || (y > 0 && *grid.get(lake_grid_index(x, y - 1, z)).unwrap_or(&false))
}

pub fn lake_can_place(
    min_y: i32,
    origin_y: i32,
    grid: &[bool],
    boundary: &[LakeBoundaryBlock],
    fluid: &'static str,
) -> bool {
    if origin_y <= min_y + 4 {
        return false;
    }
    boundary.iter().all(|block| {
        !lake_is_boundary(grid, block.x, block.y, block.z)
            || if block.y >= 4 {
                !block.liquid
            } else {
                block.solid || block.state == fluid
            }
    })
}

pub fn lake_placement_plan(
    origin: BlockPos,
    config: &LakeConfigurationModel,
    grid: &[bool],
    boundary: &[LakeBoundaryBlock],
    barrier_rolls: &[i32],
    freeze_water: bool,
) -> Option<Vec<LakePlacementBlock>> {
    let fluid = block_state_provider_sample(&config.fluid, 0)?;
    let barrier = block_state_provider_sample(&config.barrier, 0)?;
    let mut blocks = Vec::new();
    for x in 0..16 {
        for z in 0..16 {
            for y in 0..8 {
                if *grid.get(lake_grid_index(x, y, z)).unwrap_or(&false) {
                    let place_air = y >= 4;
                    blocks.push(LakePlacementBlock {
                        pos: BlockPos {
                            x: origin.x + x,
                            y: origin.y + y,
                            z: origin.z + z,
                        },
                        state: if place_air {
                            "minecraft:cave_air"
                        } else {
                            fluid
                        },
                        schedule_tick: place_air,
                        mark_above_for_post_processing: place_air,
                    });
                }
            }
        }
    }
    if barrier != "minecraft:air" {
        for block in boundary {
            if lake_is_boundary(grid, block.x, block.y, block.z)
                && (block.y < 4
                    || barrier_rolls
                        .get(lake_grid_index(block.x, block.y, block.z))
                        .copied()
                        .unwrap_or(1)
                        .rem_euclid(2)
                        != 0)
                && block.solid
                && !block.cannot_replace
            {
                blocks.push(LakePlacementBlock {
                    pos: BlockPos {
                        x: origin.x + block.x,
                        y: origin.y + block.y,
                        z: origin.z + block.z,
                    },
                    state: barrier,
                    schedule_tick: false,
                    mark_above_for_post_processing: true,
                });
            }
        }
    }
    if freeze_water && fluid == "minecraft:water" {
        for block in boundary
            .iter()
            .filter(|block| block.y == 4 && block.should_freeze)
        {
            blocks.push(LakePlacementBlock {
                pos: BlockPos {
                    x: origin.x + block.x,
                    y: origin.y + 4,
                    z: origin.z + block.z,
                },
                state: "minecraft:ice",
                schedule_tick: false,
                mark_above_for_post_processing: false,
            });
        }
    }
    Some(blocks)
}
