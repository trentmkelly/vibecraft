use super::*;

pub fn spring_feature_can_place(
    valid_above: bool,
    requires_block_below: bool,
    valid_below: bool,
    current_is_air_or_valid: bool,
    adjacent_rock_count: i32,
    adjacent_hole_count: i32,
    required_rock_count: i32,
    required_hole_count: i32,
) -> bool {
    valid_above
        && (!requires_block_below || valid_below)
        && current_is_air_or_valid
        && adjacent_rock_count == required_rock_count
        && adjacent_hole_count == required_hole_count
}

pub fn spring_placement_plan(
    config: &SpringConfigurationModel,
    context: SpringPlacementContext,
) -> Option<SpringPlacementPlan> {
    let valid_above = config.valid_blocks.contains(&context.above_block);
    let valid_below = config.valid_blocks.contains(&context.below_block);
    let current_is_air_or_valid = context.current_block == "minecraft:air"
        || config.valid_blocks.contains(&context.current_block);
    let adjacent_blocks = [
        context.west_block,
        context.east_block,
        context.north_block,
        context.south_block,
        context.below_block,
    ];
    let adjacent_rock_count = adjacent_blocks
        .iter()
        .filter(|block| config.valid_blocks.contains(block))
        .count() as i32;
    let adjacent_hole_count = adjacent_blocks
        .iter()
        .filter(|block| **block == "minecraft:air")
        .count() as i32;
    spring_feature_can_place(
        valid_above,
        config.requires_block_below,
        valid_below,
        current_is_air_or_valid,
        adjacent_rock_count,
        adjacent_hole_count,
        config.rock_count,
        config.hole_count,
    )
    .then_some(SpringPlacementPlan {
        pos: context.origin,
        state: config.state,
        schedule_tick: true,
    })
}

pub fn monster_room_opening_count_is_valid(openings: i32) -> bool {
    (MONSTER_ROOM_BOUNDS.min_openings..=MONSTER_ROOM_BOUNDS.max_openings).contains(&openings)
}

pub fn monster_room_radii(x_roll: i32, z_roll: i32) -> MonsterRoomRadii {
    MonsterRoomRadii {
        x_radius: x_roll.rem_euclid(2) + 2,
        z_radius: z_roll.rem_euclid(2) + 2,
    }
}

pub fn monster_room_bounds_for_radius(radius: i32) -> (i32, i32) {
    (-radius - 1, radius + 1)
}

pub fn monster_room_opening_count(
    radii: MonsterRoomRadii,
    probes: &[MonsterRoomProbe],
) -> Option<i32> {
    let (min_x, max_x) = monster_room_bounds_for_radius(radii.x_radius);
    let (min_z, max_z) = monster_room_bounds_for_radius(radii.z_radius);
    let mut openings = 0;
    for probe in probes {
        if probe.dy == MONSTER_ROOM_BOUNDS.min_y && !probe.solid {
            return None;
        }
        if probe.dy == MONSTER_ROOM_BOUNDS.max_y && !probe.solid {
            return None;
        }
        if (probe.dx == min_x || probe.dx == max_x || probe.dz == min_z || probe.dz == max_z)
            && probe.dy == 0
            && probe.empty
            && probe.above_empty
        {
            openings += 1;
        }
    }
    Some(openings)
}

pub fn monster_room_can_place(radii: MonsterRoomRadii, probes: &[MonsterRoomProbe]) -> bool {
    monster_room_opening_count(radii, probes).is_some_and(monster_room_opening_count_is_valid)
}

pub fn monster_room_shell_state(
    dx: i32,
    dy: i32,
    dz: i32,
    radii: MonsterRoomRadii,
    y: i32,
    below_solid: bool,
    current_solid: bool,
    current_is_chest: bool,
    mossy_roll: i32,
) -> Option<&'static str> {
    let (min_x, max_x) = monster_room_bounds_for_radius(radii.x_radius);
    let (min_z, max_z) = monster_room_bounds_for_radius(radii.z_radius);
    let boundary = dx == min_x
        || dy == MONSTER_ROOM_BOUNDS.min_y
        || dz == min_z
        || dx == max_x
        || dy == MONSTER_ROOM_BOUNDS.max_y
        || dz == max_z;
    if boundary {
        if y >= 0 && !below_solid {
            Some("minecraft:cave_air")
        } else if current_solid && !current_is_chest {
            if dy == MONSTER_ROOM_BOUNDS.min_y && mossy_roll.rem_euclid(4) != 0 {
                Some("minecraft:mossy_cobblestone")
            } else {
                Some("minecraft:cobblestone")
            }
        } else {
            None
        }
    } else if !current_is_chest {
        Some("minecraft:cave_air")
    } else {
        None
    }
}

pub fn monster_room_chest_can_place(empty: bool, horizontal_solid_neighbors: i32) -> bool {
    empty && horizontal_solid_neighbors == 1
}

pub fn monster_room_spawner_mob(mob_roll: i32) -> &'static str {
    match mob_roll.rem_euclid(4) {
        0 => "minecraft:skeleton",
        1 | 2 => "minecraft:zombie",
        _ => "minecraft:spider",
    }
}

pub fn ore_vein_sphere_is_shadowed(radius_delta: f64, dx: f64, dy: f64, dz: f64) -> bool {
    radius_delta * radius_delta > dx * dx + dy * dy + dz * dz
}

pub fn blending_height_to_offset(height: f64) -> f64 {
    let target_y = height + 0.5;
    let target_y_mod = target_y.rem_euclid(8.0);
    (32.0 * (target_y - 128.0) - 3.0 * (target_y - 120.0) * target_y_mod
        + 3.0 * target_y_mod * target_y_mod)
        / (128.0 * (32.0 - 3.0 * target_y_mod))
}

pub fn blending_smooth_alpha(distance: f64, range_cells: i32) -> f64 {
    let alpha = (distance / f64::from(range_cells + 1)).clamp(0.0, 1.0);
    3.0 * alpha * alpha - 2.0 * alpha * alpha * alpha
}

pub fn validate_blending_data_packed(data: BlendingDataPacked<'_>) -> Result<(), String> {
    match data.heights {
        Some(heights) if heights.len() != BLENDING_CELL_COLUMN_COUNT => Err(format!(
            "heights has to be of length {BLENDING_CELL_COLUMN_COUNT}"
        )),
        _ => Ok(()),
    }
}

pub fn blending_output_for_old_height(
    height: Option<f64>,
    distance_cells: Option<f64>,
) -> BlendingOutput {
    match (height, distance_cells) {
        (Some(height), Some(distance)) => BlendingOutput {
            alpha: blending_smooth_alpha(distance, BLENDING_CONSTANTS.height_blending_range_cells),
            blending_offset: blending_height_to_offset(height),
        },
        (Some(height), None) => BlendingOutput {
            alpha: 0.0,
            blending_offset: blending_height_to_offset(height),
        },
        (None, _) => BlendingOutput {
            alpha: 1.0,
            blending_offset: 0.0,
        },
    }
}

pub fn below_zero_replace_old_bedrock_action(y: i32, state: &str) -> Option<&'static str> {
    (0..=BELOW_ZERO_RETROGEN_MODEL.max_generated_bedrock_y)
        .contains(&y)
        .then_some(state)
        .filter(|state| *state == "minecraft:bedrock")
        .map(|_| "minecraft:deepslate")
}

pub fn below_zero_missing_bedrock_bit_index(x: i32, z: i32) -> usize {
    ((z & 15) * 16 + (x & 15)) as usize
}

pub fn below_zero_has_bedrock_hole(missing_bedrock: &[u64], x: i32, z: i32) -> bool {
    let bit = below_zero_missing_bedrock_bit_index(x, z);
    missing_bedrock
        .get(bit / 64)
        .is_some_and(|word| (word & (1_u64 << (bit % 64))) != 0)
}

pub fn below_zero_bedrock_mask_air_columns(
    min_y: i32,
    max_y: i32,
    missing_bedrock: &[u64],
) -> Vec<FeaturePlacementBlock> {
    let mut blocks = Vec::new();
    for x in 0..16 {
        for z in 0..16 {
            if below_zero_has_bedrock_hole(missing_bedrock, x, z) {
                for y in min_y..=max_y {
                    blocks.push(FeaturePlacementBlock {
                        pos: BlockPos { x, y, z },
                        state: "minecraft:air",
                    });
                }
            }
        }
    }
    blocks
}

pub fn below_zero_retrogen_biome(
    upgrading: bool,
    resolved_biome: &'static str,
    existing_chunk_biome_at_y0: &'static str,
) -> &'static str {
    if upgrading
        && !BELOW_ZERO_RETROGEN_MODEL
            .retained_biomes
            .contains(&resolved_biome)
    {
        existing_chunk_biome_at_y0
    } else {
        resolved_biome
    }
}
