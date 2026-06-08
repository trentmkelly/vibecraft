# Java Class-Level Port Coverage Checklist

Generated from `decompiled-server-26.1.2/net/minecraft` on 2026-05-24. This file intentionally contains one unchecked task per Java source file so full-port planning can prove every decompiled class has an explicit Rust port/parity-test placeholder. Do not mark an item complete until the corresponding behavior has been audited against Java and covered by Rust implementation tests or an explicit deferral note.

## `decompiled-server-26.1.2/net/minecraft`

- [x] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/CharPredicate.java`. Rust `JavaCharPredicate` models Java `char` as a UTF-16 `u16` code unit and covers `test`, short-circuiting `and`/`or`, and `negate`; verified by `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 char_predicate`.
- [x] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/ChatFormatting.java`. Rust `ChatFormatting` covers the Java enum constants/order, `char` codes, ids, RGB colors, format/color classification, lowercase serialized names, `toString` formatting code, cleaned-name/id/code lookup behavior, nullable strip-formatting semantics, `getNames` filters, and color-codec validation; verified by `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 chat_formatting`.
- [x] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/CrashReport.java`. Rust `CrashReportModel` models Java title/exception accessors, exception-message title substitution for null-message NPE/SOE/OOME throwables, head/category/system detail rendering, friendly report layout, save-once UTF-8 file behavior with parent creation, `addCategory` stack trace tracking, `forThrowable` CompletionException unwrapping and ReportedException reuse, and preload actions; verified by `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 crash_report`.
- [x] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/CrashReportCategory.java`. Rust `CrashReportCategoryModel` covers Java category entry formatting, detail callback error capture, location formatting for world/section/region coordinates, details rendering, stacktrace fill/validate/trim semantics, and block-location helpers; verified by `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 crash_report_category`.
- [x] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/CrashReportDetail.java`. Java's `Callable` alias behavior is represented by `CrashReportCategoryModel::set_detail_callback`, including successful values and throwable-to-error detail conversion; verified by `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 crash_report_category`.
- [x] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/DefaultUncaughtExceptionHandler.java`. Rust `default_uncaught_exception_handler_actions` models Java's single `logger.error("Caught previously unhandled exception :", throwable)` call; verified by `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 default_uncaught_exception_handler`.
- [x] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/DefaultUncaughtExceptionHandlerWithName.java`. Rust `default_uncaught_exception_handler_with_name_actions` models Java's two logger calls: first the fixed message, then `logger.error(thread.getName(), throwable)`; verified by `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 default_uncaught_exception_handler`.
- [x] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/DetectedVersion.java`. Rust `world_version` models Java's built-in development-version factory, JSON `version.json` parsing with `series_id` defaulting to `main`, pack-format extraction, `ZonedDateTime` to `Date` instant conversion, missing-resource warning fallback, and corrupt-info error message; verified by `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 world_version`.
- [x] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/IdentifierException.java`. Rust `IdentifierExceptionModel` stores the Apache `StringEscapeUtils.escapeJava`-style escaped message and optional cause, including control-character, quote/backslash, BMP Unicode, and surrogate-pair escaping behavior; verified by `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 identifier_exception`.
- [x] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/Optionull.java`. Rust `optionull` models Java null branches for `orElse`, `map`, `mapOrDefault`, lazy `mapOrElse`, `first`, `firstOrDefault`, lazy `firstOrElse`, and all object/primitive array `isNullOrEmpty` overload semantics through nullable slices; verified by `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 optionull`.
- [x] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/ReportType.java`. Rust `ReportTypeModel` covers Java's five static records, exact headers/nugget counts, `Util.getNanos() % nuggets.size()` comment selection, fallback witty-comment behavior, and `appendHeader` formatting with extra comments; verified by `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 report_type`.
- [x] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/ReportedException.java`. Rust `ReportedExceptionModel` preserves the wrapped crash report reference and delegates `getCause()` to `report.getException()` and `getMessage()` to `report.getTitle()`; verified by `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 reported_exception`.
- [x] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/SharedConstants.java`. Rust `shared_constants` models Java's 26.1.2 constants, debug system-property gates, integer debug parsing failure, mutable version state (`setVersion`, `tryDetectVersion`, `getCurrentVersion`), protocol version, `debugVoidTerrain`, illegal filename characters, and static initializer actions; verified by `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 shared_constants`.
- [x] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/SuppressForbidden.java`. Rust `SuppressForbiddenModel` captures Java's `@Retention(CLASS)`, target set `{CONSTRUCTOR, FIELD, METHOD, TYPE}`, and required `reason()` element; verified by `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 suppress_forbidden`.
- [x] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/SystemReport.java`. Rust `system_report` models Java's ordered system entries, memory/runtime/JVM flag formatting, supplier error fallback to `ERR`, OSHI hardware groups with ignored group failures, storage-space result strings, `sizeInMiB`, crash-report append format, and line-separated output; verified by `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 system_report`.
- [x] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/TracingExecutor.java`. Rust `tracing_executor` models Java's IDE named-executor branch with thread rename/restore, Tracy named branch, direct-service fallback when Tracy is unavailable, unnamed `execute` wrapping with the IDE flag, throwable close-suppression behavior from Java try-with-resources, and `shutdownAndAwait` shutdown-now behavior after false/interrupted termination; verified by `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 tracing_executor`.
- [x] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/WorldVersion.java`. Rust `WorldVersionModel` covers Java `WorldVersion.Simple` fields, `DataVersion` series compatibility, `PackType.CLIENT_RESOURCES`/`SERVER_DATA` `packVersion` switching, and the 26.1.2 constants shared with `/version`; verified by `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 world_version`.

## `decompiled-server-26.1.2/net/minecraft/advancements`

- [x] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/advancements/Advancement.java`. Rust `JavaAdvancementModel` covers Java's parent/display/rewards/criteria/requirements/telemetry aggregate, criteria non-empty and requirements validation, decorated display name/fallback holder name, root detection, criterion validation traversal, stream payload shape, and builder defaults/save/build behavior; verified by `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 advancement_builder_defaults`, `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 advancement_builder_requirements`, `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 advancement_network_read_write`, and `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 advancement_validate_walks`.
- [x] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/advancements/AdvancementHolder.java`. Rust `AdvancementHolderData` already covers the stream payload used by `ClientboundUpdateAdvancementsPacket`; this audit adds Java's id-only `equals`/`hashCode` semantics and `toString()` id rendering via `java_equals_by_id`, `java_hash_key`, and `java_to_string`; verified by `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 advancement_holder_model`.
- [x] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/advancements/AdvancementNode.java`. Rust `AdvancementNodeModel` covers Java's holder-backed node identity, parent/child access, root walking, id-based equality/hash key behavior, and `toString` id rendering; verified by `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 advancement_node_matches`.
- [x] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/advancements/AdvancementProgress.java`. Rust `JavaAdvancementProgressModel` covers Java's criteria map plus requirements state, `update` trim/insert behavior, `isDone`, `hasProgress`, grant/revoke semantics, criterion lookup, percent/progress-text rules, completed/remaining criteria, first progress date, `compareTo`, network conversion, and `toString`; verified by `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 advancement_progress_update`, `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 advancement_progress_percent`, and `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 advancement_progress_network`.
- [x] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/advancements/AdvancementRequirements.java`. Rust `AdvancementRequirementsModel` covers Java's `EMPTY`, AND/OR strategy construction, `size`, empty-aware `test`, `count`, exact criteria validation with Java error wording, `names`, and list-style `toString`; the advancement JSON loader now validates requirements through this exact-match model; verified by `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 advancement_requirements` and `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 advancement_json_loader_rejects_invalid_requirements`.
- [x] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/advancements/AdvancementRewards.java`. Rust `AdvancementRewards` now has Java-parity empty and builder helpers in `src/advancement_rewards.rs`, a grant action model for experience, loot pickup/drop behavior, recipe awards, and gamemaster/suppressed function execution, plus JSON default/field coverage through the advancement loader; verified by `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 advancement_rewards_builder`, `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 advancement_rewards_grant`, and `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 advancement_rewards_json`.
- [x] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/advancements/AdvancementTree.java`. Rust `AdvancementTreeModel` covers Java's node/root/task maps, iterative parent-deferred insertion, recursive removal, unknown-remove warning, clear behavior, listener replay/events, and unloadable-advancement error logging; verified by `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 advancement_tree_add_all`, `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 advancement_tree_remove_clear`, and `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 advancement_tree_unloadable`.
- [x] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/advancements/AdvancementType.java`. Rust `AdvancementFrame` models Java `AdvancementType` enum order, serialized names (`task`, `challenge`, `goal`), chat colors (`GREEN`, `DARK_PURPLE`, `GREEN`), toast translation keys, codec-style name lookup, and advancement announcement translation keys; verified by `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 advancement_type_model`.
- [x] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/advancements/CriteriaTriggers.java`. Rust `CriteriaTriggersModel` preserves Java's 58-entry trigger registration order, serialized names, implementation class mapping, lookup/codec-style name resolution, shared trigger implementation aliases, register behavior, and `bootstrap` returning `IMPOSSIBLE`; verified by `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 criteria_triggers_registry`, `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 criteria_triggers_model_preserves`, and `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 criteria_triggers_bootstrap`.
- [x] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/advancements/Criterion.java`. Rust `CriterionModel` mirrors Java's trigger plus trigger-instance record shape, with construction routed through `CriterionTriggerModel::create_criterion`; verified by `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 criterion_trigger_models`.
- [x] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/advancements/CriterionProgress.java`. Rust `CriterionProgressModel` covers Java's nullable `Instant` state as nullable epoch millis, `isDone`, explicit grant/revoke transitions, `getObtained`, `toString`, and nullable network payload conversion shared with `CriterionProgressData`; verified by `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 criterion_progress_model`.
- [x] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/advancements/CriterionTrigger.java`. Rust `CriterionTriggerModel` covers Java's id-backed trigger handle, `createCriterion`, and listener award callback contract via `CriterionListenerModel::run`; verified by `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 criterion_trigger_models`.
- [x] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/advancements/CriterionTriggerInstance.java`. Rust `CriterionTriggerInstanceModel` preserves Java's validation hook against a validation context sink; verified by `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 criterion_trigger_models`.
- [x] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/advancements/DisplayInfo.java`. Rust `AdvancementDisplay` and `DisplayInfoModel` now cover Java's required icon, title/description components, optional background, frame/default boolean codec fields, float location setters/getters, network flags, and network decode behavior that restores `announceChat` as false; verified by `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 display_info_json_codec`, `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 display_info_json_explicit`, `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 display_info_network`, and `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 advancement_json_loader_decodes_vanilla_codec_fields`.
- [x] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/advancements/TreeNodePosition.java`. Rust `TreeNodePositionLayout` ports Java's visible-root validation, invisible-child flattening, first/second/third walk layout state, shift/apportion/thread handling, and final position assignment; verified by `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 tree_node_position_rejects`, `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 tree_node_position_places`, `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 tree_node_position_flattens`, and `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 tree_node_position_apportions`.

## `decompiled-server-26.1.2/net/minecraft/advancements/criterion`

- [x] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/advancements/criterion/AnyBlockInteractionTrigger.java`. Rust `criterion_block_interaction` covers Java's advancement-location loot context construction with origin/player/block-state/tool parameters, optional location predicate matching, trigger id, and validation context behavior; verified by `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 any_block_interaction_trigger`.
- [x] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/advancements/criterion/BeeNestDestroyedTrigger.java`. Rust `criterion_bee_nest` covers Java's optional block predicate, optional item predicate, `num_bees_inside` int bounds default, match ordering, trigger id, and `destroyedBeeNest` factory shape; verified by `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 bee_nest_destroyed_matches`, `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 bee_nest_destroyed_omitted`, and `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 bee_nest_destroyed_factory`.
- [x] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/advancements/criterion/BlockPredicate.java`. Rust `criterion_block_predicate` covers Java's optional block holder-set filter, state-property predicate matching and unknown-property validation, loaded-position gate, block-entity NBT and data-component checks, the `BlockInWorld` overload's component-ignoring behavior, builder defaults, and `requiresNbt`; verified by `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 block_predicate`.
- [x] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/advancements/criterion/BredAnimalsTrigger.java`. Rust `criterion_bred_animals` covers Java's parent, partner, and optional child entity loot-context predicates, child-first failure behavior, symmetric parent/partner matching, omitted predicate defaults, validation labels, and all `bredAnimals` factory overload shapes; verified by `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 bred_animals`.
- [x] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/advancements/criterion/BrewedPotionTrigger.java`. Rust `criterion_brewed_potion` covers Java's optional potion holder codec field, holder-equality match behavior, omitted-potion match-any behavior, player predicate presence field, and `brewedPotion` factory trigger id/default instance; verified by `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 brewed_potion`.
- [x] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/advancements/criterion/ChangeDimensionTrigger.java`. Rust `criterion_change_dimension` covers Java's optional `from` and `to` dimension resource keys, from-first match behavior, omitted key defaults, player predicate presence field, and all `changedDimension` factory overload shapes; verified by `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 changed_dimension`.
- [x] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/advancements/criterion/ChanneledLightningTrigger.java`. Rust `criterion_channeled_lightning` covers Java's victim loot-context collection matching, default empty victim list, every-predicate-has-some-victim behavior, non-consuming victim reuse across predicates, validation labels, player predicate presence field, and the `channeledLightning` factory trigger id/list wrapping; verified by `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 channeled_lightning`.
- [x] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/advancements/criterion/CollectionContentsPredicate.java`. Rust `criterion_collection_predicates` covers Java's zero/single/multiple contents predicate shapes, codec unpack list behavior, match-any zero semantics, single predicate any-value scan, and multiple predicate `removeIf` semantics where one value can satisfy multiple remaining tests; verified by `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 collection_`.
- [x] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/advancements/criterion/CollectionCountsPredicate.java`. Rust `criterion_collection_predicates` covers Java's zero/single/multiple count predicate shapes, entry count scanning, MinMax count bounds, independent multiple-entry evaluation, and unpack list behavior; verified by `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 collection_`.
- [x] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/advancements/criterion/CollectionPredicate.java`. Rust `criterion_collection_predicates` covers Java's optional `contains`, `count`, and `size` fields, contains-then-counts-then-size evaluation, omitted section defaults, and collection size bounds; verified by `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 collection_`.
- [x] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/advancements/criterion/ConstructBeaconTrigger.java`. Rust `criterion_construct_beacon` covers Java's `level` MinMaxBounds field with ANY codec default, trigger level matching, player predicate presence field, and both `constructedBeacon` factory overload shapes; verified by `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 construct_beacon`.
- [x] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/advancements/criterion/ConsumeItemTrigger.java`. Rust `criterion_consume_item` covers Java's optional item predicate field, omitted-predicate match-any behavior, consumed item stack matching through item holder-set, count bounds, and data component checks, player predicate presence field, and all `usedItem` factory overload shapes; verified by `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 consume_item`.
- [x] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/advancements/criterion/ContextAwarePredicate.java`. Rust `criterion_context_aware_predicate` covers Java's condition-list codec shape, `create` factory, `Util.allOf` empty-list true behavior, all-condition matching, short-circuit failure behavior, condition list unpacking, and unnamed indexed validation paths; verified by `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 context_aware_predicate`.
- [x] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/advancements/criterion/CuredZombieVillagerTrigger.java`. Rust `criterion_cured_zombie_villager` covers Java's zombie and villager entity loot-context predicates, omitted-predicate match-any behavior, zombie-first then villager matching, player predicate presence field, validation labels, and the `curedZombieVillager` factory trigger id/default instance; verified by `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 cured_zombie_villager`.
- [x] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/advancements/criterion/DamagePredicate.java`. Rust `criterion_damage_predicate` covers Java's dealt/taken `MinMaxBounds.Doubles` defaults and matching, source entity predicate check, blocked flag check, damage-source predicate delegation, match ordering, and builder defaults/setters; verified by `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 damage_predicate`.
- [x] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/advancements/criterion/DamageSourcePredicate.java`. Rust `criterion_damage_source_predicate` covers Java's damage-type tag predicate list with positive/negative expectations, direct and source entity predicate checks, `is_direct` flag matching, default empty predicate behavior, builder setters, and match ordering; verified by `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 damage_source_predicate`.
- [x] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/advancements/criterion/DataComponentMatchers.java`. Rust `criterion_data_component_matchers` covers Java's `ANY` empty matcher, exact component predicate delegation before partial predicates, all-partial-predicates-required matching, `isEmpty`, builder `components`, `any`, `partial`, `exact`, partial duplicate-key rejection from `ImmutableMap.Builder.buildOrThrow`, and duplicate exact component rejection inherited from `DataComponentExactPredicate.Builder`; verified by `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 data_component_matchers`.
- [x] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/advancements/criterion/DefaultBlockInteractionTrigger.java`. Rust `criterion_block_interaction` covers Java's block-use loot context construction without a tool parameter, optional location predicate matching, trigger id, and validation context behavior; verified by `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 default_block_interaction`.
- [x] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/advancements/criterion/DistancePredicate.java`. Rust `criterion_distance_predicate` covers Java's x/y/z absolute delta bounds, horizontal squared x/z distance bounds, absolute squared x/y/z distance bounds, `horizontal`, `vertical`, and `absolute` factories, default ANY fields, and float-delta matching behavior; verified by `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 distance_predicate`.
- [x] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/advancements/criterion/DistanceTrigger.java`. Rust `criterion_distance_trigger` covers Java's optional `start_position` and `distance` predicates, start-position-before-distance match ordering, omitted-predicate match-any behavior, scoped location position bounds used by this trigger, and the `fallFromHeight`, `rideEntityInLava`, and `travelledThroughNether` factory trigger ids/fields; verified by `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 distance_trigger`.
- [x] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/advancements/criterion/EffectsChangedTrigger.java`. Rust `criterion_effects_changed` covers Java's optional effects and source predicates, effects-before-source match ordering, null-source rejection when a source predicate is present, source validation label, `hasEffects` and `gotEffectsFrom` factory trigger ids/fields, and the scoped mob-effect instance predicate behavior used by this trigger; verified by `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 effects_changed`.
- [x] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/advancements/criterion/EnchantedItemTrigger.java`. Rust `criterion_enchanted_item` covers Java's optional item predicate, item-before-level match ordering, `levels` `MinMaxBounds.Ints` default and matching, omitted-predicate match-any behavior, player predicate presence field, and the `enchantedItem` factory trigger id/default instance; verified by `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 enchanted_item`.
- [x] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/advancements/criterion/EnchantmentPredicate.java`. Rust `criterion_enchantment_predicate` covers Java's optional enchantment holder-set matching, level-bound matching for specific enchantments, level-only scans across all item enchantments, default any-enchantment-present behavior, zero-level-as-absent behavior, and direct holder/holder-set constructor shapes; verified by `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 enchantment_predicate`.
- [x] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/advancements/criterion/EnterBlockTrigger.java`. Rust `criterion_enter_block` covers Java's optional block holder and state-property predicates, block-before-state match ordering, state-property validation only when a block is present, omitted-predicate match-any behavior, player predicate presence field, and the `entersBlock` factory trigger id/block field; verified by `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 enter_block`.
- [x] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/advancements/criterion/EntityEquipmentPredicate.java`. Rust `criterion_entity_equipment_predicate` covers Java's null/non-living rejection, optional head/chest/legs/feet/body/mainhand/offhand item predicates, Java slot match ordering, empty-slot item behavior, builder setters, and `captainPredicate` white ominous banner item/component requirements from `Raid.getBannerComponentPatch`; verified by `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 entity_equipment_predicate`.
- [x] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/advancements/criterion/EntityFlagsPredicate.java`. Rust `criterion_entity_flags_predicate` covers Java's optional on-ground/on-fire/crouching/sprinting/swimming/in-water checks, `is_flying` living fall-flying/player-ability semantics, non-living behavior for `is_flying`, `is_fall_flying`, and `is_baby`, fall-flying-before-baby match ordering, and all builder setters; verified by `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 entity_flags_predicate`.
- [x] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/advancements/criterion/EntityHurtPlayerTrigger.java`. Rust `criterion_entity_hurt_player` covers Java's optional damage predicate delegation, omitted-damage match-any behavior, player predicate presence field, trigger id, and all three `entityHurtPlayer` factory overload shapes including builder build-before-store behavior; verified by `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 entity_hurt_player`.
- [x] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/advancements/criterion/EntityPredicate.java`. Rust `criterion_entity_predicate` covers Java's null-entity rejection, type/tag predicates, null-position distance failure, movement velocity scaling by 20, located/stepping-on/movement-affected-by location checks, effects/flags/equipment/type-specific predicates, recursive vehicle/passenger/targeted-entity matching, periodic tick, team, slots, components-before-NBT final ordering, builder setters, `LocationWrapper`, and `wrap` helper shapes; verified by `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 criterion_entity_predicate`.
- [x] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/advancements/criterion/EntitySubPredicate.java`. Rust `criterion_entity_sub_predicates` covers Java's dispatch-style sub-predicate interface shape with `codec()` identity and `matches(entity, level, nullable position)` call routing; verified by `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 entity_sub_predicate`.
- [x] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/advancements/criterion/EntitySubPredicates.java`. Rust `criterion_entity_sub_predicates` covers Java's registered sub-predicate type ids (`lightning`, `fishing_hook`, `player`, `slime`, `raider`, `sheep`), namespaced registry lookup behavior, and `bootstrap` returning `LIGHTNING`; concrete sub-predicate behavior remains tracked by each predicate's own checklist row; verified by `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 entity_sub_predicate`.
- [x] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/advancements/criterion/EntityTypePredicate.java`. Rust `criterion_entity_type_predicate` covers Java's homogeneous holder-set wrapper, direct entity-type factory, tag lookup factory with get-or-throw behavior, holder-set containment matching, and accessor preservation; verified by `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 entity_type_predicate`.
- [x] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/advancements/criterion/FallAfterExplosionTrigger.java`. Rust `criterion_fall_after_explosion` covers Java's optional start-position, distance, and cause predicates, start-position-then-distance-then-cause match ordering, nullable cause rejection when a cause predicate is present, `cause` validation label, and the `fallAfterExplosion` factory trigger id/distance/wrapped-cause fields; verified by `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 fall_after_explosion`.
- [x] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/advancements/criterion/FilledBucketTrigger.java`. Rust `criterion_filled_bucket` covers Java's optional item predicate, omitted-predicate match-any behavior, player predicate presence field, and `filledBucket` factory trigger id/build-before-store behavior; verified by `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 filled_bucket`.
- [x] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/advancements/criterion/FishingHookPredicate.java`. Rust `criterion_fishing_hook_predicate` covers Java's `ANY` optional-empty behavior, `inOpenWater` factory, constrained matching only for fishing hook entities, open-water flag comparison, and `codec()` returning the registered fishing hook sub-predicate type; verified by `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 fishing_hook_predicate`.
- [x] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/advancements/criterion/FishingRodHookedTrigger.java`. Rust `criterion_fishing_rod_hooked` covers Java's optional rod/entity/item predicates, trigger context selection of hooked entity before hook entity fallback, rod-before-entity-before-item match ordering, item predicate matching against hooked `ItemEntity` stack before collected item stacks, omitted predicate defaults, `fishedItem` trigger id/optional entity wrapping, player predicate presence field, and `entity` validation label; verified by `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 fishing_rod_hooked`.
- [x] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/advancements/criterion/FluidPredicate.java`. Rust `criterion_fluid_predicate` covers Java's loaded-position guard before fluid-state evaluation, optional homogeneous fluid holder-set matching, omitted predicate defaults including loaded empty fluid state, optional `StatePropertiesPredicate` exact/ranged/missing-property matching for fluid states, invalid typed range-bound failure, and builder `fluid`/direct-fluid/set/properties/build preservation; verified by `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 fluid_predicate`.
- [x] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/advancements/criterion/FoodPredicate.java`. Rust `criterion_food_predicate` covers Java's `ANY` default predicate, optional/default level and saturation bounds, level-before-saturation match ordering against `FoodData`, integer and double min/max/exact bound shapes, and builder `food`/`withLevel`/`withSaturation`/`build` field preservation; verified by `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 food_predicate`.
- [x] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/advancements/criterion/GameTypePredicate.java`. Rust `criterion_game_type_predicate` covers Java's `GameType` enum ids/names/order, `ANY` over `GameType.values()`, `SURVIVAL_LIKE` over survival/adventure only, varargs-style `of` order preservation including duplicates, and `matches` list-containment behavior; verified by `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 game_type_predicate`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/advancements/criterion/ImpossibleTrigger.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/advancements/criterion/InputPredicate.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/advancements/criterion/InventoryChangeTrigger.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/advancements/criterion/ItemDurabilityTrigger.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/advancements/criterion/ItemPredicate.java`.
- [x] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/advancements/criterion/ItemUsedOnLocationTrigger.java`. Rust `criterion_block_interaction` covers Java's advancement-location loot context construction with tool parameter, optional location predicate matching, validation context behavior, and static factory trigger ids for placed block, placed block with properties, item used on block, and allay drop item on block; verified by `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 item_used_on_location_static`.
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
- [x] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/BeaconBeamOwner.java`. — Audited against Java 26.1.2: the interface exposes `getBeamSections()`, and nested `Section` stores `color`, starts at height 1, increments height by 1, and exposes color/height getters. Rust parity is `BeaconBlockEntity::beam_sections()` plus `BeaconBeamSection::new()`, `increase_height()`, `color()`, and `height()`; covered by `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 beacon_tier_effect_payment_and_beam_state_match_vanilla_rules`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/BeaconBlockEntity.java`.
- [x] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/BedBlockEntity.java`. — Audited against Java 26.1.2: the entity stores only bed color from `BedBlock`, has no additional NBT fields, and sends the standard block-entity data packet. Rust parity is `BedBlockEntity::from_block_state()` + empty `save_additional()` + `BlockEntity::get_update_packet()`; covered by `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 bed_block_entity_is_color_only_placeholder`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/BeehiveBlockEntity.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/BellBlockEntity.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/BlastFurnaceBlockEntity.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/BlockEntity.java`.
- [x] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/BlockEntityTicker.java`. — Audited against Java 26.1.2: the functional interface ticks with `Level`, `BlockPos`, `BlockState`, and the concrete block entity instance. Rust parity is the scheduler-facing `BlockEntity::tick(client_side)` and `TickingBlockEntity`, backed by `BlockEntityTypeInfo::tick_kind` and each entity's stored position/block state/type; covered by `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 ticking_requires_level_side_match_and_not_removed`, `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 tick_dispatch_advances_scheduler_state_for_every_tickable_block_entity_type`, and `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 ticking_block_entity_wrapper_exposes_scheduler_shape`.
- [x] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/BlockEntityType.java`. — Audited against Java 26.1.2: the class registers 49 block entity types with valid block sets, creates entities from `(BlockPos, BlockState)`, checks `isValid(BlockState)`, exposes the built-in holder, returns typed entities from a level lookup, and guards op-only custom NBT for command blocks, lecterns, signs, hanging signs, mob spawners, and trial spawners. Rust parity is `BLOCK_ENTITY_TYPES`, `BlockEntity::new()`, `type_info()`, `type_by_key()`, `is_valid_block_state()`, `has_block_entity_for_block()`, and `only_op_can_set_nbt()`; covered by `block_entity_registry_matches_26_1_2_type_surface`, `block_entity_registry_valid_blocks_match_java_26_1_2`, `op_only_custom_data_matches_vanilla_guarded_types`, `validates_block_state_on_creation_and_load`, and `load_static_with_data_version_refuses_unsafe_migrations`.
- [x] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/BoundingBoxRenderable.java`. — Audited against Java 26.1.2: the interface exposes render modes `NONE`/`BOX`/`BOX_AND_INVISIBLE_BLOCKS`, `getRenderableBox()`, and `RenderableBox.fromCorners()` as min local position plus max-min size. Rust parity is `StructureRenderMode`, `StructureBlockEntity::render_mode()`, `StructureBlockEntity::renderable_box()`, and `StructureRenderableBox::{from_corners,local_pos,size}`; covered by `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 structure_block_entity_round_trips_bounds_and_render_box`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/BrewingStandBlockEntity.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/BrushableBlockEntity.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/CalibratedSculkSensorBlockEntity.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/CampfireBlockEntity.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/ChestBlockEntity.java`.
- [x] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/ChestLidController.java`. — Audited against Java 26.1.2: the controller stores `shouldBeOpen`, current openness, previous openness, advances by 0.1 per tick toward 0.0/1.0, and returns interpolated openness via `Mth.lerp`. Rust parity is `ChestLidController::{tick_lid,get_openness,should_be_open,openness,previous_openness}` wired into chest/trapped-chest `ContainerBlockEntityModel::tick_lid()`; covered by `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 container_block_entities_track_loot_openers_lids_redstone_and_hopper_like_java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/ChiseledBookShelfBlockEntity.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/CommandBlockEntity.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/ComparatorBlockEntity.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/ConduitBlockEntity.java`.
- [x] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/ContainerOpenersCounter.java`. — Audited against Java 26.1.2: the support counter increments/decrements openers, fires first-open/last-close hooks and `CONTAINER_OPEN`/`CONTAINER_CLOSE` game events, calls `openerCountChanged(previous,current)`, tracks max interaction range for the `+4` inflated search box, filters spectator/non-open container users, rechecks live users, and schedules 5-tick rechecks while open. Rust parity is `ContainerOpenersCounterModel` with `ContainerOpenersCounterEffect`/`ContainerUserOpenState`; covered by `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 container_block_entities_track_loot_openers_lids_redstone_and_hopper_like_java`.
- [x] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/CopperGolemStatueBlockEntity.java`. — Audited against Java 26.1.2: the entity binds `BlockEntityType.COPPER_GOLEM_STATUE`, copies a copper golem custom-name component into the statue, recreates a triggered copper golem at block center/Y with facing yaw, head/body yaw, and spawn sound, emits a block-entity update packet, and clones components into the statue item with the supplied `copper_golem_pose` block-state component. Rust parity is `BlockEntityTypeId::CopperGolemStatue` plus `CopperGolemStatueBlockEntity::{create_statue,remove_statue,update_packet_type,item_components_for_pose,clone_item_components}` and the existing weather/pose/comparator state model; covered by `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 copper_golem_statue_tracks_weather_pose_comparator_and_clone_components`, `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 block_entity_registry_valid_blocks_match_java_26_1_2`, and `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 block_entity_comparator_outputs_cover_boundary_states`.
- [x] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/CrafterBlockEntity.java`. — Audited against Java 26.1.2: the class is a 3x3 `RandomizableContainerBlockEntity`/`CraftingContainer` with default name `container.crafter`, `CrafterMenu` plus 10 data slots, empty-slot-only disable toggles, disabled-slot placement rejection, Java's later-slot smaller-stack balancing rule, loot-table/item persistence, `disabled_slots`, `triggered`, `crafting_ticks_remaining`, triggered recipe pulse consumption/result/remainders, 6-tick crafting countdown that clears block-state `crafting=false`, and redstone signal counting occupied or disabled slots. Rust parity is `CrafterBlockEntity::{set_slot_state,is_slot_disabled,can_place_item,pulse_craft,server_tick,save_additional,load_additional,redstone_signal,open_menu,handle_slot_state_changed}` plus `CrafterRecipe`; covered by `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 crafter_block_entity_tracks_disabled_slots_triggered_pulse_and_output_like_java`, `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 crafter_handle_slot_state_changed_gates_match_java`, `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 gui_block_entities_open_with_menu_id_initial_slots_and_close_state`, `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 block_entity_registry_valid_blocks_match_java_26_1_2`, and `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 block_entity_comparator_outputs_cover_boundary_states`.
- [x] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/CreakingHeartBlockEntity.java`. — Audited against Java 26.1.2: the block entity tracks the linked creaking UUID/entity reference, 30-tick unresolved UUID grace period, 20–24 tick update cadence, awake/dormant/uprooted state from required logs plus `CREAKING_ACTIVE`, spawn attempts/ranges, protector removal on inactive non-persistent creaking, distance >34, or stuck player, 100-tick hurt pulse with 50 particle ticks and 2–3 resin clumps while awake, pre-remove protector teardown, UUID save/load, and comparator output `15 - floor(clamp(distance,0,32)/32*15)` with neighbor-output updates when it changes. Rust parity is `CreakingHeartBlockEntity`, `CreakingHeartTickContext`, and `CreakingHeartAction`; covered by `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 creaking_heart_block_entity_tracks_state_protector_and_output_like_java`, `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 block_entity_registry_valid_blocks_match_java_26_1_2`, and `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 block_entity_comparator_outputs_cover_boundary_states`.
- [x] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/DaylightDetectorBlockEntity.java`. — Audited against Java 26.1.2: the entity class only extends `BlockEntity` and constructs with `BlockEntityType.DAYLIGHT_DETECTOR`; signal math, inversion, player interaction, shape, and ticker selection live in `DaylightDetectorBlock.java`. Rust parity is `BlockEntityTypeId::DaylightDetector` registered as `daylight_detector` for `minecraft:daylight_detector` plus the no-persistent-state `DaylightDetectorBlockEntity` model; covered by `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 daylight_detector_updates_power_with_vanilla_solar_math_and_tick_cadence` and `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 block_entity_registry_valid_blocks_match_java_26_1_2`.
- [x] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/DecoratedPotBlockEntity.java`. — Audited against Java 26.1.2: the block entity stores `PotDecorations`, one contained item, optional loot table plus seed, update packet/tag data, decorated-pot item template/instance components, implicit `POT_DECORATIONS` and `CONTAINER` components, component stripping for `sherds`/`item`, single-item container methods that unpack loot first, wobble event id `1` with positive/negative durations `7`/`10`, and destruction drops for decorations plus stored item. Rust parity is `DecoratedPotBlockEntity`, `PotDecorations`, `DecoratedPotWobbleStyle`, and `DecoratedPotDrops`; covered by `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 decorated_pot_saves_sherds_item_loot_and_wobble_like_java`, `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 decorated_pot_wobble_and_destruction_drops_preserve_sherds_and_item`, `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 block_entity_registry_valid_blocks_match_java_26_1_2`, and `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 block_entity_comparator_outputs_cover_boundary_states`.
- [x] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/DecoratedPotPattern.java`. — Audited against Java 26.1.2: the record stores only the pattern `assetId`. Rust parity is `DecoratedPotPattern { asset_id }`, embedded in each `DecoratedPotPatternEntry`; covered by `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 decorated_pot_patterns_match_java_26_1_2_registry_and_item_map`.
- [x] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/DecoratedPotPatterns.java`. — Audited against Java 26.1.2: the class defines 24 registry keys, maps brick plus every pottery sherd to its pattern key, and bootstraps the exact pattern asset ids. Rust parity is `DECORATED_POT_PATTERNS`, `decorated_pot_pattern_from_item()`, and `decorated_pot_pattern_asset_id()`; covered by `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 decorated_pot_patterns_match_java_26_1_2_registry_and_item_map`.
- [x] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/DispenserBlockEntity.java`. — Audited against Java 26.1.2: the block entity is a `RandomizableContainerBlockEntity` with 9 slots, default name `container.dispenser`, `DispenserMenu`, reservoir-random non-empty slot selection, `insertItem` merging into matching stacks or first empty slots with leftover return, item-list persistence, and loot-table-aware container access. Rust parity is `ContainerBlockEntityKind::Dispenser`, `BlockEntityTypeId::Dispenser`, `ContainerBlockEntityModel::{random_non_empty_slot,insert_item,save_additional,load_additional,create_menu}`; covered by `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 container_block_entities_track_loot_openers_lids_redstone_and_hopper_like_java`, `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 gui_block_entities_open_with_menu_id_initial_slots_and_close_state`, and `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 block_entity_registry_valid_blocks_match_java_26_1_2`.
- [x] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/DropperBlockEntity.java`. — Audited against Java 26.1.2: `DropperBlockEntity` only specializes `DispenserBlockEntity` by constructing with `BlockEntityType.DROPPER` and overriding the default name to `container.dropper`; item ejection behavior lives in `DropperBlock.java`. Rust parity is `BlockEntityTypeId::Dropper` plus `ContainerBlockEntityKind::Dropper` inheriting the 9-slot `generic_3x3` dispenser container contract with the `container.dropper` name; covered by `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 container_block_entities_track_loot_openers_lids_redstone_and_hopper_like_java`, `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 gui_block_entities_open_with_menu_id_initial_slots_and_close_state`, and `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 block_entity_registry_valid_blocks_match_java_26_1_2`.
- [x] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/EnchantingTableBlockEntity.java`. — Audited against Java 26.1.2: the block entity persists nullable `CustomName`, uses `container.enchant` fallback display name, applies/collects/removes the `CUSTOM_NAME` component, exposes `EnchantmentMenu`, and owns the client-side book animation state (`time`, open/old-open, rotation/target rotation, flip/target/acceleration) while bookshelf power is counted from `EnchantingTableBlock.BOOKSHELF_OFFSETS` by the menu. Rust parity is `EnchantingTableBlockEntity::{save_additional,load_additional,get_update_tag,collect_implicit_components,apply_implicit_components,remove_components_from_tag,open_menu,bookshelf_offsets,count_valid_bookshelves,book_animation_tick}`; covered by `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 enchanting_table_saves_name_scans_bookshelves_and_animates_book_like_java`.
- [x] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/EnderChestBlockEntity.java`. — Audited against Java 26.1.2: the block entity owns only lid/open-count state, uses `ChestLidController`, emits block event action `1` with current opener count, plays first-open/last-close ender-chest sounds, ignores removed/spectator open changes, rechecks active ender-chest users, validates by block-entity identity plus 8-block distance, and serves the player's `PlayerEnderChestContainer` through a three-row chest menu from `EnderChestBlock`. Rust parity is `EnderChestBlockEntityModel::{start_open,stop_open,recheck_open,trigger_event,lid_animate_tick,get_openness,still_valid,open_menu}` plus `open_ender_chest_menu`; covered by `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 ender_chest_lid_openers_and_menu_match_java`, `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 gui_block_entities_open_with_menu_id_initial_slots_and_close_state`, and `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 block_entity_registry_valid_blocks_match_java_26_1_2`.
- [x] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/FuelValues.java`. — Audited against Java 26.1.2: `FuelValues` stores item burn durations, reports `isFuel`, exposes the fuel item set, returns `0` for empty/non-fuel stacks, builds vanilla burn times from direct items plus item tags in `vanillaBurnTimes(..., baseUnit)`, and removes `ItemTags.NON_FLAMMABLE_WOOD`. Rust parity is `FuelValues::{vanilla_from_tags,vanilla_with_base_unit,burn_duration,is_fuel,fuel_items}` with tag expansion/removal and base-unit scaling; covered by `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 fuel_values_match_vanilla_burn_time_defaults` and `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 vanilla_from_tags_expands_tags_and_removes_non_flammable_wood`.
- [x] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/FurnaceBlockEntity.java`. — Audited against Java 26.1.2: this class only specializes `AbstractFurnaceBlockEntity` with `BlockEntityType.FURNACE`, `RecipeType.SMELTING`, default display name `container.furnace`, and `FurnaceMenu`; all tick/storage behavior is inherited from the abstract furnace base. Rust parity is `FurnaceBlockEntityKind::Furnace` plus `AbstractFurnaceBlockEntity::furnace()` and `open_menu()`, with `recipe_type() == "smelting"`, `default_cooking_time() == 200`, `default_name() == "container.furnace"`, and `menu_type() == "furnace"`; covered by `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 furnace_family_ticks_fuel_recipes_xp_sided_slots_and_speed_like_java` and `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 gui_block_entities_open_with_menu_id_initial_slots_and_close_state`.
- [x] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/HangingSignBlockEntity.java`. — Audited against Java 26.1.2: the class only specializes `SignBlockEntity` with `BlockEntityType.HANGING_SIGN`, max line width `60`, line height `9`, and failed interaction sound `WAXED_HANGING_SIGN_INTERACT_FAIL`; text save/load, wax/editing, update tags, and click-command behavior are inherited from `SignBlockEntity`, while attachment is derived from the hanging sign block state rather than persisted in block-entity NBT. Rust parity is `HangingSignBlockEntityModel`, `SignBlockEntityModel`, and `HangingSignAttachment::from_block_state`, with inherited sign NBT and hanging-specific width/height/sound; covered by `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 sign_block_entities_track_front_back_text_filtering_wax_and_hanging_shape` and `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 block_entity_registry_valid_blocks_match_java_26_1_2`.
- [x] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/Hopper.java`. — Audited against Java 26.1.2: the support interface extends `Container`, defines `SUCK_AABB = Block.column(16, 11, 32)` as `(0, 11/16, 0)..(1, 2, 1)`, returns that AABB from `getSuckAabb()`, and exposes hopper level coordinates plus grid alignment. Rust parity is `ContainerBlockEntityModel::{hopper_suck_aabb,hopper_level_x,hopper_level_y,hopper_level_z,hopper_is_grid_aligned}` backed by `world_position`; covered by `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 container_block_entities_track_loot_openers_lids_redstone_and_hopper_like_java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/HopperBlockEntity.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/JigsawBlockEntity.java`.
- [x] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/JukeboxBlockEntity.java`. — Audited against Java 26.1.2: the block entity stores a single `RecordItem`, persists `ticks_since_song_started` only when `JukeboxSong.fromStack` succeeds, starts/stops the `JukeboxSongPlayer` on item changes, exposes max stack size 1, accepts only `JUKEBOX_PLAYABLE` items into an empty jukebox, allows hopper extraction only when the destination has an empty slot, pops the item before removal, emits `JUKEBOX_STOP_PLAY` plus level event `1011` on removal, and returns the song comparator output. Rust parity is `JukeboxBlockEntity::{set_the_item,set_song_item_without_playing,tick,save_additional,load_additional,can_place_item,can_take_item,max_stack_size,pop_out_the_item,pre_remove_side_effects,set_removed,comparator_output}` with the 26.1.2 playable disc/comparator table; covered by `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 jukebox_block_entity_tracks_disc_playback_ticks_and_outputs` and `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 block_entity_comparator_outputs_cover_boundary_states`.
- [x] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/LecternBlockEntity.java`. — Audited against Java 26.1.2: the block entity stores only book/page state, accepts writable or written book contents for `hasBook`, resets page/page-count on `setBook`, `clearContent`, and item removal, saves non-empty `Book` plus `Page`, clamps loaded and menu-written pages, computes analog output as `floor(pageProgress * 14) + hasBook`, exposes a one-slot lectern menu/container that refuses placement, gates validity on same block entity, distance, and having a book, and spawns a copied book at `pos + (0.5, 1.0, 0.5) + facing * 0.25` during pre-removal when the block state has a book. Rust parity is `LecternBlockEntity::{set_book,has_book,clear_content,save_additional,load_additional,set_page,get_redstone_signal,open_menu,remove_item,remove_book_no_update,max_stack_size,can_place_item,still_valid,pre_remove_side_effects}`; covered by `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 lectern_block_entity`.
- [x] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/LidBlockEntity.java`. — Audited against Java 26.1.2: the interface exposes only `getOpenNess(float)` for partial-tick lid interpolation. Rust parity is `ContainerBlockEntityModel::chest_lid_openness(partial_tick)` delegating to `ChestLidController::get_openness()`; covered by `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 container_block_entities_track_loot_openers_lids_redstone_and_hopper_like_java`.
- [x] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/ListBackedContainer.java`. — Audited against Java 26.1.2: the interface wraps a `NonNullList<ItemStack>` and provides default `count`, `getContainerSize`, `clearContent`, `isEmpty`, `getItem`, `removeItem` with `setChanged`, `removeItemNoUpdate`, `canPlaceItem`, always-true `acceptsItemType`, `setItem`, and stack-limiting `setItemNoUpdate`. Rust parity is `ContainerBlockEntityModel::{count,container_size,clear_content,is_empty,get_item,remove_item,remove_item_no_update,can_place_item,accepts_item_type,set_item,set_item_no_update}` plus the `content_changed` flag; covered by `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 container_block_entities_track_loot_openers_lids_redstone_and_hopper_like_java`.
- [x] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/PotDecorations.java`. — Audited against Java 26.1.2: the record stores optional back/left/right/front items, normalizes `Items.BRICK` and missing list entries to empty sides, serializes in back/left/right/front order with brick fallback, and emits non-empty tooltips in front/left/right/back order. Rust parity is `PotDecorations::{new,ordered,tooltip_items,to_tag,from_tag}`; covered by `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 decorated_pot_saves_sherds_item_loot_and_wobble_like_java`.
- [x] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/RandomizableContainerBlockEntity.java`. — Audited against Java 26.1.2: the class layers nullable `lootTable` plus `lootTableSeed` over `BaseContainerBlockEntity`, unpacks pending loot before inventory reads/mutations and before menu creation, denies spectator opening while unresolved loot exists, stores `LootTableSeed` only when non-zero, collects/applies `CONTAINER_LOOT` alongside base custom-name/lock/container components, and removes `LootTable`/`LootTableSeed` from block-entity NBT when components own that state. Rust parity is `ContainerBlockEntityModel::{loot_table,loot_table_seed,unpack_loot_table,can_open,create_menu,open_menu,save_additional,load_additional,collect_implicit_components,apply_implicit_components,remove_components_from_tag}`; covered by `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 randomizable_container_components_and_loot_table_tags_match_java` and `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 container_block_entities_track_loot_openers_lids_redstone_and_hopper_like_java`.
- [x] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/SculkCatalystBlockEntity.java`. — Audited against Java 26.1.2: the block entity owns a BY_DISTANCE game-event listener with radius 8, saves/loads only the listener's `SculkSpreader` cursors, ticks by delegating to `SculkSpreader.updateCursors(..., spreadVeins=true)`, consumes mob XP once on `ENTITY_DIE`, adds cursors at the death source shifted upward by 0.5 block when XP would drop, blooms the catalyst for 8 ticks, and awards `KILL_MOB_NEAR_SCULK_CATALYST` when the mob was last hurt by a server player. Rust now matches the class-level listener constants, cursor splitting/capping, event side-effect result, XP-consumption gate, non-persistent pulse state, and cursor NBT shape in `SculkCatalystBlockEntity::{handle_entity_die,add_cursors,save_additional,load_additional}`; full `SculkSpreader.updateCursors` world mutation/particle parity is explicitly deferred with an in-code TODO because `SculkBehaviour` block mutation and sculk-vein spreading are broader missing subsystems. Covered by `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 sculk_catalyst_block_entity_queues_charge_and_pulses_on_mob_death`.
- [x] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/SculkSensorBlockEntity.java`. — Audited against Java 26.1.2: the block entity persists only `last_vibration_frequency` and VibrationSystem `listener` data, exposes a vibration listener with radius 8, can trigger avoid-vibration advancement handling, requires adjacent chunks to be ticking, rejects same-position `BLOCK_DESTROY`/`BLOCK_PLACE`, ignores zero-frequency events and inactive block states, updates last frequency and redstone strength from receiving distance, and leaves activation phase/power timing to block-state/runtime behavior rather than block-entity NBT. Rust parity is `SculkSensorBlockEntity::{save_additional,load_additional,get_update_tag,can_receive_vibration_from,queue_vibration,receive_vibration,tick}` plus `VibrationData`/`VibrationInfo`; covered by `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 sculk_sensor_block_entity_tracks_vibration_phase_frequency_and_power`.
- [x] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/SculkShriekerBlockEntity.java`. — Audited against Java 26.1.2: the block entity persists raw `warning_level` plus VibrationSystem `listener` data, resolves player sources from direct players, controlling passengers, projectile owners, and item owners, listens to `GameEventTags.SHRIEKER_CAN_LISTEN` with radius 8 and adjacent-chunk ticking, rejects vibrations while the block state is already shrieking or no player source exists, resets warning level before `tryWarn`, shrieks for 90 ticks when response is disabled or warning succeeds, responds on pre-removal if the block state is shrieking, plays warning-level reply sounds within radius 10, applies darkness within radius 40, and tries Warden spawn at warning level 4 with 20 attempts over a 5x6 range. Rust parity is `SculkShriekerBlockEntity::{save_additional,load_additional,can_receive_vibration,try_get_player,try_shriek,try_respond,pre_remove_side_effects,tick}`; covered by `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 sculk_shrieker_block_entity_tracks_warning_shriek_and_warden_response`.
- [x] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/ShelfBlockEntity.java`. — Audited against Java 26.1.2: the block entity is a 3-slot `ListBackedContainer`/`ItemOwner` with `MAX_ITEMS = 3`, saves/loads all items plus `align_items_to_bottom`, sends an update packet/tag with the same data, validates menus through block-entity identity and 8-block range, swaps held items without update, reports `BLOCK_ACTIVATE` by default from `setChanged`, sends block updates with flags `3`, applies/collects the `CONTAINER` component, removes `Items` from component-owned NBT, returns center position for item ownership, and rotates display items by the opposite block facing. Rust parity is `ShelfBlockEntity::{save_additional,load_additional,get_update_tag,collect_implicit_components,apply_implicit_components,remove_components_from_tag,still_valid,swap_item_no_update,set_changed_side_effects,default_set_changed_side_effects,item_owner_position,visual_rotation_y_degrees,get_align_items_to_bottom,comparator_output}`; covered by `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 shelf_block_entity`.
- [x] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/ShulkerBoxBlockEntity.java`. — Audited against Java 26.1.2: the block entity is a 27-slot `RandomizableContainerBlockEntity`/`WorldlyContainer` with optional dye color, event action `1` for open-count updates, first-open/last-close block events, sounds and game events gated by removed/spectator state, four animation states advancing by 0.1 per tick over 10 ticks, previous/current progress interpolation, lid-height/collision push deltas, closed-state solid collision, all 27 sided slots, nested shulker insertion rejection, always-true sided extraction, and no pre-removal side effects. Rust parity is `ContainerBlockEntityModel::{trigger_shulker_event,shulker_start_open_effects,shulker_stop_open_effects,tick_shulker_animation,shulker_progress,shulker_bounding_lid_height,shulker_collision_move_delta,shulker_slots_for_face,shulker_can_take_through_face,can_place_through_face,shulker_pre_remove_has_side_effects,shulker_forces_solid_collision,shulker_color_from_block_id}`; covered by `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 shulker_box`.
- [x] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/SignBlockEntity.java`. — Audited against Java 26.1.2: Rust models independent front/back sign text, edit ownership and 4-block range expiry, waxed edit denial, filtered player updates, first-class set/update text change reporting, custom-only update tags/packets for sign and hanging sign block entities, sign front-face hit testing from Java's angle formula, waxed-sign failed interaction sound, and command execution through a waxed/run-command gate with Java's gamemaster permission level represented as a constant. Full chat `Component` line storage, styles, and non-command click events are explicitly deferred with a TODO on `SignLine` until the sign model is migrated onto the existing `chat_component` module; the broader sign block-entity checklist row remains open. Covered by `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 sign_`.
- [x] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/SignText.java`. — Audited against Java 26.1.2: Rust covers four raw/filtered lines, omitted filtered-message serialization when filtered text equals raw text, default black color, glowing-text state, color/glow NBT round-trip, filtered/raw visibility selection, filtered-player update semantics, message presence checks, run-command click-command detection, and front/back save/load through `SignBlockEntityModel`. Full `Component[]` fidelity, style-preserving `setMessage`, render-message caching, and Java 26.1.2's non-command click-event execution payloads are explicitly deferred by the `SignLine` TODO and the still-open broad sign checklist row. Covered by `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 sign_`.
- [x] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/SkullBlockEntity.java`. — Audited against Java 26.1.2: the class persists nullable `profile`, `note_block_sound`, and `custom_name`, exposes owner and note-block sound state, sends custom-only block-entity update tags/packets, maps implicit `PROFILE`, `NOTE_BLOCK_SOUND`, and `CUSTOM_NAME` components in both directions, removes component-owned NBT fields, and advances `animationTickCount` only while the powered skull block state is true while interpolation adds partial ticks only during animation. Rust parity is `SkullBlockEntity::{save_additional,load_additional,get_update_tag,apply_implicit_components,collect_implicit_components,remove_components_from_tag,animation_tick,animation}` plus the generic `BlockEntity::get_update_tag` skull custom-only path; covered by `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 skull`.
- [x] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/SmokerBlockEntity.java`. — Audited against Java 26.1.2: the class only provides smoker specialization over `AbstractFurnaceBlockEntity`: `BlockEntityType.SMOKER`, `RecipeType.SMOKING`, default name `container.smoker`, half burn duration from the base fuel values, and `SmokerMenu`. Rust parity is `FurnaceBlockEntityKind::Smoker` plus `AbstractFurnaceBlockEntity::smoker()`/`open_menu()`, covered by `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 smoker_block_entity_specialization_matches_java`.
- [x] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/SpawnerBlockEntity.java`. — Audited against Java 26.1.2: the wrapper owns a `BaseSpawner`, delegates save/load/client/server ticking, strips `SpawnPotentials` from update tags, routes event id `1`, mutates `SpawnData` through `setEntityId`, and sends block updates with flags `260` when next spawn data changes. Rust parity is `SpawnerBlockEntity::{save_additional,load_additional,update_tag,server_tick,server_tick_with_context,client_tick_with_display,on_event_triggered,set_entity_id,set_next_spawn_data}` plus generic `BlockEntity::get_update_tag` stripping for mob spawners; covered by `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 spawner_block_entity`.
- [x] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/StructureBlockEntity.java`. — Audited against Java 26.1.2: Rust models the block entity's saved custom payload (`name`, `author`, `metadata`, position/size, mirror, rotation, mode, ignore/strict/powered/show flags, integrity, seed), default values, load-time bounds clamping, empty-name handling, custom-only update tags, gamemaster/client use behavior, creator author, mode update flags, strict placement flags, render mode, and renderable bounding-box math. Java's live `detectSize`, `saveStructure`, `loadStructureInfo`, `placeStructure`, `unloadStructure`, and `isStructureLoadable` operations require a wired `ServerLevel`/`StructureTemplateManager`; those are explicitly deferred by `TODO(structure-block-world-ops)` in `structures.rs`. Covered by `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 structure_block_entity`.
- [x] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/TestBlockEntity.java`. — Audited against Java 26.1.2: Rust models `mode`, `message`, `powered`, and transient `triggered` state; saves/loads the same custom NBT fields with default mode `fail`; emits custom-only generic update tags; exposes mode update flags `2`; handles `START` reset/trigger powered-neighbor behavior and non-start trigger tracking; and mirrors blank-message log gating. Covered by `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 test_block_entity`.
- [x] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/TestInstanceBlockEntity.java`. — Audited against Java 26.1.2: Rust models the persisted `data` payload (optional test id, size, rotation, ignore-entities, status, optional error message), optional error markers, status transitions, custom-only update tags, box render mode, beacon beam colors for cleared/running/success/required-failed/optional-failed states, structure offset `(0,1,1)`, transformed-size and start-corner rotation math, renderable box calculation, and error-marker clearing. Full `GameTestRunner`, structure placement/export, barrier processing, entity removal, forced chunk loading, and registry-backed test lookup are explicitly deferred by `TODO(test-instance-world-ops)` until the game-test/server-template systems are wired. Covered by `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 test_instance_block_entity`.
- [x] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/TheEndGatewayBlockEntity.java`. — Audited against Java 26.1.2: Rust models the persisted `Age`, nullable `exit_portal` with Java's spawnable-bounds filter, omitted-default `ExactTeleport`, spawn/cooldown animation percentages, beam and portal tick age/cooldown transitions, attention-interval cooldown trigger, event id `1`, custom-only update tags through both typed and generic block-entity paths, exact bottom-center target positions, inexact exit search origin/above-target math, and particle amount counting from rendered faces. The live chunk scan/island creation path is represented by the portal/worldgen gateway planning helpers rather than the block-entity model. Covered by `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 end_gateway_block_entity_saves_ticks_cooldown_and_exit_like_java`.
- [x] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/TheEndPortalBlockEntity.java`. — Audited against Java 26.1.2: the block entity has no custom NBT, uses `BlockEntityType.END_PORTAL`, and renders only Y-axis faces. Rust parity is `EndPortalBlockEntity::save_additional()` + `EndPortalBlockEntity::should_render_face()` + `BlockEntity::get_update_packet()`; covered by `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 end_portal_block_entity_is_zero_data_portal_placeholder`.
- [x] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/TickingBlockEntity.java`. — Audited against Java 26.1.2: the interface exposes `tick()`, `isRemoved()`, `getPos()`, and `getType()` for level/chunk scheduler wrappers. Rust parity is `TickingBlockEntity::tick()`, `is_removed()`, `pos()`, and `type_key()` delegating to the wrapped `BlockEntity`; covered by `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 ticking_block_entity_wrapper_exposes_scheduler_shape`.
- [x] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/TrappedChestBlockEntity.java`. — Audited against Java 26.1.2: the class is a `ChestBlockEntity` with `BlockEntityType.TRAPPED_CHEST` and overrides `signalOpenCount` to run the base open-count behavior plus, when previous/current counts differ, neighbor updates at the chest position and the block below using the trapped chest block and redstone orientation from the opposite facing plus `UP`. Rust parity is `ContainerBlockEntityKind::TrappedChest`, `ContainerBlockEntityModel::trapped_chest_signal()`, and `trapped_chest_signal_open_count()` returning the two neighbor update positions/orientation inputs; covered by `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 container_block_entities_track_loot_openers_lids_redstone_and_hopper_like_java` and `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 block_entity_comparator_outputs_cover_boundary_states`.
- [x] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/TrialSpawnerBlockEntity.java`. — Audited against Java 26.1.2: Rust models the wrapper's `BlockEntityType.TRIAL_SPAWNER` surface, default inactive state fallback when the block state lacks `TRIAL_SPAWNER_STATE`, state updates with block update flags `3`, level-gated `setEntityId` override behavior, mark-updated flags, state-data-only update tags/packets with component metadata omitted, and delegates save/load/ticking to the trial spawner state/config model. Covered by `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 trial_spawner`.
- [x] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/package-info.java`. — Audited against Java 26.1.2: metadata-only `@NullMarked` package annotation with no runtime behavior, protocol surface, serialization, or gameplay logic to port; Rust's type system already models nullability without a package-level annotation.

## `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/trialspawner`

- [x] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/trialspawner/PlayerDetector.java`. — Audited against Java 26.1.2: Rust models the three detector variants (`NO_CREATIVE_PLAYERS`, `INCLUDING_CREATIVE_PLAYERS`, and debug `SHEEP`), strict player range checks, spectator exclusion, creative-player inclusion/exclusion by variant, optional line-of-sight filtering, alive sheep filtering, and selector-owned candidate collection as caller-provided data. Covered by `cargo test -q -j 1 player_detector_variants_match_trial_spawner_java_filters`.
- [x] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/trialspawner/TrialSpawner.java`. — Audited against Java 26.1.2: Rust covers active/normal/ominous config access, default full config values, save/load delegation through the block-entity model, ominous apply/remove state effects, target cooldown and required player range, state accessor update flags, player detector override surface via `PlayerDetectorKind`, game-rule/difficulty spawn gating including the testing override, server tick state transitions, tracked-mob untracking at 47-block range, client spin math, flame-particle encode/decode and particle ids, spawn/reward/ominous level-event ids, reward ejection position, detect/spawn/eject particle counts, entity override reset behavior, state-data update tags, and state/config tests. Live `ServerLevel` entity creation, loot-table item ejection, clip/collision checks, particles, sounds, and ominous item-spawner placement are explicitly deferred by `TODO(trial-spawner-live-level)` until the entity/world/loot runtime is wired. Covered by `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 trial_spawner`.
- [x] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/trialspawner/TrialSpawnerConfig.java`. — Audited against Java 26.1.2: Rust covers codec/default fields (`spawn_range=4`, `total_mobs=6.0`, `simultaneous_mobs=2.0`, per-player additions `2.0`/`1.0`, `ticks_between_spawn=40`, empty default spawn potentials, consumables/key eject loot tables, and ominous item-spawner loot), codec range checks, target total/simultaneous mob floor math, `ticksBetweenItemSpawners()` returning `160`, `withSpawning(type)` replacing spawn potentials while preserving the rest of the config, weighted spawn-potential and loot-table decoding, and all bundled vanilla config resources. Covered by `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 trial_spawner` and `cargo test -q -j 1 trial_spawner_resource_defaults_and_range_checks_match_java_codec`.
- [x] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/trialspawner/TrialSpawnerConfigs.java`. — Audited against Java 26.1.2: Rust verifies the bootstrap-generated trial spawner config registry surface through the bundled vanilla resources: all 14 trial chamber config key pairs (`normal`/`ominous`) are present, all 28 generated JSON configs decode, normal/ominous spawn potentials are non-empty, ominous eject loot tables use Java's `key:3` and `consumables:7` weights, baby zombie and slime custom entity tags match the bootstrap helper output, and ominous melee/ranged configs preserve their equipment loot tables. Covered by `cargo test -q -j 1 trial_spawner`.
- [x] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/trialspawner/TrialSpawnerState.java`. — Audited against Java 26.1.2: Rust covers all serialized state names, light levels, spinning mob speeds, `hasSpinningMob`, spawn-capable flags, particle-emission mode selection, inactive/waiting/active/reward-ejection/ejecting/cooldown transitions, no-spawn and no-mob fallback states, detected-player activation, target total/simultaneous mob gates, reward shutter delay `40`, reward ejection cadence `30`, cooldown reactivation/completion behavior, ominous item-spawner cooldown timing, and client spin/state metadata. Live particle/sound/entity placement operations remain under `TODO(trial-spawner-live-level)`. Covered by `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 trial_spawner`.
- [x] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/trialspawner/TrialSpawnerStateData.java`. — Audited against Java 26.1.2: Rust covers packed save/load fields (`registered_players`, `current_mobs`, `cooldown_ends_at`, `next_mob_spawns_at`, `total_mobs_spawned`, optional `spawn_data`, optional `ejecting_loot_table`), reset/reset-statistics behavior, next-spawn-data selection/update tags, active-state update tags, spin/old-spin client state, additional-player counting, target total/simultaneous mob readiness, player scan throttle `(pos.asLong + gameTime) % 20`, detection spawn buffer `40`, bad-omen to trial-omen duration `18000 * (amplifier + 1)`, ominous reset timing, shutter/ejection/cooldown readiness helpers, low-resolution positional loot seed, and the cached dispensing/equipment-data resource surface. Live player effect mutation, display entity creation, loot item materialization, and level entity removal remain under `TODO(trial-spawner-live-level)`. Covered by `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 trial_spawner`.
- [x] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/trialspawner/package-info.java`. — Audited against Java 26.1.2: metadata-only `@NullMarked` package annotation with no runtime behavior, protocol surface, serialization, or gameplay logic to port; Rust's type system already models nullability without a package-level annotation.

## `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/vault`

- [x] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/vault/VaultBlockEntity.java`. — Audited against Java 26.1.2: Rust covers update packet/tag shape with shared data only, save/load of config/shared/server data, server/client accessor-equivalent state surfaces, activation/deactivation scan cadence, same-tick display cycling without skipping due state updates, key insertion gating, wrong-key/rewarded-player fail buffer timing, unlocking delay `14`, reward ejection queue order, display item updates, and client constants/particle geometry helpers. Live loot-table resolution, player stat/key mutation, block-state mutation, level events, sounds, spawned reward items, and actual particle emission remain under `TODO(vault-live-level)`. Covered by `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 vault`.
- [x] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/vault/VaultClientData.java`. — Audited against Java 26.1.2: Rust covers `ROTATION_SPEED=10.0`, previous/current spin accessors through fields, and `updateDisplayItemSpin()` behavior using wrapped 360-degree client spin. Covered by `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 vault`.
- [x] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/vault/VaultConfig.java`. — Audited against Java 26.1.2: Rust covers default loot table `minecraft:chests/trial_chambers/reward`, activation/deactivation ranges `4.0`/`4.5`, default trial key item, optional override display loot table, config save/load fields, and validation that activation range must be less than or equal to deactivation range. Live `PlayerDetector` selector plumbing remains under `TODO(vault-live-level)`. Covered by `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 vault`.
- [x] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/vault/VaultServerData.java`. — Audited against Java 26.1.2: Rust covers rewarded-player linked-set cap `128`, state-update pause timestamp, items-to-eject list, total-ejections-needed, transient last-insert-fail timestamp, next/pop item order from the end of the list, ejection completion reset, save/load fields, and Java ejection-progress math. Covered by `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 vault`.
- [x] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/vault/VaultSharedData.java`. — Audited against Java 26.1.2: Rust covers optional display item, connected-player set excluding rewarded players, default/persisted connected-particles range, shared-only update tag serialization, display-active-effects predicate, and Java parity that player-detection range does not mutate `connected_particles_range`. Live player detection through `ServerLevel` remains under `TODO(vault-live-level)`. Covered by `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 vault`.
- [x] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/vault/VaultState.java`. — Audited against Java 26.1.2: Rust covers serialized state names, light levels `6`/`12`, inactive/active connected-player transitions, unlocking-to-ejecting delay, 20-tick ejection cadence and post-finish pause, display clearing on inactive transition, reward item ejection order, and transition timing. Live transition sounds/events and spawned item side effects remain under `TODO(vault-live-level)`. Covered by `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 vault`.
- [x] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/vault/package-info.java`. — Audited against Java 26.1.2: metadata-only `@NullMarked` package annotation with no runtime behavior, protocol surface, serialization, or gameplay logic to port; Rust's type system already models nullability without a package-level annotation.

## `decompiled-server-26.1.2/net/minecraft/world/level/block/grower`

- [x] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/grower/TreeGrower.java`. — Audited against Java 26.1.2: Rust covers the named grower registry/codec surface (`oak`, `spruce`, `mangrove`, `azalea`, `birch`, `jungle`, `acacia`, `cherry`, `dark_oak`, `pale_oak`), secondary chance selection, flower-aware tree variants, mega-tree secondary selection, sapling-to-grower mapping, Java 2x2 mega sapling scan order `(0,0)`, `(0,-1)`, `(-1,0)`, `(-1,-1)`, sapling clear/restore plan positions, flower scan bounds from below/north(2)/west(2) through above/south(2)/east(2), and minimum-height lookup from the primary configured feature trunk base heights. Live `ConfiguredFeature.place`, fluid-state replacement, block update emission, and registry holder lookup remain under `TODO(tree-grower-live-feature)` plus the existing worldgen feature-execution checklist rows. Covered by `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 plant`.
- [x] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/grower/package-info.java`. — Audited against Java 26.1.2: metadata-only `@NullMarked` package annotation with no runtime behavior, protocol surface, serialization, or gameplay logic to port; Rust's type system already models nullability without a package-level annotation.

## `decompiled-server-26.1.2/net/minecraft/world/level/block`

- [x] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/package-info.java`. — Audited against Java 26.1.2: metadata-only `@NullMarked` package annotation with no runtime behavior, protocol surface, serialization, or gameplay logic to port; Rust's type system already models nullability without a package-level annotation.

## `decompiled-server-26.1.2/net/minecraft/world/level/block/piston`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/piston/MovingPistonBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/piston/PistonBaseBlock.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/piston/PistonHeadBlock.java`.
- [x] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/piston/PistonMath.java`. — Audited against Java 26.1.2: Rust `piston_movement_area` mirrors `getMovementArea(AABB, Direction, amount)` for all six directions, including Java's axis-direction signed delta, min/max sweep interval, and negative-amount behavior. Covered by `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 collision_shape`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/piston/PistonMovingBlockEntity.java`.
- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/piston/PistonStructureResolver.java`.
- [x] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/piston/package-info.java`. — Audited against Java 26.1.2: metadata-only `@NullMarked` package annotation with no runtime behavior, protocol surface, serialization, or gameplay logic to port; Rust's type system already models nullability without a package-level annotation.

## `decompiled-server-26.1.2/net/minecraft/world/level/block/sounds`

- [x] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/sounds/AmbientDesertBlockSoundsPlayer.java`. — Audited against Java 26.1.2: Rust `block_sounds` covers random gates (`2100`, `200`, `130`, badlands skip `3`), exact sound IDs (`block.sand.idle`, `block.dry_grass.ambient`, `block.deadbush.idle`), dry-vegetation tag membership (`#terracotta`, sand, red sand), sand tag membership (sand/red sand), two-block dry-vegetation support check, four-horizontal-column sand ambient counter/early-exit behavior, near-surface and far-column vertical scans with air-above requirement, and red-sand/terracotta badlands dead-bush skip behavior. Live `Level.playLocalSound`/`playPlayerSound` side effects remain modeled as planned sound emissions. Covered by `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 block_sounds`.
- [x] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/sounds/package-info.java`. — Audited against Java 26.1.2: metadata-only `@NullMarked` package annotation with no runtime behavior, protocol surface, serialization, or gameplay logic to port; Rust's type system already models nullability without a package-level annotation.

## `decompiled-server-26.1.2/net/minecraft/world/level/block/state`

- [ ] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/state/BlockBehaviour.java`.
  Progress: Java `BlockBehaviour.Properties` default values, builder mutations, legacy/full copy semantics, dependent loot-table/description-id derivation, and `OffsetType`/`Mth.getSeed` offset math are modeled in `block_behaviour_properties`; verified by `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 block_behaviour_properties`.
  Progress: Java base `BlockBehaviour` default methods for pathfinding, no-op state updates/events, use result defaults, render/signal/default survivability returns, replaceability gates, shape fallbacks, light dampening, shade brightness, skylight propagation, destroy progress, seed calculation, and the `BlockStateBase` solid/cache helper rules are modeled in `block_behaviour_defaults`; verified by `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 block_behaviour_defaults`.
  Progress: Java `BlockStateBase` constructor field capture, `initCache` cache/no-cache behavior, `blocksMotion` cobweb/bamboo-sapling exceptions, cached collision/occlusion/light fields, offset accessors, and exposed flags (`canBeReplaced`, terrain particles, piston push reaction, instrument, predicates) are modeled in `block_behaviour_defaults`; verified by `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 block_state_base`.
  Progress: Java `BlockStateBase` owner-delegating methods, neighbour shape update direction/opposite order, cache face-support indexing, cached-vs-live collision/full-block fallback, and `BlockBehaviour.onExplosionHit` side-effect ordering/gates are modeled as action plans in `block_behaviour_defaults`. Live `ServerLevel` loot/block-entity side effects remain explicitly TODO-gated until the shared live block-state behavior path exists; verified by `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 block_behaviour_delegation`.
- [x] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/state/BlockState.java`. Java's concrete `BlockState` wrapper is represented by `BlockStateModel`: construction from owner plus property pairs, default-state lookup through `BlockStateDefinition`, stable sorted state formatting through the audited `StateHolder` surface, and `as_state` self-reference semantics are covered by `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 block_state_concrete_wrapper`.
- [x] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/state/StateDefinition.java`. Rust `BlockStateDefinition` now covers Java's owner access, sorted property lookup/iteration, singleton detection, `any`/default state surface, cartesian possible-state generation, `toString`-style summary, and builder validation rules for `^[a-z0-9_]+$` names, duplicate properties, single-value properties, and invalid property values. Java codec/neighbour initialization is represented through the `BlockStateModel` value helpers audited with `StateHolder.java`; verified by `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 state_definition`.
- [x] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/state/StateHolder.java`. Rust `BlockStateModel` now exposes Java `StateHolder` tag names, singleton detection, sorted property/value iteration, get/default lookup, try-set no-op for absent properties, allowed-value set validation, wraparound cycle behavior, and Java-style state string formatting. Java's neighbor-array cache is represented behaviorally by immutable value updates in Rust rather than identity-cached state instances; verified by `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 block_state_holder`.
- [x] Audit, port or explicitly defer, and parity-test `decompiled-server-26.1.2/net/minecraft/world/level/block/state/package-info.java`. Metadata-only `@NullMarked` package annotation; no Rust runtime behavior needed beyond the state-holder null-free surface covered by `RUSTCRAFT_SKIP_LINE_CHECK=1 cargo test -q -j 1 block_state_holder`.

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
