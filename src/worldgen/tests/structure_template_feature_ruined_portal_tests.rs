#[test]
fn ruined_portal_setup_template_and_vertical_rules_match_vanilla() {
    assert_ruined_portal_setup_probability_and_template_rules();
    assert_ruined_portal_initial_and_suitable_y_rules();
    assert_ruined_portal_properties_and_piece_processors();
}

fn ruined_portal_land_setup() -> super::super::RuinedPortalSetupModel {
    super::super::RuinedPortalSetupModel {
        placement: super::super::RuinedPortalVerticalPlacement::OnLandSurface,
        air_pocket_probability: 0.0,
        mossiness: 0.2,
        overgrown: false,
        vines: true,
        can_be_cold: true,
        replace_with_blackstone: false,
        weight: 1.0,
    }
}

fn ruined_portal_nether_setup() -> super::super::RuinedPortalSetupModel {
    super::super::RuinedPortalSetupModel {
        placement: super::super::RuinedPortalVerticalPlacement::InNether,
        air_pocket_probability: 1.0,
        mossiness: 0.8,
        overgrown: false,
        vines: false,
        can_be_cold: false,
        replace_with_blackstone: true,
        weight: 3.0,
    }
}

fn assert_ruined_portal_setup_probability_and_template_rules() {
    let land_setup = ruined_portal_land_setup();
    let nether_setup = ruined_portal_nether_setup();
    assert_eq!(
        super::super::ruined_portal_choose_setup(&[land_setup, nether_setup], 0.24).unwrap(),
        land_setup
    );
    assert_eq!(
        super::super::ruined_portal_choose_setup(&[land_setup, nether_setup], 0.25).unwrap(),
        nether_setup
    );
    assert_eq!(
        super::super::ruined_portal_vertical_placement_id(
            super::super::RuinedPortalVerticalPlacement::PartlyBuried
        ),
        "partly_buried"
    );
    assert!(!super::super::ruined_portal_sample_probability(0.0, 0.0).unwrap());
    assert!(super::super::ruined_portal_sample_probability(1.0, 0.99).unwrap());
    assert!(super::super::ruined_portal_sample_probability(0.3, 0.29).unwrap());
    assert!(!super::super::ruined_portal_sample_probability(0.3, 0.3).unwrap());

    assert_eq!(
        super::super::ruined_portal_template_name(0.049, 2).unwrap(),
        "minecraft:ruined_portal/giant_portal_3"
    );
    assert_eq!(
        super::super::ruined_portal_template_name(0.05, 9).unwrap(),
        "minecraft:ruined_portal/portal_10"
    );
    assert_eq!(
        super::super::ruined_portal_mirror(0.49).unwrap(),
        super::super::RuinedPortalMirrorModel::None
    );
    assert_eq!(
        super::super::ruined_portal_mirror(0.5).unwrap(),
        super::super::RuinedPortalMirrorModel::FrontBack
    );
}

fn assert_ruined_portal_initial_and_suitable_y_rules() {
    assert_eq!(
        super::super::ruined_portal_initial_y(
            super::super::RuinedPortalVerticalPlacement::InNether,
            true,
            80,
            12,
            -64,
            0.75,
            5,
        )
        .unwrap(),
        37
    );
    assert_eq!(
        super::super::ruined_portal_initial_y(
            super::super::RuinedPortalVerticalPlacement::InNether,
            false,
            80,
            12,
            -64,
            0.49,
            2,
        )
        .unwrap(),
        29
    );
    assert_eq!(
        super::super::ruined_portal_initial_y(
            super::super::RuinedPortalVerticalPlacement::InNether,
            false,
            80,
            12,
            -64,
            0.5,
            0,
        )
        .unwrap(),
        29
    );
    assert_eq!(
        super::super::ruined_portal_initial_y(
            super::super::RuinedPortalVerticalPlacement::InMountain,
            false,
            96,
            20,
            -64,
            0.0,
            3,
        )
        .unwrap(),
        73
    );
    assert_eq!(
        super::super::ruined_portal_initial_y(
            super::super::RuinedPortalVerticalPlacement::Underground,
            false,
            50,
            20,
            -64,
            0.0,
            79,
        )
        .unwrap(),
        30
    );
    assert_eq!(
        super::super::ruined_portal_initial_y(
            super::super::RuinedPortalVerticalPlacement::PartlyBuried,
            false,
            90,
            16,
            -64,
            0.0,
            4,
        )
        .unwrap(),
        80
    );

    let suitable = super::super::ruined_portal_find_suitable_y(
        super::super::RuinedPortalVerticalPlacement::OnOceanFloor,
        -64,
        80,
        |y| {
            if y == 72 {
                3
            } else {
                2
            }
        },
    );
    assert_eq!(suitable, 72);
    assert_eq!(
        super::super::ruined_portal_find_suitable_y(
            super::super::RuinedPortalVerticalPlacement::Underground,
            -64,
            -40,
            |_| 0,
        ),
        -49
    );
}

fn assert_ruined_portal_properties_and_piece_processors() {
    let properties =
        super::super::ruined_portal_make_properties(ruined_portal_nether_setup(), 0.0, true)
            .unwrap();
    assert_eq!(
        properties,
        super::super::RuinedPortalPropertiesModel {
            cold: false,
            mossiness: 0.8,
            air_pocket: true,
            overgrown: false,
            vines: false,
            replace_with_blackstone: true,
        }
    );
    let piece = super::super::ruined_portal_make_piece(
        "minecraft:ruined_portal/portal_1",
        BlockPos {
            x: 16,
            y: 37,
            z: -16,
        },
        super::super::RuinedPortalVerticalPlacement::InNether,
        properties,
        super::super::StructureRotation::Clockwise90,
        super::super::RuinedPortalMirrorModel::FrontBack,
        BlockPos { x: 4, y: 0, z: 5 },
    );
    assert_eq!(
        piece.ignore_processor,
        "minecraft:block_ignore_structure_block"
    );
    assert_eq!(piece.lava_replacement, "minecraft:magma_block@0.2");
    assert!(piece.include_blackstone_replace_processor);
    assert_eq!(piece.pivot, BlockPos { x: 4, y: 0, z: 5 });

    let ocean_piece = super::super::ruined_portal_make_piece(
        "minecraft:ruined_portal/portal_2",
        BlockPos { x: 0, y: 0, z: 0 },
        super::super::RuinedPortalVerticalPlacement::OnOceanFloor,
        super::super::RuinedPortalPropertiesModel {
            cold: true,
            mossiness: 0.0,
            air_pocket: false,
            overgrown: false,
            vines: false,
            replace_with_blackstone: false,
        },
        super::super::StructureRotation::None,
        super::super::RuinedPortalMirrorModel::None,
        BlockPos { x: 0, y: 0, z: 0 },
    );
    assert_eq!(
        ocean_piece.ignore_processor,
        "minecraft:block_ignore_structure_and_air"
    );
    assert_eq!(ocean_piece.lava_replacement, "minecraft:magma_block");
}

#[test]
fn shipwreck_template_height_and_loot_rules_match_vanilla() {
    assert_shipwreck_template_catalog_and_heightmap_rules();
    assert_shipwreck_piece_save_and_worldgen_region_rules();
    assert_shipwreck_height_adjustment_and_loot_rules();
}

fn shipwreck_test_piece() -> super::super::ShipwreckPieceModel {
    super::super::shipwreck_make_piece(
        ChunkPos { x: -1, z: 4 },
        super::super::StructureRotation::Clockwise180,
        7,
        false,
    )
    .unwrap()
}

fn assert_shipwreck_template_catalog_and_heightmap_rules() {
    assert_eq!(super::super::SHIPWRECK_BEACHED_TEMPLATES.len(), 11);
    assert_eq!(super::super::SHIPWRECK_OCEAN_TEMPLATES.len(), 20);
    assert_eq!(
        super::super::SHIPWRECK_BEACHED_TEMPLATES[0],
        "minecraft:shipwreck/with_mast"
    );
    assert_eq!(
        super::super::SHIPWRECK_OCEAN_TEMPLATES[19],
        "minecraft:shipwreck/rightsideup_backhalf_degraded"
    );
    assert_eq!(
        super::super::shipwreck_heightmap_type(true),
        "minecraft:world_surface_wg"
    );
    assert_eq!(
        super::super::shipwreck_heightmap_type(false),
        "minecraft:ocean_floor_wg"
    );
    assert_eq!(
        super::super::shipwreck_template_name(true, 10).unwrap(),
        "minecraft:shipwreck/rightsideup_backhalf_degraded"
    );
    assert_eq!(
        super::super::shipwreck_template_name(false, 11).unwrap(),
        "minecraft:shipwreck/upsidedown_full_degraded"
    );
    assert_eq!(
        super::super::shipwreck_template_name(true, 11).unwrap_err(),
        "Shipwreck template index must match Util.getRandom template list".to_string()
    );
}

fn assert_shipwreck_piece_save_and_worldgen_region_rules() {
    let piece = shipwreck_test_piece();
    assert_eq!(piece.template_name, "minecraft:shipwreck/rightsideup_full");
    assert_eq!(
        piece.template_position,
        BlockPos {
            x: -16,
            y: 90,
            z: 64
        }
    );
    assert_eq!(piece.pivot, BlockPos { x: 4, y: 0, z: 15 });
    assert_eq!(piece.processor, "minecraft:block_ignore_structure_and_air");
    assert!(!piece.height_adjusted);
    assert!(!piece.is_beached);
    assert_eq!(
        super::super::shipwreck_save_tag(piece),
        super::super::ShipwreckSaveTagModel {
            is_beached: false,
            rotation: super::super::StructureRotation::Clockwise180,
            height_adjusted: false,
        }
    );

    assert!(
        !super::super::shipwreck_is_too_big_to_fit_in_worldgen_region(BlockPos {
            x: 32,
            y: 32,
            z: 80
        })
    );
    assert!(
        super::super::shipwreck_is_too_big_to_fit_in_worldgen_region(BlockPos {
            x: 33,
            y: 12,
            z: 12
        })
    );
    assert!(
        super::super::shipwreck_is_too_big_to_fit_in_worldgen_region(BlockPos {
            x: 12,
            y: 33,
            z: 12
        })
    );
}

fn assert_shipwreck_height_adjustment_and_loot_rules() {
    assert_eq!(
        super::super::shipwreck_calculate_beached_position(72, 15, 2).unwrap(),
        63
    );
    assert_eq!(
        super::super::shipwreck_calculate_beached_position(72, 15, 3).unwrap_err(),
        "Shipwreck beached height roll must match RandomSource#nextInt(3)".to_string()
    );
    let piece = shipwreck_test_piece();
    let adjusted = super::super::shipwreck_adjust_position_height(piece, 48);
    assert_eq!(adjusted.template_position.y, 48);
    assert!(adjusted.height_adjusted);
    assert_eq!(
        super::super::shipwreck_save_tag(adjusted),
        super::super::ShipwreckSaveTagModel {
            is_beached: false,
            rotation: super::super::StructureRotation::Clockwise180,
            height_adjusted: true,
        }
    );

    assert_eq!(
        super::super::shipwreck_loot_table_for_marker("map_chest"),
        Some("minecraft:chests/shipwreck_map")
    );
    assert_eq!(
        super::super::shipwreck_loot_table_for_marker("treasure_chest"),
        Some("minecraft:chests/shipwreck_treasure")
    );
    assert_eq!(
        super::super::shipwreck_loot_table_for_marker("supply_chest"),
        Some("minecraft:chests/shipwreck_supply")
    );
    assert_eq!(
        super::super::shipwreck_loot_table_for_marker("unknown"),
        None
    );
}

#[test]
fn ocean_ruin_piece_templates_markers_and_height_match_vanilla() {
    assert_ocean_ruin_template_catalog_and_probability_rules();
    assert_ocean_ruin_piece_selection_and_save_tags();
    assert_ocean_ruin_marker_actions();
    assert_ocean_ruin_floor_adjustment_rules();
}

fn ocean_ruin_warm_config() -> super::super::OceanRuinStructureConfigModel {
    super::super::OceanRuinStructureConfigModel {
        biome_type: super::super::OceanRuinBiomeType::Warm,
        large_probability: 0.3,
        cluster_probability: 0.9,
    }
}

fn ocean_ruin_cold_config() -> super::super::OceanRuinStructureConfigModel {
    super::super::OceanRuinStructureConfigModel {
        biome_type: super::super::OceanRuinBiomeType::Cold,
        large_probability: 1.0,
        cluster_probability: 0.0,
    }
}

fn ocean_ruin_warm_piece() -> super::super::OceanRuinPieceModel {
    super::super::ocean_ruin_add_piece(
        ocean_ruin_warm_config(),
        BlockPos {
            x: 16,
            y: 90,
            z: -32,
        },
        super::super::StructureRotation::Clockwise90,
        true,
        0.9,
        2,
    )
    .unwrap()[0]
}

fn ocean_ruin_cold_pieces() -> Vec<super::super::OceanRuinPieceModel> {
    super::super::ocean_ruin_add_piece(
        ocean_ruin_cold_config(),
        BlockPos { x: 0, y: 90, z: 0 },
        super::super::StructureRotation::None,
        false,
        0.8,
        4,
    )
    .unwrap()
}

fn assert_ocean_ruin_template_catalog_and_probability_rules() {
    assert_eq!(super::super::OCEAN_RUIN_WARM_TEMPLATES.len(), 8);
    assert_eq!(super::super::OCEAN_RUIN_BIG_WARM_TEMPLATES.len(), 4);
    assert_eq!(
        super::super::OCEAN_RUIN_BRICK_TEMPLATES[0],
        "minecraft:underwater_ruin/brick_1"
    );
    assert_eq!(
        super::super::OCEAN_RUIN_BIG_CRACKED_TEMPLATES[3],
        "minecraft:underwater_ruin/big_cracked_8"
    );
    assert_eq!(
        super::super::ocean_ruin_biome_type_id(super::super::OceanRuinBiomeType::Warm),
        "warm"
    );
    assert_eq!(
        super::super::ocean_ruin_biome_type_id(super::super::OceanRuinBiomeType::Cold),
        "cold"
    );
    assert!(super::super::ocean_ruin_is_large(0.4, 0.4).unwrap());
    assert!(!super::super::ocean_ruin_is_large(0.4, 0.4001).unwrap());
    assert!(super::super::ocean_ruin_should_add_cluster(0.2, 0.2).unwrap());
    assert!(!super::super::ocean_ruin_should_add_cluster(0.2, 0.21).unwrap());
}

fn assert_ocean_ruin_piece_selection_and_save_tags() {
    let warm_piece = super::super::ocean_ruin_add_piece(
        ocean_ruin_warm_config(),
        BlockPos {
            x: 16,
            y: 90,
            z: -32,
        },
        super::super::StructureRotation::Clockwise90,
        true,
        0.9,
        2,
    )
    .unwrap();
    assert_eq!(warm_piece.len(), 1);
    assert_eq!(
        warm_piece[0].template_name,
        "minecraft:underwater_ruin/big_warm_6"
    );
    assert_eq!(warm_piece[0].integrity, 0.9);
    assert_eq!(warm_piece[0].suspicious_block, "minecraft:suspicious_sand");
    assert_eq!(
        warm_piece[0].suspicious_loot_table,
        "minecraft:archaeology/ocean_ruin_warm"
    );

    let cold_pieces = super::super::ocean_ruin_add_piece(
        ocean_ruin_cold_config(),
        BlockPos { x: 0, y: 90, z: 0 },
        super::super::StructureRotation::None,
        false,
        0.8,
        4,
    )
    .unwrap();
    assert_eq!(cold_pieces.len(), 3);
    assert_eq!(
        cold_pieces
            .iter()
            .map(|piece| (piece.template_name, piece.integrity))
            .collect::<Vec<_>>(),
        vec![
            ("minecraft:underwater_ruin/brick_5", 0.8),
            ("minecraft:underwater_ruin/cracked_5", 0.7),
            ("minecraft:underwater_ruin/mossy_5", 0.5),
        ]
    );
    assert_eq!(
        cold_pieces[0].suspicious_block,
        "minecraft:suspicious_gravel"
    );
    assert_eq!(
        super::super::ocean_ruin_save_tag(cold_pieces[1]),
        super::super::OceanRuinSaveTagModel {
            rotation: super::super::StructureRotation::None,
            integrity: 0.7,
            biome_type: super::super::OceanRuinBiomeType::Cold,
            is_large: false,
        }
    );
}

fn assert_ocean_ruin_marker_actions() {
    let warm_piece = ocean_ruin_warm_piece();
    let cold_pieces = ocean_ruin_cold_pieces();
    assert_eq!(
        super::super::ocean_ruin_marker_action(
            warm_piece,
            "chest",
            BlockPos { x: 1, y: 50, z: 2 },
            63,
            true
        ),
        Some(super::super::OceanRuinMarkerActionModel {
            marker_id: "chest",
            pos: BlockPos { x: 1, y: 50, z: 2 },
            placed_block: "minecraft:chest[waterlogged=true]",
            loot_table: Some("minecraft:chests/underwater_ruin_big"),
            spawned_entity: None,
        })
    );
    assert_eq!(
        super::super::ocean_ruin_marker_action(
            cold_pieces[0],
            "drowned",
            BlockPos { x: 1, y: 70, z: 2 },
            63,
            false
        ),
        Some(super::super::OceanRuinMarkerActionModel {
            marker_id: "drowned",
            pos: BlockPos { x: 1, y: 70, z: 2 },
            placed_block: "minecraft:air",
            loot_table: None,
            spawned_entity: Some("minecraft:drowned"),
        })
    );
    assert_eq!(
        super::super::ocean_ruin_marker_action(
            cold_pieces[0],
            "drowned",
            BlockPos { x: 1, y: 50, z: 2 },
            63,
            false
        )
        .unwrap()
        .placed_block,
        "minecraft:water"
    );
    assert_eq!(
        super::super::ocean_ruin_marker_action(
            cold_pieces[0],
            "unknown",
            BlockPos { x: 0, y: 0, z: 0 },
            63,
            false
        ),
        None
    );
}

fn assert_ocean_ruin_floor_adjustment_rules() {
    let warm_piece = ocean_ruin_warm_piece();
    let adjusted = super::super::ocean_ruin_adjust_to_ocean_floor(
        warm_piece,
        70,
        BlockPos { x: 4, y: 6, z: 4 },
        |x, z| {
            if x == 16 && z == -32 {
                69
            } else {
                64
            }
        },
    );
    assert_eq!(adjusted.template_position.y, 65);
    let flat = super::super::ocean_ruin_adjust_to_ocean_floor(
        warm_piece,
        70,
        BlockPos { x: 4, y: 6, z: 4 },
        |_, _| 69,
    );
    assert_eq!(flat.template_position.y, 70);
}
