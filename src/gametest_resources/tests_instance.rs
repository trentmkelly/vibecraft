use super::*;
use crate::core_block_pos::RotationModel;

const GAME_TEST_INSTANCE_JAVA: &str = include_str!(
    "../../../decompiled-server-26.1.2/net/minecraft/gametest/framework/GameTestInstance.java"
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
