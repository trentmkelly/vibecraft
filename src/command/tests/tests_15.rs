use super::*;

#[test]
fn playsound_requires_gamemaster_and_defaults_to_source_player() {
    let mut state = ServerCommandState {
        command_source_player: Some(NameAndId::create_offline("Steve")),
        command_source_position: Vec3 {
            x: 8.0,
            y: 65.0,
            z: -3.0,
        },
        ..ServerCommandState::default()
    };
    assert_eq!(
        command_required_permission("playsound"),
        PermissionLevel::Gamemasters
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::MODERATOR,
            "playsound minecraft:block.note_block.harp"
        ),
        Err(CommandError::PermissionDenied)
    );

    let result = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "playsound minecraft:block.note_block.harp",
    )
    .unwrap();
    assert_eq!(result.success_count, 1);
    assert_eq!(result.feedback_key, "commands.playsound.success.single");
    assert_eq!(
        state.sound_events[0],
        SoundCommandEvent::Play(PlaySoundRequest {
            sound: "minecraft:block.note_block.harp".to_string(),
            source: SoundSource::Master,
            targets: vec![NameAndId::create_offline("Steve")],
            position: Vec3 {
                x: 8.0,
                y: 65.0,
                z: -3.0,
            },
            volume: 1.0,
            pitch: 1.0,
            min_volume: 0.0,
            deliveries: vec![PlaySoundDelivery {
                target: NameAndId::create_offline("Steve"),
                position: Vec3 {
                    x: 8.0,
                    y: 65.0,
                    z: -3.0,
                },
                volume: 1.0,
            }],
        })
    );
}

#[test]
fn playsound_accepts_source_targets_position_volume_pitch_and_min_volume() {
    let steve = entity_ref("Steve");
    let alex = entity_ref("Alex");
    let mut state = ServerCommandState {
        command_source_position: Vec3 {
            x: 10.0,
            y: 64.0,
            z: 10.0,
        },
        entity_positions: vec![
            EntityPosition {
                entity: steve,
                dimension: "minecraft:overworld".to_string(),
                position: Vec3 {
                    x: 10.0,
                    y: 2.0,
                    z: 6.5,
                },
            },
            EntityPosition {
                entity: alex,
                dimension: "minecraft:overworld".to_string(),
                position: Vec3 {
                    x: -100.0,
                    y: 2.0,
                    z: 6.5,
                },
            },
        ],
        ..ServerCommandState::default()
    };
    let result = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "playsound minecraft:entity.arrow.hit player Steve,Alex ~1.5 2.0 ~-3.5 4.0 0.75 0.25",
    )
    .unwrap();
    assert_eq!(result.success_count, 2);
    assert_eq!(result.feedback_key, "commands.playsound.success.multiple");
    assert_eq!(
        state.sound_events[0],
        SoundCommandEvent::Play(PlaySoundRequest {
            sound: "minecraft:entity.arrow.hit".to_string(),
            source: SoundSource::Player,
            targets: vec![
                NameAndId::create_offline("Steve"),
                NameAndId::create_offline("Alex")
            ],
            position: Vec3 {
                x: 11.5,
                y: 2.0,
                z: 6.5,
            },
            volume: 4.0,
            pitch: 0.75,
            min_volume: 0.25,
            deliveries: vec![
                PlaySoundDelivery {
                    target: NameAndId::create_offline("Steve"),
                    position: Vec3 {
                        x: 11.5,
                        y: 2.0,
                        z: 6.5,
                    },
                    volume: 4.0,
                },
                PlaySoundDelivery {
                    target: NameAndId::create_offline("Alex"),
                    position: Vec3 {
                        x: -98.0,
                        y: 2.0,
                        z: 6.5,
                    },
                    volume: 0.25,
                },
            ],
        })
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "playsound minecraft:bad player Steve 0 0 0 1 2.5"
        ),
        Err(CommandError::InvalidSyntax)
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "playsound minecraft:entity.arrow.hit player Steve 0"
        ),
        Err(CommandError::InvalidSyntax)
    );
}

#[test]
fn playsound_fails_when_no_target_receives_sound() {
    let mut state = ServerCommandState {
        command_source_position: Vec3::default(),
        entity_states: vec![EntityState {
            entity: entity_ref("Alex"),
            kind: EntityKind::Player,
            dimension: "minecraft:the_nether".to_string(),
        }],
        entity_positions: vec![
            EntityPosition {
                entity: entity_ref("Steve"),
                dimension: "minecraft:overworld".to_string(),
                position: Vec3 {
                    x: 100.0,
                    y: 0.0,
                    z: 0.0,
                },
            },
            EntityPosition {
                entity: entity_ref("Alex"),
                dimension: "minecraft:the_nether".to_string(),
                position: Vec3::default(),
            },
        ],
        ..ServerCommandState::default()
    };
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "playsound minecraft:empty player Steve,Alex 0 0 0"
        ),
        Err(CommandError::PlaySoundTooFar)
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "playsound minecraft:empty"
        ),
        Err(CommandError::PlaySoundTooFar)
    );
}

#[test]
fn stopsound_queues_source_and_sound_filters() {
    let mut state = ServerCommandState::default();
    assert_eq!(
        command_required_permission("stopsound"),
        PermissionLevel::Gamemasters
    );
    let result = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "stopsound Steve,Alex player minecraft:entity.arrow.hit",
    )
    .unwrap();
    assert_eq!(result.success_count, 2);
    assert_eq!(
        result.feedback_key,
        "commands.stopsound.success.source.sound"
    );
    assert_eq!(
        state.sound_events[0],
        SoundCommandEvent::Stop(StopSoundRequest {
            targets: vec![
                NameAndId::create_offline("Steve"),
                NameAndId::create_offline("Alex")
            ],
            source: Some(SoundSource::Player),
            sound: Some("minecraft:entity.arrow.hit".to_string()),
        })
    );

    let result = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "stopsound Steve * minecraft:music.menu",
    )
    .unwrap();
    assert_eq!(
        result.feedback_key,
        "commands.stopsound.success.sourceless.sound"
    );
    let all = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "stopsound Steve",
    )
    .unwrap();
    assert_eq!(all.feedback_key, "commands.stopsound.success.sourceless.any");
    let source_only = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "stopsound Steve block",
    )
    .unwrap();
    assert_eq!(
        source_only.feedback_key,
        "commands.stopsound.success.source.any"
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "stopsound Steve *"
        ),
        Err(CommandError::InvalidSyntax)
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "stopsound Steve players minecraft:music.menu"
        ),
        Err(CommandError::InvalidSyntax)
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "stopsound Steve player bad id"
        ),
        Err(CommandError::InvalidSyntax)
    );
}

#[test]
#[cfg(vibecraft_has_decompiled_sources)]
fn sound_command_sources_match_java_26_1_2() {
    const PLAY_SOUND_JAVA: &str =
        vibecraft_java_source!("/net/minecraft/server/commands/PlaySoundCommand.java");
    const STOP_SOUND_JAVA: &str =
        vibecraft_java_source!("/net/minecraft/server/commands/StopSoundCommand.java");

    for sentinel in [
        "Commands.literal(\"playsound\").requires(Commands.hasPermission(Commands.LEVEL_GAMEMASTERS))",
        "\"sound\", IdentifierArgument.id()",
        "for (SoundSource source : SoundSource.values())",
        "getCallingPlayerAsCollection(((CommandSourceStack)c.getSource()).getPlayer())",
        "((CommandSourceStack)c.getSource()).getPosition()",
        "Commands.argument(\"pos\", Vec3Argument.vec3())",
        "Commands.argument(\"volume\", FloatArgumentType.floatArg(0.0F))",
        "Commands.argument(\"pitch\", FloatArgumentType.floatArg(0.0F, 2.0F))",
        "Commands.argument(\"minVolume\", FloatArgumentType.floatArg(0.0F, 1.0F))",
        "double maxDistSqr = Mth.square(soundHolder.value().getRange(volume));",
        "if (player.level() == level)",
        "if (distSqr > maxDistSqr)",
        "if (minVolume <= 0.0F)",
        "localPosition = new Vec3(",
        "new ClientboundSoundPacket(soundHolder, soundSource, localPosition.x(), localPosition.y(), localPosition.z(), localVolume, pitch, seed)",
        "throw ERROR_TOO_FAR.create();",
        "Component.translatable(\"commands.playsound.success.single\"",
        "return count;",
    ] {
        assert!(
            PLAY_SOUND_JAVA.contains(sentinel),
            "PlaySoundCommand.java is missing sentinel: {sentinel}"
        );
    }

    for sentinel in [
        "Commands.literal(\"stopsound\").requires(Commands.hasPermission(Commands.LEVEL_GAMEMASTERS))",
        "Commands.argument(\n               \"targets\", EntityArgument.players()",
        "Commands.literal(\"*\")",
        "for (SoundSource source : SoundSource.values())",
        "new ClientboundStopSoundPacket(sound, soundSource)",
        "Component.translatable(\"commands.stopsound.success.source.sound\"",
        "Component.translatable(\"commands.stopsound.success.source.any\"",
        "Component.translatable(\"commands.stopsound.success.sourceless.sound\"",
        "Component.translatable(\"commands.stopsound.success.sourceless.any\")",
        "return targets.size();",
    ] {
        assert!(
            STOP_SOUND_JAVA.contains(sentinel),
            "StopSoundCommand.java is missing sentinel: {sentinel}"
        );
    }
}
