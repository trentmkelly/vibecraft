use super::*;

#[test]
fn jigsaw_projection_liquid_padding_and_distance_models_match_vanilla_codecs() {
    assert_eq!(
        super::super::JigsawProjectionModel::TerrainMatching.id(),
        "terrain_matching"
    );
    assert_eq!(super::super::JigsawProjectionModel::Rigid.id(), "rigid");
    assert_eq!(
        super::super::JigsawProjectionModel::from_id("terrain_matching"),
        Some(super::super::JigsawProjectionModel::TerrainMatching)
    );
    assert_eq!(super::super::JigsawProjectionModel::from_id("loose"), None);
    assert_eq!(
        super::super::JigsawProjectionModel::TerrainMatching.processor_ids(),
        &["minecraft:gravity"]
    );
    assert!(super::super::JigsawProjectionModel::Rigid
        .processor_ids()
        .is_empty());

    assert_eq!(
        super::super::LiquidSettingsModel::from_id("apply_waterlogging"),
        Some(super::super::LiquidSettingsModel::ApplyWaterlogging)
    );
    assert_eq!(
        super::super::LiquidSettingsModel::IgnoreWaterlogging.id(),
        "ignore_waterlogging"
    );
    assert!(super::super::LiquidSettingsModel::ApplyWaterlogging.should_apply_waterlogging());
    assert!(!super::super::LiquidSettingsModel::IgnoreWaterlogging.should_apply_waterlogging());

    assert_eq!(
        super::super::DimensionPaddingModel::ZERO,
        super::super::DimensionPaddingModel { bottom: 0, top: 0 }
    );
    assert_eq!(
        super::super::DimensionPaddingModel::uniform(12),
        Ok(super::super::DimensionPaddingModel {
            bottom: 12,
            top: 12
        })
    );
    assert_eq!(
        super::super::DimensionPaddingModel::new(3, 7),
        Ok(super::super::DimensionPaddingModel { bottom: 3, top: 7 })
    );
    assert!(super::super::DimensionPaddingModel::uniform(12)
        .unwrap()
        .has_equal_top_and_bottom());
    assert!(!super::super::DimensionPaddingModel::new(3, 7)
        .unwrap()
        .has_equal_top_and_bottom());
    assert_eq!(
        super::super::DimensionPaddingModel::new(-1, 0),
        Err("dimension padding values must be non-negative".to_string())
    );

    assert_eq!(
        super::super::JigsawMaxDistanceModel::DEFAULT,
        super::super::JigsawMaxDistanceModel {
            horizontal: 80,
            vertical: 80,
        }
    );
    assert_eq!(
        super::super::JigsawMaxDistanceModel::uniform(80),
        Ok(super::super::JigsawMaxDistanceModel {
            horizontal: 80,
            vertical: 80,
        })
    );
    assert_eq!(
        super::super::JigsawMaxDistanceModel::new(128, 384),
        Ok(super::super::JigsawMaxDistanceModel {
            horizontal: 128,
            vertical: 384,
        })
    );
    assert!(super::super::JigsawMaxDistanceModel::uniform(32)
        .unwrap()
        .can_encode_as_uniform());
    assert!(!super::super::JigsawMaxDistanceModel::new(32, 64)
        .unwrap()
        .can_encode_as_uniform());
    assert_eq!(
        super::super::JigsawMaxDistanceModel::new(0, 64),
        Err("jigsaw horizontal max distance must be in 1..=128".to_string())
    );
    assert_eq!(
        super::super::JigsawMaxDistanceModel::new(64, 385),
        Err("jigsaw vertical max distance must be in 1..=384".to_string())
    );
}

#[test]
fn jigsaw_connector_orientation_and_attachment_match_vanilla_can_attach() {
    use super::super::JigsawDirectionModel::{East, North, South, Up, West};
    use super::super::JigsawJointTypeModel::{Aligned, Rollable};

    assert_eq!(
        super::super::JigsawDirectionModel::from_id("north"),
        Some(North)
    );
    assert_eq!(
        super::super::JigsawDirectionModel::from_id("sideways"),
        None
    );
    assert_eq!(North.id(), "north");
    assert_eq!(North.opposite(), South);
    assert_eq!(East.step(), super::super::BlockPos { x: 1, y: 0, z: 0 });
    assert_eq!(
        super::super::JigsawJointTypeModel::from_id("aligned"),
        Some(Aligned)
    );
    assert_eq!(Rollable.id(), "rollable");

    let source = super::super::JigsawConnectorModel {
        name: "minecraft:road",
        target: "minecraft:house",
        pool: "minecraft:village/plains/houses",
        front: North,
        top: Up,
        joint: Aligned,
        placement_priority: 3,
        selection_priority: 7,
    };
    let matching_target = super::super::JigsawConnectorModel {
        name: "minecraft:house",
        target: "minecraft:road",
        pool: "minecraft:empty",
        front: South,
        top: Up,
        joint: Rollable,
        placement_priority: 0,
        selection_priority: 0,
    };
    assert!(super::super::jigsaw_connectors_can_attach(
        &source,
        &matching_target
    ));
    assert_eq!(
        source.target_pos(super::super::BlockPos {
            x: 10,
            y: 64,
            z: -5
        }),
        super::super::BlockPos {
            x: 10,
            y: 64,
            z: -6
        }
    );

    let wrong_front = super::super::JigsawConnectorModel {
        front: West,
        ..matching_target
    };
    assert!(!super::super::jigsaw_connectors_can_attach(
        &source,
        &wrong_front
    ));

    let wrong_top = super::super::JigsawConnectorModel {
        top: East,
        ..matching_target
    };
    assert!(!super::super::jigsaw_connectors_can_attach(
        &source, &wrong_top
    ));

    let rollable_source = super::super::JigsawConnectorModel {
        joint: Rollable,
        ..source
    };
    assert!(super::super::jigsaw_connectors_can_attach(
        &rollable_source,
        &wrong_top
    ));

    let wrong_name = super::super::JigsawConnectorModel {
        name: "minecraft:stable",
        ..matching_target
    };
    assert!(!super::super::jigsaw_connectors_can_attach(
        &source,
        &wrong_name
    ));
}

#[test]
fn sequenced_priority_queue_matches_java_highest_priority_fifo_order() {
    let mut queue = super::super::SequencedPriorityQueueModel::new();
    assert!(queue.is_empty());
    assert_eq!(queue.highest_priority(), None);

    queue.add("low-a", -2);
    queue.add("high-a", 5);
    queue.add("mid-a", 1);
    queue.add("high-b", 5);
    queue.add("top-a", 9);

    assert_eq!(queue.highest_priority(), Some(9));
    assert_eq!(queue.next_item(), Some("top-a"));

    assert_eq!(queue.highest_priority(), Some(5));
    assert_eq!(queue.next_item(), Some("high-a"));

    queue.add("high-c", 5);
    queue.add("higher-late", 7);
    assert_eq!(queue.highest_priority(), Some(7));
    assert_eq!(queue.next_item(), Some("higher-late"));

    assert_eq!(queue.highest_priority(), Some(5));
    assert_eq!(queue.next_item(), Some("high-b"));
    assert_eq!(queue.next_item(), Some("high-c"));
    assert_eq!(queue.next_item(), Some("mid-a"));
    assert_eq!(queue.next_item(), Some("low-a"));
    assert_eq!(queue.next_item(), None);
    assert!(queue.is_empty());
}

#[test]
fn jigsaw_start_height_limit_rejection_matches_dimension_padding_rules() {
    let fits_at_padded_edges = super::super::StructureBoundingBoxModel {
        min_x: 0,
        min_y: -60,
        min_z: 0,
        max_x: 8,
        max_y: 311,
        max_z: 8,
    };
    assert!(
        !super::super::jigsaw_start_too_close_to_world_height_limits(
            -64,
            384,
            super::super::DimensionPaddingModel { bottom: 4, top: 8 },
            fits_at_padded_edges,
        )
    );

    let below_padding = super::super::StructureBoundingBoxModel {
        min_y: -61,
        ..fits_at_padded_edges
    };
    assert!(super::super::jigsaw_start_too_close_to_world_height_limits(
        -64,
        384,
        super::super::DimensionPaddingModel { bottom: 4, top: 8 },
        below_padding,
    ));

    let above_padding = super::super::StructureBoundingBoxModel {
        max_y: 312,
        ..fits_at_padded_edges
    };
    assert!(super::super::jigsaw_start_too_close_to_world_height_limits(
        -64,
        384,
        super::super::DimensionPaddingModel { bottom: 4, top: 8 },
        above_padding,
    ));

    assert!(
        !super::super::jigsaw_start_too_close_to_world_height_limits(
            -64,
            384,
            super::super::DimensionPaddingModel::ZERO,
            super::super::StructureBoundingBoxModel {
                min_y: -10_000,
                max_y: 10_000,
                ..fits_at_padded_edges
            },
        )
    );
}

#[test]
fn jigsaw_initial_expansion_bounds_match_java_aabb_and_padding_clamps() {
    assert_eq!(
        super::super::jigsaw_initial_expansion_bounds(
            100,
            70,
            -30,
            super::super::JigsawMaxDistanceModel {
                horizontal: 80,
                vertical: 96,
            },
            -64,
            384,
            super::super::DimensionPaddingModel { bottom: 4, top: 8 },
        ),
        super::super::JigsawExpansionBoundsModel {
            min_x: 20,
            min_y: -26,
            min_z: -110,
            max_x_exclusive: 181,
            max_y_exclusive: 167,
            max_z_exclusive: 51,
        }
    );

    assert_eq!(
        super::super::jigsaw_initial_expansion_bounds(
            0,
            300,
            0,
            super::super::JigsawMaxDistanceModel {
                horizontal: 1,
                vertical: 80,
            },
            -64,
            384,
            super::super::DimensionPaddingModel { bottom: 0, top: 16 },
        ),
        super::super::JigsawExpansionBoundsModel {
            min_x: -1,
            min_y: 220,
            min_z: -1,
            max_x_exclusive: 2,
            max_y_exclusive: 304,
            max_z_exclusive: 2,
        }
    );
}

#[test]
fn jigsaw_start_anchor_adjustment_matches_named_start_jigsaw_math() {
    assert_eq!(
        super::super::jigsaw_start_anchor_adjustment(
            super::super::BlockPos {
                x: 160,
                y: 72,
                z: -48,
            },
            super::super::BlockPos {
                x: 166,
                y: 75,
                z: -61,
            },
        ),
        super::super::JigsawStartAnchorAdjustmentModel {
            local_anchor: super::super::BlockPos { x: 6, y: 3, z: -13 },
            adjusted_position: super::super::BlockPos {
                x: 154,
                y: 69,
                z: -35,
            },
        }
    );

    let unchanged = super::super::BlockPos { x: 0, y: 64, z: 0 };
    assert_eq!(
        super::super::jigsaw_start_anchor_adjustment(unchanged, unchanged),
        super::super::JigsawStartAnchorAdjustmentModel {
            local_anchor: super::super::BlockPos { x: 0, y: 0, z: 0 },
            adjusted_position: unchanged,
        }
    );
}

#[test]
fn jigsaw_junction_serialization_and_java_equality_match_vanilla() {
    let junction = super::super::JigsawJunctionModel {
        source_x: 12,
        source_ground_y: 70,
        source_z: -4,
        delta_y: 3,
        dest_projection: super::super::JigsawProjectionModel::TerrainMatching,
    };
    let tag = junction.serialize();
    assert_eq!(
        tag,
        super::super::JigsawJunctionTagModel {
            source_x: 12,
            source_ground_y: 70,
            source_z: -4,
            delta_y: 3,
            dest_proj: "terrain_matching",
        }
    );
    assert_eq!(
        super::super::JigsawJunctionModel::deserialize(tag),
        Some(junction.clone())
    );
    assert_eq!(
        super::super::JigsawJunctionModel::deserialize(super::super::JigsawJunctionTagModel {
            source_x: 0,
            source_ground_y: 0,
            source_z: 0,
            delta_y: 0,
            dest_proj: "",
        }),
        None
    );

    let different_ground_y = super::super::JigsawJunctionModel {
        source_ground_y: 99,
        ..junction.clone()
    };
    assert!(junction.java_equals(&different_ground_y));
    assert_ne!(
        junction.java_hash_inputs(),
        different_ground_y.java_hash_inputs()
    );

    let different_projection = super::super::JigsawJunctionModel {
        dest_projection: super::super::JigsawProjectionModel::Rigid,
        ..junction.clone()
    };
    assert!(!junction.java_equals(&different_projection));
}

#[test]
fn jigsaw_pool_alias_lookup_resolves_direct_random_and_group_bindings_like_vanilla() {
    let direct = super::super::JigsawPoolAliasBindingModel::Direct {
        alias: "minecraft:village/common/well",
        target: "minecraft:village/plains/well",
    };
    assert_eq!(direct.codec_id(), "minecraft:direct");
    assert_eq!(direct.all_targets(), vec!["minecraft:village/plains/well"]);

    let group = super::super::JigsawPoolAliasBindingModel::RandomGroup {
        groups: vec![super::super::JigsawPoolAliasWeightedGroup {
            weight: 1,
            bindings: vec![
                super::super::JigsawPoolAliasBindingModel::Random {
                    alias: "minecraft:village/common/houses",
                    targets: vec![super::super::JigsawPoolAliasWeightedTarget {
                        target: "minecraft:village/savanna/houses",
                        weight: 1,
                    }],
                },
                super::super::JigsawPoolAliasBindingModel::Direct {
                    alias: "minecraft:village/common/terminators",
                    target: "minecraft:village/savanna/terminators",
                },
            ],
        }],
    };
    assert_eq!(group.codec_id(), "minecraft:random_group");
    assert_eq!(
        group.all_targets(),
        vec![
            "minecraft:village/savanna/houses",
            "minecraft:village/savanna/terminators",
        ]
    );

    let lookup =
        super::super::JigsawPoolAliasLookupModel::create(&[direct, group], (16, 72, -32), 12345);
    assert_eq!(
        lookup.lookup("minecraft:village/common/well"),
        "minecraft:village/plains/well"
    );
    assert_eq!(
        lookup.lookup("minecraft:village/common/houses"),
        "minecraft:village/savanna/houses"
    );
    assert_eq!(
        lookup.lookup("minecraft:village/common/terminators"),
        "minecraft:village/savanna/terminators"
    );
    assert_eq!(
        lookup.lookup("minecraft:village/plains/streets"),
        "minecraft:village/plains/streets"
    );

    let empty = super::super::JigsawPoolAliasLookupModel::create(&[], (0, 0, 0), 0);
    assert_eq!(empty.lookup("minecraft:empty"), "minecraft:empty");
}

#[test]
fn jigsaw_pool_element_surfaces_match_vanilla_registry_and_pool_rules() {
    assert_jigsaw_pool_element_registry_order_matches_vanilla();
    let single = jigsaw_single_pool_element_fixture();
    assert_single_jigsaw_pool_element_processors_match_vanilla(&single);
    let legacy = jigsaw_legacy_pool_element_fixture();
    assert_legacy_jigsaw_pool_element_processors_match_vanilla(&legacy);
    let feature = jigsaw_feature_pool_element_fixture();
    assert_feature_jigsaw_pool_element_defaults_match_vanilla(&feature);
    let empty = super::super::JigsawPoolElementModel::empty();
    assert_empty_jigsaw_pool_element_contract_matches_vanilla(&empty);
    assert_list_jigsaw_pool_element_projection_matches_vanilla(single, feature.clone());
    assert_jigsaw_template_pool_weights_match_vanilla(legacy, empty, feature);
}

fn assert_jigsaw_pool_element_registry_order_matches_vanilla() {
    assert_eq!(
        super::super::JigsawPoolElementTypeModel::REGISTRY_ORDER
            .map(|element_type| element_type.id()),
        [
            "minecraft:single_pool_element",
            "minecraft:list_pool_element",
            "minecraft:feature_pool_element",
            "minecraft:empty_pool_element",
            "minecraft:legacy_single_pool_element",
        ]
    );
}

fn jigsaw_single_pool_element_fixture() -> super::super::JigsawPoolElementModel {
    super::super::JigsawPoolElementModel::single(
        "minecraft:village/plains/houses/plains_small_house_1",
        &["minecraft:mossify_10_percent"],
        super::super::JigsawProjectionModel::TerrainMatching,
        Some(super::super::LiquidSettingsModel::IgnoreWaterlogging),
    )
}

fn assert_single_jigsaw_pool_element_processors_match_vanilla(
    single: &super::super::JigsawPoolElementModel,
) {
    assert_eq!(single.ground_level_delta(), 1);
    assert_eq!(
        single.placement_processors(false),
        vec![
            "minecraft:structure_block",
            "minecraft:jigsaw_replacement",
            "minecraft:mossify_10_percent",
            "minecraft:gravity",
        ]
    );
    assert_eq!(
        single.placement_processors(true),
        vec![
            "minecraft:structure_block",
            "minecraft:mossify_10_percent",
            "minecraft:gravity",
        ]
    );
}

fn jigsaw_legacy_pool_element_fixture() -> super::super::JigsawPoolElementModel {
    super::super::JigsawPoolElementModel::legacy_single(
        "minecraft:village/plains/town_centers/plains_fountain_01",
        &[],
        super::super::JigsawProjectionModel::Rigid,
        None,
    )
}

fn assert_legacy_jigsaw_pool_element_processors_match_vanilla(
    legacy: &super::super::JigsawPoolElementModel,
) {
    assert_eq!(
        legacy.placement_processors(false),
        vec![
            "minecraft:structure_and_air",
            "minecraft:jigsaw_replacement"
        ]
    );
}

fn jigsaw_feature_pool_element_fixture() -> super::super::JigsawPoolElementModel {
    super::super::JigsawPoolElementModel::feature(
        "minecraft:patch_grass",
        super::super::JigsawProjectionModel::TerrainMatching,
    )
}

fn assert_feature_jigsaw_pool_element_defaults_match_vanilla(
    feature: &super::super::JigsawPoolElementModel,
) {
    assert_eq!(
        feature.default_feature_jigsaw(),
        Some(super::super::DefaultFeatureJigsawModel {
            name: "minecraft:bottom",
            final_state: "minecraft:air",
            pool: "minecraft:empty",
            target: "minecraft:empty",
            joint: "rollable",
            orientation: "down_south",
        })
    );
}

fn assert_empty_jigsaw_pool_element_contract_matches_vanilla(
    empty: &super::super::JigsawPoolElementModel,
) {
    assert_eq!(empty.empty_size(), Some((0, 0, 0)));
    assert_eq!(empty.empty_place_result(), Some(true));
}

fn assert_list_jigsaw_pool_element_projection_matches_vanilla(
    single: super::super::JigsawPoolElementModel,
    feature: super::super::JigsawPoolElementModel,
) {
    let list = match super::super::JigsawPoolElementModel::list(
        vec![single.clone(), feature.clone()],
        super::super::JigsawProjectionModel::Rigid,
    ) {
        Ok(list) => list,
        Err(error) => panic!("non-empty list element should be valid: {error}"),
    };
    assert!(list
        .children
        .iter()
        .all(|child| child.projection == super::super::JigsawProjectionModel::Rigid));
    assert_eq!(
        super::super::JigsawPoolElementModel::list(
            Vec::new(),
            super::super::JigsawProjectionModel::Rigid
        ),
        Err("Elements are empty".to_string())
    );
}

fn assert_jigsaw_template_pool_weights_match_vanilla(
    legacy: super::super::JigsawPoolElementModel,
    empty: super::super::JigsawPoolElementModel,
    feature: super::super::JigsawPoolElementModel,
) {
    let pool = match super::super::JigsawTemplatePoolModel::new(
        "minecraft:empty",
        vec![
            super::super::JigsawTemplatePoolElementEntry {
                element: legacy,
                weight: 2,
            },
            super::super::JigsawTemplatePoolElementEntry {
                element: empty,
                weight: 1,
            },
        ],
    ) {
        Ok(pool) => pool,
        Err(error) => panic!("valid jigsaw template pool weights should build: {error}"),
    };
    assert_eq!(pool.size(), 3);
    assert_eq!(pool.get_weighted_template_index(0), Some(0));
    assert_eq!(pool.get_weighted_template_index(1), Some(0));
    assert_eq!(pool.get_weighted_template_index(2), Some(1));
    assert_eq!(pool.get_weighted_template_index(3), None);
    assert_eq!(
        super::super::JigsawTemplatePoolModel::new(
            "minecraft:empty",
            vec![super::super::JigsawTemplatePoolElementEntry {
                element: feature,
                weight: 151,
            }],
        ),
        Err("template pool element weight must be in 1..=150".to_string())
    );
}

#[test]
fn jigsaw_candidate_pool_iteration_order_matches_java_target_fallback_and_empty_break() {
    let target_a = super::super::JigsawPoolElementModel::single(
        "minecraft:village/plains/houses/a",
        &[],
        super::super::JigsawProjectionModel::Rigid,
        None,
    );
    let target_b = super::super::JigsawPoolElementModel::single(
        "minecraft:village/plains/houses/b",
        &[],
        super::super::JigsawProjectionModel::Rigid,
        None,
    );
    let fallback = super::super::JigsawPoolElementModel::legacy_single(
        "minecraft:village/plains/fallback",
        &[],
        super::super::JigsawProjectionModel::TerrainMatching,
        None,
    );
    let target_pool = super::super::JigsawTemplatePoolModel::new(
        "minecraft:empty",
        vec![
            super::super::JigsawTemplatePoolElementEntry {
                element: target_a.clone(),
                weight: 2,
            },
            super::super::JigsawTemplatePoolElementEntry {
                element: target_b.clone(),
                weight: 1,
            },
        ],
    )
    .unwrap();
    let fallback_pool = super::super::JigsawTemplatePoolModel::new(
        "minecraft:empty",
        vec![
            super::super::JigsawTemplatePoolElementEntry {
                element: fallback.clone(),
                weight: 1,
            },
            super::super::JigsawTemplatePoolElementEntry {
                element: super::super::JigsawPoolElementModel::empty(),
                weight: 1,
            },
            super::super::JigsawTemplatePoolElementEntry {
                element: target_b.clone(),
                weight: 1,
            },
        ],
    )
    .unwrap();

    let candidates = super::super::jigsaw_candidate_elements_in_iteration_order(
        &target_pool,
        &fallback_pool,
        1,
        3,
        &[2, 0, 1],
        &[0, 1, 2],
    );
    assert_eq!(
        candidates
            .iter()
            .map(|candidate| (candidate.source, candidate.raw_template_index))
            .collect::<Vec<_>>(),
        vec![
            (super::super::JigsawCandidatePoolSource::Target, 1),
            (super::super::JigsawCandidatePoolSource::Target, 0),
            (super::super::JigsawCandidatePoolSource::Target, 0),
            (super::super::JigsawCandidatePoolSource::Fallback, 0),
        ]
    );
    assert_eq!(candidates[0].element, target_b);
    assert_eq!(candidates[3].element, fallback);

    let max_depth_candidates = super::super::jigsaw_candidate_elements_in_iteration_order(
        &target_pool,
        &fallback_pool,
        3,
        3,
        &[2, 0, 1],
        &[0, 1, 2],
    );
    assert_eq!(
        max_depth_candidates
            .iter()
            .map(|candidate| candidate.source)
            .collect::<Vec<_>>(),
        vec![super::super::JigsawCandidatePoolSource::Fallback]
    );
}

#[test]
fn jigsaw_expansion_hack_target_size_matches_java_pool_max_size_lookup() {
    let hack_box = super::super::StructureBoundingBoxModel {
        min_x: 0,
        min_y: 0,
        min_z: 0,
        max_x: 15,
        max_y: 15,
        max_z: 15,
    };
    let jigsaws = vec![
        super::super::JigsawLocalConnectorModel {
            connector: super::super::JigsawConnectorModel {
                name: "minecraft:street",
                target: "minecraft:street",
                pool: "minecraft:village/common/houses",
                front: super::super::JigsawDirectionModel::East,
                top: super::super::JigsawDirectionModel::Up,
                joint: super::super::JigsawJointTypeModel::Aligned,
                placement_priority: 0,
                selection_priority: 0,
            },
            local_pos: super::super::BlockPos { x: 14, y: 6, z: 7 },
        },
        super::super::JigsawLocalConnectorModel {
            connector: super::super::JigsawConnectorModel {
                name: "minecraft:street",
                target: "minecraft:street",
                pool: "minecraft:village/plains/ignored_outside",
                front: super::super::JigsawDirectionModel::East,
                top: super::super::JigsawDirectionModel::Up,
                joint: super::super::JigsawJointTypeModel::Aligned,
                placement_priority: 0,
                selection_priority: 0,
            },
            local_pos: super::super::BlockPos { x: 15, y: 6, z: 7 },
        },
        super::super::JigsawLocalConnectorModel {
            connector: super::super::JigsawConnectorModel {
                name: "minecraft:street",
                target: "minecraft:street",
                pool: "minecraft:village/plains/missing",
                front: super::super::JigsawDirectionModel::North,
                top: super::super::JigsawDirectionModel::Up,
                joint: super::super::JigsawJointTypeModel::Aligned,
                placement_priority: 0,
                selection_priority: 0,
            },
            local_pos: super::super::BlockPos { x: 4, y: 6, z: 4 },
        },
    ];
    let pools = [
        super::super::JigsawPoolSizeModel {
            name: "minecraft:village/plains/houses",
            fallback: Some("minecraft:village/plains/terminators"),
            max_size: 4,
        },
        super::super::JigsawPoolSizeModel {
            name: "minecraft:village/plains/terminators",
            fallback: None,
            max_size: 9,
        },
        super::super::JigsawPoolSizeModel {
            name: "minecraft:village/plains/ignored_outside",
            fallback: None,
            max_size: 99,
        },
    ];
    let lookup = super::super::JigsawPoolAliasLookupModel {
        mappings: BTreeMap::from([(
            "minecraft:village/common/houses",
            "minecraft:village/plains/houses",
        )]),
    };

    assert_eq!(
        super::super::jigsaw_expansion_hack_target_size(true, hack_box, &jigsaws, &pools, &lookup),
        9
    );
    assert_eq!(
        super::super::jigsaw_expansion_hack_target_size(false, hack_box, &jigsaws, &pools, &lookup),
        0
    );
    assert_eq!(
        super::super::jigsaw_expansion_hack_target_size(
            true,
            super::super::StructureBoundingBoxModel {
                max_y: 16,
                ..hack_box
            },
            &jigsaws,
            &pools,
            &lookup,
        ),
        0
    );
}

#[test]
fn jigsaw_pool_availability_decision_matches_java_warning_and_skip_rules() {
    assert_eq!(
        super::super::jigsaw_pool_availability_decision(false, 0, "minecraft:empty", 0),
        super::super::JigsawPoolAvailabilityDecisionModel {
            can_place_children: false,
            warning: Some(super::super::JigsawPoolAvailabilityWarning::EmptyOrNonExistentTarget),
        }
    );
    assert_eq!(
        super::super::jigsaw_pool_availability_decision(true, 0, "minecraft:empty", 0),
        super::super::JigsawPoolAvailabilityDecisionModel {
            can_place_children: false,
            warning: Some(super::super::JigsawPoolAvailabilityWarning::EmptyOrNonExistentTarget),
        }
    );
    assert_eq!(
        super::super::jigsaw_pool_availability_decision(
            true,
            3,
            "minecraft:village/bad_fallback",
            0
        ),
        super::super::JigsawPoolAvailabilityDecisionModel {
            can_place_children: false,
            warning: Some(super::super::JigsawPoolAvailabilityWarning::EmptyOrNonExistentFallback),
        }
    );
    assert_eq!(
        super::super::jigsaw_pool_availability_decision(true, 3, "minecraft:empty", 0),
        super::super::JigsawPoolAvailabilityDecisionModel {
            can_place_children: true,
            warning: None,
        }
    );
    assert_eq!(
        super::super::jigsaw_pool_availability_decision(
            true,
            3,
            "minecraft:village/plains/terminators",
            2,
        ),
        super::super::JigsawPoolAvailabilityDecisionModel {
            can_place_children: true,
            warning: None,
        }
    );
}

#[test]
fn jigsaw_structure_start_pools_match_java_bootstrap_start_keys() {
    assert_eq!(
        super::super::JIGSAW_STRUCTURE_START_POOLS
            .iter()
            .map(|entry| entry.pool)
            .collect::<Vec<_>>(),
        vec![
            "minecraft:village/plains/town_centers",
            "minecraft:village/desert/town_centers",
            "minecraft:village/savanna/town_centers",
            "minecraft:village/snowy/town_centers",
            "minecraft:village/taiga/town_centers",
            "minecraft:pillager_outpost/base_plates",
            "minecraft:bastion/starts",
            "minecraft:ancient_city/city_center",
            "minecraft:trail_ruins/tower",
            "minecraft:trial_chambers/chamber/end",
        ]
    );
    assert!(super::super::JIGSAW_STRUCTURE_START_POOLS
        .iter()
        .all(|entry| entry.pool.starts_with("minecraft:")));
    assert_eq!(
        super::super::JIGSAW_STRUCTURE_START_POOLS
            .iter()
            .filter(|entry| entry.structure_family.starts_with("village/"))
            .count(),
        5
    );
    assert!(super::super::JIGSAW_STRUCTURE_START_POOLS
        .iter()
        .any(|entry| {
            entry.source_file == "TrialChambersStructurePools.java"
                && entry.pool == "minecraft:trial_chambers/chamber/end"
        }));
}
