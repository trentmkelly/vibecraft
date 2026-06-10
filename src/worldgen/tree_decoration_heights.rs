use super::*;

#[derive(Debug, Clone)]
pub(super) struct TreeDecorationHeights {
    pub(super) ocean_floor: [i32; 16 * 16],
    pub(super) world_surface: [i32; 16 * 16],
    pub(super) motion_blocking: [i32; 16 * 16],
    pub(super) motion_blocking_no_leaves: [i32; 16 * 16],
}

#[derive(Clone, Copy)]
pub(super) enum SourceTerrainHeights<'a> {
    Full(&'a TreeDecorationHeights),
    Lazy(&'a LightweightTreeContextChunk),
}

impl SourceTerrainHeights<'_> {
    pub(super) fn local_height(
        self,
        heightmap: HeightmapKind,
        local_x: usize,
        local_z: usize,
    ) -> i32 {
        match self {
            SourceTerrainHeights::Full(heights) => {
                heights.local_height(heightmap, local_x, local_z)
            }
            SourceTerrainHeights::Lazy(chunk) => chunk
                .terrain_heights
                .local_height(heightmap, local_x, local_z),
        }
    }

    pub(super) fn world_height(
        self,
        source_pos: ChunkPos,
        heightmap: HeightmapKind,
        world_x: i32,
        world_z: i32,
        fallback_y: i32,
    ) -> i32 {
        if world_x.div_euclid(16) != source_pos.x || world_z.div_euclid(16) != source_pos.z {
            return fallback_y;
        }
        self.local_height(
            heightmap,
            world_x.rem_euclid(16) as usize,
            world_z.rem_euclid(16) as usize,
        )
    }
}

impl TreeDecorationHeights {
    pub(super) fn local_height(
        &self,
        heightmap: HeightmapKind,
        local_x: usize,
        local_z: usize,
    ) -> i32 {
        let index = local_z * 16 + local_x;
        match heightmap {
            HeightmapKind::WorldSurface | HeightmapKind::WorldSurfaceWg => {
                self.world_surface[index]
            }
            HeightmapKind::OceanFloor | HeightmapKind::OceanFloorWg => self.ocean_floor[index],
            HeightmapKind::MotionBlocking => self.motion_blocking[index],
            HeightmapKind::MotionBlockingNoLeaves => self.motion_blocking_no_leaves[index],
        }
    }
}

pub(super) fn tree_decoration_terrain_heights_from_wg(
    chunk: &LevelChunk,
    settings: &NoiseGeneratorSettings,
) -> TreeDecorationHeights {
    let mut ocean_floor = [settings.sea_level + 1; 16 * 16];
    let mut world_surface = [settings.sea_level + 1; 16 * 16];
    let mut motion_blocking = [settings.sea_level + 1; 16 * 16];
    let mut motion_blocking_no_leaves = [settings.sea_level + 1; 16 * 16];
    for z in 0..16 {
        for x in 0..16 {
            let index = z * 16 + x;
            ocean_floor[index] = chunk
                .heightmap_value(HeightmapKind::OceanFloorWg, x, z)
                .or_else(|| chunk.heightmap_value(HeightmapKind::OceanFloor, x, z))
                .or_else(|| chunk.heightmap_value(HeightmapKind::WorldSurfaceWg, x, z))
                .unwrap_or(settings.sea_level + 1);
            world_surface[index] = chunk
                .heightmap_value(HeightmapKind::WorldSurfaceWg, x, z)
                .or_else(|| chunk.heightmap_value(HeightmapKind::WorldSurface, x, z))
                .unwrap_or(ocean_floor[index]);
            motion_blocking[index] = chunk
                .heightmap_value(HeightmapKind::MotionBlocking, x, z)
                .unwrap_or(world_surface[index]);
            motion_blocking_no_leaves[index] = chunk
                .heightmap_value(HeightmapKind::MotionBlockingNoLeaves, x, z)
                .unwrap_or(motion_blocking[index]);
        }
    }
    TreeDecorationHeights {
        ocean_floor,
        world_surface,
        motion_blocking,
        motion_blocking_no_leaves,
    }
}

pub(super) fn tree_decoration_terrain_heights(
    chunk: &LevelChunk,
    settings: &NoiseGeneratorSettings,
) -> TreeDecorationHeights {
    let computed_ocean_floor = chunk.compute_heightmap_values(HeightmapKind::OceanFloor);
    let computed_world_surface = chunk.compute_heightmap_values(HeightmapKind::WorldSurface);
    let computed_motion_blocking = chunk.compute_heightmap_values(HeightmapKind::MotionBlocking);
    let computed_motion_blocking_no_leaves =
        chunk.compute_heightmap_values(HeightmapKind::MotionBlockingNoLeaves);
    let mut ocean_floor = computed_ocean_floor;
    let mut world_surface = computed_world_surface;
    let mut motion_blocking = computed_motion_blocking;
    let mut motion_blocking_no_leaves = computed_motion_blocking_no_leaves;
    for z in 0..16 {
        for x in 0..16 {
            let index = z * 16 + x;
            if ocean_floor[index] == 0 {
                ocean_floor[index] = chunk
                    .heightmap_value(HeightmapKind::OceanFloorWg, x, z)
                    .or_else(|| chunk.heightmap_value(HeightmapKind::WorldSurfaceWg, x, z))
                    .unwrap_or(settings.sea_level + 1);
            }
            if world_surface[index] == 0 {
                world_surface[index] = chunk
                    .heightmap_value(HeightmapKind::WorldSurfaceWg, x, z)
                    .unwrap_or(ocean_floor[index]);
            }
            if motion_blocking[index] == 0 {
                motion_blocking[index] = world_surface[index];
            }
            if motion_blocking_no_leaves[index] == 0 {
                motion_blocking_no_leaves[index] = motion_blocking[index];
            }
        }
    }
    TreeDecorationHeights {
        ocean_floor,
        world_surface,
        motion_blocking,
        motion_blocking_no_leaves,
    }
}
