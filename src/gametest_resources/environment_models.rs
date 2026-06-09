use super::TestEnvironmentDefinition;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TestEnvironmentWeatherType {
    Clear,
    Rain,
    Thunder,
}

impl TestEnvironmentWeatherType {
    pub fn serialized_name(self) -> &'static str {
        match self {
            Self::Clear => "clear",
            Self::Rain => "rain",
            Self::Thunder => "thunder",
        }
    }

    pub fn parameters(self) -> (i32, i32, bool, bool) {
        match self {
            Self::Clear => (100000, 0, false, false),
            Self::Rain => (0, 100000, true, false),
            Self::Thunder => (0, 100000, true, true),
        }
    }

    pub fn from_level(raining: bool, thundering: bool) -> Self {
        if thundering {
            Self::Thunder
        } else if raining {
            Self::Rain
        } else {
            Self::Clear
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TestEnvironmentLevelModel {
    pub clock_ticks: Vec<(String, i64)>,
    pub game_rules: Vec<(String, String)>,
    pub environment_attributes: Vec<String>,
    pub raining: bool,
    pub thundering: bool,
    pub weather_events: Vec<(i32, i32, bool, bool)>,
    pub functions: Vec<String>,
    pub executed_functions: Vec<String>,
    pub missing_function_logs: Vec<String>,
    pub teardown_log: Vec<String>,
}

impl TestEnvironmentLevelModel {
    pub fn clock_ticks(&self, clock: &str) -> i64 {
        self.clock_ticks
            .iter()
            .find(|(id, _)| id == clock)
            .map(|(_, ticks)| *ticks)
            .unwrap_or_default()
    }

    pub fn set_clock_ticks(&mut self, clock: impl Into<String>, ticks: i64) {
        set_pair(&mut self.clock_ticks, clock.into(), ticks);
    }

    pub fn set_game_rule(&mut self, rule: impl Into<String>, value: impl Into<String>) {
        set_pair(&mut self.game_rules, rule.into(), value.into());
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TestEnvironmentActivationModel {
    AllOf(Vec<TestEnvironmentActivationModel>),
    ClockTime { clock: String, previous: i64 },
    Functions,
    GameRules { previous: Vec<(String, String)> },
    Timelines { previous: Vec<String> },
    Weather(TestEnvironmentWeatherType),
}

impl TestEnvironmentDefinition {
    pub fn codec_name(&self) -> &'static str {
        match self {
            Self::AllOf { .. } => "all_of",
            Self::GameRules => "game_rules",
            Self::ClockTime { .. } => "clock_time",
            Self::TimelineAttributes { .. } => "timeline_attributes",
            Self::Weather { .. } => "weather",
            Self::Function { .. } => "function",
        }
    }

    pub fn setup(&self, level: &mut TestEnvironmentLevelModel) -> TestEnvironmentActivationModel {
        match self {
            Self::AllOf { definitions } => TestEnvironmentActivationModel::AllOf(
                definitions
                    .iter()
                    .map(|id| {
                        level.teardown_log.push(format!("setup:{id}"));
                        TestEnvironmentActivationModel::Functions
                    })
                    .collect(),
            ),
            Self::GameRules => TestEnvironmentActivationModel::GameRules {
                previous: level.game_rules.clone(),
            },
            Self::ClockTime { clock, time } => {
                let previous = level.clock_ticks(clock);
                level.set_clock_ticks(clock.clone(), i64::from(*time));
                TestEnvironmentActivationModel::ClockTime {
                    clock: clock.clone(),
                    previous,
                }
            }
            Self::TimelineAttributes { timelines } => {
                let previous = level.environment_attributes.clone();
                level.environment_attributes = timelines.clone();
                TestEnvironmentActivationModel::Timelines { previous }
            }
            Self::Weather { weather } => {
                let previous =
                    TestEnvironmentWeatherType::from_level(level.raining, level.thundering);
                let parsed = match weather.as_str() {
                    "clear" => TestEnvironmentWeatherType::Clear,
                    "rain" => TestEnvironmentWeatherType::Rain,
                    "thunder" => TestEnvironmentWeatherType::Thunder,
                    _ => TestEnvironmentWeatherType::Clear,
                };
                let (clear_time, rain_time, raining, thundering) = parsed.parameters();
                level.raining = raining;
                level.thundering = thundering;
                level
                    .weather_events
                    .push((clear_time, rain_time, raining, thundering));
                TestEnvironmentActivationModel::Weather(previous)
            }
            Self::Function { setup, .. } => {
                run_test_environment_function(level, setup.as_deref());
                TestEnvironmentActivationModel::Functions
            }
        }
    }

    pub fn teardown(
        &self,
        level: &mut TestEnvironmentLevelModel,
        activation: TestEnvironmentActivationModel,
    ) {
        match (self, activation) {
            (Self::AllOf { definitions }, TestEnvironmentActivationModel::AllOf(_)) => {
                for id in definitions.iter().rev() {
                    level.teardown_log.push(format!("teardown:{id}"));
                }
            }
            (
                Self::ClockTime { .. },
                TestEnvironmentActivationModel::ClockTime { clock, previous },
            ) => {
                level.set_clock_ticks(clock, previous);
            }
            (Self::Function { teardown, .. }, TestEnvironmentActivationModel::Functions) => {
                run_test_environment_function(level, teardown.as_deref());
            }
            (Self::GameRules, TestEnvironmentActivationModel::GameRules { previous }) => {
                level.game_rules = previous;
            }
            (
                Self::TimelineAttributes { .. },
                TestEnvironmentActivationModel::Timelines { previous },
            ) => {
                level.environment_attributes = previous;
            }
            (Self::Weather { .. }, TestEnvironmentActivationModel::Weather(previous)) => {
                let (clear_time, rain_time, raining, thundering) = previous.parameters();
                level.raining = raining;
                level.thundering = thundering;
                level
                    .weather_events
                    .push((clear_time, rain_time, raining, thundering));
            }
            _ => {}
        }
    }
}

fn run_test_environment_function(level: &mut TestEnvironmentLevelModel, id: Option<&str>) {
    if let Some(id) = id {
        if level.functions.iter().any(|function| function == id) {
            level.executed_functions.push(id.to_string());
        } else {
            level
                .missing_function_logs
                .push(format!("Test Batch failed for non-existent function {id}"));
        }
    }
}

fn set_pair<T>(pairs: &mut Vec<(String, T)>, key: String, value: T) {
    if let Some((_, existing)) = pairs.iter_mut().find(|(id, _)| *id == key) {
        *existing = value;
    } else {
        pairs.push((key, value));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_ENVIRONMENT_DEFINITION_JAVA: &str = include_str!(
        "../../../decompiled-server-26.1.2/net/minecraft/gametest/framework/TestEnvironmentDefinition.java"
    );

    #[test]
    fn test_environment_definition_matches_java_source_shape() {
        assert_eq!(TEST_ENVIRONMENT_DEFINITION_JAVA.lines().count(), 284);
        for sentinel in [
            "Codec<TestEnvironmentDefinition<?>> DIRECT_CODEC = BuiltInRegistries.TEST_ENVIRONMENT_DEFINITION_TYPE",
            "Codec<Holder<TestEnvironmentDefinition<?>>> CODEC = RegistryFileCodec.create(Registries.TEST_ENVIRONMENT, DIRECT_CODEC);",
            "Registry.register(registry, \"all_of\", TestEnvironmentDefinition.AllOf.CODEC);",
            "Registry.register(registry, \"game_rules\", TestEnvironmentDefinition.SetGameRules.CODEC);",
            "Registry.register(registry, \"clock_time\", TestEnvironmentDefinition.ClockTime.CODEC);",
            "Registry.register(registry, \"timeline_attributes\", TestEnvironmentDefinition.Timelines.CODEC);",
            "Registry.register(registry, \"weather\", TestEnvironmentDefinition.Weather.CODEC);",
            "return Registry.register(registry, \"function\", TestEnvironmentDefinition.Functions.CODEC);",
            "static <T> TestEnvironmentDefinition.Activation<T> activate",
            "this.definition.teardown(this.level, this.value);",
            "activations.reversed().forEach(TestEnvironmentDefinition.Activation::teardown);",
            "server.clockManager().getTotalTicks(this.clock)",
            "server.clockManager().setTotalTicks(this.clock, this.time);",
            "Identifier.CODEC.optionalFieldOf(\"setup\")",
            "Identifier.CODEC.optionalFieldOf(\"teardown\")",
            "LOGGER.error(\"Test Batch failed for non-existent function {}\", functionId);",
            "GameRuleMap.CODEC.fieldOf(\"rules\")",
            "gameRules.setAll(this.gameRulesMap, level.getServer());",
            "builder.addTimelineLayer(timeline, level.clockManager());",
            "level.setEnvironmentAttributes(saveData);",
            "TestEnvironmentDefinition.Weather.Type.CODEC.fieldOf(\"weather\")",
            "level.resetWeatherCycle();",
            "CLEAR(\"clear\", 100000, 0, false, false)",
            "RAIN(\"rain\", 0, 100000, true, false)",
            "THUNDER(\"thunder\", 0, 100000, true, true)",
            "level.getServer().setWeatherParameters(this.clearTime, this.rainTime, this.raining, this.thundering);",
        ] {
            assert!(
                TEST_ENVIRONMENT_DEFINITION_JAVA.contains(sentinel),
                "missing TestEnvironmentDefinition sentinel {sentinel}"
            );
        }
    }

    #[test]
    fn environment_weather_types_match_java_names_and_parameters() {
        assert_eq!(TestEnvironmentWeatherType::Clear.serialized_name(), "clear");
        assert_eq!(TestEnvironmentWeatherType::Rain.serialized_name(), "rain");
        assert_eq!(
            TestEnvironmentWeatherType::Thunder.serialized_name(),
            "thunder"
        );
        assert_eq!(
            TestEnvironmentWeatherType::Clear.parameters(),
            (100000, 0, false, false)
        );
        assert_eq!(
            TestEnvironmentWeatherType::Rain.parameters(),
            (0, 100000, true, false)
        );
        assert_eq!(
            TestEnvironmentWeatherType::Thunder.parameters(),
            (0, 100000, true, true)
        );
        assert_eq!(
            TestEnvironmentWeatherType::from_level(false, false),
            TestEnvironmentWeatherType::Clear
        );
        assert_eq!(
            TestEnvironmentWeatherType::from_level(true, false),
            TestEnvironmentWeatherType::Rain
        );
        assert_eq!(
            TestEnvironmentWeatherType::from_level(true, true),
            TestEnvironmentWeatherType::Thunder
        );
    }

    #[test]
    fn environment_setup_and_teardown_restore_clock_rules_timelines_weather_and_functions() {
        let mut level = TestEnvironmentLevelModel {
            clock_ticks: vec![("minecraft:daytime".to_string(), 40)],
            game_rules: vec![("doDaylightCycle".to_string(), "true".to_string())],
            environment_attributes: vec!["default".to_string()],
            raining: true,
            thundering: false,
            functions: vec![
                "minecraft:setup".to_string(),
                "minecraft:teardown".to_string(),
            ],
            ..Default::default()
        };

        let clock = TestEnvironmentDefinition::ClockTime {
            clock: "minecraft:daytime".to_string(),
            time: 1000,
        };
        let clock_activation = clock.setup(&mut level);
        assert_eq!(level.clock_ticks("minecraft:daytime"), 1000);
        clock.teardown(&mut level, clock_activation);
        assert_eq!(level.clock_ticks("minecraft:daytime"), 40);

        let rules = TestEnvironmentDefinition::GameRules;
        let rules_activation = rules.setup(&mut level);
        level.set_game_rule("doDaylightCycle", "false");
        rules.teardown(&mut level, rules_activation);
        assert_eq!(
            level.game_rules,
            vec![("doDaylightCycle".to_string(), "true".to_string())]
        );

        let timelines = TestEnvironmentDefinition::TimelineAttributes {
            timelines: vec!["minecraft:storm".to_string()],
        };
        let timelines_activation = timelines.setup(&mut level);
        assert_eq!(level.environment_attributes, vec!["minecraft:storm"]);
        timelines.teardown(&mut level, timelines_activation);
        assert_eq!(level.environment_attributes, vec!["default"]);

        let weather = TestEnvironmentDefinition::Weather {
            weather: "thunder".to_string(),
        };
        let weather_activation = weather.setup(&mut level);
        assert!(level.raining);
        assert!(level.thundering);
        weather.teardown(&mut level, weather_activation);
        assert!(level.raining);
        assert!(!level.thundering);
        assert_eq!(
            level.weather_events,
            vec![(0, 100000, true, true), (0, 100000, true, false)]
        );

        let functions = TestEnvironmentDefinition::Function {
            setup: Some("minecraft:setup".to_string()),
            teardown: Some("minecraft:teardown".to_string()),
        };
        let function_activation = functions.setup(&mut level);
        functions.teardown(&mut level, function_activation);
        assert_eq!(
            level.executed_functions,
            vec![
                "minecraft:setup".to_string(),
                "minecraft:teardown".to_string()
            ]
        );
    }

    #[test]
    fn environment_all_of_tears_down_in_reverse_and_missing_function_logs() {
        let all_of = TestEnvironmentDefinition::AllOf {
            definitions: vec!["one".to_string(), "two".to_string(), "three".to_string()],
        };
        let mut level = TestEnvironmentLevelModel::default();
        let activation = all_of.setup(&mut level);
        all_of.teardown(&mut level, activation);
        assert_eq!(
            level.teardown_log,
            vec![
                "setup:one",
                "setup:two",
                "setup:three",
                "teardown:three",
                "teardown:two",
                "teardown:one",
            ]
        );

        let function = TestEnvironmentDefinition::Function {
            setup: Some("minecraft:missing".to_string()),
            teardown: None,
        };
        function.setup(&mut level);
        assert_eq!(
            level.missing_function_logs,
            vec!["Test Batch failed for non-existent function minecraft:missing".to_string()]
        );
    }

    #[test]
    fn environment_codec_names_match_bootstrap_registration_ids() {
        assert_eq!(
            TestEnvironmentDefinition::AllOf {
                definitions: Vec::new()
            }
            .codec_name(),
            "all_of"
        );
        assert_eq!(
            TestEnvironmentDefinition::GameRules.codec_name(),
            "game_rules"
        );
        assert_eq!(
            TestEnvironmentDefinition::ClockTime {
                clock: "clock".to_string(),
                time: 1
            }
            .codec_name(),
            "clock_time"
        );
        assert_eq!(
            TestEnvironmentDefinition::TimelineAttributes {
                timelines: Vec::new()
            }
            .codec_name(),
            "timeline_attributes"
        );
        assert_eq!(
            TestEnvironmentDefinition::Weather {
                weather: "clear".to_string()
            }
            .codec_name(),
            "weather"
        );
        assert_eq!(
            TestEnvironmentDefinition::Function {
                setup: None,
                teardown: None
            }
            .codec_name(),
            "function"
        );
    }
}
