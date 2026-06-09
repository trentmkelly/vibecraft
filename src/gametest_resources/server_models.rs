use std::collections::BTreeSet;

use crate::core_block_pos::RotationModel;

pub const GAMETEST_SERVER_RUNTIME_TODO: &str = "gametest-server-runtime";
pub const GAMETEST_SERVER_PROGRESS_REPORT_INTERVAL: i32 = 20;
pub const GAMETEST_SERVER_TEST_POSITION_RANGE: i32 = 14_999_992;
pub const GAMETEST_SERVER_WORLD_SEED: i64 = 0;
pub const GAMETEST_SERVER_MAX_PLAYERS: i32 = 1;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameTestServerOptionsModel {
    pub test_selection: Option<String>,
    pub verify: bool,
    pub repeat_count: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameTestServerTestModel {
    pub id: String,
    pub manual_only: bool,
    pub required: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameTestServerRunInfo {
    pub test_id: String,
    pub rotation: RotationModel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameTestServerDecoratorModel {
    Direct,
    Verify,
    Repeat,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameTestServerEvaluationModel {
    pub decorator: GameTestServerDecoratorModel,
    pub tests: Vec<GameTestServerRunInfo>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameTestServerPropertiesModel {
    pub hard_core: bool,
    pub rcon_broadcast: bool,
    pub dedicated_server: bool,
    pub rate_limit_packets_per_second: i32,
    pub native_transport: bool,
    pub published: bool,
    pub inform_admins: bool,
    pub max_players: i32,
    pub tick_time_logging_enabled: bool,
    pub system_report_type: &'static str,
}

pub fn gametest_server_enabled_feature_policy() -> &'static str {
    "all_except_redstone_experiments_and_minecart_improvements"
}

pub fn gametest_server_properties() -> GameTestServerPropertiesModel {
    GameTestServerPropertiesModel {
        hard_core: false,
        rcon_broadcast: false,
        dedicated_server: false,
        rate_limit_packets_per_second: 0,
        native_transport: false,
        published: false,
        inform_admins: false,
        max_players: GAMETEST_SERVER_MAX_PLAYERS,
        tick_time_logging_enabled: false,
        system_report_type: "Game test server",
    }
}

pub fn evaluate_gametest_server_tests(
    options: &GameTestServerOptionsModel,
    registry_tests: &[GameTestServerTestModel],
) -> Result<GameTestServerEvaluationModel, String> {
    let selected = registry_tests
        .iter()
        .filter(|test| !test.manual_only)
        .filter(|test| {
            options
                .test_selection
                .as_deref()
                .map(|selection| selector_matches(selection, &test.id))
                .unwrap_or(true)
        })
        .collect::<Vec<_>>();

    if options.test_selection.is_some() && selected.is_empty() {
        return Err("Test selection matcher found no tests".to_string());
    }

    let (decorator, tests) = if options.test_selection.is_some() && options.verify {
        (
            GameTestServerDecoratorModel::Verify,
            selected
                .iter()
                .flat_map(|test| rotate_and_multiply(&test.id))
                .collect(),
        )
    } else if options.test_selection.is_some() && options.repeat_count > 0 {
        (
            GameTestServerDecoratorModel::Repeat,
            selected
                .iter()
                .flat_map(|test| multiply_test(&test.id, options.repeat_count))
                .collect(),
        )
    } else {
        (
            GameTestServerDecoratorModel::Direct,
            selected
                .iter()
                .map(|test| GameTestServerRunInfo {
                    test_id: test.id.clone(),
                    rotation: RotationModel::None,
                })
                .collect(),
        )
    };

    Ok(GameTestServerEvaluationModel { decorator, tests })
}

pub fn gametest_server_start_pos(random_x: i32, random_z: i32) -> Result<(i32, i32, i32), String> {
    if !(-GAMETEST_SERVER_TEST_POSITION_RANGE..=GAMETEST_SERVER_TEST_POSITION_RANGE)
        .contains(&random_x)
        || !(-GAMETEST_SERVER_TEST_POSITION_RANGE..=GAMETEST_SERVER_TEST_POSITION_RANGE)
            .contains(&random_z)
    {
        return Err("start position outside GameTestServer range".to_string());
    }
    Ok((random_x, -59, random_z))
}

pub fn gametest_server_failed_test_log(
    id: &str,
    rotation: RotationModel,
    description: &str,
) -> String {
    match rotation {
        RotationModel::None => format!("{id}: {description}"),
        other => format!("{id} with rotation {}: {description}", rotation_id(other)),
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MockUserNameToIdResolverModel {
    saved_ids: BTreeSet<(String, String)>,
}

impl MockUserNameToIdResolverModel {
    pub fn add(&mut self, name: impl Into<String>, id: impl Into<String>) {
        self.saved_ids.insert((name.into(), id.into()));
    }

    pub fn get_by_name(&self, name: &str) -> Option<(String, String)> {
        self.saved_ids
            .iter()
            .find(|(saved_name, _)| saved_name == name)
            .cloned()
            .or_else(|| Some((name.to_string(), format!("offline:{name}"))))
    }

    pub fn get_by_id(&self, id: &str) -> Option<(String, String)> {
        self.saved_ids
            .iter()
            .find(|(_, saved_id)| saved_id == id)
            .cloned()
    }

    pub fn resolve_offline_users(&self, _value: bool) {}

    pub fn save(&self) {}
}

pub fn mock_profile_fetch_by_name(_name: &str) -> Option<String> {
    None
}

pub fn mock_profile_fetch_by_id(_id: &str) -> Option<String> {
    None
}

fn rotate_and_multiply(test_id: &str) -> Vec<GameTestServerRunInfo> {
    [
        RotationModel::None,
        RotationModel::Clockwise90,
        RotationModel::Clockwise180,
        RotationModel::Counterclockwise90,
    ]
    .into_iter()
    .flat_map(|rotation| {
        (0..100).map(move |_| GameTestServerRunInfo {
            test_id: test_id.to_string(),
            rotation,
        })
    })
    .collect()
}

fn multiply_test(test_id: &str, repeat_count: i32) -> Vec<GameTestServerRunInfo> {
    (0..repeat_count)
        .map(|_| GameTestServerRunInfo {
            test_id: test_id.to_string(),
            rotation: RotationModel::None,
        })
        .collect()
}

fn selector_matches(selection: &str, id: &str) -> bool {
    if let Some(prefix) = selection.strip_suffix('*') {
        id.starts_with(prefix)
    } else {
        selection == id
    }
}

fn rotation_id(rotation: RotationModel) -> &'static str {
    match rotation {
        RotationModel::None => "none",
        RotationModel::Clockwise90 => "clockwise_90",
        RotationModel::Clockwise180 => "180",
        RotationModel::Counterclockwise90 => "counterclockwise_90",
    }
}
