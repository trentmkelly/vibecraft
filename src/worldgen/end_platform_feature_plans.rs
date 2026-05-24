use super::*;

pub fn end_platform_blocks(origin: BlockPos) -> Vec<FeaturePlacementBlock> {
    let mut blocks = Vec::new();
    for dz in -2..=2 {
        for dx in -2..=2 {
            for dy in -1..3 {
                blocks.push(FeaturePlacementBlock {
                    pos: BlockPos {
                        x: origin.x + dx,
                        y: origin.y + dy,
                        z: origin.z + dz,
                    },
                    state: if dy == -1 {
                        "minecraft:obsidian"
                    } else {
                        "minecraft:air"
                    },
                });
            }
        }
    }
    blocks
}

pub fn void_start_platform_origin(feature_origin_y: i32) -> BlockPos {
    BlockPos {
        x: 8,
        y: feature_origin_y + 3,
        z: 8,
    }
}

pub fn checkerboard_distance(xa: i32, za: i32, xb: i32, zb: i32) -> i32 {
    (xa - xb).abs().max((za - zb).abs())
}

pub fn chunk_pos_containing_block(x: i32, z: i32) -> ChunkPos {
    ChunkPos {
        x: x.div_euclid(16),
        z: z.div_euclid(16),
    }
}

pub fn void_start_platform_applies_to_chunk(chunk_pos: ChunkPos) -> bool {
    let platform_chunk = chunk_pos_containing_block(8, 8);
    checkerboard_distance(chunk_pos.x, chunk_pos.z, platform_chunk.x, platform_chunk.z) <= 1
}

pub fn void_start_platform_blocks(
    chunk_pos: ChunkPos,
    feature_origin_y: i32,
) -> Vec<FeaturePlacementBlock> {
    if !void_start_platform_applies_to_chunk(chunk_pos) {
        return Vec::new();
    }
    let origin = void_start_platform_origin(feature_origin_y);
    let mut blocks = Vec::new();
    for z in (chunk_pos.z * 16)..=(chunk_pos.z * 16 + 15) {
        for x in (chunk_pos.x * 16)..=(chunk_pos.x * 16 + 15) {
            if checkerboard_distance(origin.x, origin.z, x, z) <= 16 {
                blocks.push(FeaturePlacementBlock {
                    pos: BlockPos { x, y: origin.y, z },
                    state: if x == origin.x && z == origin.z {
                        "minecraft:cobblestone"
                    } else {
                        "minecraft:stone"
                    },
                });
            }
        }
    }
    blocks
}
