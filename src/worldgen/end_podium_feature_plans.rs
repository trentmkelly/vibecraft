use super::*;

pub fn end_podium_location(offset: BlockPos) -> BlockPos {
    offset
}

pub fn end_podium_inside_rim(pos: BlockPos, origin: BlockPos) -> bool {
    let dx = pos.x - origin.x;
    let dy = pos.y - origin.y;
    let dz = pos.z - origin.z;
    ((dx * dx + dy * dy + dz * dz) as f64) < 2.5_f64.powi(2)
}

pub fn end_podium_inside_body(pos: BlockPos, origin: BlockPos) -> bool {
    let dx = pos.x - origin.x;
    let dy = pos.y - origin.y;
    let dz = pos.z - origin.z;
    ((dx * dx + dy * dy + dz * dz) as f64) < 3.5_f64.powi(2)
}

pub fn end_podium_blocks(origin: BlockPos, active: bool) -> Vec<EndPodiumPlacementBlock> {
    let mut blocks = Vec::new();
    for y in (origin.y - 1)..=(origin.y + 32) {
        for z in (origin.z - 4)..=(origin.z + 4) {
            for x in (origin.x - 4)..=(origin.x + 4) {
                let pos = BlockPos { x, y, z };
                let inside_rim = end_podium_inside_rim(pos, origin);
                if !inside_rim && !end_podium_inside_body(pos, origin) {
                    continue;
                }
                let kind = if y < origin.y {
                    if inside_rim {
                        EndPodiumBlockKind::Bedrock
                    } else {
                        EndPodiumBlockKind::EndStone
                    }
                } else if y > origin.y {
                    EndPodiumBlockKind::Air
                } else if !inside_rim {
                    EndPodiumBlockKind::Bedrock
                } else if active {
                    EndPodiumBlockKind::EndPortal
                } else {
                    EndPodiumBlockKind::Air
                };
                blocks.push(EndPodiumPlacementBlock { pos, kind });
            }
        }
    }
    for y in 0..4 {
        blocks.push(EndPodiumPlacementBlock {
            pos: BlockPos {
                x: origin.x,
                y: origin.y + y,
                z: origin.z,
            },
            kind: EndPodiumBlockKind::Bedrock,
        });
    }
    let torch_y = origin.y + 2;
    for direction in [
        HorizontalDirection::North,
        HorizontalDirection::South,
        HorizontalDirection::West,
        HorizontalDirection::East,
    ] {
        let pos = offset_horizontal(
            BlockPos {
                x: origin.x,
                y: torch_y,
                z: origin.z,
            },
            direction,
            1,
        );
        blocks.push(EndPodiumPlacementBlock {
            pos,
            kind: EndPodiumBlockKind::WallTorch(direction),
        });
    }
    blocks
}
