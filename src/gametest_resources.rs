#![allow(dead_code)]

use crate::block_entity::TestBlockMode;
use std::collections::BTreeSet;

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExhaustedAttemptsError {
    pub message: String,
    pub cause: Option<String>,
}

pub fn exhausted_attempts_error(
    attempts: i32,
    successes: i32,
    required_successes: i32,
    max_attempts: i32,
    cause: Option<String>,
) -> ExhaustedAttemptsError {
    ExhaustedAttemptsError {
        message: format!(
            "Not enough successes: {successes} out of {attempts} attempts. Required successes: {required_successes}. max attempts: {max_attempts}."
        ),
        cause,
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FailedTestTrackerModel {
    last_failed_tests: BTreeSet<String>,
}

impl FailedTestTrackerModel {
    pub fn remember_failed_test(&mut self, test: impl Into<String>) {
        self.last_failed_tests.insert(test.into());
    }

    pub fn forget_failed_tests(&mut self) {
        self.last_failed_tests.clear();
    }

    pub fn last_failed_tests(&self) -> Vec<&str> {
        self.last_failed_tests.iter().map(String::as_str).collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FunctionGameTestRunOutcome {
    Invoked { function: String },
    MissingFunction { message: String },
}

pub fn function_gametest_run_outcome(
    function: &str,
    registered_functions: &[&str],
) -> FunctionGameTestRunOutcome {
    if registered_functions.contains(&function) {
        FunctionGameTestRunOutcome::Invoked {
            function: function.to_string(),
        }
    } else {
        FunctionGameTestRunOutcome::MissingFunction {
            message: format!("Trying to access missing test function: {function}"),
        }
    }
}

pub fn function_gametest_description_rows(function: &str) -> [(&'static str, String); 1] {
    [("test_instance.description.function", function.to_string())]
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameTestAssertError {
    pub message: String,
    pub tick: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameTestAssertDescription {
    pub translation_key: &'static str,
    pub message: String,
    pub tick: i32,
}

impl GameTestAssertError {
    pub fn new(message: impl Into<String>, tick: i32) -> Self {
        Self {
            message: message.into(),
            tick,
        }
    }

    pub fn runtime_message(&self) -> &str {
        &self.message
    }

    pub fn description(&self) -> GameTestAssertDescription {
        GameTestAssertDescription {
            translation_key: "test.error.tick",
            message: self.message.clone(),
            tick: self.tick,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GameTestBlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameTestAssertPosError {
    pub base: GameTestAssertError,
    pub absolute_pos: GameTestBlockPos,
    pub relative_pos: GameTestBlockPos,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameTestAssertPosDescription {
    pub translation_key: &'static str,
    pub message: String,
    pub absolute_pos: GameTestBlockPos,
    pub relative_pos: GameTestBlockPos,
    pub tick: i32,
}

impl GameTestAssertPosError {
    pub fn new(
        message: impl Into<String>,
        absolute_pos: GameTestBlockPos,
        relative_pos: GameTestBlockPos,
        tick: i32,
    ) -> Self {
        Self {
            base: GameTestAssertError::new(message, tick),
            absolute_pos,
            relative_pos,
        }
    }

    pub fn description(&self) -> GameTestAssertPosDescription {
        GameTestAssertPosDescription {
            translation_key: "test.error.position",
            message: self.base.message.clone(),
            absolute_pos: self.absolute_pos,
            relative_pos: self.relative_pos,
            tick: self.base.tick,
        }
    }

    pub fn message_to_show_at_block(&self) -> &str {
        &self.base.message
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameTestBatchModel {
    pub index: i32,
    pub game_test_infos: Vec<String>,
    pub environment: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameTestInfoModel {
    pub test: String,
    pub environment: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecoratedGameTestInfoModel {
    pub info: GameTestInfoModel,
    pub rotation: &'static str,
    pub retry_options: &'static str,
}

pub const MAX_GAMETESTS_PER_BATCH: usize = 50;

pub fn create_gametest_batch(
    index: i32,
    game_test_infos: Vec<String>,
    environment: impl Into<String>,
) -> Result<GameTestBatchModel, String> {
    if game_test_infos.is_empty() {
        return Err("A GameTestBatch must include at least one GameTestInfo!".to_string());
    }
    Ok(GameTestBatchModel {
        index,
        game_test_infos,
        environment: environment.into(),
    })
}

pub fn direct_gametest_decorator(
    test: impl Into<String>,
    environment: impl Into<String>,
) -> DecoratedGameTestInfoModel {
    DecoratedGameTestInfoModel {
        info: GameTestInfoModel {
            test: test.into(),
            environment: environment.into(),
        },
        rotation: "NONE",
        retry_options: "noRetries",
    }
}

pub fn gametest_batches_from_infos(
    game_test_infos: &[Option<GameTestInfoModel>],
    max_tests_per_batch: usize,
) -> Result<Vec<GameTestBatchModel>, String> {
    if max_tests_per_batch == 0 {
        return Err("partition size must be positive".to_string());
    }

    let mut grouped: Vec<(String, Vec<String>)> = Vec::new();
    for info in game_test_infos.iter().flatten() {
        match grouped
            .iter_mut()
            .find(|(environment, _)| environment == &info.environment)
        {
            Some((_, tests)) => tests.push(info.test.clone()),
            None => grouped.push((info.environment.clone(), vec![info.test.clone()])),
        }
    }

    grouped
        .into_iter()
        .flat_map(|(environment, tests)| {
            tests
                .chunks(max_tests_per_batch)
                .enumerate()
                .map(move |(index, chunk)| {
                    create_gametest_batch(index as i32, chunk.to_vec(), environment.clone())
                })
                .collect::<Vec<_>>()
        })
        .collect()
}

pub fn divide_decorated_gametests_into_batches(
    all_tests: &[DecoratedGameTestInfoModel],
) -> Result<Vec<GameTestBatchModel>, String> {
    let infos = all_tests
        .iter()
        .map(|decorated| Some(decorated.info.clone()))
        .collect::<Vec<_>>();
    gametest_batches_from_infos(&infos, MAX_GAMETESTS_PER_BATCH)
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
    const EXHAUSTED_ATTEMPTS_EXCEPTION_JAVA: &str = include_str!(
        "../../decompiled-server-26.1.2/net/minecraft/gametest/framework/ExhaustedAttemptsException.java"
    );
    const FAILED_TEST_TRACKER_JAVA: &str = include_str!(
        "../../decompiled-server-26.1.2/net/minecraft/gametest/framework/FailedTestTracker.java"
    );
    const FUNCTION_GAME_TEST_INSTANCE_JAVA: &str = include_str!(
        "../../decompiled-server-26.1.2/net/minecraft/gametest/framework/FunctionGameTestInstance.java"
    );
    const GAME_TEST_ASSERT_EXCEPTION_JAVA: &str = include_str!(
        "../../decompiled-server-26.1.2/net/minecraft/gametest/framework/GameTestAssertException.java"
    );
    const GAME_TEST_ASSERT_POS_EXCEPTION_JAVA: &str = include_str!(
        "../../decompiled-server-26.1.2/net/minecraft/gametest/framework/GameTestAssertPosException.java"
    );
    const GAME_TEST_BATCH_JAVA: &str = include_str!(
        "../../decompiled-server-26.1.2/net/minecraft/gametest/framework/GameTestBatch.java"
    );
    const GAME_TEST_BATCH_FACTORY_JAVA: &str = include_str!(
        "../../decompiled-server-26.1.2/net/minecraft/gametest/framework/GameTestBatchFactory.java"
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
                message: "Not enough successes: 2 out of 5 attempts. Required successes: 3. max attempts: 7.".to_string(),
                cause: Some("last failure".to_string()),
            }
        );
        assert_eq!(
            exhausted_attempts_error(1, 0, 1, 1, None),
            ExhaustedAttemptsError {
                message: "Not enough successes: 0 out of 1 attempts. Required successes: 1. max attempts: 1.".to_string(),
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
