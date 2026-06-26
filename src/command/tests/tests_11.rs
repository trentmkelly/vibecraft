use super::*;

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
