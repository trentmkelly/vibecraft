use super::super::*;

pub(super) fn assert_disk_dripstone_and_large_dripstone_support() {
        let disk_config = super::super::DiskConfigurationModel {
            state_provider: BlockStateProviderModel::Simple("minecraft:clay"),
            target: BlockPredicate::MatchingBlocks {
                blocks: &["minecraft:dirt"],
            },
            radius: super::super::IntProviderModel::Constant(1),
            half_height: 1,
        };
        let mut disk_contexts = Vec::new();
        for y in 63..=65 {
            disk_contexts.push((
                BlockPos { x: 0, y, z: 0 },
                BlockPredicateContext {
                    min_y: -64,
                    height: 384,
                    block: "minecraft:dirt",
                    fluid: "minecraft:empty",
                    solid: true,
                    replaceable: false,
                    unobstructed: true,
                },
            ));
        }
        let disk = super::super::disk_placement_plan(
            BlockPos { x: 0, y: 64, z: 0 },
            &disk_config,
            &disk_contexts,
            &[],
        );
        assert_eq!(disk.len(), 3);
        assert_eq!(disk[0].pos, BlockPos { x: 0, y: 65, z: 0 });
        assert!(disk[0].mark_above_for_post_processing);
        assert!(!disk[1].mark_above_for_post_processing);
        assert!(disk.iter().all(|block| block.state == "minecraft:clay"));

        disk_contexts[1].1.block = "minecraft:stone";
        let disk_with_gap = super::super::disk_placement_plan(
            BlockPos { x: 0, y: 64, z: 0 },
            &disk_config,
            &disk_contexts,
            &[],
        );
        assert_eq!(disk_with_gap.len(), 2);
        assert!(disk_with_gap[0].mark_above_for_post_processing);
        assert!(disk_with_gap[1].mark_above_for_post_processing);

        let snow_plan = super::super::snow_and_freeze_placement_plan(
            BlockPos {
                x: 32,
                y: 0,
                z: -16,
            },
            &[
                super::super::SnowAndFreezeColumn {
                    x: 32,
                    z: -16,
                    motion_blocking_height: 70,
                    should_freeze: true,
                    should_snow: true,
                    below_has_snowy_property: true,
                },
                super::super::SnowAndFreezeColumn {
                    x: 33,
                    z: -16,
                    motion_blocking_height: 65,
                    should_freeze: false,
                    should_snow: true,
                    below_has_snowy_property: false,
                },
            ],
        );
        assert_eq!(
            snow_plan,
            vec![
                super::super::SnowAndFreezePlacement {
                    pos: BlockPos {
                        x: 32,
                        y: 69,
                        z: -16
                    },
                    state: "minecraft:ice",
                },
                super::super::SnowAndFreezePlacement {
                    pos: BlockPos {
                        x: 32,
                        y: 70,
                        z: -16
                    },
                    state: "minecraft:snow",
                },
                super::super::SnowAndFreezePlacement {
                    pos: BlockPos {
                        x: 32,
                        y: 69,
                        z: -16
                    },
                    state: "minecraft:snowy=true",
                },
                super::super::SnowAndFreezePlacement {
                    pos: BlockPos {
                        x: 33,
                        y: 65,
                        z: -16
                    },
                    state: "minecraft:snow",
                },
            ]
        );

        let magma_config = super::super::UnderwaterMagmaConfigurationModel {
            floor_search_range: 12,
            placement_radius_around_floor: 1,
            placement_probability_per_valid_position: 0.5,
        };
        let valid_magma = super::super::UnderwaterMagmaCandidate {
            pos: BlockPos { x: 0, y: 62, z: 0 },
            block: "minecraft:stone",
            below_visible_from_above: false,
            horizontal_visible_from_outside: false,
        };
        assert!(super::super::underwater_magma_is_valid_placement(&valid_magma));
        assert!(!super::super::underwater_magma_is_valid_placement(
            &super::super::UnderwaterMagmaCandidate {
                block: "minecraft:water",
                ..valid_magma
            }
        ));
        assert!(!super::super::underwater_magma_is_valid_placement(
            &super::super::UnderwaterMagmaCandidate {
                below_visible_from_above: true,
                ..valid_magma
            }
        ));
        assert!(!super::super::underwater_magma_is_valid_placement(
            &super::super::UnderwaterMagmaCandidate {
                horizontal_visible_from_outside: true,
                ..valid_magma
            }
        ));
        assert_eq!(
            super::super::underwater_magma_placement_plan(
                BlockPos { x: 0, y: 70, z: 0 },
                Some(62),
                magma_config,
                &[valid_magma],
                &[0.0; 27],
            ),
            vec![BlockPos { x: 0, y: 62, z: 0 }]
        );
        assert!(super::super::underwater_magma_placement_plan(
            BlockPos { x: 0, y: 70, z: 0 },
            None,
            magma_config,
            &[valid_magma],
            &[0.0; 27],
        )
        .is_empty());
        let dripstone_config = super::super::DripstoneClusterSampledConfig {
            floor_to_ceiling_search_range: 12,
            height: 6,
            x_radius: 3,
            z_radius: 3,
            max_stalagmite_stalactite_height_diff: 1,
            height_deviation: 2,
            dripstone_block_layer_thickness: 2,
            density: 1.0,
            wetness: 0.5,
            chance_of_dripstone_column_at_max_distance_from_center: 0.2,
            max_distance_from_edge_affecting_chance_of_dripstone_column: 3,
            max_distance_from_center_affecting_height_bias: 4,
        };
        assert_eq!(
            super::super::validate_dripstone_cluster_sampled_config(dripstone_config),
            Ok(dripstone_config)
        );
        assert!(
            (super::super::dripstone_cluster_chance_of_column(3, 3, 3, 0, dripstone_config) - 0.2).abs()
                < 0.000001
        );
        assert_eq!(
            super::super::dripstone_cluster_chance_of_column(3, 3, 0, 0, dripstone_config),
            1.0
        );
        assert_eq!(
            super::super::dripstone_cluster_height_for_column(1, 1, 0.5, 6, dripstone_config, 0.75, 5.0,),
            0
        );
        assert_eq!(
            super::super::pointed_dripstone_column(
                BlockPos { x: 0, y: 70, z: 0 },
                super::super::PointedDripstoneDirection::Down,
                4,
                true,
            ),
            vec![
                super::super::PointedDripstoneBlockModel {
                    pos: BlockPos { x: 0, y: 70, z: 0 },
                    direction: super::super::PointedDripstoneDirection::Down,
                    thickness: super::super::PointedDripstoneThickness::Base,
                },
                super::super::PointedDripstoneBlockModel {
                    pos: BlockPos { x: 0, y: 69, z: 0 },
                    direction: super::super::PointedDripstoneDirection::Down,
                    thickness: super::super::PointedDripstoneThickness::Middle,
                },
                super::super::PointedDripstoneBlockModel {
                    pos: BlockPos { x: 0, y: 68, z: 0 },
                    direction: super::super::PointedDripstoneDirection::Down,
                    thickness: super::super::PointedDripstoneThickness::Frustum,
                },
                super::super::PointedDripstoneBlockModel {
                    pos: BlockPos { x: 0, y: 67, z: 0 },
                    direction: super::super::PointedDripstoneDirection::Down,
                    thickness: super::super::PointedDripstoneThickness::TipMerge,
                },
            ]
        );
        let dripstone_plan = super::super::dripstone_cluster_column_plan(
            BlockPos {
                x: 10,
                y: 64,
                z: 10,
            },
            dripstone_config,
            super::super::DripstoneClusterColumnInput {
                dx: 0,
                dz: 0,
                ceiling_y: Some(74),
                floor_y: Some(68),
                floor_pool_supported: false,
                ceiling_is_lava: false,
                floor_is_lava: false,
            },
            super::super::DripstoneClusterColumnRolls {
                water_roll: 0.75,
                stalactite_roll: 0.0,
                stalactite_density_roll: 0.0,
                stalactite_biased_height: 4.0,
                stalagmite_roll: 0.0,
                stalagmite_density_roll: 0.0,
                stalagmite_biased_height: 3.0,
                stalagmite_height_diff_roll: 1,
                overlap_split_roll: 0,
                merge_tips_roll: true,
            },
        );
        assert_eq!(
            dripstone_plan.ceiling_dripstone_blocks,
            vec![
                BlockPos {
                    x: 10,
                    y: 74,
                    z: 10
                },
                BlockPos {
                    x: 10,
                    y: 75,
                    z: 10
                },
            ]
        );
        assert_eq!(
            dripstone_plan.floor_dripstone_blocks,
            vec![
                BlockPos {
                    x: 10,
                    y: 68,
                    z: 10
                },
                BlockPos {
                    x: 10,
                    y: 67,
                    z: 10
                },
            ]
        );
        assert!(!dripstone_plan.merge_tips);
        assert_eq!(dripstone_plan.stalactite.len(), 4);
        assert_eq!(dripstone_plan.stalagmite.len(), 1);
        let pointed_config = super::super::PointedDripstoneConfigurationModel {
            chance_of_taller_dripstone: 0.5,
            chance_of_directional_spread: 0.7,
            chance_of_spread_radius2: 0.5,
            chance_of_spread_radius3: 0.5,
        };
        assert_eq!(
            super::super::validate_pointed_dripstone_configuration(pointed_config),
            Ok(pointed_config)
        );
        assert_eq!(
            super::super::pointed_dripstone_tip_direction(true, true, true),
            Some(super::super::PointedDripstoneDirection::Down)
        );
        assert_eq!(
            super::super::pointed_dripstone_tip_direction(false, true, true),
            Some(super::super::PointedDripstoneDirection::Up)
        );
        assert_eq!(
            super::super::pointed_dripstone_tip_direction(false, false, true),
            None
        );
        let pointed_plan = super::super::pointed_dripstone_feature_plan(
            super::super::PointedDripstoneFeatureInput {
                origin: BlockPos { x: 4, y: 70, z: 4 },
                config: pointed_config,
                can_place_above: true,
                can_place_below: false,
                choose_down_when_both: false,
                taller_roll: 0.25,
                next_position_empty_or_water: true,
                spread_rolls: &[
                    super::super::PointedDripstoneSpreadRoll {
                        direction: HorizontalDirection::East,
                        direction_roll: 0.2,
                        radius2_roll: 0.2,
                        radius2_direction: HorizontalDirection::South,
                        radius3_roll: 0.2,
                        radius3_direction: HorizontalDirection::West,
                    },
                    super::super::PointedDripstoneSpreadRoll {
                        direction: HorizontalDirection::North,
                        direction_roll: 0.9,
                        radius2_roll: 0.0,
                        radius2_direction: HorizontalDirection::North,
                        radius3_roll: 0.0,
                        radius3_direction: HorizontalDirection::North,
                    },
                ],
            },
        )
        .unwrap();
        assert_eq!(
            pointed_plan.dripstone_blocks,
            vec![
                BlockPos { x: 4, y: 71, z: 4 },
                BlockPos { x: 5, y: 71, z: 4 },
                BlockPos { x: 5, y: 71, z: 5 },
                BlockPos { x: 4, y: 71, z: 5 },
            ]
        );
        assert_eq!(
            pointed_plan.pointed_blocks,
            vec![
                super::super::PointedDripstoneBlockModel {
                    pos: BlockPos { x: 4, y: 70, z: 4 },
                    direction: super::super::PointedDripstoneDirection::Down,
                    thickness: super::super::PointedDripstoneThickness::Frustum,
                },
                super::super::PointedDripstoneBlockModel {
                    pos: BlockPos { x: 4, y: 69, z: 4 },
                    direction: super::super::PointedDripstoneDirection::Down,
                    thickness: super::super::PointedDripstoneThickness::Tip,
                },
            ]
        );
        let large_dripstone_config = super::super::LargeDripstoneSampledConfig {
            floor_to_ceiling_search_range: 30,
            column_radius_min: 2,
            column_radius_max: 6,
            height_scale: 2.0,
            max_column_radius_to_cave_height_ratio: 1.0,
            stalactite_bluntness: 1.0,
            stalagmite_bluntness: 1.5,
            wind_speed: 0.5,
            wind_direction_radians: 0.0,
            min_radius_for_wind: 2,
            min_bluntness_for_wind: 1.0,
        };
        assert_eq!(
            super::super::validate_large_dripstone_sampled_config(large_dripstone_config),
            Ok(large_dripstone_config)
        );
        assert_eq!(
            super::super::large_dripstone_selected_radius(4, large_dripstone_config, 99),
            Some(2)
        );
        assert_eq!(
            super::super::large_dripstone_selected_radius(3, large_dripstone_config, 0),
            None
        );
        assert_eq!(
            super::super::large_dripstone_wind_offset(
                BlockPos { x: 4, y: 62, z: 4 },
                64,
                large_dripstone_config,
            ),
            BlockPos { x: 5, y: 62, z: 4 }
        );
        assert!(
            super::super::large_dripstone_height_at_radius(0.0, 5, 2.0, 1.0)
                > super::super::large_dripstone_height_at_radius(5.0, 5, 2.0, 1.0)
        );
        let large_dripstone_plan = super::super::large_dripstone_placement_plan(
            BlockPos { x: 4, y: 64, z: 4 },
            60,
            65,
            large_dripstone_config,
            0,
            &[1.0; 13],
            &[1.0; 13],
        )
        .unwrap();
        assert_eq!(
            large_dripstone_plan.stalactite.root,
            BlockPos { x: 4, y: 64, z: 4 }
        );
        assert_eq!(
            large_dripstone_plan.stalagmite.root,
            BlockPos { x: 4, y: 61, z: 4 }
        );
        assert!(large_dripstone_plan.wind_enabled);
        assert!(large_dripstone_plan
            .stalactite_blocks
            .iter()
            .any(|block| block.pos.x > 4 && !block.pointing_up));
        let full_large_dripstone_blocks = super::super::large_dripstone_blocks(
            super::super::LargeDripstoneModel {
                root: BlockPos { x: 0, y: 70, z: 0 },
                pointing_up: false,
                radius: 2,
            },
            2.0,
            1.0,
            70,
            None,
            &[1.0; 13],
            &[1.0; 13],
        );
        let shrunken_large_dripstone_blocks = super::super::large_dripstone_blocks(
            super::super::LargeDripstoneModel {
                root: BlockPos { x: 0, y: 70, z: 0 },
                pointing_up: false,
                radius: 2,
            },
            2.0,
            1.0,
            70,
            None,
            &[0.0; 13],
            &[0.0; 13],
        );
        assert!(shrunken_large_dripstone_blocks.len() < full_large_dripstone_blocks.len());
        assert_eq!(
            super::super::feature_size_type("two_layers_feature_size"),
            Some("minecraft:two_layers_feature_size")
        );
        assert_eq!(
            super::super::feature_size_type("minecraft:three_layers_feature_size"),
            Some("minecraft:three_layers_feature_size")
        );
        assert_eq!(super::super::feature_size_type("missing"), None);

}
