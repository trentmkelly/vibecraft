# Java Class-Level Port Coverage Checklist

Generated from `decompiled-server-26.1.2/net/minecraft` on 2026-05-24. This file intentionally contains one unchecked task per Java source file so full-port planning can prove every decompiled class has an explicit Rust port/parity-test placeholder. Do not mark an item complete until the corresponding behavior has been audited against Java and covered by Rust implementation tests or an explicit deferral note.

## `decompiled-server-26.1.2/net/minecraft`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/CharPredicate.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/ChatFormatting.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/CrashReport.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/CrashReportCategory.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/CrashReportDetail.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/DefaultUncaughtExceptionHandler.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/DefaultUncaughtExceptionHandlerWithName.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/DetectedVersion.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/IdentifierException.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/Optionull.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/ReportType.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/ReportedException.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/SharedConstants.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/SuppressForbidden.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/SystemReport.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/TracingExecutor.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/WorldVersion.java`.

## `decompiled-server-26.1.2/net/minecraft/advancements`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/advancements/Advancement.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/advancements/AdvancementHolder.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/advancements/AdvancementNode.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/advancements/AdvancementProgress.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/advancements/AdvancementRequirements.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/advancements/AdvancementRewards.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/advancements/AdvancementTree.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/advancements/AdvancementType.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/advancements/CriteriaTriggers.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/advancements/Criterion.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/advancements/CriterionProgress.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/advancements/CriterionTrigger.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/advancements/CriterionTriggerInstance.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/advancements/DisplayInfo.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/advancements/TreeNodePosition.java`.

## `decompiled-server-26.1.2/net/minecraft/advancements/criterion`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/advancements/criterion/AnyBlockInteractionTrigger.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/advancements/criterion/BeeNestDestroyedTrigger.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/advancements/criterion/BlockPredicate.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/advancements/criterion/BredAnimalsTrigger.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/advancements/criterion/BrewedPotionTrigger.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/advancements/criterion/ChangeDimensionTrigger.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/advancements/criterion/ChanneledLightningTrigger.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/advancements/criterion/CollectionContentsPredicate.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/advancements/criterion/CollectionCountsPredicate.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/advancements/criterion/CollectionPredicate.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/advancements/criterion/ConstructBeaconTrigger.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/advancements/criterion/ConsumeItemTrigger.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/advancements/criterion/ContextAwarePredicate.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/advancements/criterion/CuredZombieVillagerTrigger.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/advancements/criterion/DamagePredicate.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/advancements/criterion/DamageSourcePredicate.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/advancements/criterion/DataComponentMatchers.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/advancements/criterion/DefaultBlockInteractionTrigger.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/advancements/criterion/DistancePredicate.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/advancements/criterion/DistanceTrigger.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/advancements/criterion/EffectsChangedTrigger.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/advancements/criterion/EnchantedItemTrigger.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/advancements/criterion/EnchantmentPredicate.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/advancements/criterion/EnterBlockTrigger.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/advancements/criterion/EntityEquipmentPredicate.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/advancements/criterion/EntityFlagsPredicate.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/advancements/criterion/EntityHurtPlayerTrigger.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/advancements/criterion/EntityPredicate.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/advancements/criterion/EntitySubPredicate.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/advancements/criterion/EntitySubPredicates.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/advancements/criterion/EntityTypePredicate.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/advancements/criterion/FallAfterExplosionTrigger.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/advancements/criterion/FilledBucketTrigger.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/advancements/criterion/FishingHookPredicate.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/advancements/criterion/FishingRodHookedTrigger.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/advancements/criterion/FluidPredicate.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/advancements/criterion/FoodPredicate.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/advancements/criterion/GameTypePredicate.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/advancements/criterion/ImpossibleTrigger.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/advancements/criterion/InputPredicate.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/advancements/criterion/InventoryChangeTrigger.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/advancements/criterion/ItemDurabilityTrigger.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/advancements/criterion/ItemPredicate.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/advancements/criterion/ItemUsedOnLocationTrigger.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/advancements/criterion/KilledByArrowTrigger.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/advancements/criterion/KilledTrigger.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/advancements/criterion/LevitationTrigger.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/advancements/criterion/LightPredicate.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/advancements/criterion/LightningBoltPredicate.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/advancements/criterion/LightningStrikeTrigger.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/advancements/criterion/LocationPredicate.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/advancements/criterion/LootTableTrigger.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/advancements/criterion/MinMaxBounds.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/advancements/criterion/MobEffectsPredicate.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/advancements/criterion/MovementPredicate.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/advancements/criterion/NbtPredicate.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/advancements/criterion/PickedUpItemTrigger.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/advancements/criterion/PlayerHurtEntityTrigger.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/advancements/criterion/PlayerInteractTrigger.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/advancements/criterion/PlayerPredicate.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/advancements/criterion/PlayerTrigger.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/advancements/criterion/RaiderPredicate.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/advancements/criterion/RecipeCraftedTrigger.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/advancements/criterion/RecipeUnlockedTrigger.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/advancements/criterion/SheepPredicate.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/advancements/criterion/ShotCrossbowTrigger.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/advancements/criterion/SimpleCriterionTrigger.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/advancements/criterion/SingleComponentItemPredicate.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/advancements/criterion/SlideDownBlockTrigger.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/advancements/criterion/SlimePredicate.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/advancements/criterion/SlotsPredicate.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/advancements/criterion/SpearMobsTrigger.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/advancements/criterion/StartRidingTrigger.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/advancements/criterion/StatePropertiesPredicate.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/advancements/criterion/SummonedEntityTrigger.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/advancements/criterion/TagPredicate.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/advancements/criterion/TameAnimalTrigger.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/advancements/criterion/TargetBlockTrigger.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/advancements/criterion/TradeTrigger.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/advancements/criterion/UsedEnderEyeTrigger.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/advancements/criterion/UsedTotemTrigger.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/advancements/criterion/UsingItemTrigger.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/advancements/criterion/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/advancements`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/advancements/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/commands`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/commands/ArgumentVisitor.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/commands/BrigadierExceptions.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/commands/CacheableFunction.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/commands/CommandBuildContext.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/commands/CommandResultCallback.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/commands/CommandSigningContext.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/commands/CommandSource.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/commands/CommandSourceStack.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/commands/Commands.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/commands/ExecutionCommandSource.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/commands/FunctionInstantiationException.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/commands/ParserUtils.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/commands/SharedSuggestionProvider.java`.

## `decompiled-server-26.1.2/net/minecraft/commands/arguments`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/commands/arguments/AngleArgument.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/commands/arguments/ArgumentSignatures.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/commands/arguments/ColorArgument.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/commands/arguments/ComponentArgument.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/commands/arguments/CompoundTagArgument.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/commands/arguments/DimensionArgument.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/commands/arguments/EntityAnchorArgument.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/commands/arguments/EntityArgument.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/commands/arguments/GameModeArgument.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/commands/arguments/GameProfileArgument.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/commands/arguments/HeightmapTypeArgument.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/commands/arguments/HexColorArgument.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/commands/arguments/IdentifierArgument.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/commands/arguments/MessageArgument.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/commands/arguments/NbtPathArgument.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/commands/arguments/NbtTagArgument.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/commands/arguments/ObjectiveArgument.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/commands/arguments/ObjectiveCriteriaArgument.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/commands/arguments/OperationArgument.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/commands/arguments/ParticleArgument.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/commands/arguments/RangeArgument.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/commands/arguments/ResourceArgument.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/commands/arguments/ResourceKeyArgument.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/commands/arguments/ResourceOrIdArgument.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/commands/arguments/ResourceOrTagArgument.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/commands/arguments/ResourceOrTagKeyArgument.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/commands/arguments/ResourceSelectorArgument.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/commands/arguments/ScoreHolderArgument.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/commands/arguments/ScoreboardSlotArgument.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/commands/arguments/SignedArgument.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/commands/arguments/SlotArgument.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/commands/arguments/SlotsArgument.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/commands/arguments/StringRepresentableArgument.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/commands/arguments/StyleArgument.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/commands/arguments/TeamArgument.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/commands/arguments/TemplateMirrorArgument.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/commands/arguments/TemplateRotationArgument.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/commands/arguments/TimeArgument.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/commands/arguments/UuidArgument.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/commands/arguments/WaypointArgument.java`.

## `decompiled-server-26.1.2/net/minecraft/commands/arguments/blocks`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/commands/arguments/blocks/BlockInput.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/commands/arguments/blocks/BlockPredicateArgument.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/commands/arguments/blocks/BlockStateArgument.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/commands/arguments/blocks/BlockStateParser.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/commands/arguments/blocks/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/commands/arguments/coordinates`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/commands/arguments/coordinates/BlockPosArgument.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/commands/arguments/coordinates/ColumnPosArgument.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/commands/arguments/coordinates/Coordinates.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/commands/arguments/coordinates/LocalCoordinates.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/commands/arguments/coordinates/RotationArgument.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/commands/arguments/coordinates/SwizzleArgument.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/commands/arguments/coordinates/Vec2Argument.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/commands/arguments/coordinates/Vec3Argument.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/commands/arguments/coordinates/WorldCoordinate.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/commands/arguments/coordinates/WorldCoordinates.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/commands/arguments/coordinates/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/commands/arguments/item`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/commands/arguments/item/ComponentPredicateParser.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/commands/arguments/item/FunctionArgument.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/commands/arguments/item/ItemArgument.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/commands/arguments/item/ItemInput.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/commands/arguments/item/ItemParser.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/commands/arguments/item/ItemPredicateArgument.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/commands/arguments/item/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/commands/arguments`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/commands/arguments/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/commands/arguments/selector`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/commands/arguments/selector/EntitySelector.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/commands/arguments/selector/EntitySelectorParser.java`.

## `decompiled-server-26.1.2/net/minecraft/commands/arguments/selector/options`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/commands/arguments/selector/options/EntitySelectorOptions.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/commands/arguments/selector/options/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/commands/arguments/selector`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/commands/arguments/selector/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/commands/execution`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/commands/execution/ChainModifiers.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/commands/execution/CommandQueueEntry.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/commands/execution/CustomCommandExecutor.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/commands/execution/CustomModifierExecutor.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/commands/execution/EntryAction.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/commands/execution/ExecutionContext.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/commands/execution/ExecutionControl.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/commands/execution/Frame.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/commands/execution/TraceCallbacks.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/commands/execution/UnboundEntryAction.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/commands/execution/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/commands/execution/tasks`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/commands/execution/tasks/BuildContexts.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/commands/execution/tasks/CallFunction.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/commands/execution/tasks/ContinuationTask.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/commands/execution/tasks/ExecuteCommand.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/commands/execution/tasks/FallthroughTask.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/commands/execution/tasks/IsolatedCall.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/commands/execution/tasks/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/commands/functions`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/commands/functions/CommandFunction.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/commands/functions/FunctionBuilder.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/commands/functions/InstantiatedFunction.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/commands/functions/MacroFunction.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/commands/functions/PlainTextFunction.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/commands/functions/StringTemplate.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/commands/functions/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/commands`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/commands/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/commands/synchronization`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/commands/synchronization/ArgumentTypeInfo.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/commands/synchronization/ArgumentTypeInfos.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/commands/synchronization/ArgumentUtils.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/commands/synchronization/SingletonArgumentInfo.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/commands/synchronization/SuggestionProviders.java`.

## `decompiled-server-26.1.2/net/minecraft/commands/synchronization/brigadier`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/commands/synchronization/brigadier/DoubleArgumentInfo.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/commands/synchronization/brigadier/FloatArgumentInfo.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/commands/synchronization/brigadier/IntegerArgumentInfo.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/commands/synchronization/brigadier/LongArgumentInfo.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/commands/synchronization/brigadier/StringArgumentSerializer.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/commands/synchronization/brigadier/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/commands/synchronization`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/commands/synchronization/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/core`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/core/AxisCycle.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/core/BlockBox.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/core/BlockMath.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/core/BlockPos.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/core/ClientAsset.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/core/Cloner.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/core/Cursor3D.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/core/DefaultedMappedRegistry.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/core/DefaultedRegistry.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/core/Direction.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/core/Direction8.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/core/FrontAndTop.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/core/GlobalPos.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/core/Holder.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/core/HolderGetter.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/core/HolderLookup.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/core/HolderOwner.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/core/HolderSet.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/core/IdMap.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/core/IdMapper.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/core/LayeredRegistryAccess.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/core/MappedRegistry.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/core/NonNullList.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/core/Position.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/core/QuartPos.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/core/RegistrationInfo.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/core/Registry.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/core/RegistryAccess.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/core/RegistryCodecs.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/core/RegistrySetBuilder.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/core/RegistrySynchronization.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/core/Rotations.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/core/SectionPos.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/core/TypedInstance.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/core/UUIDUtil.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/core/Vec3i.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/core/WritableRegistry.java`.

## `decompiled-server-26.1.2/net/minecraft/core/cauldron`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/core/cauldron/CauldronInteraction.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/core/cauldron/CauldronInteractions.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/core/cauldron/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/core/component`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/core/component/DataComponentExactPredicate.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/core/component/DataComponentGetter.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/core/component/DataComponentHolder.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/core/component/DataComponentInitializers.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/core/component/DataComponentLookup.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/core/component/DataComponentMap.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/core/component/DataComponentPatch.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/core/component/DataComponentType.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/core/component/DataComponents.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/core/component/PatchedDataComponentMap.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/core/component/TypedDataComponent.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/core/component/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/core/component/predicates`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/core/component/predicates/AnyValue.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/core/component/predicates/AttributeModifiersPredicate.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/core/component/predicates/BundlePredicate.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/core/component/predicates/ContainerPredicate.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/core/component/predicates/CustomDataPredicate.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/core/component/predicates/DamagePredicate.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/core/component/predicates/DataComponentPredicate.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/core/component/predicates/DataComponentPredicates.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/core/component/predicates/EnchantmentsPredicate.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/core/component/predicates/FireworkExplosionPredicate.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/core/component/predicates/FireworksPredicate.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/core/component/predicates/JukeboxPlayablePredicate.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/core/component/predicates/PotionsPredicate.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/core/component/predicates/TrimPredicate.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/core/component/predicates/VillagerTypePredicate.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/core/component/predicates/WritableBookPredicate.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/core/component/predicates/WrittenBookPredicate.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/core/component/predicates/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/core/dispenser`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/core/dispenser/BlockSource.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/core/dispenser/BoatDispenseItemBehavior.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/core/dispenser/DefaultDispenseItemBehavior.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/core/dispenser/DispenseItemBehavior.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/core/dispenser/EquipmentDispenseItemBehavior.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/core/dispenser/MinecartDispenseItemBehavior.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/core/dispenser/OptionalDispenseItemBehavior.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/core/dispenser/ProjectileDispenseBehavior.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/core/dispenser/ShearsDispenseItemBehavior.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/core/dispenser/ShulkerBoxDispenseBehavior.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/core/dispenser/SpawnEggItemBehavior.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/core/dispenser/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/core`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/core/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/core/particles`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/core/particles/BlockParticleOption.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/core/particles/ColorParticleOption.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/core/particles/DustColorTransitionOptions.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/core/particles/DustParticleOptions.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/core/particles/ExplosionParticleInfo.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/core/particles/ItemParticleOption.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/core/particles/ParticleLimit.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/core/particles/ParticleOptions.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/core/particles/ParticleType.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/core/particles/ParticleTypes.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/core/particles/PowerParticleOption.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/core/particles/ScalableParticleOptionsBase.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/core/particles/SculkChargeParticleOptions.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/core/particles/ShriekParticleOption.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/core/particles/SimpleParticleType.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/core/particles/SpellParticleOption.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/core/particles/TrailParticleOption.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/core/particles/VibrationParticleOption.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/core/particles/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/core/registries`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/core/registries/BuiltInRegistries.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/core/registries/ConcurrentHolderGetter.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/core/registries/Registries.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/core/registries/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/data`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/AtlasIds.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/BlockFamilies.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/BlockFamily.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/CachedOutput.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/DataGenerator.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/DataProvider.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/HashCache.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/Main.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/PackOutput.java`.

## `decompiled-server-26.1.2/net/minecraft/data/advancements`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/advancements/AdvancementProvider.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/advancements/AdvancementSubProvider.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/advancements/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/data/advancements/packs`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/advancements/packs/VanillaAdvancementProvider.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/advancements/packs/VanillaAdventureAdvancements.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/advancements/packs/VanillaHusbandryAdvancements.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/advancements/packs/VanillaNetherAdvancements.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/advancements/packs/VanillaStoryAdvancements.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/advancements/packs/VanillaTheEndAdvancements.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/advancements/packs/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/data/info`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/info/BiomeParametersDumpReport.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/info/BlockListReport.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/info/CommandsReport.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/info/DatapackStructureReport.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/info/PacketReport.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/info/RegistryComponentsReport.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/info/RegistryDumpReport.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/info/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/data/loot`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/loot/BlockLootSubProvider.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/loot/EntityLootSubProvider.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/loot/LootTableProvider.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/loot/LootTableSubProvider.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/loot/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/data/loot/packs`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/loot/packs/LootData.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/loot/packs/TradeRebalanceChestLoot.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/loot/packs/TradeRebalanceLootTableProvider.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/loot/packs/VanillaArchaeologyLoot.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/loot/packs/VanillaBlockInteractLoot.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/loot/packs/VanillaBlockLoot.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/loot/packs/VanillaChargedCreeperExplosionLoot.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/loot/packs/VanillaChestLoot.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/loot/packs/VanillaEntityInteractLoot.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/loot/packs/VanillaEntityLoot.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/loot/packs/VanillaEquipmentLoot.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/loot/packs/VanillaFishingLoot.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/loot/packs/VanillaGiftLoot.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/loot/packs/VanillaLootTableProvider.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/loot/packs/VanillaPiglinBarterLoot.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/loot/packs/VanillaShearingLoot.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/loot/packs/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/data/metadata`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/metadata/PackMetadataGenerator.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/metadata/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/data`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/data/recipes`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/recipes/CustomCraftingRecipeBuilder.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/recipes/RecipeBuilder.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/recipes/RecipeCategory.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/recipes/RecipeOutput.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/recipes/RecipeProvider.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/recipes/RecipeUnlockAdvancementBuilder.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/recipes/ShapedRecipeBuilder.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/recipes/ShapelessRecipeBuilder.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/recipes/SimpleCookingRecipeBuilder.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/recipes/SingleItemRecipeBuilder.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/recipes/SmithingTransformRecipeBuilder.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/recipes/SmithingTrimRecipeBuilder.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/recipes/SpecialRecipeBuilder.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/recipes/TransmuteRecipeBuilder.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/recipes/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/data/recipes/packs`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/recipes/packs/VanillaRecipeProvider.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/recipes/packs/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/data/registries`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/registries/RegistriesDatapackGenerator.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/registries/RegistryPatchGenerator.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/registries/TradeRebalanceRegistries.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/registries/VanillaRegistries.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/registries/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/data/structures`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/structures/NbtToSnbt.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/structures/SnbtDatafixer.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/structures/SnbtToNbt.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/structures/StructureUpdater.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/structures/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/data/tags`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/tags/BannerPatternTagsProvider.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/tags/BiomeTagsProvider.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/tags/BlockItemTagsProvider.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/tags/DamageTypeTagsProvider.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/tags/DialogTagsProvider.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/tags/EnchantmentTagsProvider.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/tags/EntityTypeTagsProvider.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/tags/FeatureTagsProvider.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/tags/FlatLevelGeneratorPresetTagsProvider.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/tags/FluidTagsProvider.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/tags/GameEventTagsProvider.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/tags/HolderTagProvider.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/tags/InstrumentTagsProvider.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/tags/IntrinsicHolderTagsProvider.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/tags/KeyTagProvider.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/tags/PaintingVariantTagsProvider.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/tags/PoiTypeTagsProvider.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/tags/PotionTagsProvider.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/tags/StructureTagsProvider.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/tags/TagAppender.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/tags/TagsProvider.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/tags/TimelineTagsProvider.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/tags/TradeRebalanceEnchantmentTagsProvider.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/tags/TradeRebalanceTradeTagsProvider.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/tags/VanillaBlockTagsProvider.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/tags/VanillaEnchantmentTagsProvider.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/tags/VanillaItemTagsProvider.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/tags/VillagerTradesTagsProvider.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/tags/WorldPresetTagsProvider.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/tags/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/data/worldgen`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/worldgen/AncientCityStructurePieces.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/worldgen/AncientCityStructurePools.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/worldgen/BastionBridgePools.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/worldgen/BastionHoglinStablePools.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/worldgen/BastionHousingUnitsPools.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/worldgen/BastionPieces.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/worldgen/BastionSharedPools.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/worldgen/BastionTreasureRoomPools.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/worldgen/BiomeDefaultFeatures.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/worldgen/BootstrapContext.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/worldgen/Carvers.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/worldgen/DesertVillagePools.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/worldgen/DimensionTypes.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/worldgen/NoiseData.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/worldgen/PillagerOutpostPools.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/worldgen/PlainVillagePools.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/worldgen/Pools.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/worldgen/ProcessorLists.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/worldgen/SavannaVillagePools.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/worldgen/SnowyVillagePools.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/worldgen/StructureSets.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/worldgen/Structures.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/worldgen/SurfaceRuleData.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/worldgen/TaigaVillagePools.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/worldgen/TerrainProvider.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/worldgen/TrailRuinsStructurePools.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/worldgen/TrialChambersStructurePools.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/worldgen/VillagePools.java`.

## `decompiled-server-26.1.2/net/minecraft/data/worldgen/biome`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/worldgen/biome/BiomeData.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/worldgen/biome/EndBiomes.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/worldgen/biome/NetherBiomes.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/worldgen/biome/OverworldBiomes.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/worldgen/biome/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/data/worldgen/features`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/worldgen/features/AquaticFeatures.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/worldgen/features/CaveFeatures.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/worldgen/features/EndFeatures.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/worldgen/features/FeatureUtils.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/worldgen/features/MiscOverworldFeatures.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/worldgen/features/NetherFeatures.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/worldgen/features/OreFeatures.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/worldgen/features/PileFeatures.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/worldgen/features/TreeFeatures.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/worldgen/features/VegetationFeatures.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/worldgen/features/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/data/worldgen`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/worldgen/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/data/worldgen/placement`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/worldgen/placement/AquaticPlacements.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/worldgen/placement/CavePlacements.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/worldgen/placement/EndPlacements.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/worldgen/placement/MiscOverworldPlacements.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/worldgen/placement/NetherPlacements.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/worldgen/placement/OrePlacements.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/worldgen/placement/PlacementUtils.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/worldgen/placement/TreePlacements.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/worldgen/placement/VegetationPlacements.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/worldgen/placement/VillagePlacements.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/data/worldgen/placement/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/gametest`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/gametest/Main.java`.

## `decompiled-server-26.1.2/net/minecraft/gametest/framework`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/gametest/framework/BlockBasedTestInstance.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/gametest/framework/BuiltinTestFunctions.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/gametest/framework/ExhaustedAttemptsException.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/gametest/framework/FailedTestTracker.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/gametest/framework/FunctionGameTestInstance.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/gametest/framework/GameTestAssertException.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/gametest/framework/GameTestAssertPosException.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/gametest/framework/GameTestBatch.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/gametest/framework/GameTestBatchFactory.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/gametest/framework/GameTestBatchListener.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/gametest/framework/GameTestEnvironments.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/gametest/framework/GameTestEvent.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/gametest/framework/GameTestException.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/gametest/framework/GameTestHelper.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/gametest/framework/GameTestInfo.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/gametest/framework/GameTestInstance.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/gametest/framework/GameTestInstances.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/gametest/framework/GameTestListener.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/gametest/framework/GameTestMainUtil.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/gametest/framework/GameTestRunner.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/gametest/framework/GameTestSequence.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/gametest/framework/GameTestServer.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/gametest/framework/GameTestTicker.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/gametest/framework/GameTestTimeoutException.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/gametest/framework/GeneratedTest.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/gametest/framework/GlobalTestReporter.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/gametest/framework/JUnitLikeTestReporter.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/gametest/framework/LogTestReporter.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/gametest/framework/MultipleTestTracker.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/gametest/framework/ReportGameListener.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/gametest/framework/RetryOptions.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/gametest/framework/StructureGridSpawner.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/gametest/framework/StructureUtils.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/gametest/framework/TestCommand.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/gametest/framework/TestData.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/gametest/framework/TestEnvironmentDefinition.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/gametest/framework/TestFinder.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/gametest/framework/TestFunctionLoader.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/gametest/framework/TestInstanceFinder.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/gametest/framework/TestPosFinder.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/gametest/framework/TestReporter.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/gametest/framework/UnknownGameTestException.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/gametest/framework/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/gametest`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/gametest/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/gizmos`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/gizmos/ArrowGizmo.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/gizmos/CircleGizmo.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/gizmos/CuboidGizmo.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/gizmos/Gizmo.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/gizmos/GizmoCollector.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/gizmos/GizmoPrimitives.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/gizmos/GizmoProperties.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/gizmos/GizmoStyle.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/gizmos/Gizmos.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/gizmos/LineGizmo.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/gizmos/PointGizmo.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/gizmos/RectGizmo.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/gizmos/SimpleGizmoCollector.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/gizmos/TextGizmo.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/gizmos/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/locale`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/locale/DeprecatedTranslationsInfo.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/locale/Language.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/locale/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/nbt`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/nbt/ByteArrayTag.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/nbt/ByteTag.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/nbt/CollectionTag.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/nbt/CompoundTag.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/nbt/DoubleTag.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/nbt/EndTag.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/nbt/FloatTag.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/nbt/IntArrayTag.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/nbt/IntTag.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/nbt/ListTag.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/nbt/LongArrayTag.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/nbt/LongTag.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/nbt/NbtAccounter.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/nbt/NbtAccounterException.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/nbt/NbtException.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/nbt/NbtFormatException.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/nbt/NbtIo.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/nbt/NbtOps.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/nbt/NbtUtils.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/nbt/NumericTag.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/nbt/PrimitiveTag.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/nbt/ReportedNbtException.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/nbt/ShortTag.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/nbt/SnbtGrammar.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/nbt/SnbtOperations.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/nbt/SnbtPrinterTagVisitor.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/nbt/StreamTagVisitor.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/nbt/StringTag.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/nbt/StringTagVisitor.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/nbt/Tag.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/nbt/TagParser.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/nbt/TagType.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/nbt/TagTypes.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/nbt/TagVisitor.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/nbt/TextComponentTagVisitor.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/nbt/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/nbt/visitors`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/nbt/visitors/CollectFields.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/nbt/visitors/CollectToTag.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/nbt/visitors/FieldSelector.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/nbt/visitors/FieldTree.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/nbt/visitors/SkipAll.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/nbt/visitors/SkipFields.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/nbt/visitors/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/network`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/BandwidthDebugMonitor.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/CipherBase.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/CipherDecoder.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/CipherEncoder.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/ClientboundPacketListener.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/CompressionDecoder.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/CompressionEncoder.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/Connection.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/ConnectionProtocol.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/DisconnectionDetails.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/FriendlyByteBuf.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/HandlerNames.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/HashedPatchMap.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/HashedStack.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/HiddenByteBuf.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/LocalFrameDecoder.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/LocalFrameEncoder.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/LpVec3.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/MonitoredLocalFrameDecoder.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/PacketBundlePacker.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/PacketBundleUnpacker.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/PacketDecoder.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/PacketEncoder.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/PacketListener.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/PacketProcessor.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/PacketSendListener.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/ProtocolInfo.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/ProtocolSwapHandler.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/RateKickingConnection.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/RegistryFriendlyByteBuf.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/ServerboundPacketListener.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/SkipPacketDecoderException.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/SkipPacketEncoderException.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/SkipPacketException.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/TickablePacketListener.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/UnconfiguredPipelineHandler.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/Utf8String.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/VarInt.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/VarLong.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/Varint21FrameDecoder.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/Varint21LengthFieldPrepender.java`.

## `decompiled-server-26.1.2/net/minecraft/network/chat`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/chat/ChatDecorator.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/chat/ChatType.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/chat/ChatTypeDecoration.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/chat/ClickEvent.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/chat/CommonComponents.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/chat/Component.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/chat/ComponentContents.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/chat/ComponentSerialization.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/chat/ComponentUtils.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/chat/FilterMask.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/chat/FontDescription.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/chat/FormattedText.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/chat/HoverEvent.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/chat/LastSeenMessages.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/chat/LastSeenMessagesTracker.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/chat/LastSeenMessagesValidator.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/chat/LastSeenTrackedEntry.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/chat/LocalChatSession.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/chat/MessageSignature.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/chat/MessageSignatureCache.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/chat/MutableComponent.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/chat/OutgoingChatMessage.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/chat/PlayerChatMessage.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/chat/RemoteChatSession.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/chat/ResolutionContext.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/chat/SignableCommand.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/chat/SignedMessageBody.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/chat/SignedMessageChain.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/chat/SignedMessageLink.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/chat/SignedMessageValidator.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/chat/Style.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/chat/SubStringSource.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/chat/TextColor.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/chat/ThrowingComponent.java`.

## `decompiled-server-26.1.2/net/minecraft/network/chat/contents`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/chat/contents/KeybindContents.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/chat/contents/KeybindResolver.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/chat/contents/NbtContents.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/chat/contents/ObjectContents.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/chat/contents/PlainTextContents.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/chat/contents/ScoreContents.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/chat/contents/SelectorContents.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/chat/contents/TranslatableContents.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/chat/contents/TranslatableFormatException.java`.

## `decompiled-server-26.1.2/net/minecraft/network/chat/contents/data`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/chat/contents/data/BlockDataSource.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/chat/contents/data/DataSource.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/chat/contents/data/DataSources.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/chat/contents/data/EntityDataSource.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/chat/contents/data/StorageDataSource.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/chat/contents/data/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/network/chat/contents/objects`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/chat/contents/objects/AtlasSprite.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/chat/contents/objects/ObjectInfo.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/chat/contents/objects/ObjectInfos.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/chat/contents/objects/PlayerSprite.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/chat/contents/objects/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/network/chat/contents`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/chat/contents/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/network/chat/numbers`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/chat/numbers/BlankFormat.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/chat/numbers/FixedFormat.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/chat/numbers/NumberFormat.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/chat/numbers/NumberFormatType.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/chat/numbers/NumberFormatTypes.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/chat/numbers/StyledFormat.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/chat/numbers/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/network/chat`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/chat/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/network/codec`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/codec/ByteBufCodecs.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/codec/IdDispatchCodec.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/codec/StreamCodec.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/codec/StreamDecoder.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/codec/StreamEncoder.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/codec/StreamMemberEncoder.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/codec/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/network`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/network/protocol`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/BundleDelimiterPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/BundlePacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/BundlerInfo.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/CodecModifier.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/Packet.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/PacketFlow.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/PacketType.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/PacketUtils.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/ProtocolCodecBuilder.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/ProtocolInfoBuilder.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/SimpleUnboundProtocol.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/UnboundProtocol.java`.

## `decompiled-server-26.1.2/net/minecraft/network/protocol/common`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/common/ClientCommonPacketListener.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/common/ClientboundClearDialogPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/common/ClientboundCustomPayloadPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/common/ClientboundCustomReportDetailsPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/common/ClientboundDisconnectPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/common/ClientboundKeepAlivePacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/common/ClientboundPingPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/common/ClientboundResourcePackPopPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/common/ClientboundResourcePackPushPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/common/ClientboundServerLinksPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/common/ClientboundShowDialogPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/common/ClientboundStoreCookiePacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/common/ClientboundTransferPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/common/ClientboundUpdateTagsPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/common/CommonPacketTypes.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/common/ServerCommonPacketListener.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/common/ServerboundClientInformationPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/common/ServerboundCustomClickActionPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/common/ServerboundCustomPayloadPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/common/ServerboundKeepAlivePacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/common/ServerboundPongPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/common/ServerboundResourcePackPacket.java`.

## `decompiled-server-26.1.2/net/minecraft/network/protocol/common/custom`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/common/custom/BrandPayload.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/common/custom/CustomPacketPayload.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/common/custom/DiscardedPayload.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/common/custom/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/network/protocol/common`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/common/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/network/protocol/configuration`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/configuration/ClientConfigurationPacketListener.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/configuration/ClientboundCodeOfConductPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/configuration/ClientboundFinishConfigurationPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/configuration/ClientboundRegistryDataPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/configuration/ClientboundResetChatPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/configuration/ClientboundSelectKnownPacks.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/configuration/ClientboundUpdateEnabledFeaturesPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/configuration/ConfigurationPacketTypes.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/configuration/ConfigurationProtocols.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/configuration/ServerConfigurationPacketListener.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/configuration/ServerboundAcceptCodeOfConductPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/configuration/ServerboundFinishConfigurationPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/configuration/ServerboundSelectKnownPacks.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/configuration/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/network/protocol/cookie`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/cookie/ClientCookiePacketListener.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/cookie/ClientboundCookieRequestPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/cookie/CookiePacketTypes.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/cookie/ServerCookiePacketListener.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/cookie/ServerboundCookieResponsePacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/cookie/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/network/protocol/game`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientGamePacketListener.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundAddEntityPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundAnimatePacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundAwardStatsPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundBlockChangedAckPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundBlockDestructionPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundBlockEntityDataPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundBlockEventPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundBlockUpdatePacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundBossEventPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundBundleDelimiterPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundBundlePacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundChangeDifficultyPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundChunkBatchFinishedPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundChunkBatchStartPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundChunksBiomesPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundClearTitlesPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundCommandSuggestionsPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundCommandsPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundContainerClosePacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundContainerSetContentPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundContainerSetDataPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundContainerSetSlotPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundCooldownPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundCustomChatCompletionsPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundDamageEventPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundDebugBlockValuePacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundDebugChunkValuePacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundDebugEntityValuePacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundDebugEventPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundDebugSamplePacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundDeleteChatPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundDisguisedChatPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundEntityEventPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundEntityPositionSyncPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundExplodePacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundForgetLevelChunkPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundGameEventPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundGameRuleValuesPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundGameTestHighlightPosPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundHurtAnimationPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundInitializeBorderPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundLevelChunkPacketData.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundLevelChunkWithLightPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundLevelEventPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundLevelParticlesPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundLightUpdatePacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundLightUpdatePacketData.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundLoginPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundLowDiskSpaceWarningPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundMapItemDataPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundMerchantOffersPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundMountScreenOpenPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundMoveEntityPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundMoveMinecartPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundMoveVehiclePacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundOpenBookPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundOpenScreenPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundOpenSignEditorPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundPlaceGhostRecipePacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundPlayerAbilitiesPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundPlayerChatPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundPlayerCombatEndPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundPlayerCombatEnterPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundPlayerCombatKillPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundPlayerInfoRemovePacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundPlayerInfoUpdatePacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundPlayerLookAtPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundPlayerPositionPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundPlayerRotationPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundProjectilePowerPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundRecipeBookAddPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundRecipeBookRemovePacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundRecipeBookSettingsPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundRemoveEntitiesPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundRemoveMobEffectPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundResetScorePacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundRespawnPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundRotateHeadPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundSectionBlocksUpdatePacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundSelectAdvancementsTabPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundServerDataPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundSetActionBarTextPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundSetBorderCenterPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundSetBorderLerpSizePacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundSetBorderSizePacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundSetBorderWarningDelayPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundSetBorderWarningDistancePacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundSetCameraPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundSetChunkCacheCenterPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundSetChunkCacheRadiusPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundSetCursorItemPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundSetDefaultSpawnPositionPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundSetDisplayObjectivePacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundSetEntityDataPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundSetEntityLinkPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundSetEntityMotionPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundSetEquipmentPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundSetExperiencePacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundSetHealthPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundSetHeldSlotPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundSetObjectivePacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundSetPassengersPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundSetPlayerInventoryPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundSetPlayerTeamPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundSetScorePacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundSetSimulationDistancePacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundSetSubtitleTextPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundSetTimePacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundSetTitleTextPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundSetTitlesAnimationPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundSoundEntityPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundSoundPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundStartConfigurationPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundStopSoundPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundSystemChatPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundTabListPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundTagQueryPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundTakeItemEntityPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundTeleportEntityPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundTestInstanceBlockStatus.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundTickingStatePacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundTickingStepPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundTrackedWaypointPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundUpdateAdvancementsPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundUpdateAttributesPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundUpdateMobEffectPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundUpdateRecipesPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/CommonPlayerSpawnInfo.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/DebugEntityNameGenerator.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/GamePacketTypes.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/GameProtocols.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ServerGamePacketListener.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ServerPacketListener.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ServerboundAcceptTeleportationPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ServerboundAttackPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ServerboundBlockEntityTagQueryPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ServerboundChangeDifficultyPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ServerboundChangeGameModePacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ServerboundChatAckPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ServerboundChatCommandPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ServerboundChatCommandSignedPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ServerboundChatPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ServerboundChatSessionUpdatePacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ServerboundChunkBatchReceivedPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ServerboundClientCommandPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ServerboundClientTickEndPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ServerboundCommandSuggestionPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ServerboundConfigurationAcknowledgedPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ServerboundContainerButtonClickPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ServerboundContainerClickPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ServerboundContainerClosePacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ServerboundContainerSlotStateChangedPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ServerboundDebugSubscriptionRequestPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ServerboundEditBookPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ServerboundEntityTagQueryPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ServerboundInteractPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ServerboundJigsawGeneratePacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ServerboundLockDifficultyPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ServerboundMovePlayerPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ServerboundMoveVehiclePacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ServerboundPaddleBoatPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ServerboundPickItemFromBlockPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ServerboundPickItemFromEntityPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ServerboundPlaceRecipePacket.java`.
- [x] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ServerboundPlayerAbilitiesPacket.java`. — Ported byte flag codec and Java `mayfly` gate for serverbound flying updates.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ServerboundPlayerActionPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ServerboundPlayerCommandPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ServerboundPlayerInputPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ServerboundPlayerLoadedPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ServerboundRecipeBookChangeSettingsPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ServerboundRecipeBookSeenRecipePacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ServerboundRenameItemPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ServerboundSeenAdvancementsPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ServerboundSelectBundleItemPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ServerboundSelectTradePacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ServerboundSetBeaconPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ServerboundSetCarriedItemPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ServerboundSetCommandBlockPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ServerboundSetCommandMinecartPacket.java`.
- [x] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ServerboundSetCreativeModeSlotPacket.java`. — Ported codec coverage and creative inventory slot application; negative-slot creative drop handling is explicitly deferred.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ServerboundSetGameRulePacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ServerboundSetJigsawBlockPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ServerboundSetStructureBlockPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ServerboundSetTestBlockPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ServerboundSignUpdatePacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ServerboundSpectateEntityPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ServerboundSwingPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ServerboundTeleportToEntityPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ServerboundTestInstanceBlockActionPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ServerboundUseItemOnPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/ServerboundUseItemPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/VecDeltaCodec.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/game/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/network/protocol/handshake`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/handshake/ClientIntent.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/handshake/ClientIntentionPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/handshake/HandshakePacketTypes.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/handshake/HandshakeProtocols.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/handshake/ServerHandshakePacketListener.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/handshake/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/network/protocol/login`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/login/ClientLoginPacketListener.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/login/ClientboundCustomQueryPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/login/ClientboundHelloPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/login/ClientboundLoginCompressionPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/login/ClientboundLoginDisconnectPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/login/ClientboundLoginFinishedPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/login/LoginPacketTypes.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/login/LoginProtocols.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/login/ServerLoginPacketListener.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/login/ServerboundCustomQueryAnswerPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/login/ServerboundHelloPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/login/ServerboundKeyPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/login/ServerboundLoginAcknowledgedPacket.java`.

## `decompiled-server-26.1.2/net/minecraft/network/protocol/login/custom`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/login/custom/CustomQueryAnswerPayload.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/login/custom/CustomQueryPayload.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/login/custom/DiscardedQueryAnswerPayload.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/login/custom/DiscardedQueryPayload.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/login/custom/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/network/protocol/login`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/login/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/network/protocol`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/network/protocol/ping`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/ping/ClientPongPacketListener.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/ping/ClientboundPongResponsePacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/ping/PingPacketTypes.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/ping/ServerPingPacketListener.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/ping/ServerboundPingRequestPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/ping/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/network/protocol/status`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/status/ClientStatusPacketListener.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/status/ClientboundStatusResponsePacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/status/ServerStatus.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/status/ServerStatusPacketListener.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/status/ServerboundStatusRequestPacket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/status/StatusPacketTypes.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/status/StatusProtocols.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/protocol/status/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/network/syncher`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/syncher/EntityDataAccessor.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/syncher/EntityDataSerializer.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/syncher/EntityDataSerializers.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/syncher/SyncedDataHolder.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/syncher/SynchedEntityData.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/network/syncher/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/recipebook`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/recipebook/PlaceRecipeHelper.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/recipebook/ServerPlaceRecipe.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/recipebook/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/references`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/references/BlockIds.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/references/ItemIds.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/references/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/resources`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/resources/DelegatingOps.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/resources/DependantName.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/resources/FileToIdConverter.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/resources/HolderSetCodec.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/resources/Identifier.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/resources/NetworkRegistryLoadTask.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/resources/RegistryDataLoader.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/resources/RegistryFileCodec.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/resources/RegistryFixedCodec.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/resources/RegistryLoadTask.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/resources/RegistryOps.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/resources/RegistryValidator.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/resources/ResourceKey.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/resources/ResourceManagerRegistryLoadTask.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/resources/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/server`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/Bootstrap.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/ChainedJsonException.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/ConsoleInput.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/DebugLoggedPrintStream.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/Eula.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/LoggedPrintStream.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/Main.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/MinecraftServer.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/PlayerAdvancements.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/RegistryLayer.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/ReloadableServerRegistries.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/ReloadableServerResources.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/RunningOnDifferentThreadException.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/ServerAdvancementManager.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/ServerFunctionLibrary.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/ServerFunctionManager.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/ServerInfo.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/ServerInterface.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/ServerLinks.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/ServerScoreboard.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/ServerTickRateManager.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/Services.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/SuppressedExceptionCollector.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/TickTask.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/WorldLoader.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/WorldStem.java`.

## `decompiled-server-26.1.2/net/minecraft/server/advancements`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/advancements/AdvancementVisibilityEvaluator.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/advancements/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/server/bossevents`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/bossevents/CustomBossEvent.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/bossevents/CustomBossEvents.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/bossevents/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/server/chase`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/chase/ChaseClient.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/chase/ChaseServer.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/chase/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/server/commands`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/commands/AdvancementCommands.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/commands/AttributeCommand.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/commands/BanIpCommands.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/commands/BanListCommands.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/commands/BanPlayerCommands.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/commands/BossBarCommands.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/commands/ChaseCommand.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/commands/ClearInventoryCommands.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/commands/CloneCommands.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/commands/DamageCommand.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/commands/DataPackCommand.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/commands/DeOpCommands.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/commands/DebugCommand.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/commands/DebugConfigCommand.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/commands/DebugMobSpawningCommand.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/commands/DebugPathCommand.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/commands/DefaultGameModeCommands.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/commands/DialogCommand.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/commands/DifficultyCommand.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/commands/EffectCommands.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/commands/EmoteCommands.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/commands/EnchantCommand.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/commands/ExecuteCommand.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/commands/ExperienceCommand.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/commands/FetchProfileCommand.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/commands/FillBiomeCommand.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/commands/FillCommand.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/commands/ForceLoadCommand.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/commands/FunctionCommand.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/commands/GameModeCommand.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/commands/GameRuleCommand.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/commands/GiveCommand.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/commands/HelpCommand.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/commands/InCommandFunction.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/commands/ItemCommands.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/commands/JfrCommand.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/commands/KickCommand.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/commands/KillCommand.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/commands/ListPlayersCommand.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/commands/LocateCommand.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/commands/LookAt.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/commands/LootCommand.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/commands/MsgCommand.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/commands/OpCommand.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/commands/PardonCommand.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/commands/PardonIpCommand.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/commands/ParticleCommand.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/commands/PerfCommand.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/commands/PlaceCommand.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/commands/PlaySoundCommand.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/commands/PublishCommand.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/commands/RaidCommand.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/commands/RandomCommand.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/commands/RecipeCommand.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/commands/ReloadCommand.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/commands/ReturnCommand.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/commands/RideCommand.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/commands/RotateCommand.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/commands/SaveAllCommand.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/commands/SaveOffCommand.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/commands/SaveOnCommand.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/commands/SayCommand.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/commands/ScheduleCommand.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/commands/ScoreboardCommand.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/commands/SeedCommand.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/commands/ServerPackCommand.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/commands/SetBlockCommand.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/commands/SetPlayerIdleTimeoutCommand.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/commands/SetSpawnCommand.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/commands/SetWorldSpawnCommand.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/commands/SpawnArmorTrimsCommand.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/commands/SpectateCommand.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/commands/SpreadPlayersCommand.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/commands/StopCommand.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/commands/StopSoundCommand.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/commands/StopwatchCommand.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/commands/SummonCommand.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/commands/SwingCommand.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/commands/TagCommand.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/commands/TeamCommand.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/commands/TeamMsgCommand.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/commands/TeleportCommand.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/commands/TellRawCommand.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/commands/TickCommand.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/commands/TimeCommand.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/commands/TitleCommand.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/commands/TransferCommand.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/commands/TriggerCommand.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/commands/VersionCommand.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/commands/WardenSpawnTrackerCommand.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/commands/WaypointCommand.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/commands/WeatherCommand.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/commands/WhitelistCommand.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/commands/WorldBorderCommand.java`.

## `decompiled-server-26.1.2/net/minecraft/server/commands/data`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/commands/data/BlockDataAccessor.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/commands/data/DataAccessor.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/commands/data/DataCommands.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/commands/data/EntityDataAccessor.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/commands/data/StorageDataAccessor.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/commands/data/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/server/commands`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/commands/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/server/dedicated`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/dedicated/DedicatedPlayerList.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/dedicated/DedicatedServer.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/dedicated/DedicatedServerProperties.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/dedicated/DedicatedServerSettings.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/dedicated/ServerWatchdog.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/dedicated/Settings.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/dedicated/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/server/dialog`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/dialog/ActionButton.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/dialog/ButtonListDialog.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/dialog/CommonButtonData.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/dialog/CommonDialogData.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/dialog/ConfirmationDialog.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/dialog/Dialog.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/dialog/DialogAction.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/dialog/DialogListDialog.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/dialog/DialogTypes.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/dialog/Dialogs.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/dialog/Input.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/dialog/MultiActionDialog.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/dialog/NoticeDialog.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/dialog/ServerLinksDialog.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/dialog/SimpleDialog.java`.

## `decompiled-server-26.1.2/net/minecraft/server/dialog/action`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/dialog/action/Action.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/dialog/action/ActionTypes.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/dialog/action/CommandTemplate.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/dialog/action/CustomAll.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/dialog/action/ParsedTemplate.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/dialog/action/StaticAction.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/dialog/action/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/server/dialog/body`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/dialog/body/DialogBody.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/dialog/body/DialogBodyTypes.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/dialog/body/ItemBody.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/dialog/body/PlainMessage.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/dialog/body/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/server/dialog/input`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/dialog/input/BooleanInput.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/dialog/input/InputControl.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/dialog/input/InputControlTypes.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/dialog/input/NumberRangeInput.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/dialog/input/SingleOptionInput.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/dialog/input/TextInput.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/dialog/input/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/server/dialog`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/dialog/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/server/gui`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/gui/MinecraftServerGui.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/gui/PlayerListComponent.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/gui/StatsComponent.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/gui/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/server/jsonrpc`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/jsonrpc/Connection.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/jsonrpc/IncomingRpcMethod.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/jsonrpc/IncomingRpcMethods.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/jsonrpc/JsonRPCErrors.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/jsonrpc/JsonRPCUtils.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/jsonrpc/JsonRpcLogger.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/jsonrpc/JsonRpcNotificationService.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/jsonrpc/ManagementServer.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/jsonrpc/OutgoingRpcMethod.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/jsonrpc/OutgoingRpcMethods.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/jsonrpc/PendingRpcRequest.java`.

## `decompiled-server-26.1.2/net/minecraft/server/jsonrpc/api`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/jsonrpc/api/MethodInfo.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/jsonrpc/api/ParamInfo.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/jsonrpc/api/PlayerDto.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/jsonrpc/api/ReferenceUtil.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/jsonrpc/api/ResultInfo.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/jsonrpc/api/Schema.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/jsonrpc/api/SchemaComponent.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/jsonrpc/api/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/server/jsonrpc/dataprovider`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/jsonrpc/dataprovider/JsonRpcApiSchema.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/jsonrpc/dataprovider/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/server/jsonrpc/internalapi`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/jsonrpc/internalapi/MinecraftAllowListService.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/jsonrpc/internalapi/MinecraftAllowListServiceImpl.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/jsonrpc/internalapi/MinecraftApi.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/jsonrpc/internalapi/MinecraftBanListService.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/jsonrpc/internalapi/MinecraftBanListServiceImpl.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/jsonrpc/internalapi/MinecraftExecutorService.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/jsonrpc/internalapi/MinecraftExecutorServiceImpl.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/jsonrpc/internalapi/MinecraftGameRuleService.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/jsonrpc/internalapi/MinecraftGameRuleServiceImpl.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/jsonrpc/internalapi/MinecraftOperatorListService.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/jsonrpc/internalapi/MinecraftOperatorListServiceImpl.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/jsonrpc/internalapi/MinecraftPlayerListService.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/jsonrpc/internalapi/MinecraftPlayerListServiceImpl.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/jsonrpc/internalapi/MinecraftServerSettingsService.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/jsonrpc/internalapi/MinecraftServerSettingsServiceImpl.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/jsonrpc/internalapi/MinecraftServerStateService.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/jsonrpc/internalapi/MinecraftServerStateServiceImpl.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/jsonrpc/internalapi/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/server/jsonrpc/methods`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/jsonrpc/methods/AllowlistService.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/jsonrpc/methods/BanlistService.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/jsonrpc/methods/ClientInfo.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/jsonrpc/methods/DiscoveryService.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/jsonrpc/methods/EncodeJsonRpcException.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/jsonrpc/methods/GameRulesService.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/jsonrpc/methods/InvalidParameterJsonRpcException.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/jsonrpc/methods/InvalidRequestJsonRpcException.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/jsonrpc/methods/IpBanlistService.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/jsonrpc/methods/Message.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/jsonrpc/methods/MethodNotFoundJsonRpcException.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/jsonrpc/methods/OperatorService.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/jsonrpc/methods/PlayerService.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/jsonrpc/methods/RemoteRpcErrorException.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/jsonrpc/methods/ServerSettingsService.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/jsonrpc/methods/ServerStateService.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/jsonrpc/methods/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/server/jsonrpc`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/jsonrpc/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/server/jsonrpc/security`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/jsonrpc/security/AuthenticationHandler.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/jsonrpc/security/JsonRpcSslContextProvider.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/jsonrpc/security/SecurityConfig.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/jsonrpc/security/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/server/jsonrpc/websocket`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/jsonrpc/websocket/JsonToWebSocketEncoder.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/jsonrpc/websocket/WebSocketToJsonCodec.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/jsonrpc/websocket/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/server/level`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/level/BlockDestructionProgress.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/level/ChunkGenerationTask.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/level/ChunkHolder.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/level/ChunkLevel.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/level/ChunkLoadCounter.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/level/ChunkMap.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/level/ChunkResult.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/level/ChunkTaskDispatcher.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/level/ChunkTaskPriorityQueue.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/level/ChunkTracker.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/level/ChunkTrackingView.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/level/ClientInformation.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/level/ColumnPos.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/level/DemoMode.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/level/DistanceManager.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/level/FullChunkStatus.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/level/GeneratingChunkMap.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/level/GenerationChunkHolder.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/level/LoadingChunkTracker.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/level/ParticleStatus.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/level/PlayerMap.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/level/PlayerSpawnFinder.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/level/SectionTracker.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/level/ServerBossEvent.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/level/ServerChunkCache.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/level/ServerEntity.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/level/ServerEntityGetter.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/level/ServerLevel.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/level/ServerPlayer.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/level/ServerPlayerGameMode.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/level/SimulationChunkTracker.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/level/ThreadedLevelLightEngine.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/level/ThrottlingChunkTaskDispatcher.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/level/Ticket.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/level/TicketType.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/level/WorldGenRegion.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/level/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/server/level/progress`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/level/progress/ChunkLoadStatusView.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/level/progress/LevelLoadListener.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/level/progress/LevelLoadProgressTracker.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/level/progress/LoggingLevelLoadListener.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/level/progress/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/server/network`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/network/CommonListenerCookie.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/network/ConfigurationTask.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/network/EventLoopGroupHolder.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/network/Filterable.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/network/FilteredText.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/network/LegacyProtocolUtils.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/network/LegacyQueryHandler.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/network/LegacyTextFilter.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/network/MemoryServerHandshakePacketListenerImpl.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/network/PlayerChunkSender.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/network/PlayerSafetyServiceTextFilter.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/network/ServerCommonPacketListenerImpl.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/network/ServerConfigurationPacketListenerImpl.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/network/ServerConnectionListener.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/network/ServerGamePacketListenerImpl.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/network/ServerHandshakePacketListenerImpl.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/network/ServerLoginPacketListenerImpl.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/network/ServerPlayerConnection.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/network/ServerStatusPacketListenerImpl.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/network/ServerTextFilter.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/network/TextFilter.java`.

## `decompiled-server-26.1.2/net/minecraft/server/network/config`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/network/config/JoinWorldTask.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/network/config/PrepareSpawnTask.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/network/config/ServerCodeOfConductConfigurationTask.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/network/config/ServerResourcePackConfigurationTask.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/network/config/SynchronizeRegistriesTask.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/network/config/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/server/network`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/network/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/server/notifications`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/notifications/EmptyNotificationService.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/notifications/NotificationManager.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/notifications/NotificationService.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/notifications/ServerActivityMonitor.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/notifications/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/server`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/server/packs`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/packs/AbstractPackResources.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/packs/CompositePackResources.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/packs/DownloadCacheCleaner.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/packs/DownloadQueue.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/packs/FeatureFlagsMetadataSection.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/packs/FilePackResources.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/packs/OverlayMetadataSection.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/packs/PackLocationInfo.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/packs/PackResources.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/packs/PackSelectionConfig.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/packs/PackType.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/packs/PathPackResources.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/packs/VanillaPackResources.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/packs/VanillaPackResourcesBuilder.java`.

## `decompiled-server-26.1.2/net/minecraft/server/packs/linkfs`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/packs/linkfs/LinkFSFileStore.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/packs/linkfs/LinkFSPath.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/packs/linkfs/LinkFSProvider.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/packs/linkfs/LinkFileSystem.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/packs/linkfs/PathContents.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/packs/linkfs/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/server/packs/metadata`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/packs/metadata/MetadataSectionType.java`.

## `decompiled-server-26.1.2/net/minecraft/server/packs/metadata/pack`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/packs/metadata/pack/PackFormat.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/packs/metadata/pack/PackMetadataSection.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/packs/metadata/pack/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/server/packs/metadata`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/packs/metadata/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/server/packs`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/packs/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/server/packs/repository`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/packs/repository/BuiltInPackSource.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/packs/repository/FolderRepositorySource.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/packs/repository/KnownPack.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/packs/repository/Pack.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/packs/repository/PackCompatibility.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/packs/repository/PackDetector.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/packs/repository/PackRepository.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/packs/repository/PackSource.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/packs/repository/RepositorySource.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/packs/repository/ServerPacksSource.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/packs/repository/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/server/packs/resources`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/packs/resources/CloseableResourceManager.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/packs/resources/FallbackResourceManager.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/packs/resources/IoSupplier.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/packs/resources/MultiPackResourceManager.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/packs/resources/PreparableReloadListener.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/packs/resources/ProfiledReloadInstance.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/packs/resources/ReloadInstance.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/packs/resources/ReloadableResourceManager.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/packs/resources/Resource.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/packs/resources/ResourceFilterSection.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/packs/resources/ResourceManager.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/packs/resources/ResourceManagerReloadListener.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/packs/resources/ResourceMetadata.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/packs/resources/ResourceProvider.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/packs/resources/SimpleJsonResourceReloadListener.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/packs/resources/SimplePreparableReloadListener.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/packs/resources/SimpleReloadInstance.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/packs/resources/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/server/permissions`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/permissions/LevelBasedPermissionSet.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/permissions/Permission.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/permissions/PermissionCheck.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/permissions/PermissionCheckTypes.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/permissions/PermissionLevel.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/permissions/PermissionProviderCheck.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/permissions/PermissionSet.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/permissions/PermissionSetSupplier.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/permissions/PermissionSetUnion.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/permissions/PermissionTypes.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/permissions/Permissions.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/permissions/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/server/players`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/players/BanListEntry.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/players/CachedUserNameToIdResolver.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/players/IpBanList.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/players/IpBanListEntry.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/players/NameAndId.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/players/OldUsersConverter.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/players/PlayerList.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/players/ProfileResolver.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/players/ServerOpList.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/players/ServerOpListEntry.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/players/SleepStatus.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/players/StoredUserEntry.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/players/StoredUserList.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/players/UserBanList.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/players/UserBanListEntry.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/players/UserNameToIdResolver.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/players/UserWhiteList.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/players/UserWhiteListEntry.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/players/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/server/rcon`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/rcon/NetworkDataOutputStream.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/rcon/PktUtils.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/rcon/RconConsoleSource.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/rcon/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/server/rcon/thread`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/rcon/thread/GenericThread.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/rcon/thread/QueryThreadGs4.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/rcon/thread/RconClient.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/rcon/thread/RconThread.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/rcon/thread/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/server/waypoints`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/waypoints/ServerWaypointManager.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/server/waypoints/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/sounds`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/sounds/Music.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/sounds/Musics.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/sounds/SoundEvent.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/sounds/SoundEvents.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/sounds/SoundSource.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/sounds/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/stats`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/stats/RecipeBook.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/stats/RecipeBookSettings.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/stats/ServerRecipeBook.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/stats/ServerStatsCounter.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/stats/Stat.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/stats/StatFormatter.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/stats/StatType.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/stats/Stats.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/stats/StatsCounter.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/stats/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/tags`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/tags/BannerPatternTags.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/tags/BiomeTags.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/tags/BlockTags.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/tags/DamageTypeTags.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/tags/DialogTags.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/tags/EnchantmentTags.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/tags/EntityTypeTags.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/tags/FeatureTags.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/tags/FlatLevelGeneratorPresetTags.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/tags/FluidTags.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/tags/GameEventTags.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/tags/InstrumentTags.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/tags/ItemTags.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/tags/PaintingVariantTags.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/tags/PoiTypeTags.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/tags/PotionTags.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/tags/StructureTags.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/tags/TagBuilder.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/tags/TagEntry.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/tags/TagFile.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/tags/TagKey.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/tags/TagLoader.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/tags/TagNetworkSerialization.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/tags/TimelineTags.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/tags/VillagerTradeTags.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/tags/WorldPresetTags.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/tags/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/util`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/ARGB.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/AbortableIterationConsumer.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/AbstractListBuilder.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/ArrayListDeque.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/BinaryAnimator.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/BitStorage.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/BlockUtil.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/BoundedFloatFunction.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/Brightness.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/ByIdMap.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/ClassInstanceMultiMap.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/ClassTreeIdRegistry.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/ColorRGBA.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/CommonColors.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/CommonLinks.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/CompilableString.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/CrudeIncrementalIntIdentityHashBiMap.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/Crypt.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/CryptException.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/CsvOutput.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/CubicSpline.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/DelegateDataOutput.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/DependencySorter.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/DirectoryLock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/DummyFileAttributes.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/Ease.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/EasingType.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/EncoderCache.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/ExceptionCollector.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/ExtraCodecs.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/FastBufferedInputStream.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/FileSystemUtil.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/FileUtil.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/FileZipper.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/FormattedCharSequence.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/FormattedCharSink.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/FutureChain.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/Graph.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/GsonHelper.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/HashOps.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/HttpUtil.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/IdentifierPattern.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/InclusiveRange.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/KeyDispatchDataCodec.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/Keyframe.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/KeyframeTrack.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/KeyframeTrackSampler.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/LenientJsonParser.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/LightCoordsUtil.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/LinearCongruentialGenerator.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/ListAndDeque.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/MemoryReserve.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/ModCheck.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/Mth.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/NativeModuleLister.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/NullOps.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/ParticleUtils.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/PlaceholderLookupProvider.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/PngInfo.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/ProblemReporter.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/ProgressListener.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/RandomSource.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/RegistryContextSwapper.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/SegmentedAnglePrecision.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/SequencedPriorityIterator.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/SignatureUpdater.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/SignatureValidator.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/Signer.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/SimpleBitStorage.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/SingleKeyCache.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/SmoothDouble.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/SortedArraySet.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/SpawnUtil.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/SpecialDates.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/StaticCache2D.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/StrictJsonParser.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/StringDecomposer.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/StringRepresentable.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/StringUtil.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/TaskChainer.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/ThreadingDetector.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/TickThrottler.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/TimeSource.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/TimeUtil.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/ToFloatFunction.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/TriState.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/Tuple.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/Unit.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/Util.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/VisibleForDebug.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/ZeroBitStorage.java`.

## `decompiled-server-26.1.2/net/minecraft/util/context`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/context/ContextKey.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/context/ContextKeySet.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/context/ContextMap.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/context/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/util/datafix`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/DataFixTypes.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/DataFixers.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/ExtraDataFixUtils.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/LegacyComponentDataFixUtils.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/PackedBitStorage.java`.

## `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/AbstractArrowPickupFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/AbstractBlockPropertyFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/AbstractPoiSectionFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/AbstractUUIDFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/AddFieldFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/AddFlagIfNotPresentFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/AddNewChoices.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/AdvancementsFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/AdvancementsRenameFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/AreaEffectCloudDurationScaleFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/AreaEffectCloudPotionFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/AttributeIdPrefixFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/AttributeModifierIdFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/AttributesRenameFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/AttributesRenameLegacy.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/BannerEntityCustomNameToOverrideComponentFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/BannerPatternFormatFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/BedItemColorFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/BeehiveFieldRenameFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/BiomeFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/BitStorageAlignFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/BlendingDataFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/BlendingDataRemoveFromNetherEndFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/BlockEntityBannerColorFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/BlockEntityBlockStateFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/BlockEntityCustomNameToComponentFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/BlockEntityFurnaceBurnTimeFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/BlockEntityIdFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/BlockEntityJukeboxFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/BlockEntityKeepPacked.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/BlockEntityRenameFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/BlockEntityShulkerBoxColorFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/BlockEntitySignDoubleSidedEditableTextFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/BlockEntityUUIDFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/BlockNameFlatteningFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/BlockPosFormatAndRenamesFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/BlockPropertyRenameAndFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/BlockRenameFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/BlockStateData.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/BlockStateStructureTemplateFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/BoatSplitFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/CarvingStepRemoveFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/CatTypeFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/CauldronRenameFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/CavesAndCliffsRenames.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/ChestedHorsesInventoryZeroIndexingFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/ChunkBedBlockEntityInjecterFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/ChunkBiomeFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/ChunkDeleteIgnoredLightDataFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/ChunkDeleteLightFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/ChunkHeightAndBiomeFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/ChunkLightRemoveFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/ChunkPalettedStorageFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/ChunkProtoTickListFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/ChunkRenamesFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/ChunkStatusFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/ChunkStatusFix2.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/ChunkStructuresTemplateRenameFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/ChunkTicketUnpackPosFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/ChunkToProtochunkFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/ColorlessShulkerEntityFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/ContainerBlockEntityLockPredicateFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/CopperGolemWeatherStateFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/CriteriaRenameFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/CustomModelDataExpandFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/DataComponentRemainderFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/DayTimeToClockFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/DebugProfileLookingAtSplitFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/DebugProfileOverlayReferenceFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/DecoratedPotFieldRenameFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/DropChancesFormatFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/DropInvalidSignDataFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/DyeItemRenameFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/EffectDurationFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/EmptyItemInHotbarFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/EmptyItemInVillagerTradeFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/EntityArmorStandSilentFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/EntityAttributeBaseValueFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/EntityBlockStateFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/EntityBrushableBlockFieldsRenameFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/EntityCatSplitFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/EntityCodSalmonFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/EntityCustomNameToComponentFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/EntityElderGuardianSplitFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/EntityEquipmentToArmorAndHandFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/EntityFallDistanceFloatToDoubleFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/EntityFieldsRenameFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/EntityGoatMissingStateFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/EntityHealthFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/EntityHorseSaddleFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/EntityHorseSplitFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/EntityIdFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/EntityItemFrameDirectionFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/EntityMinecartIdentifiersFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/EntityPaintingItemFrameDirectionFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/EntityPaintingMotiveFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/EntityProjectileOwnerFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/EntityPufferfishRenameFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/EntityRavagerRenameFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/EntityRedundantChanceTagsFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/EntityRenameFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/EntityRidingToPassengersFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/EntitySalmonSizeFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/EntityShulkerColorFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/EntityShulkerRotationFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/EntitySkeletonSplitFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/EntitySpawnerItemVariantComponentFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/EntityStringUuidFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/EntityTheRenameningFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/EntityTippedArrowFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/EntityUUIDFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/EntityVariantFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/EntityWolfColorFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/EntityZombieSplitFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/EntityZombieVillagerTypeFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/EntityZombifiedPiglinRenameFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/EquipmentFormatFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/EquippableAssetRenameFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/FeatureFlagRemoveFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/FilteredBooksFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/FilteredSignsFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/FireResistantToDamageResistantComponentFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/FixProjectileStoredItem.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/FixWolfHealth.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/FoodToConsumableFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/ForcePoiRebuild.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/ForcedChunkToTicketFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/FurnaceRecipeFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/GameRuleRegistryFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/GoatHornIdFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/GossipUUIDFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/HeightmapRenamingFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/HorseBodyArmorItemFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/IglooMetadataRemovalFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/InlineBlockPosFormatFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/InvalidBlockEntityLockFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/InvalidLockComponentFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/ItemBannerColorFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/ItemCustomNameToComponentFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/ItemIdFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/ItemLoreFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/ItemPotionFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/ItemRenameFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/ItemShulkerBoxColorFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/ItemSpawnEggFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/ItemStackComponentizationFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/ItemStackCustomNameToOverrideComponentFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/ItemStackEnchantmentNamesFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/ItemStackMapIdFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/ItemStackSpawnEggFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/ItemStackTagFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/ItemStackTagRemainderFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/ItemStackTheFlatteningFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/ItemStackUUIDFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/ItemWaterPotionFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/JigsawPropertiesFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/JigsawRotationFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/JukeboxTicksSinceSongStartedFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/LeavesFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/LegacyDimensionIdFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/LegacyDragonFightFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/LegacyHoverEventFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/LegacyWorldBorderFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/LevelDatDifficultyFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/LevelDatToSavedDataPreparationFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/LevelDataGeneratorOptionsFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/LevelFlatGeneratorInfoFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/LevelLegacyWorldGenSettingsFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/LevelUUIDFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/LockComponentPredicateFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/LodestoneCompassComponentFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/MapBannerBlockPosFormatFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/MapIdFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/MemoryExpiryDataFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/MissingDimensionFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/MobEffectIdFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/MobSpawnerEntityIdentifiersFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/NamedEntityConvertUncheckedFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/NamedEntityFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/NamedEntityWriteReadFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/NamespacedTypeRenameFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/NewVillageFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/ObjectiveRenderTypeFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/OminousBannerBlockEntityRenameFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/OminousBannerRarityFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/OminousBannerRenameFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/OptionsAccessibilityOnboardFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/OptionsAddTextBackgroundFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/OptionsAmbientOcclusionFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/OptionsFancyGraphicsToGraphicsModeFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/OptionsForceVBOFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/OptionsGraphicsModeSplitFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/OptionsKeyLwjgl3Fix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/OptionsKeyTranslationFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/OptionsLowerCaseLanguageFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/OptionsMenuBlurrinessFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/OptionsMusicToastFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/OptionsProgrammerArtFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/OptionsRenameFieldFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/OptionsSetGraphicsPresetToCustomFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/OverreachingTickFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/ParticleUnflatteningFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/PlayerEquipmentFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/PlayerHeadBlockProfileFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/PlayerRespawnDataFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/PlayerUUIDFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/PoiTypeRemoveFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/PoiTypeRenameFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/PrimedTntBlockStateFixer.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/ProjectileStoredWeaponFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/RaidRenamesDataFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/RandomSequenceSettingsFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/RecipesFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/RecipesRenameningFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/RedstoneWireConnectionsFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/References.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/RemapChunkStatusFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/RemoveBlockEntityTagFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/RemoveEmptyItemInBrushableBlockFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/RemoveGolemGossipFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/RenameEnchantmentsFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/RenamedCoralFansFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/RenamedCoralFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/ReorganizePoi.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/SaddleEquipmentSlotFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/SavedDataFeaturePoolElementFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/SavedDataUUIDFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/ScoreboardDisplayNameFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/ScoreboardDisplaySlotFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/SignTextStrictJsonFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/SimpleEntityRenameFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/SimplestEntityRenameFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/SpawnerDataFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/StatsCounterFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/StatsRenameFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/StonecutterRecipeRenameFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/StriderGravityFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/StructureReferenceCountFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/StructureSettingsFlattenFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/StructuresBecomeConfiguredFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/TextComponentHoverAndClickEventFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/TextComponentStringifiedFlagsFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/ThrownPotionSplitFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/TippedArrowPotionToItemFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/TooltipDisplayComponentFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/TrappedChestBlockEntityFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/TrialSpawnerConfigFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/TrialSpawnerConfigInRegistryFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/TridentAnimationFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/UnflattenTextComponentFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/VariantRenameFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/VillagerDataFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/VillagerFollowRangeFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/VillagerRebuildLevelAndXpFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/VillagerSetCanPickUpLootFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/VillagerSetVillagerDataFinalized.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/VillagerTradeFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/WallPropertyFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/WeaponSmithChestLootTableFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/WorldBorderWarningTimeFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/WorldGenSettingsDisallowOldCustomWorldsFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/WorldGenSettingsFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/WorldGenSettingsHeightAndBiomeFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/WorldSpawnDataFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/WriteAndReadFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/WrittenBookPagesStrictJsonFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/ZombieVillagerRebuildXpFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/ZombieVillagerSetVillagerDataFinalized.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/fixes/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/util/datafix`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/util/datafix/schemas`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/schemas/NamespacedSchema.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/schemas/V100.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/schemas/V102.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/schemas/V1022.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/schemas/V106.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/schemas/V107.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/schemas/V1125.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/schemas/V135.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/schemas/V143.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/schemas/V1451.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/schemas/V1451_1.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/schemas/V1451_2.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/schemas/V1451_3.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/schemas/V1451_4.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/schemas/V1451_5.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/schemas/V1451_6.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/schemas/V1458.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/schemas/V1460.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/schemas/V1466.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/schemas/V1470.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/schemas/V1481.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/schemas/V1483.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/schemas/V1486.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/schemas/V1488.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/schemas/V1510.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/schemas/V1800.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/schemas/V1801.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/schemas/V1904.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/schemas/V1906.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/schemas/V1909.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/schemas/V1920.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/schemas/V1928.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/schemas/V1929.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/schemas/V1931.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/schemas/V2100.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/schemas/V2501.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/schemas/V2502.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/schemas/V2505.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/schemas/V2509.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/schemas/V2511_1.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/schemas/V2519.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/schemas/V2522.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/schemas/V2551.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/schemas/V2568.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/schemas/V2571.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/schemas/V2684.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/schemas/V2686.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/schemas/V2688.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/schemas/V2704.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/schemas/V2707.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/schemas/V2831.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/schemas/V2832.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/schemas/V2842.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/schemas/V3076.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/schemas/V3078.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/schemas/V3081.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/schemas/V3082.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/schemas/V3083.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/schemas/V3202.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/schemas/V3203.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/schemas/V3204.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/schemas/V3325.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/schemas/V3326.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/schemas/V3327.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/schemas/V3328.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/schemas/V3438.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/schemas/V3439.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/schemas/V3439_1.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/schemas/V3448.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/schemas/V3682.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/schemas/V3683.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/schemas/V3685.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/schemas/V3689.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/schemas/V3799.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/schemas/V3807.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/schemas/V3808.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/schemas/V3808_1.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/schemas/V3808_2.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/schemas/V3813.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/schemas/V3816.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/schemas/V3818.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/schemas/V3818_3.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/schemas/V3818_4.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/schemas/V3818_5.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/schemas/V3825.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/schemas/V3938.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/schemas/V4059.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/schemas/V4067.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/schemas/V4070.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/schemas/V4071.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/schemas/V4290.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/schemas/V4292.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/schemas/V4300.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/schemas/V4301.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/schemas/V4302.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/schemas/V4306.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/schemas/V4307.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/schemas/V4312.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/schemas/V4420.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/schemas/V4421.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/schemas/V4531.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/schemas/V4532.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/schemas/V4533.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/schemas/V4543.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/schemas/V4648.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/schemas/V4656.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/schemas/V4771.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/schemas/V501.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/schemas/V700.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/schemas/V701.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/schemas/V702.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/schemas/V703.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/schemas/V704.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/schemas/V705.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/schemas/V808.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/schemas/V99.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/datafix/schemas/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/util/debug`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/debug/DebugBeeInfo.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/debug/DebugBrainDump.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/debug/DebugBreezeInfo.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/debug/DebugEntityBlockIntersection.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/debug/DebugGameEventInfo.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/debug/DebugGameEventListenerInfo.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/debug/DebugGoalInfo.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/debug/DebugHiveInfo.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/debug/DebugPathInfo.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/debug/DebugPoiInfo.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/debug/DebugStructureInfo.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/debug/DebugSubscription.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/debug/DebugSubscriptions.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/debug/DebugValueAccess.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/debug/DebugValueSource.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/debug/LevelDebugSynchronizers.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/debug/ServerDebugSubscribers.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/debug/TrackingDebugSynchronizer.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/debug/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/util/debugchart`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/debugchart/AbstractSampleLogger.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/debugchart/LocalSampleLogger.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/debugchart/RemoteDebugSampleType.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/debugchart/RemoteSampleLogger.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/debugchart/SampleLogger.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/debugchart/SampleStorage.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/debugchart/TpsDebugDimensions.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/debugchart/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/util/eventlog`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/eventlog/EventLogDirectory.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/eventlog/JsonEventLog.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/eventlog/JsonEventLogReader.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/eventlog/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/util/filefix`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/filefix/AbortedFileFixException.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/filefix/AtomicMoveNotSupportedFileFixException.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/filefix/CanceledFileFixException.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/filefix/FailedCleanupFileFixException.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/filefix/FileFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/filefix/FileFixException.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/filefix/FileFixUtil.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/filefix/FileFixerUpper.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/filefix/FileSystemCapabilities.java`.

## `decompiled-server-26.1.2/net/minecraft/util/filefix/access`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/filefix/access/ChunkNbt.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/filefix/access/CompressedNbt.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/filefix/access/FileAccess.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/filefix/access/FileAccessProvider.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/filefix/access/FileRelation.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/filefix/access/FileResourceType.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/filefix/access/FileResourceTypes.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/filefix/access/LevelDat.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/filefix/access/PlayerData.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/filefix/access/SavedDataNbt.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/filefix/access/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/util/filefix/fixes`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/filefix/fixes/DimensionStorageFileFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/filefix/fixes/GeneratedStructuresRenameFileFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/filefix/fixes/LegacyStructureFileFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/filefix/fixes/LevelDatToSavedDataFileFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/filefix/fixes/PlayerStorageFileFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/filefix/fixes/RemoveObsoleteFilesFileFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/filefix/fixes/ResourcePackLocationFileFix.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/filefix/fixes/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/util/filefix/operations`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/filefix/operations/ApplyInFolders.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/filefix/operations/DeleteFileOrEmptyDirectory.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/filefix/operations/FileFixOperation.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/filefix/operations/FileFixOperations.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/filefix/operations/GroupMove.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/filefix/operations/ModifyContent.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/filefix/operations/Move.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/filefix/operations/RegexMove.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/filefix/operations/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/util/filefix`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/filefix/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/util/filefix/virtualfilesystem`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/filefix/virtualfilesystem/CopyOnWriteFSPath.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/filefix/virtualfilesystem/CopyOnWriteFSProvider.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/filefix/virtualfilesystem/CopyOnWriteFileStore.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/filefix/virtualfilesystem/CopyOnWriteFileSystem.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/filefix/virtualfilesystem/DirectoryNode.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/filefix/virtualfilesystem/FileMove.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/filefix/virtualfilesystem/FileNode.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/filefix/virtualfilesystem/Node.java`.

## `decompiled-server-26.1.2/net/minecraft/util/filefix/virtualfilesystem/exception`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/filefix/virtualfilesystem/exception/CowFSCreationException.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/filefix/virtualfilesystem/exception/CowFSDirectoryNotEmptyException.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/filefix/virtualfilesystem/exception/CowFSFileAlreadyExistsException.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/filefix/virtualfilesystem/exception/CowFSFileSystemException.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/filefix/virtualfilesystem/exception/CowFSIllegalArgumentException.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/filefix/virtualfilesystem/exception/CowFSNoSuchFileException.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/filefix/virtualfilesystem/exception/CowFSNotDirectoryException.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/filefix/virtualfilesystem/exception/CowFSSymlinkException.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/filefix/virtualfilesystem/exception/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/util/filefix/virtualfilesystem`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/filefix/virtualfilesystem/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/util/monitoring/jmx`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/monitoring/jmx/MinecraftServerStatistics.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/monitoring/jmx/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/util`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/util/parsing`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/parsing/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/util/parsing/packrat`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/parsing/packrat/Atom.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/parsing/packrat/CachedParseState.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/parsing/packrat/Control.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/parsing/packrat/DelayedException.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/parsing/packrat/Dictionary.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/parsing/packrat/ErrorCollector.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/parsing/packrat/ErrorEntry.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/parsing/packrat/NamedRule.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/parsing/packrat/ParseState.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/parsing/packrat/Rule.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/parsing/packrat/Scope.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/parsing/packrat/SuggestionSupplier.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/parsing/packrat/Term.java`.

## `decompiled-server-26.1.2/net/minecraft/util/parsing/packrat/commands`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/parsing/packrat/commands/CommandArgumentParser.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/parsing/packrat/commands/Grammar.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/parsing/packrat/commands/GreedyPatternParseRule.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/parsing/packrat/commands/GreedyPredicateParseRule.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/parsing/packrat/commands/IdentifierParseRule.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/parsing/packrat/commands/NumberRunParseRule.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/parsing/packrat/commands/ParserBasedArgument.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/parsing/packrat/commands/ResourceLookupRule.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/parsing/packrat/commands/ResourceSuggestion.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/parsing/packrat/commands/StringReaderParserState.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/parsing/packrat/commands/StringReaderTerms.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/parsing/packrat/commands/TagParseRule.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/parsing/packrat/commands/UnquotedStringParseRule.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/parsing/packrat/commands/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/util/parsing/packrat`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/parsing/packrat/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/util/profiling`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/profiling/ActiveProfiler.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/profiling/ContinuousProfiler.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/profiling/EmptyProfileResults.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/profiling/FilledProfileResults.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/profiling/InactiveProfiler.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/profiling/ProfileCollector.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/profiling/ProfileResults.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/profiling/Profiler.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/profiling/ProfilerFiller.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/profiling/ProfilerPathEntry.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/profiling/ResultField.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/profiling/SingleTickProfiler.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/profiling/TracyZoneFiller.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/profiling/Zone.java`.

## `decompiled-server-26.1.2/net/minecraft/util/profiling/jfr`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/profiling/jfr/Environment.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/profiling/jfr/JfrProfiler.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/profiling/jfr/JvmProfiler.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/profiling/jfr/Percentiles.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/profiling/jfr/SummaryReporter.java`.

## `decompiled-server-26.1.2/net/minecraft/util/profiling/jfr/callback`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/profiling/jfr/callback/ProfiledDuration.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/profiling/jfr/callback/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/util/profiling/jfr/event`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/profiling/jfr/event/ChunkGenerationEvent.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/profiling/jfr/event/ChunkRegionIoEvent.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/profiling/jfr/event/ChunkRegionReadEvent.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/profiling/jfr/event/ChunkRegionWriteEvent.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/profiling/jfr/event/ClientFpsEvent.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/profiling/jfr/event/NetworkSummaryEvent.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/profiling/jfr/event/PacketEvent.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/profiling/jfr/event/PacketReceivedEvent.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/profiling/jfr/event/PacketSentEvent.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/profiling/jfr/event/ServerTickTimeEvent.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/profiling/jfr/event/StructureGenerationEvent.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/profiling/jfr/event/WorldLoadFinishedEvent.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/profiling/jfr/event/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/util/profiling/jfr`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/profiling/jfr/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/util/profiling/jfr/parse`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/profiling/jfr/parse/JfrStatsParser.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/profiling/jfr/parse/JfrStatsResult.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/profiling/jfr/parse/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/util/profiling/jfr/serialize`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/profiling/jfr/serialize/JfrResultJsonSerializer.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/profiling/jfr/serialize/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/util/profiling/jfr/stats`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/profiling/jfr/stats/ChunkGenStat.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/profiling/jfr/stats/ChunkIdentification.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/profiling/jfr/stats/CpuLoadStat.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/profiling/jfr/stats/FileIOStat.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/profiling/jfr/stats/FpsStat.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/profiling/jfr/stats/GcHeapStat.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/profiling/jfr/stats/IoSummary.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/profiling/jfr/stats/PacketIdentification.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/profiling/jfr/stats/StructureGenStat.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/profiling/jfr/stats/ThreadAllocationStat.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/profiling/jfr/stats/TickTimeStat.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/profiling/jfr/stats/TimedStat.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/profiling/jfr/stats/TimedStatSummary.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/profiling/jfr/stats/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/util/profiling/metrics`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/profiling/metrics/MetricCategory.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/profiling/metrics/MetricSampler.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/profiling/metrics/MetricsRegistry.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/profiling/metrics/MetricsSamplerProvider.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/profiling/metrics/ProfilerMeasured.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/profiling/metrics/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/util/profiling/metrics/profiling`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/profiling/metrics/profiling/ActiveMetricsRecorder.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/profiling/metrics/profiling/InactiveMetricsRecorder.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/profiling/metrics/profiling/MetricsRecorder.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/profiling/metrics/profiling/ProfilerSamplerAdapter.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/profiling/metrics/profiling/ServerMetricsSamplersProvider.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/profiling/metrics/profiling/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/util/profiling/metrics/storage`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/profiling/metrics/storage/MetricsPersister.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/profiling/metrics/storage/RecordedDeviation.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/profiling/metrics/storage/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/util/profiling`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/profiling/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/util/random`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/random/Weighted.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/random/WeightedList.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/random/WeightedRandom.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/random/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/util/thread`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/thread/AbstractConsecutiveExecutor.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/thread/BlockableEventLoop.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/thread/ConsecutiveExecutor.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/thread/ParallelMapTransform.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/thread/PriorityConsecutiveExecutor.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/thread/ReentrantBlockableEventLoop.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/thread/StrictQueue.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/thread/TaskScheduler.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/thread/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/util/valueproviders`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/valueproviders/BiasedToBottomInt.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/valueproviders/ClampedInt.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/valueproviders/ClampedNormalFloat.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/valueproviders/ClampedNormalInt.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/valueproviders/ConstantFloat.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/valueproviders/ConstantInt.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/valueproviders/FloatProvider.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/valueproviders/FloatProviders.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/valueproviders/IntProvider.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/valueproviders/IntProviders.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/valueproviders/MultipliedFloats.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/valueproviders/SampledFloat.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/valueproviders/TrapezoidFloat.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/valueproviders/TrapezoidInt.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/valueproviders/UniformFloat.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/valueproviders/UniformInt.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/valueproviders/WeightedListInt.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/valueproviders/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/util/worldupdate`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/worldupdate/FileToUpgrade.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/worldupdate/RegionStorageUpgrader.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/worldupdate/UpgradeProgress.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/worldupdate/UpgradeStatusTranslator.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/worldupdate/WorldUpgrader.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/util/worldupdate/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/BossEvent.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/Clearable.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/CompoundContainer.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/Container.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/ContainerHelper.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/Containers.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/Difficulty.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/DifficultyInstance.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/InteractionHand.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/InteractionResult.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/ItemStackWithSlot.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/LockCode.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/MenuProvider.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/Nameable.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/RandomSequence.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/RandomSequences.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/RandomizableContainer.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/SimpleContainer.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/SimpleMenuProvider.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/Stopwatch.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/Stopwatches.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/TickRateManager.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/WorldlyContainer.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/WorldlyContainerHolder.java`.

## `decompiled-server-26.1.2/net/minecraft/world/attribute`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/attribute/AmbientAdditionsSettings.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/attribute/AmbientMoodSettings.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/attribute/AmbientParticle.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/attribute/AmbientSounds.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/attribute/AttributeRange.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/attribute/AttributeType.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/attribute/AttributeTypes.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/attribute/BackgroundMusic.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/attribute/BedRule.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/attribute/EnvironmentAttribute.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/attribute/EnvironmentAttributeLayer.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/attribute/EnvironmentAttributeMap.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/attribute/EnvironmentAttributeProbe.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/attribute/EnvironmentAttributeReader.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/attribute/EnvironmentAttributeSystem.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/attribute/EnvironmentAttributes.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/attribute/GaussianSampler.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/attribute/LerpFunction.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/attribute/SpatialAttributeInterpolator.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/attribute/WeatherAttributes.java`.

## `decompiled-server-26.1.2/net/minecraft/world/attribute/modifier`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/attribute/modifier/AttributeModifier.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/attribute/modifier/BooleanModifier.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/attribute/modifier/ColorModifier.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/attribute/modifier/FloatModifier.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/attribute/modifier/FloatWithAlpha.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/attribute/modifier/IntegerModifier.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/attribute/modifier/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/attribute`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/attribute/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/clock`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/clock/ClockManager.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/clock/ClockNetworkState.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/clock/ClockState.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/clock/ClockTimeMarker.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/clock/ClockTimeMarkers.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/clock/PackedClockStates.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/clock/ServerClockManager.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/clock/WorldClock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/clock/WorldClocks.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/clock/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/damagesource`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/damagesource/CombatEntry.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/damagesource/CombatRules.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/damagesource/CombatTracker.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/damagesource/DamageEffects.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/damagesource/DamageScaling.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/damagesource/DamageSource.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/damagesource/DamageSources.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/damagesource/DamageType.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/damagesource/DamageTypes.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/damagesource/DeathMessageType.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/damagesource/FallLocation.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/damagesource/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/effect`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/effect/AbsorptionMobEffect.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/effect/BadOmenMobEffect.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/effect/HealOrHarmMobEffect.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/effect/HungerMobEffect.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/effect/InfestedMobEffect.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/effect/InstantenousMobEffect.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/effect/MobEffect.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/effect/MobEffectCategory.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/effect/MobEffectInstance.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/effect/MobEffectUtil.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/effect/MobEffects.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/effect/OozingMobEffect.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/effect/PoisonMobEffect.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/effect/RaidOmenMobEffect.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/effect/RegenerationMobEffect.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/effect/SaturationMobEffect.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/effect/WeavingMobEffect.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/effect/WindChargedMobEffect.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/effect/WitherMobEffect.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/effect/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/entity`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/AgeableMob.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/AnimationState.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/AreaEffectCloud.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/Attackable.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/Avatar.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ContainerUser.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ConversionParams.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ConversionType.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/Crackiness.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/Display.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/DropChances.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ElytraAnimationState.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/Entity.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/EntityAttachment.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/EntityAttachments.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/EntityDimensions.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/EntityEquipment.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/EntityEvent.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/EntityFluidInteraction.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/EntityProcessor.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/EntityReference.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/EntitySelector.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/EntitySpawnReason.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/EntityType.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/EquipmentSlot.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/EquipmentSlotGroup.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/EquipmentTable.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/EquipmentUser.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ExperienceOrb.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/HasCustomInventoryScreen.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/HumanoidArm.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/InsideBlockEffectApplier.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/InsideBlockEffectType.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/Interaction.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/InterpolationHandler.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ItemBasedSteering.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ItemOwner.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ItemSteerable.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/Leashable.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/LightningBolt.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/LivingEntity.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/Marker.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/Mob.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/MobCategory.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/MoverType.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/NeutralMob.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/OminousItemSpawner.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/OwnableEntity.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/PathfinderMob.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/PlayerRideable.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/PlayerRideableJumping.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/PortalProcessor.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/Pose.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/PositionMoveRotation.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/Relative.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ReputationEventHandler.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/Shearable.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/SlotAccess.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/SlotProvider.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/SpawnGroupData.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/SpawnPlacementType.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/SpawnPlacementTypes.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/SpawnPlacements.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/TamableAnimal.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/Targeting.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/TraceableEntity.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/WalkAnimationState.java`.

## `decompiled-server-26.1.2/net/minecraft/world/entity/ai`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/ActivityData.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/Brain.java`.

## `decompiled-server-26.1.2/net/minecraft/world/entity/ai/attributes`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/attributes/Attribute.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/attributes/AttributeInstance.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/attributes/AttributeMap.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/attributes/AttributeModifier.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/attributes/AttributeSupplier.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/attributes/Attributes.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/attributes/DefaultAttributes.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/attributes/RangedAttribute.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/attributes/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/entity/ai/behavior`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/behavior/AcquirePoi.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/behavior/AnimalMakeLove.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/behavior/AnimalPanic.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/behavior/AssignProfessionFromJobSite.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/behavior/BabyFollowAdult.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/behavior/BackUpIfTooClose.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/behavior/BecomePassiveIfMemoryPresent.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/behavior/Behavior.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/behavior/BehaviorControl.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/behavior/BehaviorUtils.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/behavior/BlockPosTracker.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/behavior/CelebrateVillagersSurvivedRaid.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/behavior/ChargeAttack.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/behavior/CopyMemoryWithExpiry.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/behavior/CountDownCooldownTicks.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/behavior/Croak.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/behavior/CrossbowAttack.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/behavior/DismountOrSkipMounting.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/behavior/DoNothing.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/behavior/EntityTracker.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/behavior/EraseMemoryIf.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/behavior/FollowTemptation.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/behavior/GateBehavior.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/behavior/GiveGiftToHero.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/behavior/GoAndGiveItemsToTarget.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/behavior/GoToClosestVillage.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/behavior/GoToPotentialJobSite.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/behavior/GoToTargetLocation.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/behavior/GoToWantedItem.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/behavior/HarvestFarmland.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/behavior/InsideBrownianWalk.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/behavior/InteractWith.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/behavior/InteractWithDoor.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/behavior/JumpOnBed.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/behavior/LocateHidingPlace.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/behavior/LongJumpMidJump.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/behavior/LongJumpToPreferredBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/behavior/LongJumpToRandomPos.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/behavior/LongJumpUtil.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/behavior/LookAndFollowTradingPlayerSink.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/behavior/LookAtTargetSink.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/behavior/MeleeAttack.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/behavior/Mount.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/behavior/MoveToSkySeeingSpot.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/behavior/MoveToTargetSink.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/behavior/OneShot.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/behavior/PlayTagWithOtherKids.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/behavior/PoiCompetitorScan.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/behavior/PositionTracker.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/behavior/PrepareRamNearestTarget.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/behavior/RamTarget.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/behavior/RandomLookAround.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/behavior/RandomStroll.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/behavior/ReactToBell.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/behavior/ResetProfession.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/behavior/ResetRaidStatus.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/behavior/RingBell.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/behavior/RunOne.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/behavior/SetClosestHomeAsWalkTarget.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/behavior/SetEntityLookTarget.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/behavior/SetEntityLookTargetSometimes.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/behavior/SetHiddenState.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/behavior/SetLookAndInteract.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/behavior/SetRaidStatus.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/behavior/SetWalkTargetAwayFrom.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/behavior/SetWalkTargetFromAttackTargetIfTargetOutOfReach.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/behavior/SetWalkTargetFromBlockMemory.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/behavior/SetWalkTargetFromLookTarget.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/behavior/ShowTradesToPlayer.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/behavior/ShufflingList.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/behavior/SleepInBed.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/behavior/SocializeAtBell.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/behavior/SpearApproach.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/behavior/SpearAttack.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/behavior/SpearRetreat.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/behavior/StartAttacking.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/behavior/StartCelebratingIfTargetDead.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/behavior/StayCloseToTarget.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/behavior/StopAttackingIfTargetInvalid.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/behavior/StopBeingAngryIfTargetDead.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/behavior/StrollAroundPoi.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/behavior/StrollToPoi.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/behavior/StrollToPoiList.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/behavior/Swim.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/behavior/TradeWithVillager.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/behavior/TransportItemsBetweenContainers.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/behavior/TriggerGate.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/behavior/TryFindLand.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/behavior/TryFindLandNearWater.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/behavior/TryFindWater.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/behavior/TryLaySpawnOnFluidNearLand.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/behavior/UpdateActivityFromSchedule.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/behavior/UseBonemeal.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/behavior/ValidateNearbyPoi.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/behavior/VillageBoundRandomStroll.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/behavior/VillagerCalmDown.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/behavior/VillagerGoalPackages.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/behavior/VillagerMakeLove.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/behavior/VillagerPanicTrigger.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/behavior/WakeUp.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/behavior/WorkAtComposter.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/behavior/WorkAtPoi.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/behavior/YieldJobSite.java`.

## `decompiled-server-26.1.2/net/minecraft/world/entity/ai/behavior/declarative`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/behavior/declarative/BehaviorBuilder.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/behavior/declarative/MemoryAccessor.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/behavior/declarative/MemoryCondition.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/behavior/declarative/Trigger.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/behavior/declarative/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/entity/ai/behavior`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/behavior/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/entity/ai/behavior/warden`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/behavior/warden/Digging.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/behavior/warden/Emerging.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/behavior/warden/ForceUnmount.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/behavior/warden/Roar.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/behavior/warden/SetRoarTarget.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/behavior/warden/SetWardenLookTarget.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/behavior/warden/Sniffing.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/behavior/warden/SonicBoom.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/behavior/warden/TryToSniff.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/behavior/warden/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/entity/ai/control`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/control/BodyRotationControl.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/control/Control.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/control/FlyingMoveControl.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/control/JumpControl.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/control/LookControl.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/control/MoveControl.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/control/SmoothSwimmingLookControl.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/control/SmoothSwimmingMoveControl.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/control/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/entity/ai/goal`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/goal/AvoidEntityGoal.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/goal/BegGoal.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/goal/BreakDoorGoal.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/goal/BreathAirGoal.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/goal/BreedGoal.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/goal/CatLieOnBedGoal.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/goal/CatSitOnBlockGoal.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/goal/ClimbOnTopOfPowderSnowGoal.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/goal/DolphinJumpGoal.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/goal/DoorInteractGoal.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/goal/EatBlockGoal.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/goal/FleeSunGoal.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/goal/FloatGoal.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/goal/FollowFlockLeaderGoal.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/goal/FollowMobGoal.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/goal/FollowOwnerGoal.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/goal/FollowParentGoal.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/goal/FollowPlayerRiddenEntityGoal.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/goal/Goal.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/goal/GoalSelector.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/goal/GolemRandomStrollInVillageGoal.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/goal/InteractGoal.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/goal/JumpGoal.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/goal/LandOnOwnersShoulderGoal.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/goal/LeapAtTargetGoal.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/goal/LlamaFollowCaravanGoal.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/goal/LookAtPlayerGoal.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/goal/LookAtTradingPlayerGoal.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/goal/MeleeAttackGoal.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/goal/MoveBackToVillageGoal.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/goal/MoveThroughVillageGoal.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/goal/MoveToBlockGoal.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/goal/MoveTowardsRestrictionGoal.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/goal/MoveTowardsTargetGoal.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/goal/OcelotAttackGoal.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/goal/OfferFlowerGoal.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/goal/OpenDoorGoal.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/goal/PanicGoal.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/goal/PathfindToRaidGoal.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/goal/RandomLookAroundGoal.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/goal/RandomStandGoal.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/goal/RandomStrollGoal.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/goal/RandomSwimmingGoal.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/goal/RangedAttackGoal.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/goal/RangedBowAttackGoal.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/goal/RangedCrossbowAttackGoal.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/goal/RemoveBlockGoal.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/goal/RestrictSunGoal.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/goal/RunAroundLikeCrazyGoal.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/goal/SitWhenOrderedToGoal.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/goal/SpearUseGoal.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/goal/StrollThroughVillageGoal.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/goal/SwellGoal.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/goal/TemptGoal.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/goal/TradeWithPlayerGoal.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/goal/TryFindWaterGoal.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/goal/UseItemGoal.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/goal/WaterAvoidingRandomFlyingGoal.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/goal/WaterAvoidingRandomStrollGoal.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/goal/WrappedGoal.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/goal/ZombieAttackGoal.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/goal/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/entity/ai/goal/target`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/goal/target/DefendVillageTargetGoal.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/goal/target/HurtByTargetGoal.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/goal/target/NearestAttackableTargetGoal.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/goal/target/NearestAttackableWitchTargetGoal.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/goal/target/NearestHealableRaiderTargetGoal.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/goal/target/NonTameRandomTargetGoal.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/goal/target/OwnerHurtByTargetGoal.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/goal/target/OwnerHurtTargetGoal.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/goal/target/ResetUniversalAngerTargetGoal.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/goal/target/TargetGoal.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/goal/target/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/entity/ai/gossip`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/gossip/GossipContainer.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/gossip/GossipType.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/gossip/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/entity/ai/memory`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/memory/ExpirableValue.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/memory/MemoryMap.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/memory/MemoryModuleType.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/memory/MemorySlot.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/memory/MemoryStatus.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/memory/NearestVisibleLivingEntities.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/memory/WalkTarget.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/memory/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/entity/ai/navigation`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/navigation/AmphibiousPathNavigation.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/navigation/FlyingPathNavigation.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/navigation/GroundPathNavigation.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/navigation/PathNavigation.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/navigation/WallClimberNavigation.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/navigation/WaterBoundPathNavigation.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/navigation/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/entity/ai`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/entity/ai/sensing`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/sensing/AdultSensor.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/sensing/AdultSensorAnyType.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/sensing/AxolotlAttackablesSensor.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/sensing/BreezeAttackEntitySensor.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/sensing/DummySensor.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/sensing/FrogAttackablesSensor.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/sensing/GolemSensor.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/sensing/HoglinSpecificSensor.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/sensing/HurtBySensor.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/sensing/IsInWaterSensor.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/sensing/MobSensor.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/sensing/NearestBedSensor.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/sensing/NearestItemSensor.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/sensing/NearestLivingEntitySensor.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/sensing/NearestVisibleLivingEntitySensor.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/sensing/PiglinBruteSpecificSensor.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/sensing/PiglinSpecificSensor.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/sensing/PlayerSensor.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/sensing/SecondaryPoiSensor.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/sensing/Sensing.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/sensing/Sensor.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/sensing/SensorType.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/sensing/TemptingSensor.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/sensing/VillagerBabiesSensor.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/sensing/VillagerHostilesSensor.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/sensing/WardenEntitySensor.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/sensing/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/entity/ai/targeting`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/targeting/TargetingConditions.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/targeting/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/entity/ai/util`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/util/AirAndWaterRandomPos.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/util/AirRandomPos.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/util/DefaultRandomPos.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/util/GoalUtils.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/util/HoverRandomPos.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/util/LandRandomPos.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/util/RandomPos.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/util/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/entity/ai/village`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/village/ReputationEventType.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/village/VillageSiege.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/village/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/entity/ai/village/poi`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/village/poi/PoiManager.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/village/poi/PoiRecord.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/village/poi/PoiSection.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/village/poi/PoiType.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/village/poi/PoiTypes.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ai/village/poi/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/entity/ambient`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ambient/AmbientCreature.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ambient/Bat.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/ambient/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/entity/animal`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/animal/AgeableWaterCreature.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/animal/Animal.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/animal/Bucketable.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/animal/FlyingAnimal.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/animal/TemperatureVariants.java`.

## `decompiled-server-26.1.2/net/minecraft/world/entity/animal/allay`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/animal/allay/Allay.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/animal/allay/AllayAi.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/animal/allay/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/entity/animal/armadillo`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/animal/armadillo/Armadillo.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/animal/armadillo/ArmadilloAi.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/animal/armadillo/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/entity/animal/axolotl`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/animal/axolotl/Axolotl.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/animal/axolotl/AxolotlAi.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/animal/axolotl/PlayDead.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/animal/axolotl/ValidatePlayDead.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/animal/axolotl/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/entity/animal/bee`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/animal/bee/Bee.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/animal/bee/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/entity/animal/camel`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/animal/camel/Camel.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/animal/camel/CamelAi.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/animal/camel/CamelHusk.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/animal/camel/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/entity/animal/chicken`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/animal/chicken/Chicken.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/animal/chicken/ChickenSoundVariant.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/animal/chicken/ChickenSoundVariants.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/animal/chicken/ChickenVariant.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/animal/chicken/ChickenVariants.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/animal/chicken/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/entity/animal/cow`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/animal/cow/AbstractCow.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/animal/cow/Cow.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/animal/cow/CowSoundVariant.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/animal/cow/CowSoundVariants.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/animal/cow/CowVariant.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/animal/cow/CowVariants.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/animal/cow/MushroomCow.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/animal/cow/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/entity/animal/dolphin`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/animal/dolphin/Dolphin.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/animal/dolphin/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/entity/animal/equine`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/animal/equine/AbstractChestedHorse.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/animal/equine/AbstractHorse.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/animal/equine/Donkey.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/animal/equine/Horse.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/animal/equine/Llama.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/animal/equine/Markings.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/animal/equine/Mule.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/animal/equine/SkeletonHorse.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/animal/equine/SkeletonTrapGoal.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/animal/equine/TraderLlama.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/animal/equine/Variant.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/animal/equine/ZombieHorse.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/animal/equine/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/entity/animal/feline`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/animal/feline/Cat.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/animal/feline/CatSoundVariant.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/animal/feline/CatSoundVariants.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/animal/feline/CatVariant.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/animal/feline/CatVariants.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/animal/feline/Ocelot.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/animal/feline/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/entity/animal/fish`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/animal/fish/AbstractFish.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/animal/fish/AbstractSchoolingFish.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/animal/fish/Cod.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/animal/fish/Pufferfish.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/animal/fish/Salmon.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/animal/fish/TropicalFish.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/animal/fish/WaterAnimal.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/animal/fish/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/entity/animal/fox`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/animal/fox/Fox.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/animal/fox/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/entity/animal/frog`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/animal/frog/Frog.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/animal/frog/FrogAi.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/animal/frog/FrogVariant.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/animal/frog/FrogVariants.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/animal/frog/ShootTongue.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/animal/frog/Tadpole.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/animal/frog/TadpoleAi.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/animal/frog/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/entity/animal/goat`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/animal/goat/Goat.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/animal/goat/GoatAi.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/animal/goat/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/entity/animal/golem`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/animal/golem/AbstractGolem.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/animal/golem/CopperGolem.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/animal/golem/CopperGolemAi.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/animal/golem/CopperGolemOxidationLevel.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/animal/golem/CopperGolemOxidationLevels.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/animal/golem/CopperGolemState.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/animal/golem/IronGolem.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/animal/golem/SnowGolem.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/animal/golem/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/entity/animal/happyghast`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/animal/happyghast/HappyGhast.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/animal/happyghast/HappyGhastAi.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/animal/happyghast/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/entity/animal/nautilus`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/animal/nautilus/AbstractNautilus.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/animal/nautilus/Nautilus.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/animal/nautilus/NautilusAi.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/animal/nautilus/ZombieNautilus.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/animal/nautilus/ZombieNautilusAi.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/animal/nautilus/ZombieNautilusVariant.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/animal/nautilus/ZombieNautilusVariants.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/animal/nautilus/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/entity/animal`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/animal/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/entity/animal/panda`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/animal/panda/Panda.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/animal/panda/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/entity/animal/parrot`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/animal/parrot/Parrot.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/animal/parrot/ShoulderRidingEntity.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/animal/parrot/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/entity/animal/pig`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/animal/pig/Pig.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/animal/pig/PigSoundVariant.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/animal/pig/PigSoundVariants.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/animal/pig/PigVariant.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/animal/pig/PigVariants.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/animal/pig/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/entity/animal/polarbear`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/animal/polarbear/PolarBear.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/animal/polarbear/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/entity/animal/rabbit`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/animal/rabbit/Rabbit.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/animal/rabbit/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/entity/animal/sheep`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/animal/sheep/Sheep.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/animal/sheep/SheepColorSpawnRules.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/animal/sheep/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/entity/animal/sniffer`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/animal/sniffer/Sniffer.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/animal/sniffer/SnifferAi.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/animal/sniffer/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/entity/animal/squid`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/animal/squid/GlowSquid.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/animal/squid/Squid.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/animal/squid/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/entity/animal/turtle`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/animal/turtle/Turtle.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/animal/turtle/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/entity/animal/wolf`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/animal/wolf/Wolf.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/animal/wolf/WolfSoundVariant.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/animal/wolf/WolfSoundVariants.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/animal/wolf/WolfVariant.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/animal/wolf/WolfVariants.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/animal/wolf/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/entity/boss/enderdragon`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/boss/enderdragon/DragonFlightHistory.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/boss/enderdragon/EndCrystal.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/boss/enderdragon/EnderDragon.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/boss/enderdragon/EnderDragonPart.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/boss/enderdragon/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/entity/boss/enderdragon/phases`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/boss/enderdragon/phases/AbstractDragonPhaseInstance.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/boss/enderdragon/phases/AbstractDragonSittingPhase.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/boss/enderdragon/phases/DragonChargePlayerPhase.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/boss/enderdragon/phases/DragonDeathPhase.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/boss/enderdragon/phases/DragonHoldingPatternPhase.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/boss/enderdragon/phases/DragonHoverPhase.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/boss/enderdragon/phases/DragonLandingApproachPhase.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/boss/enderdragon/phases/DragonLandingPhase.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/boss/enderdragon/phases/DragonPhaseInstance.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/boss/enderdragon/phases/DragonSittingAttackingPhase.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/boss/enderdragon/phases/DragonSittingFlamingPhase.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/boss/enderdragon/phases/DragonSittingScanningPhase.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/boss/enderdragon/phases/DragonStrafePlayerPhase.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/boss/enderdragon/phases/DragonTakeoffPhase.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/boss/enderdragon/phases/EnderDragonPhase.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/boss/enderdragon/phases/EnderDragonPhaseManager.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/boss/enderdragon/phases/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/entity/boss/wither`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/boss/wither/WitherBoss.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/boss/wither/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/entity/decoration`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/decoration/ArmorStand.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/decoration/BlockAttachedEntity.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/decoration/GlowItemFrame.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/decoration/HangingEntity.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/decoration/ItemFrame.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/decoration/LeashFenceKnotEntity.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/decoration/Mannequin.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/decoration/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/entity/decoration/painting`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/decoration/painting/Painting.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/decoration/painting/PaintingVariant.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/decoration/painting/PaintingVariants.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/decoration/painting/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/entity/item`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/item/FallingBlockEntity.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/item/ItemEntity.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/item/PrimedTnt.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/item/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/entity/monster`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/monster/Blaze.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/monster/Creeper.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/monster/CrossbowAttackMob.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/monster/ElderGuardian.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/monster/EnderMan.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/monster/Endermite.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/monster/Enemy.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/monster/Ghast.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/monster/Giant.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/monster/Guardian.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/monster/MagmaCube.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/monster/Monster.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/monster/PatrollingMonster.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/monster/Phantom.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/monster/RangedAttackMob.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/monster/Ravager.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/monster/Shulker.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/monster/Silverfish.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/monster/Slime.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/monster/Strider.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/monster/Vex.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/monster/Witch.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/monster/Zoglin.java`.

## `decompiled-server-26.1.2/net/minecraft/world/entity/monster/breeze`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/monster/breeze/Breeze.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/monster/breeze/BreezeAi.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/monster/breeze/BreezeUtil.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/monster/breeze/LongJump.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/monster/breeze/Shoot.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/monster/breeze/ShootWhenStuck.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/monster/breeze/Slide.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/monster/breeze/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/entity/monster/creaking`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/monster/creaking/Creaking.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/monster/creaking/CreakingAi.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/monster/creaking/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/entity/monster/hoglin`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/monster/hoglin/Hoglin.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/monster/hoglin/HoglinAi.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/monster/hoglin/HoglinBase.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/monster/hoglin/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/entity/monster/illager`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/monster/illager/AbstractIllager.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/monster/illager/Evoker.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/monster/illager/Illusioner.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/monster/illager/Pillager.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/monster/illager/SpellcasterIllager.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/monster/illager/Vindicator.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/monster/illager/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/entity/monster`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/monster/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/entity/monster/piglin`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/monster/piglin/AbstractPiglin.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/monster/piglin/Piglin.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/monster/piglin/PiglinAi.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/monster/piglin/PiglinArmPose.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/monster/piglin/PiglinBrute.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/monster/piglin/PiglinBruteAi.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/monster/piglin/RememberIfHoglinWasKilled.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/monster/piglin/StartAdmiringItemIfSeen.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/monster/piglin/StartHuntingHoglin.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/monster/piglin/StopAdmiringIfItemTooFarAway.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/monster/piglin/StopAdmiringIfTiredOfTryingToReachItem.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/monster/piglin/StopHoldingItemIfNoLongerAdmiring.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/monster/piglin/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/entity/monster/skeleton`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/monster/skeleton/AbstractSkeleton.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/monster/skeleton/Bogged.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/monster/skeleton/Parched.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/monster/skeleton/Skeleton.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/monster/skeleton/Stray.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/monster/skeleton/WitherSkeleton.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/monster/skeleton/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/entity/monster/spider`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/monster/spider/CaveSpider.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/monster/spider/Spider.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/monster/spider/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/entity/monster/warden`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/monster/warden/AngerLevel.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/monster/warden/AngerManagement.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/monster/warden/Warden.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/monster/warden/WardenAi.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/monster/warden/WardenSpawnTracker.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/monster/warden/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/entity/monster/zombie`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/monster/zombie/Drowned.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/monster/zombie/Husk.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/monster/zombie/Zombie.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/monster/zombie/ZombieVillager.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/monster/zombie/ZombifiedPiglin.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/monster/zombie/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/entity/npc`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/npc/CatSpawner.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/npc/ClientSideMerchant.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/npc/InventoryCarrier.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/npc/Npc.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/npc/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/entity/npc/villager`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/npc/villager/AbstractVillager.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/npc/villager/Villager.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/npc/villager/VillagerData.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/npc/villager/VillagerDataHolder.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/npc/villager/VillagerProfession.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/npc/villager/VillagerType.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/npc/villager/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/entity/npc/wanderingtrader`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/npc/wanderingtrader/WanderingTrader.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/npc/wanderingtrader/WanderingTraderSpawner.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/npc/wanderingtrader/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/entity`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/entity/player`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/player/Abilities.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/player/ChatVisiblity.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/player/Input.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/player/Inventory.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/player/Player.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/player/PlayerEquipment.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/player/PlayerModelPart.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/player/PlayerModelType.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/player/PlayerSkin.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/player/ProfileKeyPair.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/player/ProfilePublicKey.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/player/StackedContents.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/player/StackedItemContents.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/player/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/entity/projectile`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/projectile/EvokerFangs.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/projectile/EyeOfEnder.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/projectile/FireworkRocketEntity.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/projectile/FishingHook.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/projectile/ItemSupplier.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/projectile/LlamaSpit.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/projectile/Projectile.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/projectile/ProjectileDeflection.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/projectile/ProjectileUtil.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/projectile/ShulkerBullet.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/projectile/ThrowableProjectile.java`.

## `decompiled-server-26.1.2/net/minecraft/world/entity/projectile/arrow`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/projectile/arrow/AbstractArrow.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/projectile/arrow/Arrow.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/projectile/arrow/SpectralArrow.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/projectile/arrow/ThrownTrident.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/projectile/arrow/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/entity/projectile/hurtingprojectile`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/projectile/hurtingprojectile/AbstractHurtingProjectile.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/projectile/hurtingprojectile/DragonFireball.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/projectile/hurtingprojectile/Fireball.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/projectile/hurtingprojectile/LargeFireball.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/projectile/hurtingprojectile/SmallFireball.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/projectile/hurtingprojectile/WitherSkull.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/projectile/hurtingprojectile/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/entity/projectile/hurtingprojectile/windcharge`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/projectile/hurtingprojectile/windcharge/AbstractWindCharge.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/projectile/hurtingprojectile/windcharge/BreezeWindCharge.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/projectile/hurtingprojectile/windcharge/WindCharge.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/projectile/hurtingprojectile/windcharge/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/entity/projectile`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/projectile/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/entity/projectile/throwableitemprojectile`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/projectile/throwableitemprojectile/AbstractThrownPotion.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/projectile/throwableitemprojectile/Snowball.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/projectile/throwableitemprojectile/ThrowableItemProjectile.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/projectile/throwableitemprojectile/ThrownEgg.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/projectile/throwableitemprojectile/ThrownEnderpearl.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/projectile/throwableitemprojectile/ThrownExperienceBottle.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/projectile/throwableitemprojectile/ThrownLingeringPotion.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/projectile/throwableitemprojectile/ThrownSplashPotion.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/projectile/throwableitemprojectile/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/entity/raid`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/raid/Raid.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/raid/Raider.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/raid/Raids.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/raid/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/entity/schedule`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/schedule/Activity.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/schedule/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/entity/variant`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/variant/BiomeCheck.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/variant/ModelAndTexture.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/variant/MoonBrightnessCheck.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/variant/PriorityProvider.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/variant/SpawnCondition.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/variant/SpawnConditions.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/variant/SpawnContext.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/variant/SpawnPrioritySelectors.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/variant/StructureCheck.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/variant/VariantUtils.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/variant/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/entity/vehicle`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/vehicle/ContainerEntity.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/vehicle/DismountHelper.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/vehicle/VehicleEntity.java`.

## `decompiled-server-26.1.2/net/minecraft/world/entity/vehicle/boat`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/vehicle/boat/AbstractBoat.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/vehicle/boat/AbstractChestBoat.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/vehicle/boat/Boat.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/vehicle/boat/ChestBoat.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/vehicle/boat/ChestRaft.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/vehicle/boat/Raft.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/vehicle/boat/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/entity/vehicle/minecart`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/vehicle/minecart/AbstractMinecart.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/vehicle/minecart/AbstractMinecartContainer.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/vehicle/minecart/Minecart.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/vehicle/minecart/MinecartBehavior.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/vehicle/minecart/MinecartChest.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/vehicle/minecart/MinecartCommandBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/vehicle/minecart/MinecartFurnace.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/vehicle/minecart/MinecartHopper.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/vehicle/minecart/MinecartSpawner.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/vehicle/minecart/MinecartTNT.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/vehicle/minecart/NewMinecartBehavior.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/vehicle/minecart/OldMinecartBehavior.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/vehicle/minecart/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/entity/vehicle`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/entity/vehicle/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/flag`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/flag/FeatureElement.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/flag/FeatureFlag.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/flag/FeatureFlagRegistry.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/flag/FeatureFlagSet.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/flag/FeatureFlagUniverse.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/flag/FeatureFlags.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/flag/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/food`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/food/FoodConstants.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/food/FoodData.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/food/FoodProperties.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/food/Foods.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/food/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/inventory`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/inventory/AbstractContainerMenu.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/inventory/AbstractCraftingMenu.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/inventory/AbstractFurnaceMenu.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/inventory/AbstractMountInventoryMenu.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/inventory/AnvilMenu.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/inventory/ArmorSlot.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/inventory/BeaconMenu.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/inventory/BlastFurnaceMenu.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/inventory/BrewingStandMenu.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/inventory/CartographyTableMenu.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/inventory/ChestMenu.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/inventory/ClickAction.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/inventory/ContainerData.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/inventory/ContainerInput.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/inventory/ContainerLevelAccess.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/inventory/ContainerListener.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/inventory/ContainerSynchronizer.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/inventory/CrafterMenu.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/inventory/CrafterSlot.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/inventory/CraftingContainer.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/inventory/CraftingMenu.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/inventory/DataSlot.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/inventory/DispenserMenu.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/inventory/EnchantmentMenu.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/inventory/FurnaceFuelSlot.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/inventory/FurnaceMenu.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/inventory/FurnaceResultSlot.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/inventory/GrindstoneMenu.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/inventory/HopperMenu.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/inventory/HorseInventoryMenu.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/inventory/InventoryMenu.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/inventory/ItemCombinerMenu.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/inventory/ItemCombinerMenuSlotDefinition.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/inventory/LecternMenu.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/inventory/LoomMenu.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/inventory/MenuConstructor.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/inventory/MenuType.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/inventory/MerchantContainer.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/inventory/MerchantMenu.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/inventory/MerchantResultSlot.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/inventory/NautilusInventoryMenu.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/inventory/NonInteractiveResultSlot.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/inventory/PlayerEnderChestContainer.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/inventory/RecipeBookMenu.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/inventory/RecipeBookType.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/inventory/RecipeCraftingHolder.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/inventory/RemoteSlot.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/inventory/ResultContainer.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/inventory/ResultSlot.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/inventory/ShulkerBoxMenu.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/inventory/ShulkerBoxSlot.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/inventory/SimpleContainerData.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/inventory/Slot.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/inventory/SlotRange.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/inventory/SlotRanges.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/inventory/SmithingMenu.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/inventory/SmokerMenu.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/inventory/StackedContentsCompatible.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/inventory/StonecutterMenu.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/inventory/TransientCraftingContainer.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/inventory/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/inventory/tooltip`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/inventory/tooltip/BundleTooltip.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/inventory/tooltip/TooltipComponent.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/inventory/tooltip/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/item`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/AdventureModePredicate.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/AirItem.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/ArmorStandItem.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/ArrowItem.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/AxeItem.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/BannerItem.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/BedItem.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/BlockItem.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/BoatItem.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/BoneMealItem.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/BottleItem.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/BowItem.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/BrushItem.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/BucketItem.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/BundleItem.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/CompassItem.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/CreativeModeTab.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/CreativeModeTabs.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/CrossbowItem.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/DebugStickItem.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/DiscFragmentItem.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/DispensibleContainerItem.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/DoubleHighBlockItem.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/DyeColor.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/DyeItem.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/EggItem.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/EmptyMapItem.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/EndCrystalItem.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/EnderEyeItem.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/EnderpearlItem.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/ExperienceBottleItem.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/FireChargeItem.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/FireworkRocketItem.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/FishingRodItem.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/FlintAndSteelItem.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/FoodOnAStickItem.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/GameMasterBlockItem.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/GlowInkSacItem.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/HangingEntityItem.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/HangingSignItem.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/HoeItem.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/HoneycombItem.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/InkSacItem.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/Instrument.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/InstrumentItem.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/Instruments.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/Item.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/ItemCooldowns.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/ItemDisplayContext.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/ItemFrameItem.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/ItemInstance.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/ItemStack.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/ItemStackLinkedSet.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/ItemStackTemplate.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/ItemUseAnimation.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/ItemUtils.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/Items.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/JukeboxPlayable.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/JukeboxSong.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/JukeboxSongPlayer.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/JukeboxSongs.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/KnowledgeBookItem.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/LeadItem.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/LingeringPotionItem.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/MaceItem.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/MapItem.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/MinecartItem.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/MobBucketItem.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/NameTagItem.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/PlaceOnWaterBlockItem.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/PlayerHeadItem.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/PotionItem.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/ProjectileItem.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/ProjectileWeaponItem.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/Rarity.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/ScaffoldingBlockItem.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/ServerItemCooldowns.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/ShearsItem.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/ShieldItem.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/ShovelItem.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/SignApplicator.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/SignItem.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/SmithingTemplateItem.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/SnowballItem.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/SolidBucketItem.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/SpawnEggItem.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/SpectralArrowItem.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/SplashPotionItem.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/SpyglassItem.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/StandingAndWallBlockItem.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/SwingAnimationType.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/ThrowablePotionItem.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/TippedArrowItem.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/ToolMaterial.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/TooltipFlag.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/TridentItem.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/WeatheringCopperItems.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/WindChargeItem.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/WritableBookItem.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/WrittenBookItem.java`.

## `decompiled-server-26.1.2/net/minecraft/world/item/alchemy`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/alchemy/Potion.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/alchemy/PotionBrewing.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/alchemy/PotionContents.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/alchemy/Potions.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/alchemy/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/item/component`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/component/AttackRange.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/component/Bees.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/component/BlockItemStateProperties.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/component/BlocksAttacks.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/component/BookContent.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/component/BundleContents.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/component/ChargedProjectiles.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/component/Consumable.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/component/ConsumableListener.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/component/Consumables.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/component/CustomData.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/component/CustomModelData.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/component/DamageResistant.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/component/DeathProtection.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/component/DebugStickState.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/component/DyedItemColor.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/component/FireworkExplosion.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/component/Fireworks.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/component/InstrumentComponent.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/component/ItemAttributeModifiers.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/component/ItemContainerContents.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/component/ItemLore.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/component/KineticWeapon.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/component/LodestoneTracker.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/component/MapDecorations.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/component/MapItemColor.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/component/MapPostProcessing.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/component/OminousBottleAmplifier.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/component/PiercingWeapon.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/component/ProvidesTrimMaterial.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/component/ResolvableProfile.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/component/SeededContainerLoot.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/component/SuspiciousStewEffects.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/component/SwingAnimation.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/component/Tool.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/component/TooltipDisplay.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/component/TooltipProvider.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/component/TypedEntityData.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/component/UseCooldown.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/component/UseEffects.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/component/UseRemainder.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/component/Weapon.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/component/WritableBookContent.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/component/WrittenBookContent.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/component/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/item/consume_effects`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/consume_effects/ApplyStatusEffectsConsumeEffect.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/consume_effects/ClearAllStatusEffectsConsumeEffect.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/consume_effects/ConsumeEffect.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/consume_effects/PlaySoundConsumeEffect.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/consume_effects/RemoveStatusEffectsConsumeEffect.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/consume_effects/TeleportRandomlyConsumeEffect.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/consume_effects/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/item/context`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/context/BlockPlaceContext.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/context/DirectionalPlaceContext.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/context/UseOnContext.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/context/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/item/crafting`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/crafting/AbstractCookingRecipe.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/crafting/BannerDuplicateRecipe.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/crafting/BlastingRecipe.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/crafting/BookCloningRecipe.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/crafting/CampfireCookingRecipe.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/crafting/CookingBookCategory.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/crafting/CraftingBookCategory.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/crafting/CraftingInput.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/crafting/CraftingRecipe.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/crafting/CustomRecipe.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/crafting/DecoratedPotRecipe.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/crafting/DyeRecipe.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/crafting/ExtendedRecipeBookCategory.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/crafting/FireworkRocketRecipe.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/crafting/FireworkStarFadeRecipe.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/crafting/FireworkStarRecipe.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/crafting/ImbueRecipe.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/crafting/Ingredient.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/crafting/MapExtendingRecipe.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/crafting/NormalCraftingRecipe.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/crafting/PlacementInfo.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/crafting/Recipe.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/crafting/RecipeAccess.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/crafting/RecipeBookCategories.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/crafting/RecipeBookCategory.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/crafting/RecipeCache.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/crafting/RecipeHolder.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/crafting/RecipeInput.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/crafting/RecipeManager.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/crafting/RecipeMap.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/crafting/RecipePropertySet.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/crafting/RecipeSerializer.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/crafting/RecipeSerializers.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/crafting/RecipeType.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/crafting/RepairItemRecipe.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/crafting/SelectableRecipe.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/crafting/ShapedRecipe.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/crafting/ShapedRecipePattern.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/crafting/ShapelessRecipe.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/crafting/ShieldDecorationRecipe.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/crafting/SimpleSmithingRecipe.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/crafting/SingleItemRecipe.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/crafting/SingleRecipeInput.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/crafting/SmeltingRecipe.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/crafting/SmithingRecipe.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/crafting/SmithingRecipeInput.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/crafting/SmithingTransformRecipe.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/crafting/SmithingTrimRecipe.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/crafting/SmokingRecipe.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/crafting/StonecutterRecipe.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/crafting/TransmuteRecipe.java`.

## `decompiled-server-26.1.2/net/minecraft/world/item/crafting/display`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/crafting/display/DisplayContentsFactory.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/crafting/display/FurnaceRecipeDisplay.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/crafting/display/RecipeDisplay.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/crafting/display/RecipeDisplayEntry.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/crafting/display/RecipeDisplayId.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/crafting/display/RecipeDisplays.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/crafting/display/ShapedCraftingRecipeDisplay.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/crafting/display/ShapelessCraftingRecipeDisplay.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/crafting/display/SlotDisplay.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/crafting/display/SlotDisplayContext.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/crafting/display/SlotDisplays.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/crafting/display/SmithingRecipeDisplay.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/crafting/display/StonecutterRecipeDisplay.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/crafting/display/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/item/crafting`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/crafting/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/item/enchantment`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/enchantment/ConditionalEffect.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/enchantment/Enchantable.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/enchantment/EnchantedItemInUse.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/enchantment/Enchantment.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/enchantment/EnchantmentEffectComponents.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/enchantment/EnchantmentHelper.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/enchantment/EnchantmentInstance.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/enchantment/EnchantmentTarget.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/enchantment/Enchantments.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/enchantment/ItemEnchantments.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/enchantment/LevelBasedValue.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/enchantment/Repairable.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/enchantment/TargetedConditionalEffect.java`.

## `decompiled-server-26.1.2/net/minecraft/world/item/enchantment/effects`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/enchantment/effects/AddValue.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/enchantment/effects/AllOf.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/enchantment/effects/ApplyEntityImpulse.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/enchantment/effects/ApplyExhaustion.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/enchantment/effects/ApplyMobEffect.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/enchantment/effects/ChangeItemDamage.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/enchantment/effects/DamageEntity.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/enchantment/effects/DamageImmunity.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/enchantment/effects/EnchantmentAttributeEffect.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/enchantment/effects/EnchantmentEntityEffect.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/enchantment/effects/EnchantmentLocationBasedEffect.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/enchantment/effects/EnchantmentValueEffect.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/enchantment/effects/ExplodeEffect.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/enchantment/effects/Ignite.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/enchantment/effects/MultiplyValue.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/enchantment/effects/PlaySoundEffect.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/enchantment/effects/RemoveBinomial.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/enchantment/effects/ReplaceBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/enchantment/effects/ReplaceDisk.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/enchantment/effects/RunFunction.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/enchantment/effects/ScaleExponentially.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/enchantment/effects/SetBlockProperties.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/enchantment/effects/SetValue.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/enchantment/effects/SpawnParticlesEffect.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/enchantment/effects/SummonEntityEffect.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/enchantment/effects/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/item/enchantment`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/enchantment/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/item/enchantment/providers`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/enchantment/providers/EnchantmentProvider.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/enchantment/providers/EnchantmentProviderTypes.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/enchantment/providers/EnchantmentsByCost.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/enchantment/providers/EnchantmentsByCostWithDifficulty.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/enchantment/providers/SingleEnchantment.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/enchantment/providers/VanillaEnchantmentProviders.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/enchantment/providers/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/item/equipment`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/equipment/AllowedEntitiesProvider.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/equipment/ArmorMaterial.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/equipment/ArmorMaterials.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/equipment/ArmorType.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/equipment/EquipmentAsset.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/equipment/EquipmentAssets.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/equipment/Equippable.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/equipment/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/item/equipment/trim`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/equipment/trim/ArmorTrim.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/equipment/trim/MaterialAssetGroup.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/equipment/trim/TrimMaterial.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/equipment/trim/TrimMaterials.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/equipment/trim/TrimPattern.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/equipment/trim/TrimPatterns.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/equipment/trim/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/item`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/item/slot`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/slot/CompositeSlotSource.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/slot/ContentsSlotSource.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/slot/EmptySlotSource.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/slot/FilteredSlotSource.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/slot/GroupSlotSource.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/slot/LimitSlotSource.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/slot/RangeSlotSource.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/slot/SlotCollection.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/slot/SlotSource.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/slot/SlotSources.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/slot/TransformedSlotSource.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/slot/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/item/trading`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/trading/ItemCost.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/trading/Merchant.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/trading/MerchantOffer.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/trading/MerchantOffers.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/trading/TradeCost.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/trading/TradeRebalanceVillagerTrades.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/trading/TradeSet.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/trading/TradeSets.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/trading/VillagerTrade.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/trading/VillagerTrades.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/item/trading/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/level`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/BaseCommandBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/BaseSpawner.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/BlockAndLightGetter.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/BlockCollisions.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/BlockEventData.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/BlockGetter.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/CardinalLighting.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/ChunkPos.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/ClipBlockStateContext.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/ClipContext.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/CollisionGetter.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/ColorMapColorUtil.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/ColorResolver.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/CommonLevelAccessor.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/CustomSpawner.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/DataPackConfig.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/DryFoliageColor.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/EmptyBlockGetter.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/EntityBasedExplosionDamageCalculator.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/EntityGetter.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/Explosion.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/ExplosionDamageCalculator.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/FoliageColor.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/GameType.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/GrassColor.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/ItemLike.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/Level.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/LevelAccessor.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/LevelHeightAccessor.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/LevelReader.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/LevelSettings.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/LevelSimulatedRW.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/LevelSimulatedReader.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/LevelWriter.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/LightLayer.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/LocalMobCapCalculator.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/MoonPhase.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/NaturalSpawner.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/NoiseColumn.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/PathNavigationRegion.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/PotentialCalculator.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/ScheduledTickAccess.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/ServerExplosion.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/ServerLevelAccessor.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/SignalGetter.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/SimpleExplosionDamageCalculator.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/SpawnData.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/Spawner.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/StructureManager.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/TicketStorage.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/WorldDataConfiguration.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/WorldGenLevel.java`.

## `decompiled-server-26.1.2/net/minecraft/world/level/biome`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/biome/Biome.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/biome/BiomeGenerationSettings.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/biome/BiomeManager.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/biome/BiomeResolver.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/biome/BiomeSource.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/biome/BiomeSources.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/biome/BiomeSpecialEffects.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/biome/Biomes.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/biome/CheckerboardColumnBiomeSource.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/biome/Climate.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/biome/FeatureSorter.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/biome/FixedBiomeSource.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/biome/MobSpawnSettings.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/biome/MultiNoiseBiomeSource.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/biome/MultiNoiseBiomeSourceParameterList.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/biome/MultiNoiseBiomeSourceParameterLists.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/biome/OverworldBiomeBuilder.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/biome/TheEndBiomeSource.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/biome/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/level/block`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/AbstractBannerBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/AbstractCandleBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/AbstractCauldronBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/AbstractChestBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/AbstractFurnaceBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/AbstractSkullBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/AirBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/AmethystBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/AmethystClusterBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/AnvilBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/AttachedStemBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/AzaleaBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/BambooSaplingBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/BambooStalkBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/BannerBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/BarrelBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/BarrierBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/BaseCoralFanBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/BaseCoralPlantBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/BaseCoralPlantTypeBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/BaseCoralWallFanBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/BaseEntityBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/BaseFireBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/BasePressurePlateBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/BaseRailBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/BaseTorchBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/BeaconBeamBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/BeaconBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/BedBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/BeehiveBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/BeetrootBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/BellBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/BigDripleafBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/BigDripleafStemBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/BlastFurnaceBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/Block.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/BlockTypes.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/Blocks.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/BonemealableBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/BonemealableFeaturePlacerBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/BrewingStandBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/BrushableBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/BubbleColumnBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/BucketPickup.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/BuddingAmethystBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/BushBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/ButtonBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/CactusBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/CactusFlowerBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/CakeBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/CalibratedSculkSensorBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/CampfireBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/CandleBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/CandleCakeBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/CarpetBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/CarrotBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/CartographyTableBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/CarvedPumpkinBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/CauldronBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/CaveVines.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/CaveVinesBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/CaveVinesPlantBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/CeilingHangingSignBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/ChainBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/ChangeOverTimeBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/ChestBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/ChiseledBookShelfBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/ChorusFlowerBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/ChorusPlantBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/CocoaBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/ColoredFallingBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/CommandBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/ComparatorBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/ComposterBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/ConcretePowderBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/ConduitBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/CopperBulbBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/CopperChestBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/CopperGolemStatueBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/CoralBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/CoralFanBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/CoralPlantBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/CoralWallFanBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/CrafterBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/CraftingTableBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/CreakingHeartBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/CropBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/CrossCollisionBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/CryingObsidianBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/DaylightDetectorBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/DecoratedPotBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/DetectorRailBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/DiodeBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/DirectionalBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/DirtPathBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/DispenserBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/DoorBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/DoubleBlockCombiner.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/DoublePlantBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/DragonEggBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/DriedGhastBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/DropExperienceBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/DropperBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/DryVegetationBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/EnchantingTableBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/EndGatewayBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/EndPortalBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/EndPortalFrameBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/EndRodBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/EnderChestBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/EntityBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/EyeblossomBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/FaceAttachedHorizontalDirectionalBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/Fallable.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/FallingBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/FarmlandBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/FenceBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/FenceGateBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/FireBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/FireflyBushBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/FlowerBedBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/FlowerBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/FlowerPotBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/FrogspawnBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/FrostedIceBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/FurnaceBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/GameMasterBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/GlazedTerracottaBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/GlowLichenBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/GrassBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/GrindstoneBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/GrowingPlantBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/GrowingPlantBodyBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/GrowingPlantHeadBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/HalfTransparentBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/HangingMossBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/HangingRootsBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/HangingSignBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/HayBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/HeavyCoreBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/HoneyBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/HopperBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/HorizontalDirectionalBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/HugeMushroomBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/IceBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/InfestedBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/InfestedRotatedPillarBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/IronBarsBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/JigsawBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/JukeboxBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/KelpBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/KelpPlantBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/LadderBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/LanternBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/LavaCauldronBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/LayeredCauldronBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/LeafLitterBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/LeavesBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/LecternBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/LevelEvent.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/LeverBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/LightBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/LightningRodBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/LilyPadBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/LiquidBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/LiquidBlockContainer.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/LoomBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/MagmaBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/MangroveLeavesBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/MangrovePropaguleBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/MangroveRootsBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/Mirror.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/MossyCarpetBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/MudBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/MultifaceBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/MultifaceSpreadeableBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/MultifaceSpreader.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/MushroomBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/MyceliumBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/NetherFungusBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/NetherPortalBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/NetherRootsBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/NetherSproutsBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/NetherVines.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/NetherWartBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/NetherrackBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/NoteBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/NyliumBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/ObserverBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/PiglinWallSkullBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/PipeBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/PitcherCropBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/PlainSignBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/PlayerHeadBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/PlayerWallHeadBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/PointedDripstoneBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/Portal.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/PotatoBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/PowderSnowBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/PoweredBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/PoweredRailBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/PressurePlateBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/PumpkinBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/RailBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/RailState.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/RedStoneOreBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/RedStoneWireBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/RedstoneLampBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/RedstoneTorchBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/RedstoneWallTorchBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/RenderShape.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/RepeaterBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/RespawnAnchorBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/RodBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/RootedDirtBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/RotatedPillarBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/Rotation.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/SandBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/SaplingBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/ScaffoldingBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/SculkBehaviour.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/SculkBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/SculkCatalystBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/SculkSensorBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/SculkShriekerBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/SculkSpreader.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/SculkVeinBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/SeaPickleBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/SeagrassBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/SegmentableBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/SelectableSlotContainer.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/ShelfBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/ShortDryGrassBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/ShulkerBoxBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/SideChainPartBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/SignBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/SimpleWaterloggedBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/SkullBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/SlabBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/SlimeBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/SmallDripleafBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/SmithingTableBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/SmokerBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/SnifferEggBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/SnowLayerBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/SnowyBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/SoulFireBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/SoulSandBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/SoundType.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/SpawnerBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/SpongeBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/SporeBlossomBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/SpreadingSnowyBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/StainedGlassBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/StainedGlassPaneBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/StairBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/StandingSignBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/StemBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/StonecutterBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/StructureBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/StructureVoidBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/SugarCaneBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/SupportType.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/SuspiciousEffectHolder.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/SweetBerryBushBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/TallDryGrassBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/TallFlowerBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/TallGrassBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/TallSeagrassBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/TargetBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/TestBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/TestInstanceBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/TintedGlassBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/TintedParticleLeavesBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/TntBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/TorchBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/TorchflowerCropBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/TransparentBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/TrapDoorBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/TrappedChestBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/TrialSpawnerBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/TripWireBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/TripWireHookBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/TurtleEggBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/TwistingVinesBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/TwistingVinesPlantBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/UntintedParticleLeavesBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/VaultBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/VegetationBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/VineBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/WallBannerBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/WallBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/WallHangingSignBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/WallSignBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/WallSkullBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/WallTorchBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/WaterloggedTransparentBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/WeatheringCopper.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/WeatheringCopperBarsBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/WeatheringCopperBlocks.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/WeatheringCopperBulbBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/WeatheringCopperChainBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/WeatheringCopperChestBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/WeatheringCopperDoorBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/WeatheringCopperFullBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/WeatheringCopperGolemStatueBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/WeatheringCopperGrateBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/WeatheringCopperSlabBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/WeatheringCopperStairBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/WeatheringCopperTrapDoorBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/WeatheringLanternBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/WeatheringLightningRodBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/WebBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/WeepingVinesBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/WeepingVinesPlantBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/WeightedPressurePlateBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/WetSpongeBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/WitherRoseBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/WitherSkullBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/WitherWallSkullBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/WoolCarpetBlock.java`.

## `decompiled-server-26.1.2/net/minecraft/world/level/block/entity`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/AbstractFurnaceBlockEntity.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/BannerBlockEntity.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/BannerPattern.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/BannerPatternLayers.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/BannerPatterns.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/BarrelBlockEntity.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/BaseContainerBlockEntity.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/BeaconBeamOwner.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/BeaconBlockEntity.java`.
- [x] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/BedBlockEntity.java`. — Audited against Java 26.1.2: the entity stores only bed color from `BedBlock`, has no additional NBT fields, and sends the standard block-entity data packet. Rust parity is `BedBlockEntity::from_block_state()` + empty `save_additional()` + `BlockEntity::get_update_packet()`; covered by `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 bed_block_entity_is_color_only_placeholder`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/BeehiveBlockEntity.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/BellBlockEntity.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/BlastFurnaceBlockEntity.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/BlockEntity.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/BlockEntityTicker.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/BlockEntityType.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/BoundingBoxRenderable.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/BrewingStandBlockEntity.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/BrushableBlockEntity.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/CalibratedSculkSensorBlockEntity.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/CampfireBlockEntity.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/ChestBlockEntity.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/ChestLidController.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/ChiseledBookShelfBlockEntity.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/CommandBlockEntity.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/ComparatorBlockEntity.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/ConduitBlockEntity.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/ContainerOpenersCounter.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/CopperGolemStatueBlockEntity.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/CrafterBlockEntity.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/CreakingHeartBlockEntity.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/DaylightDetectorBlockEntity.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/DecoratedPotBlockEntity.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/DecoratedPotPattern.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/DecoratedPotPatterns.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/DispenserBlockEntity.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/DropperBlockEntity.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/EnchantingTableBlockEntity.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/EnderChestBlockEntity.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/FuelValues.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/FurnaceBlockEntity.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/HangingSignBlockEntity.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/Hopper.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/HopperBlockEntity.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/JigsawBlockEntity.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/JukeboxBlockEntity.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/LecternBlockEntity.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/LidBlockEntity.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/ListBackedContainer.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/PotDecorations.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/RandomizableContainerBlockEntity.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/SculkCatalystBlockEntity.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/SculkSensorBlockEntity.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/SculkShriekerBlockEntity.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/ShelfBlockEntity.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/ShulkerBoxBlockEntity.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/SignBlockEntity.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/SignText.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/SkullBlockEntity.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/SmokerBlockEntity.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/SpawnerBlockEntity.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/StructureBlockEntity.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/TestBlockEntity.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/TestInstanceBlockEntity.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/TheEndGatewayBlockEntity.java`.
- [x] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/TheEndPortalBlockEntity.java`. — Audited against Java 26.1.2: the block entity has no custom NBT, uses `BlockEntityType.END_PORTAL`, and renders only Y-axis faces. Rust parity is `EndPortalBlockEntity::save_additional()` + `EndPortalBlockEntity::should_render_face()` + `BlockEntity::get_update_packet()`; covered by `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 end_portal_block_entity_is_zero_data_portal_placeholder`.
- [x] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/TickingBlockEntity.java`. — Audited against Java 26.1.2: the interface exposes `tick()`, `isRemoved()`, `getPos()`, and `getType()` for level/chunk scheduler wrappers. Rust parity is `TickingBlockEntity::tick()`, `is_removed()`, `pos()`, and `type_key()` delegating to the wrapped `BlockEntity`; covered by `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 ticking_block_entity_wrapper_exposes_scheduler_shape`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/TrappedChestBlockEntity.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/TrialSpawnerBlockEntity.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/trialspawner`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/trialspawner/PlayerDetector.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/trialspawner/TrialSpawner.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/trialspawner/TrialSpawnerConfig.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/trialspawner/TrialSpawnerConfigs.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/trialspawner/TrialSpawnerState.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/trialspawner/TrialSpawnerStateData.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/trialspawner/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/vault`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/vault/VaultBlockEntity.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/vault/VaultClientData.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/vault/VaultConfig.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/vault/VaultServerData.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/vault/VaultSharedData.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/vault/VaultState.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/vault/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/level/block/grower`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/grower/TreeGrower.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/grower/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/level/block`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/level/block/piston`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/piston/MovingPistonBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/piston/PistonBaseBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/piston/PistonHeadBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/piston/PistonMath.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/piston/PistonMovingBlockEntity.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/piston/PistonStructureResolver.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/piston/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/level/block/sounds`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/sounds/AmbientDesertBlockSoundsPlayer.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/sounds/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/level/block/state`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/state/BlockBehaviour.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/state/BlockState.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/state/StateDefinition.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/state/StateHolder.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/state/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/level/block/state/pattern`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/state/pattern/BlockInWorld.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/state/pattern/BlockPattern.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/state/pattern/BlockPatternBuilder.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/state/pattern/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/level/block/state/predicate`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/state/predicate/BlockPredicate.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/state/predicate/BlockStatePredicate.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/state/predicate/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/level/block/state/properties`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/state/properties/AttachFace.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/state/properties/BambooLeaves.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/state/properties/BedPart.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/state/properties/BellAttachType.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/state/properties/BlockSetType.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/state/properties/BlockStateProperties.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/state/properties/BooleanProperty.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/state/properties/ChestType.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/state/properties/ComparatorMode.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/state/properties/CreakingHeartState.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/state/properties/DoorHingeSide.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/state/properties/DoubleBlockHalf.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/state/properties/DripstoneThickness.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/state/properties/EnumProperty.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/state/properties/Half.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/state/properties/IntegerProperty.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/state/properties/NoteBlockInstrument.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/state/properties/PistonType.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/state/properties/Property.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/state/properties/RailShape.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/state/properties/RedstoneSide.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/state/properties/RotationSegment.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/state/properties/SculkSensorPhase.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/state/properties/SideChainPart.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/state/properties/SlabType.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/state/properties/StairsShape.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/state/properties/StructureMode.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/state/properties/TestBlockMode.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/state/properties/Tilt.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/state/properties/WallSide.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/state/properties/WoodType.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/state/properties/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/level/border`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/border/BorderChangeListener.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/border/BorderStatus.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/border/WorldBorder.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/border/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/level/chunk`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/chunk/BlockColumn.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/chunk/BulkSectionAccess.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/chunk/CarvingMask.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/chunk/ChunkAccess.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/chunk/ChunkGenerator.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/chunk/ChunkGeneratorStructureState.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/chunk/ChunkGenerators.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/chunk/ChunkSource.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/chunk/Configuration.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/chunk/DataLayer.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/chunk/EmptyLevelChunk.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/chunk/GlobalPalette.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/chunk/HashMapPalette.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/chunk/ImposterProtoChunk.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/chunk/LevelChunk.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/chunk/LevelChunkSection.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/chunk/LightChunk.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/chunk/LightChunkGetter.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/chunk/LinearPalette.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/chunk/MissingPaletteEntryException.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/chunk/Palette.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/chunk/PaletteResize.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/chunk/PalettedContainer.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/chunk/PalettedContainerFactory.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/chunk/PalettedContainerRO.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/chunk/ProtoChunk.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/chunk/SingleValuePalette.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/chunk/Strategy.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/chunk/StructureAccess.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/chunk/UpgradeData.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/chunk/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/level/chunk/status`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/chunk/status/ChunkDependencies.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/chunk/status/ChunkPyramid.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/chunk/status/ChunkStatus.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/chunk/status/ChunkStatusTask.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/chunk/status/ChunkStatusTasks.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/chunk/status/ChunkStep.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/chunk/status/ChunkType.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/chunk/status/WorldGenContext.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/chunk/status/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/level/chunk/storage`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/chunk/storage/ChunkIOErrorReporter.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/chunk/storage/ChunkScanAccess.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/chunk/storage/EntityStorage.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/chunk/storage/IOWorker.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/chunk/storage/RecreatingSimpleRegionStorage.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/chunk/storage/RegionBitmap.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/chunk/storage/RegionFile.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/chunk/storage/RegionFileStorage.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/chunk/storage/RegionFileVersion.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/chunk/storage/RegionStorageInfo.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/chunk/storage/SectionStorage.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/chunk/storage/SerializableChunkData.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/chunk/storage/SimpleRegionStorage.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/chunk/storage/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/level/dimension`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/dimension/BuiltinDimensionTypes.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/dimension/DimensionDefaults.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/dimension/DimensionType.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/dimension/LevelStem.java`.

## `decompiled-server-26.1.2/net/minecraft/world/level/dimension/end`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/dimension/end/DragonRespawnStage.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/dimension/end/EnderDragonFight.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/dimension/end/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/level/dimension`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/dimension/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/level/entity`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/entity/ChunkEntities.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/entity/ChunkStatusUpdateListener.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/entity/EntityAccess.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/entity/EntityInLevelCallback.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/entity/EntityLookup.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/entity/EntityPersistentStorage.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/entity/EntitySection.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/entity/EntitySectionStorage.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/entity/EntityTickList.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/entity/EntityTypeTest.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/entity/LevelCallback.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/entity/LevelEntityGetter.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/entity/LevelEntityGetterAdapter.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/entity/PersistentEntitySectionManager.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/entity/TransientEntitySectionManager.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/entity/UUIDLookup.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/entity/UniquelyIdentifyable.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/entity/Visibility.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/entity/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/level/gameevent`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/gameevent/BlockPositionSource.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/gameevent/DynamicGameEventListener.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/gameevent/EntityPositionSource.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/gameevent/EuclideanGameEventListenerRegistry.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/gameevent/GameEvent.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/gameevent/GameEventDispatcher.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/gameevent/GameEventListener.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/gameevent/GameEventListenerRegistry.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/gameevent/PositionSource.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/gameevent/PositionSourceType.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/gameevent/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/level/gameevent/vibrations`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/gameevent/vibrations/VibrationInfo.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/gameevent/vibrations/VibrationSelector.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/gameevent/vibrations/VibrationSystem.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/gameevent/vibrations/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/level/gamerules`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/gamerules/GameRule.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/gamerules/GameRuleCategory.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/gamerules/GameRuleMap.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/gamerules/GameRuleType.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/gamerules/GameRuleTypeVisitor.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/gamerules/GameRules.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/gamerules/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/level/levelgen`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/Aquifer.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/Beardifier.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/BelowZeroRetrogen.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/BitRandomSource.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/Column.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/DebugLevelSource.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/Density.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/DensityFunction.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/DensityFunctions.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/FlatLevelSource.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/GenerationStep.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/GeodeBlockSettings.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/GeodeCrackSettings.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/GeodeLayerSettings.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/Heightmap.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/LegacyRandomSource.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/MarsagliaPolarGaussian.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/NoiseBasedChunkGenerator.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/NoiseChunk.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/NoiseGeneratorSettings.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/NoiseRouter.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/NoiseRouterData.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/NoiseSettings.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/Noises.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/OreVeinifier.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/PatrolSpawner.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/PhantomSpawner.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/PositionalRandomFactory.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/RandomState.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/RandomSupport.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/SingleThreadedRandomSource.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/SurfaceRules.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/SurfaceSystem.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/ThreadSafeLegacyRandomSource.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/VerticalAnchor.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/WorldDimensions.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/WorldGenSettings.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/WorldGenerationContext.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/WorldOptions.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/WorldgenRandom.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/Xoroshiro128PlusPlus.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/XoroshiroRandomSource.java`.

## `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/blending`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/blending/Blender.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/blending/BlendingData.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/blending/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/blockpredicates`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/blockpredicates/AllOfPredicate.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/blockpredicates/AnyOfPredicate.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/blockpredicates/BlockPredicate.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/blockpredicates/BlockPredicateType.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/blockpredicates/CombiningPredicate.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/blockpredicates/HasSturdyFacePredicate.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/blockpredicates/InsideWorldBoundsPredicate.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/blockpredicates/MatchingBlockTagPredicate.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/blockpredicates/MatchingBlocksPredicate.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/blockpredicates/MatchingFluidsPredicate.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/blockpredicates/NotPredicate.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/blockpredicates/ReplaceablePredicate.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/blockpredicates/SolidPredicate.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/blockpredicates/StateTestingPredicate.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/blockpredicates/TrueBlockPredicate.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/blockpredicates/UnobstructedPredicate.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/blockpredicates/WouldSurvivePredicate.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/blockpredicates/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/carver`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/carver/CanyonCarverConfiguration.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/carver/CanyonWorldCarver.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/carver/CarverConfiguration.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/carver/CarverDebugSettings.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/carver/CarvingContext.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/carver/CaveCarverConfiguration.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/carver/CaveWorldCarver.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/carver/ConfiguredWorldCarver.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/carver/NetherWorldCarver.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/carver/WorldCarver.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/carver/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/AbstractHugeMushroomFeature.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/BambooFeature.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/BasaltColumnsFeature.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/BasaltPillarFeature.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/BlockBlobFeature.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/BlockColumnFeature.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/BlockPileFeature.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/BlueIceFeature.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/BonusChestFeature.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/ChorusPlantFeature.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/ConfiguredFeature.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/CoralClawFeature.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/CoralFeature.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/CoralMushroomFeature.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/CoralTreeFeature.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/DeltaFeature.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/DesertWellFeature.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/DiskFeature.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/DripstoneClusterFeature.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/DripstoneUtils.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/EndGatewayFeature.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/EndIslandFeature.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/EndPlatformFeature.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/EndPodiumFeature.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/EndSpikeFeature.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/FallenTreeFeature.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/Feature.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/FeatureCountTracker.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/FeaturePlaceContext.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/FillLayerFeature.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/FossilFeature.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/FossilFeatureConfiguration.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/GeodeFeature.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/GlowstoneFeature.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/HugeBrownMushroomFeature.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/HugeFungusConfiguration.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/HugeFungusFeature.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/HugeRedMushroomFeature.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/IcebergFeature.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/KelpFeature.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/LakeFeature.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/LargeDripstoneFeature.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/MonsterRoomFeature.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/MultifaceGrowthFeature.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/NetherForestVegetationFeature.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/NoOpFeature.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/OreFeature.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/PointedDripstoneFeature.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/RandomBooleanSelectorFeature.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/RandomSelectorFeature.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/ReplaceBlobsFeature.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/ReplaceBlockFeature.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/RootSystemFeature.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/ScatteredOreFeature.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/SculkPatchFeature.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/SeaPickleFeature.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/SeagrassFeature.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/SimpleBlockFeature.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/SimpleRandomSelectorFeature.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/SnowAndFreezeFeature.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/SpikeFeature.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/SpringFeature.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/TreeFeature.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/TwistingVinesFeature.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/UnderwaterMagmaFeature.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/VegetationPatchFeature.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/VinesFeature.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/VoidStartPlatformFeature.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/WaterloggedVegetationPatchFeature.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/WeepingVinesFeature.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/WeightedPlacedFeature.java`.

## `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/configurations`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/configurations/BlockBlobConfiguration.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/configurations/BlockColumnConfiguration.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/configurations/BlockPileConfiguration.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/configurations/BlockStateConfiguration.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/configurations/ColumnFeatureConfiguration.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/configurations/CountConfiguration.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/configurations/DeltaFeatureConfiguration.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/configurations/DiskConfiguration.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/configurations/DripstoneClusterConfiguration.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/configurations/EndGatewayConfiguration.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/configurations/EndSpikeConfiguration.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/configurations/FallenTreeConfiguration.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/configurations/FeatureConfiguration.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/configurations/GeodeConfiguration.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/configurations/HugeMushroomFeatureConfiguration.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/configurations/LargeDripstoneConfiguration.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/configurations/LayerConfiguration.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/configurations/MultifaceGrowthConfiguration.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/configurations/NetherForestVegetationConfig.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/configurations/NoneFeatureConfiguration.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/configurations/OreConfiguration.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/configurations/PointedDripstoneConfiguration.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/configurations/ProbabilityFeatureConfiguration.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/configurations/RandomBooleanFeatureConfiguration.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/configurations/RandomFeatureConfiguration.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/configurations/ReplaceBlockConfiguration.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/configurations/ReplaceSphereConfiguration.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/configurations/RootSystemConfiguration.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/configurations/SculkPatchConfiguration.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/configurations/SimpleBlockConfiguration.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/configurations/SimpleRandomFeatureConfiguration.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/configurations/SpikeConfiguration.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/configurations/SpringConfiguration.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/configurations/TreeConfiguration.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/configurations/TwistingVinesConfig.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/configurations/UnderwaterMagmaConfiguration.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/configurations/VegetationPatchConfiguration.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/configurations/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/featuresize`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/featuresize/FeatureSize.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/featuresize/FeatureSizeType.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/featuresize/ThreeLayersFeatureSize.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/featuresize/TwoLayersFeatureSize.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/featuresize/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/foliageplacers`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/foliageplacers/AcaciaFoliagePlacer.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/foliageplacers/BlobFoliagePlacer.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/foliageplacers/BushFoliagePlacer.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/foliageplacers/CherryFoliagePlacer.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/foliageplacers/DarkOakFoliagePlacer.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/foliageplacers/FancyFoliagePlacer.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/foliageplacers/FoliagePlacer.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/foliageplacers/FoliagePlacerType.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/foliageplacers/MegaJungleFoliagePlacer.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/foliageplacers/MegaPineFoliagePlacer.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/foliageplacers/PineFoliagePlacer.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/foliageplacers/RandomSpreadFoliagePlacer.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/foliageplacers/SpruceFoliagePlacer.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/foliageplacers/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/rootplacers`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/rootplacers/AboveRootPlacement.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/rootplacers/MangroveRootPlacement.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/rootplacers/MangroveRootPlacer.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/rootplacers/RootPlacer.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/rootplacers/RootPlacerType.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/rootplacers/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/stateproviders`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/stateproviders/BlockStateProvider.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/stateproviders/BlockStateProviderType.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/stateproviders/DualNoiseProvider.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/stateproviders/NoiseBasedStateProvider.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/stateproviders/NoiseProvider.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/stateproviders/NoiseThresholdProvider.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/stateproviders/RandomizedIntStateProvider.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/stateproviders/RotatedBlockProvider.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/stateproviders/RuleBasedStateProvider.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/stateproviders/SimpleStateProvider.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/stateproviders/WeightedStateProvider.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/stateproviders/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/treedecorators`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/treedecorators/AlterGroundDecorator.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/treedecorators/AttachedToLeavesDecorator.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/treedecorators/AttachedToLogsDecorator.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/treedecorators/BeehiveDecorator.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/treedecorators/CocoaDecorator.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/treedecorators/CreakingHeartDecorator.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/treedecorators/LeaveVineDecorator.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/treedecorators/PaleMossDecorator.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/treedecorators/PlaceOnGroundDecorator.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/treedecorators/TreeDecorator.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/treedecorators/TreeDecoratorType.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/treedecorators/TrunkVineDecorator.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/treedecorators/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/trunkplacers`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/trunkplacers/BendingTrunkPlacer.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/trunkplacers/CherryTrunkPlacer.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/trunkplacers/DarkOakTrunkPlacer.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/trunkplacers/FancyTrunkPlacer.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/trunkplacers/ForkingTrunkPlacer.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/trunkplacers/GiantTrunkPlacer.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/trunkplacers/MegaJungleTrunkPlacer.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/trunkplacers/StraightTrunkPlacer.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/trunkplacers/TrunkPlacer.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/trunkplacers/TrunkPlacerType.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/trunkplacers/UpwardsBranchingTrunkPlacer.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/trunkplacers/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/flat`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/flat/FlatLayerInfo.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/flat/FlatLevelGeneratorPreset.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/flat/FlatLevelGeneratorPresets.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/flat/FlatLevelGeneratorSettings.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/flat/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/heightproviders`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/heightproviders/BiasedToBottomHeight.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/heightproviders/ConstantHeight.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/heightproviders/HeightProvider.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/heightproviders/HeightProviderType.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/heightproviders/TrapezoidHeight.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/heightproviders/UniformHeight.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/heightproviders/VeryBiasedToBottomHeight.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/heightproviders/WeightedListHeight.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/heightproviders/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/material`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/material/MaterialRuleList.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/material/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/level/levelgen`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/placement`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/placement/BiomeFilter.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/placement/BlockPredicateFilter.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/placement/CaveSurface.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/placement/CountOnEveryLayerPlacement.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/placement/CountPlacement.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/placement/EnvironmentScanPlacement.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/placement/FixedPlacement.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/placement/HeightRangePlacement.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/placement/HeightmapPlacement.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/placement/InSquarePlacement.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/placement/NoiseBasedCountPlacement.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/placement/NoiseThresholdCountPlacement.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/placement/PlacedFeature.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/placement/PlacementContext.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/placement/PlacementFilter.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/placement/PlacementModifier.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/placement/PlacementModifierType.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/placement/RandomOffsetPlacement.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/placement/RarityFilter.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/placement/RepeatingPlacement.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/placement/SurfaceRelativeThresholdFilter.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/placement/SurfaceWaterDepthFilter.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/placement/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/presets`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/presets/WorldPreset.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/presets/WorldPresets.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/presets/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/structure`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/structure/BoundingBox.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/structure/BuiltinStructureSets.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/structure/BuiltinStructures.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/structure/PoolElementStructurePiece.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/structure/PostPlacementProcessor.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/structure/ScatteredFeaturePiece.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/structure/SinglePieceStructure.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/structure/Structure.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/structure/StructureCheck.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/structure/StructureCheckResult.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/structure/StructurePiece.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/structure/StructurePieceAccessor.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/structure/StructureSet.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/structure/StructureSpawnOverride.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/structure/StructureStart.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/structure/StructureType.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/structure/TemplateStructurePiece.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/structure/TerrainAdjustment.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/structure/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/structure/pieces`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/structure/pieces/PieceGenerator.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/structure/pieces/PieceGeneratorSupplier.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/structure/pieces/PiecesContainer.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/structure/pieces/StructurePieceSerializationContext.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/structure/pieces/StructurePieceType.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/structure/pieces/StructurePiecesBuilder.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/structure/pieces/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/structure/placement`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/structure/placement/ConcentricRingsStructurePlacement.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/structure/placement/RandomSpreadStructurePlacement.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/structure/placement/RandomSpreadType.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/structure/placement/StructurePlacement.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/structure/placement/StructurePlacementType.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/structure/placement/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/structure/pools`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/structure/pools/DimensionPadding.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/structure/pools/EmptyPoolElement.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/structure/pools/FeaturePoolElement.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/structure/pools/JigsawJunction.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/structure/pools/JigsawPlacement.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/structure/pools/LegacySinglePoolElement.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/structure/pools/ListPoolElement.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/structure/pools/SinglePoolElement.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/structure/pools/StructurePoolElement.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/structure/pools/StructurePoolElementType.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/structure/pools/StructureTemplatePool.java`.

## `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/structure/pools/alias`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/structure/pools/alias/DirectPoolAlias.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/structure/pools/alias/PoolAliasBinding.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/structure/pools/alias/PoolAliasBindings.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/structure/pools/alias/PoolAliasLookup.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/structure/pools/alias/RandomGroupPoolAlias.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/structure/pools/alias/RandomPoolAlias.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/structure/pools/alias/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/structure/pools`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/structure/pools/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/structure/structures`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/structure/structures/BuriedTreasurePieces.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/structure/structures/BuriedTreasureStructure.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/structure/structures/DesertPyramidPiece.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/structure/structures/DesertPyramidStructure.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/structure/structures/EndCityPieces.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/structure/structures/EndCityStructure.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/structure/structures/IglooPieces.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/structure/structures/IglooStructure.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/structure/structures/JigsawStructure.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/structure/structures/JungleTemplePiece.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/structure/structures/JungleTempleStructure.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/structure/structures/MineshaftPieces.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/structure/structures/MineshaftStructure.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/structure/structures/NetherFortressPieces.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/structure/structures/NetherFortressStructure.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/structure/structures/NetherFossilPieces.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/structure/structures/NetherFossilStructure.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/structure/structures/OceanMonumentPieces.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/structure/structures/OceanMonumentStructure.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/structure/structures/OceanRuinPieces.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/structure/structures/OceanRuinStructure.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/structure/structures/RuinedPortalPiece.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/structure/structures/RuinedPortalStructure.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/structure/structures/ShipwreckPieces.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/structure/structures/ShipwreckStructure.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/structure/structures/StrongholdPieces.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/structure/structures/StrongholdStructure.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/structure/structures/SwampHutPiece.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/structure/structures/SwampHutStructure.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/structure/structures/WoodlandMansionPieces.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/structure/structures/WoodlandMansionStructure.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/structure/structures/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/structure/templatesystem`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/structure/templatesystem/AlwaysTrueTest.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/structure/templatesystem/AxisAlignedLinearPosTest.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/structure/templatesystem/BlackstoneReplaceProcessor.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/structure/templatesystem/BlockAgeProcessor.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/structure/templatesystem/BlockIgnoreProcessor.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/structure/templatesystem/BlockMatchTest.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/structure/templatesystem/BlockRotProcessor.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/structure/templatesystem/BlockStateMatchTest.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/structure/templatesystem/CappedProcessor.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/structure/templatesystem/GravityProcessor.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/structure/templatesystem/JigsawReplacementProcessor.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/structure/templatesystem/LavaSubmergedBlockProcessor.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/structure/templatesystem/LinearPosTest.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/structure/templatesystem/LiquidSettings.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/structure/templatesystem/NopProcessor.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/structure/templatesystem/PosAlwaysTrueTest.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/structure/templatesystem/PosRuleTest.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/structure/templatesystem/PosRuleTestType.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/structure/templatesystem/ProcessorRule.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/structure/templatesystem/ProtectedBlockProcessor.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/structure/templatesystem/RandomBlockMatchTest.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/structure/templatesystem/RandomBlockStateMatchTest.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/structure/templatesystem/RuleProcessor.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/structure/templatesystem/RuleTest.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/structure/templatesystem/RuleTestType.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/structure/templatesystem/StructurePlaceSettings.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/structure/templatesystem/StructureProcessor.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/structure/templatesystem/StructureProcessorList.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/structure/templatesystem/StructureProcessorType.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/structure/templatesystem/StructureTemplate.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/structure/templatesystem/StructureTemplateManager.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/structure/templatesystem/TagMatchTest.java`.

## `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/structure/templatesystem/loader`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/structure/templatesystem/loader/DirectoryTemplateSource.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/structure/templatesystem/loader/ResourceManagerTemplateSource.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/structure/templatesystem/loader/TemplatePathFactory.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/structure/templatesystem/loader/TemplateSource.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/structure/templatesystem/loader/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/structure/templatesystem`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/structure/templatesystem/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/structure/templatesystem/rule/blockentity`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/structure/templatesystem/rule/blockentity/AppendLoot.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/structure/templatesystem/rule/blockentity/AppendStatic.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/structure/templatesystem/rule/blockentity/Clear.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/structure/templatesystem/rule/blockentity/Passthrough.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/structure/templatesystem/rule/blockentity/RuleBlockEntityModifier.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/structure/templatesystem/rule/blockentity/RuleBlockEntityModifierType.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/structure/templatesystem/rule/blockentity/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/synth`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/synth/BlendedNoise.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/synth/ImprovedNoise.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/synth/NoiseUtils.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/synth/NormalNoise.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/synth/PerlinNoise.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/synth/PerlinSimplexNoise.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/synth/SimplexNoise.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/levelgen/synth/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/level/lighting`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/lighting/BlockLightEngine.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/lighting/BlockLightSectionStorage.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/lighting/ChunkSkyLightSources.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/lighting/DataLayerStorageMap.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/lighting/DynamicGraphMinFixedPoint.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/lighting/LayerLightEventListener.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/lighting/LayerLightSectionStorage.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/lighting/LevelLightEngine.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/lighting/LeveledPriorityQueue.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/lighting/LightEngine.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/lighting/LightEventListener.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/lighting/SkyLightEngine.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/lighting/SkyLightSectionStorage.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/lighting/SpatialLongSet.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/lighting/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/level/material`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/material/EmptyFluid.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/material/FlowingFluid.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/material/Fluid.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/material/FluidState.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/material/Fluids.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/material/FogType.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/material/LavaFluid.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/material/MapColor.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/material/PushReaction.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/material/WaterFluid.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/material/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/level`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/level/pathfinder`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/pathfinder/AmphibiousNodeEvaluator.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/pathfinder/BinaryHeap.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/pathfinder/FlyNodeEvaluator.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/pathfinder/Node.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/pathfinder/NodeEvaluator.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/pathfinder/Path.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/pathfinder/PathComputationType.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/pathfinder/PathFinder.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/pathfinder/PathType.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/pathfinder/PathTypeCache.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/pathfinder/PathfindingContext.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/pathfinder/SwimNodeEvaluator.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/pathfinder/Target.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/pathfinder/WalkNodeEvaluator.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/pathfinder/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/level/portal`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/portal/PortalForcer.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/portal/PortalShape.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/portal/TeleportTransition.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/portal/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/level/redstone`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/redstone/CollectingNeighborUpdater.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/redstone/DefaultRedstoneWireEvaluator.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/redstone/ExperimentalRedstoneUtils.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/redstone/ExperimentalRedstoneWireEvaluator.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/redstone/InstantNeighborUpdater.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/redstone/NeighborUpdater.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/redstone/Orientation.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/redstone/Redstone.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/redstone/RedstoneWireEvaluator.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/redstone/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/level/saveddata`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/saveddata/SavedData.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/saveddata/SavedDataType.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/saveddata/WanderingTraderData.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/saveddata/WeatherData.java`.

## `decompiled-server-26.1.2/net/minecraft/world/level/saveddata/maps`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/saveddata/maps/MapBanner.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/saveddata/maps/MapDecoration.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/saveddata/maps/MapDecorationType.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/saveddata/maps/MapDecorationTypes.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/saveddata/maps/MapFrame.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/saveddata/maps/MapId.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/saveddata/maps/MapIndex.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/saveddata/maps/MapItemSavedData.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/saveddata/maps/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/level/saveddata`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/saveddata/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/level/storage`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/storage/CommandStorage.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/storage/DataVersion.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/storage/DerivedLevelData.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/storage/FileNameDateFormatter.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/storage/LevelData.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/storage/LevelDataAndDimensions.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/storage/LevelResource.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/storage/LevelStorageException.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/storage/LevelStorageSource.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/storage/LevelSummary.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/storage/LevelVersion.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/storage/PlayerDataStorage.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/storage/PrimaryLevelData.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/storage/SavedDataStorage.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/storage/ServerLevelData.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/storage/TagValueInput.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/storage/TagValueOutput.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/storage/ValueInput.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/storage/ValueInputContextHelper.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/storage/ValueOutput.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/storage/WorldData.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/storage/WritableLevelData.java`.

## `decompiled-server-26.1.2/net/minecraft/world/level/storage/loot`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/storage/loot/BuiltInLootTables.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/storage/loot/ContainerComponentManipulator.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/storage/loot/ContainerComponentManipulators.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/storage/loot/IntRange.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/storage/loot/LootContext.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/storage/loot/LootContextArg.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/storage/loot/LootContextUser.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/storage/loot/LootDataType.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/storage/loot/LootParams.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/storage/loot/LootPool.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/storage/loot/LootTable.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/storage/loot/Validatable.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/storage/loot/ValidationContext.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/storage/loot/ValidationContextSource.java`.

## `decompiled-server-26.1.2/net/minecraft/world/level/storage/loot/entries`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/storage/loot/entries/AlternativesEntry.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/storage/loot/entries/ComposableEntryContainer.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/storage/loot/entries/CompositeEntryBase.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/storage/loot/entries/DynamicLoot.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/storage/loot/entries/EmptyLootItem.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/storage/loot/entries/EntryGroup.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/storage/loot/entries/LootItem.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/storage/loot/entries/LootPoolEntries.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/storage/loot/entries/LootPoolEntry.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/storage/loot/entries/LootPoolEntryContainer.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/storage/loot/entries/LootPoolSingletonContainer.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/storage/loot/entries/NestedLootTable.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/storage/loot/entries/SequentialEntry.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/storage/loot/entries/SlotLoot.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/storage/loot/entries/TagEntry.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/storage/loot/entries/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/level/storage/loot/functions`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/storage/loot/functions/ApplyBonusCount.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/storage/loot/functions/ApplyExplosionDecay.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/storage/loot/functions/CopyBlockState.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/storage/loot/functions/CopyComponentsFunction.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/storage/loot/functions/CopyCustomDataFunction.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/storage/loot/functions/CopyNameFunction.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/storage/loot/functions/DiscardItem.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/storage/loot/functions/EnchantRandomlyFunction.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/storage/loot/functions/EnchantWithLevelsFunction.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/storage/loot/functions/EnchantedCountIncreaseFunction.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/storage/loot/functions/ExplorationMapFunction.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/storage/loot/functions/FillPlayerHead.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/storage/loot/functions/FilteredFunction.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/storage/loot/functions/FunctionReference.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/storage/loot/functions/FunctionUserBuilder.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/storage/loot/functions/LimitCount.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/storage/loot/functions/ListOperation.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/storage/loot/functions/LootItemConditionalFunction.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/storage/loot/functions/LootItemFunction.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/storage/loot/functions/LootItemFunctions.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/storage/loot/functions/ModifyContainerContents.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/storage/loot/functions/SequenceFunction.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/storage/loot/functions/SetAttributesFunction.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/storage/loot/functions/SetBannerPatternFunction.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/storage/loot/functions/SetBookCoverFunction.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/storage/loot/functions/SetComponentsFunction.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/storage/loot/functions/SetContainerContents.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/storage/loot/functions/SetContainerLootTable.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/storage/loot/functions/SetCustomDataFunction.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/storage/loot/functions/SetCustomModelDataFunction.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/storage/loot/functions/SetEnchantmentsFunction.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/storage/loot/functions/SetFireworkExplosionFunction.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/storage/loot/functions/SetFireworksFunction.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/storage/loot/functions/SetInstrumentFunction.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/storage/loot/functions/SetItemCountFunction.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/storage/loot/functions/SetItemDamageFunction.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/storage/loot/functions/SetItemFunction.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/storage/loot/functions/SetLoreFunction.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/storage/loot/functions/SetNameFunction.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/storage/loot/functions/SetOminousBottleAmplifierFunction.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/storage/loot/functions/SetPotionFunction.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/storage/loot/functions/SetRandomDyesFunction.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/storage/loot/functions/SetRandomPotionFunction.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/storage/loot/functions/SetStewEffectFunction.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/storage/loot/functions/SetWritableBookPagesFunction.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/storage/loot/functions/SetWrittenBookPagesFunction.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/storage/loot/functions/SmeltItemFunction.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/storage/loot/functions/ToggleTooltips.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/storage/loot/functions/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/level/storage/loot`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/storage/loot/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/level/storage/loot/parameters`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/storage/loot/parameters/LootContextParamSets.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/storage/loot/parameters/LootContextParams.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/storage/loot/parameters/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/level/storage/loot/predicates`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/storage/loot/predicates/AllOfCondition.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/storage/loot/predicates/AnyOfCondition.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/storage/loot/predicates/BonusLevelTableCondition.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/storage/loot/predicates/CompositeLootItemCondition.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/storage/loot/predicates/ConditionReference.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/storage/loot/predicates/ConditionUserBuilder.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/storage/loot/predicates/DamageSourceCondition.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/storage/loot/predicates/EnchantmentActiveCheck.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/storage/loot/predicates/EntityHasScoreCondition.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/storage/loot/predicates/EnvironmentAttributeCheck.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/storage/loot/predicates/ExplosionCondition.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/storage/loot/predicates/InvertedLootItemCondition.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/storage/loot/predicates/LocationCheck.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/storage/loot/predicates/LootItemBlockStatePropertyCondition.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/storage/loot/predicates/LootItemCondition.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/storage/loot/predicates/LootItemConditions.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/storage/loot/predicates/LootItemEntityPropertyCondition.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/storage/loot/predicates/LootItemKilledByPlayerCondition.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/storage/loot/predicates/LootItemRandomChanceCondition.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/storage/loot/predicates/LootItemRandomChanceWithEnchantedBonusCondition.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/storage/loot/predicates/MatchTool.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/storage/loot/predicates/TimeCheck.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/storage/loot/predicates/ValueCheckCondition.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/storage/loot/predicates/WeatherCheck.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/storage/loot/predicates/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/level/storage/loot/providers/nbt`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/storage/loot/providers/nbt/ContextNbtProvider.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/storage/loot/providers/nbt/NbtProvider.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/storage/loot/providers/nbt/NbtProviders.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/storage/loot/providers/nbt/StorageNbtProvider.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/storage/loot/providers/nbt/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/level/storage/loot/providers/number`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/storage/loot/providers/number/BinomialDistributionGenerator.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/storage/loot/providers/number/ConstantValue.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/storage/loot/providers/number/EnchantmentLevelProvider.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/storage/loot/providers/number/EnvironmentAttributeValue.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/storage/loot/providers/number/NumberProvider.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/storage/loot/providers/number/NumberProviders.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/storage/loot/providers/number/ScoreboardValue.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/storage/loot/providers/number/StorageValue.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/storage/loot/providers/number/Sum.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/storage/loot/providers/number/UniformGenerator.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/storage/loot/providers/number/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/level/storage/loot/providers/score`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/storage/loot/providers/score/ContextScoreboardNameProvider.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/storage/loot/providers/score/FixedScoreboardNameProvider.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/storage/loot/providers/score/ScoreboardNameProvider.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/storage/loot/providers/score/ScoreboardNameProviders.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/storage/loot/providers/score/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/level/storage`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/storage/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/level/timers`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/timers/FunctionCallback.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/timers/FunctionTagCallback.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/timers/TimerCallback.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/timers/TimerCallbacks.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/timers/TimerQueue.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/timers/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/level/validation`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/validation/ContentValidationException.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/validation/DirectoryValidator.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/validation/ForbiddenSymlinkInfo.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/validation/PathAllowList.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/validation/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/phys`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/phys/AABB.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/phys/BlockHitResult.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/phys/EntityHitResult.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/phys/HitResult.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/phys/Vec2.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/phys/Vec3.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/phys/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/phys/shapes`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/phys/shapes/ArrayVoxelShape.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/phys/shapes/BitSetDiscreteVoxelShape.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/phys/shapes/BooleanOp.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/phys/shapes/CollisionContext.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/phys/shapes/CubePointRange.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/phys/shapes/CubeVoxelShape.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/phys/shapes/DiscreteCubeMerger.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/phys/shapes/DiscreteVoxelShape.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/phys/shapes/EntityCollisionContext.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/phys/shapes/IdenticalMerger.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/phys/shapes/IndexMerger.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/phys/shapes/IndirectMerger.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/phys/shapes/MinecartCollisionContext.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/phys/shapes/NonOverlappingMerger.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/phys/shapes/OffsetDoubleList.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/phys/shapes/Shapes.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/phys/shapes/SliceShape.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/phys/shapes/SubShape.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/phys/shapes/VoxelShape.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/phys/shapes/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/scores`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/scores/DisplaySlot.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/scores/Objective.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/scores/PlayerScoreEntry.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/scores/PlayerScores.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/scores/PlayerTeam.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/scores/ReadOnlyScoreInfo.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/scores/Score.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/scores/ScoreAccess.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/scores/ScoreHolder.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/scores/Scoreboard.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/scores/ScoreboardSaveData.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/scores/Team.java`.

## `decompiled-server-26.1.2/net/minecraft/world/scores/criteria`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/scores/criteria/ObjectiveCriteria.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/scores/criteria/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/scores`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/scores/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/ticks`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/ticks/BlackholeTickAccess.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/ticks/ContainerSingleItem.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/ticks/LevelChunkTicks.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/ticks/LevelTickAccess.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/ticks/LevelTicks.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/ticks/ProtoChunkTicks.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/ticks/SavedTick.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/ticks/ScheduledTick.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/ticks/SerializableTickContainer.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/ticks/TickAccess.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/ticks/TickContainerAccess.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/ticks/TickPriority.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/ticks/WorldGenTickAccess.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/ticks/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/timeline`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/timeline/AttributeTrack.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/timeline/AttributeTrackSampler.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/timeline/Timeline.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/timeline/Timelines.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/timeline/package-info.java`.

## `decompiled-server-26.1.2/net/minecraft/world/waypoints`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/waypoints/PartialTickSupplier.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/waypoints/TrackedWaypoint.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/waypoints/TrackedWaypointManager.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/waypoints/Waypoint.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/waypoints/WaypointManager.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/waypoints/WaypointStyleAsset.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/waypoints/WaypointStyleAssets.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/waypoints/WaypointTransmitter.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/waypoints/package-info.java`.
