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
fn gamemode_command_rejects_extra_target_arguments_like_java() {
    let mut state = ServerCommandState::default();
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "gamemode adventure Steve Alex"
        ),
        Err(CommandError::InvalidSyntax)
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
fn function_command_source_matches_java_26_1_2() {
    const FUNCTION_COMMAND: &str =
        vibecraft_java_source!("/net/minecraft/server/commands/FunctionCommand.java");

    for sentinel in [
        "Commands.literal(\"function\").requires(Commands.hasPermission(Commands.LEVEL_GAMEMASTERS))",
        "Commands.argument(\"name\", FunctionArgument.functions())",
        ".suggests(SUGGEST_FUNCTION)",
        "Commands.argument(\"arguments\", CompoundTagArgument.compoundTag())",
        "DataCommands.SOURCE_PROVIDERS",
        "Commands.argument(\"path\", NbtPathArgument.nbtPath())",
        "throw ERROR_ARGUMENT_NOT_COMPOUND.create(tag.getType().getName());",
        "return sender.withSuppressedOutput().withMaximumPermission(LevelBasedPermissionSet.GAMEMASTER);",
        "if (modifiers.isReturn())",
        "queueFunctionsAsReturn(functions, arguments, originalSource, functionSource, output, callbacks);",
        "queueFunctionsNoReturn(functions, arguments, originalSource, functionSource, output, callbacks);",
        "throw ERROR_FUNCTION_INSTANTATION_FAILURE.create(id, exception.messageComponent());",
        "callbacks.signalResult(originalSource, id, result);",
        "output.queueNext(new CallFunction<>(instantiatedFunction, functionResultCollector, returnParentFrame).bind(noCallbackSource));",
        "output.queueNext(FallthroughTask.instance());",
        "throw FunctionCommand.ERROR_NO_FUNCTIONS.create(Component.translationArg((Identifier)nameAndFunctions.getFirst()));",
        "commands.function.scheduled.single",
        "commands.function.scheduled.multiple",
        "FunctionCommand.queueFunctions(functions, arguments, sender, commonFunctionContext, output, FunctionCommand.FULL_CONTEXT_CALLBACKS, modifiers);",
    ] {
        assert!(
            FUNCTION_COMMAND.contains(sentinel),
            "FunctionCommand.java is missing sentinel: {sentinel}"
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

#[test]
fn give_command_uses_java_item_registry_stack_limits_and_feedback() {
    let mut state = ServerCommandState::default();

    let eggs = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "give Steve egg 1600",
    )
    .unwrap();
    assert_eq!(eggs.success_count, 1);
    assert_eq!(eggs.feedback_key, "commands.give.success.single");
    assert_eq!(state.player_inventories[0].items.len(), 100);
    assert!(state.player_inventories[0]
        .items
        .iter()
        .all(|stack| stack.item == "minecraft:egg" && stack.count == 16));

    let horse_armor = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "give Alex diamond_horse_armor 100",
    )
    .unwrap();
    assert_eq!(horse_armor.success_count, 1);
    let alex = state
        .player_inventories
        .iter()
        .find(|inventory| inventory.player.name == "Alex")
        .unwrap();
    assert_eq!(alex.items.len(), 100);
    assert!(alex
        .items
        .iter()
        .all(|stack| stack.item == "minecraft:diamond_horse_armor" && stack.count == 1));

    let multiple = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "give Steve,Alex stone 64",
    )
    .unwrap();
    assert_eq!(multiple.success_count, 2);
    assert_eq!(multiple.feedback_key, "commands.give.success.single");
}

#[test]
#[cfg(vibecraft_has_decompiled_sources)]
fn give_command_source_matches_java_26_1_2() {
    const GIVE: &str = vibecraft_java_source!("/net/minecraft/server/commands/GiveCommand.java");
    const ITEMS: &str = vibecraft_java_source!("/net/minecraft/world/item/Items.java");
    const ITEM: &str = vibecraft_java_source!("/net/minecraft/world/item/Item.java");

    assert!(GIVE.contains("public static final int MAX_ALLOWED_ITEMSTACKS = 100;"));
    assert!(GIVE.contains("Commands.literal(\"give\").requires(Commands.hasPermission(Commands.LEVEL_GAMEMASTERS))"));
    assert!(GIVE.contains("EntityArgument.players()"));
    assert!(GIVE.contains("ItemArgument.item(context)"));
    assert!(GIVE.contains("IntegerArgumentType.integer(1)"));
    assert!(GIVE.contains("int maxAllowedCount = maxStackSize * 100;"));
    assert!(GIVE.contains(
        "source.sendFailure(Component.translatable(\"commands.give.failed.toomanyitems\""
    ));
    assert!(GIVE.contains("return 0;"));
    assert!(GIVE.contains("while (remaining > 0)"));
    assert!(GIVE.contains("int size = Math.min(maxStackSize, remaining);"));
    assert!(GIVE.contains("return players.size();"));
    assert!(GIVE.contains(
        "\"commands.give.success.single\", count, prototypeItemStack.getDisplayName(), players.size()"
    ));
    assert!(ITEMS.contains("public static final Item EGG = registerItem("));
    assert!(ITEMS.contains("\"egg\", EggItem::new, new Item.Properties().stacksTo(16)"));
    assert!(ITEMS.contains(
        "public static final Item DIAMOND_HORSE_ARMOR = registerItem(\"diamond_horse_armor\""
    ));
    assert!(ITEM.contains("public Item.Properties horseArmor(final ArmorMaterial material)"));
    assert!(ITEM.contains(".stacksTo(1);"));
}

#[test]
#[cfg(vibecraft_has_decompiled_sources)]
fn help_command_source_matches_java_26_1_2() {
    const HELP: &str = vibecraft_java_source!("/net/minecraft/server/commands/HelpCommand.java");

    assert!(HELP.contains("Commands.literal(\"help\").executes"));
    assert!(HELP.contains("dispatcher.getSmartUsage(dispatcher.getRoot()"));
    assert!(HELP.contains("Component.literal(\"/\" + line)"));
    assert!(HELP.contains("return usage.size();"));
    assert!(HELP.contains("Commands.argument(\"command\", StringArgumentType.greedyString())"));
    assert!(HELP.contains("dispatcher.parse("));
    assert!(HELP.contains("if (command.getContext().getNodes().isEmpty())"));
    assert!(HELP.contains("throw ERROR_FAILED.create();"));
    assert!(HELP.contains("Iterables.getLast(command.getContext().getNodes())"));
    assert!(HELP.contains("Component.literal(\"/\" + command.getReader().getString() + \" \" + line)"));
}

#[test]
#[cfg(vibecraft_has_decompiled_sources)]
fn item_command_source_matches_java_26_1_2() {
    const ITEM_COMMANDS: &str =
        vibecraft_java_source!("/net/minecraft/server/commands/ItemCommands.java");
    const ITEM_INPUT: &str =
        vibecraft_java_source!("/net/minecraft/commands/arguments/item/ItemInput.java");
    const SLOT_ARGUMENT: &str =
        vibecraft_java_source!("/net/minecraft/commands/arguments/SlotArgument.java");
    const SLOT_RANGES: &str =
        vibecraft_java_source!("/net/minecraft/world/inventory/SlotRanges.java");

    for sentinel in [
        "Commands.literal(\"item\")",
        ".requires(Commands.hasPermission(Commands.LEVEL_GAMEMASTERS))",
        "Commands.literal(\"replace\")",
        "Commands.literal(\"modify\")",
        "Commands.literal(\"block\")",
        "Commands.literal(\"entity\")",
        "ItemArgument.item(context)",
        "IntegerArgumentType.integer(1, 99)",
        "ResourceOrIdArgument.lootModifier(context)",
        "getContainer(source, pos, ERROR_TARGET_NOT_A_CONTAINER)",
        "getContainer(source, pos, ERROR_SOURCE_NOT_A_CONTAINER)",
        "throw ERROR_TARGET_INAPPLICABLE_SLOT.create(slot)",
        "throw ERROR_SOURCE_INAPPLICABLE_SLOT.create(slot)",
        "throw ERROR_TARGET_NO_CHANGES.create(slot)",
        "throw ERROR_TARGET_NO_CHANGES_KNOWN_ITEM.create(itemStack.getDisplayName(), slot)",
        "newItem.limitSize(newItem.getMaxStackSize());",
        "return changedEntities.size();",
    ] {
        assert!(
            ITEM_COMMANDS.contains(sentinel),
            "ItemCommands.java is missing sentinel: {sentinel}"
        );
    }

    for sentinel in [
        "public ItemStack createItemStack(final int count)",
        "if (count > result.getMaxStackSize())",
        "throw ERROR_STACK_TOO_BIG.create",
        "ItemStack.validateStrict(result)",
    ] {
        assert!(
            ITEM_INPUT.contains(sentinel),
            "ItemInput.java is missing sentinel: {sentinel}"
        );
    }

    assert!(SLOT_ARGUMENT.contains("SlotRanges.nameToIds(name)"));
    assert!(SLOT_ARGUMENT.contains("throw ERROR_UNKNOWN_SLOT.createWithContext(reader, name)"));
    assert!(SLOT_ARGUMENT.contains("throw ERROR_ONLY_SINGLE_SLOT_ALLOWED.createWithContext(reader, name)"));
    assert!(SLOT_RANGES.contains("addSlotRange(values, \"container.\", 0, 54)"));
    assert!(SLOT_RANGES.contains("addSlotRange(values, \"hotbar.\", 0, 9)"));
    assert!(SLOT_RANGES.contains("addSlotRange(values, \"inventory.\", 9, 27)"));
    assert!(SLOT_RANGES.contains("addSingleSlot(values, \"armor.body\", body)"));
    assert!(SLOT_RANGES.contains("addSingleSlot(values, \"player.cursor\", 499)"));
}

#[test]
#[cfg(vibecraft_has_decompiled_sources)]
fn locate_command_source_matches_java_26_1_2() {
    const LOCATE: &str =
        vibecraft_java_source!("/net/minecraft/server/commands/LocateCommand.java");

    for sentinel in [
        "Commands.literal(\"locate\")",
        ".requires(Commands.hasPermission(Commands.LEVEL_GAMEMASTERS))",
        "Commands.literal(\"structure\")",
        "ResourceOrTagKeyArgument.resourceOrTagKey(Registries.STRUCTURE)",
        "Commands.literal(\"biome\")",
        "ResourceOrTagArgument.resourceOrTag(context, Registries.BIOME)",
        "Commands.literal(\"poi\")",
        "ResourceOrTagArgument.resourceOrTag(context, Registries.POINT_OF_INTEREST_TYPE)",
        "private static final int MAX_STRUCTURE_SEARCH_RADIUS = 100;",
        "private static final int MAX_BIOME_SEARCH_RADIUS = 6400;",
        "private static final int BIOME_SAMPLE_RESOLUTION_HORIZONTAL = 32;",
        "private static final int BIOME_SAMPLE_RESOLUTION_VERTICAL = 64;",
        "private static final int POI_SEARCH_RADIUS = 256;",
        "findNearestMapStructure(serverLevel, target, sourcePos, 100, false)",
        "findClosestBiome3d(elementOrTag, sourcePos, 6400, 32, 64)",
        "findClosestWithType(resourceOrTag, sourcePos, 256, PoiManager.Occupancy.ANY)",
        "showLocateResult(source, resourceOrTag, sourcePos, nearest, \"commands.locate.structure.success\", false",
        "showLocateResult(source, elementOrTag, sourcePos, nearest, \"commands.locate.biome.success\", true",
        "showLocateResult(source, resourceOrTag, sourcePos, closestWithType.get().swap(), \"commands.locate.poi.success\", false",
        "return distance;",
    ] {
        assert!(
            LOCATE.contains(sentinel),
            "LocateCommand.java is missing sentinel: {sentinel}"
        );
    }
}

#[test]
#[cfg(vibecraft_has_decompiled_sources)]
fn loot_command_source_matches_java_26_1_2() {
    const LOOT: &str = vibecraft_java_source!("/net/minecraft/server/commands/LootCommand.java");

    for sentinel in [
        "Commands.literal(\"loot\")",
        ".requires(Commands.hasPermission(Commands.LEVEL_GAMEMASTERS))",
        "Commands.literal(\"replace\")",
        "Commands.literal(\"entity\")",
        "Commands.literal(\"block\")",
        "Commands.literal(\"insert\")",
        "Commands.literal(\"give\")",
        "Commands.literal(\"spawn\")",
        "Commands.literal(\"fish\")",
        "Commands.literal(\"kill\")",
        "Commands.literal(\"mine\")",
        "ResourceOrIdArgument.lootTable(context)",
        "ItemArgument.item(context)",
        "SlotArgument.slot()",
        "IntegerArgumentType.integer(0)",
        "Vec3Argument.vec3()",
        "private static Container getContainer(final CommandSourceStack source, final BlockPos pos)",
        "throw ItemCommands.ERROR_TARGET_NOT_A_CONTAINER.create",
        "private static int blockDistribute",
        "if (distributeToContainer(container, drop.copy()))",
        "private static boolean distributeToContainer",
        "if (current.isEmpty())",
        "if (canMergeItems(current, itemStack))",
        "private static int blockReplace",
        "throw ItemCommands.ERROR_TARGET_INAPPLICABLE_SLOT.create(startSlot)",
        "usedItems.add(toAdd);",
        "private static int playerGive",
        "if (player.getInventory().add(drop.copy()))",
        "private static void setSlots",
        "ItemStack item = i < itemsToSet.size() ? itemsToSet.get(i) : ItemStack.EMPTY;",
        "private static int entityReplace",
        "private static int dropInWorld",
        "commands.drop.success.single",
        "commands.drop.success.multiple",
        "commands.drop.success.single_with_table",
        "commands.drop.success.multiple_with_table",
        "private static ItemStack getSourceHandItem",
        "throw ERROR_NO_HELD_ITEMS.create",
        "private static int dropBlockLoot",
        "throw ERROR_NO_BLOCK_LOOT_TABLE.create",
        "withParameter(LootContextParams.BLOCK_STATE, blockState)",
        "withOptionalParameter(LootContextParams.BLOCK_ENTITY, blockEntity)",
        "withParameter(LootContextParams.TOOL, tool)",
        "private static int dropKillLoot",
        "throw ERROR_NO_ENTITY_LOOT_TABLE.create",
        "withParameter(LootContextParams.DAMAGE_SOURCE, target.damageSources().magic())",
        "private static int dropChestLoot",
        "create(LootContextParamSets.CHEST)",
        "private static int dropFishingLoot",
        "create(LootContextParamSets.FISHING)",
        "return output.accept(context, drops, usedItems -> callback(source, usedItems));",
    ] {
        assert!(
            LOOT.contains(sentinel),
            "LootCommand.java is missing sentinel: {sentinel}"
        );
    }
}

#[test]
#[cfg(vibecraft_has_decompiled_sources)]
fn place_command_source_matches_java_26_1_2() {
    const PLACE: &str = vibecraft_java_source!("/net/minecraft/server/commands/PlaceCommand.java");

    for sentinel in [
        "Commands.literal(\"place\")",
        ".requires(Commands.hasPermission(Commands.LEVEL_GAMEMASTERS))",
        "Commands.literal(\"feature\")",
        "ResourceKeyArgument.key(Registries.CONFIGURED_FEATURE)",
        "BlockPos.containing(((CommandSourceStack)c.getSource()).getPosition())",
        "BlockPosArgument.getLoadedBlockPos(c, \"pos\")",
        "Commands.literal(\"jigsaw\")",
        "ResourceKeyArgument.key(Registries.TEMPLATE_POOL)",
        "IdentifierArgument.id()",
        "IntegerArgumentType.integer(1, 20)",
        "JigsawPlacement.generateJigsaw(level, pool, target, maxDepth, pos, false)",
        "throw ERROR_JIGSAW_FAILED.create();",
        "Commands.literal(\"structure\")",
        "ResourceKeyArgument.key(Registries.STRUCTURE)",
        "structure.generate(",
        "throw ERROR_STRUCTURE_FAILED.create();",
        "Commands.literal(\"template\")",
        "Commands.argument(\"template\", IdentifierArgument.id())",
        ".suggests(SUGGEST_TEMPLATES)",
        "TemplateRotationArgument.templateRotation()",
        "TemplateMirrorArgument.templateMirror()",
        "FloatArgumentType.floatArg(0.0F, 1.0F)",
        "IntegerArgumentType.integer()",
        "Commands.literal(\"strict\")",
        "throw ERROR_TEMPLATE_INVALID.create(template);",
        "new StructurePlaceSettings().setMirror(mirror).setRotation(rotation).setKnownShape(strict)",
        "new BlockRotProcessor(integrity)",
        "StructureBlockEntity.createRandom(seed)",
        "structureTemplate.placeInWorld(level, pos, pos, placeSettings, StructureBlockEntity.createRandom(seed), 2 | (strict ? 816 : 0))",
        "throw ERROR_TEMPLATE_FAILED.create();",
        "commands.place.feature.success",
        "commands.place.jigsaw.success",
        "commands.place.structure.success",
        "commands.place.template.success",
        "private static void checkLoaded(final ServerLevel level, final ChunkPos chunkMin, final ChunkPos chunkMax)",
        "BlockPosArgument.ERROR_NOT_LOADED.create()",
        "return 1;",
    ] {
        assert!(
            PLACE.contains(sentinel),
            "PlaceCommand.java is missing sentinel: {sentinel}"
        );
    }
}
