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
fn enchant_command_allows_zero_level_and_rejects_duplicate_existing_enchantment() {
    let mut state = ServerCommandState {
        player_inventories: vec![CommandPlayerInventory {
            player: NameAndId::create_offline("Steve"),
            items: vec![CommandItemStack {
                item: "minecraft:diamond_sword".to_string(),
                count: 1,
            }],
        }],
        ..ServerCommandState::default()
    };

    let level_zero = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "enchant Steve sharpness 0",
    )
    .unwrap();
    assert_eq!(level_zero.success_count, 1);
    assert_eq!(
        state.item_enchantments,
        vec![CommandItemEnchantment {
            target: super::entity_ref("Steve"),
            item: "minecraft:diamond_sword".to_string(),
            enchantment: "minecraft:sharpness".to_string(),
            level: 0,
        }]
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "enchant Steve sharpness 1"
        ),
        Err(CommandError::EnchantIncompatible)
    );
}

#[test]
fn fill_command_allows_filtered_modes_like_java_wrap_with_mode() {
    let mut state = ServerCommandState {
        blocks: vec![BlockStateEntry {
            dimension: "minecraft:overworld".to_string(),
            position: BlockPos { x: 1, y: 1, z: 1 },
            block: "minecraft:stone".to_string(),
        }],
        ..ServerCommandState::default()
    };

    let hollow = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "fill 0 0 0 2 2 2 glass replace stone hollow",
    )
    .unwrap();
    assert_eq!(hollow.success_count, 1);
    let event = state.fill_events.last().unwrap();
    assert_eq!(event.mode, FillMode::Hollow);
    assert_eq!(event.filter, Some("minecraft:stone".to_string()));
    assert!(state.blocks.iter().any(|entry| {
        entry.position == BlockPos { x: 1, y: 1, z: 1 } && entry.block == "minecraft:air"
    }));

    state.blocks.push(BlockStateEntry {
        dimension: "minecraft:overworld".to_string(),
        position: BlockPos { x: 3, y: 0, z: 0 },
        block: "minecraft:dirt".to_string(),
    });
    let strict = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "fill 3 0 0 3 0 0 gold_block replace dirt strict",
    )
    .unwrap();
    assert_eq!(strict.success_count, 1);
    let event = state.fill_events.last().unwrap();
    assert_eq!(event.mode, FillMode::Replace);
    assert_eq!(event.filter, Some("minecraft:dirt".to_string()));
    assert!(event.strict);
}

#[test]
fn fillbiome_command_counts_matching_cells_even_when_biome_is_unchanged() {
    let mut state = ServerCommandState {
        biomes: vec![BiomeEntry {
            dimension: "minecraft:overworld".to_string(),
            position: BlockPos { x: 0, y: 0, z: 0 },
            biome: "minecraft:plains".to_string(),
        }],
        ..ServerCommandState::default()
    };

    let result = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "fillbiome 0 0 0 0 0 0 plains",
    )
    .unwrap();
    assert_eq!(result.success_count, 1);
    assert_eq!(state.fill_biome_events[0].count, 1);
}

#[test]
#[cfg(vibecraft_has_decompiled_sources)]
fn fill_commands_source_match_java_26_1_2() {
    const FILL_COMMAND_JAVA: &str =
        vibecraft_java_source!("/net/minecraft/server/commands/FillCommand.java");
    const FILL_BIOME_COMMAND_JAVA: &str =
        vibecraft_java_source!("/net/minecraft/server/commands/FillBiomeCommand.java");

    for sentinel in [
        "Commands.literal(\"fill\").requires(Commands.hasPermission(Commands.LEVEL_GAMEMASTERS))",
        "Commands.argument(\"from\", BlockPosArgument.blockPos())",
        "Commands.argument(\"to\", BlockPosArgument.blockPos())",
        "Commands.argument(\"block\", BlockStateArgument.block(context))",
        "Commands.literal(\"replace\")",
        "Commands.argument(\"filter\", BlockPredicateArgument.blockPredicate(context))",
        "Commands.literal(\"keep\")",
        "Commands.literal(\"outline\")",
        "Commands.literal(\"hollow\")",
        "Commands.literal(\"destroy\")",
        "Commands.literal(\"strict\")",
        "source.getLevel().getGameRules().get(GameRules.MAX_BLOCK_MODIFICATIONS)",
        "if (level.isDebug())",
        "throw ERROR_FAILED.create();",
        "Component.translatable(\"commands.fill.success\", finalCount)",
        "return count;",
    ] {
        assert!(
            FILL_COMMAND_JAVA.contains(sentinel),
            "FillCommand.java is missing sentinel: {sentinel}"
        );
    }

    for sentinel in [
        "Commands.literal(\"fillbiome\").requires(Commands.hasPermission(Commands.LEVEL_GAMEMASTERS))",
        "Commands.argument(\"biome\", ResourceArgument.resource(context, Registries.BIOME))",
        "Commands.literal(\"replace\")",
        "Commands.argument(\"filter\", ResourceOrTagArgument.resourceOrTag(context, Registries.BIOME))",
        "private static int quantize(final int blockCoord)",
        "BlockPos from = quantize(rawFrom);",
        "if (volume > limit)",
        "return Either.right(ERROR_NOT_LOADED.create());",
        "count.increment();",
        "chunk.fillBiomesFromNoise",
        "resendBiomesForChunks(chunks)",
        "commands.fillbiome.success.count",
    ] {
        assert!(
            FILL_BIOME_COMMAND_JAVA.contains(sentinel),
            "FillBiomeCommand.java is missing sentinel: {sentinel}"
        );
    }
}

#[test]
#[cfg(vibecraft_has_decompiled_sources)]
fn enchant_command_source_matches_java_26_1_2() {
    const ENCHANT_COMMAND_JAVA: &str =
        vibecraft_java_source!("/net/minecraft/server/commands/EnchantCommand.java");
    const ENCHANTMENT_JAVA: &str =
        vibecraft_java_source!("/net/minecraft/world/item/enchantment/Enchantment.java");

    for sentinel in [
        "Commands.literal(\"enchant\").requires(Commands.hasPermission(Commands.LEVEL_GAMEMASTERS))",
        "Commands.argument(\"targets\", EntityArgument.entities())",
        "Commands.argument(\"enchantment\", ResourceArgument.resource(context, Registries.ENCHANTMENT))",
        "Commands.argument(\"level\", IntegerArgumentType.integer(0))",
        "if (level > enchantment.getMaxLevel())",
        "if (entity instanceof LivingEntity target)",
        "ItemStack item = target.getMainHandItem();",
        "enchantment.canEnchant(item)",
        "EnchantmentHelper.isEnchantmentCompatible(EnchantmentHelper.getEnchantmentsForCrafting(item).keySet(), enchantmentHolder)",
        "item.enchant(enchantmentHolder, level);",
        "throw ERROR_INCOMPATIBLE.create(item.getHoverName().getString());",
        "throw ERROR_NO_ITEM.create(target.getName().getString());",
        "throw ERROR_NOT_LIVING_ENTITY.create(entity.getName().getString());",
        "throw ERROR_NOTHING_HAPPENED.create();",
        "commands.enchant.success.single",
        "commands.enchant.success.multiple",
        "return success;",
    ] {
        assert!(
            ENCHANT_COMMAND_JAVA.contains(sentinel),
            "EnchantCommand.java is missing sentinel: {sentinel}"
        );
    }

    for sentinel in [
        "public static boolean areCompatible(final Holder<Enchantment> enchantment, final Holder<Enchantment> other)",
        "return !enchantment.equals(other)",
        "!enchantment.value().exclusiveSet.contains(other)",
        "!other.value().exclusiveSet.contains(enchantment)",
    ] {
        assert!(
            ENCHANTMENT_JAVA.contains(sentinel),
            "Enchantment.java is missing sentinel: {sentinel}"
        );
    }
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
