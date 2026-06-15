use super::*;
use std::collections::BTreeMap;

const GENERATED_TEST_JAVA: &str = vibecraft_java_source!("/net/minecraft/gametest/framework/GeneratedTest.java");
const REGISTRIES_JAVA: &str =
    vibecraft_java_source!("/net/minecraft/core/registries/Registries.java");
const TEST_DATA_JAVA: &str = vibecraft_java_source!("/net/minecraft/gametest/framework/TestData.java");
const TEST_FUNCTION_LOADER_JAVA: &str = vibecraft_java_source!("/net/minecraft/gametest/framework/TestFunctionLoader.java");

#[test]
fn generated_test_matches_java_record_and_constructor_shape() {
    assert_eq!(GENERATED_TEST_JAVA.lines().count(), 23);
    assert!(REGISTRIES_JAVA.contains(
        "public static final ResourceKey<Registry<Consumer<GameTestHelper>>> TEST_FUNCTION = createRegistryKey(\"test_function\");"
    ));
    for sentinel in [
        "public record GeneratedTest(",
        "Map<Identifier, TestData<ResourceKey<TestEnvironmentDefinition<?>>>> tests,",
        "ResourceKey<Consumer<GameTestHelper>> functionKey,",
        "Consumer<GameTestHelper> function",
        "ResourceKey.create(Registries.TEST_FUNCTION, functionId)",
        "this(Map.of(id, testData), id, function);",
    ] {
        assert!(
            GENERATED_TEST_JAVA.contains(sentinel),
            "missing GeneratedTest sentinel {sentinel}"
        );
    }
}

#[test]
fn generated_test_constructors_match_java_resource_key_and_singleton_map_behavior() {
    let mut tests = BTreeMap::new();
    tests.insert("minecraft:first".to_string(), "data-a".to_string());
    tests.insert("minecraft:second".to_string(), "data-b".to_string());

    let generated = GeneratedTestModel::from_function_id(tests.clone(), "minecraft:runner", "run");

    assert_eq!(generated.tests, tests);
    assert_eq!(
        generated.function_key,
        GeneratedTestFunctionKey {
            registry: "test_function",
            id: "minecraft:runner".to_string(),
        }
    );
    assert_eq!(generated.function, "run");

    let singleton = GeneratedTestModel::single("minecraft:single", "data-only".to_string(), "run");
    assert_eq!(
        singleton.tests,
        BTreeMap::from([("minecraft:single".to_string(), "data-only".to_string())])
    );
    assert_eq!(
        singleton.function_key,
        GeneratedTestFunctionKey {
            registry: "test_function",
            id: "minecraft:single".to_string(),
        }
    );
}

#[test]
fn test_data_matches_java_record_codec_and_constructor_shape() {
    assert_eq!(TEST_DATA_JAVA.lines().count(), 67);
    for sentinel in [
        "public record TestData<EnvironmentType>(",
        "EnvironmentType environment,",
        "Identifier structure,",
        "int maxTicks,",
        "int setupTicks,",
        "boolean required,",
        "Rotation rotation,",
        "boolean manualOnly,",
        "int maxAttempts,",
        "int requiredSuccesses,",
        "boolean skyAccess,",
        "int padding",
        "TestEnvironmentDefinition.CODEC.fieldOf(\"environment\").forGetter(TestData::environment)",
        "Identifier.CODEC.fieldOf(\"structure\").forGetter(TestData::structure)",
        "ExtraCodecs.POSITIVE_INT.fieldOf(\"max_ticks\").forGetter(TestData::maxTicks)",
        "ExtraCodecs.NON_NEGATIVE_INT.optionalFieldOf(\"setup_ticks\", 0).forGetter(TestData::setupTicks)",
        "Codec.BOOL.optionalFieldOf(\"required\", true).forGetter(TestData::required)",
        "Rotation.CODEC.optionalFieldOf(\"rotation\", Rotation.NONE).forGetter(TestData::rotation)",
        "Codec.BOOL.optionalFieldOf(\"manual_only\", false).forGetter(TestData::manualOnly)",
        "ExtraCodecs.POSITIVE_INT.optionalFieldOf(\"max_attempts\", 1).forGetter(TestData::maxAttempts)",
        "ExtraCodecs.POSITIVE_INT.optionalFieldOf(\"required_successes\", 1).forGetter(TestData::requiredSuccesses)",
        "Codec.BOOL.optionalFieldOf(\"sky_access\", false).forGetter(TestData::skyAccess)",
        "ExtraCodecs.intRange(0, 128).optionalFieldOf(\"padding\", 0).forGetter(TestData::padding)",
        "this(environment, structure, maxTicks, setupTicks, required, rotation, false, 1, 1, false, 0);",
        "this(environment, structure, maxTicks, setupTicks, required, Rotation.NONE);",
        "mapper.apply(this.environment)",
    ] {
        assert!(
            TEST_DATA_JAVA.contains(sentinel),
            "missing TestData sentinel {sentinel}"
        );
    }
}

#[test]
fn test_data_constructors_and_map_preserve_java_defaults_and_fields() {
    let data = GameTestDataModel::new_default_rotation(
        "minecraft:default",
        "minecraft:empty",
        20,
        0,
        true,
    );

    assert_eq!(
        data,
        GameTestDataModel {
            environment: "minecraft:default",
            structure: "minecraft:empty".to_string(),
            max_ticks: 20,
            setup_ticks: 0,
            required: true,
            rotation: RotationModel::None,
            manual_only: false,
            max_attempts: 1,
            required_successes: 1,
            sky_access: false,
            padding: 0,
        }
    );

    let full = GameTestDataModel::new_full(
        "env",
        "minecraft:structure",
        200,
        5,
        false,
        RotationModel::Clockwise90,
        true,
        4,
        2,
        true,
        3,
    );
    let mapped = full.map_environment(|environment| format!("mapped:{environment}"));

    assert_eq!(
        mapped,
        GameTestDataModel {
            environment: "mapped:env".to_string(),
            structure: "minecraft:structure".to_string(),
            max_ticks: 200,
            setup_ticks: 5,
            required: false,
            rotation: RotationModel::Clockwise90,
            manual_only: true,
            max_attempts: 4,
            required_successes: 2,
            sky_access: true,
            padding: 3,
        }
    );
}

#[test]
fn test_function_loader_matches_java_source_shape() {
    assert_eq!(TEST_FUNCTION_LOADER_JAVA.lines().count(), 24);
    for sentinel in [
        "public abstract class TestFunctionLoader",
        "private static final List<TestFunctionLoader> loaders = new ArrayList<>();",
        "public static void registerLoader(final TestFunctionLoader loader)",
        "loaders.add(loader);",
        "public static void runLoaders(final Registry<Consumer<GameTestHelper>> registry)",
        "for (TestFunctionLoader loader : loaders)",
        "loader.load((key, function) -> Registry.register(registry, key, function));",
        "public abstract void load(BiConsumer<ResourceKey<Consumer<GameTestHelper>>, Consumer<GameTestHelper>> register);",
    ] {
        assert!(
            TEST_FUNCTION_LOADER_JAVA.contains(sentinel),
            "missing TestFunctionLoader sentinel {sentinel}"
        );
    }
}

#[test]
fn test_function_loader_registers_and_runs_loaders_in_insertion_order() {
    let mut registry = TestFunctionLoaderRegistryModel::default();
    registry.register_loader(TestFunctionLoaderModel::new(vec![
        ("minecraft:first".to_string(), "succeed".to_string()),
        ("minecraft:second".to_string(), "fail".to_string()),
    ]));
    registry.register_loader(TestFunctionLoaderModel::new(vec![(
        "minecraft:third".to_string(),
        "noop".to_string(),
    )]));

    assert!(registry.registry.is_empty());
    registry.run_loaders();

    assert_eq!(
        registry.registry,
        vec![
            ("minecraft:first".to_string(), "succeed".to_string()),
            ("minecraft:second".to_string(), "fail".to_string()),
            ("minecraft:third".to_string(), "noop".to_string()),
        ]
    );
}
