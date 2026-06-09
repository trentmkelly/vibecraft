use std::collections::BTreeMap;

use super::RotationModel;

pub const GENERATED_TEST_FUNCTION_REGISTRY: &str = "test_function";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneratedTestFunctionKey {
    pub registry: &'static str,
    pub id: String,
}

impl GeneratedTestFunctionKey {
    pub fn create(function_id: impl Into<String>) -> Self {
        Self {
            registry: GENERATED_TEST_FUNCTION_REGISTRY,
            id: function_id.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneratedTestModel<TestData> {
    pub tests: BTreeMap<String, TestData>,
    pub function_key: GeneratedTestFunctionKey,
    pub function: String,
}

impl<TestData> GeneratedTestModel<TestData> {
    pub fn new(
        tests: BTreeMap<String, TestData>,
        function_key: GeneratedTestFunctionKey,
        function: impl Into<String>,
    ) -> Self {
        Self {
            tests,
            function_key,
            function: function.into(),
        }
    }

    pub fn from_function_id(
        tests: BTreeMap<String, TestData>,
        function_id: impl Into<String>,
        function: impl Into<String>,
    ) -> Self {
        Self::new(
            tests,
            GeneratedTestFunctionKey::create(function_id),
            function,
        )
    }

    pub fn single(id: impl Into<String>, test_data: TestData, function: impl Into<String>) -> Self {
        let id = id.into();
        let mut tests = BTreeMap::new();
        tests.insert(id.clone(), test_data);
        Self::from_function_id(tests, id, function)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameTestDataModel<EnvironmentType> {
    pub environment: EnvironmentType,
    pub structure: String,
    pub max_ticks: i32,
    pub setup_ticks: i32,
    pub required: bool,
    pub rotation: RotationModel,
    pub manual_only: bool,
    pub max_attempts: i32,
    pub required_successes: i32,
    pub sky_access: bool,
    pub padding: i32,
}

impl<EnvironmentType> GameTestDataModel<EnvironmentType> {
    pub fn new(
        environment: EnvironmentType,
        structure: impl Into<String>,
        max_ticks: i32,
        setup_ticks: i32,
        required: bool,
        rotation: RotationModel,
    ) -> Self {
        Self::new_full(
            environment,
            structure,
            max_ticks,
            setup_ticks,
            required,
            rotation,
            false,
            1,
            1,
            false,
            0,
        )
    }

    pub fn new_default_rotation(
        environment: EnvironmentType,
        structure: impl Into<String>,
        max_ticks: i32,
        setup_ticks: i32,
        required: bool,
    ) -> Self {
        Self::new(
            environment,
            structure,
            max_ticks,
            setup_ticks,
            required,
            RotationModel::None,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn new_full(
        environment: EnvironmentType,
        structure: impl Into<String>,
        max_ticks: i32,
        setup_ticks: i32,
        required: bool,
        rotation: RotationModel,
        manual_only: bool,
        max_attempts: i32,
        required_successes: i32,
        sky_access: bool,
        padding: i32,
    ) -> Self {
        Self {
            environment,
            structure: structure.into(),
            max_ticks,
            setup_ticks,
            required,
            rotation,
            manual_only,
            max_attempts,
            required_successes,
            sky_access,
            padding,
        }
    }

    pub fn map_environment<T>(
        self,
        mapper: impl FnOnce(EnvironmentType) -> T,
    ) -> GameTestDataModel<T> {
        GameTestDataModel {
            environment: mapper(self.environment),
            structure: self.structure,
            max_ticks: self.max_ticks,
            setup_ticks: self.setup_ticks,
            required: self.required,
            rotation: self.rotation,
            manual_only: self.manual_only,
            max_attempts: self.max_attempts,
            required_successes: self.required_successes,
            sky_access: self.sky_access,
            padding: self.padding,
        }
    }
}
