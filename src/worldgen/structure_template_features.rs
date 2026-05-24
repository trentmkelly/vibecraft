use super::*;

pub fn igloo_template_name(kind: IglooTemplateKind) -> &'static str {
    match kind {
        IglooTemplateKind::Top => "minecraft:igloo/top",
        IglooTemplateKind::Middle => "minecraft:igloo/middle",
        IglooTemplateKind::Bottom => "minecraft:igloo/bottom",
    }
}

pub fn igloo_template_pivot(kind: IglooTemplateKind) -> BlockPos {
    match kind {
        IglooTemplateKind::Top => BlockPos { x: 3, y: 5, z: 5 },
        IglooTemplateKind::Middle => BlockPos { x: 1, y: 3, z: 1 },
        IglooTemplateKind::Bottom => BlockPos { x: 3, y: 6, z: 7 },
    }
}

pub fn igloo_template_offset(kind: IglooTemplateKind) -> BlockPos {
    match kind {
        IglooTemplateKind::Top => BlockPos { x: 0, y: 0, z: 0 },
        IglooTemplateKind::Middle => BlockPos { x: 2, y: -3, z: 4 },
        IglooTemplateKind::Bottom => BlockPos { x: 0, y: -3, z: -2 },
    }
}

pub fn structure_template_relative_position(
    pos: BlockPos,
    rotation: StructureRotation,
    pivot: BlockPos,
) -> BlockPos {
    match rotation {
        StructureRotation::Counterclockwise90 => BlockPos {
            x: pivot.x - pivot.z + pos.z,
            y: pos.y,
            z: pivot.x + pivot.z - pos.x,
        },
        StructureRotation::Clockwise90 => BlockPos {
            x: pivot.x + pivot.z - pos.z,
            y: pos.y,
            z: pivot.z - pivot.x + pos.x,
        },
        StructureRotation::Clockwise180 => BlockPos {
            x: pivot.x + pivot.x - pos.x,
            y: pos.y,
            z: pivot.z + pivot.z - pos.z,
        },
        StructureRotation::None => pos,
    }
}

pub fn igloo_make_piece(
    kind: IglooTemplateKind,
    position: BlockPos,
    rotation: StructureRotation,
    depth: i32,
) -> IglooPieceModel {
    let offset = igloo_template_offset(kind);
    IglooPieceModel {
        template: kind,
        template_name: igloo_template_name(kind),
        template_position: BlockPos {
            x: position.x + offset.x,
            y: position.y + offset.y - depth,
            z: position.z + offset.z,
        },
        rotation,
        pivot: igloo_template_pivot(kind),
        offset,
        depth,
    }
}

pub fn igloo_generation_pieces(
    chunk_pos: ChunkPos,
    rotation: StructureRotation,
    basement_roll: f64,
    depth_roll: i32,
) -> Result<Vec<IglooPieceModel>, String> {
    if !(0.0..1.0).contains(&basement_roll) {
        return Err("Igloo basement roll must be in [0.0, 1.0)".to_string());
    }
    if !(0..8).contains(&depth_roll) {
        return Err("Igloo depth roll must match RandomSource#nextInt(8)".to_string());
    }
    let start_pos = BlockPos {
        x: chunk_pos.x * 16,
        y: 90,
        z: chunk_pos.z * 16,
    };
    let mut pieces = Vec::new();
    if basement_roll < 0.5 {
        let depth = depth_roll + 4;
        pieces.push(igloo_make_piece(
            IglooTemplateKind::Bottom,
            start_pos,
            rotation,
            depth * 3,
        ));
        for i in 0..(depth - 1) {
            pieces.push(igloo_make_piece(
                IglooTemplateKind::Middle,
                start_pos,
                rotation,
                i * 3,
            ));
        }
    }
    pieces.push(igloo_make_piece(
        IglooTemplateKind::Top,
        start_pos,
        rotation,
        0,
    ));
    Ok(pieces)
}

pub fn igloo_post_process(
    piece: IglooPieceModel,
    mut surface_height_at: impl FnMut(i32, i32) -> i32,
    mut block_below_trapdoor_at: impl FnMut(BlockPos) -> &'static str,
) -> IglooPostProcessModel {
    let entrance_local = BlockPos {
        x: 3 - piece.offset.x,
        y: 0,
        z: -piece.offset.z,
    };
    let relative_entrance =
        structure_template_relative_position(entrance_local, piece.rotation, piece.pivot);
    let entrance_pos = BlockPos {
        x: piece.template_position.x + relative_entrance.x,
        y: piece.template_position.y + relative_entrance.y,
        z: piece.template_position.z + relative_entrance.z,
    };
    let height = surface_height_at(entrance_pos.x, entrance_pos.z);
    let adjusted_template_position = BlockPos {
        x: piece.template_position.x,
        y: piece.template_position.y + height - 90 - 1,
        z: piece.template_position.z,
    };
    let trapdoor_pos = (piece.template == IglooTemplateKind::Top).then(|| {
        let relative_trapdoor = structure_template_relative_position(
            BlockPos { x: 3, y: 0, z: 5 },
            piece.rotation,
            piece.pivot,
        );
        BlockPos {
            x: adjusted_template_position.x + relative_trapdoor.x,
            y: adjusted_template_position.y + relative_trapdoor.y,
            z: adjusted_template_position.z + relative_trapdoor.z,
        }
    });
    let should_cover_trapdoor = trapdoor_pos
        .map(|pos| {
            let below = block_below_trapdoor_at(BlockPos {
                x: pos.x,
                y: pos.y - 1,
                z: pos.z,
            });
            below != "minecraft:air" && below != "minecraft:ladder"
        })
        .unwrap_or(false);
    IglooPostProcessModel {
        piece,
        entrance_pos,
        adjusted_template_position,
        trapdoor_pos,
        should_cover_trapdoor,
    }
}

pub const NETHER_FOSSIL_TEMPLATES: [&str; 14] = [
    "minecraft:nether_fossils/fossil_1",
    "minecraft:nether_fossils/fossil_2",
    "minecraft:nether_fossils/fossil_3",
    "minecraft:nether_fossils/fossil_4",
    "minecraft:nether_fossils/fossil_5",
    "minecraft:nether_fossils/fossil_6",
    "minecraft:nether_fossils/fossil_7",
    "minecraft:nether_fossils/fossil_8",
    "minecraft:nether_fossils/fossil_9",
    "minecraft:nether_fossils/fossil_10",
    "minecraft:nether_fossils/fossil_11",
    "minecraft:nether_fossils/fossil_12",
    "minecraft:nether_fossils/fossil_13",
    "minecraft:nether_fossils/fossil_14",
];

pub fn nether_fossil_make_piece(
    position: BlockPos,
    template_index: usize,
    rotation: StructureRotation,
) -> Result<NetherFossilPieceModel, String> {
    let Some(template_name) = NETHER_FOSSIL_TEMPLATES.get(template_index).copied() else {
        return Err("Nether fossil template index must match Util.getRandom(FOSSILS)".to_string());
    };
    Ok(NetherFossilPieceModel {
        template_index,
        template_name,
        template_position: position,
        rotation,
        processor: "minecraft:block_ignore_structure_and_air",
    })
}

pub fn nether_fossil_find_generation_point(
    chunk_pos: ChunkPos,
    block_x_roll: i32,
    block_z_roll: i32,
    sampled_y: i32,
    sea_level: i32,
    template_index: usize,
    rotation: StructureRotation,
    mut block_at: impl FnMut(i32) -> &'static str,
    mut is_sturdy_support: impl FnMut(i32) -> bool,
) -> Result<Option<NetherFossilGenerationPointModel>, String> {
    if !(0..16).contains(&block_x_roll) || !(0..16).contains(&block_z_roll) {
        return Err(
            "Nether fossil horizontal rolls must match RandomSource#nextInt(16)".to_string(),
        );
    }
    let block_x = chunk_pos.x * 16 + block_x_roll;
    let block_z = chunk_pos.z * 16 + block_z_roll;
    let mut y = sampled_y;
    while y > sea_level {
        let current = block_at(y);
        y -= 1;
        let below = block_at(y);
        if current == "minecraft:air" && (below == "minecraft:soul_sand" || is_sturdy_support(y)) {
            break;
        }
    }
    if y <= sea_level {
        return Ok(None);
    }
    let position = BlockPos {
        x: block_x,
        y,
        z: block_z,
    };
    Ok(Some(NetherFossilGenerationPointModel {
        position,
        piece: nether_fossil_make_piece(position, template_index, rotation)?,
    }))
}

pub fn nether_fossil_dried_ghast_placement(
    fossil_bb: StructureBoundingBoxModel,
    chunk_bb: StructureBoundingBoxModel,
    placement_roll: f32,
    x_roll: i32,
    z_roll: i32,
    rotation: StructureRotation,
    block_at_random_pos: &'static str,
) -> Result<Option<DriedGhastPlacementModel>, String> {
    if !(0.0..1.0).contains(&placement_roll) {
        return Err("Dried ghast placement roll must be in [0.0, 1.0)".to_string());
    }
    let x_span = fossil_bb.max_x - fossil_bb.min_x + 1;
    let z_span = fossil_bb.max_z - fossil_bb.min_z + 1;
    if !(0..x_span).contains(&x_roll) || !(0..z_span).contains(&z_roll) {
        return Err(
            "Dried ghast coordinate rolls must be inside the fossil bounding-box spans".to_string(),
        );
    }
    if placement_roll >= 0.5 {
        return Ok(None);
    }
    let pos = BlockPos {
        x: fossil_bb.min_x + x_roll,
        y: fossil_bb.min_y,
        z: fossil_bb.min_z + z_roll,
    };
    if block_at_random_pos == "minecraft:air" && chunk_bb.is_inside(pos) {
        Ok(Some(DriedGhastPlacementModel { pos, rotation }))
    } else {
        Ok(None)
    }
}

pub const RUINED_PORTAL_TEMPLATES: [&str; 10] = [
    "minecraft:ruined_portal/portal_1",
    "minecraft:ruined_portal/portal_2",
    "minecraft:ruined_portal/portal_3",
    "minecraft:ruined_portal/portal_4",
    "minecraft:ruined_portal/portal_5",
    "minecraft:ruined_portal/portal_6",
    "minecraft:ruined_portal/portal_7",
    "minecraft:ruined_portal/portal_8",
    "minecraft:ruined_portal/portal_9",
    "minecraft:ruined_portal/portal_10",
];

pub const RUINED_PORTAL_GIANT_TEMPLATES: [&str; 3] = [
    "minecraft:ruined_portal/giant_portal_1",
    "minecraft:ruined_portal/giant_portal_2",
    "minecraft:ruined_portal/giant_portal_3",
];

pub fn ruined_portal_vertical_placement_id(
    placement: RuinedPortalVerticalPlacement,
) -> &'static str {
    match placement {
        RuinedPortalVerticalPlacement::OnLandSurface => "on_land_surface",
        RuinedPortalVerticalPlacement::PartlyBuried => "partly_buried",
        RuinedPortalVerticalPlacement::OnOceanFloor => "on_ocean_floor",
        RuinedPortalVerticalPlacement::InMountain => "in_mountain",
        RuinedPortalVerticalPlacement::Underground => "underground",
        RuinedPortalVerticalPlacement::InNether => "in_nether",
    }
}

pub fn ruined_portal_choose_setup(
    setups: &[RuinedPortalSetupModel],
    pick_roll: f32,
) -> Result<RuinedPortalSetupModel, String> {
    if setups.is_empty() {
        return Err("Ruined portal setup list must be non-empty".to_string());
    }
    if !(0.0..1.0).contains(&pick_roll) {
        return Err("Ruined portal setup pick roll must be in [0.0, 1.0)".to_string());
    }
    if setups.len() == 1 {
        return Ok(setups[0]);
    }
    let total = setups.iter().map(|setup| setup.weight).sum::<f32>();
    if total <= 0.0 {
        return Err("Ruined portal setup weights must sum to a positive value".to_string());
    }
    let mut pick = pick_roll;
    for setup in setups {
        if setup.weight <= 0.0 {
            return Err("Ruined portal setup weights must be positive".to_string());
        }
        pick -= setup.weight / total;
        if pick < 0.0 {
            return Ok(*setup);
        }
    }
    Err("Ruined portal setup pick did not resolve".to_string())
}

pub fn ruined_portal_sample_probability(limit: f32, roll: f32) -> Result<bool, String> {
    if !(0.0..=1.0).contains(&limit) {
        return Err("Ruined portal probability limit must be in [0.0, 1.0]".to_string());
    }
    if !(0.0..1.0).contains(&roll) {
        return Err("Ruined portal probability roll must be in [0.0, 1.0)".to_string());
    }
    Ok(if limit == 0.0 {
        false
    } else if limit == 1.0 {
        true
    } else {
        roll < limit
    })
}

pub fn ruined_portal_template_name(
    giant_roll: f32,
    template_index: usize,
) -> Result<&'static str, String> {
    if !(0.0..1.0).contains(&giant_roll) {
        return Err("Ruined portal giant-template roll must be in [0.0, 1.0)".to_string());
    }
    if giant_roll < 0.05 {
        RUINED_PORTAL_GIANT_TEMPLATES
            .get(template_index)
            .copied()
            .ok_or_else(|| "Ruined portal giant template index is out of range".to_string())
    } else {
        RUINED_PORTAL_TEMPLATES
            .get(template_index)
            .copied()
            .ok_or_else(|| "Ruined portal template index is out of range".to_string())
    }
}

pub fn ruined_portal_mirror(mirror_roll: f32) -> Result<RuinedPortalMirrorModel, String> {
    if !(0.0..1.0).contains(&mirror_roll) {
        return Err("Ruined portal mirror roll must be in [0.0, 1.0)".to_string());
    }
    Ok(if mirror_roll < 0.5 {
        RuinedPortalMirrorModel::None
    } else {
        RuinedPortalMirrorModel::FrontBack
    })
}

pub fn ruined_portal_get_random_within_interval(
    min_preferred: i32,
    max: i32,
    inclusive_roll: i32,
) -> Result<i32, String> {
    if min_preferred < max {
        let span = max - min_preferred + 1;
        if !(0..span).contains(&inclusive_roll) {
            return Err("Ruined portal interval roll is outside the inclusive range".to_string());
        }
        Ok(min_preferred + inclusive_roll)
    } else {
        Ok(max)
    }
}

pub fn ruined_portal_initial_y(
    placement: RuinedPortalVerticalPlacement,
    air_pocket: bool,
    surface_y_at_center: i32,
    y_span: i32,
    min_y: i32,
    branch_roll: f32,
    inclusive_roll: i32,
) -> Result<i32, String> {
    if !(0.0..1.0).contains(&branch_roll) {
        return Err("Ruined portal vertical branch roll must be in [0.0, 1.0)".to_string());
    }
    match placement {
        RuinedPortalVerticalPlacement::InNether => {
            if air_pocket {
                ruined_portal_get_random_within_interval(32, 100, inclusive_roll)
            } else if branch_roll < 0.5 {
                ruined_portal_get_random_within_interval(27, 29, inclusive_roll)
            } else {
                ruined_portal_get_random_within_interval(29, 100, inclusive_roll)
            }
        }
        RuinedPortalVerticalPlacement::InMountain => ruined_portal_get_random_within_interval(
            70,
            surface_y_at_center - y_span,
            inclusive_roll,
        ),
        RuinedPortalVerticalPlacement::Underground => ruined_portal_get_random_within_interval(
            min_y + 15,
            surface_y_at_center - y_span,
            inclusive_roll,
        ),
        RuinedPortalVerticalPlacement::PartlyBuried => {
            ruined_portal_get_random_within_interval(2, 8, inclusive_roll)
                .map(|offset| surface_y_at_center - y_span + offset)
        }
        RuinedPortalVerticalPlacement::OnLandSurface
        | RuinedPortalVerticalPlacement::OnOceanFloor => Ok(surface_y_at_center),
    }
}

pub fn ruined_portal_find_suitable_y(
    placement: RuinedPortalVerticalPlacement,
    min_y: i32,
    initial_y: i32,
    mut opaque_corners_at_y: impl FnMut(i32) -> usize,
) -> i32 {
    let min_scan_y = min_y + 15;
    for projected_y in (min_scan_y + 1..=initial_y).rev() {
        if opaque_corners_at_y(projected_y) >= 3 {
            return projected_y;
        }
    }
    let _heightmap = if placement == RuinedPortalVerticalPlacement::OnOceanFloor {
        "minecraft:ocean_floor_wg"
    } else {
        "minecraft:world_surface_wg"
    };
    min_scan_y
}

pub fn ruined_portal_make_properties(
    setup: RuinedPortalSetupModel,
    air_pocket_roll: f32,
    cold: bool,
) -> Result<RuinedPortalPropertiesModel, String> {
    Ok(RuinedPortalPropertiesModel {
        cold: setup.can_be_cold && cold,
        mossiness: setup.mossiness,
        air_pocket: ruined_portal_sample_probability(
            setup.air_pocket_probability,
            air_pocket_roll,
        )?,
        overgrown: setup.overgrown,
        vines: setup.vines,
        replace_with_blackstone: setup.replace_with_blackstone,
    })
}

pub fn ruined_portal_make_piece(
    template_name: &'static str,
    template_position: BlockPos,
    vertical_placement: RuinedPortalVerticalPlacement,
    properties: RuinedPortalPropertiesModel,
    rotation: StructureRotation,
    mirror: RuinedPortalMirrorModel,
    pivot: BlockPos,
) -> RuinedPortalPieceModel {
    let ignore_processor = if properties.air_pocket {
        "minecraft:block_ignore_structure_block"
    } else {
        "minecraft:block_ignore_structure_and_air"
    };
    let lava_replacement = if vertical_placement == RuinedPortalVerticalPlacement::OnOceanFloor {
        "minecraft:magma_block"
    } else if properties.cold {
        "minecraft:netherrack"
    } else {
        "minecraft:magma_block@0.2"
    };
    RuinedPortalPieceModel {
        template_name,
        template_position,
        vertical_placement,
        properties,
        rotation,
        mirror,
        pivot,
        ignore_processor,
        lava_replacement,
        include_blackstone_replace_processor: properties.replace_with_blackstone,
    }
}

pub const SHIPWRECK_BEACHED_TEMPLATES: [&str; 11] = [
    "minecraft:shipwreck/with_mast",
    "minecraft:shipwreck/sideways_full",
    "minecraft:shipwreck/sideways_fronthalf",
    "minecraft:shipwreck/sideways_backhalf",
    "minecraft:shipwreck/rightsideup_full",
    "minecraft:shipwreck/rightsideup_fronthalf",
    "minecraft:shipwreck/rightsideup_backhalf",
    "minecraft:shipwreck/with_mast_degraded",
    "minecraft:shipwreck/rightsideup_full_degraded",
    "minecraft:shipwreck/rightsideup_fronthalf_degraded",
    "minecraft:shipwreck/rightsideup_backhalf_degraded",
];

pub const SHIPWRECK_OCEAN_TEMPLATES: [&str; 20] = [
    "minecraft:shipwreck/with_mast",
    "minecraft:shipwreck/upsidedown_full",
    "minecraft:shipwreck/upsidedown_fronthalf",
    "minecraft:shipwreck/upsidedown_backhalf",
    "minecraft:shipwreck/sideways_full",
    "minecraft:shipwreck/sideways_fronthalf",
    "minecraft:shipwreck/sideways_backhalf",
    "minecraft:shipwreck/rightsideup_full",
    "minecraft:shipwreck/rightsideup_fronthalf",
    "minecraft:shipwreck/rightsideup_backhalf",
    "minecraft:shipwreck/with_mast_degraded",
    "minecraft:shipwreck/upsidedown_full_degraded",
    "minecraft:shipwreck/upsidedown_fronthalf_degraded",
    "minecraft:shipwreck/upsidedown_backhalf_degraded",
    "minecraft:shipwreck/sideways_full_degraded",
    "minecraft:shipwreck/sideways_fronthalf_degraded",
    "minecraft:shipwreck/sideways_backhalf_degraded",
    "minecraft:shipwreck/rightsideup_full_degraded",
    "minecraft:shipwreck/rightsideup_fronthalf_degraded",
    "minecraft:shipwreck/rightsideup_backhalf_degraded",
];

pub fn shipwreck_heightmap_type(is_beached: bool) -> &'static str {
    if is_beached {
        "minecraft:world_surface_wg"
    } else {
        "minecraft:ocean_floor_wg"
    }
}

pub fn shipwreck_template_name(
    is_beached: bool,
    template_index: usize,
) -> Result<&'static str, String> {
    let templates = if is_beached {
        &SHIPWRECK_BEACHED_TEMPLATES[..]
    } else {
        &SHIPWRECK_OCEAN_TEMPLATES[..]
    };
    templates.get(template_index).copied().ok_or_else(|| {
        "Shipwreck template index must match Util.getRandom template list".to_string()
    })
}

pub fn shipwreck_make_piece(
    chunk_pos: ChunkPos,
    rotation: StructureRotation,
    template_index: usize,
    is_beached: bool,
) -> Result<ShipwreckPieceModel, String> {
    Ok(ShipwreckPieceModel {
        template_name: shipwreck_template_name(is_beached, template_index)?,
        template_position: BlockPos {
            x: chunk_pos.x * 16,
            y: 90,
            z: chunk_pos.z * 16,
        },
        rotation,
        is_beached,
        height_adjusted: false,
        pivot: BlockPos { x: 4, y: 0, z: 15 },
        processor: "minecraft:block_ignore_structure_and_air",
    })
}

pub fn shipwreck_is_too_big_to_fit_in_worldgen_region(template_size: BlockPos) -> bool {
    template_size.x > 32 || template_size.y > 32
}

pub fn shipwreck_calculate_beached_position(
    min_y: i32,
    template_height: i32,
    random_roll: i32,
) -> Result<i32, String> {
    if !(0..3).contains(&random_roll) {
        return Err("Shipwreck beached height roll must match RandomSource#nextInt(3)".to_string());
    }
    Ok(min_y - template_height / 2 - random_roll)
}

pub fn shipwreck_adjust_position_height(
    mut piece: ShipwreckPieceModel,
    new_height: i32,
) -> ShipwreckPieceModel {
    piece.height_adjusted = true;
    piece.template_position.y = new_height;
    piece
}

pub fn shipwreck_save_tag(piece: ShipwreckPieceModel) -> ShipwreckSaveTagModel {
    ShipwreckSaveTagModel {
        is_beached: piece.is_beached,
        rotation: piece.rotation,
        height_adjusted: piece.height_adjusted,
    }
}

pub fn shipwreck_loot_table_for_marker(marker_id: &str) -> Option<&'static str> {
    match marker_id {
        "map_chest" => Some("minecraft:chests/shipwreck_map"),
        "treasure_chest" => Some("minecraft:chests/shipwreck_treasure"),
        "supply_chest" => Some("minecraft:chests/shipwreck_supply"),
        _ => None,
    }
}

