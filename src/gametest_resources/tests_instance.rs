use super::*;
use crate::core_block_pos::RotationModel;

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

    let vanilla = parse_test_instance_json(include_str!(
        "../../../decompiled-server-26.1.2/data/minecraft/test_instance/always_pass.json"
    ))
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
