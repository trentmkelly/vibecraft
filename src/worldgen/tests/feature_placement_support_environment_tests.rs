use super::super::*;

fn pos(x: i32, y: i32, z: i32) -> BlockPos {
    BlockPos { x, y, z }
}

pub(super) fn assert_aquatic_placement_helpers() {
    assert_eq!(
        super::super::aquatic_feature_offset(pos(20, 60, 30), (7, 3), (1, 6)),
        (24, 25)
    );
    assert_seagrass_and_pickle_helpers();
    assert_kelp_placement_helpers();
}

fn assert_seagrass_and_pickle_helpers() {
    assert_eq!(
        super::super::seagrass_placement_plan(
            pos(1, 62, 1),
            "minecraft:water",
            "minecraft:water",
            true,
            0.7,
            0.1,
        ),
        vec![
            aquatic_block(pos(1, 62, 1), "minecraft:tall_seagrass"),
            aquatic_block(pos(1, 63, 1), "minecraft:tall_seagrass[half=upper]",),
        ]
    );
    assert_eq!(
        super::super::seagrass_placement_plan(
            pos(1, 62, 1),
            "minecraft:water",
            "minecraft:air",
            true,
            0.7,
            0.1,
        ),
        Vec::new()
    );
    assert_eq!(
        super::super::sea_pickle_placement_plan(pos(2, 61, 2), "minecraft:water", true, 2,),
        Some(aquatic_block(
            pos(2, 61, 2),
            "minecraft:sea_pickle[pickles=3]",
        ))
    );
}

fn assert_kelp_placement_helpers() {
    assert_eq!(
        super::super::kelp_placement_plan(
            pos(3, 50, 3),
            &[true, true, true, true],
            &[true, true, true],
            1,
            &[2],
            false,
        ),
        vec![
            aquatic_block(pos(3, 50, 3), "minecraft:kelp_plant"),
            aquatic_block(pos(3, 51, 3), "minecraft:kelp_plant"),
            aquatic_block(pos(3, 52, 3), "minecraft:kelp[age=22]"),
        ]
    );
    assert_eq!(
        super::super::kelp_placement_plan(
            pos(3, 50, 3),
            &[true, true, false],
            &[true, true],
            5,
            &[0],
            false,
        ),
        vec![aquatic_block(pos(3, 50, 3), "minecraft:kelp[age=20]",)]
    );
}

fn aquatic_block(pos: BlockPos, state: &'static str) -> super::super::AquaticPlacementBlock {
    super::super::AquaticPlacementBlock { pos, state }
}

pub(super) fn assert_coral_placement_helpers() {
    assert_eq!(
        super::super::coral_block_placement_plan(super::super::CoralBlockPlacementInput {
            pos: pos(4, 55, 4),
            current_block: "minecraft:water",
            above_block: "minecraft:water",
            coral_state: "minecraft:brain_coral_block",
            coral_roll: 0.9,
            sea_pickle_roll: 0.01,
            pickle_count_roll: 1,
            wall_fan_rolls: &[(super::super::HorizontalDirection::East, 0.1, true)],
        }),
        vec![
            aquatic_block(pos(4, 55, 4), "minecraft:brain_coral_block"),
            aquatic_block(pos(4, 56, 4), "minecraft:sea_pickle[pickles=2]",),
            aquatic_block(pos(5, 55, 4), "minecraft:tube_coral_wall_fan[facing=east]",),
        ]
    );
    assert!(
        super::super::coral_block_placement_plan(super::super::CoralBlockPlacementInput {
            pos: pos(4, 55, 4),
            current_block: "minecraft:stone",
            above_block: "minecraft:water",
            coral_state: "minecraft:brain_coral_block",
            coral_roll: 0.0,
            sea_pickle_roll: 0.0,
            pickle_count_roll: 0,
            wall_fan_rolls: &[],
        })
        .is_empty()
    );
    assert_coral_shape_helpers();
}

fn assert_coral_shape_helpers() {
    let coral_tree = super::super::coral_tree_positions(
        pos(0, 60, 0),
        1,
        &[
            super::super::HorizontalDirection::North,
            super::super::HorizontalDirection::East,
        ],
        &[0, 1],
        &[1.0; 10],
    );
    assert!(coral_tree.contains(&pos(0, 60, 0)));
    assert!(coral_tree.contains(&pos(0, 62, -1)));
    assert!(coral_tree.contains(&pos(1, 62, 0)));
    let coral_mushroom =
        super::super::coral_mushroom_positions(pos(0, 60, 0), 0, 0, 0, 0, &[1.0; 128]);
    assert!(coral_mushroom.contains(&pos(1, 59, 1)));
    assert!(!coral_mushroom.contains(&pos(0, 59, 0)));
    let coral_claw = super::super::coral_claw_positions(
        pos(0, 60, 0),
        super::super::HorizontalDirection::North,
        &[
            super::super::HorizontalDirection::North,
            super::super::HorizontalDirection::East,
        ],
        &[0, 0],
        &[0, 0],
        &[1.0; 10],
    );
    assert!(coral_claw.contains(&pos(0, 60, 0)));
    assert!(coral_claw.contains(&pos(0, 60, -1)));
    assert!(coral_claw.contains(&pos(1, 61, 0)));
}

pub(super) fn assert_vegetation_patch_helpers() {
    let vegetation_config = sample_vegetation_patch_config();
    assert_eq!(super::super::vegetation_patch_radius(1, 2, 1), 3);
    assert!(!super::super::vegetation_patch_should_try_column(
        3, 3, 3, 3, 1.0, 0.0
    ));
    assert!(!super::super::vegetation_patch_should_try_column(
        3, 0, 3, 3, 0.25, 0.5
    ));
    assert!(super::super::vegetation_patch_should_try_column(
        3, 0, 3, 3, 0.25, 0.25
    ));
    assert_eq!(super::super::vegetation_patch_depth(1, 2, 0, 0.5, 0.25), 2);
    assert_vegetation_ground_placement(&vegetation_config);
    assert_vegetation_patch_plan(&vegetation_config);
}

fn sample_vegetation_patch_config() -> super::super::VegetationPatchConfigurationModel {
    super::super::VegetationPatchConfigurationModel {
        replaceable: &["minecraft:dirt", "minecraft:grass_block"],
        ground_state: BlockStateProviderModel::Simple("minecraft:moss_block"),
        vegetation_feature: "minecraft:patch_grass",
        surface: CaveSurface::Floor,
        depth_min: 1,
        depth_max: 2,
        extra_bottom_block_chance: 0.5,
        vertical_range: 5,
        vegetation_chance: 0.75,
        xz_radius_min: 1,
        xz_radius_max: 2,
        extra_edge_column_chance: 0.25,
    }
}

fn assert_vegetation_ground_placement(
    vegetation_config: &super::super::VegetationPatchConfigurationModel,
) {
    assert_eq!(
        super::super::vegetation_patch_place_ground(
            vegetation_config,
            pos(5, 63, 5),
            &["minecraft:dirt", "minecraft:stone"],
            3,
            0,
        ),
        Some(vec![super::super::VegetationPatchBlock {
            pos: pos(5, 63, 5),
            state: "minecraft:moss_block",
        }])
    );
}

fn assert_vegetation_patch_plan(
    vegetation_config: &super::super::VegetationPatchConfigurationModel,
) {
    let vegetation_plan = super::super::vegetation_patch_plan(
        vegetation_config,
        &[super::super::VegetationPatchGroundColumn {
            surface_pos: pos(5, 64, 5),
            ground_start: pos(5, 63, 5),
            depth: 1,
        }],
        &[&["minecraft:dirt"]],
        &[0.25],
    );
    assert_eq!(
        vegetation_plan.ground,
        vec![super::super::VegetationPatchBlock {
            pos: pos(5, 63, 5),
            state: "minecraft:moss_block",
        }]
    );
    assert_eq!(vegetation_plan.vegetation_origins, vec![pos(5, 65, 5)]);
}

pub(super) fn assert_lake_placement_helpers() {
    let lake_config = super::super::LakeConfigurationModel {
        fluid: BlockStateProviderModel::Simple("minecraft:water"),
        barrier: BlockStateProviderModel::Simple("minecraft:stone"),
    };
    let mut lake_grid = vec![false; 2048];
    lake_grid[super::super::lake_grid_index(8, 3, 8)] = true;
    assert!(super::super::lake_is_boundary(&lake_grid, 8, 4, 8));
    assert_eq!(
        super::super::lake_grid_index(8, 3, 8),
        ((8 * 16 + 8) * 8 + 3) as usize
    );
    let lake_boundary = sample_lake_boundary();
    assert!(super::super::lake_can_place(
        -64,
        70,
        &lake_grid,
        &lake_boundary,
        "minecraft:water"
    ));
    assert_invalid_lake_boundary(&lake_grid);
    assert_lake_placement_plan(&lake_config, &lake_grid, &lake_boundary);
}

fn sample_lake_boundary() -> [super::super::LakeBoundaryBlock; 2] {
    [
        super::super::LakeBoundaryBlock {
            x: 8,
            y: 4,
            z: 8,
            state: "minecraft:stone",
            solid: true,
            liquid: false,
            cannot_replace: false,
            should_freeze: true,
        },
        super::super::LakeBoundaryBlock {
            x: 8,
            y: 2,
            z: 8,
            state: "minecraft:stone",
            solid: true,
            liquid: false,
            cannot_replace: false,
            should_freeze: false,
        },
    ]
}

fn assert_invalid_lake_boundary(lake_grid: &[bool]) {
    let invalid_lake_boundary = [super::super::LakeBoundaryBlock {
        x: 8,
        y: 4,
        z: 8,
        state: "minecraft:water",
        solid: false,
        liquid: true,
        cannot_replace: false,
        should_freeze: false,
    }];
    assert!(!super::super::lake_can_place(
        -64,
        70,
        lake_grid,
        &invalid_lake_boundary,
        "minecraft:water"
    ));
}

fn assert_lake_placement_plan(
    lake_config: &super::super::LakeConfigurationModel,
    lake_grid: &[bool],
    lake_boundary: &[super::super::LakeBoundaryBlock],
) {
    let lake_plan = super::super::lake_placement_plan(
        pos(0, 60, 0),
        lake_config,
        lake_grid,
        lake_boundary,
        &[1; 2048],
        true,
    )
    .unwrap();
    assert!(lake_plan.contains(&super::super::LakePlacementBlock {
        pos: pos(8, 63, 8),
        state: "minecraft:water",
        schedule_tick: false,
        mark_above_for_post_processing: false,
    }));
    assert!(lake_plan.contains(&super::super::LakePlacementBlock {
        pos: pos(8, 64, 8),
        state: "minecraft:stone",
        schedule_tick: false,
        mark_above_for_post_processing: true,
    }));
    assert!(lake_plan.contains(&super::super::LakePlacementBlock {
        pos: pos(8, 64, 8),
        state: "minecraft:ice",
        schedule_tick: false,
        mark_above_for_post_processing: false,
    }));
}

pub(super) fn assert_fossil_placement_helpers() {
    let fossil_config = sample_fossil_config();
    assert_eq!(super::super::validate_fossil_config(&fossil_config), Ok(()));
    assert_eq!(
        super::super::validate_fossil_config(&super::super::FossilFeatureConfigurationModel {
            fossil_structures: Vec::new(),
            overlay_structures: Vec::new(),
            fossil_processors: "minecraft:fossil_rot",
            overlay_processors: "minecraft:fossil_coal",
            max_empty_corners_allowed: 4,
        }),
        Err("Fossil structure lists need at least one entry")
    );
    assert_eq!(
        super::super::fossil_rotation(3),
        super::super::StructureRotation::Counterclockwise90
    );
    assert_eq!(super::super::fossil_target_y(50, -64, 9), 26);
    assert_eq!(
        super::super::fossil_low_corner(pos(100, 40, 200), 12, 8),
        pos(94, 40, 196)
    );
    assert_valid_fossil_placement_plan(&fossil_config);
    assert_invalid_fossil_empty_corners(&fossil_config);
}

fn sample_fossil_config() -> super::super::FossilFeatureConfigurationModel {
    super::super::FossilFeatureConfigurationModel {
        fossil_structures: vec!["minecraft:fossil/spine_1", "minecraft:fossil/skull_1"],
        overlay_structures: vec![
            "minecraft:fossil/spine_1_coal",
            "minecraft:fossil/skull_1_coal",
        ],
        fossil_processors: "minecraft:fossil_rot",
        overlay_processors: "minecraft:fossil_coal",
        max_empty_corners_allowed: 4,
    }
}

fn assert_valid_fossil_placement_plan(
    fossil_config: &super::super::FossilFeatureConfigurationModel,
) {
    assert_eq!(
        super::super::fossil_placement_plan(fossil_input(fossil_config, 4)),
        Some(super::super::FossilPlacementPlan {
            fossil_structure: "minecraft:fossil/skull_1",
            overlay_structure: "minecraft:fossil/skull_1_coal",
            rotation: super::super::StructureRotation::Clockwise90,
            target_pos: pos(94, 35, 196),
            fossil_processors: "minecraft:fossil_rot",
            overlay_processors: "minecraft:fossil_coal",
        })
    );
}

fn assert_invalid_fossil_empty_corners(
    fossil_config: &super::super::FossilFeatureConfigurationModel,
) {
    assert_eq!(
        super::super::fossil_placement_plan(fossil_input(fossil_config, 5)),
        None
    );
}

fn fossil_input(
    config: &super::super::FossilFeatureConfigurationModel,
    empty_corners: i32,
) -> super::super::FossilPlacementInput<'_> {
    super::super::FossilPlacementInput {
        config,
        origin: pos(100, 40, 200),
        rotated_size_x: 12,
        rotated_size_z: 8,
        lowest_surface_y: 50,
        min_y: -64,
        rotation_roll: 1,
        fossil_index_roll: 1,
        depth_roll: 0,
        empty_corners,
    }
}
