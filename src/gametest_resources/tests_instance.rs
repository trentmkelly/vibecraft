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
