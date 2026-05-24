use super::*;

pub fn end_spike_from_size(index: i32, size: i32) -> EndSpikeModel {
    let angle = 2.0 * (-std::f64::consts::PI + (std::f64::consts::PI / 10.0) * index as f64);
    EndSpikeModel {
        center_x: (42.0 * angle.cos()).floor() as i32,
        center_z: (42.0 * angle.sin()).floor() as i32,
        radius: 2 + size / 3,
        height: 76 + size * 3,
        guarded: size == 1 || size == 2,
    }
}

pub fn end_spike_is_center_within_chunk(spike: EndSpikeModel, chunk_origin: BlockPos) -> bool {
    chunk_origin.x.div_euclid(16) == spike.center_x.div_euclid(16)
        && chunk_origin.z.div_euclid(16) == spike.center_z.div_euclid(16)
}

pub fn end_spike_top_bounding_box(
    spike: EndSpikeModel,
    min_y: i32,
    max_y: i32,
) -> (BlockPos, BlockPos) {
    (
        BlockPos {
            x: spike.center_x - spike.radius,
            y: min_y,
            z: spike.center_z - spike.radius,
        },
        BlockPos {
            x: spike.center_x + spike.radius,
            y: max_y,
            z: spike.center_z + spike.radius,
        },
    )
}

pub fn end_spike_cylinder_and_air_blocks(
    spike: EndSpikeModel,
    min_y: i32,
) -> Vec<EndSpikePlacementBlock> {
    let mut blocks = Vec::new();
    for y in min_y..=(spike.height + 10) {
        for z in (spike.center_z - spike.radius)..=(spike.center_z + spike.radius) {
            for x in (spike.center_x - spike.radius)..=(spike.center_x + spike.radius) {
                let dx = x - spike.center_x;
                let dz = z - spike.center_z;
                if dx * dx + dz * dz <= spike.radius * spike.radius + 1 && y < spike.height {
                    blocks.push(EndSpikePlacementBlock {
                        pos: BlockPos { x, y, z },
                        kind: EndSpikeBlockKind::Obsidian,
                    });
                } else if y > 65 {
                    blocks.push(EndSpikePlacementBlock {
                        pos: BlockPos { x, y, z },
                        kind: EndSpikeBlockKind::Air,
                    });
                }
            }
        }
    }
    blocks
}

pub fn end_spike_guard_cage_blocks(spike: EndSpikeModel) -> Vec<EndSpikePlacementBlock> {
    if !spike.guarded {
        return Vec::new();
    }
    let mut blocks = Vec::new();
    for dx in -2i32..=2 {
        for dz in -2i32..=2 {
            for dy in 0i32..=3 {
                let x_side = dx.abs() == 2;
                let z_side = dz.abs() == 2;
                let top = dy == 3;
                if !x_side && !z_side && !top {
                    continue;
                }
                let x_edge = dx == -2 || dx == 2 || top;
                let z_edge = dz == -2 || dz == 2 || top;
                blocks.push(EndSpikePlacementBlock {
                    pos: BlockPos {
                        x: spike.center_x + dx,
                        y: spike.height + dy,
                        z: spike.center_z + dz,
                    },
                    kind: EndSpikeBlockKind::IronBars {
                        north: x_edge && dz != -2,
                        south: x_edge && dz != 2,
                        west: z_edge && dx != -2,
                        east: z_edge && dx != 2,
                    },
                });
            }
        }
    }
    blocks
}

pub fn end_crystal_for_spike(
    spike: EndSpikeModel,
    config: &EndSpikeConfigurationModel,
    yaw_roll: f32,
) -> EndCrystalPlacement {
    EndCrystalPlacement {
        x: spike.center_x as f64 + 0.5,
        y: spike.height as f64 + 1.0,
        z: spike.center_z as f64 + 0.5,
        yaw: yaw_roll * 360.0,
        beam_target: config.crystal_beam_target,
        invulnerable: config.crystal_invulnerable,
    }
}

pub fn end_spike_crystal_support_blocks(spike: EndSpikeModel) -> Vec<EndSpikePlacementBlock> {
    let crystal_block_pos = BlockPos {
        x: spike.center_x,
        y: spike.height + 1,
        z: spike.center_z,
    };
    vec![
        EndSpikePlacementBlock {
            pos: BlockPos {
                x: crystal_block_pos.x,
                y: crystal_block_pos.y - 1,
                z: crystal_block_pos.z,
            },
            kind: EndSpikeBlockKind::Bedrock,
        },
        EndSpikePlacementBlock {
            pos: crystal_block_pos,
            kind: EndSpikeBlockKind::Fire,
        },
    ]
}
