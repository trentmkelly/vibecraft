use super::*;

#[test]
fn effect_command_reapply_counts_only_java_effect_updates() {
    let mut state = ServerCommandState {
        active_effects: vec![ActiveEffect {
            target: super::entity_ref("Steve"),
            effect: "minecraft:speed".to_string(),
            duration_ticks: 600,
            amplifier: 1,
            show_particles: true,
        }],
        ..ServerCommandState::default()
    };

    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "effect give Steve speed 10 0 false"
        ),
        Err(CommandError::EffectGiveFailed)
    );
    assert_eq!(state.active_effects[0].duration_ticks, 600);
    assert_eq!(state.active_effects[0].amplifier, 1);

    let stronger = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "effect give Steve speed 1 2 true",
    )
    .unwrap();
    assert_eq!(stronger.success_count, 1);
    assert_eq!(state.active_effects[0].duration_ticks, 20);
    assert_eq!(state.active_effects[0].amplifier, 2);
    assert!(!state.active_effects[0].show_particles);

    let longer = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "effect give Steve speed infinite 2 false",
    )
    .unwrap();
    assert_eq!(longer.success_count, 1);
    assert_eq!(state.active_effects[0].duration_ticks, -1);
    assert_eq!(state.active_effects[0].amplifier, 2);
    assert!(state.active_effects[0].show_particles);
}

#[test]
fn effect_command_reports_changed_count_but_feedback_uses_target_count() {
    let mut state = ServerCommandState {
        entity_states: vec![EntityState {
            entity: super::entity_ref("armor_stand"),
            kind: EntityKind::NonLiving,
            dimension: "minecraft:overworld".to_string(),
        }],
        ..ServerCommandState::default()
    };

    let result = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "effect give Steve,armor_stand speed",
    )
    .unwrap();
    assert_eq!(result.success_count, 1);
    assert_eq!(result.feedback_key, "commands.effect.give.success.multiple");

    let clear = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "effect clear Steve,armor_stand",
    )
    .unwrap();
    assert_eq!(clear.success_count, 1);
    assert_eq!(
        clear.feedback_key,
        "commands.effect.clear.everything.success.multiple"
    );
}

#[test]
#[cfg(vibecraft_has_decompiled_sources)]
fn effect_command_source_matches_java_26_1_2() {
    const EFFECT_COMMAND_JAVA: &str =
        vibecraft_java_source!("/net/minecraft/server/commands/EffectCommands.java");
    const LIVING_ENTITY_JAVA: &str =
        vibecraft_java_source!("/net/minecraft/world/entity/LivingEntity.java");
    const MOB_EFFECT_INSTANCE_JAVA: &str =
        vibecraft_java_source!("/net/minecraft/world/effect/MobEffectInstance.java");

    for sentinel in [
        "Commands.literal(\"effect\")",
        ".requires(Commands.hasPermission(Commands.LEVEL_GAMEMASTERS))",
        "Commands.literal(\"clear\")",
        "ImmutableList.of(((CommandSourceStack)c.getSource()).getEntityOrException())",
        "Commands.argument(\"targets\", EntityArgument.entities())",
        "Commands.argument(\"effect\", ResourceArgument.resource(context, Registries.MOB_EFFECT))",
        "Commands.literal(\"give\")",
        "Commands.argument(\"seconds\", IntegerArgumentType.integer(1, 1000000))",
        "Commands.argument(\"amplifier\", IntegerArgumentType.integer(0, 255))",
        "Commands.argument(\"hideParticles\", BoolArgumentType.bool())",
        "Commands.literal(\"infinite\")",
        "duration = seconds * 20;",
        "duration = 600;",
        "new MobEffectInstance(effectHolder, duration, amplifier, false, particles)",
        "throw ERROR_GIVE_FAILED.create();",
        "throw ERROR_CLEAR_EVERYTHING_FAILED.create();",
        "throw ERROR_CLEAR_SPECIFIC_FAILED.create();",
        "return count;",
    ] {
        assert!(
            EFFECT_COMMAND_JAVA.contains(sentinel),
            "EffectCommands.java is missing sentinel: {sentinel}"
        );
    }

    for sentinel in [
        "if (!this.canBeAffected(newEffect))",
        "if (effect == null)",
        "} else if (effect.update(newEffect))",
        "return changed;",
    ] {
        assert!(
            LIVING_ENTITY_JAVA.contains(sentinel),
            "LivingEntity.java is missing sentinel: {sentinel}"
        );
    }

    for sentinel in [
        "public boolean update(final MobEffectInstance takeOver)",
        "if (takeOver.amplifier > this.amplifier)",
        "} else if (this.isShorterDurationThan(takeOver))",
        "if (takeOver.visible != this.visible)",
        "return changed;",
    ] {
        assert!(
            MOB_EFFECT_INSTANCE_JAVA.contains(sentinel),
            "MobEffectInstance.java is missing sentinel: {sentinel}"
        );
    }
}
