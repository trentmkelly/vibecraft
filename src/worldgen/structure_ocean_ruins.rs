use super::*;

pub const OCEAN_RUIN_WARM_TEMPLATES: [&str; 8] = [
    "minecraft:underwater_ruin/warm_1",
    "minecraft:underwater_ruin/warm_2",
    "minecraft:underwater_ruin/warm_3",
    "minecraft:underwater_ruin/warm_4",
    "minecraft:underwater_ruin/warm_5",
    "minecraft:underwater_ruin/warm_6",
    "minecraft:underwater_ruin/warm_7",
    "minecraft:underwater_ruin/warm_8",
];

pub const OCEAN_RUIN_BIG_WARM_TEMPLATES: [&str; 4] = [
    "minecraft:underwater_ruin/big_warm_4",
    "minecraft:underwater_ruin/big_warm_5",
    "minecraft:underwater_ruin/big_warm_6",
    "minecraft:underwater_ruin/big_warm_7",
];

pub const OCEAN_RUIN_BRICK_TEMPLATES: [&str; 8] = [
    "minecraft:underwater_ruin/brick_1",
    "minecraft:underwater_ruin/brick_2",
    "minecraft:underwater_ruin/brick_3",
    "minecraft:underwater_ruin/brick_4",
    "minecraft:underwater_ruin/brick_5",
    "minecraft:underwater_ruin/brick_6",
    "minecraft:underwater_ruin/brick_7",
    "minecraft:underwater_ruin/brick_8",
];

pub const OCEAN_RUIN_CRACKED_TEMPLATES: [&str; 8] = [
    "minecraft:underwater_ruin/cracked_1",
    "minecraft:underwater_ruin/cracked_2",
    "minecraft:underwater_ruin/cracked_3",
    "minecraft:underwater_ruin/cracked_4",
    "minecraft:underwater_ruin/cracked_5",
    "minecraft:underwater_ruin/cracked_6",
    "minecraft:underwater_ruin/cracked_7",
    "minecraft:underwater_ruin/cracked_8",
];

pub const OCEAN_RUIN_MOSSY_TEMPLATES: [&str; 8] = [
    "minecraft:underwater_ruin/mossy_1",
    "minecraft:underwater_ruin/mossy_2",
    "minecraft:underwater_ruin/mossy_3",
    "minecraft:underwater_ruin/mossy_4",
    "minecraft:underwater_ruin/mossy_5",
    "minecraft:underwater_ruin/mossy_6",
    "minecraft:underwater_ruin/mossy_7",
    "minecraft:underwater_ruin/mossy_8",
];

pub const OCEAN_RUIN_BIG_BRICK_TEMPLATES: [&str; 4] = [
    "minecraft:underwater_ruin/big_brick_1",
    "minecraft:underwater_ruin/big_brick_2",
    "minecraft:underwater_ruin/big_brick_3",
    "minecraft:underwater_ruin/big_brick_8",
];

pub const OCEAN_RUIN_BIG_CRACKED_TEMPLATES: [&str; 4] = [
    "minecraft:underwater_ruin/big_cracked_1",
    "minecraft:underwater_ruin/big_cracked_2",
    "minecraft:underwater_ruin/big_cracked_3",
    "minecraft:underwater_ruin/big_cracked_8",
];

pub const OCEAN_RUIN_BIG_MOSSY_TEMPLATES: [&str; 4] = [
    "minecraft:underwater_ruin/big_mossy_1",
    "minecraft:underwater_ruin/big_mossy_2",
    "minecraft:underwater_ruin/big_mossy_3",
    "minecraft:underwater_ruin/big_mossy_8",
];

pub fn ocean_ruin_biome_type_id(biome_type: OceanRuinBiomeType) -> &'static str {
    match biome_type {
        OceanRuinBiomeType::Warm => "warm",
        OceanRuinBiomeType::Cold => "cold",
    }
}

pub fn ocean_ruin_is_large(large_probability: f32, roll: f32) -> Result<bool, String> {
    if !(0.0..=1.0).contains(&large_probability) || !(0.0..1.0).contains(&roll) {
        return Err(
            "Ocean ruin large probability must be [0.0, 1.0] and roll [0.0, 1.0)".to_string(),
        );
    }
    Ok(roll <= large_probability)
}

pub fn ocean_ruin_should_add_cluster(cluster_probability: f32, roll: f32) -> Result<bool, String> {
    if !(0.0..=1.0).contains(&cluster_probability) || !(0.0..1.0).contains(&roll) {
        return Err(
            "Ocean ruin cluster probability must be [0.0, 1.0] and roll [0.0, 1.0)".to_string(),
        );
    }
    Ok(roll <= cluster_probability)
}

pub fn ocean_ruin_make_piece(
    template_name: &'static str,
    position: BlockPos,
    rotation: StructureRotation,
    integrity: f32,
    biome_type: OceanRuinBiomeType,
    is_large: bool,
) -> OceanRuinPieceModel {
    let (suspicious_block, suspicious_loot_table) = match biome_type {
        OceanRuinBiomeType::Warm => (
            "minecraft:suspicious_sand",
            "minecraft:archaeology/ocean_ruin_warm",
        ),
        OceanRuinBiomeType::Cold => (
            "minecraft:suspicious_gravel",
            "minecraft:archaeology/ocean_ruin_cold",
        ),
    };
    OceanRuinPieceModel {
        template_name,
        template_position: position,
        rotation,
        integrity,
        biome_type,
        is_large,
        block_rot_processor_integrity: integrity,
        suspicious_block,
        suspicious_loot_table,
    }
}

pub fn ocean_ruin_add_piece(
    config: OceanRuinStructureConfigModel,
    position: BlockPos,
    rotation: StructureRotation,
    is_large: bool,
    base_integrity: f32,
    template_index: usize,
) -> Result<Vec<OceanRuinPieceModel>, String> {
    match config.biome_type {
        OceanRuinBiomeType::Warm => {
            let templates = if is_large {
                &OCEAN_RUIN_BIG_WARM_TEMPLATES[..]
            } else {
                &OCEAN_RUIN_WARM_TEMPLATES[..]
            };
            let template = templates
                .get(template_index)
                .copied()
                .ok_or_else(|| "Ocean ruin warm template index is out of range".to_string())?;
            Ok(vec![ocean_ruin_make_piece(
                template,
                position,
                rotation,
                base_integrity,
                config.biome_type,
                is_large,
            )])
        }
        OceanRuinBiomeType::Cold => {
            let (brick, cracked, mossy) = if is_large {
                (
                    &OCEAN_RUIN_BIG_BRICK_TEMPLATES[..],
                    &OCEAN_RUIN_BIG_CRACKED_TEMPLATES[..],
                    &OCEAN_RUIN_BIG_MOSSY_TEMPLATES[..],
                )
            } else {
                (
                    &OCEAN_RUIN_BRICK_TEMPLATES[..],
                    &OCEAN_RUIN_CRACKED_TEMPLATES[..],
                    &OCEAN_RUIN_MOSSY_TEMPLATES[..],
                )
            };
            let brick = brick
                .get(template_index)
                .copied()
                .ok_or_else(|| "Ocean ruin cold template index is out of range".to_string())?;
            let cracked = cracked[template_index];
            let mossy = mossy[template_index];
            Ok(vec![
                ocean_ruin_make_piece(
                    brick,
                    position,
                    rotation,
                    base_integrity,
                    config.biome_type,
                    is_large,
                ),
                ocean_ruin_make_piece(
                    cracked,
                    position,
                    rotation,
                    0.7,
                    config.biome_type,
                    is_large,
                ),
                ocean_ruin_make_piece(mossy, position, rotation, 0.5, config.biome_type, is_large),
            ])
        }
    }
}

pub fn ocean_ruin_save_tag(piece: OceanRuinPieceModel) -> OceanRuinSaveTagModel {
    OceanRuinSaveTagModel {
        rotation: piece.rotation,
        integrity: piece.integrity,
        biome_type: piece.biome_type,
        is_large: piece.is_large,
    }
}

pub fn ocean_ruin_marker_action(
    piece: OceanRuinPieceModel,
    marker_id: &'static str,
    position: BlockPos,
    sea_level: i32,
    is_water_at_marker: bool,
) -> Option<OceanRuinMarkerActionModel> {
    match marker_id {
        "chest" => Some(OceanRuinMarkerActionModel {
            marker_id,
            pos: position,
            placed_block: if is_water_at_marker {
                "minecraft:chest[waterlogged=true]"
            } else {
                "minecraft:chest[waterlogged=false]"
            },
            loot_table: Some(if piece.is_large {
                "minecraft:chests/underwater_ruin_big"
            } else {
                "minecraft:chests/underwater_ruin_small"
            }),
            spawned_entity: None,
        }),
        "drowned" => Some(OceanRuinMarkerActionModel {
            marker_id,
            pos: position,
            placed_block: if position.y > sea_level {
                "minecraft:air"
            } else {
                "minecraft:water"
            },
            loot_table: None,
            spawned_entity: Some("minecraft:drowned"),
        }),
        _ => None,
    }
}

pub fn ocean_ruin_adjust_to_ocean_floor(
    mut piece: OceanRuinPieceModel,
    ocean_floor_height: i32,
    template_size: BlockPos,
    mut floor_y_at: impl FnMut(i32, i32) -> i32,
) -> OceanRuinPieceModel {
    piece.template_position.y = ocean_floor_height;
    let corner_rel = structure_template_relative_position(
        BlockPos {
            x: template_size.x - 1,
            y: 0,
            z: template_size.z - 1,
        },
        piece.rotation,
        BlockPos { x: 0, y: 0, z: 0 },
    );
    let corner = BlockPos {
        x: piece.template_position.x + corner_rel.x,
        y: piece.template_position.y,
        z: piece.template_position.z + corner_rel.z,
    };
    let min_x = piece.template_position.x.min(corner.x);
    let max_x = piece.template_position.x.max(corner.x);
    let min_z = piece.template_position.z.min(corner.z);
    let max_z = piece.template_position.z.max(corner.z);
    let top_y = ocean_floor_height - 1;
    let mut min_floor_y = 512;
    let mut low_area = 0;
    for x in min_x..=max_x {
        for z in min_z..=max_z {
            let floor_y = floor_y_at(x, z);
            min_floor_y = min_floor_y.min(floor_y);
            if floor_y < top_y - 2 {
                low_area += 1;
            }
        }
    }
    let width = (piece.template_position.x - corner.x).abs();
    if top_y - min_floor_y > 2 && low_area > width - 2 {
        piece.template_position.y = min_floor_y + 1;
    }
    piece
}

