use super::*;

#[test]
fn jigsaw_and_processor_registries_match_vanilla_bootstrap_surface() {
    assert_eq!(STRUCTURE_POOL_ELEMENT_TYPES.len(), 5);
    assert_eq!(STRUCTURE_PROCESSOR_TYPES.len(), 11);
    assert_eq!(STRUCTURE_RULE_TEST_TYPES.len(), 6);
    assert_eq!(STRUCTURE_POS_RULE_TEST_TYPES.len(), 3);
    assert_eq!(STRUCTURE_PIECE_TYPES.len(), 56);
    assert_eq!(STRUCTURE_PROCESSOR_LISTS.len(), 40);
    assert_eq!(
        STRUCTURE_PROCESSOR_LISTS.first().copied(),
        Some("minecraft:empty")
    );
    assert_eq!(
        STRUCTURE_PROCESSOR_LISTS.last().copied(),
        Some("minecraft:trial_chambers_copper_bulb_degradation")
    );
    assert!(STRUCTURE_PROCESSOR_TYPES.contains(&"minecraft:jigsaw_replacement"));
    assert!(STRUCTURE_POOL_ELEMENT_TYPES.contains(&"minecraft:legacy_single_pool_element"));
    assert_eq!(STRUCTURE_PIECE_TYPES.first().copied(), Some("mscorridor"));
    assert_eq!(STRUCTURE_PIECE_TYPES.last().copied(), Some("jigsaw"));
    assert!(STRUCTURE_PIECE_TYPES.contains(&"shpr"));
    assert!(STRUCTURE_PIECE_TYPES.contains(&"shipwreck"));

    assert_eq!(JIGSAW_POOL_BOOTSTRAP_SOURCES.len(), 18);
    assert_eq!(
        JIGSAW_POOL_BOOTSTRAP_SOURCES
            .iter()
            .map(|source| source.registrations)
            .sum::<usize>(),
        176
    );
    assert_eq!(
        JIGSAW_POOL_BOOTSTRAP_SOURCES
            .iter()
            .find(|source| source.source_file == "TrialChambersStructurePools.java")
            .map(|source| source.registrations),
        Some(34)
    );
    assert_eq!(
        JIGSAW_POOL_BOOTSTRAP_SOURCES
            .iter()
            .find(|source| source.source_file == "PlainVillagePools.java")
            .map(|source| source.registrations),
        Some(17)
    );
}

#[test]
fn terrain_blending_constants_match_vanilla() {
    assert_eq!(BLENDING_CONSTANTS.height_blending_range_cells, 27);
    assert_eq!(BLENDING_CONSTANTS.height_blending_range_chunks, 7);
    assert_eq!(BLENDING_CONSTANTS.density_blending_range_cells, 2);
    assert_eq!(BLENDING_CONSTANTS.density_blending_range_chunks, 2);
    assert_eq!(BLENDING_CONSTANTS.old_chunk_xz_radius, 8);
    assert_eq!(BLENDING_CONSTANTS.cell_width, 4);
    assert_eq!(BLENDING_CONSTANTS.cell_height, 8);
    assert_eq!(BLENDING_CONSTANTS.cell_ratio, 2);
    assert_eq!(BLENDING_CELL_COLUMN_COUNT, 16);
    assert_eq!(BLENDING_NO_VALUE, f64::MAX);
}

#[test]
fn terrain_blending_output_and_validation_match_vanilla() {
    assert_eq!(super::super::blending_smooth_alpha(0.0, 27), 0.0);
    assert_eq!(super::super::blending_smooth_alpha(28.0, 27), 1.0);
    assert!((super::super::blending_smooth_alpha(14.0, 27) - 0.5).abs() < f64::EPSILON);
    assert!((super::super::blending_height_to_offset(127.5)).abs() < f64::EPSILON);
    assert!(super::super::blending_height_to_offset(63.5) < 0.0);
    assert!(super::super::blending_height_to_offset(191.5) > 0.0);
    assert_eq!(
        super::super::blending_output_for_old_height(None, None),
        BlendingOutput {
            alpha: 1.0,
            blending_offset: 0.0,
        }
    );
    assert_eq!(
        super::super::blending_output_for_old_height(Some(127.5), None),
        BlendingOutput {
            alpha: 0.0,
            blending_offset: 0.0,
        }
    );
    let blended = super::super::blending_output_for_old_height(Some(63.5), Some(14.0));
    assert!((blended.alpha - 0.5).abs() < f64::EPSILON);
    assert!(blended.blending_offset < 0.0);
    assert_eq!(
        super::super::validate_blending_data_packed(BlendingDataPacked {
            min_section: -4,
            max_section: 20,
            heights: Some(&[0.0; BLENDING_CELL_COLUMN_COUNT]),
        }),
        Ok(())
    );
    assert_eq!(
        super::super::validate_blending_data_packed(BlendingDataPacked {
            min_section: -4,
            max_section: 20,
            heights: Some(&[0.0; BLENDING_CELL_COLUMN_COUNT - 1]),
        }),
        Err("heights has to be of length 16".to_string())
    );
}

#[test]
fn upgrade_data_model_matches_vanilla_constants() {
    assert_eq!(UPGRADE_DATA_MODEL.tag_indices, "Indices");
    assert_eq!(UPGRADE_DATA_MODEL.tag_sides, "Sides");
    assert_eq!(
        UPGRADE_DATA_MODEL.tag_neighbor_block_ticks,
        "neighbor_block_ticks"
    );
    assert_eq!(
        UPGRADE_DATA_MODEL.tag_neighbor_fluid_ticks,
        "neighbor_fluid_ticks"
    );
    assert_eq!(
        UPGRADE_DATA_MODEL.block_fixers,
        &["blacklist", "default", "chest", "leaves", "stem_block"]
    );
    assert_eq!(UPGRADE_DATA_MODEL.chunky_fixers, &["leaves"]);
}

#[test]
fn below_zero_retrogen_model_matches_vanilla_constants() {
    assert_eq!(
        super::super::BELOW_ZERO_RETROGEN_MODEL.target_status_field,
        "target_status"
    );
    assert_eq!(
        super::super::BELOW_ZERO_RETROGEN_MODEL.missing_bedrock_field,
        "missing_bedrock"
    );
    assert_eq!(super::super::BELOW_ZERO_RETROGEN_MODEL.upgrade_min_y, -64);
    assert_eq!(super::super::BELOW_ZERO_RETROGEN_MODEL.upgrade_height, 64);
    assert_eq!(
        super::super::BELOW_ZERO_RETROGEN_MODEL.max_generated_bedrock_y,
        4
    );
    assert_eq!(
        super::super::BELOW_ZERO_RETROGEN_MODEL.retained_biomes,
        &[
            "minecraft:lush_caves",
            "minecraft:dripstone_caves",
            "minecraft:deep_dark",
        ]
    );
}

#[test]
fn below_zero_retrogen_bedrock_replacement_matches_vanilla() {
    assert_eq!(
        super::super::below_zero_replace_old_bedrock_action(4, "minecraft:bedrock"),
        Some("minecraft:deepslate")
    );
    assert_eq!(
        super::super::below_zero_replace_old_bedrock_action(5, "minecraft:bedrock"),
        None
    );
    assert_eq!(
        super::super::below_zero_replace_old_bedrock_action(4, "minecraft:stone"),
        None
    );
    assert_eq!(super::super::below_zero_missing_bedrock_bit_index(1, 2), 33);
    assert_eq!(
        super::super::below_zero_missing_bedrock_bit_index(17, 18),
        33
    );
    let missing_bedrock = [1_u64 << 33];
    assert!(super::super::below_zero_has_bedrock_hole(
        &missing_bedrock,
        1,
        2
    ));
    assert!(!super::super::below_zero_has_bedrock_hole(
        &missing_bedrock,
        2,
        2
    ));
    assert_eq!(
        super::super::below_zero_bedrock_mask_air_columns(-1, 1, &missing_bedrock),
        vec![
            super::super::FeaturePlacementBlock {
                pos: BlockPos { x: 1, y: -1, z: 2 },
                state: "minecraft:air",
            },
            super::super::FeaturePlacementBlock {
                pos: BlockPos { x: 1, y: 0, z: 2 },
                state: "minecraft:air",
            },
            super::super::FeaturePlacementBlock {
                pos: BlockPos { x: 1, y: 1, z: 2 },
                state: "minecraft:air",
            },
        ]
    );
}

#[test]
fn below_zero_retrogen_biome_selection_matches_vanilla() {
    assert_eq!(
        super::super::below_zero_retrogen_biome(
            true,
            "minecraft:plains",
            "minecraft:old_growth_birch_forest",
        ),
        "minecraft:old_growth_birch_forest"
    );
    assert_eq!(
        super::super::below_zero_retrogen_biome(
            true,
            "minecraft:lush_caves",
            "minecraft:old_growth_birch_forest",
        ),
        "minecraft:lush_caves"
    );
    assert_eq!(
        super::super::below_zero_retrogen_biome(
            false,
            "minecraft:plains",
            "minecraft:old_growth_birch_forest",
        ),
        "minecraft:plains"
    );
}
