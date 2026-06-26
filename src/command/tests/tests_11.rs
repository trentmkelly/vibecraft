use super::*;

#[test]
fn particle_command_requires_gamemaster_and_defaults_to_all_online_players() {
    let mut state = ServerCommandState {
        command_source_position: Vec3 {
            x: 10.0,
            y: 65.0,
            z: -4.0,
        },
        online_players: vec![
            NameAndId::create_offline("Steve"),
            NameAndId::create_offline("Alex"),
        ],
        ..ServerCommandState::default()
    };
    assert_eq!(
        command_required_permission("particle"),
        PermissionLevel::Gamemasters
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::MODERATOR,
            "particle flame"
        ),
        Err(CommandError::PermissionDenied)
    );

    let result = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "particle flame",
    )
    .unwrap();
    assert_eq!(result.success_count, 2);
    assert_eq!(result.feedback_key, "commands.particle.success");
    assert_eq!(
        state.particle_events[0],
        ParticleCommandEvent {
            name: "flame".to_string(),
            viewers: vec![
                NameAndId::create_offline("Steve"),
                NameAndId::create_offline("Alex")
            ],
            position: Vec3 {
                x: 10.0,
                y: 65.0,
                z: -4.0,
            },
            delta: Vec3::default(),
            speed: 0.0,
            count: 0,
            force: false,
        }
    );
}

#[test]
fn particle_command_parses_position_delta_speed_count_mode_and_viewers() {
    let mut state = ServerCommandState {
        command_source_position: Vec3 {
            x: 10.0,
            y: 64.0,
            z: 10.0,
        },
        ..ServerCommandState::default()
    };
    let result = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "particle minecraft:dust ~1 2 ~-3 0.1 0.2 0.3 0.4 12 force Steve,Alex",
    )
    .unwrap();
    assert_eq!(result.success_count, 2);
    assert_eq!(
        state.particle_events[0],
        ParticleCommandEvent {
            name: "minecraft:dust".to_string(),
            viewers: vec![
                NameAndId::create_offline("Steve"),
                NameAndId::create_offline("Alex")
            ],
            position: Vec3 {
                x: 11.0,
                y: 2.0,
                z: 7.0,
            },
            delta: Vec3 {
                x: 0.1,
                y: 0.2,
                z: 0.3,
            },
            speed: 0.4,
            count: 12,
            force: true,
        }
    );

    state.online_players = vec![NameAndId::create_offline("Steve")];
    let normal = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "particle flame 1 2 3 0 0 0 0 1 normal",
    )
    .unwrap();
    assert_eq!(normal.success_count, 1);
    assert!(!state.particle_events.last().unwrap().force);
}

#[test]
fn particle_command_fails_when_no_players_receive_particles() {
    let mut state = ServerCommandState::default();
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "particle flame"
        ),
        Err(CommandError::ParticleFailed)
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "particle flame 0 0 0 0 0 0 -1 1"
        ),
        Err(CommandError::InvalidSyntax)
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "particle flame 0 0 0 0 0 0 0 1 Steve"
        ),
        Err(CommandError::InvalidSyntax)
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "particle flame 0 0 0 0 0 0 0 2147483648"
        ),
        Err(CommandError::InvalidSyntax)
    );
}

#[test]
#[cfg(vibecraft_has_decompiled_sources)]
fn particle_command_source_matches_java_26_1_2() {
    const PARTICLE_COMMAND_JAVA: &str =
        vibecraft_java_source!("/net/minecraft/server/commands/ParticleCommand.java");

    for sentinel in [
        "Commands.literal(\"particle\").requires(Commands.hasPermission(Commands.LEVEL_GAMEMASTERS))",
        "Commands.argument(\"name\", ParticleArgument.particle(context))",
        "((CommandSourceStack)c.getSource()).getPosition()",
        "Commands.argument(\"pos\", Vec3Argument.vec3())",
        "Commands.argument(\"delta\", Vec3Argument.vec3(false))",
        "Commands.argument(\"speed\", FloatArgumentType.floatArg(0.0F))",
        "Commands.argument(\"count\", IntegerArgumentType.integer(0))",
        "Commands.literal(\"force\")",
        "Commands.literal(\"normal\")",
        "Commands.argument(\"viewers\", EntityArgument.players())",
        "source.getLevel().sendParticles(player, particle, force, false",
        "throw ERROR_FAILED.create();",
        "Component.translatable(\"commands.particle.success\"",
        "return result;",
    ] {
        assert!(
            PARTICLE_COMMAND_JAVA.contains(sentinel),
            "ParticleCommand.java is missing sentinel: {sentinel}"
        );
    }
}

#[test]
fn attribute_command_rejects_non_living_targets_separately_from_missing_attributes() {
    let mut state = ServerCommandState {
        entity_states: vec![EntityState {
            entity: EntityRef {
                id: "minecart".to_string(),
                display_name: "Minecart".to_string(),
            },
            kind: EntityKind::NonLiving,
            dimension: "minecraft:overworld".to_string(),
        }],
        entity_attributes: vec![EntityAttributeState {
            target: "minecart".to_string(),
            attribute: "minecraft:max_health".to_string(),
            default_base: 20.0,
            base: 20.0,
            modifiers: Vec::new(),
        }],
        ..ServerCommandState::default()
    };

    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "attribute minecart minecraft:max_health get",
        ),
        Err(CommandError::AttributeNotLiving)
    );
}

#[test]
fn attribute_command_computes_all_java_modifier_operations_in_order() {
    let mut state = ServerCommandState {
        entity_attributes: vec![EntityAttributeState {
            target: "Steve".to_string(),
            attribute: "minecraft:attack_damage".to_string(),
            default_base: 10.0,
            base: 10.0,
            modifiers: Vec::new(),
        }],
        ..ServerCommandState::default()
    };

    for command in [
        "attribute Steve minecraft:attack_damage modifier add minecraft:value 2 add_value",
        "attribute Steve minecraft:attack_damage modifier add minecraft:base 0.5 add_multiplied_base",
        "attribute Steve minecraft:attack_damage modifier add minecraft:total 0.25 add_multiplied_total",
    ] {
        execute_builtin_command(&mut state, LevelBasedPermissionSet::GAMEMASTER, command).unwrap();
    }

    assert_eq!(
        state.entity_attributes[0].modifiers,
        vec![
            AttributeModifierState {
                id: "minecraft:value".to_string(),
                value: 2.0,
                operation: AttributeOperation::Value,
            },
            AttributeModifierState {
                id: "minecraft:base".to_string(),
                value: 0.5,
                operation: AttributeOperation::MultipliedBase,
            },
            AttributeModifierState {
                id: "minecraft:total".to_string(),
                value: 0.25,
                operation: AttributeOperation::MultipliedTotal,
            },
        ]
    );

    let value = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "attribute Steve minecraft:attack_damage get 10",
    )
    .unwrap();
    assert_eq!(value.success_count, 212);
}

#[test]
#[cfg(vibecraft_has_decompiled_sources)]
fn attribute_command_source_matches_java_26_1_2() {
    const ATTRIBUTE_COMMAND_JAVA: &str =
        vibecraft_java_source!("/net/minecraft/server/commands/AttributeCommand.java");

    for sentinel in [
        "Commands.literal(\"attribute\").requires(Commands.hasPermission(Commands.LEVEL_GAMEMASTERS))",
        "Commands.argument(\"target\", EntityArgument.entity())",
        "\"attribute\", ResourceArgument.resource(context, Registries.ATTRIBUTE)",
        "Commands.literal(\"get\")",
        "Commands.literal(\"base\")",
        "Commands.literal(\"set\")",
        "Commands.literal(\"reset\")",
        "Commands.literal(\"modifier\")",
        "Commands.literal(\"add\")",
        "Commands.literal(\"add_value\")",
        "Commands.literal(\"add_multiplied_base\")",
        "Commands.literal(\"add_multiplied_total\")",
        "Commands.literal(\"remove\")",
        "Commands.literal(\"value\")",
        "ERROR_NOT_LIVING_ENTITY.create(target.getName())",
        "ERROR_NO_SUCH_ATTRIBUTE.create(target.getName(), getAttributeDescription(attribute))",
        "ERROR_NO_SUCH_MODIFIER.create(target.getName(), getAttributeDescription(attribute), id)",
        "ERROR_MODIFIER_ALREADY_PRESENT.create(target.getName(), getAttributeDescription(attribute), id)",
        "return (int)(result * scale);",
        "commands.attribute.value.get.success",
        "commands.attribute.base_value.get.success",
        "commands.attribute.base_value.set.success",
        "commands.attribute.base_value.reset.success",
        "commands.attribute.modifier.add.success",
        "commands.attribute.modifier.remove.success",
        "commands.attribute.modifier.value.get.success",
    ] {
        assert!(
            ATTRIBUTE_COMMAND_JAVA.contains(sentinel),
            "AttributeCommand.java is missing sentinel: {sentinel}"
        );
    }
}
