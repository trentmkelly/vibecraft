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

pub const DEFAULT_GAMETEST_ENVIRONMENT_NAME: &str = "default";
pub const DEFAULT_GAMETEST_ENVIRONMENT_KEY: &str = "minecraft:default";

pub fn bootstrap_gametest_environments() -> Vec<(String, TestEnvironmentDefinition)> {
    vec![(
        DEFAULT_GAMETEST_ENVIRONMENT_KEY.to_string(),
        TestEnvironmentDefinition::AllOf {
            definitions: Vec::new(),
        },
    )]
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
pub struct GameTestExceptionBase {
    pub message: String,
}

impl GameTestExceptionBase {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    pub fn runtime_message(&self) -> &str {
        &self.message
    }
}

pub trait GameTestExceptionModel {
    type Description;

    fn base_exception(&self) -> &GameTestExceptionBase;

    fn get_description(&self) -> Self::Description;
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameTestBatchListenerEvent {
    Starting(GameTestBatchModel),
    Finished(GameTestBatchModel),
}

pub trait GameTestBatchListenerModel {
    fn test_batch_starting(&mut self, batch: &GameTestBatchModel);

    fn test_batch_finished(&mut self, batch: &GameTestBatchModel);
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RecordingGameTestBatchListener {
    pub events: Vec<GameTestBatchListenerEvent>,
}

impl GameTestBatchListenerModel for RecordingGameTestBatchListener {
    fn test_batch_starting(&mut self, batch: &GameTestBatchModel) {
        self.events
            .push(GameTestBatchListenerEvent::Starting(batch.clone()));
    }

    fn test_batch_finished(&mut self, batch: &GameTestBatchModel) {
        self.events
            .push(GameTestBatchListenerEvent::Finished(batch.clone()));
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameTestEventModel {
    pub expected_delay: Option<i64>,
    pub minimum_delay: Option<i64>,
    pub assertion: String,
}

pub fn create_gametest_event(assertion: impl Into<String>) -> GameTestEventModel {
    GameTestEventModel {
        expected_delay: None,
        minimum_delay: None,
        assertion: assertion.into(),
    }
}

pub fn create_gametest_event_at(
    expected_tick: i64,
    assertion: impl Into<String>,
) -> GameTestEventModel {
    GameTestEventModel {
        expected_delay: Some(expected_tick),
        minimum_delay: None,
        assertion: assertion.into(),
    }
}

pub fn create_gametest_event_with_minimum_delay(
    minimum_delay: i64,
    assertion: impl Into<String>,
) -> GameTestEventModel {
    GameTestEventModel {
        expected_delay: None,
        minimum_delay: Some(minimum_delay),
        assertion: assertion.into(),
    }
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
mod tests;
