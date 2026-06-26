use super::*;

#[test]
fn setworldspawn_uses_source_or_explicit_position_and_rotation() {
    let mut state = ServerCommandState {
        command_source_position: Vec3 {
            x: 12.9,
            y: 64.0,
            z: -3.1,
        },
        command_source_yaw: 0.0,
        command_source_pitch: 0.0,
        command_source_dimension: "minecraft:the_nether".to_string(),
        ..ServerCommandState::default()
    };
    assert_eq!(
        command_required_permission("setworldspawn"),
        PermissionLevel::Gamemasters
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::MODERATOR,
            "setworldspawn"
        ),
        Err(CommandError::PermissionDenied)
    );

    let source = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "setworldspawn",
    )
    .unwrap();
    assert_eq!(source.success_count, 1);
    assert_eq!(source.feedback_key, "commands.setworldspawn.success");
    assert_eq!(
        state.world_spawn,
        RespawnData {
            dimension: "minecraft:the_nether".to_string(),
            position: BlockPos {
                x: 12,
                y: 64,
                z: -4
            },
            yaw: 0.0,
            pitch: 0.0,
        }
    );

    execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "setworldspawn 1 70 2 270 -120",
    )
    .unwrap();
    assert_eq!(
        state.world_spawn,
        RespawnData {
            dimension: "minecraft:the_nether".to_string(),
            position: BlockPos { x: 1, y: 70, z: 2 },
            yaw: 270.0,
            pitch: -120.0,
        }
    );

    execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "setworldspawn ~1 ~ ~-2",
    )
    .unwrap();
    assert_eq!(
        state.world_spawn.position,
        BlockPos {
            x: 13,
            y: 64,
            z: -6
        }
    );

    execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "setworldspawn ^1 ^ ^2",
    )
    .unwrap();
    assert_eq!(
        state.world_spawn.position,
        BlockPos {
            x: 13,
            y: 64,
            z: -2
        }
    );
}

#[test]
fn spawnpoint_sets_single_or_multiple_player_respawns() {
    let mut state = ServerCommandState {
        command_source_player: Some(NameAndId::create_offline("Steve")),
        command_source_position: Vec3 {
            x: 10.0,
            y: 65.5,
            z: -2.0,
        },
        command_source_yaw: 0.0,
        command_source_pitch: 0.0,
        ..ServerCommandState::default()
    };
    assert_eq!(
        command_required_permission("spawnpoint"),
        PermissionLevel::Gamemasters
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::MODERATOR,
            "spawnpoint Steve"
        ),
        Err(CommandError::PermissionDenied)
    );

    let own = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "spawnpoint",
    )
    .unwrap();
    assert_eq!(own.success_count, 1);
    assert_eq!(own.feedback_key, "commands.spawnpoint.success.single");
    assert_eq!(
        state.player_spawns[0],
        PlayerSpawn {
            player: NameAndId::create_offline("Steve"),
            respawn: RespawnData {
                dimension: "minecraft:overworld".to_string(),
                position: BlockPos {
                    x: 10,
                    y: 65,
                    z: -2
                },
                yaw: 0.0,
                pitch: 0.0,
            },
            forced: true,
        }
    );

    let multiple = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "spawnpoint Steve,Alex 1 70 2 270 -120",
    )
    .unwrap();
    assert_eq!(multiple.success_count, 2);
    assert_eq!(
        multiple.feedback_key,
        "commands.spawnpoint.success.multiple"
    );
    assert_eq!(state.player_spawns[0].respawn.yaw, -90.0);
    assert_eq!(state.player_spawns[0].respawn.pitch, -90.0);
    assert_eq!(state.player_spawns[1].player.name, "Alex");

    execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "spawnpoint Steve ~1 ~ ~-2",
    )
    .unwrap();
    assert_eq!(
        state.player_spawns[0].respawn.position,
        BlockPos { x: 11, y: 65, z: -4 }
    );
    execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "spawnpoint Steve ^1 ^ ^2",
    )
    .unwrap();
    assert_eq!(
        state.player_spawns[0].respawn.position,
        BlockPos { x: 10, y: 65, z: 0 }
    );
}

#[test]
fn spawnpoint_and_setworldspawn_reject_invalid_syntax_or_missing_player() {
    let mut state = ServerCommandState::default();
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "spawnpoint"
        ),
        Err(CommandError::InvalidSyntax)
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "spawnpoint Steve 1 2"
        ),
        Err(CommandError::InvalidSyntax)
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "setworldspawn 1 2"
        ),
        Err(CommandError::InvalidSyntax)
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "spawnpoint Steve 0 20000000 0"
        ),
        Err(CommandError::InvalidSyntax)
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "setworldspawn 30000000 0 0"
        ),
        Err(CommandError::InvalidSyntax)
    );
}
