use super::*;

pub fn end_city_rotation_from_roll(rotation_roll: i32) -> Result<StructureRotation, String> {
    if !(0..4).contains(&rotation_roll) {
        return Err("End city rotation roll must match RandomSource#nextInt(4)".to_string());
    }
    Ok(fossil_rotation(rotation_roll))
}

pub fn end_city_generation_start(start_pos: BlockPos) -> Option<BlockPos> {
    (start_pos.y >= 60).then_some(start_pos)
}

pub fn end_city_template_id(template_name: &'static str) -> String {
    format!("minecraft:end_city/{template_name}")
}

pub fn end_city_template_piece(
    template_name: &'static str,
    position: BlockPos,
    rotation: StructureRotation,
    overwrite: bool,
    gen_depth: i32,
) -> EndCityTemplatePieceModel {
    EndCityTemplatePieceModel {
        template_name,
        template_id: match template_name {
            "base_floor" => "minecraft:end_city/base_floor",
            "second_floor_1" => "minecraft:end_city/second_floor_1",
            "third_floor_1" => "minecraft:end_city/third_floor_1",
            "third_roof" => "minecraft:end_city/third_roof",
            "tower_base" => "minecraft:end_city/tower_base",
            "tower_piece" => "minecraft:end_city/tower_piece",
            "tower_top" => "minecraft:end_city/tower_top",
            "bridge_end" => "minecraft:end_city/bridge_end",
            "bridge_piece" => "minecraft:end_city/bridge_piece",
            "bridge_steep_stairs" => "minecraft:end_city/bridge_steep_stairs",
            "bridge_gentle_stairs" => "minecraft:end_city/bridge_gentle_stairs",
            "ship" => "minecraft:end_city/ship",
            "fat_tower_base" => "minecraft:end_city/fat_tower_base",
            "fat_tower_middle" => "minecraft:end_city/fat_tower_middle",
            "fat_tower_top" => "minecraft:end_city/fat_tower_top",
            other => {
                let _ = other;
                "minecraft:end_city/unknown"
            }
        },
        position,
        rotation,
        overwrite,
        processor: if overwrite {
            "STRUCTURE_BLOCK"
        } else {
            "STRUCTURE_AND_AIR"
        },
        gen_depth,
    }
}

pub fn end_city_start_house_tower_seed(
    origin: BlockPos,
    rotation: StructureRotation,
) -> Vec<EndCityTemplatePieceModel> {
    vec![
        end_city_template_piece("base_floor", origin, rotation, true, 0),
        end_city_template_piece(
            "second_floor_1",
            BlockPos {
                x: origin.x - 1,
                y: origin.y,
                z: origin.z - 1,
            },
            rotation,
            false,
            0,
        ),
        end_city_template_piece(
            "third_floor_1",
            BlockPos {
                x: origin.x - 2,
                y: origin.y + 4,
                z: origin.z - 2,
            },
            rotation,
            false,
            0,
        ),
        end_city_template_piece(
            "third_roof",
            BlockPos {
                x: origin.x - 3,
                y: origin.y + 12,
                z: origin.z - 3,
            },
            rotation,
            true,
            0,
        ),
    ]
}

pub fn end_city_tower_bridge_candidates() -> Vec<EndCityBridgeCandidateModel> {
    vec![
        EndCityBridgeCandidateModel {
            rotation: StructureRotation::None,
            offset: BlockPos { x: 1, y: -1, z: 0 },
        },
        EndCityBridgeCandidateModel {
            rotation: StructureRotation::Clockwise90,
            offset: BlockPos { x: 6, y: -1, z: 1 },
        },
        EndCityBridgeCandidateModel {
            rotation: StructureRotation::Counterclockwise90,
            offset: BlockPos { x: 0, y: -1, z: 5 },
        },
        EndCityBridgeCandidateModel {
            rotation: StructureRotation::Clockwise180,
            offset: BlockPos { x: 5, y: -1, z: 6 },
        },
    ]
}

pub fn end_city_fat_tower_bridge_candidates() -> Vec<EndCityBridgeCandidateModel> {
    vec![
        EndCityBridgeCandidateModel {
            rotation: StructureRotation::None,
            offset: BlockPos { x: 4, y: -1, z: 0 },
        },
        EndCityBridgeCandidateModel {
            rotation: StructureRotation::Clockwise90,
            offset: BlockPos { x: 12, y: -1, z: 4 },
        },
        EndCityBridgeCandidateModel {
            rotation: StructureRotation::Counterclockwise90,
            offset: BlockPos { x: 0, y: -1, z: 8 },
        },
        EndCityBridgeCandidateModel {
            rotation: StructureRotation::Clockwise180,
            offset: BlockPos { x: 8, y: -1, z: 12 },
        },
    ]
}

pub fn end_city_marker_action(
    marker_id: &'static str,
    position: BlockPos,
    rotation: StructureRotation,
    chunk_bb: StructureBoundingBoxModel,
    in_spawnable_bounds: bool,
) -> Option<EndCityMarkerActionModel> {
    if marker_id.starts_with("Chest") {
        let target_pos = BlockPos {
            x: position.x,
            y: position.y - 1,
            z: position.z,
        };
        return chunk_bb
            .is_inside(target_pos)
            .then_some(EndCityMarkerActionModel {
                marker_id,
                target_pos,
                loot_table: Some("minecraft:chests/end_city_treasure"),
                spawned_entity: None,
                item_frame_facing: None,
                item: None,
            });
    }
    if !chunk_bb.is_inside(position) || !in_spawnable_bounds {
        return None;
    }
    if marker_id.starts_with("Sentry") {
        Some(EndCityMarkerActionModel {
            marker_id,
            target_pos: position,
            loot_table: None,
            spawned_entity: Some("minecraft:shulker"),
            item_frame_facing: None,
            item: None,
        })
    } else if marker_id.starts_with("Elytra") {
        Some(EndCityMarkerActionModel {
            marker_id,
            target_pos: position,
            loot_table: None,
            spawned_entity: Some("minecraft:item_frame"),
            item_frame_facing: Some(end_city_rotated_south(rotation)),
            item: Some("minecraft:elytra"),
        })
    } else {
        None
    }
}

pub fn end_city_rotated_south(rotation: StructureRotation) -> HorizontalDirection {
    match rotation {
        StructureRotation::None => HorizontalDirection::South,
        StructureRotation::Clockwise90 => HorizontalDirection::West,
        StructureRotation::Clockwise180 => HorizontalDirection::North,
        StructureRotation::Counterclockwise90 => HorizontalDirection::East,
    }
}

