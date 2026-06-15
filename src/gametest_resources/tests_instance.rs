use super::*;
use crate::core_block_pos::RotationModel;

fn vanilla_data_path(parts: &[&str]) -> std::path::PathBuf {
    let Some(source_root) = option_env!("VIBECRAFT_DECOMPILED_SOURCE_ROOT") else {
        panic!("VIBECRAFT_DECOMPILED_SOURCE_ROOT is required for Java-source parity tests");
    };
    parts
        .iter()
        .fold(std::path::PathBuf::from(source_root), |path, part| {
            path.join(part)
        })
}

const GAME_TEST_INSTANCE_JAVA: &str = include_str!(
    "../../../decompiled-server-26.1.2/net/minecraft/gametest/framework/GameTestInstance.java"
);
const GAME_TEST_INSTANCES_JAVA: &str = include_str!(
    "../../../decompiled-server-26.1.2/net/minecraft/gametest/framework/GameTestInstances.java"
);
const GAME_TEST_LISTENER_JAVA: &str = include_str!(
    "../../../decompiled-server-26.1.2/net/minecraft/gametest/framework/GameTestListener.java"
);
const GAME_TEST_MAIN_UTIL_JAVA: &str = include_str!(
    "../../../decompiled-server-26.1.2/net/minecraft/gametest/framework/GameTestMainUtil.java"
);
const GAME_TEST_RUNNER_JAVA: &str = include_str!(
    "../../../decompiled-server-26.1.2/net/minecraft/gametest/framework/GameTestRunner.java"
);
const GAME_TEST_SEQUENCE_JAVA: &str = include_str!(
    "../../../decompiled-server-26.1.2/net/minecraft/gametest/framework/GameTestSequence.java"
);
const GAME_TEST_SERVER_JAVA: &str = include_str!(
    "../../../decompiled-server-26.1.2/net/minecraft/gametest/framework/GameTestServer.java"
);
const STRUCTURE_GRID_SPAWNER_JAVA: &str = include_str!(
    "../../../decompiled-server-26.1.2/net/minecraft/gametest/framework/StructureGridSpawner.java"
);
const GAME_TEST_TICKER_JAVA: &str = include_str!(
    "../../../decompiled-server-26.1.2/net/minecraft/gametest/framework/GameTestTicker.java"
);

fn instance() -> GameTestInstanceModel {
    GameTestInstanceModel {
        kind: GameTestInstanceTypeModel::Function,
        data: GameTestInstanceDataModel {
            environment: "minecraft:default".to_string(),
            structure: "minecraft:empty".to_string(),
            max_ticks: 100,
            setup_ticks: 5,
            required: false,
            rotation: RotationModel::Clockwise90,
            manual_only: true,
            max_attempts: 4,
            required_successes: 2,
            sky_access: true,
            padding: 3,
        },
    }
}

#[test]
fn gametest_instance_matches_java_abstract_base_shape() {
    assert_eq!(GAME_TEST_INSTANCE_JAVA.lines().count(), 109);
    for sentinel in [
        "public static final Codec<GameTestInstance> DIRECT_CODEC",
        "BuiltInRegistries.TEST_INSTANCE_TYPE.byNameCodec().dispatch(GameTestInstance::codec, i -> i)",
        "register(registry, \"block_based\", BlockBasedTestInstance.CODEC);",
        "return register(registry, \"function\", FunctionGameTestInstance.CODEC);",
        "ResourceKey.create(Registries.TEST_INSTANCE_TYPE, Identifier.withDefaultNamespace(name))",
        "public abstract void run(GameTestHelper helper);",
        "public abstract MapCodec<? extends GameTestInstance> codec();",
        "return this.info.environment();",
        "return this.info.structure();",
        "return this.info.maxTicks();",
        "return this.info.setupTicks();",
        "return this.info.required();",
        "return this.info.manualOnly();",
        "return this.info.maxAttempts();",
        "return this.info.requiredSuccesses();",
        "return this.info.skyAccess();",
        "return this.info.rotation();",
        "return this.info.padding();",
        "return this.describeType().append(this.describeInfo());",
        "return this.descriptionRow(\"test_instance.description.type\", this.typeDescription());",
        "return this.descriptionRow(\"test_instance.description.structure\", this.info.structure().toString())",
        ".append(this.descriptionRow(\"test_instance.description.batch\", this.info.environment().getRegisteredName()))",
        "return Component.translatable(translationKey, value.withStyle(ChatFormatting.BLUE)).append(Component.literal(\"\\n\"));",
    ] {
        assert!(
            GAME_TEST_INSTANCE_JAVA.contains(sentinel),
            "missing GameTestInstance sentinel {sentinel}"
        );
    }
}

#[test]
fn gametest_instance_bootstraps_block_based_then_function_types() {
    assert_eq!(
        bootstrap_gametest_instance_types(),
        vec![
            (
                "minecraft:block_based",
                GameTestInstanceTypeModel::BlockBased
            ),
            ("minecraft:function", GameTestInstanceTypeModel::Function),
        ]
    );
}

#[test]
fn gametest_instance_accessors_delegate_to_test_data() {
    let instance = instance();
    assert_eq!(instance.batch(), "minecraft:default");
    assert_eq!(instance.structure(), "minecraft:empty");
    assert_eq!(instance.max_ticks(), 100);
    assert_eq!(instance.setup_ticks(), 5);
    assert!(!instance.required());
    assert!(instance.manual_only());
    assert_eq!(instance.max_attempts(), 4);
    assert_eq!(instance.required_successes(), 2);
    assert!(instance.sky_access());
    assert_eq!(instance.rotation(), RotationModel::Clockwise90);
    assert_eq!(instance.padding(), 3);
}

#[test]
fn gametest_instance_description_rows_match_java_order() {
    let instance = instance();
    assert_eq!(instance.type_description(), "test_instance.type.function");
    assert_eq!(
        instance.describe_info(),
        [
            (
                "test_instance.description.structure",
                "minecraft:empty".to_string()
            ),
            (
                "test_instance.description.batch",
                "minecraft:default".to_string()
            ),
        ]
    );
    assert_eq!(
        instance.describe(),
        vec![
            (
                "test_instance.description.type",
                "test_instance.type.function".to_string()
            ),
            (
                "test_instance.description.structure",
                "minecraft:empty".to_string()
            ),
            (
                "test_instance.description.batch",
                "minecraft:default".to_string()
            ),
        ]
    );
}

#[test]
fn gametest_instances_matches_java_bootstrap_shape() {
    assert_eq!(GAME_TEST_INSTANCES_JAVA.lines().count(), 28);
    for sentinel in [
        "ResourceKey<GameTestInstance> ALWAYS_PASS = create(\"always_pass\");",
        "HolderGetter<Consumer<GameTestHelper>> functions = context.lookup(Registries.TEST_FUNCTION);",
        "HolderGetter<TestEnvironmentDefinition<?>> batches = context.lookup(Registries.TEST_ENVIRONMENT);",
        "BuiltinTestFunctions.ALWAYS_PASS",
        "batches.getOrThrow(GameTestEnvironments.DEFAULT_KEY)",
        "Identifier.withDefaultNamespace(\"empty\")",
        "new TestData<>(batches.getOrThrow(GameTestEnvironments.DEFAULT_KEY), Identifier.withDefaultNamespace(\"empty\"), 1, 1, false)",
        "ResourceKey.create(Registries.TEST_INSTANCE, Identifier.withDefaultNamespace(id))",
    ] {
        assert!(
            GAME_TEST_INSTANCES_JAVA.contains(sentinel),
            "missing GameTestInstances sentinel {sentinel}"
        );
    }
}

#[test]
fn gametest_instances_bootstraps_vanilla_always_pass_instance() {
    let instances = bootstrap_gametest_instances();
    assert_eq!(instances.len(), 1);
    assert_eq!(instances[0].0, ALWAYS_PASS_GAMETEST_INSTANCE_ID);
    assert_eq!(instances[0].1.kind, GameTestInstanceTypeModel::Function);
    assert_eq!(
        instances[0].1.data,
        GameTestInstanceDataModel {
            environment: DEFAULT_GAMETEST_ENVIRONMENT_KEY.to_string(),
            structure: "minecraft:empty".to_string(),
            max_ticks: 1,
            setup_ticks: 1,
            required: false,
            rotation: RotationModel::None,
            manual_only: false,
            max_attempts: 1,
            required_successes: 1,
            sky_access: false,
            padding: 0,
        }
    );

    let vanilla_path =
        vanilla_data_path(&["data", "minecraft", "test_instance", "always_pass.json"]);
    let vanilla = parse_test_instance_json(
        &std::fs::read_to_string(&vanilla_path)
            .unwrap_or_else(|err| panic!("failed to read {vanilla_path:?}: {err}")),
    )
    .unwrap();
    assert_eq!(vanilla.environment, instances[0].1.data.environment);
    assert_eq!(vanilla.structure, instances[0].1.data.structure);
    assert_eq!(vanilla.max_ticks, instances[0].1.data.max_ticks);
    assert_eq!(vanilla.setup_ticks, instances[0].1.data.setup_ticks);
    assert_eq!(vanilla.required, instances[0].1.data.required);
}

#[test]
fn gametest_listener_matches_java_interface_shape() {
    assert_eq!(GAME_TEST_LISTENER_JAVA.lines().count(), 11);
    for sentinel in [
        "public interface GameTestListener",
        "void testStructureLoaded(GameTestInfo testInfo);",
        "void testPassed(GameTestInfo testInfo, GameTestRunner runner);",
        "void testFailed(GameTestInfo testInfo, GameTestRunner runner);",
        "void testAddedForRerun(GameTestInfo original, GameTestInfo copy, GameTestRunner runner);",
    ] {
        assert!(
            GAME_TEST_LISTENER_JAVA.contains(sentinel),
            "missing GameTestListener sentinel {sentinel}"
        );
    }
}

#[test]
fn gametest_listener_records_all_callbacks_in_order() {
    let original = GameTestInfoStateModel::new(
        "minecraft:original",
        true,
        20,
        0,
        RotationModel::None,
        RotationModel::None,
        "noRetries",
    );
    let copy = GameTestInfoStateModel::new(
        "minecraft:copy",
        true,
        20,
        0,
        RotationModel::None,
        RotationModel::None,
        "noRetries",
    );
    let mut listener = RecordingGameTestListener::default();

    listener.test_structure_loaded(&original);
    listener.test_passed(&original, "runner");
    listener.test_failed(&copy, "runner");
    listener.test_added_for_rerun(&original, &copy, "runner");

    assert_eq!(
        listener.events,
        vec![
            GameTestListenerEvent::StructureLoaded {
                test_id: "minecraft:original".to_string(),
            },
            GameTestListenerEvent::Passed {
                test_id: "minecraft:original".to_string(),
                runner: "runner".to_string(),
            },
            GameTestListenerEvent::Failed {
                test_id: "minecraft:copy".to_string(),
                runner: "runner".to_string(),
            },
            GameTestListenerEvent::AddedForRerun {
                original_id: "minecraft:original".to_string(),
                copy_id: "minecraft:copy".to_string(),
                runner: "runner".to_string(),
            },
        ]
    );
}

#[test]
fn gametest_main_util_matches_java_cli_and_server_bootstrap_shape() {
    assert_eq!(GAME_TEST_MAIN_UTIL_JAVA.lines().count(), 138);
    for sentinel in [
        "private static final String DEFAULT_UNIVERSE_DIR = \"gametestserver\";",
        "private static final String LEVEL_NAME = \"gametestworld\";",
        "parser.accepts(\n         \"universe\"",
        "parser.accepts(\"report\", \"Exports results in a junit-like XML report at the given path.\")",
        "parser.accepts(\n         \"tests\"",
        "parser.accepts(\n         \"verify\"",
        "parser.accepts(\"repeatCount\", \"Runs each of the specified tests this many times\")",
        "parser.accepts(\"packs\", \"A folder of datapacks to include in the world\")",
        "parser.allowsUnrecognizedOptions();",
        "Please specify a test selection to run the verify option. For example: --verify --tests example:test_something_*",
        "Flag --verify is true, the --repeatCount value will be ignored",
        "GlobalTestReporter.replaceWith(new JUnitLikeTestReporter((File)report.value(options)));",
        "Bootstrap.bootStrap();",
        "Util.startTimerHackThread();",
        "createOrResetDir(universePath);",
        "onUniverseCreated.accept(universePath);",
        "copyPacks(universePath, packFolder);",
        "LevelStorageSource.createDefault(Paths.get(universePath)).createAccess(\"gametestworld\")",
        "ServerPacksSource.createPackRepository(levelStorageSource)",
        "MinecraftServer.spin(",
        "optionalFromOption(options, tests)",
        "FileUtils.deleteDirectory(universeDir.toFile());",
        "Files.createDirectories(universeDir);",
        "Paths.get(serverPath).resolve(\"gametestworld\").resolve(\"datapacks\")",
        "Files.isRegularFile(path.resolve(\"pack.mcmeta\"))",
        "path.toString().endsWith(\".zip\")",
    ] {
        assert!(
            GAME_TEST_MAIN_UTIL_JAVA.contains(sentinel),
            "missing GameTestMainUtil sentinel {sentinel}"
        );
    }
    assert_eq!(
        GAMETEST_MAIN_UTIL_SERVER_RUNTIME_TODO,
        "gametest-main-util-server-runtime"
    );
}

#[test]
fn gametest_main_util_models_help_verify_and_launch_decisions() {
    assert_eq!(
        GameTestMainOptionsModel::parse(&["--help"])
            .unwrap()
            .decision(false),
        GameTestMainDecision::PrintHelp
    );
    assert_eq!(
        GameTestMainOptionsModel::parse(&["--verify", "true"])
            .unwrap()
            .decision(false),
        GameTestMainDecision::VerifyRequiresTests {
            exit_code: -1,
            message: "Please specify a test selection to run the verify option. For example: --verify --tests example:test_something_*",
        }
    );

    let options = GameTestMainOptionsModel::parse(&[
        "--unknown",
        "--universe",
        "tmp-gametest",
        "--report",
        "report.xml",
        "--tests",
        "minecraft:always_*",
        "--verify",
        "true",
        "--repeatCount",
        "10",
        "--packs",
        "packs",
    ])
    .unwrap();
    assert_eq!(
        options.decision(true),
        GameTestMainDecision::Launch(GameTestServerLaunchModel {
            universe_path: "tmp-gametest".to_string(),
            level_name: GAMETEST_LEVEL_NAME,
            report: Some("report.xml".to_string()),
            tests: Some("minecraft:always_*".to_string()),
            verify: true,
            repeat_count: 10,
            packs: Some("packs".to_string()),
            verify_ignores_repeat_count: true,
        })
    );
}

#[test]
fn gametest_main_util_filters_pack_copy_targets_like_java() {
    let entries = vec![
        PackSourceEntryModel {
            name: "folder-pack".to_string(),
            is_directory: true,
            has_pack_mcmeta: true,
            is_zip: false,
        },
        PackSourceEntryModel {
            name: "missing-meta".to_string(),
            is_directory: true,
            has_pack_mcmeta: false,
            is_zip: false,
        },
        PackSourceEntryModel {
            name: "archive.zip".to_string(),
            is_directory: false,
            has_pack_mcmeta: false,
            is_zip: true,
        },
        PackSourceEntryModel {
            name: "notes.txt".to_string(),
            is_directory: false,
            has_pack_mcmeta: false,
            is_zip: false,
        },
    ];
    assert_eq!(
        gametest_main_pack_copy_targets(&entries),
        vec!["folder-pack".to_string(), "archive.zip".to_string()]
    );
}

#[test]
fn gametest_runner_matches_java_batch_lifecycle_shape() {
    assert_eq!(GAME_TEST_RUNNER_JAVA.lines().count(), 259);
    for sentinel in [
        "public static final int DEFAULT_TESTS_PER_ROW = 8;",
        "private final List<GameTestBatchListener> batchListeners = Lists.newArrayList();",
        "private final List<GameTestInfo> scheduledForRerun = Lists.newArrayList();",
        "private boolean stopped = true;",
        "testTicker.setRunner(this);",
        "this.allTestInfos.forEach(info -> info.addListener(new ReportGameListener()));",
        "this.stopped = false;",
        "this.runBatch(0);",
        "this.stopped = true;",
        "GameTestInfo copy = info.copyReset();",
        "listener.testAddedForRerun(info, copy, this)",
        "if (batchIndex >= this.batches.size())",
        "this.endCurrentEnvironment();",
        "this.runScheduledRerunTests();",
        "if (batchIndex > 0 && this.clearBetweenBatches)",
        "this.batchListeners.forEach(listener -> listener.testBatchStarting(currentBatch));",
        "this.batchListeners.forEach(listener -> listener.testBatchFinished(currentBatch));",
        "forcedChunks.forEach(pos -> GameTestRunner.this.level.setChunkForced(ChunkPos.getX(pos), ChunkPos.getZ(pos), false));",
        "GameTestTicker.SINGLETON.clear();",
        "this.batches = ImmutableList.copyOf(this.testBatcher.batch(this.scheduledForRerun));",
        "this.batches = ImmutableList.of();",
        "private GameTestRunner.GameTestBatcher batcher = GameTestBatchFactory.fromGameTestInfo();",
        "private GameTestRunner.StructureSpawner existingStructureSpawner = GameTestRunner.StructureSpawner.IN_PLACE;",
        "private GameTestRunner.StructureSpawner newStructureSpawner = GameTestRunner.StructureSpawner.NOT_SET;",
        "private boolean haltOnError = false;",
        "private boolean clearBetweenBatches = false;",
        "StructureSpawner IN_PLACE = testInfo -> Optional.ofNullable(testInfo.prepareTestStructure()).map(e -> e.startExecution(1));",
        "StructureSpawner NOT_SET = testInfo -> Optional.empty();",
    ] {
        assert!(
            GAME_TEST_RUNNER_JAVA.contains(sentinel),
            "missing GameTestRunner sentinel {sentinel}"
        );
    }
    assert_eq!(DEFAULT_GAMETESTS_PER_ROW, 8);
    assert_eq!(
        GAMETEST_RUNNER_SERVER_RUNTIME_TODO,
        "gametest-runner-server-runtime"
    );
}

#[test]
fn gametest_runner_builder_defaults_match_java() {
    let batch = create_gametest_batch(
        0,
        vec!["minecraft:always_pass".to_string()],
        "minecraft:default",
    )
    .unwrap();
    let builder = GameTestRunnerBuilderModel::from_batches(vec![batch]);
    assert_eq!(builder.batcher, "GameTestBatchFactory.fromGameTestInfo");
    assert_eq!(
        builder.existing_structure_spawner,
        StructureSpawnerModel::InPlace
    );
    assert_eq!(builder.new_structure_spawner, StructureSpawnerModel::NotSet);
    assert!(!builder.halt_on_error);
    assert!(!builder.clear_between_batches);
}

#[test]
fn gametest_runner_starts_batches_tears_down_and_completes() {
    let batches = vec![
        create_gametest_batch(0, vec!["one".to_string()], "minecraft:env_a").unwrap(),
        create_gametest_batch(1, vec!["two".to_string()], "minecraft:env_b").unwrap(),
    ];
    let mut runner = GameTestRunnerBuilderModel::from_batches(batches)
        .clear_between_batches()
        .build();
    assert_eq!(
        runner.all_test_infos,
        vec!["one".to_string(), "two".to_string()]
    );

    runner.start();
    assert!(!runner.stopped);
    assert_eq!(
        runner.current_environment,
        Some("minecraft:env_a".to_string())
    );
    runner.complete_batch(0, false);
    assert_eq!(
        runner.current_environment,
        Some("minecraft:env_b".to_string())
    );
    runner.complete_batch(1, false);
    assert!(runner.stopped);
    assert!(runner.current_environment.is_none());
    assert!(runner.batches.is_empty());
    assert!(runner.events.contains(&GameTestRunnerEvent::BatchStarting {
        index: 0,
        environment: "minecraft:env_a".to_string(),
    }));
    assert!(runner.events.contains(&GameTestRunnerEvent::BatchFinished {
        index: 1,
        environment: "minecraft:env_b".to_string(),
    }));
}

#[test]
fn gametest_runner_halt_on_error_tears_down_and_clears_ticker() {
    let batch = create_gametest_batch(0, vec!["failing".to_string()], "minecraft:default").unwrap();
    let mut runner = GameTestRunnerBuilderModel::from_batches(vec![batch])
        .halt_on_error()
        .build();

    runner.start();
    runner.complete_batch(0, true);

    assert!(runner.stopped);
    assert!(runner.current_environment.is_none());
    assert!(runner
        .events
        .contains(&GameTestRunnerEvent::ForcedChunksCleared));
    assert!(runner.events.contains(&GameTestRunnerEvent::TickerCleared));
}

#[test]
fn gametest_runner_rerun_copy_is_batched_when_stopped() {
    let mut runner = GameTestRunnerBuilderModel::from_batches(Vec::new()).build();
    let original = GameTestInfoStateModel::new(
        "minecraft:rerun",
        true,
        20,
        0,
        RotationModel::None,
        RotationModel::None,
        "noRetries",
    );

    runner.rerun_test(&original);

    assert!(!runner.stopped);
    assert_eq!(runner.scheduled_for_rerun, Vec::<String>::new());
    assert_eq!(runner.batches.len(), 1);
    assert_eq!(runner.batches[0].game_test_infos, vec!["minecraft:rerun"]);
    assert!(runner
        .events
        .contains(&GameTestRunnerEvent::TestAddedForRerun {
            original_id: "minecraft:rerun".to_string(),
            copy_id: "minecraft:rerun".to_string(),
        }));
}

#[test]
fn gametest_runner_structure_spawners_match_java_defaults() {
    let mut info = GameTestInfoStateModel::new(
        "minecraft:spawn",
        true,
        20,
        2,
        RotationModel::None,
        RotationModel::None,
        "noRetries",
    );
    assert_eq!(
        StructureSpawnerModel::NotSet.spawn_structure(&mut info),
        None
    );
    assert_eq!(
        StructureSpawnerModel::InPlace.spawn_structure(&mut info),
        Some("minecraft:spawn".to_string())
    );
    assert_eq!(info.tick_count, -4);
}

#[test]
fn gametest_sequence_matches_java_event_queue_shape() {
    assert_eq!(GAME_TEST_SEQUENCE_JAVA.lines().count(), 146);
    for sentinel in [
        "private final GameTestInfo parent;",
        "private final List<GameTestEvent> events = Lists.newArrayList();",
        "private int lastTick;",
        "this.lastTick = parent.getTick();",
        "this.events.add(GameTestEvent.create(assertion));",
        "this.events.add(GameTestEvent.create(expectedDelay, assertion));",
        "this.events.add(GameTestEvent.createWithMinimumDelay(minimumDelay, assertion));",
        "return this.thenExecuteAfter(delta, () -> {});",
        "this.events.add(GameTestEvent.create(() -> this.executeWithoutFail(assertion)));",
        "if (this.parent.getTick() < this.lastTick + delta)",
        "Component.translatable(\"test.error.sequence.not_completed\")",
        "this.events.add(GameTestEvent.create(this.parent::succeed));",
        "this.events.add(GameTestEvent.create(() -> this.parent.fail(e.get())));",
        "this.events.add(GameTestEvent.create(() -> result.trigger(this.parent.getTick())));",
        "this.parent.fail(e);",
        "event.assertion.run();",
        "iterator.remove();",
        "event.minimumDelay != null && event.minimumDelay > delay",
        "test.error.sequence.minimum_tick",
        "event.expectedDelay != null && event.expectedDelay != delay",
        "test.error.sequence.invalid_tick",
        "private static final int NOT_TRIGGERED = -1;",
        "throw new IllegalStateException(\"Condition already triggered at \" + this.triggerTime);",
        "test.error.sequence.condition_not_triggered",
        "test.error.sequence.condition_already_triggered",
    ] {
        assert!(
            GAME_TEST_SEQUENCE_JAVA.contains(sentinel),
            "missing GameTestSequence sentinel {sentinel}"
        );
    }
}

#[test]
fn gametest_sequence_validates_expected_and_minimum_delays() {
    let mut sequence = GameTestSequenceModel::new(10);
    sequence
        .then_wait_until_delay(3, "expected")
        .then_wait_at_least(5, "minimum")
        .then_succeed();
    assert_eq!(sequence.tick(13), GameTestSequenceOutcome::Continue);
    assert_eq!(
        sequence.tick(16),
        GameTestSequenceOutcome::ParentFailed {
            message: "test.error.sequence.minimum_tick:18".to_string(),
            tick: 16,
        }
    );

    let mut invalid = GameTestSequenceModel::new(0);
    invalid.then_wait_until_delay(2, "expected");
    assert_eq!(
        invalid.tick(3),
        GameTestSequenceOutcome::ParentFailed {
            message: "test.error.sequence.invalid_tick:2".to_string(),
            tick: 3,
        }
    );
}

#[test]
fn gametest_sequence_tick_variants_and_parent_events_match_java() {
    let mut sequence = GameTestSequenceModel::new(0);
    sequence.then_execute_after(4, "late");
    assert_eq!(
        sequence.tick_and_continue(2),
        GameTestSequenceOutcome::Continue
    );
    assert_eq!(sequence.events.len(), 1);
    assert_eq!(
        sequence.tick_and_fail_if_not_complete(2),
        GameTestSequenceOutcome::ParentFailed {
            message: "test.error.sequence.not_completed".to_string(),
            tick: 2,
        }
    );

    let mut parent = GameTestSequenceModel::new(0);
    parent.then_fail("boom");
    assert_eq!(
        parent.tick(0),
        GameTestSequenceOutcome::ParentFailed {
            message: "boom".to_string(),
            tick: 0,
        }
    );

    let mut succeed = GameTestSequenceModel::new(0);
    succeed.then_succeed();
    assert_eq!(succeed.tick(0), GameTestSequenceOutcome::ParentSucceeded);
}

#[test]
fn gametest_sequence_condition_matches_java_trigger_rules() {
    let mut condition = GameTestSequenceConditionModel::default();
    assert_eq!(
        condition.assert_triggered_this_tick(5),
        Err("test.error.sequence.condition_not_triggered".to_string())
    );
    assert_eq!(condition.trigger(4), Ok(()));
    assert_eq!(
        condition.assert_triggered_this_tick(5),
        Err("test.error.sequence.condition_already_triggered:4".to_string())
    );
    assert_eq!(condition.assert_triggered_this_tick(4), Ok(()));
    assert_eq!(
        condition.trigger(6),
        Err("Condition already triggered at 4".to_string())
    );
}

#[test]
fn gametest_server_matches_java_server_bootstrap_and_runtime_shape() {
    assert_eq!(GAME_TEST_SERVER_JAVA.lines().count(), 433);
    for sentinel in [
        "private static final int PROGRESS_REPORT_INTERVAL = 20;",
        "private static final int TEST_POSITION_RANGE = 14999992;",
        "FeatureFlags.REGISTRY",
        "FeatureFlags.REDSTONE_EXPERIMENTS, FeatureFlags.MINECART_IMPROVEMENTS",
        "private static final WorldOptions WORLD_OPTIONS = new WorldOptions(0L, false, false);",
        "packRepository.reload();",
        "enabledPacks.remove(\"vanilla\");",
        "enabledPacks.addFirst(\"vanilla\");",
        "new LevelSettings(\"Test Level\", GameType.CREATIVE",
        "WorldPresets.FLAT",
        "System.exit(-1);",
        "this.setPlayerList(new PlayerList(this, this.registries(), this.playerDataStorage, new EmptyNotificationService()) {});",
        "Gizmos.withCollector(GizmoCollector.NOOP);",
        "this.testBatches = this.evaluateTestsToRun(level);",
        "getTestsForSelection(level.registryAccess(), this.testSelection.get()).filter(test -> !test.value().manualOnly()).toList();",
        "decorator = GameTestServer::rotateAndMultiply;",
        "100 * Rotation.values().length",
        "decorator = this::multiplyTest;",
        "decorator = GameTestBatchFactory.DIRECT;",
        "testRegistry.listElements().filter(test -> !test.value().manualOnly()).toList();",
        "for (Rotation rotation : Rotation.values())",
        "for (int i = 0; i < 100; i++)",
        "ResourceSelectorArgument.parse(new StringReader(selection)",
        "for (int i = 0; i < this.repeatCount; i++)",
        "if (level.getGameTime() % 20L == 0L)",
        "GlobalTestReporter.finish();",
        "this.testTracker.hasFailedRequired()",
        "this.testTracker.hasFailedOptional()",
        "testInfo.getRotation() != Rotation.NONE",
        "systemReport.setDetail(\"Type\", \"Game test server\");",
        "System.exit(this.testTracker != null ? this.testTracker.getFailedRequiredCount() : -1);",
        "System.exit(1);",
        "random.nextIntBetweenInclusive(-14999992, 14999992)",
        "new StructureGridSpawner(startPos, 8, false)",
        "return false;",
        "return LevelBasedPermissionSet.ALL;",
        "return LevelBasedPermissionSet.OWNER;",
        "return 0;",
        "return 1;",
        "return Optional.empty();",
        "Optional.of(NameAndId.createOffline(name))",
    ] {
        assert!(
            GAME_TEST_SERVER_JAVA.contains(sentinel),
            "missing GameTestServer sentinel {sentinel}"
        );
    }
    assert_eq!(GAMETEST_SERVER_RUNTIME_TODO, "gametest-server-runtime");
    assert_eq!(GAMETEST_SERVER_PROGRESS_REPORT_INTERVAL, 20);
    assert_eq!(GAMETEST_SERVER_TEST_POSITION_RANGE, 14_999_992);
    assert_eq!(GAMETEST_SERVER_WORLD_SEED, 0);
    assert_eq!(
        gametest_server_enabled_feature_policy(),
        "all_except_redstone_experiments_and_minecart_improvements"
    );
}

fn server_tests() -> Vec<GameTestServerTestModel> {
    vec![
        GameTestServerTestModel {
            id: "minecraft:always_pass".to_string(),
            manual_only: false,
            required: false,
        },
        GameTestServerTestModel {
            id: "minecraft:manual".to_string(),
            manual_only: true,
            required: true,
        },
    ]
}

#[test]
fn gametest_server_evaluates_selection_verify_repeat_and_direct_paths() {
    let verify = evaluate_gametest_server_tests(
        &GameTestServerOptionsModel {
            test_selection: Some("minecraft:always_*".to_string()),
            verify: true,
            repeat_count: 1,
        },
        &server_tests(),
    )
    .unwrap();
    assert_eq!(verify.decorator, GameTestServerDecoratorModel::Verify);
    assert_eq!(verify.tests.len(), 400);
    assert_eq!(verify.tests[0].rotation, RotationModel::None);
    assert_eq!(verify.tests[100].rotation, RotationModel::Clockwise90);

    let repeated = evaluate_gametest_server_tests(
        &GameTestServerOptionsModel {
            test_selection: Some("minecraft:always_pass".to_string()),
            verify: false,
            repeat_count: 3,
        },
        &server_tests(),
    )
    .unwrap();
    assert_eq!(repeated.decorator, GameTestServerDecoratorModel::Repeat);
    assert_eq!(repeated.tests.len(), 3);
    assert!(repeated
        .tests
        .iter()
        .all(|test| test.rotation == RotationModel::None));

    let direct = evaluate_gametest_server_tests(
        &GameTestServerOptionsModel {
            test_selection: None,
            verify: false,
            repeat_count: 1,
        },
        &server_tests(),
    )
    .unwrap();
    assert_eq!(direct.decorator, GameTestServerDecoratorModel::Direct);
    assert_eq!(direct.tests.len(), 1);
    assert_eq!(direct.tests[0].test_id, "minecraft:always_pass");

    assert_eq!(
        evaluate_gametest_server_tests(
            &GameTestServerOptionsModel {
                test_selection: Some("minecraft:missing".to_string()),
                verify: false,
                repeat_count: 1,
            },
            &server_tests(),
        ),
        Err("Test selection matcher found no tests".to_string())
    );
}

#[test]
fn gametest_server_models_properties_start_positions_and_failure_logs() {
    assert_eq!(
        gametest_server_properties(),
        GameTestServerPropertiesModel {
            hard_core: false,
            rcon_broadcast: false,
            dedicated_server: false,
            rate_limit_packets_per_second: 0,
            native_transport: false,
            published: false,
            inform_admins: false,
            max_players: 1,
            tick_time_logging_enabled: false,
            system_report_type: "Game test server",
        }
    );
    assert_eq!(
        gametest_server_start_pos(-14_999_992, 14_999_992),
        Ok((-14_999_992, -59, 14_999_992))
    );
    assert_eq!(
        gametest_server_start_pos(14_999_993, 0),
        Err("start position outside GameTestServer range".to_string())
    );
    assert_eq!(
        gametest_server_failed_test_log("minecraft:fail", RotationModel::None, "boom"),
        "minecraft:fail: boom"
    );
    assert_eq!(
        gametest_server_failed_test_log("minecraft:fail", RotationModel::Clockwise90, "boom"),
        "minecraft:fail with rotation clockwise_90: boom"
    );
}

#[test]
fn gametest_server_mock_resolvers_match_java_empty_and_offline_behavior() {
    assert_eq!(mock_profile_fetch_by_name("Steve"), None);
    assert_eq!(mock_profile_fetch_by_id("uuid"), None);

    let mut resolver = MockUserNameToIdResolverModel::default();
    assert_eq!(
        resolver.get_by_name("Alex"),
        Some(("Alex".to_string(), "offline:Alex".to_string()))
    );
    assert_eq!(resolver.get_by_id("known"), None);
    resolver.add("Alex", "known");
    assert_eq!(
        resolver.get_by_name("Alex"),
        Some(("Alex".to_string(), "known".to_string()))
    );
    assert_eq!(
        resolver.get_by_id("known"),
        Some(("Alex".to_string(), "known".to_string()))
    );
    resolver.resolve_offline_users(true);
    resolver.save();
}

#[test]
fn gametest_ticker_matches_java_singleton_state_machine_shape() {
    assert_eq!(GAME_TEST_TICKER_JAVA.lines().count(), 62);
    for sentinel in [
        "public static final GameTestTicker SINGLETON = new GameTestTicker();",
        "private final Collection<GameTestInfo> testInfos = Lists.newCopyOnWriteArrayList();",
        "private @Nullable GameTestRunner runner;",
        "private GameTestTicker.State state = GameTestTicker.State.IDLE;",
        "private GameTestTicker()",
        "this.testInfos.add(testInfo);",
        "if (this.state != GameTestTicker.State.IDLE)",
        "this.state = GameTestTicker.State.HALTING;",
        "this.testInfos.clear();",
        "this.runner.stop();",
        "this.runner = null;",
        "Util.logAndPauseIfInIde(\"The runner was already set in GameTestTicker\");",
        "this.state = GameTestTicker.State.RUNNING;",
        "this.testInfos.forEach(i -> i.tick(this.runner));",
        "this.testInfos.removeIf(GameTestInfo::isDone);",
        "GameTestTicker.State finishingState = this.state;",
        "this.state = GameTestTicker.State.IDLE;",
        "if (finishingState == GameTestTicker.State.HALTING)",
        "IDLE,",
        "RUNNING,",
        "HALTING;",
    ] {
        assert!(
            GAME_TEST_TICKER_JAVA.contains(sentinel),
            "missing GameTestTicker sentinel {sentinel}"
        );
    }
    assert_eq!(GAMETEST_TICKER_SINGLETON, "GameTestTicker.SINGLETON");
}

#[test]
fn gametest_ticker_clear_and_set_runner_match_java() {
    let mut ticker = GameTestTickerModel::default();
    ticker.add(GameTestInfoStateModel::new(
        "minecraft:one",
        true,
        1,
        0,
        RotationModel::None,
        RotationModel::None,
        "noRetries",
    ));
    ticker.set_runner("runner-a");
    ticker.set_runner("runner-b");
    assert_eq!(
        ticker.warnings,
        vec!["The runner was already set in GameTestTicker"]
    );

    ticker.clear();
    assert!(ticker.test_infos.is_empty());
    assert_eq!(ticker.runner, None);
    assert_eq!(ticker.runner_stop_count, 1);

    ticker.state = GameTestTickerStateModel::Running;
    ticker.clear();
    assert_eq!(ticker.state, GameTestTickerStateModel::Halting);
}

#[test]
fn gametest_ticker_ticks_only_when_runner_exists_and_removes_done_tests() {
    let mut ticker = GameTestTickerModel::default();
    let mut done = GameTestInfoStateModel::new(
        "minecraft:done",
        true,
        1,
        0,
        RotationModel::None,
        RotationModel::None,
        "noRetries",
    );
    done.succeed();
    ticker.add(done);
    assert!(ticker.tick().is_empty());
    assert_eq!(ticker.test_infos.len(), 1);

    ticker.set_runner("runner");
    let outcomes = ticker.tick();
    assert_eq!(outcomes, vec![GameTestInfoTickOutcome::Passed]);
    assert!(ticker.test_infos.is_empty());
    assert_eq!(ticker.state, GameTestTickerStateModel::Idle);
}

#[test]
fn gametest_ticker_halting_during_tick_recursively_clears_when_idle() {
    let mut ticker = GameTestTickerModel::default();
    ticker.add(GameTestInfoStateModel::new(
        "minecraft:one",
        true,
        1,
        0,
        RotationModel::None,
        RotationModel::None,
        "noRetries",
    ));
    ticker.set_runner("runner");

    let outcomes = ticker.tick_with_clear_after(Some(0));

    assert_eq!(outcomes, vec![GameTestInfoTickOutcome::Started]);
    assert!(ticker.test_infos.is_empty());
    assert_eq!(ticker.runner, None);
    assert_eq!(ticker.runner_stop_count, 1);
    assert_eq!(ticker.state, GameTestTickerStateModel::Idle);
}

#[test]
fn structure_grid_spawner_matches_java_source_shape() {
    assert_eq!(STRUCTURE_GRID_SPAWNER_JAVA.lines().count(), 71);
    for sentinel in [
        "private static final int SPACE_BETWEEN_COLUMNS = 5;",
        "private static final int SPACE_BETWEEN_ROWS = 6;",
        "private final int testsPerRow;",
        "private int currentRowCount;",
        "private AABB rowBounds;",
        "private final BlockPos.MutableBlockPos nextTestNorthWestCorner;",
        "private final BlockPos firstTestNorthWestCorner;",
        "private final boolean clearOnBatch;",
        "private float maxX = -1.0F;",
        "private final Collection<GameTestInfo> testInLastBatch = new ArrayList<>();",
        "this.nextTestNorthWestCorner = firstTestNorthWestCorner.mutable();",
        "this.rowBounds = new AABB(this.nextTestNorthWestCorner);",
        "StructureUtils.clearSpaceForStructure(boundingBox, level);",
        "this.testInLastBatch.clear();",
        "this.nextTestNorthWestCorner.set(this.firstTestNorthWestCorner);",
        "testInfo.setTestBlockPos(northWestCorner);",
        "GameTestInfo infoWithStructure = testInfo.prepareTestStructure();",
        "return Optional.empty();",
        "infoWithStructure.startExecution(1);",
        "this.nextTestNorthWestCorner.move((int)structureBounds.getXsize() + 5, 0, 0);",
        "this.nextTestNorthWestCorner.move(0, 0, (int)this.rowBounds.getZsize() + 6);",
        "this.nextTestNorthWestCorner.setX(this.firstTestNorthWestCorner.getX());",
        "this.testInLastBatch.add(testInfo);",
        "return Optional.of(testInfo);",
    ] {
        assert!(
            STRUCTURE_GRID_SPAWNER_JAVA.contains(sentinel),
            "missing StructureGridSpawner sentinel {sentinel}"
        );
    }
}

#[test]
fn structure_grid_spawner_places_tests_in_rows_and_tracks_bounds_like_java() {
    let first = BlockPosModel::new(10, 64, 20);
    let mut spawner = StructureGridSpawnerModel::new(first, 2, false);
    let mut first_test = StructureGridSpawnerTestModel::new("minecraft:first", 4, 3);
    let mut second_test = StructureGridSpawnerTestModel::new("minecraft:second", 6, 5);
    let mut third_test = StructureGridSpawnerTestModel::new("minecraft:third", 2, 2);

    assert_eq!(
        spawner.spawn_structure(&mut first_test),
        Some("minecraft:first".to_string())
    );
    assert_eq!(first_test.test_block_pos, Some(first));
    assert_eq!(first_test.start_delay, Some(1));
    assert_eq!(
        spawner.next_test_north_west_corner,
        BlockPosModel::new(19, 64, 20)
    );
    assert_eq!(spawner.max_x, 19.0);
    assert_eq!(spawner.current_row_count, 1);

    assert_eq!(
        spawner.spawn_structure(&mut second_test),
        Some("minecraft:second".to_string())
    );
    assert_eq!(
        second_test.test_block_pos,
        Some(BlockPosModel::new(19, 64, 20))
    );
    assert_eq!(spawner.current_row_count, 0);
    assert_eq!(
        spawner.next_test_north_west_corner,
        BlockPosModel::new(10, 64, 31)
    );
    assert_eq!(
        spawner.row_bounds,
        StructureGridBoundsModel::point(BlockPosModel::new(10, 64, 31))
    );

    assert_eq!(
        spawner.spawn_structure(&mut third_test),
        Some("minecraft:third".to_string())
    );
    assert_eq!(
        third_test.test_block_pos,
        Some(BlockPosModel::new(10, 64, 31))
    );
    assert_eq!(spawner.tests_in_last_batch.len(), 3);
}

#[test]
fn structure_grid_spawner_unprepared_tests_do_not_advance_or_start() {
    let first = BlockPosModel::new(0, 70, 0);
    let mut spawner = StructureGridSpawnerModel::new(first, 1, false);
    let mut unprepared = StructureGridSpawnerTestModel::new("minecraft:missing", 4, 4).unprepared();

    assert_eq!(spawner.spawn_structure(&mut unprepared), None);
    assert_eq!(unprepared.test_block_pos, Some(first));
    assert_eq!(unprepared.start_delay, None);
    assert_eq!(spawner.next_test_north_west_corner, first);
    assert_eq!(spawner.current_row_count, 0);
    assert!(spawner.tests_in_last_batch.is_empty());
}

#[test]
fn structure_grid_spawner_clear_on_batch_clears_previous_batch_and_resets_cursor() {
    let first = BlockPosModel::new(5, 80, 5);
    let mut spawner = StructureGridSpawnerModel::new(first, 3, true);
    let mut test = StructureGridSpawnerTestModel::new("minecraft:first", 4, 4);
    spawner.spawn_structure(&mut test);

    assert_eq!(spawner.tests_in_last_batch.len(), 1);
    assert_ne!(spawner.next_test_north_west_corner, first);

    spawner.on_batch_start();

    assert_eq!(spawner.cleared_bounds, vec![test.test_bounding_box]);
    assert!(spawner.tests_in_last_batch.is_empty());
    assert_eq!(spawner.next_test_north_west_corner, first);
    assert_eq!(spawner.row_bounds, StructureGridBoundsModel::point(first));
}
