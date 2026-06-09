#![allow(dead_code)]

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
                    function: "minecraft:always_pass".to_string()
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
