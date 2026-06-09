use super::*;

const GAMETEST_MAIN_JAVA: &str =
    include_str!("../../../decompiled-server-26.1.2/net/minecraft/gametest/Main.java");
const BLOCK_BASED_TEST_INSTANCE_JAVA: &str = include_str!(
        "../../../decompiled-server-26.1.2/net/minecraft/gametest/framework/BlockBasedTestInstance.java"
    );
const BUILTIN_TEST_FUNCTIONS_JAVA: &str = include_str!(
    "../../../decompiled-server-26.1.2/net/minecraft/gametest/framework/BuiltinTestFunctions.java"
);
const EXHAUSTED_ATTEMPTS_EXCEPTION_JAVA: &str = include_str!(
        "../../../decompiled-server-26.1.2/net/minecraft/gametest/framework/ExhaustedAttemptsException.java"
    );
const FAILED_TEST_TRACKER_JAVA: &str = include_str!(
    "../../../decompiled-server-26.1.2/net/minecraft/gametest/framework/FailedTestTracker.java"
);
const FUNCTION_GAME_TEST_INSTANCE_JAVA: &str = include_str!(
        "../../../decompiled-server-26.1.2/net/minecraft/gametest/framework/FunctionGameTestInstance.java"
    );
const GAME_TEST_ASSERT_EXCEPTION_JAVA: &str = include_str!(
        "../../../decompiled-server-26.1.2/net/minecraft/gametest/framework/GameTestAssertException.java"
    );
const GAME_TEST_ASSERT_POS_EXCEPTION_JAVA: &str = include_str!(
        "../../../decompiled-server-26.1.2/net/minecraft/gametest/framework/GameTestAssertPosException.java"
    );
const GAME_TEST_BATCH_JAVA: &str = include_str!(
    "../../../decompiled-server-26.1.2/net/minecraft/gametest/framework/GameTestBatch.java"
);
const GAME_TEST_BATCH_FACTORY_JAVA: &str = include_str!(
    "../../../decompiled-server-26.1.2/net/minecraft/gametest/framework/GameTestBatchFactory.java"
);
const GAME_TEST_BATCH_LISTENER_JAVA: &str = include_str!(
    "../../../decompiled-server-26.1.2/net/minecraft/gametest/framework/GameTestBatchListener.java"
);
const GAME_TEST_ENVIRONMENTS_JAVA: &str = include_str!(
    "../../../decompiled-server-26.1.2/net/minecraft/gametest/framework/GameTestEnvironments.java"
);
const GAME_TEST_EVENT_JAVA: &str = include_str!(
    "../../../decompiled-server-26.1.2/net/minecraft/gametest/framework/GameTestEvent.java"
);

#[test]
fn gametest_main_entrypoint_matches_java_launcher_contract() {
    assert_eq!(GAMETEST_MAIN_JAVA.lines().count(), 11);
    assert_eq!(
        GAMETEST_MAIN_JAVA
            .match_indices("SharedConstants.tryDetectVersion();")
            .count(),
        1
    );
    assert_eq!(
        GAMETEST_MAIN_JAVA
            .match_indices("GameTestMainUtil.runGameTestServer(args, path -> {});")
            .count(),
        1
    );
    assert!(GAMETEST_MAIN_JAVA
        .contains("public static void main(final String[] args) throws Exception"));
    assert_eq!(
        gametest_main_entrypoint_contract(),
        GameTestMainEntrypoint {
            detects_version: true,
            forwards_args_to_server: true,
            output_path_callback_writes: false,
        }
    );
}

#[test]
fn block_based_test_instance_matches_java_source_shape() {
    assert_eq!(BLOCK_BASED_TEST_INSTANCE_JAVA.lines().count(), 91);
    assert_eq!(
        BLOCK_BASED_TEST_INSTANCE_JAVA
            .match_indices("RecordCodecBuilder.mapCodec")
            .count(),
        1
    );
    assert_eq!(
        BLOCK_BASED_TEST_INSTANCE_JAVA
            .match_indices("TestData.CODEC.forGetter(GameTestInstance::info)")
            .count(),
        1
    );
    assert_eq!(
        BLOCK_BASED_TEST_INSTANCE_JAVA
            .match_indices("helper.onEachTick")
            .count(),
        1
    );
    for sentinel in [
        "blockEntity.trigger();",
        "test_block.error.missing",
        "test_block.error.too_many",
        "TestBlockMode.ACCEPT",
        "TestBlockMode.FAIL",
        "TestBlockMode.LOG",
        "blockEntity.reset();",
        "Component.translatable(\"test_instance.type.block_based\")",
    ] {
        assert!(
            BLOCK_BASED_TEST_INSTANCE_JAVA.contains(sentinel),
            "missing BlockBasedTestInstance sentinel {sentinel}"
        );
    }
}

#[test]
fn block_based_test_instance_decodes_block_based_resources() {
    let raw = r#"{
            "type": "minecraft:block_based",
            "environment": "minecraft:default",
            "structure": "minecraft:test_block_based",
            "max_ticks": 200,
            "setup_ticks": 5,
            "required": true
        }"#;
    assert_eq!(
        parse_test_instance_json(raw).expect("block_based test instance"),
        GameTestInstanceDefinition {
            kind: GameTestInstanceKind::BlockBased,
            environment: "minecraft:default".to_string(),
            structure: "minecraft:test_block_based".to_string(),
            max_ticks: 200,
            setup_ticks: 5,
            required: true,
        }
    );
}

#[test]
fn block_based_test_instance_start_block_rules_match_java() {
    assert_eq!(
        block_based_test_start_outcome(&[]),
        BlockBasedTestRunOutcome::MissingStart
    );
    assert_eq!(
        block_based_test_start_outcome(&[BlockBasedTestBlockState {
            mode: TestBlockMode::Start,
            triggered: false,
            message: String::new(),
        }]),
        BlockBasedTestRunOutcome::StartTriggered
    );
    assert_eq!(
        block_based_test_start_outcome(&[
            BlockBasedTestBlockState {
                mode: TestBlockMode::Start,
                triggered: false,
                message: String::new(),
            },
            BlockBasedTestBlockState {
                mode: TestBlockMode::Start,
                triggered: false,
                message: String::new(),
            },
        ]),
        BlockBasedTestRunOutcome::TooManyStarts
    );
}

#[test]
fn block_based_test_instance_tick_rules_match_java() {
    assert_eq!(
        block_based_test_tick_outcome(&[]),
        BlockBasedTestRunOutcome::MissingAccept
    );
    assert_eq!(
        block_based_test_tick_outcome(&[BlockBasedTestBlockState {
            mode: TestBlockMode::Accept,
            triggered: true,
            message: String::new(),
        }]),
        BlockBasedTestRunOutcome::Succeeded
    );
    assert_eq!(
        block_based_test_tick_outcome(&[
            BlockBasedTestBlockState {
                mode: TestBlockMode::Accept,
                triggered: false,
                message: String::new(),
            },
            BlockBasedTestBlockState {
                mode: TestBlockMode::Fail,
                triggered: true,
                message: "boom".to_string(),
            },
        ]),
        BlockBasedTestRunOutcome::Failed {
            message: "boom".to_string(),
        }
    );
    assert_eq!(
        block_based_test_tick_outcome(&[
            BlockBasedTestBlockState {
                mode: TestBlockMode::Accept,
                triggered: false,
                message: String::new(),
            },
            BlockBasedTestBlockState {
                mode: TestBlockMode::Log,
                triggered: true,
                message: "note".to_string(),
            },
        ]),
        BlockBasedTestRunOutcome::Continue { log_reset_count: 1 }
    );
}

#[test]
fn builtin_test_functions_match_java_source_shape() {
    assert_eq!(BUILTIN_TEST_FUNCTIONS_JAVA.lines().count(), 28);
    assert_eq!(
        BUILTIN_TEST_FUNCTIONS_JAVA
            .match_indices("ALWAYS_PASS")
            .count(),
        5
    );
    assert_eq!(
        BUILTIN_TEST_FUNCTIONS_JAVA
            .match_indices("Identifier.withDefaultNamespace(name)")
            .count(),
        1
    );
    assert_eq!(
        BUILTIN_TEST_FUNCTIONS_JAVA
            .match_indices("registerLoader(new BuiltinTestFunctions())")
            .count(),
        1
    );
    for sentinel in [
            "public static final ResourceKey<Consumer<GameTestHelper>> ALWAYS_PASS = create(\"always_pass\");",
            "public static final Consumer<GameTestHelper> ALWAYS_PASS_INSTANCE = GameTestHelper::succeed;",
            "runLoaders(registry);",
            "return ALWAYS_PASS_INSTANCE;",
            "register.accept(ALWAYS_PASS, ALWAYS_PASS_INSTANCE);",
        ] {
            assert!(
                BUILTIN_TEST_FUNCTIONS_JAVA.contains(sentinel),
                "missing BuiltinTestFunctions sentinel {sentinel}"
            );
        }
}

#[test]
fn builtin_test_functions_register_always_pass_only() {
    assert_eq!(BUILTIN_ALWAYS_PASS_FUNCTION_ID, "minecraft:always_pass");
    assert_eq!(
        builtin_gametest_function("minecraft:always_pass"),
        Some(BuiltinGameTestFunctionAction::Succeed)
    );
    assert_eq!(
        builtin_gametest_function("always_pass"),
        Some(BuiltinGameTestFunctionAction::Succeed)
    );
    assert_eq!(builtin_gametest_function("minecraft:unknown"), None);
    assert_eq!(
        builtin_gametest_bootstrap_return(),
        BuiltinGameTestFunctionAction::Succeed
    );
}

#[test]
fn exhausted_attempts_exception_matches_java_source_shape() {
    assert_eq!(EXHAUSTED_ATTEMPTS_EXCEPTION_JAVA.lines().count(), 18);
    assert_eq!(
        EXHAUSTED_ATTEMPTS_EXCEPTION_JAVA
            .match_indices("class ExhaustedAttemptsException extends Throwable")
            .count(),
        1
    );
    assert_eq!(
        EXHAUSTED_ATTEMPTS_EXCEPTION_JAVA
            .match_indices("testInfo.requiredSuccesses()")
            .count(),
        1
    );
    assert_eq!(
        EXHAUSTED_ATTEMPTS_EXCEPTION_JAVA
            .match_indices("testInfo.maxAttempts()")
            .count(),
        1
    );
    assert_eq!(
        EXHAUSTED_ATTEMPTS_EXCEPTION_JAVA
            .match_indices("testInfo.getError()")
            .count(),
        1
    );
}

#[test]
fn exhausted_attempts_exception_message_and_cause_match_java() {
    assert_eq!(
        exhausted_attempts_error(5, 2, 3, 7, Some("last failure".to_string())),
        ExhaustedAttemptsError {
            message:
                "Not enough successes: 2 out of 5 attempts. Required successes: 3. max attempts: 7."
                    .to_string(),
            cause: Some("last failure".to_string()),
        }
    );
    assert_eq!(
        exhausted_attempts_error(1, 0, 1, 1, None),
        ExhaustedAttemptsError {
            message:
                "Not enough successes: 0 out of 1 attempts. Required successes: 1. max attempts: 1."
                    .to_string(),
            cause: None,
        }
    );
}

#[test]
fn failed_test_tracker_matches_java_source_shape() {
    assert_eq!(FAILED_TEST_TRACKER_JAVA.lines().count(), 22);
    assert_eq!(
            FAILED_TEST_TRACKER_JAVA
                .match_indices("private static final Set<Holder.Reference<GameTestInstance>> LAST_FAILED_TESTS = Sets.newHashSet();")
                .count(),
            1
        );
    for sentinel in [
        "public static Stream<Holder.Reference<GameTestInstance>> getLastFailedTests()",
        "return LAST_FAILED_TESTS.stream();",
        "public static void rememberFailedTest(final Holder.Reference<GameTestInstance> test)",
        "LAST_FAILED_TESTS.add(test);",
        "public static void forgetFailedTests()",
        "LAST_FAILED_TESTS.clear();",
    ] {
        assert!(
            FAILED_TEST_TRACKER_JAVA.contains(sentinel),
            "missing FailedTestTracker sentinel {sentinel}"
        );
    }
}

#[test]
fn failed_test_tracker_remembers_uniquely_and_forgets_like_java_set() {
    let mut tracker = FailedTestTrackerModel::default();
    assert!(tracker.last_failed_tests().is_empty());

    tracker.remember_failed_test("minecraft:always_pass");
    tracker.remember_failed_test("minecraft:block_based");
    tracker.remember_failed_test("minecraft:always_pass");
    assert_eq!(
        tracker.last_failed_tests(),
        vec!["minecraft:always_pass", "minecraft:block_based"]
    );

    tracker.forget_failed_tests();
    assert!(tracker.last_failed_tests().is_empty());
}

#[test]
fn function_gametest_instance_matches_java_source_shape() {
    assert_eq!(FUNCTION_GAME_TEST_INSTANCE_JAVA.lines().count(), 57);
    assert_eq!(
        FUNCTION_GAME_TEST_INSTANCE_JAVA
            .match_indices("ResourceKey.codec(Registries.TEST_FUNCTION).fieldOf(\"function\")")
            .count(),
        1
    );
    assert_eq!(
        FUNCTION_GAME_TEST_INSTANCE_JAVA
            .match_indices("TestData.CODEC.forGetter(GameTestInstance::info)")
            .count(),
        1
    );
    for sentinel in [
            "orElseThrow(() -> new IllegalStateException(\"Trying to access missing test function: \" + this.function.identifier()))",
            ".accept(helper);",
            "return this.function;",
            "Component.translatable(\"test_instance.type.function\")",
            "this.descriptionRow(\"test_instance.description.function\", this.function.identifier().toString())",
        ] {
            assert!(
                FUNCTION_GAME_TEST_INSTANCE_JAVA.contains(sentinel),
                "missing FunctionGameTestInstance sentinel {sentinel}"
            );
        }
}

#[test]
fn function_gametest_instance_run_and_description_match_java() {
    assert_eq!(
        function_gametest_run_outcome(
            BUILTIN_ALWAYS_PASS_FUNCTION_ID,
            &[BUILTIN_ALWAYS_PASS_FUNCTION_ID]
        ),
        FunctionGameTestRunOutcome::Invoked {
            function: BUILTIN_ALWAYS_PASS_FUNCTION_ID.to_string(),
        }
    );
    assert_eq!(
        function_gametest_run_outcome("minecraft:missing", &[BUILTIN_ALWAYS_PASS_FUNCTION_ID]),
        FunctionGameTestRunOutcome::MissingFunction {
            message: "Trying to access missing test function: minecraft:missing".to_string(),
        }
    );
    assert_eq!(
        function_gametest_description_rows(BUILTIN_ALWAYS_PASS_FUNCTION_ID),
        [(
            "test_instance.description.function",
            BUILTIN_ALWAYS_PASS_FUNCTION_ID.to_string()
        )]
    );
}

#[test]
fn gametest_assert_exception_matches_java_source_shape() {
    assert_eq!(GAME_TEST_ASSERT_EXCEPTION_JAVA.lines().count(), 24);
    assert_eq!(
        GAME_TEST_ASSERT_EXCEPTION_JAVA
            .match_indices("extends GameTestException")
            .count(),
        1
    );
    for sentinel in [
        "protected final Component message;",
        "protected final int tick;",
        "super(message.getString());",
        "Component.translatable(\"test.error.tick\", this.message, this.tick)",
        "return this.getDescription().getString();",
    ] {
        assert!(
            GAME_TEST_ASSERT_EXCEPTION_JAVA.contains(sentinel),
            "missing GameTestAssertException sentinel {sentinel}"
        );
    }
}

#[test]
fn gametest_assert_exception_runtime_message_and_description_match_java() {
    let error = GameTestAssertError::new("expected block", 42);
    assert_eq!(error.runtime_message(), "expected block");
    assert_eq!(
        error.description(),
        GameTestAssertDescription {
            translation_key: "test.error.tick",
            message: "expected block".to_string(),
            tick: 42,
        }
    );
}

#[test]
fn gametest_assert_pos_exception_matches_java_source_shape() {
    assert_eq!(GAME_TEST_ASSERT_POS_EXCEPTION_JAVA.lines().count(), 43);
    assert_eq!(
        GAME_TEST_ASSERT_POS_EXCEPTION_JAVA
            .match_indices("extends GameTestAssertException")
            .count(),
        1
    );
    for sentinel in [
        "private final BlockPos absolutePos;",
        "private final BlockPos relativePos;",
        "super(baseMessage, tick);",
        "Component.translatable(",
        "\"test.error.position\"",
        "this.absolutePos.getX()",
        "this.relativePos.getZ()",
        "public Component getMessageToShowAtBlock()",
        "public @Nullable BlockPos getRelativePos()",
        "public @Nullable BlockPos getAbsolutePos()",
    ] {
        assert!(
            GAME_TEST_ASSERT_POS_EXCEPTION_JAVA.contains(sentinel),
            "missing GameTestAssertPosException sentinel {sentinel}"
        );
    }
}

#[test]
fn gametest_assert_pos_exception_description_and_accessors_match_java() {
    let absolute_pos = GameTestBlockPos {
        x: 10,
        y: 64,
        z: -3,
    };
    let relative_pos = GameTestBlockPos { x: 1, y: 2, z: 3 };
    let error = GameTestAssertPosError::new("wrong block", absolute_pos, relative_pos, 99);
    assert_eq!(error.message_to_show_at_block(), "wrong block");
    assert_eq!(error.absolute_pos, absolute_pos);
    assert_eq!(error.relative_pos, relative_pos);
    assert_eq!(
        error.description(),
        GameTestAssertPosDescription {
            translation_key: "test.error.position",
            message: "wrong block".to_string(),
            absolute_pos,
            relative_pos,
            tick: 99,
        }
    );
}

#[test]
fn gametest_batch_matches_java_record_shape() {
    assert_eq!(GAME_TEST_BATCH_JAVA.lines().count(), 12);
    for sentinel in [
            "public record GameTestBatch(int index, Collection<GameTestInfo> gameTestInfos, Holder<TestEnvironmentDefinition<?>> environment)",
            "if (gameTestInfos.isEmpty())",
            "throw new IllegalArgumentException(\"A GameTestBatch must include at least one GameTestInfo!\");",
        ] {
            assert!(
                GAME_TEST_BATCH_JAVA.contains(sentinel),
                "missing GameTestBatch sentinel {sentinel}"
            );
        }
}

#[test]
fn gametest_batch_rejects_empty_and_preserves_record_fields() {
    assert_eq!(
        create_gametest_batch(0, Vec::new(), "minecraft:default"),
        Err("A GameTestBatch must include at least one GameTestInfo!".to_string())
    );
    assert_eq!(
        create_gametest_batch(
            2,
            vec!["minecraft:always_pass".to_string()],
            "minecraft:default"
        ),
        Ok(GameTestBatchModel {
            index: 2,
            game_test_infos: vec!["minecraft:always_pass".to_string()],
            environment: "minecraft:default".to_string(),
        })
    );
}

#[test]
fn gametest_batch_factory_matches_java_source_shape() {
    assert_eq!(GAME_TEST_BATCH_FACTORY_JAVA.lines().count(), 66);
    for sentinel in [
            "private static final int MAX_TESTS_PER_BATCH = 50;",
            "public static final GameTestBatchFactory.TestDecorator DIRECT",
            "new GameTestInfo(test, Rotation.NONE, level, RetryOptions.noRetries())",
            ".collect(Collectors.groupingBy(info -> info.getTest().batch()))",
            "Lists.partition(testsInBatch, 50)",
            ".filter(Objects::nonNull)",
            "Lists.partition(testsInBatch, maxTestsPerBatch)",
            "return new GameTestBatch(counter, tests, batch);",
            "Stream<GameTestInfo> decorate(Holder.Reference<GameTestInstance> test, ServerLevel level);",
        ] {
            assert!(
                GAME_TEST_BATCH_FACTORY_JAVA.contains(sentinel),
                "missing GameTestBatchFactory sentinel {sentinel}"
            );
        }
}

#[test]
fn gametest_batch_factory_direct_decorator_matches_java_defaults() {
    let decorated = direct_gametest_decorator("minecraft:always_pass", "minecraft:default");
    assert_eq!(
        decorated,
        DecoratedGameTestInfoModel {
            info: GameTestInfoModel {
                test: "minecraft:always_pass".to_string(),
                environment: "minecraft:default".to_string(),
            },
            rotation: "NONE",
            retry_options: "noRetries",
        }
    );
    assert_eq!(
        divide_decorated_gametests_into_batches(&[decorated]).unwrap(),
        vec![GameTestBatchModel {
            index: 0,
            game_test_infos: vec!["minecraft:always_pass".to_string()],
            environment: "minecraft:default".to_string(),
        }]
    );
}

#[test]
fn gametest_batch_factory_filters_nulls_groups_and_partitions() {
    let mut infos = (0..51)
        .map(|index| {
            Some(GameTestInfoModel {
                test: format!("minecraft:default_{index}"),
                environment: "minecraft:default".to_string(),
            })
        })
        .collect::<Vec<_>>();
    infos.push(None);
    infos.push(Some(GameTestInfoModel {
        test: "minecraft:rainy".to_string(),
        environment: "minecraft:rain".to_string(),
    }));

    let batches = gametest_batches_from_infos(&infos, MAX_GAMETESTS_PER_BATCH).unwrap();
    assert_eq!(batches.len(), 3);
    assert_eq!(batches[0].environment, "minecraft:default");
    assert_eq!(batches[0].index, 0);
    assert_eq!(batches[0].game_test_infos.len(), 50);
    assert_eq!(batches[1].environment, "minecraft:default");
    assert_eq!(batches[1].index, 1);
    assert_eq!(batches[1].game_test_infos, vec!["minecraft:default_50"]);
    assert_eq!(
        batches[2],
        GameTestBatchModel {
            index: 0,
            game_test_infos: vec!["minecraft:rainy".to_string()],
            environment: "minecraft:rain".to_string(),
        }
    );
}

#[test]
fn gametest_batch_factory_rejects_zero_partition_size() {
    assert_eq!(
        gametest_batches_from_infos(
            &[Some(GameTestInfoModel {
                test: "minecraft:always_pass".to_string(),
                environment: "minecraft:default".to_string(),
            })],
            0
        ),
        Err("partition size must be positive".to_string())
    );
}

#[test]
fn gametest_batch_listener_matches_java_interface_shape() {
    assert_eq!(GAME_TEST_BATCH_LISTENER_JAVA.lines().count(), 7);
    for sentinel in [
        "public interface GameTestBatchListener",
        "void testBatchStarting(final GameTestBatch batch);",
        "void testBatchFinished(final GameTestBatch batch);",
    ] {
        assert!(
            GAME_TEST_BATCH_LISTENER_JAVA.contains(sentinel),
            "missing GameTestBatchListener sentinel {sentinel}"
        );
    }
}

#[test]
fn gametest_batch_listener_records_start_and_finish_callbacks() {
    let batch = create_gametest_batch(
        0,
        vec!["minecraft:always_pass".to_string()],
        "minecraft:default",
    )
    .unwrap();
    let mut listener = RecordingGameTestBatchListener::default();

    listener.test_batch_starting(&batch);
    listener.test_batch_finished(&batch);

    assert_eq!(
        listener.events,
        vec![
            GameTestBatchListenerEvent::Starting(batch.clone()),
            GameTestBatchListenerEvent::Finished(batch),
        ]
    );
}

#[test]
fn gametest_environments_matches_java_bootstrap_shape() {
    assert_eq!(GAME_TEST_ENVIRONMENTS_JAVA.lines().count(), 20);
    for sentinel in [
        "String DEFAULT = \"default\";",
        "ResourceKey<TestEnvironmentDefinition<?>> DEFAULT_KEY = create(\"default\");",
        "ResourceKey.create(Registries.TEST_ENVIRONMENT, Identifier.withDefaultNamespace(name))",
        "context.register(DEFAULT_KEY, new TestEnvironmentDefinition.AllOf(List.of()));",
    ] {
        assert!(
            GAME_TEST_ENVIRONMENTS_JAVA.contains(sentinel),
            "missing GameTestEnvironments sentinel {sentinel}"
        );
    }
}

#[test]
fn gametest_environments_bootstraps_default_all_of_empty() {
    assert_eq!(DEFAULT_GAMETEST_ENVIRONMENT_NAME, "default");
    assert_eq!(DEFAULT_GAMETEST_ENVIRONMENT_KEY, "minecraft:default");
    assert_eq!(
        bootstrap_gametest_environments(),
        vec![(
            "minecraft:default".to_string(),
            TestEnvironmentDefinition::AllOf {
                definitions: Vec::new(),
            }
        )]
    );
}

#[test]
fn gametest_event_matches_java_factory_shape() {
    assert_eq!(GAME_TEST_EVENT_JAVA.lines().count(), 27);
    for sentinel in [
        "public final @Nullable Long expectedDelay;",
        "public final @Nullable Long minimumDelay;",
        "public final Runnable assertion;",
        "private GameTestEvent(final @Nullable Long expectedDelay, final @Nullable Long minimumDelay, final Runnable assertion)",
        "return new GameTestEvent(null, null, runnable);",
        "return new GameTestEvent(expectedTick, null, runnable);",
        "return new GameTestEvent(null, minimumDelay, runnable);",
    ] {
        assert!(
            GAME_TEST_EVENT_JAVA.contains(sentinel),
            "missing GameTestEvent sentinel {sentinel}"
        );
    }
}

#[test]
fn gametest_event_factories_set_only_their_delay_field() {
    assert_eq!(
        create_gametest_event("assert-now"),
        GameTestEventModel {
            expected_delay: None,
            minimum_delay: None,
            assertion: "assert-now".to_string(),
        }
    );
    assert_eq!(
        create_gametest_event_at(12, "assert-at"),
        GameTestEventModel {
            expected_delay: Some(12),
            minimum_delay: None,
            assertion: "assert-at".to_string(),
        }
    );
    assert_eq!(
        create_gametest_event_with_minimum_delay(5, "assert-after"),
        GameTestEventModel {
            expected_delay: None,
            minimum_delay: Some(5),
            assertion: "assert-after".to_string(),
        }
    );
}

#[test]
fn gametest_environment_and_instance_decode_vanilla_resources() {
    let environment = parse_test_environment_json(include_str!(
        "../../../decompiled-server-26.1.2/data/minecraft/test_environment/default.json"
    ))
    .expect("vanilla default test environment should decode");
    assert_eq!(
        environment,
        TestEnvironmentDefinition::AllOf {
            definitions: Vec::new()
        }
    );

    let instance = parse_test_instance_json(include_str!(
        "../../../decompiled-server-26.1.2/data/minecraft/test_instance/always_pass.json"
    ))
    .expect("vanilla always_pass test instance should decode");
    assert_eq!(
        instance,
        GameTestInstanceDefinition {
            kind: GameTestInstanceKind::Function {
                function: BUILTIN_ALWAYS_PASS_FUNCTION_ID.to_string()
            },
            environment: "minecraft:default".to_string(),
            structure: "minecraft:empty".to_string(),
            max_ticks: 1,
            setup_ticks: 1,
            required: false,
        }
    );
}

#[test]
fn gametest_resource_decoders_reject_unknown_types() {
    assert!(
        parse_test_environment_json(r#"{"type":"minecraft:unknown"}"#)
            .unwrap_err()
            .contains("unsupported test environment type")
    );
    assert!(parse_test_instance_json(r#"{"type":"minecraft:unknown"}"#)
        .unwrap_err()
        .contains("unsupported test instance type"));
}
