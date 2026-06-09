use super::*;
use std::collections::BTreeMap;

const GENERATED_TEST_JAVA: &str = include_str!(
    "../../../decompiled-server-26.1.2/net/minecraft/gametest/framework/GeneratedTest.java"
);
const REGISTRIES_JAVA: &str =
    include_str!("../../../decompiled-server-26.1.2/net/minecraft/core/registries/Registries.java");

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
