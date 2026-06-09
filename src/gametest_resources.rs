#![allow(dead_code)]

use crate::block_entity::TestBlockMode;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TestEnvironmentDefinition {
    AllOf {
        definitions: Vec<String>,
    },
    GameRules,
    ClockTime {
        clock: String,
        time: i32,
    },
    TimelineAttributes {
        timelines: Vec<String>,
    },
    Weather {
        weather: String,
    },
    Function {
        setup: Option<String>,
        teardown: Option<String>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameTestInstanceKind {
    Function { function: String },
    BlockBased,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameTestInstanceDefinition {
    pub kind: GameTestInstanceKind,
    pub environment: String,
    pub structure: String,
    pub max_ticks: i32,
    pub setup_ticks: i32,
    pub required: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GameTestMainEntrypoint {
    pub detects_version: bool,
    pub forwards_args_to_server: bool,
    pub output_path_callback_writes: bool,
}

pub const GAME_TEST_MAIN_ENTRYPOINT: GameTestMainEntrypoint = GameTestMainEntrypoint {
    detects_version: true,
    forwards_args_to_server: true,
    output_path_callback_writes: false,
};

// TODO(gametest-main-server): wire a real Rust GameTest server runner once the
// framework classes and structure-template execution system are implemented.
pub fn gametest_main_entrypoint_contract() -> GameTestMainEntrypoint {
    GAME_TEST_MAIN_ENTRYPOINT
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockBasedTestBlockState {
    pub mode: TestBlockMode,
    pub triggered: bool,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlockBasedTestRunOutcome {
    StartTriggered,
    MissingStart,
    TooManyStarts,
    MissingAccept,
    Succeeded,
    Failed { message: String },
    Continue { log_reset_count: usize },
}

pub fn block_based_test_start_outcome(
    blocks: &[BlockBasedTestBlockState],
) -> BlockBasedTestRunOutcome {
    match blocks
        .iter()
        .filter(|block| block.mode == TestBlockMode::Start)
        .count()
    {
        0 => BlockBasedTestRunOutcome::MissingStart,
        1 => BlockBasedTestRunOutcome::StartTriggered,
        _ => BlockBasedTestRunOutcome::TooManyStarts,
    }
}

// TODO(gametest-block-based-world-scan): replace this pure decision model with
// GameTestHelper-backed structure scanning when the GameTest framework is wired.
pub fn block_based_test_tick_outcome(
    blocks: &[BlockBasedTestBlockState],
) -> BlockBasedTestRunOutcome {
    let accept_blocks = blocks
        .iter()
        .filter(|block| block.mode == TestBlockMode::Accept)
        .collect::<Vec<_>>();
    if accept_blocks.is_empty() {
        return BlockBasedTestRunOutcome::MissingAccept;
    }
    if accept_blocks.iter().any(|block| block.triggered) {
        return BlockBasedTestRunOutcome::Succeeded;
    }
    if let Some(fail) = blocks
        .iter()
        .find(|block| block.mode == TestBlockMode::Fail && block.triggered)
    {
        return BlockBasedTestRunOutcome::Failed {
            message: fail.message.clone(),
        };
    }
    BlockBasedTestRunOutcome::Continue {
        log_reset_count: blocks
            .iter()
            .filter(|block| block.mode == TestBlockMode::Log && block.triggered)
            .count(),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuiltinGameTestFunctionAction {
    Succeed,
}

pub const BUILTIN_ALWAYS_PASS_FUNCTION_ID: &str = "minecraft:always_pass";

pub fn builtin_gametest_function(id: &str) -> Option<BuiltinGameTestFunctionAction> {
    let normalized = id.strip_prefix("minecraft:").unwrap_or(id);
    match normalized {
        "always_pass" => Some(BuiltinGameTestFunctionAction::Succeed),
        _ => None,
    }
}

pub fn builtin_gametest_bootstrap_return() -> BuiltinGameTestFunctionAction {
    BuiltinGameTestFunctionAction::Succeed
}

pub fn parse_test_environment_json(raw: &str) -> Result<TestEnvironmentDefinition, String> {
    let value: serde_json::Value =
        serde_json::from_str(raw).map_err(|err| format!("invalid test environment JSON: {err}"))?;
    let object = value
        .as_object()
        .ok_or_else(|| "test environment must be a JSON object".to_string())?;
    let ty = json_str(object, "type")?
        .strip_prefix("minecraft:")
        .unwrap_or(json_str(object, "type")?);

    match ty {
        "all_of" => Ok(TestEnvironmentDefinition::AllOf {
            definitions: optional_string_array(object, "definitions")?,
        }),
        "game_rules" => {
            object
                .get("rules")
                .ok_or_else(|| "game_rules test environment requires rules".to_string())?;
            Ok(TestEnvironmentDefinition::GameRules)
        }
        "clock_time" => Ok(TestEnvironmentDefinition::ClockTime {
            clock: json_str(object, "clock")?.to_string(),
            time: json_i32(object, "time")?,
        }),
        "timeline_attributes" => Ok(TestEnvironmentDefinition::TimelineAttributes {
            timelines: optional_string_array(object, "timelines")?,
        }),
        "weather" => Ok(TestEnvironmentDefinition::Weather {
            weather: json_str(object, "weather")?.to_string(),
        }),
        "function" => Ok(TestEnvironmentDefinition::Function {
            setup: optional_string(object, "setup")?,
            teardown: optional_string(object, "teardown")?,
        }),
        other => Err(format!(
            "unsupported test environment type minecraft:{other}"
        )),
    }
}

pub fn parse_test_instance_json(raw: &str) -> Result<GameTestInstanceDefinition, String> {
    let value: serde_json::Value =
        serde_json::from_str(raw).map_err(|err| format!("invalid test instance JSON: {err}"))?;
    let object = value
        .as_object()
        .ok_or_else(|| "test instance must be a JSON object".to_string())?;
    let ty = json_str(object, "type")?
        .strip_prefix("minecraft:")
        .unwrap_or(json_str(object, "type")?);
    let kind = match ty {
        "function" => GameTestInstanceKind::Function {
            function: json_str(object, "function")?.to_string(),
        },
        "block_based" => GameTestInstanceKind::BlockBased,
        other => return Err(format!("unsupported test instance type minecraft:{other}")),
    };

    Ok(GameTestInstanceDefinition {
        kind,
        environment: json_str(object, "environment")?.to_string(),
        structure: json_str(object, "structure")?.to_string(),
        max_ticks: json_i32(object, "max_ticks")?,
        setup_ticks: json_i32(object, "setup_ticks")?,
        required: json_bool(object, "required")?,
    })
}

fn json_str<'a>(
    object: &'a serde_json::Map<String, serde_json::Value>,
    field: &str,
) -> Result<&'a str, String> {
    object
        .get(field)
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| format!("missing string field {field}"))
}

fn optional_string(
    object: &serde_json::Map<String, serde_json::Value>,
    field: &str,
) -> Result<Option<String>, String> {
    object
        .get(field)
        .map(|value| {
            value
                .as_str()
                .map(ToString::to_string)
                .ok_or_else(|| format!("field {field} must be a string"))
        })
        .transpose()
}

fn optional_string_array(
    object: &serde_json::Map<String, serde_json::Value>,
    field: &str,
) -> Result<Vec<String>, String> {
    object
        .get(field)
        .map(|value| {
            value
                .as_array()
                .ok_or_else(|| format!("field {field} must be an array"))?
                .iter()
                .map(|entry| {
                    entry
                        .as_str()
                        .map(ToString::to_string)
                        .ok_or_else(|| format!("field {field} entries must be strings"))
                })
                .collect()
        })
        .unwrap_or_else(|| Ok(Vec::new()))
}

fn json_i32(
    object: &serde_json::Map<String, serde_json::Value>,
    field: &str,
) -> Result<i32, String> {
    let value = object
        .get(field)
        .and_then(serde_json::Value::as_i64)
        .ok_or_else(|| format!("missing integer field {field}"))?;
    i32::try_from(value).map_err(|_| format!("field {field} is outside i32 range"))
}

fn json_bool(
    object: &serde_json::Map<String, serde_json::Value>,
    field: &str,
) -> Result<bool, String> {
    object
        .get(field)
        .and_then(serde_json::Value::as_bool)
        .ok_or_else(|| format!("missing boolean field {field}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    const GAMETEST_MAIN_JAVA: &str =
        include_str!("../../decompiled-server-26.1.2/net/minecraft/gametest/Main.java");
    const BLOCK_BASED_TEST_INSTANCE_JAVA: &str = include_str!(
        "../../decompiled-server-26.1.2/net/minecraft/gametest/framework/BlockBasedTestInstance.java"
    );
    const BUILTIN_TEST_FUNCTIONS_JAVA: &str = include_str!(
        "../../decompiled-server-26.1.2/net/minecraft/gametest/framework/BuiltinTestFunctions.java"
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
    fn gametest_environment_and_instance_decode_vanilla_resources() {
        let environment = parse_test_environment_json(include_str!(
            "../../decompiled-server-26.1.2/data/minecraft/test_environment/default.json"
        ))
        .expect("vanilla default test environment should decode");
        assert_eq!(
            environment,
            TestEnvironmentDefinition::AllOf {
                definitions: Vec::new()
            }
        );

        let instance = parse_test_instance_json(include_str!(
            "../../decompiled-server-26.1.2/data/minecraft/test_instance/always_pass.json"
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
}
