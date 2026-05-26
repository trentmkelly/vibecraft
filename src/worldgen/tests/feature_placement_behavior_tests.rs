use super::*;

#[test]
fn feature_placement_behavior_models_cover_required_families() {
    assert_eq!(
        FEATURE_BEHAVIOR_MODELS
            .iter()
            .map(|model| model.feature_type)
            .collect::<Vec<_>>(),
        vec![
            "minecraft:tree",
            "minecraft:vegetation_patch",
            "minecraft:spring_feature",
            "minecraft:ore",
            "minecraft:scattered_ore",
            "minecraft:disk",
            "minecraft:lake",
            "minecraft:geode",
            "minecraft:fossil",
            "minecraft:monster_room",
        ]
    );
}

#[test]
fn spring_feature_placement_rules_match_vanilla() {
    assert!(super::super::spring_feature_can_place(
        super::super::SpringCanPlaceInput {
            valid_above: true,
            requires_block_below: true,
            valid_below: true,
            current_is_air_or_valid: true,
            adjacent_rock_count: 4,
            adjacent_hole_count: 1,
            required_rock_count: 4,
            required_hole_count: 1,
        }
    ));
    assert!(!super::super::spring_feature_can_place(
        super::super::SpringCanPlaceInput {
            valid_above: true,
            requires_block_below: true,
            valid_below: false,
            current_is_air_or_valid: true,
            adjacent_rock_count: 4,
            adjacent_hole_count: 1,
            required_rock_count: 4,
            required_hole_count: 1,
        }
    ));
    assert!(!super::super::spring_feature_can_place(
        super::super::SpringCanPlaceInput {
            valid_above: true,
            requires_block_below: false,
            valid_below: false,
            current_is_air_or_valid: true,
            adjacent_rock_count: 3,
            adjacent_hole_count: 1,
            required_rock_count: 4,
            required_hole_count: 1,
        }
    ));

    let spring_config = super::super::SpringConfigurationModel {
        state: "minecraft:water",
        requires_block_below: true,
        rock_count: 4,
        hole_count: 1,
        valid_blocks: &["minecraft:stone", "minecraft:dirt"],
    };
    assert_eq!(
        super::super::spring_placement_plan(
            &spring_config,
            super::super::SpringPlacementContext {
                origin: BlockPos { x: 4, y: 32, z: 4 },
                above_block: "minecraft:stone",
                below_block: "minecraft:stone",
                current_block: "minecraft:air",
                west_block: "minecraft:stone",
                east_block: "minecraft:stone",
                north_block: "minecraft:air",
                south_block: "minecraft:dirt",
            },
        ),
        Some(super::super::SpringPlacementPlan {
            pos: BlockPos { x: 4, y: 32, z: 4 },
            state: "minecraft:water",
            schedule_tick: true,
        })
    );
    assert_eq!(
        super::super::spring_placement_plan(
            &spring_config,
            super::super::SpringPlacementContext {
                origin: BlockPos { x: 4, y: 32, z: 4 },
                above_block: "minecraft:stone",
                below_block: "minecraft:air",
                current_block: "minecraft:air",
                west_block: "minecraft:stone",
                east_block: "minecraft:stone",
                north_block: "minecraft:air",
                south_block: "minecraft:dirt",
            },
        ),
        None
    );
}

#[test]
fn monster_room_opening_rules_match_vanilla() {
    assert_eq!(MONSTER_ROOM_BOUNDS.min_y, -1);
    assert_eq!(MONSTER_ROOM_BOUNDS.max_y, 4);
    assert!(!super::super::monster_room_opening_count_is_valid(0));
    assert!(super::super::monster_room_opening_count_is_valid(1));
    assert!(super::super::monster_room_opening_count_is_valid(5));
    assert!(!super::super::monster_room_opening_count_is_valid(6));

    let room_radii = expected_room_radii();
    assert_eq!(
        super::super::monster_room_bounds_for_radius(room_radii.x_radius),
        (-3, 3)
    );
    let probes = [
        super::super::MonsterRoomProbe {
            dx: 0,
            dy: -1,
            dz: 0,
            solid: true,
            empty: false,
            above_empty: false,
        },
        super::super::MonsterRoomProbe {
            dx: 0,
            dy: 4,
            dz: 0,
            solid: true,
            empty: false,
            above_empty: false,
        },
        super::super::MonsterRoomProbe {
            dx: -3,
            dy: 0,
            dz: 0,
            solid: false,
            empty: true,
            above_empty: true,
        },
    ];
    assert_eq!(
        super::super::monster_room_opening_count(room_radii, &probes),
        Some(1)
    );
    assert!(super::super::monster_room_can_place(room_radii, &probes));

    let invalid_floor = [super::super::MonsterRoomProbe {
        dx: 0,
        dy: -1,
        dz: 0,
        solid: false,
        empty: true,
        above_empty: true,
    }];
    assert_eq!(
        super::super::monster_room_opening_count(room_radii, &invalid_floor),
        None
    );
}

#[test]
fn monster_room_shell_spawner_and_ore_rules_match_vanilla() {
    let room_radii = expected_room_radii();

    assert_eq!(
        super::super::monster_room_shell_state(super::super::MonsterRoomShellInput {
            relative_pos: BlockPos { x: -3, y: -1, z: 0 },
            radii: room_radii,
            world_y: 31,
            below_solid: true,
            current_solid: true,
            current_is_chest: false,
            mossy_roll: 1,
        }),
        Some("minecraft:mossy_cobblestone")
    );
    assert_eq!(
        super::super::monster_room_shell_state(super::super::MonsterRoomShellInput {
            relative_pos: BlockPos { x: -3, y: 0, z: 0 },
            radii: room_radii,
            world_y: 32,
            below_solid: false,
            current_solid: true,
            current_is_chest: false,
            mossy_roll: 0,
        }),
        Some("minecraft:cave_air")
    );
    assert_eq!(
        super::super::monster_room_shell_state(super::super::MonsterRoomShellInput {
            relative_pos: BlockPos { x: 0, y: 0, z: 0 },
            radii: room_radii,
            world_y: 32,
            below_solid: true,
            current_solid: true,
            current_is_chest: false,
            mossy_roll: 0,
        }),
        Some("minecraft:cave_air")
    );
    assert!(super::super::monster_room_chest_can_place(true, 1));
    assert!(!super::super::monster_room_chest_can_place(true, 2));
    assert_eq!(
        super::super::monster_room_spawner_mob(0),
        "minecraft:skeleton"
    );
    assert_eq!(
        super::super::monster_room_spawner_mob(1),
        "minecraft:zombie"
    );
    assert_eq!(
        super::super::monster_room_spawner_mob(2),
        "minecraft:zombie"
    );
    assert_eq!(
        super::super::monster_room_spawner_mob(3),
        "minecraft:spider"
    );
    assert!(super::super::ore_vein_sphere_is_shadowed(
        3.0, 1.0, 1.0, 1.0
    ));
    assert!(!super::super::ore_vein_sphere_is_shadowed(
        1.0, 2.0, 0.0, 0.0
    ));
}

fn expected_room_radii() -> super::super::MonsterRoomRadii {
    let room_radii = super::super::monster_room_radii(0, 1);
    assert_eq!(
        room_radii,
        super::super::MonsterRoomRadii {
            x_radius: 2,
            z_radius: 3,
        }
    );
    room_radii
}
